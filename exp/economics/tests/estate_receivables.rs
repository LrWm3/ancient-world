use economics_compute_smoke::{
    compute::Backend,
    credit::{self, Advance, LoanOffer},
    financial_reporting::Audit,
    model::*,
    recovery::{self, ProceedingTerms, Receipt, Stage},
    recovery_claims,
    scenario::{self, PERSON, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
const BORROWER: AgentId = 97;
const ESTATE: AgentId = 99;
const DEBT: u32 = 10;
const ASSET: u32 = 11;
const HOME: AgentId = 10000;

fn fixture(principal: i32, term: u32, funded: bool) -> Simulation {
    fixture_for(principal, term, funded, false)
}
fn fixture_for(principal: i32, term: u32, funded: bool, household: bool) -> Simulation {
    use economics_compute_smoke::{
        household_governance::Governance,
        households::{self, Agreement},
    };
    let (mut w, mut s) = scenario::baseline();
    w.participants[0].needs.clear();
    w.participants[0].capacity.quantity = 0;
    if !household {
        w.participants.clear();
    }
    w.definitions.clear();
    w.assets.clear();
    w.rights.clear();
    w.agreements.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    for id in [BORROWER, ESTATE] {
        w.agents.push(Agent {
            id,
            name: format!("agent {id}"),
        });
    }
    let debtor = if household {
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
    let loan = |id, creditor, debtor, principal, term| Advance {
        id,
        debtor,
        month: 1,
        principal,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor,
            denomination: TOKEN,
            max_principal: principal,
            monthly_rate_bps: 0,
            term_months: term,
            grace_months: 10,
        },
    };
    w.lending = vec![
        loan(DEBT, STATE_AGENT, debtor, 10, 1),
        loan(ASSET, debtor, BORROWER, principal, term),
    ];
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor,
        estate: ESTATE,
        authority: STATE_AGENT,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 3,
        assets: vec![],
        discharge_deficiency: true,
    });
    s.balances.clear();
    s.balances.insert((STATE_AGENT, TOKEN), 10);
    s.balances.insert((debtor, TOKEN), principal);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    // A disclosed distressed opening snapshot. Reporting begins here; the
    // depletion is not claimed as a transaction or loss produced by this run.
    sim.state.balances.insert((debtor, TOKEN), 0);
    if !funded {
        sim.state.balances.insert((BORROWER, TOKEN), 0);
    }
    if household {
        households::dissolution::request(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
    }
    Simulation::new(sim.world, sim.state, Backend::Reference).unwrap()
}

#[test]
fn household_wind_down_retains_financial_assets_until_collection_and_estate_release() {
    use economics_compute_smoke::households::dissolution;
    let opening = fixture_for(3, 4, true, true);
    let run = |backend| {
        let mut sim = opening.clone();
        sim.backend = backend;
        let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
        until(&mut sim, &mut audit, 4, Phase::Open);
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Active
        );
        assert!(dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).is_err());
        assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
        assert!(
            dissolution::blockers(&sim.world, &sim.state, HOME)
                .contains(&dissolution::Blocker::Loan)
        );
        let checkpoint = (sim.clone(), audit.clone());
        until(&mut sim, &mut audit, 8, Phase::Open);
        assert!(dissolution::blockers(&sim.world, &sim.state, HOME).is_empty());
        dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
        until(&mut sim, &mut audit, 9, Phase::Open);
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 3);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Closed
        );
        let (mut resumed, mut resumed_audit) = checkpoint;
        until(&mut resumed, &mut resumed_audit, 8, Phase::Open);
        dissolution::finish(&mut resumed.world, &resumed.state, HOME, PERSON).unwrap();
        until(&mut resumed, &mut resumed_audit, 9, Phase::Open);
        assert_eq!(
            (&sim.state, &sim.ledger, &audit),
            (&resumed.state, &resumed.ledger, &resumed_audit)
        );
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
fn until(sim: &mut Simulation, audit: &mut Audit, month: u32, phase: Phase) {
    while (sim.state.month, sim.state.phase) != (month, phase) {
        audit.step(sim).unwrap();
    }
}

#[test]
fn estate_waits_for_receivable_collection_and_new_cash_without_rehypothecation() {
    for (principal, term) in [(1, 2), (3, 4)] {
        let opening = fixture(principal, term, true);
        let run = |backend| {
            let mut sim = opening.clone();
            sim.backend = backend;
            let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
            until(&mut sim, &mut audit, 3, Phase::Acquire);
            let case = &sim.state.credit.recovery.proceedings[&1];
            assert_eq!(case.stage, Stage::Active);
            assert!(sim.state.credit.loans[&DEBT].principal > 0);
            let b = sim.ledger.last().unwrap().credit.as_ref().unwrap();
            assert!(
                b.recovery
                    .iter()
                    .any(|r| matches!(r, Receipt::AssetsPending { proceeding: 1, .. }))
            );
            if principal == 1 {
                // The asset is fully collected this Due, but its coin cannot be
                // swept or distributed until subsequent opening windows.
                assert_eq!(
                    sim.state.credit.loans[&ASSET].status,
                    credit::Status::Repaid
                );
                assert_eq!(sim.state.balance(PERSON, TOKEN), 1);
                assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
                assert!(b.recovery.iter().any(|r| matches!(r, Receipt::AssetsPending { receivables, uncollected_cash: 1, .. } if receivables.is_empty())));
            } else {
                assert_eq!(
                    recovery_claims::receivables(&sim.world, &sim.state, PERSON).unwrap()[0]
                        .remaining
                        .quantity,
                    2
                );
                let mut forged = sim.state.clone();
                let case = forged.credit.recovery.proceedings.get_mut(&1).unwrap();
                case.stage = Stage::Closed;
                case.closed = Some(3);
                let loan = forged.credit.loans.get_mut(&DEBT).unwrap();
                loan.principal = 0;
                loan.status = credit::Status::Discharged;
                assert!(Simulation::new(sim.world.clone(), forged, Backend::Reference).is_err());
            }
            let checkpoint = (sim.clone(), audit.clone());
            until(&mut sim, &mut audit, 8, Phase::Open);
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                Stage::Closed
            );
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), principal);
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
            assert_eq!(
                sim.state.credit.loans[&DEBT].status,
                credit::Status::Discharged
            );
            let written_off: i32 = sim
                .ledger
                .iter()
                .filter_map(|b| b.credit.as_ref())
                .flat_map(|b| &b.recovery)
                .filter_map(|r| match r {
                    Receipt::WrittenOff {
                        loan: DEBT,
                        principal,
                        interest,
                        ..
                    } => Some(principal + interest),
                    _ => None,
                })
                .sum();
            assert_eq!(written_off, 10 - principal);
            let (mut resumed, mut resumed_audit) = checkpoint;
            until(&mut resumed, &mut resumed_audit, 8, Phase::Open);
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &resumed_audit)
            );
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn unperformed_receivable_defers_closure_without_inventing_estate_cash() {
    let mut sim = fixture(3, 4, false);
    let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
    until(&mut sim, &mut audit, 8, Phase::Open);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
    assert_eq!(sim.state.credit.loans[&DEBT].principal, 10);
    assert_eq!(
        recovery_claims::receivables(&sim.world, &sim.state, PERSON).unwrap()[0]
            .remaining
            .quantity,
        3
    );
    assert!(sim.ledger.iter().filter_map(|b| b.credit.as_ref()).flat_map(|b| &b.recovery)
        .any(|r| matches!(r, recovery::Receipt::AssetsPending { uncollected_cash: 0, receivables, .. } if !receivables.is_empty())));
}

#[test]
fn counterparty_discharge_resolves_the_asset_before_the_creditor_estate_closes() {
    let mut opening = fixture(3, 4, false);
    opening.world.agents.push(Agent {
        id: 100,
        name: "counterparty estate".into(),
    });
    opening.world.recovery.proceedings.push(ProceedingTerms {
        id: 2,
        debtor: BORROWER,
        estate: 100,
        authority: STATE_AGENT,
        denomination: TOKEN,
        opening_month: 4,
        earliest_close: 4,
        assets: vec![],
        discharge_deficiency: true,
    });
    let run = |backend| {
        let mut sim = opening.clone();
        sim.backend = backend;
        if matches!(backend, Backend::CubeCpu) {
            sim.world.recovery.proceedings.reverse();
        }
        let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
        until(&mut sim, &mut audit, 4, Phase::Acquire);
        assert_eq!(
            sim.state.credit.recovery.proceedings[&2].stage,
            Stage::Closed
        );
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Active
        );
        assert_eq!(
            sim.state.credit.loans[&ASSET].status,
            credit::Status::Discharged
        );
        assert!(
            recovery_claims::receivables(&sim.world, &sim.state, PERSON)
                .unwrap()
                .is_empty()
        );
        until(&mut sim, &mut audit, 5, Phase::Acquire);
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Closed
        );
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
        assert_eq!(
            sim.state.credit.loans[&DEBT].status,
            credit::Status::Discharged
        );
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn receivable_counterparty_filter_explains_deferred_estate_closure() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let mut sim = fixture(3, 4, false);
    let mut observer = Observer::new(
        vec![],
        "receivable-estate",
        Config {
            settlement: true,
            agents: [BORROWER].into(),
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 3).unwrap();
    let log = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert!(
        log.lines()
            .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .any(|r| r["kind"] == "estate_recovery"
                && r["detail"]["event"] == "AssetsPending"
                && r["detail"]["receivables"][0]["debtor"] == BORROWER
                && r["detail"]["receivables"][0]["outstanding"] == 3)
    );
}

#[test]
fn earned_wage_receivables_wait_for_real_payment_but_post_closure_work_is_not_backdated() {
    use economics_compute_smoke::{
        employment::{ArrearsPolicy, Terms},
        finance::ContractId,
    };
    let mut sim = fixture(1, 2, false);
    sim.world.lending.retain(|a| a.id != ASSET);
    sim.state.credit.loans.remove(&ASSET);
    let (baseline, _) = scenario::baseline();
    sim.world.participants = baseline.participants;
    sim.world.participants[0].needs.clear();
    sim.world.participants[0].capacity.quantity = 1;
    let labor = sim.world.participants[0].capacity.resource;
    for (id, month) in [(20, 2), (21, 6)] {
        sim.world.employment.push(Terms {
            id,
            employer: BORROWER,
            worker: PERSON,
            from: month,
            through: month,
            capacity: Amount::new(labor, 1),
            wage_per_unit: Amount::new(TOKEN, 2),
            on_arrears: ArrearsPolicy::Continue,
            rank: 0,
        });
    }
    sim.state.balances.insert((STATE_AGENT, TOKEN), 2);
    sim.world.lending.push(Advance {
        id: 12,
        debtor: BORROWER,
        month: 4,
        principal: 2,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: TOKEN,
            max_principal: 2,
            monthly_rate_bps: 0,
            term_months: 1,
            grace_months: 10,
        },
    });
    let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
    until(&mut sim, &mut audit, 3, Phase::Acquire);
    let assets = recovery_claims::receivables(&sim.world, &sim.state, PERSON).unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].contract, ContractId::Wages(20));
    assert_eq!(assets[0].remaining.quantity, 2);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    until(&mut sim, &mut audit, 5, Phase::Open);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 2);
    assert_eq!(sim.state.employment.earned[&(20, 2)].claim.outstanding(), 0);
    until(&mut sim, &mut audit, 7, Phase::Open);
    assert_eq!(sim.state.credit.recovery.proceedings[&1].closed, Some(6));
    assert_eq!(
        sim.state.credit.loans[&DEBT].status,
        credit::Status::Discharged
    );
    assert_eq!(sim.state.employment.earned[&(21, 6)].claim.outstanding(), 2);
    // Work earned after Due closure is a new asset, not evidence of premature closure.
    Simulation::new(sim.world, sim.state, Backend::CubeCpu).unwrap();
}
