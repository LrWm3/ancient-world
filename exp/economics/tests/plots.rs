use economics_compute_smoke::{
    compute::Backend,
    model::*,
    plots::{self, Reason},
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

fn fixture(capacity: i32, seeds: i32) -> Simulation {
    let (mut w, mut s) = plots::scenario(true).unwrap();
    w.market
        .as_mut()
        .unwrap()
        .plots
        .as_mut()
        .unwrap()
        .first_review = 1;
    w.market.as_mut().unwrap().tools.clear();
    w.market.as_mut().unwrap().targets.clear();
    w.participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .capacity
        .quantity = capacity * economics_compute_smoke::trading_scenario::LABOR_TICKS_PER_UNIT;
    // Isolate productive expansion from the fixture's unavoidable startup shelter gap.
    for need in &mut w
        .participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .needs
    {
        if need.resource != NUTRITION && need.resource != WARMTH {
            need.quantity = 0;
        }
    }
    s.balances.insert((PERSON, SEED), seeds);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    sim
}

#[test]
fn expansion_requires_actual_extra_work_inputs_and_tax_capacity() {
    let mut productive = fixture(12, 200);
    let request = plots::evaluate(&productive.world, &productive.state)
        .unwrap()
        .unwrap();
    assert_eq!(request.reason, Reason::Accepted, "{request:?}");
    assert!(request.expanded.as_ref().unwrap().extra_completions > 0);
    assert!(
        request.expanded.as_ref().unwrap().closing_stock
            > request.baseline.as_ref().unwrap().closing_stock
    );
    productive.step().unwrap();
    let id = request.offer.unwrap();
    let a = productive.state.accepted_agreements[&id].clone();
    assert_eq!(a.activated, 1);
    productive.world.market.as_mut().unwrap().plots = None;
    productive.run_months(13).unwrap();
    assert_eq!(productive.state.obligations[&(id, 13)].owed, 200);
    assert_eq!(productive.state.obligations[&(id, 13)].paid, 200);
    let extra_plot = productive
        .world
        .rights
        .iter()
        .find(|r| r.id == a.right)
        .unwrap()
        .asset;
    assert!(
        productive
            .state
            .processes
            .values()
            .any(|p| p.asset == Some(extra_plot) && p.status == Status::Completed)
    );
    for (capacity, seeds) in [(1, 200), (12, 0)] {
        let sim = fixture(capacity, seeds);
        let r = plots::evaluate(&sim.world, &sim.state).unwrap().unwrap();
        assert_eq!(
            r.reason,
            Reason::InsufficientProductivity,
            "capacity {capacity}, seeds {seeds}: {r:?}"
        );
    }
}

#[test]
fn request_replay_cpu_order_and_continuation_preserve_the_grant() {
    let mut sim = fixture(12, 200);
    let opening = sim.state.clone();
    sim.step().unwrap();
    let batch = sim.ledger.last().unwrap().clone();
    assert_eq!(
        batch.plot_request.as_ref().unwrap().reason,
        Reason::Accepted
    );
    let mut bad = batch.clone();
    bad.plot_request
        .as_mut()
        .unwrap()
        .expanded
        .as_mut()
        .unwrap()
        .closing_stock += 1;
    let mut replay = opening.clone();
    assert!(
        commit(
            &sim.world,
            &mut replay,
            &bad,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, opening);
    commit(
        &sim.world,
        &mut replay,
        &batch,
        Backend::CubeCpu,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    assert_eq!(replay, sim.state);
    let mut w = sim.world.clone();
    w.participants.reverse();
    w.rights.reverse();
    w.access_offers.reverse();
    w.activities.orders.reverse();
    let mut reversed = Simulation::new(w, opening, Backend::Reference).unwrap();
    reversed.step().unwrap();
    assert_eq!(reversed.state, sim.state);
    assert_eq!(reversed.ledger.last().unwrap(), &batch);
    let mut resumed = Simulation::new(sim.world.clone(), replay, Backend::CubeCpu).unwrap();
    sim.run_months(3).unwrap();
    resumed.run_months(3).unwrap();
    assert_eq!(resumed.state, sim.state);
}

#[test]
fn a_claimed_plot_is_unavailable_to_other_people_and_dormant_rights_stay_dormant() {
    let mut sim = fixture(12, 200);
    sim.step().unwrap();
    let a = sim.state.accepted_agreements.values().next().unwrap();
    let asset = sim
        .world
        .rights
        .iter()
        .find(|r| r.id == a.right)
        .unwrap()
        .asset;
    let competing = sim
        .world
        .access_offers
        .iter()
        .find(|o| {
            o.debtor != PERSON
                && sim
                    .world
                    .rights
                    .iter()
                    .any(|r| r.id == o.right && r.asset == asset)
        })
        .unwrap()
        .id;
    sim.state.phase = Phase::Acquire;
    assert!(
        economics_compute_smoke::commitments::acceptance(&sim.world, &sim.state, competing)
            .is_err()
    );
}

#[test]
fn one_seed_never_finances_two_concurrent_crops() {
    let mut sim = fixture(12, 100);
    // A spare plot can still help while the first is occupied by construction.
    while sim.state.month < 14 {
        sim.step().unwrap();
        assert!(
            sim.state
                .processes
                .values()
                .filter(|p| p.operator == PERSON
                    && p.definition == GROW
                    && p.status == Status::Active)
                .count()
                <= 1
        );
    }
}

#[test]
fn annual_tax_window_existing_arrears_and_holding_limit_are_enforced() {
    let mut sim = fixture(12, 200);
    for r in &mut sim.world.rights {
        if sim.world.access_offers.iter().any(|a| a.right == r.id) {
            r.through = 12;
        }
    }
    assert_eq!(
        plots::evaluate(&sim.world, &sim.state)
            .unwrap()
            .unwrap()
            .reason,
        Reason::NoVacancy
    );
    let mut sim = fixture(12, 200);
    sim.step().unwrap();
    sim.state.phase = Phase::Acquire;
    sim.state.month = 33;
    sim.world
        .market
        .as_mut()
        .unwrap()
        .plots
        .as_mut()
        .unwrap()
        .max_extra = 1;
    assert_eq!(
        plots::evaluate(&sim.world, &sim.state)
            .unwrap()
            .unwrap()
            .reason,
        Reason::Limit
    );
    let mut sim = fixture(12, 200);
    sim.state.month = 13;
    sim.world
        .market
        .as_mut()
        .unwrap()
        .plots
        .as_mut()
        .unwrap()
        .first_review = 13;
    let id = sim
        .world
        .agreements
        .iter()
        .find(|a| a.debtor == PERSON)
        .unwrap()
        .id;
    sim.state.obligations.insert(
        (id, 13),
        economics_compute_smoke::commitments::Obligation {
            agreement: id,
            due: 13,
            owed: 200,
            paid: 0,
            in_kind_paid: 0,
        },
    );
    assert_eq!(
        plots::evaluate(&sim.world, &sim.state)
            .unwrap()
            .unwrap()
            .reason,
        Reason::ExistingDebt
    );
}
