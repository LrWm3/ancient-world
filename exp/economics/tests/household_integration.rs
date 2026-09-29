use economics_compute_smoke::{
    compute::Backend,
    household_governance::Policy,
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME, support::Mandate},
    model::*,
    scenario::{GRAIN, NUTRITION, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

fn support_fixture() -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    w.households[0].governance.charter.initial_policy = Policy::NeedsFirst;
    w.activities.orders.clear();
    s.balances.insert((HOME, TOKEN), 0);
    s.balances.insert((PERSON, GRAIN), 6);
    w.households[0].support.push(Mandate {
        member: PERSON,
        resource: GRAIN,
        from: 1,
        through: 12,
        revoked_from: None,
        reserve_months: 1,
        private_reserve: 2,
        household_target: 2,
        monthly_limit: 2,
    });
    (w, s)
}

#[test]
fn voluntary_food_support_meets_collective_needs_without_a_market() {
    let (w, s) = support_fixture();
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        sim.run_months(1).unwrap();
        let receipts: Vec<_> = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|b| &b.support)
            .collect();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].accepted, 2);
        assert!(receipts[0].baseline_income.is_none());
        assert!(
            sim.reports
                .iter()
                .filter(|r| [PERSON, 91].contains(&r.agent))
                .all(|r| r.deficit(NUTRITION) == 0)
        );
        let mut replay = s.clone();
        for b in &sim.ledger {
            commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        }
        assert_eq!(replay, sim.state);
        (sim.state, sim.ledger)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn support_protects_private_needs_and_rejects_forged_acceptance() {
    let (w, mut s) = support_fixture();
    s.balances.insert((PERSON, GRAIN), 2);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    let opening = sim.state.clone();
    sim.step().unwrap();
    let mut forged = sim.ledger.last().unwrap().clone();
    let r = &mut forged.household.as_mut().unwrap().support[0];
    assert_eq!(r.accepted, 0);
    r.accepted = 1;
    let mut unchanged = opening.clone();
    assert!(
        commit(
            &sim.world,
            &mut unchanged,
            &forged,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(unchanged, opening);
}
