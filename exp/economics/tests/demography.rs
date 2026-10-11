use economics_compute_smoke::{
    compute::Backend,
    demography::{self, scenario::*},
    financial_reporting::{Audit, Opening},
    model::*,
    scenario::{GRAIN, LABOR, NUTRITION, PERSON, TOKEN},
    simulation::Simulation,
};
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| {
                    *r != TOKEN
                        && **q > 0
                        && w.resources
                            .iter()
                            .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
                })
                .map(|(a, q)| (*a, i128::from(*q)))
                .collect(),
            services: Some(Default::default()),
            processes: Some(Default::default()),
            ..Default::default()
        },
    )
    .unwrap()
}
#[test]
fn reproduction_has_finite_support_child_needs_and_an_exact_maturity_boundary() {
    let (mut w, s) = scenario(true, false);
    w.demography.as_mut().unwrap().max_births = 2;
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        a.step(&mut sim).unwrap();
        assert_eq!(sim.state.demography.births.len(), 2);
        assert_eq!(sim.state.balance(PERSON, GRAIN), 4);
        assert_eq!(sim.state.balance(FIRST_CHILD, GRAIN), 2);
        assert_eq!(sim.state.balance(FIRST_CHILD, LABOR), 0);
        assert!(!economics_compute_smoke::opportunities::permits(
            &sim.world,
            &sim.state,
            FIRST_CHILD,
            economics_compute_smoke::opportunities::Action::Process(FORAGE)
        ));
        assert!(!economics_compute_smoke::opportunities::permits(
            &sim.world,
            &sim.state,
            FIRST_CHILD,
            economics_compute_smoke::opportunities::Action::FoundHousehold
        ));
        assert!(sim.state.memberships.is_empty());
        while sim.state.month < 7 {
            a.step(&mut sim).unwrap();
        }
        assert!(
            sim.reports
                .iter()
                .filter(|r| r.agent >= FIRST_CHILD)
                .all(|r| r.needs[&NUTRITION].desired == 1)
        );
        assert!(
            sim.ledger
                .iter()
                .filter(|b| b.phase == Phase::Open)
                .all(|b| b
                    .transactions
                    .iter()
                    .flat_map(|t| &t.effects)
                    .all(|e| e.account != (FIRST_CHILD, LABOR) || e.delta <= 0))
        );
        a.step(&mut sim).unwrap();
        assert_eq!(sim.state.balance(FIRST_CHILD, LABOR), 1);
        assert!(sim.state.demography.matured.contains(&FIRST_CHILD));
        assert!(
            sim.ledger
                .last()
                .unwrap()
                .demography
                .as_ref()
                .unwrap()
                .support
                .is_empty()
        );
        for _ in 0..12 {
            a.step(&mut sim).unwrap();
        }
        (sim.world, sim.state, sim.ledger, sim.reports, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
#[test]
fn population_pressure_runs_with_conserved_support_and_independent_books() {
    for scarce in [false, true] {
        for reproduce in [false, true] {
            let (w, s) = scenario(reproduce, scarce);
            let run = |backend| {
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                let mut a = audit(&w, &s);
                while sim.state.month <= MONTHS {
                    a.step(&mut sim).unwrap();
                }
                let deficits: i32 = sim.reports.iter().map(|r| r.deficit(NUTRITION)).sum();
                let expected = match (scarce, reproduce) {
                    (_, false) => (0, 0, 0),
                    (false, true) => (12, 0, 14),
                    (true, true) => (10, 7, 59),
                };
                assert_eq!(
                    (
                        sim.state.demography.births.len(),
                        sim.state.terminal.len(),
                        deficits
                    ),
                    expected
                );
                for b in &sim.ledger {
                    if let Some(d) = &b.demography {
                        assert_eq!(
                            d.transactions
                                .iter()
                                .flat_map(|t| &t.effects)
                                .map(|e| i64::from(e.delta))
                                .sum::<i64>(),
                            0
                        );
                    }
                }
                for p in &sim.world.participants {
                    let st = a.book().statements(p.agent, 1, MONTHS).unwrap();
                    assert_eq!(st.assets, st.liabilities + st.equity);
                }
                (sim.world, sim.state, sim.ledger, sim.reports, a)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}
#[test]
fn lifecycle_replay_checkpoint_and_batch_size_preserve_dynamic_catalogs() {
    let (w, s) = scenario(true, true);
    let mut direct = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    direct.run_months(12).unwrap();
    let mut monthly = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    for _ in 0..12 {
        monthly.run_months(1).unwrap();
    }
    assert_eq!(direct.world, monthly.world);
    assert_eq!(direct.state, monthly.state);
    assert_eq!(direct.ledger, monthly.ledger);
    let (mut rw, mut rs) = (w.clone(), s.clone());
    for batch in &direct.ledger {
        demography::replay(
            &mut rw,
            &mut rs,
            batch,
            Backend::CubeCpu,
            economics_compute_smoke::settlement::DEFAULT_EFFECT_LIMIT,
        )
        .unwrap();
    }
    assert_eq!(rw, direct.world);
    assert_eq!(rs, direct.state);
    let mut resumed = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while resumed.state.month <= 12 {
        let ledger = resumed.ledger.clone();
        let reports = resumed.reports.clone();
        resumed = Simulation::new(resumed.world, resumed.state, Backend::CubeCpu).unwrap();
        resumed.ledger = ledger;
        resumed.reports = reports;
        resumed.step().unwrap();
    }
    assert_eq!(resumed.world, direct.world);
    assert_eq!(resumed.state, direct.state);
    assert_eq!(resumed.ledger, direct.ledger);
}

#[test]
fn unfunded_births_and_cooldowns_do_not_create_people() {
    let (mut w, mut s) = scenario(true, false);
    w.demography.as_mut().unwrap().parents = [PERSON].into();
    s.balances.insert((PERSON, GRAIN), 2); // one reserve + two birth units is unaffordable
    let mut sim = Simulation::new(w.clone(), s, Backend::Reference).unwrap();
    sim.step().unwrap();
    assert!(sim.state.demography.births.is_empty());
    let (_, s) = scenario(true, false);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(6).unwrap();
    assert_eq!(sim.state.demography.births.len(), 1);
    sim.state.balances.insert((PERSON, GRAIN), 5); // isolate cooldown from current funding
    sim.step().unwrap();
    assert_eq!(sim.state.demography.births.len(), 2);
    assert_eq!(sim.state.demography.last_birth[&PERSON], 7);
}
#[test]
fn unsupported_children_have_real_deprivation_and_do_not_revive_at_maturity() {
    let (mut w, s) = scenario(true, false);
    let c = w.demography.as_mut().unwrap();
    c.max_births = 2;
    c.maturity_months = 12;
    w.definitions
        .iter_mut()
        .find(|d| d.id == FORAGE)
        .unwrap()
        .enabled = false;
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut a = audit(&w, &s);
    while sim.state.month <= 18 {
        a.step(&mut sim).unwrap();
    }
    assert_eq!(sim.state.demography.births.len(), 2);
    assert_eq!(sim.state.terminal.len(), 4);
    assert!(!sim.state.demography.matured.contains(&FIRST_CHILD));
    assert_eq!(sim.state.balance(FIRST_CHILD, LABOR), 0);
    assert!(
        sim.ledger
            .iter()
            .filter_map(|b| b.demography.as_ref())
            .flat_map(|d| &d.support)
            .any(|r| r.child == FIRST_CHILD && r.requested > r.delivered)
    );
}
#[test]
fn forged_lifecycle_and_support_are_atomic_and_catalog_collisions_do_not_publish() {
    use economics_compute_smoke::settlement::{DEFAULT_EFFECT_LIMIT, commit};
    let (w, s) = scenario(true, false);
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    sim.step().unwrap();
    let good = sim.ledger[0].clone();
    for case in 0..5 {
        let mut bad = good.clone();
        match case {
            0 => bad.demography = None,
            1 => bad.demography.as_mut().unwrap().births[0].parent = PERSON + 1,
            2 => {
                bad.demography
                    .as_mut()
                    .unwrap()
                    .after
                    .matured
                    .insert(FIRST_CHILD);
            }
            3 => {
                bad.transactions.pop();
            }
            _ => {
                bad.transactions
                    .push(bad.transactions.last().unwrap().clone());
            }
        };
        let mut before = s.clone();
        assert!(
            commit(
                &sim.world,
                &mut before,
                &bad,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(before, s);
    }
    let mut forged_world = sim.world.clone();
    forged_world
        .participants
        .iter_mut()
        .find(|p| p.agent == FIRST_CHILD)
        .unwrap()
        .capacity
        .quantity = 1;
    let mut before = s.clone();
    assert!(
        commit(
            &forged_world,
            &mut before,
            &good,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, s);
    let mut collision = w;
    collision.agents.push(Agent {
        id: FIRST_CHILD,
        name: "occupied".into(),
    });
    let mut sim = Simulation::new(collision.clone(), s.clone(), Backend::Reference).unwrap();
    assert!(sim.step().is_err());
    assert_eq!(sim.world, collision);
    assert_eq!(sim.state, s);
    assert!(sim.ledger.is_empty());
}
#[test]
fn forecast_preserves_known_dependents_but_does_not_invent_future_births() {
    let (w, s) = scenario(true, false);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    let before = sim.clone();
    let (w, s) = economics_compute_smoke::forecast::ForecastContext::new(&sim.world, &sim.state)
        .into_parts();
    let mut forecast = Simulation::new(w, s, Backend::Reference).unwrap();
    forecast.run_months(12).unwrap();
    assert_eq!(forecast.state.demography.births.len(), 2);
    assert_eq!(forecast.state.demography.matured.len(), 2);
    assert_eq!(sim.world, before.world);
    assert_eq!(sim.state, before.state);
}

#[test]
fn checkpoint_rejects_forged_birth_history_and_templates() {
    let (w, s) = scenario(true, false);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    for case in 0..5 {
        let mut bad = sim.state.clone();
        match case {
            0 => bad.demography.last_birth.clear(),
            1 => {
                bad.demography.matured.insert(FIRST_CHILD);
            }
            2 => {
                let b = bad.demography.births.remove(&FIRST_CHILD).unwrap();
                bad.demography.births.insert(FIRST_CHILD + 99, b);
            }
            3 => bad.demography.births.get_mut(&FIRST_CHILD).unwrap().month += 1,
            _ => bad.demography.births.get_mut(&FIRST_CHILD).unwrap().parent = FIRST_CHILD + 1,
        }
        assert!(
            Simulation::new(sim.world.clone(), bad, Backend::Reference).is_err(),
            "case {case}"
        );
    }
    let mut bad = sim.world.clone();
    bad.storage.capacities.insert(FIRST_CHILD, 1000);
    assert!(Simulation::new(bad, sim.state, Backend::Reference).is_err());
    let (mut bad, s) = scenario(true, false);
    bad.demography.as_mut().unwrap().person.storage = 1;
    assert!(Simulation::new(bad, s, Backend::Reference).is_err());
}
#[test]
fn birth_priority_is_independent_of_catalog_iteration_order_and_zero_service_recipes_do_not_employ_children()
 {
    let (w, s) = scenario(true, false);
    let mut reverse = w.clone();
    reverse.agents.reverse();
    reverse.participants.reverse();
    let mut first = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut second = Simulation::new(reverse, s.clone(), Backend::CubeCpu).unwrap();
    first.step().unwrap();
    second.step().unwrap();
    assert_eq!(first.state, second.state);
    assert_eq!(first.ledger[0].demography, second.ledger[0].demography);
    let mut w = w;
    // One unit is below the two-month planning buffer, so children would have
    // reason to select this recipe if zero service costs bypassed the age gate.
    w.demography.as_mut().unwrap().birth_buffer_months = 1;
    w.definitions
        .iter_mut()
        .find(|d| d.id == FORAGE)
        .unwrap()
        .stages[0]
        .monthly_services
        .clear();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert!(!sim.ledger.iter().flat_map(|b| &b.transactions).any(|t| {
        t.process
            .as_ref()
            .is_some_and(|p| p.after.operator >= FIRST_CHILD && p.after.definition == FORAGE)
    }));
}
#[test]
fn funded_loans_and_dependent_support_share_the_existing_accounting_and_execution_model() {
    use economics_compute_smoke::{
        credit::{Advance, LoanOffer},
        scenario::STATE_AGENT,
    };
    let (mut w, mut s) = scenario(true, false);
    w.demography.as_mut().unwrap().max_births = 2;
    w.lending.push(Advance {
        id: 1,
        debtor: PERSON,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: TOKEN,
            max_principal: 4,
            monthly_rate_bps: 0,
            term_months: 2,
            grace_months: 1,
        },
        principal: 4,
        month: 1,
        collateral: None,
        priority: 0,
    });
    s.balances.insert((STATE_AGENT, TOKEN), 4);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        while sim.state.month <= 4 {
            a.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.demography.births.len(), 2);
        assert_eq!(
            sim.state.credit.loans[&1].status,
            economics_compute_smoke::credit::Status::Repaid
        );
        (sim.world, sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
#[test]
fn household_residual_release_funds_birth_at_the_same_opening_boundary_and_replays() {
    use economics_compute_smoke::{
        household_governance::Governance,
        households::{self, Agreement, dissolution},
    };
    const HOME: AgentId = 10000;
    let (mut w, mut s) = scenario(true, false);
    w.demography.as_mut().unwrap().parents = [PERSON].into();
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
    s.month = 2;
    s.balances.insert((PERSON, GRAIN), 0);
    s.balances.insert((HOME, GRAIN), 4);
    dissolution::request(&mut w, &s, HOME, PERSON).unwrap();
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        a.step(&mut sim).unwrap();
        assert_eq!(sim.state.demography.births.len(), 1);
        assert_eq!(sim.state.balance(HOME, GRAIN), 0);
        assert_eq!(sim.state.balance(PERSON, GRAIN), 2);
        assert_eq!(sim.state.balance(FIRST_CHILD, GRAIN), 2);
        assert_eq!(
            households::membership::current(&sim.world.households[0]),
            vec![PERSON]
        );
        let (mut rw, mut rs) = (w.clone(), s.clone());
        demography::replay(
            &mut rw,
            &mut rs,
            &sim.ledger[0],
            backend,
            economics_compute_smoke::settlement::DEFAULT_EFFECT_LIMIT,
        )
        .unwrap();
        assert_eq!(rw, sim.world);
        assert_eq!(rs, sim.state);
        (sim.world, sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn opening_credit_outflows_cannot_be_spent_again_on_birth_support() {
    use economics_compute_smoke::{borrowing, credit, finance, scenario::STATE_AGENT};
    for outgoing in [0, 2] {
        let (mut w, mut s) = scenario(true, false);
        let (credit_world, _) = credit::scenario("default").unwrap();
        w.assets = credit_world.assets;
        let mut c = credit_world.credit.unwrap();
        c.purchase_policy = borrowing::Policy::Decline;
        c.endowments.clear();
        if outgoing > 0 {
            c.transfers.push(credit::ScheduledTransfer {
                month: 2,
                transfer: finance::Transfer {
                    from: PERSON,
                    to: STATE_AGENT,
                    amount: Amount::new(TOKEN, outgoing),
                },
            });
        }
        w.credit = Some(c);
        let config = w.demography.as_mut().unwrap();
        config.parents = [PERSON].into();
        config.support = Amount::new(TOKEN, 1);
        s.month = 2;
        s.balances.insert((PERSON, TOKEN), 3);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            sim.step().unwrap();
            assert_eq!(
                sim.state.demography.births.len(),
                usize::from(outgoing == 0)
            );
            assert_eq!(sim.state.balance(PERSON, TOKEN), 1);
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), outgoing);
            (sim.world, sim.state, sim.ledger)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn support_aware_planning_produces_before_the_next_open() {
    let run = |backend, enabled, reserve| {
        let (mut w, mut s) = scenario(true, false);
        let c = w.demography.as_mut().unwrap();
        c.parents = [PERSON].into();
        c.max_births = 1;
        c.birth_buffer_months = 1;
        c.plan_support = enabled;
        c.parent_reserve = reserve;
        s.balances.insert((PERSON, GRAIN), 2 + reserve);
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, backend).unwrap();
        while sim.state.month <= 2 {
            a.step(&mut sim).unwrap();
        }
        let unmet: i32 = sim
            .reports
            .iter()
            .filter(|r| r.agent == FIRST_CHILD)
            .map(|r| r.deficit(NUTRITION))
            .sum();
        assert_eq!(unmet, if enabled { 0 } else { 1 });
        (sim.world, sim.state, sim.ledger, sim.reports, a)
    };
    for (enabled, reserve) in [(false, 1), (true, 1), (false, 2), (true, 2)] {
        assert_eq!(
            run(Backend::Reference, enabled, reserve),
            run(Backend::CubeCpu, enabled, reserve)
        );
    }
}

#[test]
fn alternative_recipes_do_not_disable_support_planning() {
    const BERRIES: ResourceId = 950;
    let run = |backend, alternative| {
        let (mut w, mut s) = scenario(true, false);
        let c = w.demography.as_mut().unwrap();
        c.parents = [PERSON].into();
        c.max_births = 1;
        c.birth_buffer_months = 1;
        c.plan_support = true;
        s.balances.insert((PERSON, GRAIN), 3);
        if alternative {
            w.resources.push(Resource {
                id: BERRIES,
                name: "berries".into(),
                kind: ResourceKind::Stock,
            });
            let mut recipe = w
                .definitions
                .iter()
                .find(|d| d.execution == Execution::Consumption)
                .unwrap()
                .clone();
            recipe.id = BERRIES;
            recipe.stages[0].entry_inputs = vec![Amount::new(BERRIES, 1)];
            recipe.outputs[0].quantity = 2;
            w.definitions.push(recipe);
        }
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, backend).unwrap();
        a.step(&mut sim).unwrap();
        if alternative {
            let mut stocked = sim.state.clone();
            stocked.balances.insert((FIRST_CHILD, BERRIES), 1);
            let asks = demography::provision::requests(&sim.world, &stocked, PERSON, GRAIN, 2);
            assert!(
                asks.is_empty(),
                "one alternative meal preserves the child's grain for next month"
            );
        }
        while sim.state.month <= 2 {
            a.step(&mut sim).unwrap();
        }
        assert_eq!(
            sim.reports
                .iter()
                .filter(|r| r.agent == FIRST_CHILD)
                .map(|r| r.deficit(NUTRITION))
                .sum::<i32>(),
            0
        );
        (sim.state, sim.ledger, sim.reports, a)
    };
    for alternative in [false, true] {
        assert_eq!(
            run(Backend::Reference, alternative),
            run(Backend::CubeCpu, alternative)
        );
    }
}

#[test]
fn consumption_preserves_the_same_support_stock_as_the_planner() {
    let run = |backend| {
        const BERRIES: ResourceId = 950;
        let (mut w, mut s) = scenario(true, false);
        let c = w.demography.as_mut().unwrap();
        c.parents = [PERSON].into();
        c.max_births = 1;
        c.birth_buffer_months = 1;
        c.plan_support = true;
        c.parent_reserve = 0;
        s.balances.insert((PERSON, GRAIN), 2);
        s.balances.insert((PERSON, BERRIES), 2);
        w.resources.push(Resource {
            id: BERRIES,
            name: "berries".into(),
            kind: ResourceKind::Stock,
        });
        let meal = w
            .definitions
            .iter_mut()
            .find(|d| d.execution == Execution::Consumption)
            .unwrap();
        meal.outputs[0].quantity = 2;
        let mut berries = meal.clone();
        berries.id = BERRIES;
        berries.stages[0].entry_inputs = vec![Amount::new(BERRIES, 1)];
        berries.outputs[0].quantity = 1;
        w.definitions.push(berries);
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, backend).unwrap();
        while sim.state.month == 1 {
            a.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.balance(PERSON, GRAIN), 1);
        a.step(&mut sim).unwrap();
        assert_eq!(sim.state.balance(FIRST_CHILD, GRAIN), 1);
        assert_eq!(
            sim.ledger
                .last()
                .unwrap()
                .demography
                .as_ref()
                .unwrap()
                .support[0]
                .delivered,
            1
        );
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn support_only_production_does_not_invent_personal_needs() {
    let run = |backend, enabled, work| {
        let (mut w, mut s) = scenario(true, false);
        let c = w.demography.as_mut().unwrap();
        c.parents = [PERSON].into();
        c.max_births = 1;
        c.birth_buffer_months = 1;
        c.plan_support = enabled;
        w.participants
            .iter_mut()
            .find(|p| p.agent == PERSON)
            .unwrap()
            .needs[0]
            .quantity = 0;
        w.condition_rules.retain(|r| r.subject != PERSON);
        w.definitions
            .iter_mut()
            .find(|d| d.id == FORAGE)
            .unwrap()
            .enabled = work;
        let mut costly = w
            .definitions
            .iter()
            .find(|d| d.id == FORAGE)
            .unwrap()
            .clone();
        costly.id = FORAGE - 1;
        costly.stages[0].monthly_services[0].quantity = 2;
        w.pool_inputs
            .push(economics_compute_smoke::pools::PoolInput {
                definition: costly.id,
                account: (economics_compute_smoke::scenario::STATE_AGENT, WILD_FOOD),
            });
        w.definitions.push(costly);
        s.balances.insert((PERSON, GRAIN), 2);
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, backend).unwrap();
        while sim.state.month <= 2 {
            a.step(&mut sim).unwrap();
        }
        let deficit: i32 = sim
            .reports
            .iter()
            .filter(|r| r.agent == FIRST_CHILD)
            .map(|r| r.deficit(NUTRITION))
            .sum();
        assert_eq!(deficit, if enabled && work { 0 } else { 1 });
        assert!(
            sim.reports
                .iter()
                .filter(|r| r.agent == PERSON)
                .all(|r| r.needs[&NUTRITION].desired == 0)
        );
        assert!(
            sim.state
                .processes
                .values()
                .filter(|p| p.operator == PERSON)
                .all(|p| p.goal.is_none())
        );
        (sim.state, sim.ledger, sim.reports, a)
    };
    for (enabled, work) in [(true, true), (false, true), (true, false)] {
        assert_eq!(
            run(Backend::Reference, enabled, work),
            run(Backend::CubeCpu, enabled, work)
        );
    }
}
