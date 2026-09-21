use economics_compute_smoke::{
    agreements::{Consequence, Grant, Identity, Status, Use},
    commitments,
    compute::Backend,
    membership,
    model::Phase,
    opportunities::{self, Action},
    scenario::*,
    simulation::Simulation,
};

#[test]
fn citizenship_and_land_share_terms_but_not_obligations_or_consequences() {
    let (world, mut state) = membership::scenario().unwrap();
    state.phase = Phase::Acquire;
    let citizen = membership::acceptance(&world, &state, 1, PERSON).unwrap();
    state
        .memberships
        .insert((PERSON, STATE_AGENT, membership::CITIZEN), citizen.clone());
    let land = commitments::acceptance(&world, &state, 1).unwrap();
    state.accepted_agreements.insert(land.id, land.clone());
    let membership = citizen.contract();
    let access = land.contract(&world, &state).unwrap();
    assert_ne!(membership.identity, access.identity);
    assert_eq!(access.identity, Identity::Land(1));
    assert_eq!(
        (membership.grantor, membership.holder),
        (access.grantor, access.holder)
    );
    assert_eq!(membership.evaluate(0).status, Status::Pending);
    assert_eq!(membership.evaluate(1).status, Status::Active);
    assert!(membership.payments.is_empty());
    assert!(membership.obligations.is_empty());
    let terms = &access.payments[0];
    assert_eq!((terms.first_due, terms.interval_months), (13, 12));
    assert_eq!(
        terms.on_unpaid,
        Consequence::SuspendNewUse(Grant::LandUse(land.right))
    );

    state.month = 13;
    state.obligations.insert(
        (land.id, 13),
        commitments::Obligation {
            agreement: land.id,
            due: 13,
            owed: land.payment.quantity,
            paid: 0,
            in_kind_paid: 0,
        },
    );
    let access = land.contract(&world, &state).unwrap();
    assert_eq!(access.evaluate(12).status, Status::Active);
    assert_eq!(access.evaluate(13).status, Status::Restricted);
    assert_eq!(
        access.evaluate(13).breaches[0].outstanding,
        land.payment.quantity
    );
    assert!(!access.permits(13, &Grant::LandUse(land.right), Use::Start));
    assert!(access.permits(13, &Grant::LandUse(land.right), Use::Continue));
    assert!(opportunities::permits(
        &world,
        &state,
        PERSON,
        Action::Process(GROW)
    ));
    assert!(!commitments::can_start(&world, &state, land.right));
    assert!(membership.evaluate(13).breaches.is_empty());

    let o = state.obligations.get_mut(&(land.id, 13)).unwrap();
    o.paid = o.owed - 1;
    assert!(!commitments::can_start(&world, &state, land.right));
    let o = state.obligations.get_mut(&(land.id, 13)).unwrap();
    o.paid = o.owed;
    o.in_kind_paid = o.owed;
    assert!(commitments::can_start(&world, &state, land.right));
    assert_eq!(
        land.contract(&world, &state).unwrap().evaluate(13).status,
        Status::Active
    );
}

#[test]
fn expiration_ends_grants_without_erasing_claims_and_consequence_is_scoped() {
    let (world, mut state) = named("annual-access").unwrap();
    let land = &world.agreements[0];
    state.obligations.insert(
        (land.id, 13),
        commitments::Obligation {
            agreement: land.id,
            due: 13,
            owed: 1,
            paid: 0,
            in_kind_paid: 0,
        },
    );
    let mut contract = land.contract(&world, &state).unwrap();
    let unrelated = Grant::LandUse(999);
    contract.grants.push(unrelated.clone());
    assert!(contract.permits(13, &unrelated, Use::Start));
    let expired = contract.through.unwrap() + 1;
    assert_eq!(contract.evaluate(expired).status, Status::Expired);
    assert_eq!(contract.evaluate(expired).breaches.len(), 1);
    assert!(!contract.permits(expired, &Grant::LandUse(land.right), Use::Continue));
}

#[test]
fn cpu_committed_arrears_restrict_new_use_then_harvest_payment_restores_it() {
    let (world, state) = named("annual-arrears").unwrap();
    let mut sim = Simulation::new(world, state, Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    sim.step().unwrap();
    let agreement = sim.world.agreements[0].clone();
    let contract = agreement.contract(&sim.world, &sim.state).unwrap();
    assert_eq!(
        contract.evaluate(sim.state.month).status,
        Status::Restricted
    );
    assert_eq!(contract.evaluate(sim.state.month).breaches[0].obligation, 0);
    assert!(contract.permits(
        sim.state.month,
        &Grant::LandUse(agreement.right),
        Use::Continue
    ));
    sim.step().unwrap(); // Existing crop completes before arrears settlement.
    assert_eq!(sim.state.balance(PERSON, GRAIN), 8);
    assert_eq!(
        agreement
            .contract(&sim.world, &sim.state)
            .unwrap()
            .evaluate(sim.state.month)
            .status,
        Status::Restricted
    );
    sim.step().unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 1);
    assert_eq!(
        agreement
            .contract(&sim.world, &sim.state)
            .unwrap()
            .evaluate(sim.state.month)
            .status,
        Status::Active
    );
}
