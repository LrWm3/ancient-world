use economics_compute_smoke::{
    accounting::Account as A,
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::Audit,
    household_governance::Governance,
    households::{self, Agreement, dissolution},
    model::*,
    recovery::{Bid, Listing, ProceedingTerms, Stage},
    scenario::*,
    simulation::Simulation,
};
const HOME: AgentId = 10000;
const BUYER: AgentId = 97;
const ESTATE: AgentId = 98;
fn fixture() -> (World, State) {
    let (mut w, mut s) = baseline();
    w.assets.clear();
    w.rights.clear();
    w.definitions.clear();
    w.participants[0].needs.clear();
    s.balances.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    for id in [BUYER, ESTATE] {
        w.agents.push(Agent {
            id,
            name: format!("agent {id}"),
        });
    }
    let mut governance = Governance::contributed(PERSON);
    governance.constitution.allow_dissolution = true;
    households::form(
        &mut w,
        &s,
        Agreement {
            id: 1,
            agent: HOME,
            governance,
            adults: vec![PERSON],
            membership: vec![],
            asset_sales: vec![],
            equipment_retirements: vec![],
            formed: 1,
            dwelling_process: None,
            admission: None,
        },
    )
    .unwrap();
    for (id, creditor) in [(1, STATE_AGENT), (2, PERSON)] {
        w.lending.push(Advance {
            id,
            debtor: HOME,
            terms: LoanOffer {
                creditor,
                denomination: TOKEN,
                max_principal: 10,
                monthly_rate_bps: 0,
                term_months: 1,
                grace_months: 10,
            },
            principal: 10,
            month: 1,
            priority: id,
            collateral: None,
        });
        s.balances.insert((creditor, TOKEN), 10);
    }
    s.balances.insert((BUYER, TOKEN), 8);
    (w, s)
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
#[test]
fn household_and_member_loans_remain_separate_and_repay_before_residuals_on_cpu() {
    let (w, s) = fixture();
    let run = |backend| {
        let mut a =
            Audit::with_inventory(&w, &s, TOKEN, Default::default(), Default::default()).unwrap();
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        through(&mut a, &mut sim, 1);
        assert_eq!(sim.state.balance(HOME, TOKEN), 20);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
        assert_eq!(a.book().statements(HOME, 1, 1).unwrap().liabilities, 20);
        assert_eq!(a.book().statements(PERSON, 1, 1).unwrap().assets, 10);
        dissolution::request(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
        let checkpoint = (sim.clone(), a.clone());
        through(&mut a, &mut sim, 2);
        assert_eq!(sim.state.balance(HOME, TOKEN), 0);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 10);
        dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
        through(&mut a, &mut sim, 3);
        let (mut resumed, mut ra) = checkpoint;
        through(&mut ra, &mut resumed, 2);
        dissolution::finish(&mut resumed.world, &resumed.state, HOME, PERSON).unwrap();
        through(&mut ra, &mut resumed, 3);
        assert_eq!(sim.state, resumed.state);
        assert_eq!(sim.ledger, resumed.ledger);
        assert_eq!(a, ra);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
fn distressed(discharge: bool) -> (World, State) {
    let (mut w, s) = fixture();
    w.assets.push(Asset {
        id: PLOT,
        owner: HOME,
        kind: 1,
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: HOME,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 4,
        assets: vec![Listing {
            asset: PLOT,
            minimum_price: 8,
        }],
        discharge_deficiency: discharge,
    });
    w.recovery.bids.push(Bid {
        id: 1,
        proceeding: 1,
        buyer: BUYER,
        asset: PLOT,
        month: 3,
        price: 8,
    });
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    // Controlled pre-audit loss sets the opening distressed balance sheet.
    sim.state.balances.insert((HOME, TOKEN), 0);
    sim.state.balances.insert((HOME, GRAIN), 2);
    dissolution::request(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
    (sim.world, sim.state)
}
#[test]
fn household_liquidation_respects_priority_custody_and_authorized_deficiency_on_cpu() {
    for discharge in [false, true] {
        let (w, s) = distressed(discharge);
        let run = |backend| {
            let mut a = Audit::with_inventory(
                &w,
                &s,
                TOKEN,
                [(PLOT, 6)].into(),
                [((HOME, GRAIN), 4)].into(),
            )
            .unwrap();
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            through(&mut a, &mut sim, 3);
            assert_eq!(sim.state.credit.owners[&PLOT], BUYER);
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 8);
            assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
            let checkpoint = (sim.clone(), a.clone());
            through(&mut a, &mut sim, 4);
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 8);
            assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                Stage::Closed
            );
            if discharge {
                through(&mut a, &mut sim, 5);
                assert_eq!(sim.state.balance(PERSON, GRAIN), 2);
                dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
                through(&mut a, &mut sim, 6);
                let report = a.book().statements(HOME, 2, 6).unwrap();
                assert_eq!(report.assets, 0);
                assert_eq!(report.liabilities, 0);
                assert_eq!(report.income[&A::DisposalGain], 2);
            } else {
                assert_eq!(
                    sim.state.credit.loans[&1].principal + sim.state.credit.loans[&2].principal,
                    12
                );
                through(&mut a, &mut sim, 5);
                assert_eq!(sim.state.balance(HOME, GRAIN), 2);
                assert!(dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).is_err());
            }
            let (mut resumed, mut ra) = checkpoint;
            through(&mut ra, &mut resumed, 5);
            if discharge {
                dissolution::finish(&mut resumed.world, &resumed.state, HOME, PERSON).unwrap();
                through(&mut ra, &mut resumed, 6);
            }
            assert_eq!(sim.state, resumed.state);
            assert_eq!(sim.ledger, resumed.ledger);
            assert_eq!(a, ra);
            for id in [HOME, PERSON, STATE_AGENT, ESTATE, BUYER] {
                let r = a.book().statements(id, 2, sim.state.month - 1).unwrap();
                assert_eq!(r.assets, r.liabilities + r.equity);
            }
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn household_recovery_requires_wind_down_and_independent_custody() {
    let (w, s) = distressed(true);
    for estate in [HOME, PERSON] {
        let mut bad = w.clone();
        bad.recovery.proceedings[0].estate = estate;
        assert!(Simulation::new(bad, s.clone(), Backend::Reference).is_err());
    }
    let mut active = w;
    active.households[0].membership.clear();
    let mut opening = s;
    opening.month = 3;
    let err = Simulation::new(active, opening, Backend::Reference)
        .err()
        .unwrap();
    assert!(err.contains("wind-down"), "{err}");
}
