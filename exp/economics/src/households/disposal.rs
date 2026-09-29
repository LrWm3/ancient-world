//! Explicit, dated cash sales during solvent wind-down. No auction or forced sale.
use super::*;
use crate::{asset_exchange, credit, finance, opportunities};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sale {
    pub month: u32,
    pub asset: AssetId,
    pub buyer: AgentId,
    pub price: Amount,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    Unavailable,
    Encumbered,
    Attached,
    Permission,
    FundingOrStorage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub household: AgentId,
    pub sale: Sale,
    pub rejection: Option<Rejection>,
    /// Opening condition, present for durable equipment. Replay checks it; the
    /// transfer preserves wear and attachment state rather than creating a tool.
    pub equipment: Option<crate::equipment::DurableAsset>,
}

/// Supply mutually accepted sale terms at Open. The last member authorizes the
/// household side; the named buyer's consent is an explicit scenario input.
/// Funds and permissions are checked again at execution. Failed sales expire.
pub fn accept(
    world: &mut World,
    state: &State,
    household: AgentId,
    person: AgentId,
    sale: Sale,
) -> Result<(), String> {
    let a = world
        .households
        .iter()
        .find(|a| a.agent == household)
        .ok_or("unknown household")?;
    if state.phase != Phase::Open
        || sale.month != state.month
        || state.pending_production.is_some()
        || dissolution::winding_at(a, state.month).is_none_or(|(_, p, _)| p != person)
        || rejection(world, state, a, &sale).is_some()
    {
        return Err("asset disposal requires a living last member, wind-down and transferable property at Open".into());
    }
    let mut next = world.clone();
    next.households
        .iter_mut()
        .find(|a| a.agent == household)
        .unwrap()
        .asset_sales
        .push(sale);
    crate::settlement::validate_world(&next, state)?;
    *world = next;
    Ok(())
}

pub(super) fn validate(world: &World, state: &State) -> Result<(), String> {
    for a in &world.households {
        let mut dated = BTreeSet::new();
        for s in &a.asset_sales {
            if !dated.insert((s.month, s.asset))
                || dissolution::winding_at(a, s.month).is_none()
                || dissolution::closed_at(a, s.month)
                || s.buyer == a.agent
                || s.price.quantity <= 0
                || !world.agents.iter().any(|b| b.id == s.buyer)
                || (!world.assets.iter().any(|b| b.id == s.asset)
                    && !state.equipment.contains_key(&s.asset))
                || !world
                    .resources
                    .iter()
                    .any(|r| r.id == s.price.resource && r.kind == ResourceKind::Stock)
                || world.activities.perishable.contains(&s.price.resource)
            {
                return Err("invalid dated household asset sale".into());
            }
        }
    }
    Ok(())
}

fn rejection(world: &World, state: &State, a: &Agreement, sale: &Sale) -> Option<Rejection> {
    let person = dissolution::winding_at(a, state.month).map(|(_, p, _)| p);
    if person.is_none_or(|p| state.terminal.contains_key(&p))
        || dissolution::closed_at(a, state.month)
        || state.terminal.contains_key(&sale.buyer)
        || world
            .households
            .iter()
            .any(|h| h.agent == sale.buyer && dissolution::winding_at(h, state.month).is_some())
        || asset_exchange::owner(world, state, &state.credit, sale.asset) != Some(a.agent)
    {
        return Some(Rejection::Unavailable);
    }
    if let Some(tool) = state.equipment.get(&sale.asset) {
        if tool.attached_to.is_some() {
            return Some(Rejection::Attached);
        }
        if !crate::equipment::transferable(tool, state.month) {
            return Some(Rejection::Unavailable);
        }
    }
    if state.exchange.contracts.contains_key(&sale.asset)
        || world
            .offers
            .iter()
            .any(|o| o.asset == sale.asset && !state.filled_offers.contains(&o.id))
    {
        return Some(Rejection::Encumbered);
    }
    if state.credit.loans.values().any(|l| {
        l.collateral
            .as_ref()
            .is_some_and(|c| c.asset == sale.asset && c.pledged)
    }) || world.lending.iter().any(|l| {
        l.month >= state.month && l.collateral.as_ref().is_some_and(|c| c.asset == sale.asset)
    }) || world
        .credit
        .as_ref()
        .is_some_and(|c| c.offers.iter().any(|o| o.sale.asset == sale.asset))
        || world
            .recovery
            .proceedings
            .iter()
            .any(|p| p.assets.iter().any(|l| l.asset == sale.asset))
    {
        return Some(Rejection::Encumbered);
    }
    if state
        .equipment
        .values()
        .any(|tool| tool.attached_to == Some(sale.asset))
        || world
            .rights
            .iter()
            .any(|r| r.asset == sale.asset && r.through >= state.month)
        || state
            .processes
            .values()
            .any(|p| p.asset == Some(sale.asset) && p.status == Status::Active)
    {
        return Some(Rejection::Attached);
    }
    if [a.agent, sale.buyer]
        .into_iter()
        .any(|who| !opportunities::permits(world, state, who, opportunities::Action::AssetTrade))
    {
        return Some(Rejection::Permission);
    }
    None
}

pub(super) fn prepare(world: &World, state: &State) -> Result<(Vec<Receipt>, Vec<Effect>), String> {
    let mut sales: Vec<_> = world
        .households
        .iter()
        .flat_map(|a| {
            a.asset_sales
                .iter()
                .filter(|s| s.month == state.month)
                .map(move |s| (a, s))
        })
        .collect();
    // Explicit stable priority for accepted sales sharing the buyer's opening
    // budget. Receipts never replenish this boundary's spending allowance.
    sales.sort_by_key(|(a, s)| (a.agent, s.asset));
    let mut execution = finance::Execution::opening(world, state);
    let mut receipts = vec![];
    let mut effects = vec![];
    for (a, sale) in sales {
        let mut reason = rejection(world, state, a, sale);
        if reason.is_none() {
            match asset_exchange::fund(
                world,
                state,
                &state.credit,
                &mut execution,
                &asset_exchange::AcceptedSale {
                    sale: credit::Sale {
                        asset: sale.asset,
                        seller: a.agent,
                        price: sale.price.clone(),
                    },
                    buyer: sale.buyer,
                    payees: vec![(a.agent, sale.price.quantity)],
                },
            ) {
                Ok(payment) => effects.extend(payment),
                Err(_) => reason = Some(Rejection::FundingOrStorage),
            }
        }
        receipts.push(Receipt {
            household: a.agent,
            sale: sale.clone(),
            rejection: reason,
            equipment: state.equipment.get(&sale.asset).cloned(),
        });
    }
    Ok((receipts, effects))
}

pub(super) fn publish(state: &mut State, receipts: &[Receipt]) {
    for r in receipts.iter().filter(|r| r.rejection.is_none()) {
        if r.equipment.is_some() {
            state
                .equipment
                .get_mut(&r.sale.asset)
                .expect("verified durable disposal")
                .owner = r.sale.buyer;
        } else {
            state.credit.owners.insert(r.sale.asset, r.sale.buyer);
            state
                .credit
                .values
                .insert(r.sale.asset, r.sale.price.quantity);
        }
    }
}
