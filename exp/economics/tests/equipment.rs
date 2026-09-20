use economics_compute_smoke::{
    compute::Backend,
    equipment::{self, Trade},
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
fn new(name: &str, backend: Backend) -> Simulation {
    let (w, s) = named(name).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn run(name: &str, backend: Backend) -> Simulation {
    let mut s = new(name, backend);
    s.run_months(60).unwrap();
    s
}
fn commit_batch(sim: &mut Simulation, batch: &Batch) -> Result<(), String> {
    commit(
        &sim.world,
        &mut sim.state,
        batch,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
}

#[test]
fn buying_settles_before_dated_joint_work_and_spends_exactly_one_tool_use() {
    let mut sim = new("tool-beneficial", Backend::Reference);
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Acquire);
    let opening = sim.state.clone();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Productive);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 1);
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 3);
    assert_eq!(opening.balance(PERSON, GRAIN), 4);
    assert_eq!(sim.state.equipment[&TOOL].owner, PERSON);
    assert_eq!(sim.state.equipment[&TOOL].remaining_uses, 6);
    assert!(sim.state.pending_production.is_some());
    let acquisition = sim.ledger.last().unwrap();
    let d = acquisition.decision.as_ref().unwrap();
    assert_eq!(d.alternatives[d.selected].offer, Some(1));
    assert!(
        d.alternatives
            .iter()
            .any(|a| a.offer.is_none() && a.score > d.alternatives[d.selected].score)
    );
    let plan = sim.state.pending_production.clone().unwrap();
    sim.step().unwrap();
    assert_eq!(sim.ledger.last().unwrap(), plan.as_ref());
    assert!(sim.state.pending_production.is_none());
    assert_eq!(sim.state.balance(PERSON, GRAIN), 9);
    assert_eq!(sim.state.balance(PERSON, SEED), 1);
    assert_eq!(sim.state.equipment[&TOOL].remaining_uses, 5);
    assert_eq!(sim.state.equipment[&TOOL].last_used_month, Some(6));
    for definition in [GROW, PREPARE_FUEL] {
        let r = plan
            .receipts
            .iter()
            .find(|r| r.definition == Some(definition))
            .unwrap();
        assert_eq!((r.requested, r.allocated, r.completed), (1, 1, 1));
    }
    let harvest = plan
        .transactions
        .iter()
        .find(|t| {
            t.process
                .as_ref()
                .is_some_and(|p| p.after.definition == GROW)
        })
        .unwrap();
    assert_eq!(harvest.process.as_ref().unwrap().after.asset, Some(PLOT));
    assert_eq!(harvest.technique_use.as_ref().unwrap().asset, Some(TOOL));
}

#[test]
fn useful_investment_survives_and_food_risk_is_declined() {
    let sim = run("tool-beneficial", Backend::Reference);
    let control = run("tool-no-offer", Backend::Reference);
    assert!(sim.state.terminal.is_empty());
    assert!(!control.state.terminal.is_empty());
    assert!(
        sim.reports
            .iter()
            .all(|r| r.deficit(NUTRITION) == 0 && r.deficit(WARMTH) == 0)
    );
    assert_eq!(sim.state.balance(PERSON, GRAIN), 5);
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 3);
    assert_eq!(
        sim.ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter(|t| t.trade.is_some())
            .count(),
        1
    );
    assert_eq!(sim.state.equipment[&TOOL].remaining_uses, 0);
    let uses: Vec<_> = sim
        .ledger
        .iter()
        .filter(|b| b.transactions.iter().any(|t| t.technique_use.is_some()))
        .map(|b| b.month)
        .collect();
    assert_eq!(uses, [6, 15, 23, 31, 39, 47]);
    assert!(sim.state.processes.values().any(|p| p.definition == GROW
        && p.status == Status::Completed
        && p.reserved_through == 63));
    let mut risk = new("tool-food-risk", Backend::Reference);
    risk.step().unwrap();
    let mut forced = risk.clone();
    let mut purchase = Batch::empty(&forced.state);
    purchase.transactions.push(
        equipment::transaction(
            &forced.world,
            &forced.state,
            Trade {
                offer: 1,
                buyer: PERSON,
            },
        )
        .unwrap(),
    );
    commit_batch(&mut forced, &purchase).unwrap();
    risk.step().unwrap();
    let decision = risk.ledger.last().unwrap().decision.as_ref().unwrap();
    assert_eq!(decision.alternatives[decision.selected].offer, None);
    assert!(decision.alternatives.iter().any(|a| a.offer == Some(1)));
    while risk.state.month <= 6 {
        risk.step().unwrap();
    }
    while forced.state.month <= 6 {
        forced.step().unwrap();
    }
    assert!(risk.reports.iter().all(|r| r.deficit(NUTRITION) == 0));
    assert!(forced.reports.iter().any(|r| r.deficit(NUTRITION) > 0));
}

#[test]
fn unavailable_exhausted_and_last_use_tools_fall_back_to_manual() {
    for name in ["tool-unavailable", "tool-exhausted", "tool-lifetime"] {
        let sim = run(name, Backend::Reference);
        assert!(sim.state.terminal.is_empty(), "{name}");
        let harvests: Vec<_> = sim
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter(|t| {
                t.process.as_ref().is_some_and(|p| {
                    p.after.definition == GROW && p.after.status == Status::Completed
                })
            })
            .collect();
        assert!(harvests.len() > 1);
        let assisted = harvests
            .iter()
            .filter(|t| t.technique_use.is_some())
            .count();
        assert_eq!(assisted, usize::from(name == "tool-lifetime"));
        let manual = harvests.iter().find(|t| t.technique_use.is_none()).unwrap();
        assert_eq!(
            manual
                .effects
                .iter()
                .find(|e| e.account == (PERSON, LABOR))
                .unwrap()
                .delta,
            -2
        );
        if name == "tool-lifetime" {
            assert!(harvests[0].technique_use.is_some());
            assert_eq!(sim.state.equipment[&TOOL].last_used_month, Some(6));
            assert_eq!(sim.state.equipment[&TOOL].remaining_uses, 0);
        }
    }
}

#[test]
fn rejected_and_idle_work_do_not_wear_tools_and_shared_use_is_exclusive() {
    let mut failed = new("tool-lifetime", Backend::Reference);
    failed.world.capacity_overrides.insert((6, PERSON), 0);
    failed.run_months(1).unwrap();
    assert_eq!(failed.state.equipment[&TOOL].remaining_uses, 1);
    assert_eq!(failed.state.equipment[&TOOL].last_used_month, None);
    let mut idle = new("tool-lifetime", Backend::Reference);
    idle.state.processes.clear();
    idle.state.balances.insert((PERSON, GRAIN), 100);
    idle.state.balances.insert((PERSON, FUEL), 100);
    idle.run_months(3).unwrap();
    assert_eq!(idle.state.equipment[&TOOL].remaining_uses, 1);
    let mut shared = new("tool-lifetime", Backend::Reference);
    shared.world.participants[0].capacity.quantity = 4;
    shared.world.assets.push(Asset {
        id: 413,
        owner: STATE_AGENT,
        kind: 1,
    });
    let mut right = shared.world.rights[0].clone();
    right.id = 2;
    right.asset = 413;
    shared.world.rights.push(right);
    let mut other = shared.state.processes[&1].clone();
    other.id = 99;
    other.asset = Some(413);
    other.right = Some(2);
    shared.state.processes.insert(99, other);
    shared.step().unwrap();
    shared.step().unwrap();
    let batch = shared.ledger.last().unwrap();
    assert_eq!(
        batch
            .transactions
            .iter()
            .filter(|t| t.technique_use.is_some())
            .count(),
        1
    );
    assert_eq!(
        batch
            .receipts
            .iter()
            .filter(|r| r.definition == Some(GROW))
            .map(|r| r.completed)
            .sum::<i32>(),
        3
    );
    assert_eq!(shared.state.equipment[&TOOL].remaining_uses, 0);
}

#[test]
fn barter_is_atomic_for_duplicate_buyers_insufficient_joint_food_and_forged_terms() {
    let mut sim = new("tool-beneficial", Backend::Reference);
    sim.world.agents.push(Agent {
        id: 99,
        name: "second buyer".into(),
    });
    sim.state.balances.insert((99, GRAIN), 4);
    sim.step().unwrap();
    let original = sim.state.clone();
    let one = equipment::transaction(
        &sim.world,
        &sim.state,
        Trade {
            offer: 1,
            buyer: PERSON,
        },
    )
    .unwrap();
    let two = equipment::transaction(
        &sim.world,
        &sim.state,
        Trade {
            offer: 1,
            buyer: 99,
        },
    )
    .unwrap();
    let mut batch = Batch::empty(&sim.state);
    batch.transactions = vec![one.clone(), two];
    assert!(commit_batch(&mut sim, &batch).is_err());
    assert_eq!(sim.state, original);
    batch.transactions = vec![one];
    batch.transactions[0].effects[0].delta = -2;
    assert!(commit_batch(&mut sim, &batch).is_err());
    assert_eq!(sim.state, original);
    let mut extra = sim.state.equipment[&TOOL].clone();
    extra.id = 501;
    sim.state.equipment.insert(501, extra);
    let mut offer = sim.world.offers[0].clone();
    offer.id = 2;
    offer.asset = 501;
    sim.world.offers.push(offer);
    let original = sim.state.clone();
    batch = Batch::empty(&sim.state);
    for offer in [1, 2] {
        batch.transactions.push(
            equipment::transaction(
                &sim.world,
                &sim.state,
                Trade {
                    offer,
                    buyer: PERSON,
                },
            )
            .unwrap(),
        );
    }
    assert!(commit_batch(&mut sim, &batch).is_err());
    assert_eq!(sim.state, original);
}

#[test]
fn altered_dated_plan_and_equipment_effects_fail_without_partial_publication() {
    let mut sim = new("tool-beneficial", Backend::Reference);
    sim.step().unwrap();
    sim.step().unwrap();
    let original = sim.state.clone();
    let mut wrong = *sim.state.pending_production.clone().unwrap();
    wrong.transactions[0].effects[0].delta -= 1;
    assert!(commit_batch(&mut sim, &wrong).is_err());
    assert_eq!(sim.state, original);
    // Independently validate assisted work even without a stored plan.
    let valid = *sim.state.pending_production.clone().unwrap();
    sim.state.pending_production = None;
    let original = sim.state.clone();
    for variant in 0..3 {
        let mut bad = valid.clone();
        let t = bad
            .transactions
            .iter_mut()
            .find(|t| t.technique_use.is_some())
            .unwrap();
        match variant {
            0 => t.technique_use.as_mut().unwrap().asset = Some(999),
            1 => {
                t.effects
                    .iter_mut()
                    .find(|e| e.account == (PERSON, LABOR))
                    .unwrap()
                    .delta = -2
            }
            _ => {
                t.process.as_mut().unwrap().after.status = Status::Aborted;
            }
        }
        assert!(commit_batch(&mut sim, &bad).is_err());
        assert_eq!(sim.state, original);
    }
}

#[test]
fn short_horizons_can_miss_tool_value_but_longer_forecasts_see_lost_seed() {
    let mut short = new("tool-beneficial", Backend::Reference);
    short.state.balances.insert((PERSON, GRAIN), 20);
    let mut long = short.clone();
    long.world.horizon = 30;
    for sim in [&mut short, &mut long] {
        sim.step().unwrap();
        sim.step().unwrap();
    }
    assert!(short.state.filled_offers.is_empty());
    assert!(long.state.filled_offers.contains(&1));
}

#[test]
fn cpu_replay_predictions_and_checkpoints_include_ownership_wear_and_pending_work() {
    for &name in TOOL_SCENARIOS {
        let reference = run(name, Backend::Reference);
        let cpu = run(name, Backend::CubeCpu);
        assert_eq!(cpu.state, reference.state, "{name}");
        assert_eq!(cpu.ledger, reference.ledger, "{name}");
        assert_eq!(cpu.reports, reference.reports, "{name}");
        let mut replay = new(name, Backend::Reference);
        for batch in &cpu.ledger {
            if let Some(d) = &batch.decision {
                let selected = &d.alternatives[d.selected];
                for predicted in selected.outcomes.iter().filter(|r| r.month == batch.month) {
                    assert_eq!(
                        Some(predicted),
                        cpu.reports
                            .iter()
                            .find(|r| r.month == predicted.month && r.agent == predicted.agent)
                    );
                }
                if let Some(plan) = &batch.production_plan {
                    assert_eq!(selected.first_work, plan.receipts);
                }
            }
            commit_batch(&mut replay, batch).unwrap();
        }
        assert_eq!(replay.state, cpu.state);
        let mut monthly = new(name, Backend::Reference);
        for _ in 0..60 {
            monthly.run_months(1).unwrap();
        }
        assert_eq!(monthly.state, reference.state);
        assert_eq!(monthly.ledger, reference.ledger);
    }
    let expected = run("tool-beneficial", Backend::Reference);
    let mut boundary = new("tool-beneficial", Backend::Reference);
    let end = boundary.state.month + 60;
    for _ in 0..5 {
        let mut resumed = boundary.clone();
        while resumed.state.month < end {
            resumed.step().unwrap();
        }
        assert_eq!(resumed.state, expected.state);
        assert_eq!(resumed.ledger, expected.ledger);
        boundary.step().unwrap();
    }
    let mut reordered = new("tool-beneficial", Backend::Reference);
    reordered.world.definitions.reverse();
    reordered.world.resources.reverse();
    reordered.world.participants[0].needs.reverse();
    reordered.run_months(60).unwrap();
    assert_eq!(reordered.ledger, expected.ledger);
}
