use economics_compute_smoke::{
    accounting::{Account as A, Flow},
    activities::DurableKind,
    compute::Backend,
    equipment::{DurableAsset, Offer},
    exchange::Delivery,
    financial_reporting::Audit,
    household_governance::Governance,
    households::{self, Agreement, disposal, dissolution},
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
    telemetry::{Config, Observer},
};

const HOME: AgentId = 10000;
const KIND: u32 = 99;

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
    w.activities.kinds.insert(
        KIND,
        DurableKind {
            name: "portable tool".into(),
            lifetime: 12,
            attached: false,
            monthly_decay: 1,
        },
    );
    s.equipment.insert(
        TOOL,
        DurableAsset {
            id: TOOL,
            owner: HOME,
            attached_to: None,
            kind: KIND,
            remaining_uses: 6,
            last_used_month: Some(1),
        },
    );
    s.month = 2;
    s.balances.insert((HOME, TOKEN), 5);
    s.balances.insert((HOME, GRAIN), 2);
    s.balances.insert((STATE_AGENT, TOKEN), 20);
    dissolution::request(&mut w, &s, HOME, PERSON).unwrap();
    (w, s)
}
fn terms(price: i32) -> disposal::Sale {
    disposal::Sale {
        attachments: vec![],
        month: 2,
        asset: TOOL,
        buyer: STATE_AGENT,
        price: Amount::new(TOKEN, price),
    }
}
fn accept(w: &mut World, s: &State, price: i32) {
    disposal::accept(w, s, HOME, PERSON, terms(price)).unwrap();
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_inventory(
        w,
        s,
        TOKEN,
        [(TOOL, 12)].into(),
        [((HOME, GRAIN), 4)].into(),
    )
    .unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}

#[test]
fn equipment_sale_preserves_condition_then_charges_buyer_decay_with_balanced_books() {
    for price in [9, 15] {
        let (mut w, s) = fixture();
        accept(&mut w, &s, price);
        let run = |backend| {
            let mut a = audit(&w, &s);
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            a.step(&mut sim).unwrap();
            let tool = &sim.state.equipment[&TOOL];
            assert_eq!(tool.owner, STATE_AGENT);
            assert_eq!(tool.remaining_uses, 5); // transfer then one monthly decay
            assert_eq!(tool.last_used_month, Some(1));
            assert_eq!(tool.kind, KIND);
            assert_eq!(tool.attached_to, None);
            assert!(sim.state.credit.owners.is_empty());
            assert!(sim.state.credit.values.is_empty());
            assert_eq!(sim.state.balance(HOME, TOKEN), 5 + price);
            assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
            let receipt = &sim.ledger[0].household.as_ref().unwrap().disposals[0];
            assert_eq!(receipt.equipment.as_ref(), Some(&s.equipment[&TOOL]));
            assert_eq!(receipt.rejection, None);
            let buyer = a.book().statements(STATE_AGENT, 2, 2).unwrap();
            assert_eq!(
                buyer.trial_balance[&A::Tangible(TOOL)],
                i128::from(price - price / 6)
            );
            assert_eq!(buyer.expenses[&A::Depreciation], i128::from(price / 6));
            assert_eq!(buyer.cash_flows[&Flow::Investing], -i128::from(price));
            let seller = a.book().statements(HOME, 2, 2).unwrap();
            assert_eq!(
                seller.expenses.get(&A::Depreciation).copied().unwrap_or(0),
                0
            );
            if price > 12 {
                assert_eq!(seller.income[&A::DisposalGain], i128::from(price - 12));
            } else {
                assert_eq!(seller.expenses[&A::DisposalLoss], i128::from(12 - price));
            }
            let checkpoint = (sim.clone(), a.clone());
            through(&mut a, &mut sim, 3);
            assert_eq!(sim.state.balance(PERSON, TOKEN), 5 + price);
            dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
            through(&mut a, &mut sim, 4);
            a.finalize_through(4).unwrap();
            let (mut resumed, mut resumed_audit) = checkpoint;
            through(&mut resumed_audit, &mut resumed, 3);
            dissolution::finish(&mut resumed.world, &resumed.state, HOME, PERSON).unwrap();
            through(&mut resumed_audit, &mut resumed, 4);
            resumed_audit.finalize_through(4).unwrap();
            assert_eq!(sim.state, resumed.state);
            assert_eq!(sim.ledger, resumed.ledger);
            assert_eq!(a, resumed_audit);
            for who in [HOME, PERSON, STATE_AGENT] {
                let report = a.book().finalized_statements(who, 2, 4).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
                if who == HOME {
                    assert_eq!(report.assets, 0);
                    assert_eq!(report.expenses[&A::TransferExpense], i128::from(9 + price));
                }
            }
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn scheduled_decay_can_exhaust_bought_equipment_without_resetting_its_life() {
    let (mut w, mut s) = fixture();
    s.equipment.get_mut(&TOOL).unwrap().remaining_uses = 1;
    accept(&mut w, &s, 9);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    a.step(&mut sim).unwrap();
    assert_eq!(sim.state.equipment[&TOOL].owner, STATE_AGENT);
    assert_eq!(sim.state.equipment[&TOOL].remaining_uses, 0);
    let buyer = a.book().statements(STATE_AGENT, 2, 2).unwrap();
    assert_eq!(buyer.expenses[&A::Depreciation], 9);
    assert_eq!(
        buyer
            .trial_balance
            .get(&A::Tangible(TOOL))
            .copied()
            .unwrap_or(0),
        0
    );
}

#[test]
fn unfilled_offers_and_delivery_agreements_block_disposal_admission() {
    for delivery in [false, true] {
        let (mut w, mut s) = fixture();
        if delivery {
            s.exchange.contracts.insert(
                TOOL,
                Delivery {
                    purchase: None,
                    asset: TOOL,
                    provider: STATE_AGENT,
                    buyer: HOME,
                    capture_percent: 25,
                },
            );
        } else {
            w.offers.push(Offer {
                id: 1,
                seller: HOME,
                asset: TOOL,
                price: Amount::new(TOKEN, 9),
            });
        }
        let before = w.clone();
        assert!(disposal::accept(&mut w, &s, HOME, PERSON, terms(9)).is_err());
        assert_eq!(w, before);
    }
}

#[test]
fn equipment_condition_and_new_contracts_are_rechecked_at_execution() {
    for case in 0..4 {
        let (mut w, mut s) = fixture();
        accept(&mut w, &s, 9);
        let expected = match case {
            0 => {
                s.equipment.get_mut(&TOOL).unwrap().remaining_uses = 0;
                disposal::Rejection::Unavailable
            }
            1 => {
                s.equipment.get_mut(&TOOL).unwrap().last_used_month = Some(2);
                disposal::Rejection::Unavailable
            }
            2 => {
                s.equipment.get_mut(&TOOL).unwrap().owner = PERSON;
                disposal::Rejection::Unavailable
            }
            _ => {
                w.offers.push(Offer {
                    id: 1,
                    seller: HOME,
                    asset: TOOL,
                    price: Amount::new(TOKEN, 9),
                });
                disposal::Rejection::Encumbered
            }
        };
        let owner = s.equipment[&TOOL].owner;
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.step().unwrap();
        assert_eq!(sim.state.equipment[&TOOL].owner, owner);
        assert_eq!(sim.state.balance(HOME, TOKEN), 5);
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 20);
        assert_eq!(
            sim.ledger[0].household.as_ref().unwrap().disposals[0].rejection,
            Some(expected)
        );
    }
}

#[test]
fn attached_equipment_and_its_plot_cannot_be_sold_independently() {
    for asset in [PLOT, TOOL] {
        let (mut w, mut s) = fixture();
        w.assets.push(Asset {
            id: PLOT,
            owner: HOME,
            kind: 1,
        });
        w.activities.kinds.get_mut(&KIND).unwrap().attached = true;
        s.equipment.get_mut(&TOOL).unwrap().attached_to = Some(PLOT);
        let mut sale = terms(9);
        sale.asset = asset;
        let before = w.clone();
        assert!(disposal::accept(&mut w, &s, HOME, PERSON, sale).is_err());
        assert_eq!(w, before);
    }
}

#[test]
fn equipment_and_catalog_sales_share_the_buyers_finite_opening_budget() {
    let (mut w, mut s) = fixture();
    w.assets.push(Asset {
        id: PLOT,
        owner: HOME,
        kind: 1,
    });
    s.balances.insert((STATE_AGENT, TOKEN), 10);
    let mut sale = terms(8);
    sale.asset = PLOT;
    disposal::accept(&mut w, &s, HOME, PERSON, sale).unwrap();
    accept(&mut w, &s, 8);
    let run = |mut w: World, reverse: bool| {
        if reverse {
            w.households[0].asset_sales.reverse();
        }
        let mut sim = Simulation::new(w, s.clone(), Backend::Reference).unwrap();
        sim.step().unwrap();
        let receipts = &sim.ledger[0].household.as_ref().unwrap().disposals;
        assert_eq!(receipts.iter().filter(|r| r.rejection.is_none()).count(), 1);
        assert_eq!(
            receipts[1].rejection,
            Some(disposal::Rejection::FundingOrStorage)
        );
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 2);
        assert_eq!(sim.state.balance(HOME, TOKEN), 13);
        (sim.state, sim.ledger)
    };
    assert_eq!(run(w.clone(), false), run(w, true));
}

#[test]
fn forged_equipment_condition_is_rejected_and_valid_sale_replays_once() {
    let (mut w, s) = fixture();
    accept(&mut w, &s, 9);
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    sim.step().unwrap();
    let mut replay = s.clone();
    commit(
        &w,
        &mut replay,
        &sim.ledger[0],
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    assert_eq!(replay, sim.state);
    let unchanged = replay.clone();
    assert!(
        commit(
            &w,
            &mut replay,
            &sim.ledger[0],
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, unchanged);
    for omit in [false, true] {
        let mut forged = sim.ledger[0].clone();
        let receipt = &mut forged.household.as_mut().unwrap().disposals[0];
        if omit {
            receipt.equipment = None;
        } else {
            receipt.equipment.as_mut().unwrap().remaining_uses += 1;
        }
        let mut untouched = s.clone();
        assert!(
            commit(
                &w,
                &mut untouched,
                &forged,
                Backend::CubeCpu,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(untouched, s);
    }
}

#[test]
fn observer_reports_opening_equipment_condition_without_affecting_wear() {
    let (mut w, s) = fixture();
    accept(&mut w, &s, 9);
    let mut observed = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut plain = observed.clone();
    let mut observer = Observer::new(
        vec![],
        "equipment-disposal",
        Config {
            settlement: true,
            agents: [STATE_AGENT].into(),
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut observed, 1).unwrap();
    plain.run_months(1).unwrap();
    assert_eq!(observed.state, plain.state);
    assert_eq!(observed.ledger, plain.ledger);
    let log = String::from_utf8(observer.finish().unwrap()).unwrap();
    let rows: Vec<serde_json::Value> = log
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|r: &serde_json::Value| r["kind"] == "household_asset_disposal")
        .collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["settled"], true);
    assert_eq!(rows[0]["equipment"]["remaining_uses"], 6);
    assert_eq!(rows[0]["equipment"]["last_used_month"], 1);
    assert_eq!(observed.state.equipment[&TOOL].remaining_uses, 5);
}

#[test]
fn exhausted_equipment_retires_after_depreciation_then_household_closes_on_cpu() {
    use households::retirement;
    let (w, mut s) = fixture();
    s.equipment.get_mut(&TOOL).unwrap().remaining_uses = 1;
    let run = |backend| {
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        assert!(retirement::request(&mut sim.world, &sim.state, HOME, PERSON, TOOL).is_err());
        through(&mut a, &mut sim, 2);
        assert_eq!(sim.state.equipment[&TOOL].remaining_uses, 0);
        assert_eq!(sim.state.balance(HOME, TOKEN), 5);
        assert!(dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).is_err());
        retirement::request(&mut sim.world, &sim.state, HOME, PERSON, TOOL).unwrap();
        let opening_tool = sim.state.equipment[&TOOL].clone();
        let batch = sim.state.next_batch;
        a.step(&mut sim).unwrap();
        assert!(!sim.state.equipment.contains_key(&TOOL));
        let retired = &sim.state.retired_equipment[&TOOL];
        assert_eq!(retired.equipment, opening_tool);
        assert_eq!(retired.month, 3);
        assert_eq!(retired.batch, batch);
        assert_eq!(sim.state.balance(HOME, TOKEN), 5);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
        let checkpoint = (sim.clone(), a.clone());
        through(&mut a, &mut sim, 4);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 5);
        dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
        through(&mut a, &mut sim, 5);
        a.finalize_through(5).unwrap();
        let (mut resumed, mut resumed_audit) = checkpoint;
        through(&mut resumed_audit, &mut resumed, 4);
        dissolution::finish(&mut resumed.world, &resumed.state, HOME, PERSON).unwrap();
        through(&mut resumed_audit, &mut resumed, 5);
        resumed_audit.finalize_through(5).unwrap();
        assert_eq!(sim.state, resumed.state);
        assert_eq!(sim.ledger, resumed.ledger);
        assert_eq!(a, resumed_audit);
        let report = a.book().finalized_statements(HOME, 2, 5).unwrap();
        assert_eq!(report.assets, 0);
        assert_eq!(report.liabilities, 0);
        assert_eq!(report.expenses[&A::Depreciation], 12);
        assert_eq!(report.expenses[&A::TransferExpense], 9);
        assert_eq!(
            report.expenses.get(&A::DisposalLoss).copied().unwrap_or(0),
            0
        );
        assert_eq!(
            report
                .cash_flows
                .get(&Flow::Investing)
                .copied()
                .unwrap_or(0),
            0
        );
        assert_eq!(sim.state.retired_equipment[&TOOL].equipment, opening_tool);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

fn retirement_fixture() -> (World, State) {
    let (mut w, mut s) = fixture();
    s.equipment.get_mut(&TOOL).unwrap().remaining_uses = 0;
    households::retirement::request(&mut w, &s, HOME, PERSON, TOOL).unwrap();
    (w, s)
}

#[test]
fn retirement_requires_authority_open_and_exhausted_uncommitted_property() {
    use households::retirement;
    let (mut w, mut s) = retirement_fixture();
    let before = w.clone();
    assert!(retirement::request(&mut w, &s, HOME, PERSON, TOOL).is_err()); // duplicate
    assert!(retirement::request(&mut w, &s, HOME, STATE_AGENT, TOOL).is_err());
    assert!(retirement::request(&mut w, &s, HOME, PERSON, u32::MAX).is_err());
    s.phase = Phase::Productive;
    assert!(retirement::request(&mut w, &s, HOME, PERSON, TOOL).is_err());
    assert_eq!(w, before);
    for attached in [true, false] {
        let (mut w, mut s) = fixture();
        s.equipment.get_mut(&TOOL).unwrap().remaining_uses = 0;
        if attached {
            w.assets.push(Asset {
                id: PLOT,
                owner: HOME,
                kind: 1,
            });
            s.equipment.get_mut(&TOOL).unwrap().attached_to = Some(PLOT);
            w.activities.kinds.get_mut(&KIND).unwrap().attached = true;
        } else {
            w.offers.push(Offer {
                id: 1,
                seller: HOME,
                asset: TOOL,
                price: Amount::new(TOKEN, 9),
            });
        }
        let before = w.clone();
        assert!(retirement::request(&mut w, &s, HOME, PERSON, TOOL).is_err());
        assert_eq!(w, before);
    }
}

#[test]
fn repaired_reassigned_or_committed_equipment_is_not_retired_by_stale_instructions() {
    for case in 0..4 {
        let (mut w, mut s) = retirement_fixture();
        let reason = match case {
            0 => {
                s.equipment.get_mut(&TOOL).unwrap().remaining_uses = 2;
                disposal::Rejection::NotExhausted
            }
            1 => {
                s.equipment.get_mut(&TOOL).unwrap().owner = PERSON;
                disposal::Rejection::Unavailable
            }
            2 => {
                s.equipment.get_mut(&TOOL).unwrap().last_used_month = Some(s.month);
                disposal::Rejection::Unavailable
            }
            _ => {
                w.offers.push(Offer {
                    id: 1,
                    seller: HOME,
                    asset: TOOL,
                    price: Amount::new(TOKEN, 9),
                });
                disposal::Rejection::Encumbered
            }
        };
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.step().unwrap();
        assert!(sim.state.retired_equipment.is_empty());
        assert!(sim.state.equipment.contains_key(&TOOL));
        assert_eq!(
            sim.ledger[0].household.as_ref().unwrap().retirements[0].rejection,
            Some(reason)
        );
        // Reassignment independently clears the household holding and permits payout.
        assert_eq!(
            sim.state.balance(HOME, TOKEN),
            if case == 1 { 0 } else { 5 }
        );
        assert_eq!(
            sim.state.balance(HOME, TOKEN) + sim.state.balance(PERSON, TOKEN),
            5
        );
    }
}

#[test]
fn retirement_replay_rejects_forgery_repetition_and_revival() {
    let (w, s) = retirement_fixture();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    sim.step().unwrap();
    let batch = sim.ledger[0].clone();
    let mut replay = s.clone();
    commit(
        &w,
        &mut replay,
        &batch,
        Backend::CubeCpu,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    assert_eq!(replay, sim.state);
    let unchanged = replay.clone();
    assert!(
        commit(
            &w,
            &mut replay,
            &batch,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, unchanged);
    for omit in [true, false] {
        let mut forged = batch.clone();
        if omit {
            forged.household.as_mut().unwrap().retirements.clear();
        } else {
            forged.household.as_mut().unwrap().retirements[0]
                .equipment
                .as_mut()
                .unwrap()
                .owner = PERSON;
        }
        let mut untouched = s.clone();
        assert!(
            commit(
                &w,
                &mut untouched,
                &forged,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(untouched, s);
    }
    let mut revived = sim.state.clone();
    revived.equipment.insert(TOOL, s.equipment[&TOOL].clone());
    assert!(Simulation::new(w.clone(), revived, Backend::Reference).is_err());
    let mut claimed = w.clone();
    claimed.offers.push(Offer {
        id: 1,
        seller: HOME,
        asset: TOOL,
        price: Amount::new(TOKEN, 1),
    });
    assert!(Simulation::new(claimed, sim.state.clone(), Backend::Reference).is_err());
    let mut missing_authority = w;
    missing_authority.households[0]
        .equipment_retirements
        .clear();
    assert!(Simulation::new(missing_authority, sim.state, Backend::Reference).is_err());
}

#[test]
fn retired_equipment_preserves_filled_offer_history_without_counting_as_a_holding() {
    let (mut w, mut s) = fixture();
    s.equipment.get_mut(&TOOL).unwrap().remaining_uses = 0;
    w.offers.push(Offer {
        id: 1,
        seller: HOME,
        asset: TOOL,
        price: Amount::new(TOKEN, 1),
    });
    s.filled_offers.insert(1); // completed historical offer, not a current commitment
    households::retirement::request(&mut w, &s, HOME, PERSON, TOOL).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
    sim.run_months(1).unwrap();
    assert!(sim.state.filled_offers.contains(&1));
    assert_eq!(sim.world.offers.len(), 1);
    assert!(sim.state.equipment.is_empty());
    assert_eq!(sim.state.retired_equipment.len(), 1);
}

#[test]
fn retired_equipment_has_no_hidden_value_or_repair_target_and_ids_cannot_be_reused() {
    use economics_compute_smoke::activities::{self, Outcome};
    let (mut w, mut s) = retirement_fixture();
    let retired_id = 1_000_001;
    let mut tool = s.equipment.remove(&TOOL).unwrap();
    tool.id = retired_id;
    s.equipment.insert(retired_id, tool);
    w.households[0].equipment_retirements[0].asset = retired_id;
    assert!(
        Audit::with_inventory(
            &w,
            &s,
            TOKEN,
            [(retired_id, 1)].into(),
            [((HOME, GRAIN), 4)].into()
        )
        .is_err()
    );
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    assert!(
        Audit::with_inventory(
            &sim.world,
            &sim.state,
            TOKEN,
            [(retired_id, 1)].into(),
            [((HOME, GRAIN), 4)].into()
        )
        .is_err()
    );
    let mut w = sim.world.clone();
    w.definitions.push(ProcessDefinition {
        id: 999,
        name: "repair or replace".into(),
        enabled: true,
        execution: Execution::Productive,
        asset_kind: None,
        stages: vec![Stage {
            name: "work".into(),
            months: 1,
            entry_inputs: vec![],
            monthly_services: vec![Amount::new(LABOR, 1)],
        }],
        outputs: vec![],
    });
    w.activities.outcomes.insert(
        999,
        Outcome::Repair {
            kind: KIND,
            restore: 2,
        },
    );
    economics_compute_smoke::settlement::validate_world(&w, &sim.state).unwrap();
    let process = ProcessInstance {
        id: 1,
        definition: 999,
        operator: HOME,
        beneficiary: HOME,
        goal: None,
        asset: None,
        right: None,
        start: 2,
        reserved_through: 2,
        stage: 0,
        elapsed: 1,
        status: Status::Completed,
    };
    let mut unchanged = sim.state.clone();
    assert!(
        activities::apply(
            &w,
            &mut unchanged,
            &ProcessChange {
                before: None,
                after: process.clone()
            }
        )
        .is_err()
    );
    assert_eq!(unchanged, sim.state);
    w.activities.outcomes.insert(999, Outcome::Create(KIND));
    economics_compute_smoke::settlement::validate_world(&w, &sim.state).unwrap();
    assert!(
        activities::apply(
            &w,
            &mut unchanged,
            &ProcessChange {
                before: None,
                after: process
            }
        )
        .is_err()
    );
    assert_eq!(unchanged, sim.state);
}

#[test]
fn retirement_rechecks_its_own_permission_and_observer_records_outcome() {
    use economics_compute_smoke::opportunities::{Action, HOUSEHOLD_TYPE, PERSON_TYPE, STATE_TYPE};
    for allowed in [false, true] {
        let (mut w, s) = retirement_fixture();
        let (policy_world, _) = economics_compute_smoke::membership::scenario().unwrap();
        let mut policy = policy_world.transaction_policy.unwrap();
        policy.laws.clear();
        policy.membership_offers.clear();
        policy.membership_permissions.clear();
        policy.agent_types = [
            (HOME, HOUSEHOLD_TYPE),
            (PERSON, PERSON_TYPE),
            (STATE_AGENT, STATE_TYPE),
        ]
        .into();
        policy.permissions = [(HOUSEHOLD_TYPE, Action::AssetTrade)].into();
        if allowed {
            policy
                .permissions
                .insert((HOUSEHOLD_TYPE, Action::RetireEquipment));
        }
        w.transaction_policy = Some(policy);
        let mut observed = Simulation::new(w, s, Backend::Reference).unwrap();
        let mut plain = observed.clone();
        let mut observer = Observer::new(
            vec![],
            "equipment-retirement",
            Config {
                settlement: true,
                agents: [HOME].into(),
                ..Config::default()
            },
        )
        .unwrap();
        observer.run_months(&mut observed, 1).unwrap();
        plain.run_months(1).unwrap();
        assert_eq!(observed.state, plain.state);
        assert_eq!(observed.ledger, plain.ledger);
        assert_eq!(
            observed.state.retired_equipment.contains_key(&TOOL),
            allowed
        );
        let log = String::from_utf8(observer.finish().unwrap()).unwrap();
        let rows: Vec<serde_json::Value> = log
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .filter(|r: &serde_json::Value| r["kind"] == "household_equipment_retirement")
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["retired"], allowed);
        assert_eq!(rows[0]["remaining_uses"], 0);
        assert_eq!(
            rows[0]["rejection"],
            if allowed {
                serde_json::Value::Null
            } else {
                "Permission".into()
            }
        );
    }
    let (mut w, s) = retirement_fixture();
    w.households[0].asset_sales.push(terms(1));
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
}
