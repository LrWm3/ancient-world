use economics_compute_smoke::{
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::{Audit, Opening},
    model::*,
    recovery::{
        self, ProceedingTerms, Stage,
        inventory::{Bid, Listing},
    },
    scenario::{self, GRAIN, PERSON, STATE_AGENT, TOKEN},
    settlement,
    simulation::Simulation,
};
const BUYER: AgentId = 98;
const UNFUNDED: AgentId = 97;
const ESTATE: AgentId = 99;

fn fixture(funded: bool, room: i32) -> (World, State) {
    let (mut w, mut s) = scenario::baseline();
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
    for id in [BUYER, UNFUNDED, ESTATE] {
        w.agents.push(Agent {
            id,
            name: format!("agent {id}"),
        });
    }
    w.storage.weights.insert(GRAIN, 1);
    w.storage.capacities.insert(PERSON, 6);
    w.storage.capacities.insert(BUYER, room);
    s.balances.clear();
    s.balances.insert((STATE_AGENT, TOKEN), 10);
    s.balances.insert((PERSON, GRAIN), 6);
    s.balances
        .insert((BUYER, TOKEN), if funded { 8 } else { 0 });
    w.lending.push(Advance {
        id: 1,
        debtor: PERSON,
        principal: 10,
        month: 1,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: TOKEN,
            max_principal: 10,
            monthly_rate_bps: 0,
            term_months: 1,
            grace_months: 10,
        },
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 4,
        assets: vec![],
        discharge_deficiency: false,
    });
    w.recovery.inventory_listings.push(Listing {
        id: 1,
        proceeding: 1,
        goods: Amount::new(GRAIN, 4),
        minimum_price: 6,
    });
    w.recovery.inventory_bids = vec![
        Bid {
            id: 1,
            listing: 1,
            buyer: UNFUNDED,
            month: 3,
            price: 10,
        },
        Bid {
            id: 2,
            listing: 1,
            buyer: BUYER,
            month: 3,
            price: 8,
        },
        Bid {
            id: 3,
            listing: 1,
            buyer: BUYER,
            month: 4,
            price: 8,
        },
    ];
    (w, s)
}
fn opening(w: World, s: State, backend: Backend) -> (Simulation, Audit) {
    let mut sim = Simulation::new(w, s, backend).unwrap();
    sim.run_months(1).unwrap();
    // Identical external loss before this reporting interval, not hidden financing.
    sim.state.balances.insert((PERSON, TOKEN), 0);
    let audit = Audit::with_opening(
        &sim.world,
        &sim.state,
        TOKEN,
        Opening {
            inventory: [((PERSON, GRAIN), 12)].into(),
            exchange_values: [(GRAIN, 2)].into(),
            ..Opening::default()
        },
    )
    .unwrap();
    (sim, audit)
}

#[test]
fn inventory_sales_fund_custody_then_creditors_and_reconcile_costs() {
    for (funded, room) in [(true, 4), (false, 4), (true, 3)] {
        let (w, s) = fixture(funded, room);
        let sold = funded && room >= 4;
        let run = |backend| {
            let (mut sim, mut audit) = opening(w.clone(), s.clone(), backend);
            while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
                audit.step(&mut sim).unwrap();
            }
            let (mut resumed, mut ra) = (sim.clone(), audit.clone());
            audit.step(&mut sim).unwrap();
            let batch = sim.ledger.last().unwrap();
            assert_eq!(sim.state.balance(ESTATE, TOKEN), if sold { 8 } else { 0 });
            assert_eq!(sim.state.credit.loans[&1].principal, 10);
            assert_eq!(sim.state.balance(PERSON, GRAIN), if sold { 2 } else { 6 });
            let mut bad = batch.clone();
            bad.credit
                .as_mut()
                .unwrap()
                .after
                .recovery
                .proceedings
                .get_mut(&1)
                .unwrap()
                .sold_inventory
                .insert(999);
            // Mutate both state receipt and sale evidence to ensure the exact
            // boundary, not a self-consistent fabricated outcome, is validated.
            bad.transactions.clear();
            let mut state = resumed.state.clone();
            let before = state.clone();
            assert!(settlement::commit(&w, &mut state, &bad, backend, sim.effect_limit).is_err());
            assert_eq!(state, before);
            while sim.state.month < 5 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month < 5 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &ra)
            );
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
            assert_eq!(
                sim.state.credit.loans[&1].principal,
                if sold { 2 } else { 10 }
            );
            assert_eq!(
                sim.state.balance(STATE_AGENT, TOKEN),
                if sold { 8 } else { 0 }
            );
            assert_eq!(sim.state.balance(BUYER, GRAIN), if sold { 4 } else { 0 });
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                if sold { Stage::Closed } else { Stage::Active }
            );
            assert_eq!(
                sim.ledger
                    .iter()
                    .flat_map(|b| b.credit.iter().flat_map(|c| &c.recovery))
                    .filter(|r| matches!(r, recovery::Receipt::InventorySold { .. }))
                    .count(),
                usize::from(sold)
            );
            use economics_compute_smoke::accounting::Account as A;
            let balance = |agent, account| {
                audit
                    .book()
                    .balances()
                    .get(&(agent, account))
                    .copied()
                    .unwrap_or(0)
            };
            assert_eq!(
                balance(PERSON, A::Inventory(GRAIN)),
                if sold { 4 } else { 12 }
            );
            assert_eq!(
                balance(BUYER, A::Inventory(GRAIN)),
                if sold { 8 } else { 0 }
            );
            assert_eq!(balance(PERSON, A::CostOfSales), if sold { 8 } else { 0 });
            assert_eq!(balance(PERSON, A::Sales), if sold { -8 } else { 0 });
            for agent in &w.agents {
                let report = audit.book().statements(agent.id, 2, 4).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn competing_lots_share_actual_stock_cash_and_space_independent_of_catalog_order() {
    for (grain, coins, space, expected) in
        [(6, 16, 8, 1), (8, 8, 8, 1), (8, 16, 4, 1), (8, 16, 8, 2)]
    {
        let run = |reverse| {
            let (mut w, mut s) = fixture(true, space);
            w.storage.capacities.insert(PERSON, grain);
            s.balances.insert((PERSON, GRAIN), grain);
            s.balances.insert((BUYER, TOKEN), coins);
            w.recovery.inventory_listings.push(Listing {
                id: 2,
                proceeding: 1,
                goods: Amount::new(GRAIN, 4),
                minimum_price: 6,
            });
            w.recovery.inventory_bids.retain(|b| b.id == 2);
            w.recovery.inventory_bids.push(Bid {
                id: 4,
                listing: 2,
                buyer: BUYER,
                month: 3,
                price: 8,
            });
            if reverse {
                w.recovery.inventory_listings.reverse();
                w.recovery.inventory_bids.reverse();
            }
            let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
            sim.run_months(1).unwrap();
            sim.state.balances.insert((PERSON, TOKEN), 0);
            sim.run_months(2).unwrap();
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1]
                    .sold_inventory
                    .len(),
                expected
            );
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 8 * expected as i32);
            assert_eq!(sim.state.balance(BUYER, GRAIN), 4 * expected as i32);
            assert_eq!(
                sim.state.balance(PERSON, GRAIN),
                grain - 4 * expected as i32
            );
            (sim.state, sim.ledger)
        };
        assert_eq!(run(false), run(true));
    }
}

#[test]
fn household_agent_buys_with_separate_books_while_unadapted_member_purchase_is_rejected() {
    use economics_compute_smoke::{
        household_governance::Governance,
        households::{self, Agreement},
    };
    const HOME: AgentId = 100;
    for collective in [false, true] {
        let (mut w, mut s) = fixture(true, 8);
        let mut member = scenario::baseline().0.participants[0].clone();
        member.agent = BUYER;
        member.needs.clear();
        member.capacity.quantity = 0;
        w.participants.push(member);
        households::form(
            &mut w,
            &s,
            Agreement {
                id: 1,
                agent: HOME,
                adults: vec![BUYER],
                governance: Governance::contributed(BUYER),
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
        if collective {
            s.balances.insert((BUYER, TOKEN), 0);
            s.balances.insert((HOME, TOKEN), 8);
            for bid in &mut w.recovery.inventory_bids {
                if bid.buyer == BUYER {
                    bid.buyer = HOME;
                }
            }
        }
        let run = |backend| {
            let (mut sim, mut audit) = opening(w.clone(), s.clone(), backend);
            while sim.state.month < 5 {
                audit.step(&mut sim).unwrap();
            }
            assert_eq!(
                sim.state.balance(HOME, GRAIN),
                if collective { 4 } else { 0 }
            );
            assert_eq!(sim.state.balance(BUYER, GRAIN), 0);
            assert_eq!(
                sim.state.balance(STATE_AGENT, TOKEN),
                if collective { 8 } else { 0 }
            );
            for agent in &w.agents {
                let report = audit.book().statements(agent.id, 2, 4).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn common_inventory_offer_acceptance_is_dated_atomic_and_matches_normal_execution() {
    use economics_compute_smoke::offers::{self, Id, Request};
    for funded in [false, true] {
        let (w, s) = fixture(funded, 4);
        let (mut sim, _) = opening(w, s, Backend::CubeCpu);
        assert!(recovery::inventory::discover(&sim.world, &sim.state, BUYER).is_empty());
        while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
            sim.step().unwrap();
        }
        let request = Request::new(Id::InventoryLiquidationBid(2), BUYER);
        let discovered = offers::discover(&sim.world, &sim.state, BUYER);
        assert!(discovered.iter().any(|o| o.id == request.offer));
        for bad in [
            vec![Request::new(request.offer, PERSON)],
            vec![request.clone(), request.clone()],
            vec![Request {
                need: Some(GRAIN),
                ..request.clone()
            }],
        ] {
            assert!(offers::prepare(&sim, &bad).is_err());
        }
        let before = sim.clone();
        if !funded {
            assert!(offers::accept(&mut sim, &[request]).is_err());
            assert_eq!((&sim.state, &sim.ledger), (&before.state, &before.ledger));
            continue;
        }
        let prepared = offers::prepare(&sim, std::slice::from_ref(&request)).unwrap();
        assert_eq!((&sim.state, &sim.ledger), (&before.state, &before.ledger));
        let mut normal = sim.clone();
        normal.step().unwrap();
        offers::accept(&mut sim, std::slice::from_ref(&request)).unwrap();
        assert_eq!(sim.ledger.last(), Some(&prepared));
        assert_eq!((&sim.state, &sim.ledger), (&normal.state, &normal.ledger));
        assert!(recovery::inventory::discover(&sim.world, &sim.state, BUYER).is_empty());
        assert!(offers::accept(&mut sim, &[request]).is_err());
    }
}

#[test]
fn estate_inventory_sales_respect_explicit_current_essential_exemptions() {
    use economics_compute_smoke::commitments::{self, PaymentPolicy};
    let (w, s) = fixture(true, 4);
    let (mut opening, _) = self::opening(w, s, Backend::Reference);
    while (opening.state.month, opening.state.phase) != (3, Phase::Acquire) {
        opening.step().unwrap();
    }
    // Identical opening property and claims; only the collection exemption varies.
    let baseline = scenario::baseline().0;
    opening.world.participants = baseline.participants;
    opening.world.participants[0].needs[0].quantity = 3;
    opening.world.definitions = baseline
        .definitions
        .into_iter()
        .filter(|d| d.id == scenario::CONSUME)
        .collect();
    scenario::add_condition_rules(&mut opening.world, "dead");
    for policy in [PaymentPolicy::DebtFirst, PaymentPolicy::ProtectEssentials] {
        let mut sim = opening.clone();
        sim.world.payment_policy = policy;
        let protected = commitments::protected_stock(&sim.world, &sim.state).unwrap();
        assert_eq!(
            protected.get(&(PERSON, GRAIN)).copied().unwrap_or(0),
            if policy == PaymentPolicy::ProtectEssentials {
                3
            } else {
                0
            }
        );
        sim.step().unwrap();
        assert_eq!(
            sim.state.balance(BUYER, GRAIN),
            if policy == PaymentPolicy::ProtectEssentials {
                0
            } else {
                4
            }
        );
    }
}
