use economics_compute_smoke::{
    accounting::{Account as A, Flow},
    compute::Backend,
    financial_reporting::Audit,
    model::*,
    scenario::{self, PERSON, PLOT, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
const OTHER: AgentId = 97;
const BUYER: AgentId = 98;
const ESTATE: AgentId = 99;

fn fixture(quantity: i32) -> Simulation {
    let (mut w, mut s) = scenario::baseline();
    w.participants.clear();
    w.definitions.clear();
    w.rights.clear();
    w.agreements.clear();
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
    s.balances.clear();
    s.month = 13;
    s.balances.insert((PERSON, scenario::GRAIN), quantity);
    with_accepted_forward(Simulation::new(w, s, Backend::Reference).unwrap(), 13)
}
fn audit(sim: &Simulation) -> Audit {
    let costs = sim
        .state
        .balances
        .iter()
        .filter(|((_, r), q)| *r != TOKEN && **q > 0)
        .map(|(key, q)| (*key, i128::from(*q) * 2))
        .collect();
    Audit::with_inventory(
        &sim.world,
        &sim.state,
        TOKEN,
        BTreeMap::from([(PLOT, 0), (9000, 2)]),
        costs,
    )
    .unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
fn with_accepted_forward(mut sim: Simulation, due: u32) -> Simulation {
    use economics_compute_smoke::{
        currency::Bid,
        equipment::DurableAsset,
        exchange::{Delivery, Market, ToolRequest},
        forward::{Contract, Policy, Price, Projection, Purchase},
    };
    use std::collections::BTreeMap;
    let id = 9000;
    let goods = scenario::GRAIN;
    for agent in [PERSON, BUYER] {
        sim.world.participants.push(Participant {
            agent,
            capacity: Amount::new(scenario::LABOR, 0),
            needs: vec![],
        });
    }
    let mut definition = scenario::baseline().0.definitions[0].clone();
    definition.enabled = false;
    definition.outputs.clear();
    sim.world.activities.outcomes.insert(
        definition.id,
        economics_compute_smoke::activities::Outcome::Create(1),
    );
    sim.world.activities.kinds.insert(
        1,
        economics_compute_smoke::activities::DurableKind {
            name: "accepted tool".into(),
            lifetime: 4,
            attached: false,
            monthly_decay: 0,
        },
    );
    sim.world.definitions.push(definition);
    let c = Contract {
        id,
        debtor: PERSON,
        creditor: OTHER,
        issued: due - 12,
        due,
        goods: Amount::new(goods, 4),
        advance: Amount::new(TOKEN, 2),
        price: Price { goods: 2, coins: 1 },
        delivered: 0,
        relief: vec![],
    };
    sim.world.bids.push(Bid {
        id: 1,
        buyer: OTHER,
        goods: Amount::new(goods, 1),
        payment: Amount::new(TOKEN, 1),
    });
    sim.world.market = Some(Market {
        capture_percent: 25,
        cash: Some(Policy {
            lender: OTHER,
            coin: TOKEN,
            months: 12,
            enabled: true,
            prices: BTreeMap::from([(goods, Price { goods: 1, coins: 1 })]),
            advance_prices: BTreeMap::from([(goods, Price { goods: 2, coins: 1 })]),
            protected: BTreeMap::new(),
        }),
        tools: vec![ToolRequest {
            buyer: PERSON,
            provider: BUYER,
            kind: 1,
        }],
        ..Default::default()
    });
    sim.state.equipment.insert(
        id,
        DurableAsset {
            id,
            owner: PERSON,
            kind: 1,
            remaining_uses: 4,
            attached_to: None,
            last_used_month: None,
        },
    );
    sim.state.exchange.forwards.insert(id, c.clone());
    sim.state.exchange.contracts.insert(
        id,
        Delivery {
            asset: id,
            buyer: PERSON,
            provider: BUYER,
            capture_percent: 0,
            purchase: Some(Purchase {
                price: Amount::new(TOKEN, 2),
                projection: Projection {
                    incremental_value: 4,
                    from: due - 12,
                    through: due - 1,
                    assisted: BTreeMap::from([(goods, 8)]),
                    surplus: BTreeMap::from([(goods, 4)]),
                },
                advance: Some(c),
            }),
        },
    );
    Simulation::new(sim.world, sim.state, Backend::Reference).unwrap()
}

#[test]
fn partial_delivery_recognizes_sales_cost_and_prepaid_inventory_without_cash() {
    for quantity in [1, 3, 4] {
        let mut sim = fixture(quantity);
        let mut a = audit(&sim);
        let mut cpu =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
        let mut b = a.clone();
        through(&mut a, &mut sim, 13);
        through(&mut b, &mut cpu, 13);
        assert_eq!(a, b);
        assert_eq!(sim.state, cpu.state);
        let producer = a.book().statements(PERSON, 13, 13).unwrap();
        let creditor = a.book().statements(OTHER, 13, 13).unwrap();
        let value = i128::from(quantity) / 2;
        assert_eq!(producer.income.get(&A::Sales).copied().unwrap_or(0), value);
        assert_eq!(producer.expenses[&A::CostOfSales], i128::from(quantity) * 2);
        assert_eq!(producer.liabilities, 2 - value);
        assert_eq!(
            creditor
                .trial_balance
                .get(&A::Inventory(scenario::GRAIN))
                .copied()
                .unwrap_or(0),
            value
        );
        assert_eq!(creditor.assets, 2);
        assert_eq!(producer.cash_flows.values().sum::<i128>(), 0);
        let mut resumed = a.clone();
        let mut checkpoint = sim.clone();
        through(&mut a, &mut sim, 15);
        through(&mut resumed, &mut checkpoint, 15);
        assert_eq!(a, resumed);
        assert_eq!(a.book().statements(PERSON, 14, 15).unwrap().net_income, 0);
    }
}
#[test]
fn forward_and_spot_delivery_share_opening_inventory_cost() {
    let mut sim = fixture(5);
    sim.state.balances.insert((OTHER, TOKEN), 1);
    let m = sim.world.market.as_mut().unwrap();
    m.targets.insert(1, 5);
    m.reserves.insert((PERSON, scenario::GRAIN), 0);
    let mut a = audit(&sim);
    through(&mut a, &mut sim, 13);
    assert_eq!(sim.state.balance(PERSON, scenario::GRAIN), 0);
    assert_eq!(sim.state.balance(OTHER, scenario::GRAIN), 5);
    let seller = a.book().statements(PERSON, 13, 13).unwrap();
    assert_eq!(seller.income[&A::Sales], 3); // prepaid 2 + actual spot cash 1
    assert_eq!(seller.expenses[&A::CostOfSales], 10);
    let buyer = a.book().statements(OTHER, 13, 13).unwrap();
    assert_eq!(buyer.trial_balance[&A::Inventory(scenario::GRAIN)], 3);
    assert_eq!(buyer.cash_flows[&Flow::Operating], -1);
}
#[test]
fn extension_and_writeoff_preserve_delivery_provenance_and_release_residual_cost() {
    use economics_compute_smoke::{
        delivery_relief::{Action, Terms},
        recovery::ProceedingTerms,
    };
    let mut sim = fixture(3);
    sim.world.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 14,
        earliest_close: 14,
        assets: vec![],
        discharge_deficiency: true,
    });
    sim.world.recovery.delivery_relief = vec![
        Terms {
            id: 1,
            proceeding: 1,
            contract: 9000,
            debtor: PERSON,
            creditor: OTHER,
            month: 14,
            expected_due: 13,
            expected_remaining: 1,
            action: Action::Extend { due: 16 },
        },
        Terms {
            id: 2,
            proceeding: 1,
            contract: 9000,
            debtor: PERSON,
            creditor: OTHER,
            month: 17,
            expected_due: 16,
            expected_remaining: 1,
            action: Action::WriteOff { quantity: 1 },
        },
    ];
    sim = Simulation::new(sim.world, sim.state, Backend::CubeCpu).unwrap();
    let mut a = audit(&sim);
    through(&mut a, &mut sim, 14);
    assert_eq!(sim.state.exchange.forwards[&9000].effective_due(), 16);
    assert_eq!(a.book().statements(PERSON, 14, 14).unwrap().net_income, 0);
    through(&mut a, &mut sim, 17);
    let c = &sim.state.exchange.forwards[&9000];
    assert_eq!((c.delivered, c.written_off()), (3, 1));
    let debtor = a.book().statements(PERSON, 17, 17).unwrap();
    let creditor = a.book().statements(OTHER, 17, 17).unwrap();
    assert_eq!(debtor.liabilities, 0);
    assert_eq!(debtor.income[&A::DebtRelief], 1);
    assert_eq!(creditor.expenses[&A::CreditLoss], 1);
    assert_eq!(sim.state.balance(OTHER, scenario::GRAIN), 3);
    assert_eq!(creditor.cash_flows.values().sum::<i128>(), 0);
}

#[test]
fn actual_tool_purchase_recognizes_prepaid_asset_not_cash_or_income_for_producer() {
    use economics_compute_smoke::{crafts::HOE, equipment::DurableAsset, trading_scenario};
    for cash in [0, 100, 10000] {
        let (mut w, mut s) = trading_scenario::cash_scenario(1, true).unwrap();
        let market = w.market.as_mut().unwrap();
        market.tools.retain(|r| r.buyer == PERSON && r.kind == HOE);
        market.targets.clear();
        let provider = market.tools[0].provider;
        s.equipment.insert(
            9000,
            DurableAsset {
                id: 9000,
                owner: provider,
                kind: HOE,
                remaining_uses: 24,
                attached_to: None,
                last_used_month: None,
            },
        );
        for n in &mut w
            .participants
            .iter_mut()
            .find(|p| p.agent == PERSON)
            .unwrap()
            .needs
        {
            n.quantity = 0;
        }
        s.balances.insert((PERSON, TOKEN), cash);
        s.balances.insert((STATE_AGENT, TOKEN), 10000);
        let mut values: BTreeMap<_, _> = w.assets.iter().map(|a| (a.id, 0)).collect();
        values.extend(s.equipment.keys().map(|id| (*id, 0)));
        values.insert(9000, 5);
        let costs = s
            .balances
            .iter()
            .filter(|((_, r), q)| {
                *r != TOKEN
                    && **q > 0
                    && w.resources
                        .iter()
                        .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
            })
            .map(|(k, q)| (*k, i128::from(*q)))
            .collect();
        let mut a = Audit::with_inventory(&w, &s, TOKEN, values, costs).unwrap();
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        while sim.state.phase != Phase::Acquire {
            a.step(&mut sim).unwrap();
        }
        a.step(&mut sim).unwrap();
        let p = sim.state.exchange.contracts[&9000]
            .purchase
            .as_ref()
            .unwrap();
        let price = i128::from(p.price.quantity);
        let financed = p
            .advance
            .as_ref()
            .map_or(0, |c| i128::from(c.advance.quantity));
        if cash == 0 {
            assert_eq!(financed, price);
        }
        if cash == 100 {
            assert!(financed > 0);
        }
        if cash == 10000 {
            assert_eq!(financed, 0);
        }
        let producer = a.book().statements(PERSON, 1, 1).unwrap();
        let lender = a.book().statements(STATE_AGENT, 1, 1).unwrap();
        let seller = a.book().statements(provider, 1, 1).unwrap();
        assert_eq!(producer.trial_balance[&A::Tangible(9000)], price);
        assert_eq!(
            producer
                .trial_balance
                .get(&A::DeferredRevenue(9000))
                .copied()
                .unwrap_or(0),
            -financed
        );
        assert_eq!(producer.net_income, 0);
        assert_eq!(
            producer
                .cash_flows
                .get(&Flow::Investing)
                .copied()
                .unwrap_or(0),
            -(price - financed)
        );
        assert_eq!(
            lender
                .trial_balance
                .get(&A::ForwardPrepayment(9000))
                .copied()
                .unwrap_or(0),
            financed
        );
        assert_eq!(
            lender
                .cash_flows
                .get(&Flow::Operating)
                .copied()
                .unwrap_or(0),
            -financed
        );
        assert_eq!(seller.income[&A::DisposalGain], price - 5);
    }
}

#[test]
fn forged_delivery_does_not_publish_inventory_or_claim_changes() {
    let mut sim = fixture(4);
    let mut a = audit(&sim);
    while sim.state.phase != Phase::Acquire {
        a.step(&mut sim).unwrap();
    }
    let mut preview = sim.clone();
    preview.step().unwrap();
    let mut bad = preview.ledger.last().unwrap().clone();
    let tx = bad
        .transactions
        .iter_mut()
        .find(|t| t.forward.is_some())
        .unwrap();
    tx.effects[0].delta -= 1;
    let old = a.clone();
    assert!(
        a.record(&sim.world, &sim.state, &bad, &preview.state)
            .is_err()
    );
    assert_eq!(a, old);
    assert_eq!(sim.state.exchange.forwards[&9000].delivered, 0);
    a.step(&mut sim).unwrap();
    assert_eq!(sim.state.exchange.forwards[&9000].delivered, 4);
}
