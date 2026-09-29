use economics_compute_smoke::{
    accounting::{Account as A, Flow},
    activities::DurableKind,
    compute::Backend,
    equipment::{DurableAsset, Offer},
    financial_reporting::Audit,
    household_governance::Governance,
    households::{
        self, Agreement,
        disposal::{self, Attachment, Rejection, Sale},
        dissolution,
    },
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit, validate_world},
    simulation::Simulation,
    telemetry::{Config, Observer},
};

const HOME: AgentId = 10000;
const BUILDING: u32 = 99;
const WORKSHOP: AssetId = TOOL + 1;

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
            support: vec![],
            formed: 1,
            dwelling_process: None,
            admission: None,
        },
    )
    .unwrap();
    w.assets.push(Asset {
        id: PLOT,
        owner: HOME,
        kind: 1,
    });
    // Accelerated fixture wear exercises acquisition-cost depreciation; not a housing calibration.
    w.activities.kinds.insert(
        BUILDING,
        DurableKind {
            name: "attached building".into(),
            lifetime: 12,
            attached: true,
            monthly_decay: 1,
        },
    );
    for (id, uses) in [(TOOL, 6), (WORKSHOP, 3)] {
        s.equipment.insert(
            id,
            DurableAsset {
                id,
                owner: HOME,
                attached_to: Some(PLOT),
                kind: BUILDING,
                remaining_uses: uses,
                last_used_month: Some(1),
            },
        );
    }
    s.month = 2;
    s.balances.insert((HOME, TOKEN), 5);
    s.balances.insert((HOME, GRAIN), 2);
    s.balances.insert((STATE_AGENT, TOKEN), 40);
    dissolution::request(&mut w, &s, HOME, PERSON).unwrap();
    (w, s)
}
fn sale(total: i32, dwelling: i32, workshop: i32) -> Sale {
    Sale {
        month: 2,
        asset: PLOT,
        buyer: STATE_AGENT,
        price: Amount::new(TOKEN, total),
        control: None,
        attachments: vec![
            Attachment {
                asset: TOOL,
                consideration: dwelling,
            },
            Attachment {
                asset: WORKSHOP,
                consideration: workshop,
            },
        ],
    }
}
fn accept(w: &mut World, s: &State, terms: Sale) {
    disposal::accept(w, s, HOME, PERSON, terms).unwrap();
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_inventory(
        w,
        s,
        TOKEN,
        [(PLOT, 6), (TOOL, 12), (WORKSHOP, 6)].into(),
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
fn package_transfers_intact_with_allocated_costs_depreciation_and_later_closure_on_cpu() {
    for (total, dwelling, workshop) in [(30, 12, 10), (15, 6, 6), (1, 0, 0)] {
        let (mut w, s) = fixture();
        accept(&mut w, &s, sale(total, dwelling, workshop));
        let run = |backend| {
            let mut a = audit(&w, &s);
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            a.step(&mut sim).unwrap();
            assert_eq!(sim.state.credit.owners[&PLOT], STATE_AGENT);
            assert_eq!(sim.state.credit.values[&PLOT], total - dwelling - workshop);
            assert_eq!(sim.state.balance(HOME, TOKEN), 5 + total);
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 40 - total);
            assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
            let receipt = &sim.ledger[0].household.as_ref().unwrap().disposals[0];
            assert_eq!(receipt.rejection, None);
            assert_eq!(
                receipt.attachments,
                s.equipment.values().cloned().collect::<Vec<_>>()
            );
            let buyer = a.book().statements(STATE_AGENT, 2, 2).unwrap();
            let mut expected_decay = 0;
            for (id, cost) in [(TOOL, dwelling), (WORKSHOP, workshop)] {
                let old = &s.equipment[&id];
                let current = &sim.state.equipment[&id];
                assert_eq!(current.owner, STATE_AGENT);
                assert_eq!(current.attached_to, old.attached_to);
                assert_eq!(current.last_used_month, old.last_used_month);
                assert_eq!(current.remaining_uses, old.remaining_uses - 1);
                let decay = cost / old.remaining_uses as i32;
                expected_decay += decay;
                assert_eq!(
                    buyer
                        .trial_balance
                        .get(&A::Tangible(id))
                        .copied()
                        .unwrap_or(0),
                    i128::from(cost - decay)
                );
            }
            assert_eq!(
                buyer.expenses.get(&A::Depreciation).copied().unwrap_or(0),
                i128::from(expected_decay)
            );
            assert_eq!(buyer.cash_flows[&Flow::Investing], -i128::from(total));
            let seller = a.book().statements(HOME, 2, 2).unwrap();
            let differences = [total - dwelling - workshop - 6, dwelling - 12, workshop - 6];
            assert_eq!(
                seller.income.get(&A::DisposalGain).copied().unwrap_or(0),
                differences
                    .iter()
                    .map(|x| i128::from((*x).max(0)))
                    .sum::<i128>()
            );
            assert_eq!(
                seller.expenses.get(&A::DisposalLoss).copied().unwrap_or(0),
                differences
                    .iter()
                    .map(|x| i128::from((-*x).max(0)))
                    .sum::<i128>()
            );
            assert_eq!(
                seller.expenses.get(&A::Depreciation).copied().unwrap_or(0),
                0
            );
            assert_eq!(seller.cash_flows[&Flow::Investing], i128::from(total));
            let checkpoint = (sim.clone(), a.clone());
            through(&mut a, &mut sim, 3);
            assert_eq!(sim.state.balance(PERSON, TOKEN), 5 + total);
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
                    assert_eq!(report.liabilities, 0);
                }
            }
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn package_requires_explicit_complete_unique_components_and_valid_price_allocation() {
    for case in 0..9 {
        let (mut w, s) = fixture();
        let mut terms = sale(30, 12, 10);
        match case {
            0 => {
                terms.attachments.pop();
            }
            1 => terms.attachments.push(terms.attachments[0].clone()),
            2 => terms.attachments[0].consideration = -1,
            3 => terms.attachments[0].consideration = i32::MAX,
            4 => terms.attachments[0].asset = PLOT,
            5 => terms.attachments[0].asset = u32::MAX,
            6 => terms.asset = TOOL,
            7 => terms.price.quantity = 22, // catalog registry requires a positive remainder
            _ => terms.price.quantity = 0,
        }
        let unchanged = w.clone();
        assert!(
            disposal::accept(&mut w, &s, HOME, PERSON, terms).is_err(),
            "case {case}"
        );
        assert_eq!(w, unchanged);
    }
    let (mut w, s) = fixture();
    accept(&mut w, &s, sale(30, 12, 10));
    // Historical instructions may not double-sell a package component at this boundary.
    w.households[0].asset_sales.push(Sale {
        month: 2,
        asset: TOOL,
        buyer: STATE_AGENT,
        price: Amount::new(TOKEN, 1),
        control: None,
        attachments: vec![],
    });
    assert!(validate_world(&w, &s).is_err());
    w.households[0].asset_sales.pop();
    w.households[0]
        .equipment_retirements
        .push(households::retirement::Request {
            mode: households::retirement::Mode::Exhausted,
            month: 2,
            asset: TOOL,
        });
    assert!(validate_world(&w, &s).is_err());
}

#[test]
fn changed_packages_ownership_use_and_claims_reject_whole_sale_at_execution() {
    for case in 0..7 {
        let (mut w, mut s) = fixture();
        accept(&mut w, &s, sale(30, 12, 10));
        let reason = match case {
            0 => {
                let mut extra = s.equipment[&TOOL].clone();
                extra.id = TOOL + 2;
                s.equipment.insert(extra.id, extra);
                Rejection::Attached
            }
            1 => {
                s.equipment.get_mut(&TOOL).unwrap().owner = PERSON;
                Rejection::Unavailable
            }
            2 => {
                s.equipment.get_mut(&TOOL).unwrap().last_used_month = Some(2);
                Rejection::Unavailable
            }
            3 => {
                s.equipment.get_mut(&TOOL).unwrap().remaining_uses = 0;
                Rejection::Unavailable
            }
            4 => {
                w.offers.push(Offer {
                    id: 1,
                    seller: HOME,
                    asset: TOOL,
                    price: Amount::new(TOKEN, 1),
                });
                Rejection::Encumbered
            }
            5 => {
                let (baseline, _) = baseline();
                w.rights.push(baseline.rights[0].clone());
                Rejection::Attached
            }
            _ => {
                let mut portable = w.activities.kinds[&BUILDING].clone();
                portable.attached = false;
                w.activities.kinds.insert(BUILDING + 1, portable);
                let tool = s.equipment.get_mut(&TOOL).unwrap();
                tool.kind = BUILDING + 1;
                tool.attached_to = None;
                Rejection::Attached
            }
        };
        let mut sim = Simulation::new(w, s.clone(), Backend::Reference).unwrap();
        sim.step().unwrap();
        assert_eq!(
            sim.ledger[0].household.as_ref().unwrap().disposals[0].rejection,
            Some(reason)
        );
        assert_eq!(sim.state.balance(HOME, TOKEN), 5);
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 40);
        assert!(sim.state.credit.owners.is_empty());
        for (&id, old) in &s.equipment {
            assert_eq!(sim.state.equipment[&id].owner, old.owner);
            assert_eq!(sim.state.equipment[&id].attached_to, old.attached_to);
        }
    }
}

#[test]
fn package_replay_checks_every_condition_allocation_and_payment_atomically() {
    let (mut w, s) = fixture();
    accept(&mut w, &s, sale(30, 12, 10));
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    sim.step().unwrap();
    let mut replay = s.clone();
    commit(
        &w,
        &mut replay,
        &sim.ledger[0],
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
            &sim.ledger[0],
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, unchanged);
    for case in 0..5 {
        let mut forged = sim.ledger[0].clone();
        let boundary = forged.household.as_mut().unwrap();
        match case {
            0 => {
                boundary.disposals[0].attachments.pop();
            }
            1 => boundary.disposals[0].attachments[0].remaining_uses += 1,
            2 => boundary.disposals[0].attachments[0].attached_to = None,
            3 => boundary.disposals[0].sale.attachments[0].consideration += 1,
            _ => boundary.disposal_effects.clear(),
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
    let mut untouched = s.clone();
    assert!(commit(&w, &mut untouched, &sim.ledger[0], Backend::Reference, 1).is_err());
    assert_eq!(untouched, s);
}

#[test]
fn package_funding_is_indivisible_and_shares_opening_budget_with_bare_sales() {
    for budget in [29, 30] {
        let (mut w, mut s) = fixture();
        s.balances.insert((STATE_AGENT, TOKEN), budget);
        w.assets.push(Asset {
            id: PLOT + 1,
            owner: HOME,
            kind: 1,
        });
        accept(&mut w, &s, sale(30, 12, 10));
        accept(
            &mut w,
            &s,
            Sale {
                month: 2,
                asset: PLOT + 1,
                buyer: STATE_AGENT,
                price: Amount::new(TOKEN, 1),
                control: None,
                attachments: vec![],
            },
        );
        let run = |mut w: World, reverse| {
            if reverse {
                w.households[0].asset_sales.reverse();
            }
            let mut sim = Simulation::new(w, s.clone(), Backend::Reference).unwrap();
            sim.step().unwrap();
            let r = &sim.ledger[0].household.as_ref().unwrap().disposals;
            assert_eq!(r[0].rejection.is_none(), budget == 30);
            assert_eq!(r[1].rejection.is_none(), budget == 29);
            assert_eq!(
                sim.state.equipment[&TOOL].owner,
                if budget == 30 { STATE_AGENT } else { HOME }
            );
            assert_eq!(
                sim.state.equipment[&WORKSHOP].owner,
                sim.state.equipment[&TOOL].owner
            );
            assert_eq!(
                sim.state.balance(STATE_AGENT, TOKEN),
                if budget == 30 { 0 } else { 28 }
            );
            (sim.state, sim.ledger)
        };
        assert_eq!(run(w.clone(), false), run(w, true));
    }
}

#[test]
fn package_observation_is_read_only_and_component_order_is_canonical() {
    let (mut w, s) = fixture();
    let mut reversed_world = w.clone();
    let mut reversed = sale(30, 12, 10);
    reversed.attachments.reverse();
    accept(&mut reversed_world, &s, reversed);
    accept(&mut w, &s, sale(30, 12, 10));
    assert_eq!(w, reversed_world);
    let mut observed = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut plain = observed.clone();
    let mut observer = Observer::new(
        vec![],
        "property-package",
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
    assert_eq!(
        rows[0]["allocations"],
        serde_json::json!([[PLOT, 8], [TOOL, 12], [WORKSHOP, 10]])
    );
    assert_eq!(rows[0]["attachments"][0]["attached_to"], PLOT);
    assert_eq!(rows[0]["attachments"][0]["remaining_uses"], 6);
    assert_eq!(rows[0]["attachments"][1]["remaining_uses"], 3);
}

#[test]
fn package_does_not_novate_crop_work_or_pledged_components() {
    use economics_compute_smoke::credit::{Advance, Collateral, CollateralSettlement, LoanOffer};
    let (mut w, mut s) = fixture();
    let (base, _) = baseline();
    let definition = base.definition(GROW).clone();
    let duration = definition.duration();
    w.definitions.push(definition);
    w.rights = base.rights;
    s.processes.insert(
        1,
        ProcessInstance {
            id: 1,
            definition: GROW,
            operator: PERSON,
            beneficiary: PERSON,
            goal: None,
            asset: Some(PLOT),
            right: Some(w.rights[0].id),
            start: 1,
            reserved_through: duration,
            stage: 0,
            elapsed: 1,
            status: Status::Active,
        },
    );
    validate_world(&w, &s).unwrap();
    let before = (w.clone(), s.clone());
    let err = disposal::accept(&mut w, &s, HOME, PERSON, sale(30, 12, 10)).unwrap_err();
    assert!(err.contains("transferable property"));
    assert_eq!((w, s), before);
    // Future collateral reservations block admission: assets cannot be sold
    // out from under their promised pledge.
    for asset in [PLOT, TOOL, WORKSHOP] {
        let (mut w, s) = fixture();
        w.lending.push(Advance {
            id: 77,
            debtor: HOME,
            terms: LoanOffer {
                creditor: STATE_AGENT,
                denomination: TOKEN,
                max_principal: 2,
                monthly_rate_bps: 0,
                term_months: 1,
                grace_months: 1,
            },
            principal: 2,
            month: 3,
            priority: 0,
            collateral: Some(Collateral {
                asset,
                priority: 0,
                pledged: true,
                settlement: CollateralSettlement::FixedValue { value: 2 },
            }),
        });
        let before = w.clone();
        let err = disposal::accept(&mut w, &s, HOME, PERSON, sale(30, 12, 10)).unwrap_err();
        assert!(err.contains("transferable property"));
        assert_eq!(w, before);
    }
}

#[test]
fn package_rechecks_both_parties_law_and_storage_and_requires_fresh_terms_after_failure() {
    use economics_compute_smoke::opportunities::{Action, HOUSEHOLD_TYPE, PERSON_TYPE, STATE_TYPE};
    for missing in [HOUSEHOLD_TYPE, STATE_TYPE] {
        let (mut w, s) = fixture();
        accept(&mut w, &s, sale(30, 12, 10));
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
        policy.permissions = [HOUSEHOLD_TYPE, STATE_TYPE]
            .into_iter()
            .filter(|t| *t != missing)
            .map(|t| (t, Action::AssetTrade))
            .collect();
        w.transaction_policy = Some(policy);
        let mut sim = Simulation::new(w, s.clone(), Backend::Reference).unwrap();
        sim.step().unwrap();
        assert_eq!(
            sim.ledger[0].household.as_ref().unwrap().disposals[0].rejection,
            Some(Rejection::Permission)
        );
        assert_eq!(sim.state.balance(HOME, TOKEN), 5);
        assert_eq!(sim.state.equipment[&TOOL].owner, HOME);
        assert!(sim.state.credit.owners.is_empty());
    }
    let (mut w, s) = fixture();
    accept(&mut w, &s, sale(30, 12, 10));
    w.storage.weights.insert(TOKEN, 1);
    w.storage.capacities.insert(PERSON, 20); // household can hold ten, not 35
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(
        sim.ledger[0].household.as_ref().unwrap().disposals[0].rejection,
        Some(Rejection::FundingOrStorage)
    );
    assert!(sim.state.credit.owners.is_empty());
    assert_eq!(sim.state.equipment[&WORKSHOP].owner, HOME);
    sim.world.storage.capacities.insert(PERSON, 100);
    let mut no_consent = sim.clone();
    no_consent.run_months(1).unwrap();
    assert!(no_consent.state.credit.owners.is_empty());
    assert_eq!(no_consent.state.balance(HOME, TOKEN), 5);
    let mut fresh = sale(30, 12, 10);
    fresh.month = 3;
    accept(&mut sim.world, &sim.state, fresh);
    sim.step().unwrap();
    assert_eq!(sim.state.credit.owners[&PLOT], STATE_AGENT);
    assert_eq!(sim.state.equipment[&WORKSHOP].owner, STATE_AGENT);
    assert_eq!(sim.state.balance(HOME, TOKEN), 35);
}

#[test]
fn unsupported_package_payment_valuation_rolls_back_state_and_every_statement() {
    let (mut w, mut s) = fixture();
    s.balances.insert((STATE_AGENT, GRAIN), 30);
    let mut terms = sale(30, 12, 10);
    terms.price.resource = GRAIN;
    accept(&mut w, &s, terms);
    let mut a = Audit::with_inventory(
        &w,
        &s,
        TOKEN,
        [(PLOT, 6), (TOOL, 12), (WORKSHOP, 6)].into(),
        [((HOME, GRAIN), 4), ((STATE_AGENT, GRAIN), 60)].into(),
    )
    .unwrap();
    let before = a.clone();
    let mut sim = Simulation::new(w, s.clone(), Backend::Reference).unwrap();
    let err = a.step(&mut sim).unwrap_err();
    assert!(err.contains("reporting-denomination"));
    assert_eq!(a, before);
    assert_eq!(sim.state, s);
    assert!(sim.ledger.is_empty());
}

#[test]
fn explicit_title_and_crop_transfer_preserves_work_cost_and_requires_future_labor() {
    use economics_compute_smoke::{financial_reporting::Opening, process_accounting::Costs};
    for works in [false, true] {
        let (mut w, mut s) = fixture();
        let (base, _) = baseline();
        let d = base.definition(GROW).clone();
        let duration = d.duration();
        w.definitions.push(d);
        w.rights = base.rights;
        w.rights[0].holder = HOME;
        w.rights[0].output_owner = HOME;
        w.ownership_rights.insert(w.rights[0].id);
        s.processes.insert(
            1,
            ProcessInstance {
                id: 1,
                definition: GROW,
                operator: HOME,
                beneficiary: HOME,
                goal: None,
                asset: Some(PLOT),
                right: Some(w.rights[0].id),
                start: 1,
                reserved_through: duration,
                stage: 1,
                elapsed: 0,
                status: Status::Active,
            },
        );
        let mut terms = sale(30, 12, 10);
        terms.buyer = PERSON;
        terms.control = Some(disposal::Control {
            rights: vec![w.rights[0].id],
            processes: vec![1],
        });
        s.balances.insert((PERSON, TOKEN), 40);
        if !works {
            w.participants[0].capacity.quantity = 0;
        }
        accept(&mut w, &s, terms);
        let run = |backend| {
            let mut a = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    assets: [(PLOT, 6), (TOOL, 12), (WORKSHOP, 6)].into(),
                    inventory: [((HOME, GRAIN), 4)].into(),
                    processes: Some(Costs {
                        work: [(1, (HOME, 3))].into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .unwrap();
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            a.step(&mut sim).unwrap();
            let p = &sim.state.processes[&1];
            assert_eq!(
                (p.operator, p.beneficiary, p.elapsed, p.start),
                (PERSON, PERSON, 0, 1)
            );
            assert_eq!(
                sim.ledger[0].household.as_ref().unwrap().disposals[0]
                    .processes
                    .len(),
                1
            );
            assert_eq!(
                a.book().statements(HOME, 2, 2).unwrap().expenses[&A::TransferExpense],
                3
            );
            assert_eq!(sim.state.credit.values[&PLOT], 8); // Crop cost does not inflate collateral.
            let (mut resumed, mut ra) = (sim.clone(), a.clone());
            through(&mut a, &mut sim, duration + 1);
            through(&mut ra, &mut resumed, duration + 1);
            assert_eq!(
                (sim.state.clone(), sim.ledger.clone(), a.clone()),
                (resumed.state, resumed.ledger, ra)
            );
            assert_eq!(
                sim.state.processes[&1].status,
                if works {
                    Status::Completed
                } else {
                    Status::Aborted
                }
            );
            assert_eq!(sim.state.balance(PERSON, GRAIN), if works { 10 } else { 2 });
            dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn live_right_transfer_requires_exact_consent_rechecks_it_and_replays_atomically() {
    let setup = || {
        let (mut w, s) = fixture();
        w.rights = baseline().0.rights;
        w.rights[0].holder = HOME;
        w.rights[0].output_owner = HOME;
        w.ownership_rights.insert(w.rights[0].id);
        let mut terms = sale(30, 12, 10);
        terms.control = Some(disposal::Control {
            rights: vec![w.rights[0].id],
            processes: vec![],
        });
        (w, s, terms)
    };
    for mode in 0..3 {
        let (mut w, s, mut terms) = setup();
        match mode {
            0 => terms.control.as_mut().unwrap().rights.clear(),
            1 => w.ownership_rights.clear(),
            _ => {
                let mut r = w.rights[0].clone();
                r.id += 1;
                w.rights.push(r);
            }
        }
        let before = w.clone();
        assert!(disposal::accept(&mut w, &s, HOME, PERSON, terms).is_err());
        assert_eq!(w, before);
    }
    let (mut w, s, terms) = setup();
    accept(&mut w, &s, terms);
    let mut changed = w.clone();
    changed.ownership_rights.clear();
    let mut sim = Simulation::new(changed, s.clone(), Backend::Reference).unwrap();
    sim.step().unwrap();
    assert_eq!(
        sim.ledger[0].household.as_ref().unwrap().disposals[0].rejection,
        Some(Rejection::Attached)
    );
    assert_eq!(sim.state.balance(HOME, TOKEN), 5);
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    sim.step().unwrap();
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let mut forged = sim.ledger[0].clone();
        forged.household.as_mut().unwrap().disposals[0]
            .sale
            .control
            .as_mut()
            .unwrap()
            .rights
            .clear();
        let mut unchanged = s.clone();
        assert!(commit(&w, &mut unchanged, &forged, backend, DEFAULT_EFFECT_LIMIT).is_err());
        assert_eq!(unchanged, s);
    }
}
