//! Independently accepted working-capital advances tied to an authorized program.
//! Admission and repayment use ordinary credit; publication grants no spendable cash.
use super::*;
use crate::{credit, opportunities::Action};

const MAX_PRINCIPAL: i32 = 16;
const MAX_TERM: u32 = 12;
const MAX_CANDIDATES: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terms {
    pub denomination: ResourceId,
    pub max_principal: i32,
    pub monthly_rate_bps: u32,
    pub term_months: u32,
    pub grace_months: u32,
    /// Fixed program IDs explicitly permitted to be financed. No authority from goals.
    pub programs: BTreeSet<u32>,
    pub lend: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub advance: credit::Advance,
    pub program: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Pending,
    Activated(u32),
    AdmissionRejected(u32),
    WorkRejected(u32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutsideOption {
    pub month: u32,
    pub losses: Vec<i128>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comparison {
    pub governor: AgentId,
    pub baseline: Vec<i128>,
    /// Same authorized work without a loan, if it can complete and service debt.
    pub self_funded: Option<OutsideOption>,
    pub proposed: Vec<i128>,
    pub performed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    pub comparisons: BTreeMap<AgentId, Comparison>,
}
impl Assessment {
    pub fn accepted(&self) -> bool {
        self.comparisons.len() == 2
            && self.comparisons.values().all(|c| {
                c.performed
                    && c.proposed <= c.baseline
                    && c.self_funded
                        .as_ref()
                        .is_none_or(|v| c.proposed <= v.losses)
            })
            && self.comparisons.values().any(|c| {
                c.proposed < c.baseline
                    && c.self_funded.as_ref().is_none_or(|v| c.proposed < v.losses)
            })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub month: u32,
    pub proposal: Proposal,
    pub assessment: Result<Assessment, String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agreement {
    pub month: u32,
    pub proposal: Proposal,
    pub program: Program,
    pub assessment: Assessment,
    pub status: Status,
    // Only project() creates this marker; disabling live planning never waives consent.
    hypothetical: bool,
}

pub(super) fn validate_terms(w: &World, actor: AgentId) -> Result<(), String> {
    let c = &w.agency[&actor].config;
    let Some(t) = &c.financing else { return Ok(()) };
    if !(1..=MAX_PRINCIPAL).contains(&t.max_principal)
        || !(1..=MAX_TERM).contains(&t.term_months)
        || t.grace_months == 0
        || t.grace_months > t.term_months
        || t.monthly_rate_bps > credit::RATE_SCALE as u32
        || !w
            .resources
            .iter()
            .any(|r| r.id == t.denomination && r.kind == ResourceKind::Stock)
        || t.programs.iter().any(|id| {
            !c.programs
                .get(id)
                .is_some_and(|p| matches!(p.commands.as_slice(), [Command::StartProcess(_)]))
        })
    {
        return Err("invalid productive financing mandate".into());
    }
    Ok(())
}

pub(super) fn last_agreement(w: &World, agent: AgentId) -> Option<u32> {
    w.agency
        .values()
        .flat_map(|c| &c.financed_programs)
        .filter(|a| [a.proposal.advance.debtor, a.proposal.advance.terms.creditor].contains(&agent))
        .map(|a| a.month)
        .max()
}

fn reviewable(w: &World, s: &State, agent: AgentId) -> bool {
    let c = &w.agency[&agent];
    // This second review can fund the operating choice just published at this
    // boundary. Older choices still impose tenure; cadence is shared, not bypassed.
    let last = c
        .history
        .iter()
        .filter(|d| d.chosen.is_some() && d.month < s.month)
        .map(|d| d.month)
        .chain(last_agreement(w, agent))
        .chain(forwards::execution::last_agreement(w, agent))
        .max();
    c.autonomous
        && governor(w, s, agent).is_some()
        && !s.terminal.contains_key(&agent)
        && (s.month - 1).is_multiple_of(c.config.review_every)
        && last.is_none_or(|m| s.month - m >= c.config.minimum_tenure)
        && !forwards::execution::pending(w, agent)
        && !pending(w, agent)
}

pub fn candidates(w: &World, s: &State) -> Result<Vec<Proposal>, String> {
    let id = w
        .lending
        .iter()
        .map(|a| a.id)
        .chain(s.credit.loans.keys().copied())
        .chain(w.credit.iter().flat_map(|c| c.offers.iter().map(|o| o.id)))
        .chain(w.recovery.guarantees.iter().map(|g| g.recourse))
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("financing ID overflow")?;
    let mut result = vec![];
    for (&debtor, c) in &w.agency {
        let Some(b) = &c.config.financing else {
            continue;
        };
        if !reviewable(w, s, debtor)
            || !c.financed_programs.is_empty()
            || !crate::opportunities::permits(w, s, debtor, Action::Borrow)
            || forwards::execution::pending(w, debtor)
        {
            continue;
        }
        for (&lender, lc) in &w.agency {
            let Some(l) = &lc.config.financing else {
                continue;
            };
            if debtor == lender
                || !reviewable(w, s, lender)
                || !l.lend
                || (
                    l.denomination,
                    l.monthly_rate_bps,
                    l.term_months,
                    l.grace_months,
                ) != (
                    b.denomination,
                    b.monthly_rate_bps,
                    b.term_months,
                    b.grace_months,
                )
                || !crate::opportunities::permits(w, s, lender, Action::Lend)
            {
                continue;
            }
            for &program in &b.programs {
                validate_program(w, debtor, &c.config.programs[&program])?;
                for principal in 1..=b
                    .max_principal
                    .min(l.max_principal)
                    .min(s.balance(lender, l.denomination))
                {
                    result.push(Proposal {
                        program,
                        advance: credit::Advance {
                            id,
                            debtor,
                            principal,
                            month: s.month.checked_add(1).ok_or("loan date overflow")?,
                            collateral: None,
                            priority: 0,
                            terms: credit::LoanOffer {
                                creditor: lender,
                                denomination: b.denomination,
                                max_principal: principal,
                                monthly_rate_bps: b.monthly_rate_bps,
                                term_months: b.term_months,
                                grace_months: b.grace_months,
                            },
                        },
                    });
                    if result.len() > MAX_CANDIDATES {
                        return Err("financing candidate budget exceeded".into());
                    }
                }
            }
        }
    }
    Ok(result)
}

pub fn pending(w: &World, agent: AgentId) -> bool {
    w.agency.values().any(|c| {
        c.financed_programs.iter().any(|a| {
            a.status == Status::Pending
                && [a.proposal.advance.debtor, a.proposal.advance.terms.creditor].contains(&agent)
        })
    })
}

fn record(w: &mut World, s: &State, p: &Proposal, assessment: Assessment, hypothetical: bool) {
    for actor in [p.advance.debtor, p.advance.terms.creditor] {
        let c = w.agency.get_mut(&actor).unwrap();
        c.mandate
            .get_or_insert_with(|| (c.config.clone(), c.discover_programs));
    }
    let program = w.agency[&p.advance.debtor].config.programs[&p.program].clone();
    w.lending.push(p.advance.clone());
    w.agency
        .get_mut(&p.advance.debtor)
        .unwrap()
        .financed_programs
        .push(Agreement {
            month: s.month,
            proposal: p.clone(),
            program,
            assessment,
            status: Status::Pending,
            hypothetical,
        });
}

pub fn assess(w: &World, s: &State, p: &Proposal) -> Result<Assessment, String> {
    if s.phase != Phase::Open || !candidates(w, s)?.contains(p) {
        return Err("unsigned productive financing proposal".into());
    }
    let mut comparisons = BTreeMap::new();
    let debtor = p.advance.debtor;
    let maturity = s
        .credit
        .loans
        .values()
        .filter(|l| [debtor, p.advance.terms.creditor].contains(&l.debtor))
        .map(|l| {
            l.opened
                .checked_add(l.term_months)
                .ok_or("maturity overflow")
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max()
        .unwrap_or(s.month)
        .max(
            p.advance
                .month
                .checked_add(p.advance.terms.term_months)
                .ok_or("maturity overflow")?,
        );
    let mut borrower_options = BTreeSet::new();
    for actor in [debtor, p.advance.terms.creditor] {
        let c = &w.agency[&actor];
        let horizon = c.config.horizon.max(maturity - s.month + 1);
        if horizon > MAX_HORIZON_MONTHS {
            return Err("financing exceeds forecast horizon".into());
        }
        let goals = objectives::freeze(
            w,
            s,
            actor,
            objectives::governing(w, s, actor).ok_or("vacant financing office")?,
        );
        let peers = expectations::Snapshot::observe(w, s, actor, c.config.peer_expectations)?;
        let (base, base_path, _) = project(w, s, horizon, &peers, Plan::Retain)?;
        let (proposed, proposed_path, performed) =
            project(w, s, horizon, &peers, Plan::Financed(p))?;
        let baseline = objectives::measure_projection(
            &base.world,
            &base.state,
            &base.reports,
            actor,
            &goals,
            &base_path,
        )?;
        let mut self_funded: Option<OutsideOption> = None;
        for month in [p.advance.month, p.advance.month + 1] {
            if actor != debtor && !borrower_options.contains(&month) {
                continue;
            }
            let (unfunded, path, viable) =
                project(w, s, horizon, &peers, Plan::SelfFunded(p, month))?;
            if !viable {
                continue;
            }
            let losses = objectives::measure_projection(
                &unfunded.world,
                &unfunded.state,
                &unfunded.reports,
                actor,
                &goals,
                &path,
            )?;
            // The lender cannot assume work that the borrower would refuse.
            if actor == debtor {
                if losses > baseline {
                    continue;
                }
                borrower_options.insert(month);
            }
            if self_funded
                .as_ref()
                .is_none_or(|option| losses < option.losses)
            {
                self_funded = Some(OutsideOption { month, losses });
            }
        }
        comparisons.insert(
            actor,
            Comparison {
                governor: governor(w, s, actor).ok_or("vacant financing office")?,
                baseline,
                proposed: objectives::measure_projection(
                    &proposed.world,
                    &proposed.state,
                    &proposed.reports,
                    actor,
                    &goals,
                    &proposed_path,
                )?,
                performed,
                self_funded,
            },
        );
    }
    Ok(Assessment { comparisons })
}
enum Plan<'a> {
    Retain,
    SelfFunded(&'a Proposal, u32),
    Financed(&'a Proposal),
}
fn project(
    w: &World,
    s: &State,
    horizon: u32,
    peers: &expectations::Snapshot,
    plan: Plan<'_>,
) -> Result<(Simulation, path::Path, bool), String> {
    let (p, finance, work_month) = match plan {
        Plan::Retain => (None, false, 0),
        Plan::SelfFunded(p, month) => (Some(p), false, month),
        Plan::Financed(p) => (Some(p), true, p.advance.month + 1),
    };
    let (mut world, state) = crate::forecast::ForecastContext::new(w, s).into_parts();
    if let Some(p) = p {
        if finance {
            record(
                &mut world,
                s,
                p,
                Assessment {
                    comparisons: BTreeMap::new(),
                },
                true,
            );
        } else {
            let [Command::StartProcess(definition)] =
                world.agency[&p.advance.debtor].config.programs[&p.program]
                    .commands
                    .as_slice()
            else {
                unreachable!()
            };
            let definition = *definition;
            // An independently acceptable outside option, not assumed income.
            // Test both the next ordinary Open and the delayed financed date.
            schedule_start(&mut world, p.advance.debtor, definition, work_month)?;
        }
    }
    let mut sim = Simulation::new(world, state, crate::compute::Backend::Reference)?;
    let end = s
        .month
        .checked_add(horizon)
        .ok_or("financing horizon overflow")?;
    let mut path = path::Path::new(s.month);
    while sim.state.month < end {
        if sim.state.phase == Phase::Open {
            peers.apply_ballots(&mut sim.world, &sim.state)?;
            peers.apply_hypotheses(&mut sim.world, &sim.state)?;
        }
        sim.step()?;
        if sim.state.phase == Phase::Open {
            path.record(&sim.world, &sim.state)?;
        }
    }
    path.record_completions(&sim.ledger)?;
    let performed = if let Some(p) = p {
        let loan = sim.state.credit.loans.get(&p.advance.id);
        let [Command::StartProcess(definition)] = w.agency[&p.advance.debtor].config.programs
            [&p.program]
            .commands
            .as_slice()
        else {
            unreachable!()
        };
        let parties = [p.advance.debtor, p.advance.terms.creditor];
        let wage_miss = sim
            .ledger
            .iter()
            .filter_map(|b| b.employment.as_ref())
            .flat_map(|b| &b.receipts)
            .any(|r| {
                matches!(
                    r.reason,
                    crate::employment::Reason::Collection
                        | crate::employment::Reason::CollectionStayed
                ) && r.requested > r.paid
                    && sim
                        .world
                        .employment
                        .iter()
                        .any(|t| t.id == r.agreement && parties.contains(&t.employer))
            });
        !wage_miss
            && !crate::forecast::lending::missed(&sim.ledger, p.advance.debtor)
            && !crate::forecast::lending::missed(&sim.ledger, p.advance.terms.creditor)
            && (!finance || loan.is_some_and(|l| l.status == credit::Status::Repaid))
            && sim
                .state
                .credit
                .loans
                .values()
                .filter(|l| l.debtor == p.advance.debtor)
                .all(|l| l.status == credit::Status::Repaid)
            && (!finance
                || matches!(
                    sim.world.agency[&p.advance.debtor].financed_programs[0].status,
                    Status::Activated(_)
                ))
            && sim.state.processes.values().any(|v| {
                v.operator == p.advance.debtor
                    && v.definition == *definition
                    && v.start == work_month
                    && v.status == crate::model::Status::Completed
            })
    } else {
        false
    };
    Ok((sim, path, performed))
}

pub fn publish(w: &mut World, s: &State, p: &Proposal) -> Result<Assessment, String> {
    let assessment = assess(w, s, p)?;
    if !assessment.accepted() {
        return Err("financing lacks mutual benefit and timely performance".into());
    }
    let mut candidate = w.clone();
    record(&mut candidate, s, p, assessment.clone(), false);
    crate::settlement::validate_world(&candidate, s)?;
    *w = candidate;
    Ok(assessment)
}

pub(super) fn review(w: &mut World, s: &State) -> Result<(), String> {
    if !w
        .agency
        .values()
        .any(|c| c.autonomous && c.config.financing.is_some())
    {
        return Ok(());
    }
    for p in candidates(w, s)? {
        let result = assess(w, s, &p);
        w.agency
            .get_mut(&p.advance.debtor)
            .unwrap()
            .financing_reviews
            .push(Attempt {
                month: s.month,
                proposal: p.clone(),
                assessment: result.clone(),
            });
        if let Ok(a) = result
            && a.accepted()
        {
            let mut candidate = w.clone();
            record(&mut candidate, s, &p, a, false);
            crate::settlement::validate_world(&candidate, s)?;
            *w = candidate;
            break;
        }
    }
    Ok(())
}

pub(super) fn activate_due(w: &mut World, s: &State) -> Result<(), String> {
    let due: Vec<_> = w
        .agency
        .iter()
        .flat_map(|(&actor, c)| {
            c.financed_programs
                .iter()
                .enumerate()
                .filter(|(_, a)| a.status == Status::Pending && s.month > a.proposal.advance.month)
                .map(move |(i, a)| (actor, i, a.clone()))
        })
        .collect();
    for (actor, i, a) in due {
        let [Command::StartProcess(definition)] = a.program.commands.as_slice() else {
            return Err("unsupported funded program".into());
        };
        let status = if !s.credit.loans.contains_key(&a.proposal.advance.id) {
            Status::AdmissionRejected(s.month)
        } else if !crate::opportunities::permits(w, s, actor, Action::Process(*definition)) {
            Status::WorkRejected(s.month)
        } else {
            schedule_start(w, actor, *definition, s.month)?;
            Status::Activated(s.month)
        };
        w.agency.get_mut(&actor).unwrap().financed_programs[i].status = status;
    }
    Ok(())
}

pub(super) fn validate_history(w: &World, s: &State) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for (&actor, c) in &w.agency {
        if c.financed_programs.len() > 1 {
            return Err("only one productive financing bridge per borrower".into());
        }
        for a in &c.financed_programs {
            let p = &a.proposal;
            let b = c
                .config
                .financing
                .as_ref()
                .ok_or("unsigned borrower history")?;
            let lender = w
                .agency
                .get(&p.advance.terms.creditor)
                .ok_or("missing lender")?;
            let l = lender
                .config
                .financing
                .as_ref()
                .ok_or("unsigned lender history")?;
            let terms = &p.advance.terms;
            if !ids.insert(p.advance.id)
                || actor != p.advance.debtor
                || a.month == 0
                || a.month > s.month
                || a.month.checked_add(1) != Some(p.advance.month)
                || !b.programs.contains(&p.program)
                || c.config.programs.get(&p.program) != Some(&a.program)
                || !w.lending.contains(&p.advance)
                || !l.lend
                || actor == terms.creditor
                || p.advance.principal <= 0
                || p.advance.principal > b.max_principal.min(l.max_principal)
                || terms.max_principal != p.advance.principal
                || p.advance.collateral.is_some()
                || p.advance.priority != 0
                || [b, l].iter().any(|t| {
                    (
                        t.denomination,
                        t.monthly_rate_bps,
                        t.term_months,
                        t.grace_months,
                    ) != (
                        terms.denomination,
                        terms.monthly_rate_bps,
                        terms.term_months,
                        terms.grace_months,
                    )
                })
                || (!a.hypothetical && !a.assessment.accepted())
                || (a.hypothetical
                    && (w.agency.values().any(|c| c.autonomous)
                        || !a.assessment.comparisons.is_empty()))
            {
                return Err("invalid financed program history".into());
            }
            // Forecast-only hypothetical proposals have no consent receipt. Live
            // agreements always retain both exact parties and historical governors.
            if !a.hypothetical
                && (a
                    .assessment
                    .comparisons
                    .keys()
                    .copied()
                    .collect::<BTreeSet<_>>()
                    != [actor, terms.creditor].into()
                    || a.assessment.comparisons.iter().any(|(&party, comparison)| {
                        Some(comparison.governor) != governor_at_open(w, s, party, a.month)
                            || comparison.baseline.is_empty()
                            || comparison.baseline.len() != comparison.proposed.len()
                            || comparison.self_funded.as_ref().is_some_and(|v| {
                                v.losses.len() != comparison.baseline.len()
                                    || ![p.advance.month, p.advance.month + 1].contains(&v.month)
                            })
                    }))
            {
                return Err("financing consent does not match parties and authority".into());
            }
            if c.mandate.is_none() || lender.mandate.is_none() {
                return Err("unsealed financing authority".into());
            }
            let activation = p
                .advance
                .month
                .checked_add(1)
                .ok_or("activation overflow")?;
            match a.status {
                Status::Pending
                    if s.month > activation
                        || (s.month == activation && s.phase != Phase::Open) =>
                {
                    return Err("stale financing activation".into());
                }
                Status::Activated(m) | Status::AdmissionRejected(m) | Status::WorkRejected(m)
                    if m != activation || m > s.month =>
                {
                    return Err("invalid financing activation date".into());
                }
                Status::Activated(_) | Status::WorkRejected(_)
                    if !s.credit.loans.contains_key(&p.advance.id) =>
                {
                    return Err("financed work without admitted loan".into());
                }
                Status::AdmissionRejected(_) if s.credit.loans.contains_key(&p.advance.id) => {
                    return Err("admitted loan marked rejected".into());
                }
                _ => {}
            }
        }
    }
    Ok(())
}
