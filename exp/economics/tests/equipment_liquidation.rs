use economics_compute_smoke::{
    accounting::{Account as A, Flow},
    activities::DurableKind,
    compute::Backend,
    credit::{Advance, LoanOffer},
    equipment::DurableAsset,
    finance::CollectionPolicy,
    financial_reporting::Audit,
    household_governance::Governance,
    households::{self, Agreement, dissolution, retirement},
    model::*,
    offers::{self, Id, Request},
    recovery::{Bid, Listing, ProceedingTerms, Stage},
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

const OTHER: AgentId = 97;
const BUYER: AgentId = 98;
const ESTATE: AgentId = 99;
const HOME: AgentId = 10000;
const KIND: u32 = 99;

fn fixture(household: bool, funded: bool, backend: Backend) -> (Simulation, Audit) {
    let (mut w, mut s) = baseline();
    w.participants.clear();
    w.definitions.clear();
    w.rights.clear();
    w.agreements.clear();
    w.assets.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    for id in [OTHER, BUYER, ESTATE] {
        w.agents.push(Agent {
            id,
            name: format!("agent {id}"),
        });
    }
    w.collection_policy = CollectionPolicy::Proportional;
    for (id, creditor) in [(10, STATE_AGENT), (11, OTHER)] {
        w.lending.push(Advance {
            id,
            debtor: PERSON,
            principal: 10,
            month: 1,
            collateral: None,
            priority: 0,
            terms: LoanOffer {
                creditor,
                denomination: TOKEN,
                max_principal: 10,
                monthly_rate_bps: 0,
                term_months: 1,
                grace_months: 10,
            },
        });
    }
    s.balances.clear();
    for who in [STATE_AGENT, OTHER] {
        s.balances.insert((who, TOKEN), 10);
    }
    let buyer = if household { HOME } else { BUYER };
    if household {
        let mut member = baseline().0.participants[0].clone();
        member.agent = BUYER;
        member.needs.clear();
        member.capacity.quantity = 0;
        w.participants.push(member);
        let mut governance = Governance::contributed(BUYER);
        governance.constitution.allow_dissolution = true;
        households::form(
            &mut w,
            &s,
            Agreement {
                id: 1,
                agent: HOME,
                adults: vec![BUYER],
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
    }
    s.balances
        .insert((buyer, TOKEN), if funded { 8 } else { 0 });
    w.activities.kinds.insert(
        KIND,
        DurableKind {
            name: "portable tool".into(),
            lifetime: 8,
            monthly_decay: 1,
            attached: false,
        },
    );
    s.equipment.insert(
        TOOL,
        DurableAsset {
            id: TOOL,
            owner: PERSON,
            kind: KIND,
            attached_to: None,
            remaining_uses: 8,
            last_used_month: None,
        },
    );
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 4,
        assets: vec![Listing {
            asset: TOOL,
            minimum_price: 6,
        }],
        discharge_deficiency: true,
    });
    w.recovery.bids.push(Bid {
        id: 1,
        proceeding: 1,
        buyer,
        asset: TOOL,
        month: 3,
        price: 8,
    });
    let mut sim = Simulation::new(w, s, backend).unwrap();
    sim.run_months(1).unwrap();
    // Controlled loss before opening the reporting book, identical on both backends.
    sim.state.balances.insert((PERSON, TOKEN), 0);
    let audit = Audit::with_assets(&sim.world, &sim.state, TOKEN, [(TOOL, 7)].into()).unwrap();
    (sim, audit)
}
fn through(sim: &mut Simulation, audit: &mut Audit, month: u32) {
    while sim.state.month <= month {
        audit.step(sim).unwrap();
    }
}

#[test]
fn funded_estate_equipment_sale_preserves_wear_and_books_actual_cost_on_cpu() {
    let run = |backend| {
        let (mut sim, mut audit) = fixture(false, true, backend);
        while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
            audit.step(&mut sim).unwrap();
        }
        let opening = sim.state.clone();
        let tool = opening.equipment[&TOOL].clone();
        let batch = offers::prepare(&sim, &[Request::new(Id::LiquidationBid(1), BUYER)]).unwrap();
        assert_eq!(tool.remaining_uses, 5);
        assert_eq!(
            batch.credit.as_ref().unwrap().equipment[&TOOL].remaining_uses,
            5
        );
        for corrupt_owner in [false, true] {
            let mut forged = batch.clone();
            let transferred = forged
                .credit
                .as_mut()
                .unwrap()
                .equipment
                .get_mut(&TOOL)
                .unwrap();
            if corrupt_owner {
                transferred.owner = OTHER;
            } else {
                transferred.remaining_uses += 1;
            }
            let mut untouched = opening.clone();
            assert!(
                commit(
                    &sim.world,
                    &mut untouched,
                    &forged,
                    backend,
                    DEFAULT_EFFECT_LIMIT
                )
                .is_err()
            );
            assert_eq!(untouched, opening);
        }
        audit.step(&mut sim).unwrap();
        assert_eq!(sim.ledger.last(), Some(&batch));
        let transferred = &sim.state.equipment[&TOOL];
        assert_eq!(transferred.owner, BUYER);
        assert_eq!(transferred.remaining_uses, tool.remaining_uses);
        assert_eq!(transferred.last_used_month, tool.last_used_month);
        assert!(sim.state.credit.owners.is_empty());
        assert_eq!(sim.state.balance(ESTATE, TOKEN), 8);
        let buyer = audit.book().statements(BUYER, 2, 3).unwrap();
        assert_eq!(buyer.trial_balance[&A::Tangible(TOOL)], 8);
        assert_eq!(buyer.cash_flows[&Flow::Investing], -8);
        assert_eq!(
            audit.book().statements(PERSON, 2, 3).unwrap().income[&A::DisposalGain],
            3
        );
        let mut replay = opening;
        commit(
            &sim.world,
            &mut replay,
            &batch,
            backend,
            DEFAULT_EFFECT_LIMIT,
        )
        .unwrap();
        assert_eq!(replay, sim.state);
        assert!(
            commit(
                &sim.world,
                &mut replay,
                &batch,
                backend,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        let (mut resumed, mut ra) = (sim.clone(), audit.clone());
        through(&mut sim, &mut audit, 8);
        through(&mut resumed, &mut ra, 8);
        assert_eq!(
            (&sim.state, &sim.ledger, &audit),
            (&resumed.state, &resumed.ledger, &ra)
        );
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Closed
        );
        assert_eq!(sim.state.equipment[&TOOL].remaining_uses, 0);
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 4);
        assert_eq!(sim.state.balance(OTHER, TOKEN), 4);
        assert_eq!(
            audit.book().statements(BUYER, 2, 8).unwrap().expenses[&A::Depreciation],
            8
        );
        for who in [BUYER, PERSON, ESTATE, STATE_AGENT, OTHER] {
            let report = audit.book().statements(who, 2, 8).unwrap();
            assert_eq!(report.assets, report.liabilities + report.equity);
        }
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn unfunded_bid_leaves_equipment_and_no_appraisal_cash_in_estate() {
    let (mut sim, mut audit) = fixture(false, false, Backend::CubeCpu);
    through(&mut sim, &mut audit, 4);
    assert_eq!(sim.state.equipment[&TOOL].owner, PERSON);
    assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
    assert_ne!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    assert!(sim.state.credit.recovery.proceedings[&1].sold.is_empty());
}

#[test]
fn household_can_retire_bought_estate_equipment_and_dissolve_after_its_life() {
    let run = |backend| {
        let (mut sim, mut audit) = fixture(true, true, backend);
        through(&mut sim, &mut audit, 8);
        assert_eq!(sim.state.equipment[&TOOL].owner, HOME);
        assert_eq!(sim.state.equipment[&TOOL].remaining_uses, 0);
        dissolution::request(&mut sim.world, &sim.state, HOME, BUYER).unwrap();
        retirement::request(&mut sim.world, &sim.state, HOME, BUYER, TOOL).unwrap();
        through(&mut sim, &mut audit, 9);
        assert!(!sim.state.equipment.contains_key(&TOOL));
        assert_eq!(sim.state.retired_equipment[&TOOL].equipment.owner, HOME);
        through(&mut sim, &mut audit, 10);
        dissolution::finish(&mut sim.world, &sim.state, HOME, BUYER).unwrap();
        through(&mut sim, &mut audit, 11);
        assert_eq!(audit.book().statements(HOME, 2, 11).unwrap().assets, 0);
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn sale_rechecks_exclusive_use_and_condition_after_listing() {
    for used in [false, true] {
        let (mut sim, _) = fixture(false, true, Backend::Reference);
        while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
            sim.step().unwrap();
        }
        assert_eq!(
            economics_compute_smoke::recovery::market::discover(&sim.world, &sim.state, BUYER)
                .len(),
            1
        );
        let tool = sim.state.equipment.get_mut(&TOOL).unwrap();
        if used {
            tool.last_used_month = Some(3);
        } else {
            tool.remaining_uses = 0;
        }
        assert!(
            economics_compute_smoke::recovery::market::discover(&sim.world, &sim.state, BUYER)
                .is_empty()
        );
        assert!(offers::prepare(&sim, &[Request::new(Id::LiquidationBid(1), BUYER)]).is_err());
        sim.step().unwrap();
        assert_eq!(sim.state.equipment[&TOOL].owner, PERSON);
        assert_eq!(sim.state.balance(BUYER, TOKEN), 8);
        assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
    }
}
