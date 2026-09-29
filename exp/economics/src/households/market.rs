//! Collective consumption orders using existing member allocation and private stocks.
use super::*;
use crate::{
    household_governance::{self, Policy},
    need_orders,
};

pub const EXAMPLE_HOUSEHOLD: AgentId = 10_000;

pub(crate) fn validate(world: &World, traders: &BTreeSet<AgentId>) -> Result<(), String> {
    if !world.households.is_empty() && world.production_market.is_some() {
        return Err("household town orders do not compose with joint production planning".into());
    }
    for h in &world.households {
        if traders.iter().any(|id| membership::ever_member(h, *id)) {
            return Err("household members use their collective town account in this pilot".into());
        }
    }
    Ok(())
}

pub(crate) fn active(world: &World, state: &State, agent: AgentId) -> bool {
    world
        .households
        .iter()
        .find(|h| h.agent == agent)
        .is_none_or(|h| {
            members(h, state).next().is_some()
                && household_governance::authority(h, state).leader.is_some()
        })
}

/// NeedsFirst authorizes collective consumption purchases. Other objectives only
/// offer protected surplus in this bounded adapter; no speculative value is inferred.
pub(crate) fn buys(world: &World, state: &State, agent: AgentId) -> bool {
    world
        .households
        .iter()
        .find(|h| h.agent == agent)
        .is_none_or(|h| h.governance.policy(state.month) == Policy::NeedsFirst)
}

/// Freeze current membership, permissions and private holdings over the order
/// horizon. Project only consumption: no assumed harvests, market fills or labor.
/// Private resources reduce demand without entering the collective sale budget.
pub(crate) fn consume(
    world: &World,
    state: &State,
    h: &Agreement,
    months: u32,
    stocks: &mut BTreeMap<ResourceId, i128>,
    protect_partial: bool,
) -> Result<BTreeMap<ResourceId, i64>, String> {
    let people: BTreeSet<_> = members(h, state).collect();
    let mut preview = state.clone();
    preview.phase = Phase::Consumption;
    let mut w = world.clone();
    // Need value and physical settlement feasibility are separate. The matcher
    // checks real storage; an unaffordable/unstorable useful order retains a receipt.
    w.storage.capacities.clear();
    for p in &mut w.participants {
        if !people.contains(&p.agent) {
            p.needs.clear();
        }
    }
    for member in &people {
        for (r, q) in need_orders::claims(world, state, *member, months)? {
            let held = preview.balance(*member, r);
            preview
                .balances
                .insert((*member, r), (i128::from(held) - q).max(0) as i32);
        }
    }
    let mut deficits = BTreeMap::<ResourceId, i64>::new();
    for month in 0..months {
        for r in w.resources.iter().filter(|r| r.kind == ResourceKind::Stock) {
            preview.balances.insert(
                (h.agent, r.id),
                i32::try_from(stocks.get(&r.id).copied().unwrap_or(0))
                    .map_err(|_| "household order stock overflow")?,
            );
        }
        if month > 0 {
            for member in &people {
                for r in w
                    .resources
                    .iter()
                    .filter(|r| r.kind == ResourceKind::Fulfillment)
                {
                    preview.balances.insert((*member, r.id), 0);
                }
            }
        }
        let (_, effects) = allocate(&w, &preview, requests(&w, &preview)?)?;
        apply(&w, &mut preview, &effects, Backend::Reference)?;
        for member in &people {
            let mut private = crate::substitution::stocks(&preview, *member);
            let unmet = need_orders::consume_person(
                &w,
                &preview,
                *member,
                1,
                &mut private,
                protect_partial,
            );
            for (resource, q) in &unmet {
                *deficits.entry(*resource).or_default() += q;
                if protect_partial && *q > 0 {
                    for d in crate::substitution::recipes_for(&w, &preview, *member, *resource) {
                        let input = &d.stages[0].entry_inputs[0];
                        let held = preview.balance(h.agent, input.resource);
                        let lots = (i128::from(*q) + i128::from(d.outputs[0].quantity) - 1)
                            / i128::from(d.outputs[0].quantity);
                        let retained = i128::from(held).min(lots * i128::from(input.quantity));
                        preview
                            .balances
                            .insert((h.agent, input.resource), held - retained as i32);
                    }
                }
            }
            for (r, q) in private {
                preview.balances.insert(
                    (*member, r),
                    i32::try_from(q).map_err(|_| "household private stock overflow")?,
                );
            }
        }
        for r in w.resources.iter().filter(|r| r.kind == ResourceKind::Stock) {
            stocks.insert(r.id, i128::from(preview.balance(h.agent, r.id)));
        }
    }
    Ok(deficits)
}

/// Two adults share one buyer account; two independent persons offer food.
pub fn scenario() -> Result<(World, State), String> {
    use crate::{
        marketplace::Side,
        opportunities::{Action, HOUSEHOLD_TYPE, PERSON_TYPE},
        scenario::TOKEN,
    };
    let (mut w, mut s) = crate::town_market::scenario();
    let mut c = w.town_market.take().unwrap();
    let book = std::mem::take(&mut s.town_market);
    let mut buyers: Vec<_> = c
        .traders
        .iter()
        .filter(|e| e.side == Side::Buy)
        .map(|e| e.trader.agent)
        .collect();
    buyers.sort();
    let leader = buyers[0];
    let mut collective = c
        .traders
        .iter()
        .find(|e| e.trader.agent == leader)
        .unwrap()
        .clone();
    collective.trader.agent = EXAMPLE_HOUSEHOLD;
    c.traders.retain(|e| !buyers.contains(&e.trader.agent));
    c.traders.push(collective);
    let law = w.transaction_policy.as_mut().unwrap();
    law.permissions
        .insert((PERSON_TYPE, Action::FoundHousehold));
    law.permissions.insert((HOUSEHOLD_TYPE, Action::StockTrade));
    w.marketplaces
        .iter_mut()
        .find(|m| m.agent == c.venue)
        .unwrap()
        .allowed_types
        .insert(HOUSEHOLD_TYPE);
    let mut governance = household_governance::Governance::contributed(leader);
    governance.charter.initial_policy = Policy::NeedsFirst;
    let cash = s.balance(leader, TOKEN);
    for id in &buyers {
        s.balances.insert((*id, TOKEN), 0);
    }
    form(
        &mut w,
        &s,
        Agreement {
            id: 1,
            agent: EXAMPLE_HOUSEHOLD,
            adults: buyers,
            membership: vec![],
            asset_sales: vec![],
            equipment_retirements: vec![],
            formed: s.month,
            dwelling_process: None,
            admission: None,
            governance,
        },
    )?;
    s.balances.insert((EXAMPLE_HOUSEHOLD, TOKEN), cash);
    s.town_market = book;
    s.town_market
        .positions
        .insert(EXAMPLE_HOUSEHOLD, s.town_market.positions[&c.town]);
    w.town_market = Some(c);
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}
