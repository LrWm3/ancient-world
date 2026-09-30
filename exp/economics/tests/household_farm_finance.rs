use economics_compute_smoke::{
    borrowing, commitments,
    compute::Backend,
    credit,
    dues_accounting::Valuation,
    financial_reporting::{Audit, Opening},
    forward::direct::Terms,
    household_governance::Governance,
    households::{self, Agreement},
    model::*,
    process_accounting::{BeneficiaryPolicy, Costs, Output},
    scenario::*,
    simulation::Simulation,
    stock_sale,
};
const HOME: AgentId = 10000;
const RENTED: AssetId = 999;

#[test]
fn joint_household_forecasts_preserve_other_members_work_and_individual_need_limits() {
    use economics_compute_smoke::{joint_plan, settlement};
    const PEER: AgentId = 89;
    let (mut w, mut s) = fixture(true, true);
    w.horizon = 12;
    for d in &mut w.definitions {
        if d.execution == Execution::Productive && d.id != GROW {
            d.enabled = false;
        }
    }
    let (warmth, _) = with_warmth(false);
    for r in warmth.resources {
        if !w.resources.iter().any(|existing| existing.id == r.id) {
            w.resources.push(r);
        }
    }
    for d in warmth
        .definitions
        .into_iter()
        .filter(|d| [PREPARE_FUEL, USE_FUEL].contains(&d.id))
    {
        w.definitions.retain(|existing| existing.id != d.id);
        w.definitions.push(d);
    }
    w.agents.push(Agent {
        id: PEER,
        name: "fuel worker".into(),
    });
    let mut peer = w.participants[0].clone();
    peer.agent = PEER;
    peer.needs = vec![Requirement {
        resource: WARMTH,
        quantity: 1,
        priority: 0,
    }];
    w.participants.push(peer);
    w.storage.capacities.insert(PEER, 100);
    w.storage.weights.extend([(RAW_WOOD, 1), (FUEL, 1)]);
    w.households[0].adults.push(PEER);
    // Household work cannot substitute for an unavailable fuel worker in the control.
    w.households[0].governance.constitution.activities = Some([GROW].into());
    s.balances.insert((PEER, RAW_WOOD), 24);
    s.balances.insert((PERSON, SEED), 0);
    s.balances.insert((HOME, SEED), 1);
    let policy = w.credit.as_mut().unwrap().stock_sales.as_mut().unwrap();
    policy.forecast = None;
    policy.joint = Some(joint_plan::Policy {
        horizon_months: 12,
        need_limits: [(NUTRITION, 0), (WARMTH, 0)].into(),
        future_reserves: vec![6],
    });
    // Isolate joint work from the existing private bridge-funding shortfall.
    w.credit
        .as_mut()
        .unwrap()
        .endowments
        .iter_mut()
        .find(|e| e.agent == PERSON)
        .unwrap()
        .amount
        .quantity = 20_000;
    w.households[0].admission =
        Some(economics_compute_smoke::laws::households::admit(&w, &s, &w.households[0]).unwrap());
    let run = |backend, unavailable: bool, reordered: bool| {
        let mut world = w.clone();
        if unavailable {
            world
                .participants
                .iter_mut()
                .find(|p| p.agent == PEER)
                .unwrap()
                .capacity
                .quantity = 0;
        }
        if reordered {
            world.participants.reverse();
            world.definitions.reverse();
        }
        let mut sim = Simulation::new(world, s.clone(), backend).unwrap();
        let mut audit = opening(&sim.world, &sim.state);
        while sim.state.phase != Phase::Acquire {
            audit.step(&mut sim).unwrap();
        }
        let before = sim.state.clone();
        audit.step(&mut sim).unwrap();
        let accepted = sim.ledger.last().unwrap();
        let decision = accepted
            .credit
            .as_ref()
            .unwrap()
            .stock_sale
            .as_ref()
            .unwrap()
            .joint
            .as_ref()
            .unwrap();
        assert_eq!(decision.feasible, !unavailable, "decision={decision:?}");
        for alternative in &decision.alternatives {
            let deficit = alternative.participant_deficits[&PEER][&WARMTH];
            assert_eq!(deficit > 0, unavailable, "alternative={alternative:?}");
            if unavailable {
                assert!(!alternative.admissible);
            }
        }
        assert!(
            decision
                .alternatives
                .iter()
                .any(|a| a.work == joint_plan::Work::Wait)
        );
        assert!(
            decision
                .alternatives
                .iter()
                .any(|a| a.work == joint_plan::Work::Produce(GROW))
        );
        let mut tampered = accepted.clone();
        tampered
            .credit
            .as_mut()
            .unwrap()
            .stock_sale
            .as_mut()
            .unwrap()
            .joint
            .as_mut()
            .unwrap()
            .alternatives[0]
            .participant_deficits
            .clear();
        let mut unchanged = before.clone();
        assert!(
            settlement::commit(
                &sim.world,
                &mut unchanged,
                &tampered,
                backend,
                sim.effect_limit
            )
            .is_err()
        );
        assert_eq!(unchanged, before);
        let plan = sim.state.pending_production.clone().unwrap();
        assert!(plan.household.is_some());
        let prefix = sim.ledger.len();
        let mut resumed_audit = audit.clone();
        let mut resumed = Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
        audit.step(&mut sim).unwrap();
        assert_eq!(sim.ledger.last().unwrap(), plan.as_ref());
        assert_eq!(
            sim.state
                .processes
                .values()
                .any(|p| p.operator == PEER && p.definition == PREPARE_FUEL),
            !unavailable
        );
        while sim.state.month < 3 {
            audit.step(&mut sim).unwrap();
        }
        while resumed.state.month < 3 {
            resumed_audit.step(&mut resumed).unwrap();
        }
        assert_eq!(
            (&sim.state, &sim.ledger[prefix..], &audit),
            (&resumed.state, &resumed.ledger[..], &resumed_audit)
        );
        for p in &sim.world.participants {
            assert!(sim.state.balance(p.agent, LABOR) >= 0);
        }
        (sim.state, sim.ledger, audit)
    };
    for unavailable in [false, true] {
        let reference = run(Backend::Reference, unavailable, false);
        assert_eq!(reference, run(Backend::CubeCpu, unavailable, false));
        assert_eq!(reference, run(Backend::Reference, unavailable, true));
    }
}

fn fixture(household: bool, sales: bool) -> (World, State) {
    let (mut w, s) = stock_sale::scenario("funded").unwrap();
    // This test concerns performance of accepted commitments, not underwriting.
    let c = w.credit.as_mut().unwrap();
    c.purchase_policy = borrowing::Policy::Scripted;
    c.stock_sales.as_mut().unwrap().reserve_months = 0;
    c.stock_sales.as_mut().unwrap().forecast = Some(economics_compute_smoke::sale_plan::Policy {
        horizon_months: 6,
        need_limits: [(NUTRITION, 0)].into(),
    });
    if !sales {
        c.stock_sales.as_mut().unwrap().purchase_budget = 0;
    }
    w.definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .outputs
        .iter_mut()
        .find(|a| a.resource == GRAIN)
        .unwrap()
        .quantity = 20;
    if household {
        households::form(
            &mut w,
            &s,
            Agreement {
                id: 1,
                agent: HOME,
                adults: vec![PERSON],
                governance: Governance::contributed(PERSON),
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
    }
    w.assets.push(Asset {
        id: RENTED,
        owner: STATE_AGENT,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: 99,
        holder: PERSON,
        asset: RENTED,
        from: 1,
        through: 36,
        output_owner: PERSON,
    });
    w.agreements.push(commitments::Agreement {
        id: 99,
        right: 99,
        creditor: STATE_AGENT,
        debtor: PERSON,
        activated: 1,
        payment: Amount::new(GRAIN, 2),
    });
    for (id, month) in [(5, 7), (6, 15)] {
        w.prepaid_deliveries.push(Terms {
            id,
            seller: PERSON,
            buyer: STATE_AGENT,
            month,
            due: month + 1,
            goods: Amount::new(GRAIN, 2),
            prepayment: Amount::new(TOKEN, 600),
        });
    }
    (w, s)
}

fn opening(w: &World, s: &State) -> Audit {
    let mut output_weights = std::collections::BTreeMap::from([(
        GROW,
        [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
    )]);
    if w.definitions.iter().any(|d| d.id == PREPARE_FUEL) {
        output_weights.insert(PREPARE_FUEL, [(Output::Stock(FUEL), 1)].into());
    }
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            assets: [(PLOT, 10000), (RENTED, 10000)].into(),
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| [SEED, GRAIN, RAW_WOOD, FUEL].contains(r) && **q > 0)
                .map(|(account, q)| (*account, i128::from(*q)))
                .collect(),
            exchange_values: [SEED, GRAIN, RAW_WOOD, FUEL]
                .into_iter()
                .filter(|id| w.resources.iter().any(|r| r.id == *id))
                .map(|id| (id, 1))
                .collect(),
            dues: Some(Valuation([(99, 1)].into())),
            processes: Some(Costs {
                beneficiary_policy: Some(BeneficiaryPolicy::TransferAtCost),
                output_weights,
                ..Costs::default()
            }),
            ..Opening::default()
        },
    )
    .unwrap()
}

#[test]
fn repeated_household_harvests_service_mortgage_rent_and_forwards_through_actual_sales() {
    for household in [false, true] {
        for sales in [false, true] {
            let (w, s) = fixture(household, sales);
            let run = |backend| {
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                let mut audit = opening(&w, &s);
                while sim.state.month < 8 {
                    audit.step(&mut sim).unwrap();
                }
                let mut resumed =
                    Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
                let mut ra = audit.clone();
                let prefix = sim.ledger.len();
                while sim.state.month <= 24 {
                    audit.step(&mut sim).unwrap();
                }
                while resumed.state.month <= 24 {
                    ra.step(&mut resumed).unwrap();
                }
                assert_eq!(
                    (&sim.state, &sim.ledger[prefix..], &audit),
                    (&resumed.state, &resumed.ledger[..], &ra)
                );
                if sales {
                    assert_eq!(credit::owner(&w, &sim.state, PLOT), Some(PERSON));
                    assert_eq!(sim.state.credit.loans[&1].status, credit::Status::Repaid);
                    assert_eq!(sim.state.obligations[&(99, 13)].paid, 2);
                    for id in [5, 6] {
                        assert_eq!(sim.state.exchange.forwards[&id].delivered, 2);
                    }
                    let seeds: i32 = sim
                        .state
                        .balances
                        .iter()
                        .filter(|((_, r), _)| *r == SEED)
                        .map(|(_, q)| *q)
                        .sum();
                    let planted = sim
                        .state
                        .processes
                        .values()
                        .filter(|p| p.definition == GROW && p.status == Status::Active)
                        .count() as i32;
                    assert_eq!(seeds + planted, 1);
                    assert_eq!(sim.state.credit.loans[&1].debtor, PERSON);
                    // Collective balances never silently assume the member's loan.
                    assert!(
                        sim.ledger
                            .iter()
                            .filter_map(|b| b.household.as_ref())
                            .flat_map(|h| &h.reservations)
                            .all(|r| r.allocated == 0
                                || !matches!(
                                    r.request.purpose,
                                    households::Purpose::LoanSupport { .. }
                                ))
                    );
                    assert!(
                        sim.reports
                            .iter()
                            .filter(|r| r.agent == PERSON)
                            .all(|r| r.deficit(NUTRITION) == 0)
                    );
                    assert!(
                        sim.state
                            .processes
                            .values()
                            .filter(|p| p.definition == GROW && p.status == Status::Completed)
                            .count()
                            >= 2
                    );
                } else {
                    assert_eq!(sim.state.credit.stock_spent, 0);
                    assert_ne!(credit::owner(&w, &sim.state, PLOT), Some(PERSON));
                }
                for agent in &w.agents {
                    let report = audit.book().statements(agent.id, 1, 24).unwrap();
                    assert_eq!(report.assets, report.liabilities + report.equity);
                }
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}

#[test]
fn fixed_food_buffer_can_pay_financial_claims_while_missing_a_meal() {
    let (mut w, s) = fixture(false, true);
    let policy = w.credit.as_mut().unwrap().stock_sales.as_mut().unwrap();
    policy.forecast = None;
    policy.reserve_months = 6;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(24).unwrap();
    assert_eq!(sim.state.credit.loans[&1].status, credit::Status::Repaid);
    assert_eq!(sim.state.obligations[&(99, 13)].paid, 2);
    for id in [5, 6] {
        assert_eq!(sim.state.exchange.forwards[&id].delivered, 2);
    }
    assert!(
        sim.reports
            .iter()
            .filter(|r| r.agent == PERSON)
            .any(|r| r.deficit(NUTRITION) > 0)
    );
}

#[test]
fn joint_work_plan_observes_lease_and_prepaid_performance_before_reserving_production() {
    use economics_compute_smoke::{joint_plan, settlement};
    for transfers_seed in [false, true] {
        let (mut w, s) = fixture(false, true);
        if transfers_seed {
            w.prepaid_deliveries[0].month = 6;
            w.prepaid_deliveries[0].due = 7;
            w.prepaid_deliveries[0].goods = Amount::new(SEED, 1);
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut audit = opening(&w, &s);
            while (sim.state.month, sim.state.phase) != (7, Phase::Acquire) {
                audit.step(&mut sim).unwrap();
            }
            let policy = sim
                .world
                .credit
                .as_mut()
                .unwrap()
                .stock_sales
                .as_mut()
                .unwrap();
            policy.forecast = None;
            policy.joint = Some(joint_plan::Policy {
                horizon_months: 12,
                need_limits: [(NUTRITION, 0)].into(),
                future_reserves: vec![6],
            });
            settlement::validate_world(&sim.world, &sim.state).unwrap();
            let before = sim.state.clone();
            let mut resumed = Simulation::new(sim.world.clone(), before.clone(), backend).unwrap();
            let mut ra = audit.clone();
            let prefix = sim.ledger.len();
            audit.step(&mut sim).unwrap();
            assert!(sim.state.exchange.forwards.contains_key(&5));
            let accepted = sim.ledger.last().unwrap();
            let decision = accepted
                .credit
                .as_ref()
                .unwrap()
                .stock_sale
                .as_ref()
                .unwrap()
                .joint
                .as_ref()
                .unwrap();
            assert_eq!(decision.feasible, !transfers_seed);
            let plan = accepted.production_plan.as_ref().unwrap().clone();
            assert_eq!(sim.state.pending_production, Some(plan.clone()));
            assert_eq!((plan.month, plan.phase), (7, Phase::Productive));
            if transfers_seed {
                assert_eq!(sim.state.balance(PERSON, SEED), 0);
                assert_eq!(sim.state.exchange.forwards[&5].delivered, 1);
                assert!(
                    !plan
                        .transactions
                        .iter()
                        .filter_map(|t| t.process.as_ref())
                        .any(|p| p.before.is_none() && p.after.definition == GROW)
                );
            }
            let mut altered = accepted.clone();
            altered.production_plan.as_mut().unwrap().month += 1;
            let mut rejected = before.clone();
            assert!(
                settlement::commit(
                    &sim.world,
                    &mut rejected,
                    &altered,
                    backend,
                    sim.effect_limit
                )
                .is_err()
            );
            assert_eq!(rejected, before);
            audit.step(&mut sim).unwrap();
            assert_eq!(sim.ledger.last().unwrap(), plan.as_ref());
            while sim.state.month < 9 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month < 9 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                sim.state.exchange.forwards[&5].delivered,
                if transfers_seed { 1 } else { 2 }
            );
            assert!(
                sim.reports
                    .iter()
                    .filter(|r| r.agent == PERSON)
                    .all(|r| r.deficit(NUTRITION) == 0)
            );
            assert_eq!(
                (&sim.state, &sim.ledger[prefix..], &audit),
                (&resumed.state, &resumed.ledger[..], &ra)
            );
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn household_joint_plan_reserves_shared_seed_and_replays_allocation_once() {
    use economics_compute_smoke::{joint_plan, settlement};
    let (mut w, mut s) = fixture(true, true);
    w.horizon = 12;
    for d in &mut w.definitions {
        if d.execution == Execution::Productive && d.id != GROW {
            d.enabled = false;
        }
    }
    s.balances.insert((PERSON, SEED), 0);
    s.balances.insert((HOME, SEED), 1);
    let policy = w.credit.as_mut().unwrap().stock_sales.as_mut().unwrap();
    policy.forecast = None;
    policy.joint = Some(joint_plan::Policy {
        horizon_months: 12,
        need_limits: [(NUTRITION, 0)].into(),
        future_reserves: vec![6],
    });
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut audit = opening(&w, &s);
        while sim.state.phase != Phase::Acquire {
            audit.step(&mut sim).unwrap();
        }
        audit.step(&mut sim).unwrap();
        let plan = sim.state.pending_production.clone().unwrap();
        let receipt = plan.household.as_ref().unwrap();
        assert!(
            receipt
                .reservations
                .iter()
                .any(|r| r.request.member == PERSON
                    && r.request.resource == SEED
                    && r.allocated == 1),
            "receipt={receipt:?}, balances={:?}, plan={plan:?}",
            sim.state.balances
        );
        assert!(!receipt.labor.is_empty());
        assert_eq!(sim.state.balance(HOME, SEED), 1);
        assert_eq!(sim.state.balance(PERSON, SEED), 0);
        let before = sim.state.clone();
        for missing_envelope in [false, true] {
            let mut invalid = before.clone();
            let plan = invalid.pending_production.as_mut().unwrap();
            if missing_envelope {
                plan.household = None;
            } else {
                plan.month += 1;
            }
            assert!(Simulation::new(sim.world.clone(), invalid, backend).is_err());
        }
        let mut altered = *plan.clone();
        altered.household.as_mut().unwrap().before.clear();
        let mut rejected = before.clone();
        assert!(
            settlement::commit(
                &sim.world,
                &mut rejected,
                &altered,
                backend,
                sim.effect_limit
            )
            .is_err()
        );
        assert_eq!(rejected, before);
        let mut resumed = Simulation::new(sim.world.clone(), before, backend).unwrap();
        let mut ra = audit.clone();
        let prefix = sim.ledger.len();
        audit.step(&mut sim).unwrap();
        ra.step(&mut resumed).unwrap();
        assert_eq!(sim.ledger.last().unwrap(), plan.as_ref());
        assert_eq!(sim.state.balance(HOME, SEED), 0);
        assert_eq!(sim.state.balance(PERSON, SEED), 0);
        assert!(
            sim.state
                .processes
                .values()
                .any(|p| p.definition == GROW && p.status == Status::Active)
        );
        assert!(sim.state.pending_production.is_none());
        while sim.state.month < 7 {
            audit.step(&mut sim).unwrap();
        }
        while resumed.state.month < 7 {
            ra.step(&mut resumed).unwrap();
        }
        assert_eq!(
            (&sim.state, &sim.ledger[prefix..], &audit),
            (&resumed.state, &resumed.ledger[..], &ra)
        );
        assert!(
            sim.ledger
                .iter()
                .filter_map(|b| b.household.as_ref())
                .any(|h| h
                    .after
                    .iter()
                    .any(|e| e.account == (HOME, GRAIN) && e.delta > 0))
        );
        assert!(
            sim.state
                .processes
                .values()
                .any(|p| p.definition == GROW && p.status == Status::Completed)
        );
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    // A wish to start the same crop cannot create absent household seed.
    s.balances.insert((HOME, SEED), 0);
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let mut control = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        while control.state.phase != Phase::Productive {
            control.step().unwrap();
        }
        control.step().unwrap();
        assert!(
            !control
                .state
                .processes
                .values()
                .any(|p| p.definition == GROW)
        );
        assert_eq!(control.state.balance(HOME, SEED), 0);
        assert_eq!(control.state.balance(PERSON, SEED), 0);
    }
}
