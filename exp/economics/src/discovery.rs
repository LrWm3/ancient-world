//! Bounded endogenous offer posting and institutional formation at Open.
//! Recipes, law, preferences and term rules are inputs; counterparties, dates,
//! signatures and operational policies are decisions. Forecasts freeze discovery.
use crate::{
    agency,
    compute::Backend,
    household_governance as h,
    model::*,
    opportunities::{self, Action},
    simulation::Simulation,
};
use std::collections::BTreeMap;

mod admission;
mod finance;
mod institutions;
mod market;
pub mod scenario;

const MAX_PARTICIPANTS: usize = 8;
const FORECAST_BUFFER_MONTHS: u32 = 2;
const MAX_HORIZON: u32 = 24;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LandRule {
    pub asset_kind: u32,
    pub duration: u32,
    pub annual_payment: Amount,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseholdRule {
    pub constitution: h::Constitution,
    /// Static parameter template. Leader and initial policy are chosen at formation.
    pub charter: h::Charter,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateRule {
    pub constitution: crate::state_governance::Constitution,
    pub term_months: u32,
    pub objectives: Vec<agency::objectives::Objective>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub enabled: bool,
    pub horizon: u32,
    pub land: Option<LandRule>,
    pub household: Option<HouseholdRule>,
    /// Install governance of the existing public agent after voluntary citizenship.
    pub state: Option<StateRule>,
    pub finance: Option<FinanceRule>,
    pub through: u32,
    pub receipts: Vec<Receipt>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinanceRule {
    pub denomination: ResourceId,
    pub loan_months: u32,
    pub monthly_rate_bps: u32,
    pub delivery_months: u32,
    /// Explicit valuation assumptions in denomination ticks per stock unit.
    pub unit_values: BTreeMap<ResourceId, i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub month: u32,
    pub description: String,
    /// One independently evaluated comparison per proposed signatory.
    pub comparisons: BTreeMap<AgentId, (Vec<i128>, Vec<i128>)>,
    /// A proposal passes its comparison, or a posting/formation was published.
    /// Several initial-policy proposals can pass; the formation record names the winner.
    pub accepted: bool,
}

fn people(w: &World, s: &State) -> Vec<AgentId> {
    w.transaction_policy
        .as_ref()
        .map(|p| {
            p.agent_types
                .iter()
                .filter(|(id, kind)| {
                    **kind == opportunities::PERSON_TYPE && !s.terminal.contains_key(id)
                })
                .map(|(&id, _)| id)
                .collect()
        })
        .unwrap_or_default()
}
fn next_id(ids: impl Iterator<Item = u32>) -> Result<u32, String> {
    ids.max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("discovery identifier overflow".into())
}
pub fn validate(w: &World, s: &State) -> Result<(), String> {
    let Some(c) = &w.discovery else {
        return Ok(());
    };
    if !(1..=MAX_HORIZON).contains(&c.horizon)
        || w.transaction_policy.is_none()
        || c.through > s.month
        || c.receipts.iter().any(|r| r.month == 0 || r.month > s.month)
        || w.participants.len() > MAX_PARTICIPANTS
        || w.competition.is_some()
        || !w.employment.is_empty()
        || w.priority == Priority::ConsequenceAware
        || c.land.as_ref().is_some_and(|l| {
            l.duration <= crate::commitments::MONTHS_PER_YEAR || l.annual_payment.quantity <= 0
        })
        || c.state.as_ref().is_some_and(|g| {
            g.term_months == 0 || g.constitution.policies.is_empty() || g.objectives.is_empty()
        })
    {
        return Err("invalid bounded discovery configuration".into());
    }
    let stock = |id| {
        w.resources
            .iter()
            .any(|r| r.id == id && r.kind == ResourceKind::Stock)
    };
    if c.land
        .as_ref()
        .is_some_and(|l| !stock(l.annual_payment.resource))
        || c.finance.as_ref().is_some_and(|r| {
            !stock(r.denomination)
                || !(1..=MAX_HORIZON - FORECAST_BUFFER_MONTHS).contains(&r.loan_months)
                || !(1..=MAX_HORIZON - FORECAST_BUFFER_MONTHS).contains(&r.delivery_months)
                || r.unit_values
                    .iter()
                    .any(|(id, value)| !stock(*id) || *value <= 0)
        })
    {
        return Err("invalid discovery resource or financial horizon".into());
    }
    if let Some(g) = &c.state {
        agency::objectives::validate(w, &g.objectives)?;
    }
    Ok(())
}
fn record(
    w: &mut World,
    s: &State,
    description: String,
    comparisons: BTreeMap<AgentId, (Vec<i128>, Vec<i128>)>,
    accepted: bool,
) {
    w.discovery.as_mut().unwrap().receipts.push(Receipt {
        month: s.month,
        description,
        comparisons,
        accepted,
    });
}
pub(crate) fn open(w: &mut World, s: &State) -> Result<(), String> {
    let Some(c) = w
        .discovery
        .clone()
        .filter(|c| c.enabled && c.through < s.month)
    else {
        return Ok(());
    };
    crate::settlement::validate_world(w, s)?;
    let authority = w.transaction_policy.as_ref().unwrap().authority;
    if !s.terminal.contains_key(&authority) {
        post_membership(w, s, authority)?;
        post_land(w, s, authority, &c)?;
        institutions::govern(w, s, &c)?;
    }
    market::quotes(w, s, &c)?;
    institutions::households(w, s, &c)?;
    finance::discover(w, s, &c)?;
    w.discovery.as_mut().unwrap().through = s.month;
    Ok(())
}
fn post_membership(w: &mut World, s: &State, authority: AgentId) -> Result<(), String> {
    let p = w.transaction_policy.as_mut().unwrap();
    if p.membership_offers
        .iter()
        .any(|o| o.organization == authority && o.role == crate::membership::CITIZEN)
    {
        return Ok(());
    }
    let id = next_id(p.membership_offers.iter().map(|o| o.id))?;
    p.membership_offers.push(crate::membership::Offer {
        id,
        organization: authority,
        role: crate::membership::CITIZEN,
        eligible_type: opportunities::PERSON_TYPE,
    });
    record(
        w,
        s,
        format!("posted citizenship {id}"),
        BTreeMap::new(),
        true,
    );
    Ok(())
}
fn post_land(w: &mut World, s: &State, authority: AgentId, c: &Config) -> Result<(), String> {
    let Some(rule) = &c.land else {
        return Ok(());
    };
    let mut assets: Vec<_> = w
        .assets
        .iter()
        .filter(|a| {
            crate::credit::owner(w, s, a.id) == Some(authority)
                && a.kind == rule.asset_kind
                && !w
                    .rights
                    .iter()
                    .any(|r| r.asset == a.id && r.through >= s.month)
        })
        .map(|a| a.id)
        .collect();
    assets.sort_unstable();
    for asset in assets {
        let right = next_id(w.rights.iter().map(|r| r.id))?;
        let id = next_id(w.access_offers.iter().chain(&w.agreements).map(|a| a.id))?;
        w.rights.push(UseRight {
            id: right,
            asset,
            holder: authority,
            output_owner: authority,
            from: s.month,
            through: s
                .month
                .checked_add(rule.duration)
                .ok_or("land term overflow")?,
        });
        w.access_offers.push(crate::commitments::Agreement {
            id,
            right,
            creditor: authority,
            debtor: authority,
            activated: s.month,
            payment: rule.annual_payment.clone(),
        });
        w.open_access_offers.insert(id);
        record(
            w,
            s,
            format!("posted land {id} on asset {asset}"),
            BTreeMap::new(),
            true,
        );
    }
    Ok(())
}
fn needs(w: &World, agent: AgentId) -> Vec<agency::objectives::Objective> {
    use agency::objectives::*;
    let mut result = vec![Objective {
        scope: Scope::Agents([agent].into()),
        metric: Metric::Deaths,
    }];
    if let Some(p) = w.participants.iter().find(|p| p.agent == agent) {
        for n in crate::forecast::needs::ordered(&p.needs) {
            result.push(Objective {
                scope: Scope::Agents([agent].into()),
                metric: Metric::NeedDeficit(n.resource),
            });
        }
    }
    result
}
fn forecast(w: &World, s: &State, months: u32) -> Result<Simulation, String> {
    let (w, s) = crate::forecast::ForecastContext::new(w, s).into_parts();
    let mut sim = Simulation::new(w, s, Backend::Reference)?;
    sim.run_months(months)?;
    Ok(sim)
}
fn loss(
    sim: &Simulation,
    agent: AgentId,
    objectives: &[agency::objectives::Objective],
) -> Result<Vec<i128>, String> {
    agency::objectives::measure(&sim.world, &sim.state, &sim.reports, agent, objectives)
}

/// Free, obligation-free citizenship uses a standing acceptance policy. Land
/// admission is a bounded joint reservation of plots, seed and usable labor.
/// Receipts retain the distinction between a feasible request and a grant.
pub(crate) fn acquire(w: &World, s: &State, b: &mut Batch) -> Result<(), String> {
    if s.phase != Phase::Acquire || w.discovery.is_none() {
        return Ok(());
    }
    let mut preview = s.clone();
    for person in people(w, s) {
        for (offer, member) in crate::membership::candidates(w, &preview, person) {
            let a = crate::membership::acceptance(w, &preview, offer, member)?;
            preview
                .memberships
                .insert((member, a.organization, a.role), a);
            b.additional_memberships.push((offer, member));
        }
    }
    b.discovery_allocation =
        admission::land(w, &mut preview, &mut b.additional_access, &b.transactions)?;
    Ok(())
}
pub(crate) fn validate_batch(w: &World, s: &State, b: &Batch) -> Result<(), String> {
    if w.discovery.is_none() || s.phase != Phase::Acquire {
        return if b.discovery_allocation.is_empty() {
            Ok(())
        } else {
            Err("orphan discovery allocation receipt".into())
        };
    }
    let mut expected = Batch::empty(s);
    expected.transactions = b.transactions.clone();
    acquire(w, s, &mut expected)?;
    if b.additional_memberships != expected.additional_memberships
        || b.additional_access != expected.additional_access
        || b.discovery_allocation != expected.discovery_allocation
    {
        return Err("altered discovered offer acceptance".into());
    }
    Ok(())
}
