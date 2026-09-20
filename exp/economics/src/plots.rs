//! Bounded requests for additional taxed productive sites; no occupation branches.
use crate::{commitments, compute::Backend, model::*, simulation::Simulation};

pub const FORECAST_MONTHS: u32 = commitments::MONTHS_PER_YEAR + 1;
pub const FIRST_REVIEW_MONTH: u32 = commitments::MONTHS_PER_YEAR + 1;
pub const EXTRA_PLOTS: u32 = 8;
pub const MAX_EXTRA_PER_AGENT: usize = 2;
const EXTRA_ASSET_BASE: u32 = 50_000;
const EXTRA_RIGHT_BASE: u32 = 60_000;
const EXTRA_AGREEMENT_BASE: u32 = 70_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub offers: Vec<u32>,
    pub output: ResourceId,
    pub first_review: u32,
    pub max_extra: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    Accepted,
    NoVacancy,
    Limit,
    ExistingDebt,
    InsufficientProductivity,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Forecast {
    pub closing_stock: i32,
    pub extra_completions: usize,
    pub deficits: i64,
    pub unpaid: i64,
    pub terminal: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub agent: AgentId,
    pub offer: Option<u32>,
    pub reason: Reason,
    pub baseline: Option<Forecast>,
    pub expanded: Option<Forecast>,
}

fn forecast(
    world: &World,
    state: &State,
    agent: AgentId,
    resource: ResourceId,
    asset: AssetId,
) -> Result<Forecast, String> {
    let (w, s) = crate::forward::local(world, state, agent);
    let mut sim = Simulation::new(w, s, Backend::Reference)?;
    sim.run_months(FORECAST_MONTHS)?;
    Ok(Forecast {
        closing_stock: sim.state.balance(agent, resource),
        extra_completions: sim
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter_map(|t| t.process.as_ref())
            .filter(|p| {
                p.after.asset == Some(asset)
                    && p.after.status == Status::Completed
                    && world
                        .definition(p.after.definition)
                        .outputs
                        .iter()
                        .any(|a| a.resource == resource)
            })
            .count(),
        deficits: sim
            .reports
            .iter()
            .flat_map(|r| r.needs.values())
            .map(|n| i64::from(n.deficit))
            .sum(),
        unpaid: sim
            .state
            .obligations
            .values()
            .map(|o| i64::from(o.owed - o.paid))
            .sum(),
        terminal: sim.state.terminal.contains_key(&agent),
    })
}

/// One applicant per month, rotating stable IDs. Allocation and timing are explicit.
/// Evidence includes reserved market transfers, before production begins.
pub fn evaluate(world: &World, state: &State) -> Result<Option<Request>, String> {
    let Some(policy) = world.market.as_ref().and_then(|m| m.plots.as_ref()) else {
        return Ok(None);
    };
    if state.month < policy.first_review {
        return Ok(None);
    }
    let mut agents: Vec<_> = world.participants.iter().map(|p| p.agent).collect();
    agents.sort_unstable();
    if agents.is_empty() {
        return Ok(None);
    }
    let agent = agents[((state.month - policy.first_review) as usize) % agents.len()];
    let mut r = Request {
        agent,
        offer: None,
        reason: Reason::NoVacancy,
        baseline: None,
        expanded: None,
    };
    if state
        .accepted_agreements
        .values()
        .filter(|a| a.debtor == agent && policy.offers.contains(&a.id))
        .count()
        >= policy.max_extra
    {
        r.reason = Reason::Limit;
        return Ok(Some(r));
    }
    if state.terminal.contains_key(&agent)
        || state.obligations.values().any(|o| {
            o.paid < o.owed
                && commitments::active(world, state)
                    .any(|a| a.id == o.agreement && a.debtor == agent)
        })
        || state
            .exchange
            .forwards
            .values()
            .any(|c| c.debtor == agent && c.delivered < c.goods.quantity)
    {
        r.reason = Reason::ExistingDebt;
        return Ok(Some(r));
    }
    let mut offers: Vec<_> = world
        .access_offers
        .iter()
        .filter(|a| a.debtor == agent && policy.offers.contains(&a.id))
        .collect();
    offers.sort_by_key(|a| a.id);
    let first_due = state
        .month
        .checked_add(commitments::MONTHS_PER_YEAR)
        .ok_or("plot due date overflow")?;
    let Some(offer) = offers.into_iter().find(|a| {
        commitments::acceptance(world, state, a.id).is_ok()
            && world
                .rights
                .iter()
                .any(|r| r.id == a.right && r.through >= first_due)
    }) else {
        return Ok(Some(r));
    };
    let asset = world
        .rights
        .iter()
        .find(|r| r.id == offer.right)
        .ok_or("missing expansion right")?
        .asset;
    r.offer = Some(offer.id);
    let baseline = forecast(world, state, agent, policy.output, asset)?;
    let mut expanded = state.clone();
    expanded
        .accepted_agreements
        .insert(offer.id, commitments::acceptance(world, state, offer.id)?);
    let outcome = forecast(world, &expanded, agent, policy.output, asset)?;
    r.reason = if outcome.extra_completions > 0
        && outcome.closing_stock > baseline.closing_stock
        && outcome.deficits == 0
        && outcome.unpaid == 0
        && !outcome.terminal
    {
        Reason::Accepted
    } else {
        Reason::InsufficientProductivity
    };
    r.baseline = Some(baseline);
    r.expanded = Some(outcome);
    Ok(Some(r))
}

pub fn scenario(enabled: bool) -> Result<(World, State), String> {
    use crate::{scenario::*, trading_scenario::*};
    let (mut w, mut s) = cash_scenario(DEFAULT_PROVIDERS, true)?;
    // Controlled endowment in both treatments: no seed is created when a right is granted.
    for p in &w.participants {
        s.balances.insert((p.agent, SEED), 2 * STOCK_UNIT);
    }
    let template_right = w
        .rights
        .iter()
        .find(|r| r.holder == PERSON)
        .unwrap()
        .clone();
    let kind = w
        .assets
        .iter()
        .find(|a| a.id == template_right.asset)
        .unwrap()
        .kind;
    let mut offers = Vec::new();
    for plot in 0..EXTRA_PLOTS {
        let asset = EXTRA_ASSET_BASE + plot;
        w.assets.push(Asset {
            id: asset,
            owner: STATE_AGENT,
            kind,
        });
        for (index, p) in w.participants.iter().enumerate() {
            let slot = plot * w.participants.len() as u32 + index as u32;
            let right = EXTRA_RIGHT_BASE + slot;
            let id = EXTRA_AGREEMENT_BASE + slot;
            w.rights.push(UseRight {
                id: right,
                holder: p.agent,
                asset,
                from: 1,
                through: template_right.through,
                output_owner: p.agent,
            });
            w.access_offers.push(commitments::Agreement {
                id,
                right,
                creditor: STATE_AGENT,
                debtor: p.agent,
                activated: 1,
                payment: Amount::new(GRAIN, 2 * STOCK_UNIT),
            });
            w.activities.coin_payments.insert(
                id,
                crate::activities::CoinPayment {
                    resource: TOKEN,
                    coins_per_unit: 1,
                },
            );
            w.issuance.push(crate::currency::Issuance {
                agreement: id,
                token: TOKEN,
                collected_per_token: 2,
            });
            offers.push(id);
        }
    }
    if enabled {
        w.market.as_mut().unwrap().plots = Some(Policy {
            offers,
            output: GRAIN,
            first_review: FIRST_REVIEW_MONTH,
            max_extra: MAX_EXTRA_PER_AGENT,
        });
    }
    Ok((w, s))
}

pub fn after_market(
    world: &World,
    state: &State,
    transactions: &[Transaction],
) -> Result<Option<Request>, String> {
    if world
        .market
        .as_ref()
        .and_then(|m| m.plots.as_ref())
        .is_none()
    {
        return Ok(None);
    }
    let mut observed = state.clone();
    for t in transactions {
        for e in &t.effects {
            *observed.balances.entry(e.account).or_default() += e.delta;
        }
        crate::exchange::record(&mut observed, t);
    }
    evaluate(world, &observed)
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let Some(policy) = world.market.as_ref().and_then(|m| m.plots.as_ref()) else {
        return Ok(());
    };
    let ids: std::collections::BTreeSet<_> = policy.offers.iter().copied().collect();
    if ids.len() != policy.offers.len()
        || policy.first_review == 0
        || policy.max_extra == 0
        || !world
            .resources
            .iter()
            .any(|r| r.id == policy.output && r.kind == ResourceKind::Stock)
        || ids
            .iter()
            .any(|id| !world.access_offers.iter().any(|a| a.id == *id))
    {
        return Err("invalid additional-site policy".into());
    }
    for a in state
        .accepted_agreements
        .values()
        .filter(|a| ids.contains(&a.id))
    {
        let right = world
            .rights
            .iter()
            .find(|r| r.id == a.right)
            .ok_or("missing additional-site right")?;
        for other in commitments::active(world, state).filter(|b| b.id != a.id) {
            let r = world
                .rights
                .iter()
                .find(|r| r.id == other.right)
                .ok_or("missing competing right")?;
            if r.asset == right.asset
                && a.activated <= r.through
                && other.activated <= right.through
            {
                return Err("additional site granted to competing holders".into());
            }
        }
    }
    Ok(())
}
