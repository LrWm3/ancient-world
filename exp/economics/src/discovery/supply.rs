//! Conservative opportunity-cost checks for discovered whole-lot supplier portfolios.
use super::*;
use crate::marketplace::{MarketId, Side};

const MAX_SUPPLY_LOTS: i32 = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alternative {
    pub lots: i32,
    /// Conditional proceeds, never credited to the forecast or opening budget.
    pub proceeds: i128,
    pub losses: Option<Vec<i128>>,
    pub failure: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub month: u32,
    pub agent: AgentId,
    pub market: MarketId,
    pub resource: ResourceId,
    pub available: i32,
    pub protected: i32,
    pub selected_lots: i32,
    pub baseline: Vec<i128>,
    pub alternatives: Vec<Alternative>,
}

fn objectives(w: &World, s: &State, agent: AgentId) -> Vec<agency::objectives::Objective> {
    // A supplier cannot treat harm to other members as free collective labor.
    let members: Vec<_> = w
        .households
        .iter()
        .find(|h| h.agent == agent)
        .or_else(|| {
            crate::households::parent(w, s, agent)
                .and_then(|id| w.households.iter().find(|h| h.agent == id))
        })
        .map(|h| crate::households::members(h, s).collect())
        .unwrap_or_else(|| vec![agent]);
    let mut goals: Vec<_> = members.iter().flat_map(|a| needs(w, *a)).collect();
    goals.push(agency::objectives::Objective {
        scope: agency::objectives::Scope::Agents(members.into_iter().collect()),
        metric: agency::objectives::Metric::FailedProcesses,
    });
    goals
}

/// Shared reserve calculation for people, passive owners and households.
pub(super) fn protected(
    w: &World,
    s: &State,
    agent: AgentId,
    horizon: u32,
) -> Result<BTreeMap<ResourceId, i128>, String> {
    if w.participants.iter().any(|p| p.agent == agent)
        || w.households.iter().any(|h| h.agent == agent)
    {
        crate::need_orders::protected_stock(w, s, agent, horizon)
    } else {
        crate::need_orders::claims(w, s, agent, horizon)
    }
}

fn project(
    opening: &Simulation,
    agent: AgentId,
    quantities: &BTreeMap<ResourceId, i32>,
    horizon: u32,
    goals: &[agency::objectives::Objective],
) -> Result<Vec<i128>, String> {
    let mut sim = opening.clone();
    for (&resource, &quantity) in quantities {
        let held = sim.state.balances.entry((agent, resource)).or_default();
        *held = held
            .checked_sub(quantity)
            .filter(|n| *n >= 0)
            .ok_or("supplier forecast overspending")?;
    }
    // Debits are isolated hypotheses about opportunity cost. Proceeds and another
    // party's goods are not invented; actual trades use ordinary settlement.
    sim.run_months(horizon)?;
    loss(&sim, agent, goals)
}

pub(super) fn choose(
    w: &World,
    s: &State,
    c: &Config,
    quotes: &mut [crate::minting::orders::Quote],
) -> Result<Vec<Decision>, String> {
    let Some(mint) = &w.minting else {
        return Ok(vec![]);
    };
    let venue = crate::marketplace::venue(w, mint.venue).ok_or("missing supply venue")?;
    let (mut observed, state) = crate::forecast::ForecastContext::new(w, s).into_parts();
    // No supplied ask can sell these inputs a second time inside the forecast.
    observed
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes = quotes
        .iter()
        .filter(|q| q.side == Side::Buy)
        .cloned()
        .collect();
    // This month's opening endowment is the boundary being decided, not a
    // guessed future shock. Future overrides remain stripped by ForecastContext.
    for (&key, &quantity) in &w.capacity_overrides {
        if key.0 == s.month {
            observed.capacity_overrides.insert(key, quantity);
        }
    }
    let mut opening = Simulation::new(observed, state, Backend::Reference)?;
    opening.step()?; // normal Open regeneration, before any Acquire sales
    if opening.state.phase == Phase::Due {
        opening.step()?;
    }
    if opening.state.phase != Phase::Acquire {
        return Err("supplier discovery requires Acquire after Open".into());
    }
    let mut decisions = vec![];
    let sellers: BTreeSet<_> = quotes
        .iter()
        .filter(|q| q.side == Side::Sell)
        .map(|q| q.agent)
        .collect();
    for agent in sellers {
        if !quotes
            .iter()
            .any(|q| q.agent == agent && q.side == Side::Sell)
        {
            continue;
        }
        let goals = objectives(&opening.world, &opening.state, agent);
        let baseline = project(&opening, agent, &BTreeMap::new(), c.horizon, &goals)?;
        let floors = protected(&opening.world, &opening.state, agent, c.horizon)?;
        let mut portfolio = BTreeMap::new();
        for quote in quotes
            .iter_mut()
            .filter(|q| q.agent == agent && q.side == Side::Sell)
        {
            let market = venue.markets.iter().find(|m| m.id == quote.market).unwrap();
            let resource = market.goods.resource;
            let available = (opening.state.balance(agent, resource)
                - crate::households::labor_reserve(
                    &opening.world,
                    &opening.state,
                    agent,
                    resource,
                ))
            .max(0);
            let protected = floors
                .get(&resource)
                .copied()
                .unwrap_or(0)
                .min(i128::from(available)) as i32;
            let maximum = ((available - protected) / market.goods.quantity).min(MAX_SUPPLY_LOTS);
            let mut decision = Decision {
                month: s.month,
                agent,
                market: market.id,
                resource,
                available,
                protected,
                selected_lots: 0,
                baseline: baseline.clone(),
                alternatives: vec![Alternative {
                    lots: 0,
                    proceeds: 0,
                    losses: Some(project(&opening, agent, &portfolio, c.horizon, &goals)?),
                    failure: None,
                }],
            };
            // Highest potential proceeds first; all protected outcome dimensions
            // must remain no worse than declining the entire supply portfolio.
            for lots in (1..=maximum).rev() {
                let mut candidate = portfolio.clone();
                candidate.insert(resource, lots * market.goods.quantity);
                let result = project(&opening, agent, &candidate, c.horizon, &goals);
                let acceptable = result
                    .as_ref()
                    .is_ok_and(|losses| losses.iter().zip(&baseline).all(|(a, b)| a <= b));
                let (losses, failure) = match result {
                    Ok(v) => (Some(v), None),
                    Err(e) => (None, Some(e)),
                };
                decision.alternatives.push(Alternative {
                    lots,
                    proceeds: i128::from(lots) * i128::from(quote.limit),
                    losses,
                    failure,
                });
                if acceptable {
                    decision.selected_lots = lots;
                    portfolio = candidate;
                    break;
                }
            }
            // Household consumption may be distributed before the market boundary.
            // Freeze quantity authorization, then recheck its live reserve at clearing.
            quote.holding = if w.households.iter().any(|h| h.agent == agent) {
                0
            } else {
                available - decision.selected_lots * market.goods.quantity
            };
            quote.max_lots = Some(decision.selected_lots);
            decisions.push(decision);
        }
    }
    Ok(decisions)
}
