use economics_compute_smoke::{
    acceptance::{WORKSHOP, continuation::Case, financing},
    agency::work_requests::{
        Snapshot,
        routes::{self, RequirementKind, Route},
    },
    minting::{HOURS, ISSUER, METAL, ORE, REFINE},
    model::{Amount, State, World},
};

fn fixture() -> (World, State) {
    let (mut world, mut state) = financing::scenario(Case::Baseline).unwrap();
    let policy = world
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap();
    policy.month = 5;
    policy.additional_months.clear();
    state.balances.insert((ISSUER, METAL), 0);
    state.balances.insert((WORKSHOP, METAL), 0);
    (world, state)
}

fn metal_route(world: &World, state: &State) -> Route {
    let snapshot = Snapshot::observe(world, state, WORKSHOP).unwrap();
    routes::explain(world, state, &snapshot)
        .unwrap()
        .into_iter()
        .find(|route| route.definition == REFINE)
        .unwrap()
}

#[test]
fn multiple_batches_keep_only_the_first_batch_recipe_and_calendar() {
    let (mut world, mut state) = fixture();
    let recipe = world
        .definitions
        .iter_mut()
        .find(|d| d.id == REFINE)
        .unwrap();
    recipe
        .outputs
        .iter_mut()
        .find(|a| a.resource == METAL)
        .unwrap()
        .quantity = 1;
    recipe.stages[0].entry_inputs = vec![Amount::new(ORE, 2)];
    recipe.stages[0].monthly_services = vec![Amount::new(HOURS, 2)];
    recipe.stages[0].months = 1;
    let mut second_stage = recipe.stages[0].clone();
    second_stage.months = 2;
    recipe.stages.push(second_stage);
    state.balances.insert((WORKSHOP, ORE), 3);
    state.balances.insert((WORKSHOP, HOURS), 100);

    let route = metal_route(&world, &state);
    assert!(route.batches > 1);
    assert_eq!(route.batches, i64::from(route.target));
    assert_eq!(route.first_output_available, 5);
    assert!(route.first_batch_meets_date);
    let inputs: Vec<_> = route
        .requirements
        .iter()
        .filter(|r| r.kind == RequirementKind::EntryStock)
        .collect();
    assert_eq!(inputs.len(), 2);
    assert_eq!(
        (inputs[0].from, inputs[0].through, inputs[0].quantity),
        (2, 2, 2)
    );
    assert_eq!(
        (inputs[1].from, inputs[1].through, inputs[1].quantity),
        (3, 3, 2)
    );
    assert_eq!((inputs[0].covered_now, inputs[1].covered_now), (2, 1));
    assert_eq!((inputs[0].missing_now, inputs[1].missing_now), (0, 1));
    let services: Vec<_> = route
        .requirements
        .iter()
        .filter(|r| r.kind == RequirementKind::MonthlyService)
        .collect();
    assert_eq!(services.len(), 2);
    assert_eq!((services[0].from, services[0].through), (2, 2));
    assert_eq!((services[1].from, services[1].through), (3, 4));
    assert!(
        services
            .iter()
            .all(|r| r.quantity == 2 && r.covered_now == 0 && r.missing_now == 2)
    );
}

#[test]
fn stock_at_target_needs_no_batch_even_when_an_accepted_claim_protects_it() {
    let (world, mut state) = fixture();
    let target = metal_route(&world, &state).target;
    state.balances.insert((WORKSHOP, METAL), target);
    // An independent accepted-delivery fixture isolates stock protection from
    // the raw stock target that ordinary WorkTargets actually evaluates.
    let claim = economics_compute_smoke::forward::Contract {
        id: 999,
        debtor: WORKSHOP,
        creditor: ISSUER,
        issued: state.month,
        due: state.month,
        goods: Amount::new(METAL, target),
        price: economics_compute_smoke::forward::Price {
            goods: target,
            coins: 1,
        },
        advance: Amount::new(economics_compute_smoke::minting::COIN, 1),
        delivered: 0,
        substituted: 0,
        relief: vec![],
    };
    state.exchange.forwards.insert(claim.id, claim);

    let route = metal_route(&world, &state);
    assert_eq!(route.target, target);
    assert_eq!(route.batches, 0);
    assert!(route.protected_output_now >= i128::from(target));
    assert!(route.requirements.is_empty());
    assert_eq!(route.first_output_available, state.month + 1);
    assert!(route.first_batch_meets_date);
    assert!(!route.buyer_funding_verified);
}

#[test]
fn raw_stock_gap_rounds_up_whole_batches_without_requiring_a_full_target() {
    let (world, mut state) = fixture();
    let target = metal_route(&world, &state).target;
    state.balances.insert((WORKSHOP, METAL), target - 1);
    let route = metal_route(&world, &state);
    assert_eq!(route.batches, 1);
    assert!(!route.requirements.is_empty());
    assert_eq!(route.first_output_available, state.month + 2);
}
