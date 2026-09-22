use economics_compute_smoke::{
    allocation::{Claim, Context, Outcome, Policy, RankingPolicy},
    compute::Backend,
    consequence_priority::{Assessment, Harm, Mode, Ranking, Severity},
    intermediary,
    model::*,
    scenario::*,
    settlement,
};
use std::collections::BTreeMap;
fn claim(id: u64) -> Claim {
    Claim {
        id,
        priority: 0,
        requested: 1,
        minimum: 1,
    }
}
fn assessment(month: u32, terminal: u64, impaired: u64, deprivation: u64) -> Assessment {
    Assessment {
        denied: vec![Harm {
            month,
            terminal,
            impaired,
            deprivation,
        }],
        accepted: vec![Harm {
            month,
            terminal: 0,
            impaired: 0,
            deprivation: 0,
        }],
    }
}
#[test]
fn generic_ranking_respects_severity_timing_magnitude_and_fallback() {
    let context = Context {
        seed: 7,
        pool: 1,
        round: 2,
    };
    let claims: Vec<_> = (1..=6).map(claim).collect();
    let reports = BTreeMap::from([
        (1, assessment(2, 0, 0, 1000)),
        (2, assessment(3, 0, 1, 0)),
        (3, assessment(5, 1, 0, 0)),
        (4, assessment(4, 1, 0, 0)),
        (5, assessment(4, 1, 0, 0)),
        (6, assessment(4, 2, 0, 0)),
    ]);
    let ranking = Ranking::new(context, &Policy::StablePriority, &claims, &reports).unwrap();
    let mut order = claims.clone();
    order.sort_by_key(|c| ranking.key(context, c));
    assert_eq!(
        order.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![6, 4, 5, 3, 2, 1]
    );
    let mut reversed = claims;
    reversed.reverse();
    let other = Ranking::new(context, &Policy::StablePriority, &reversed, &reports).unwrap();
    for c in &order {
        assert_eq!(ranking.key(context, c), other.key(context, c));
    }
    let equal: BTreeMap<_, _> = order
        .iter()
        .map(|c| (c.id, assessment(2, 0, 0, 0)))
        .collect();
    let ranking = Ranking::new(context, &Policy::Lottery, &order, &equal).unwrap();
    let mut expected = order.clone();
    expected.sort_by_key(|c| (Policy::Lottery.key(context, c), c.id));
    order.sort_by_key(|c| ranking.key(context, c));
    assert_eq!(order, expected);
}
#[test]
fn malformed_or_higher_severity_worsening_does_not_claim_benefit() {
    let mut report = assessment(2, 0, 0, 100);
    report.accepted[0].impaired = 1;
    assert_eq!(report.priority().unwrap().severity.0, Severity::None);
    report.accepted[0].month = 3;
    assert!(report.priority().is_err());
}
fn acquire(e: &mut intermediary::Experiment) {
    while e.simulation.state.phase != Phase::Acquire {
        e.step().unwrap();
    }
}
fn prepare(e: &intermediary::Experiment, mode: Mode) -> (Batch, intermediary::Round) {
    intermediary::prepare_with_policy(
        &e.simulation,
        e.policy,
        e.seed,
        e.access_mode,
        &e.access_memory,
        mode,
    )
    .unwrap()
}
fn winner(round: &intermediary::Round) -> u64 {
    round
        .resolution
        .receipts
        .iter()
        .find(|r| matches!(r.outcome, Outcome::Reserved(_)))
        .unwrap()
        .claim
        .id
}
#[test]
fn identical_requests_change_recipient_and_preserve_dated_cpu_work() {
    let mut e = intermediary::consequence_scenario(false, Backend::CubeCpu).unwrap();
    acquire(&mut e);
    let opening = e.simulation.state.clone();
    let (_, existing) = prepare(&e, Mode::Existing);
    let (batch, urgent) = prepare(&e, Mode::AvoidHarm);
    assert_eq!(e.simulation.state, opening);
    assert_eq!(existing.assessments, urgent.assessments);
    for (a, b) in existing.decisions.iter().zip(&urgent.decisions) {
        assert_eq!(a.selected, b.selected);
        assert_eq!(a.alternatives, b.alternatives);
    }
    assert_eq!(
        existing.decisions[0].selected,
        Some(intermediary::MAKE_TOOL)
    );
    assert_eq!(existing.decisions[1].selected, Some(PREPARE_FUEL));
    assert_eq!(winner(&existing), u64::from(PERSON));
    assert_eq!(winner(&urgent), u64::from(PERSON + 1));
    assert_eq!(urgent.decisions[0].fallback, Some(PREPARE_FUEL));
    e.allocation_mode = Mode::AvoidHarm;
    e.step().unwrap();
    assert_eq!(e.simulation.state.pending_production, batch.production_plan);
    let mut forged = *e.simulation.state.pending_production.clone().unwrap();
    forged.transactions.clear();
    let before = e.simulation.state.clone();
    assert!(
        settlement::commit(
            &e.simulation.world,
            &mut e.simulation.state,
            &forged,
            e.simulation.backend,
            e.simulation.effect_limit
        )
        .is_err()
    );
    assert_eq!(e.simulation.state, before);
    e.step().unwrap();
    assert!(e.simulation.state.equipment.is_empty());
    assert_eq!(e.simulation.state.balance(STATE_AGENT, RAW_WOOD), 0);
    assert_eq!(e.simulation.state.balance(PERSON, LABOR), 1);
    assert_eq!(e.simulation.state.balance(PERSON + 1, LABOR), 1);
}
#[test]
fn covered_inactive_and_unavailable_claims_do_not_block_investment() {
    let mut e = intermediary::consequence_scenario(true, Backend::Reference).unwrap();
    acquire(&mut e);
    let (_, r) = prepare(&e, Mode::AvoidHarm);
    assert_eq!(winner(&r), u64::from(PERSON));
    assert!(
        r.decisions
            .iter()
            .all(|d| d.selected == Some(intermediary::MAKE_TOOL))
    );
    for no_labor in [false, true] {
        let mut e = intermediary::consequence_scenario(false, Backend::Reference).unwrap();
        if !no_labor {
            e.simulation
                .world
                .participants
                .iter_mut()
                .find(|p| p.agent == PERSON + 1)
                .unwrap()
                .needs
                .iter_mut()
                .find(|n| n.resource == WARMTH)
                .unwrap()
                .quantity = 0;
        }
        acquire(&mut e);
        if no_labor {
            e.simulation.state.balances.insert((PERSON + 1, LABOR), 1);
        }
        let (_, r) = prepare(&e, Mode::AvoidHarm);
        assert_eq!(winner(&r), u64::from(PERSON));
        assert!(
            !r.decisions
                .iter()
                .any(|d| d.agent == PERSON + 1 && d.selected == Some(PREPARE_FUEL))
        );
    }
}
#[test]
fn cpu_outcomes_continuation_and_reordering_show_actual_consequences() {
    let mut existing = intermediary::consequence_scenario(false, Backend::CubeCpu).unwrap();
    let mut urgent = existing.clone();
    urgent.allocation_mode = Mode::AvoidHarm;
    let mut reference = urgent.clone();
    reference.simulation.backend = Backend::Reference;
    reference.simulation.world.participants.reverse();
    reference.simulation.world.agents.reverse();
    existing.run_months(12).unwrap();
    urgent.run_months(6).unwrap();
    reference.run_months(6).unwrap();
    assert_eq!(urgent.simulation.state, reference.simulation.state);
    assert_eq!(urgent.rounds, reference.rounds);
    assert_eq!(urgent.simulation.ledger, reference.simulation.ledger);
    let mut resumed = urgent.clone();
    urgent.run_months(6).unwrap();
    for _ in 0..6 {
        resumed.run_months(1).unwrap();
    }
    assert_eq!(urgent.simulation.state, resumed.simulation.state);
    assert_eq!(urgent.rounds, resumed.rounds);
    assert_eq!(urgent.access_memory, resumed.access_memory);
    let warmth = |e: &intermediary::Experiment| {
        e.simulation
            .reports
            .iter()
            .map(|r| r.deficit(WARMTH))
            .sum::<i32>()
    };
    assert_eq!(warmth(&existing), 6);
    assert_eq!(warmth(&urgent), 0);
    assert!(
        existing
            .simulation
            .state
            .processes
            .values()
            .any(|p| p.definition == GROW && p.status == Status::Aborted)
    );
    assert!(
        !urgent
            .simulation
            .state
            .processes
            .values()
            .any(|p| p.definition == GROW && p.status == Status::Aborted)
    );
    assert!(
        existing
            .simulation
            .state
            .terminal
            .contains_key(&(PERSON + 1))
    );
    assert!(urgent.simulation.state.terminal.is_empty());
    assert_eq!(existing.simulation.state.equipment.len(), 1);
    assert_eq!(urgent.simulation.state.equipment.len(), 0);
}
