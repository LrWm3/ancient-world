use economics_compute_smoke::{
    compute::Backend,
    credit,
    model::*,
    offers::{self, Id, Request, Terms},
    scenario::{PERSON, STATE_AGENT},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};

fn sim(case: &str) -> Simulation {
    let (w, s) = credit::scenario(case).unwrap();
    Simulation::new(w, s, Backend::CubeCpu).unwrap()
}
fn request() -> Request {
    Request::new(Id::FinancedPurchase(1), PERSON)
}
fn acquire(s: &mut Simulation) {
    s.step().unwrap();
    s.step().unwrap();
    assert_eq!(s.state.phase, Phase::Acquire);
}

#[test]
fn common_discovery_and_acceptance_match_existing_credit_receipts() {
    let mut s = sim("repaid");
    let found = offers::discover(&s.world, &s.state, PERSON);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, request().offer);
    assert_ne!(found[0].id, Id::Land(1));
    let Terms::FinancedPurchase { offer, application } = &found[0].terms else {
        panic!()
    };
    let c = s.world.credit.as_ref().unwrap();
    assert_eq!(offer, &c.offers[0]);
    assert_eq!(application, &c.application);
    assert!(offers::discover(&s.world, &s.state, STATE_AGENT).is_empty());
    assert!(offers::discover(&s.world, &s.state, 999).is_empty());
    assert!(offers::feasible(&s, &[request()]).is_err()); // Open is not acceptance.
    acquire(&mut s);
    let opening = s.clone();
    let prepared = offers::prepare(&s, &[request()]).unwrap();
    assert_eq!(
        prepared.credit,
        credit::evaluate(&s.world, &s.state).unwrap()
    );
    offers::feasible(&s, &[request()]).unwrap();
    assert_eq!(s.state, opening.state);
    assert_eq!(s.ledger, opening.ledger);
    let mut automatic = opening.clone();
    automatic.step().unwrap();
    offers::accept(&mut s, &[request()]).unwrap();
    assert_eq!(s.state, automatic.state);
    assert_eq!(s.ledger, automatic.ledger);
    assert!(offers::discover(&s.world, &s.state, PERSON).is_empty());
    s.run_months(5).unwrap();
    automatic.backend = Backend::Reference;
    automatic.run_months(5).unwrap();
    assert_eq!(s.state, automatic.state);
    assert_eq!(s.ledger, automatic.ledger);
}

#[test]
fn insufficient_funds_fail_without_mutation_and_driver_retains_rejection_receipts() {
    for borrower_short in [true, false] {
        let mut s = sim(if borrower_short {
            "downpayment"
        } else {
            "repaid"
        });
        if !borrower_short {
            s.world
                .credit
                .as_mut()
                .unwrap()
                .endowments
                .iter_mut()
                .find(|e| e.agent == STATE_AGENT)
                .unwrap()
                .amount
                .quantity = 1;
        }
        acquire(&mut s);
        assert_eq!(offers::discover(&s.world, &s.state, PERSON).len(), 1);
        let before = s.clone();
        assert!(offers::feasible(&s, &[request()]).is_err());
        assert!(offers::accept(&mut s, &[request()]).is_err());
        assert_eq!(s.state, before.state);
        assert_eq!(s.ledger, before.ledger);
        let expected = credit::evaluate(&s.world, &s.state).unwrap().unwrap();
        s.step().unwrap();
        assert_eq!(s.ledger.last().unwrap().credit.as_ref(), Some(&expected));
        assert!(
            expected
                .events
                .iter()
                .any(|e| matches!(e, credit::Event::Rejected { .. }))
        );
        assert!(s.state.credit.loans.is_empty());
    }
}

#[test]
fn stale_unavailable_or_duplicate_requests_cannot_publish_partial_purchases() {
    let mut s = sim("default");
    acquire(&mut s);
    let prepared = offers::prepare(&s, &[request()]).unwrap();
    let before = s.state.clone();
    for requests in [
        vec![request(), request()],
        vec![request(), Request::new(Id::Process(1), PERSON)],
        vec![Request::new(Id::FinancedPurchase(99), PERSON)],
        vec![Request::new(Id::FinancedPurchase(1), STATE_AGENT)],
    ] {
        assert!(offers::accept(&mut s, &requests).is_err());
        assert_eq!(s.state, before);
    }
    let mut changed = s.clone();
    changed.world.credit.as_mut().unwrap().offers[0]
        .sale
        .price
        .quantity -= 1;
    assert!(
        settlement::commit(
            &changed.world,
            &mut changed.state,
            &prepared,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(changed.state, before);
    let asset = s.world.credit.as_ref().unwrap().offers[0].sale.asset;
    let mut unavailable = s.clone();
    unavailable.state.credit.owners.insert(asset, PERSON);
    let unavailable_before = unavailable.state.clone();
    assert!(offers::discover(&unavailable.world, &unavailable.state, PERSON).is_empty());
    assert!(offers::accept(&mut unavailable, &[request()]).is_err());
    assert_eq!(unavailable.state, unavailable_before);
    offers::accept(&mut s, &[request()]).unwrap();
    let purchased = s.state.clone();
    assert!(offers::accept(&mut s, &[request()]).is_err());
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &prepared,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s.state, purchased);
    s.run_months(3).unwrap(); // Fixed repossession returns title to the seller.
    assert_eq!(credit::owner(&s.world, &s.state, asset), Some(STATE_AGENT));
    assert!(credit::discover(&s.world, &s.state, PERSON).is_empty());
    assert!(offers::discover(&s.world, &s.state, PERSON).is_empty());
}

#[test]
fn explicit_purchase_supports_independent_lender_and_checkpoint_continuation() {
    let mut s = sim("repaid");
    s.world.agents.push(Agent {
        id: 7,
        name: "Lender".into(),
    });
    let c = s.world.credit.as_mut().unwrap();
    c.offers[0].loan.creditor = 7;
    c.endowments.push(credit::Endowment {
        agent: 7,
        amount: Amount::new(c.offers[0].loan.denomination, 8000),
    });
    acquire(&mut s);
    let mut checkpoint = s.clone();
    checkpoint.backend = Backend::Reference;
    offers::accept(&mut s, &[request()]).unwrap();
    offers::accept(&mut checkpoint, &[request()]).unwrap();
    s.run_months(5).unwrap();
    for _ in 0..5 {
        checkpoint.run_months(1).unwrap();
    }
    assert_eq!(s.state, checkpoint.state);
    assert_eq!(s.ledger, checkpoint.ledger);
    assert_eq!(s.reports, checkpoint.reports);
    assert_eq!(s.state.credit.loans[&1].creditor, 7);
}
