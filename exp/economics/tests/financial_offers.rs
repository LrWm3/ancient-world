use economics_compute_smoke::{
    compute::Backend,
    credit::{self, Advance, LoanOffer},
    financial_reporting::Audit,
    forward::{self, direct},
    household_governance::Governance,
    households::{self, Agreement},
    model::*,
    offers::{self, Id, Request, Terms},
    scenario::{self, GRAIN, PERSON, STATE_AGENT, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};
const HOME: AgentId = 10000;
const LOAN: u32 = 70;
const DELIVERY: u32 = 71;

fn fixture(household: bool, coins: i32) -> (World, State, AgentId) {
    let (mut w, mut s) = scenario::baseline();
    w.assets.clear();
    w.rights.clear();
    w.definitions.clear();
    w.agreements.clear();
    w.participants[0].needs.clear();
    w.participants[0].capacity.quantity = 0;
    w.storage.capacities.insert(PERSON, 100);
    w.storage.weights.insert(GRAIN, 1);
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    s.balances.clear();
    let actor = if household {
        let mut governance = Governance::contributed(PERSON);
        governance.constitution.allow_dissolution = true;
        households::form(
            &mut w,
            &s,
            Agreement {
                id: 1,
                agent: HOME,
                adults: vec![PERSON],
                governance,
                formed: 1,
                dwelling_process: None,
                admission: None,
                membership: vec![],
                asset_sales: vec![],
                equipment_retirements: vec![],
                support: vec![],
            },
        )
        .unwrap();
        HOME
    } else {
        PERSON
    };
    s.balances.insert((STATE_AGENT, TOKEN), coins);
    s.balances.insert((actor, GRAIN), 6);
    w.lending.push(Advance {
        id: LOAN,
        debtor: actor,
        month: 1,
        principal: 3,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: TOKEN,
            max_principal: 3,
            monthly_rate_bps: 0,
            term_months: 2,
            grace_months: 1,
        },
    });
    w.prepaid_deliveries.push(direct::Terms {
        id: DELIVERY,
        seller: actor,
        buyer: STATE_AGENT,
        month: 1,
        due: 2,
        goods: Amount::new(GRAIN, 3),
        prepayment: Amount::new(TOKEN, 2),
    });
    (w, s, actor)
}
fn requests(actor: AgentId) -> Vec<Request> {
    vec![
        Request::new(Id::Advance(LOAN), actor),
        Request::new(Id::PrepaidDelivery(DELIVERY), actor),
    ]
}
fn acquire(sim: &mut Simulation) {
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
}

#[test]
fn loan_and_delivery_offers_share_preparation_household_wrapper_and_continuation() {
    for household in [false, true] {
        let (w, s, actor) = fixture(household, 5);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let found = offers::discover(&w, &s, actor);
            assert!(
                found
                    .iter()
                    .any(|o| matches!(&o.terms, Terms::Advance(a) if a.id == LOAN))
            );
            assert!(
                found
                    .iter()
                    .any(|o| matches!(&o.terms, Terms::PrepaidDelivery(t) if t.id == DELIVERY))
            );
            assert!(offers::discover(&w, &s, STATE_AGENT).is_empty());
            assert!(offers::prepare(&sim, &requests(actor)).is_err());
            let mut audit = Audit::with_inventory(
                &w,
                &s,
                TOKEN,
                Default::default(),
                [((actor, GRAIN), 6)].into(),
            )
            .unwrap();
            while sim.state.phase != Phase::Acquire {
                audit.step(&mut sim).unwrap();
            }
            let opening = sim.clone();
            let prepared = offers::prepare(&sim, &requests(actor)).unwrap();
            assert_eq!(sim.state, opening.state);
            assert_eq!(sim.ledger, opening.ledger);
            assert_eq!(prepared.household.is_some(), household);
            let mut reversed = requests(actor);
            reversed.reverse();
            assert_eq!(offers::prepare(&sim, &reversed).unwrap(), prepared);
            let mut accepted = sim.clone();
            offers::accept(&mut accepted, &requests(actor)).unwrap();
            audit.step(&mut sim).unwrap();
            assert_eq!(sim.state, accepted.state);
            assert_eq!(sim.ledger, accepted.ledger);
            assert_eq!(sim.state.balance(actor, TOKEN), 5);
            assert_eq!(sim.ledger.last(), Some(&prepared));
            assert!(offers::discover(&w, &sim.state, actor).is_empty());
            let mut checkpoint = (sim.clone(), audit.clone());
            while sim.state.month <= 3 {
                audit.step(&mut sim).unwrap();
            }
            while checkpoint.0.state.month <= 3 {
                checkpoint.1.step(&mut checkpoint.0).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&checkpoint.0.state, &checkpoint.0.ledger, &checkpoint.1)
            );
            assert_eq!(sim.state.credit.loans[&LOAN].status, credit::Status::Repaid);
            assert_eq!(sim.state.exchange.forwards[&DELIVERY].delivered, 3);
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn failed_bundle_leaves_live_state_untouched_but_normal_execution_keeps_partial_outcomes() {
    let (w, s, actor) = fixture(true, 4);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    acquire(&mut sim);
    let before = sim.clone();
    assert!(offers::accept(&mut sim, &requests(actor)).is_err());
    assert_eq!(sim.state, before.state);
    assert_eq!(sim.ledger, before.ledger);
    let loan = [Request::new(Id::Advance(LOAN), actor)];
    let prepared = offers::prepare(&sim, &loan).unwrap();
    assert!(prepared.transactions.iter().any(|t| matches!(
        &t.forward,
        Some(forward::Event::AdmissionRejected {
            contract: DELIVERY,
            reason: direct::Rejection::FundingShortfall
        })
    )));
    let mut forged = prepared.clone();
    forged
        .credit
        .as_mut()
        .unwrap()
        .after
        .loans
        .get_mut(&LOAN)
        .unwrap()
        .principal += 1;
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &forged,
            sim.backend,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(sim.state, before.state);
    offers::accept(&mut sim, &loan).unwrap();
    assert_eq!(sim.state.credit.loans[&LOAN].principal, 3);
    assert!(sim.state.exchange.forwards.is_empty());
    assert!(offers::accept(&mut sim, &loan).is_err());
}

#[test]
fn requests_cannot_change_party_date_or_reuse_same_boundary_loan_proceeds() {
    let (mut w, s, actor) = fixture(false, 5);
    w.prepaid_deliveries[0].buyer = actor;
    w.prepaid_deliveries[0].seller = STATE_AGENT;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    acquire(&mut sim);
    let loan = Request::new(Id::Advance(LOAN), actor);
    for bad in [
        vec![Request::new(Id::Advance(LOAN), STATE_AGENT)],
        vec![loan.clone(), loan.clone()],
        vec![Request {
            need: Some(GRAIN),
            ..loan.clone()
        }],
        vec![
            loan.clone(),
            Request::new(Id::PrepaidDelivery(DELIVERY), STATE_AGENT),
        ],
    ] {
        assert!(offers::prepare(&sim, &bad).is_err());
    }
    sim.step().unwrap();
    assert_eq!(sim.state.balance(actor, TOKEN), 3);
    assert!(sim.state.exchange.forwards.is_empty());
    assert!(
        sim.ledger
            .last()
            .unwrap()
            .transactions
            .iter()
            .any(|t| matches!(
                &t.forward,
                Some(forward::Event::AdmissionRejected {
                    reason: direct::Rejection::FundingShortfall,
                    ..
                })
            ))
    );
}

#[test]
fn scheduled_loan_consent_remains_a_commitment_during_household_wind_down() {
    let (mut w, mut s, _) = fixture(true, 5);
    w.prepaid_deliveries.clear();
    s.month = 2;
    w.lending[0].month = 3;
    households::dissolution::request(&mut w, &s, HOME, PERSON).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert!(
        households::dissolution::blockers(&sim.world, &sim.state, HOME)
            .contains(&households::dissolution::Blocker::Loan)
    );
    acquire(&mut sim);
    let prepared = offers::prepare(&sim, &[Request::new(Id::Advance(LOAN), HOME)]).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.ledger.last(), Some(&prepared));
    assert_eq!(sim.state.credit.loans[&LOAN].principal, 3);
    sim.run_months(3).unwrap();
    assert_eq!(sim.state.credit.loans[&LOAN].status, credit::Status::Repaid);
    assert!(
        !households::dissolution::blockers(&sim.world, &sim.state, HOME)
            .contains(&households::dissolution::Blocker::Loan)
    );
}
