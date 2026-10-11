//! Dated, admission-contingent activation shared by real and hypothetical runs.
use super::*;
use crate::agency;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Pending,
    Activated(u32),
    AdmissionRejected(u32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agreement {
    pub month: u32,
    pub proposal: Proposal,
    pub governors: BTreeMap<AgentId, AgentId>,
    pub programs: BTreeMap<AgentId, agency::Program>,
    pub status: Status,
}

pub fn reviewable(w: &World, s: &State, agent: AgentId) -> bool {
    let Some(c) = w.agency.get(&agent) else {
        return false;
    };
    let last = c.history.iter().rev().find(|d| d.chosen.is_some());
    let recent_forward = w
        .agency
        .values()
        .flat_map(|c| &c.startup_forwards)
        .filter(|a| [a.proposal.terms.buyer, a.proposal.terms.seller].contains(&agent))
        .map(|a| a.month)
        .max();
    c.autonomous
        && c.discover_programs
        && agency::governor(w, s, agent).is_some()
        && (s.month - 1).is_multiple_of(c.config.review_every)
        && last.is_none_or(|d| s.month - d.month >= c.config.minimum_tenure)
        && recent_forward.is_none_or(|month| s.month - month >= c.config.minimum_tenure)
        && !pending(w, agent)
}

pub fn pending(w: &World, agent: AgentId) -> bool {
    w.agency
        .values()
        .flat_map(|c| &c.startup_forwards)
        .any(|a| {
            a.status == Status::Pending
                && [a.proposal.terms.buyer, a.proposal.terms.seller].contains(&agent)
        })
}

/// Reassess from the actual current boundary, then publish atomically. A forecast
/// receipt supplied by a caller is never itself authorization or a balance change.
pub fn publish(
    w: &mut World,
    s: &State,
    proposal: &Proposal,
) -> Result<assessment::Assessment, String> {
    if s.phase != Phase::Open
        || ![proposal.terms.buyer, proposal.terms.seller]
            .iter()
            .all(|a| reviewable(w, s, *a))
    {
        return Err("forward parties are not due for authorized review".into());
    }
    let result = assessment::assess(w, s, proposal)?;
    if !result.mutually_beneficial() {
        return Err("forward lacks mutual projected benefit and delivery".into());
    }
    let mut candidate = w.clone();
    record(&mut candidate, s, proposal)?;
    crate::settlement::validate_world(&candidate, s)?;
    *w = candidate;
    Ok(result)
}

pub(super) fn record(w: &mut World, s: &State, p: &Proposal) -> Result<(), String> {
    let mut governors = BTreeMap::new();
    let mut programs = BTreeMap::new();
    for (agent, id) in [
        (p.terms.buyer, p.buyer_program),
        (p.terms.seller, p.seller_program),
    ] {
        let catalog = agency::discovery::programs(w, agent)?;
        let program = catalog.get(&id).ok_or("unknown conditional program")?;
        agency::issue(w, s, agent, program)?;
        programs.insert(agent, program.clone());
        let controller = w.agency.get_mut(&agent).unwrap();
        controller
            .mandate
            .get_or_insert_with(|| (controller.config.clone(), controller.discover_programs));
        governors.insert(
            agent,
            agency::governor(w, s, agent).ok_or("vacant forward office")?,
        );
    }
    w.prepaid_deliveries.push(p.terms.clone());
    w.agency
        .get_mut(&p.terms.buyer)
        .unwrap()
        .startup_forwards
        .push(Agreement {
            month: s.month,
            proposal: p.clone(),
            governors,
            programs,
            status: Status::Pending,
        });
    Ok(())
}

pub(crate) fn activate_due(w: &mut World, s: &State) -> Result<(), String> {
    let due: Vec<_> =
        w.agency
            .iter()
            .flat_map(|(&agent, c)| {
                c.startup_forwards
                    .iter()
                    .enumerate()
                    .filter_map(move |(i, a)| {
                        (a.status == Status::Pending && s.month > a.proposal.terms.month)
                            .then_some((agent, i, a.proposal.clone()))
                    })
            })
            .collect();
    for (agent, i, p) in due {
        let admitted = s.exchange.forwards.contains_key(&p.terms.id);
        if admitted {
            assessment::activate(w, &p)?;
        }
        w.agency.get_mut(&agent).unwrap().startup_forwards[i].status = if admitted {
            Status::Activated(s.month)
        } else {
            Status::AdmissionRejected(s.month)
        };
    }
    Ok(())
}

pub(crate) fn validate_history(w: &World, s: &State) -> Result<(), String> {
    use crate::marketplace::Side;
    let mut ids = std::collections::BTreeSet::new();
    let mut uses = std::collections::BTreeSet::new();
    for (&owner, c) in &w.agency {
        for a in &c.startup_forwards {
            let p = &a.proposal;
            let t = &p.terms;
            let listing = super::listing(w, p.market).ok_or("missing forward listing")?;
            if owner != t.buyer
                || !ids.insert(t.id)
                || a.month == 0
                || a.month > s.month
                || a.month.checked_add(1) != Some(t.month)
                || !w.prepaid_deliveries.contains(t)
                || t.goods != listing.goods
                || t.prepayment.resource != listing.payment
                || a.governors.len() != 2
                || a.programs.len() != 2
            {
                return Err("invalid organization forward history".into());
            }
            for (agent, program, side) in [
                (t.buyer, p.buyer_program, Side::Buy),
                (t.seller, p.seller_program, Side::Sell),
            ] {
                let controller = w.agency.get(&agent).ok_or("missing forward controller")?;
                let agreement = controller
                    .operating_agreement()
                    .ok_or("missing signed forward authority")?;
                let option = agreement
                    .forwards
                    .get(&p.market)
                    .ok_or("unsigned forward history")?;
                if !uses.insert((agent, t.goods.resource))
                    || controller.mandate.is_none()
                    || !controller.discover_programs
                    || a.programs.get(&agent)
                        != agency::discovery::programs(w, agent)?.get(&program)
                    || option.prepayment != t.prepayment.quantity
                    || a.month.checked_add(option.delivery_months) != Some(t.due)
                    || !super::productive_programs(w, agent, p.market, side)?.contains(&program)
                    || a.governors.get(&agent).copied()
                        != agency::organization::at_open(w, s, agreement, a.month)
                {
                    return Err("forward history exceeds signed authority".into());
                }
            }
            let activation = t.month.checked_add(1).ok_or("activation date overflow")?;
            match a.status {
                Status::Pending if s.month > activation => {
                    return Err("stale pending forward activation".into());
                }
                Status::Activated(month) | Status::AdmissionRejected(month)
                    if month != activation || month > s.month =>
                {
                    return Err("invalid forward activation date".into());
                }
                Status::Activated(_) if !s.exchange.forwards.contains_key(&t.id) => {
                    return Err("forward work activated without admission".into());
                }
                Status::AdmissionRejected(_) if s.exchange.forwards.contains_key(&t.id) => {
                    return Err("admitted forward marked rejected".into());
                }
                _ => {}
            }
        }
    }
    Ok(())
}
