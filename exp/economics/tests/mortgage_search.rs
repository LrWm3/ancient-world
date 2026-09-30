use economics_compute_smoke::{
    compute::Backend,
    credit,
    financial_reporting::{Audit, Opening},
    model::*,
    offers::{self, Id, Request},
    process_accounting::{Costs, Output},
    scenario::*,
    settlement,
    simulation::Simulation,
};

#[test]
fn financed_property_and_planting_share_acceptance_and_later_crop_control() {
    for downpayment in [false, true] {
        for seed in [false, true] {
            for maintain in [false, true] {
                let (mut w, mut s) = credit::crop_scenario(maintain).unwrap();
                w.scheduled_starts.clear();
                // Opening funds are explicit; subsequent mortgage servicing supplies
                // no transfers to rescue this deliberately defaulting borrower.
                w.credit.as_mut().unwrap().endowments.clear();
                s.balances
                    .insert((PERSON, TOKEN), if downpayment { 2000 } else { 0 });
                s.balances.insert((STATE_AGENT, TOKEN), 100000);
                s.balances.insert((PERSON, SEED), i32::from(seed));
                let run = |backend| {
                    let mut audit = Audit::with_opening(
                        &w,
                        &s,
                        TOKEN,
                        Opening {
                            assets: [(PLOT, 10000)].into(),
                            inventory: if seed {
                                [((PERSON, SEED), 1)].into()
                            } else {
                                Default::default()
                            },
                            exchange_values: [(SEED, 1), (GRAIN, 1)].into(),
                            processes: Some(Costs {
                                output_weights: [(
                                    GROW,
                                    [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
                                )]
                                .into(),
                                ..Costs::default()
                            }),
                            ..Opening::default()
                        },
                    )
                    .unwrap();
                    let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                    while sim.state.phase != Phase::Acquire {
                        audit.step(&mut sim).unwrap();
                    }
                    let requests = [
                        Request::new(Id::FinancedPurchase(1), PERSON),
                        Request::new(Id::Process(GROW), PERSON),
                    ];
                    let before = sim.state.clone();
                    let prepared = offers::prepare(&sim, &requests);
                    assert_eq!(sim.state, before);
                    if !downpayment || !seed {
                        assert!(prepared.is_err());
                        assert!(offers::accept(&mut sim, &requests).is_err());
                        assert_eq!(sim.state, before);
                        assert_eq!(
                            credit::owner(&sim.world, &sim.state, PLOT),
                            Some(STATE_AGENT)
                        );
                        assert!(sim.state.credit.loans.is_empty());
                        return (sim.state, sim.ledger, audit);
                    }
                    let prepared = prepared.unwrap();
                    let mut altered = prepared.clone();
                    altered
                        .credit
                        .as_mut()
                        .unwrap()
                        .after
                        .owners
                        .insert(PLOT, STATE_AGENT);
                    let mut unchanged = before.clone();
                    assert!(
                        settlement::commit(
                            &sim.world,
                            &mut unchanged,
                            &altered,
                            backend,
                            sim.effect_limit
                        )
                        .is_err()
                    );
                    assert_eq!(unchanged, before);
                    offers::accept(&mut sim, &requests).unwrap();
                    audit
                        .record(&sim.world, &before, &prepared, &sim.state)
                        .unwrap();
                    assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(PERSON));
                    assert_eq!(sim.state.credit.loans[&1].principal, 8000);
                    assert!(sim.state.processes.is_empty());
                    audit.step(&mut sim).unwrap();
                    assert_eq!(sim.state.balance(PERSON, SEED), 0);
                    assert_eq!(sim.state.processes.len(), 1);
                    let mut resumed =
                        Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
                    let mut resumed_audit = audit.clone();
                    while sim.state.month < 7 {
                        audit.step(&mut sim).unwrap();
                    }
                    while resumed.state.month < 7 {
                        resumed_audit.step(&mut resumed).unwrap();
                    }
                    assert_eq!((&sim.state, &audit), (&resumed.state, &resumed_audit));
                    assert_eq!(
                        credit::owner(&sim.world, &sim.state, PLOT),
                        Some(STATE_AGENT)
                    );
                    assert_eq!(sim.state.credit.loans[&1].principal, 2160);
                    let crop = sim.state.processes.values().next().unwrap();
                    assert_eq!(crop.operator, STATE_AGENT);
                    assert_eq!(
                        crop.status,
                        if maintain {
                            Status::Completed
                        } else {
                            Status::Aborted
                        }
                    );
                    assert_eq!(
                        sim.state.balance(STATE_AGENT, GRAIN),
                        if maintain { 8 } else { 0 }
                    );
                    (sim.state, sim.ledger, audit)
                };
                assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
            }
        }
    }
}
