//! Cost-only composition tests. The scheduler still admits each transaction at
//! its existing phase; these tests do not fabricate a mixed simulation batch.
use crate::{
    accounting::{Account as A, Book, Entry, Line},
    commitments::{Agreement, Obligation},
    dues_accounting::Valuation,
    inventory_accounting::{CostAllocation, Holding, Inventory},
    model::*,
    process_accounting::Costs,
    scenario::{self, GRAIN, GROW, PERSON, STATE_AGENT, TOKEN},
};
use std::collections::BTreeMap;
fn transaction(effects: Vec<Effect>) -> Transaction {
    Transaction {
        cause: "cost fixture".into(),
        effects,
        process: None,
        technique_use: None,
        trade: None,
        stock_trade: None,
        forward: None,
        delivery: None,
        royalty: None,
    }
}
fn transfer(from: AgentId, to: AgentId, r: ResourceId, q: i32) -> Vec<Effect> {
    vec![
        Effect {
            account: (from, r),
            delta: -q,
        },
        Effect {
            account: (to, r),
            delta: q,
        },
    ]
}
fn sale(from: AgentId, to: AgentId, price: i32) -> Transaction {
    let mut effects = transfer(from, to, GRAIN, 1);
    effects.extend(transfer(to, from, TOKEN, price));
    transaction(effects)
}
fn combined(reverse: bool) -> (Inventory, Costs, Book) {
    let (mut w, mut before) = scenario::baseline();
    w.agreements.push(Agreement {
        id: 1,
        right: 1,
        creditor: STATE_AGENT,
        debtor: PERSON,
        activated: 1,
        payment: Amount::new(GRAIN, 1),
    });
    before.month = 13;
    let mut after = before.clone();
    after.obligations.insert(
        (1, 13),
        Obligation {
            agreement: 1,
            due: 13,
            owed: 1,
            paid: 1,
            in_kind_paid: 1,
        },
    );
    let opening = Inventory(BTreeMap::from([
        (
            (PERSON, GRAIN),
            Holding {
                quantity: 3,
                cost: 10,
            },
        ),
        (
            (STATE_AGENT, GRAIN),
            Holding {
                quantity: 2,
                cost: 100,
            },
        ),
    ]));
    let mut allocator = CostAllocation::new(&opening);
    let a = sale(PERSON, STATE_AGENT, 6);
    let b = sale(STATE_AGENT, PERSON, 50);
    let trades = if reverse { vec![&b, &a] } else { vec![&a, &b] };
    let (inventory, mut lines) = opening
        .settle_allocated(&trades, TOKEN, &[], &mut allocator)
        .unwrap();
    let mut work = transaction(vec![Effect {
        account: (PERSON, GRAIN),
        delta: -1,
    }]);
    work.process = Some(ProcessChange {
        before: None,
        after: ProcessInstance {
            id: 1,
            definition: GROW,
            operator: PERSON,
            beneficiary: PERSON,
            goal: None,
            asset: None,
            right: None,
            start: 13,
            reserved_through: 18,
            stage: 0,
            elapsed: 1,
            status: Status::Active,
        },
    });
    w.definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .stages[0]
        .entry_inputs = vec![Amount::new(GRAIN, 1)];
    let (costs, inventory, process_lines) = Costs::default()
        .settle_allocated(
            &w,
            &inventory,
            &[&work],
            TOKEN,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            &mut allocator,
        )
        .unwrap();
    lines.extend(process_lines);
    let (inventory, dues_lines) = Valuation(BTreeMap::from([(1, 4)]))
        .settle_allocated(
            &w,
            &before,
            &after,
            &inventory,
            TOKEN,
            &[transaction(transfer(PERSON, STATE_AGENT, GRAIN, 1))],
            &mut allocator,
        )
        .unwrap();
    lines.extend(dues_lines);
    assert_eq!(costs.work[&1], (PERSON, 3));
    assert_eq!(
        inventory.0[&(PERSON, GRAIN)],
        Holding {
            quantity: 1,
            cost: 50
        }
    );
    assert_eq!(
        inventory.0[&(STATE_AGENT, GRAIN)],
        Holding {
            quantity: 3,
            cost: 60
        }
    );
    // The fresh receipt remains, but every opening unit was already allocated.
    assert!(allocator.take((PERSON, GRAIN), 1).is_err());
    let mut book = Book::open(
        TOKEN,
        BTreeMap::from([
            ((PERSON, A::Cash), 100),
            ((STATE_AGENT, A::Cash), 100),
            ((PERSON, A::Inventory(GRAIN)), 10),
            ((STATE_AGENT, A::Inventory(GRAIN)), 100),
        ]),
    )
    .unwrap();
    for agent in [PERSON, STATE_AGENT] {
        lines.push(Line {
            agent,
            account: A::Inventory(GRAIN),
            debit: inventory.0[&(agent, GRAIN)].cost - opening.0[&(agent, GRAIN)].cost,
            flow: None,
        });
    }
    lines.push(Line {
        agent: PERSON,
        account: A::WorkInProgress(1),
        debit: 3,
        flow: None,
    });
    book.post(Entry {
        id: "combined".into(),
        month: 13,
        batch: None,
        description: "shared opening-cost allocation".into(),
        lines,
    })
    .unwrap();
    assert_eq!(book.statements(PERSON, 13, 13).unwrap().net_income, -1);
    assert_eq!(book.statements(STATE_AGENT, 13, 13).unwrap().net_income, 4);
    (inventory, costs, book)
}
#[test]
fn trades_work_and_dues_share_basis_without_spending_new_receipts() {
    let (inventory, costs, book) = combined(false);
    let (reordered_inventory, reordered_costs, reordered_book) = combined(true);
    assert_eq!(inventory, reordered_inventory);
    assert_eq!(costs, reordered_costs);
    assert_eq!(book.balances(), reordered_book.balances());
}
#[test]
fn failed_allocation_preserves_cursor_and_final_depletion_releases_remainder() {
    let inventory = Inventory(BTreeMap::from([(
        (PERSON, GRAIN),
        Holding {
            quantity: 3,
            cost: 10,
        },
    )]));
    let mut allocation = CostAllocation::new(&inventory);
    assert_eq!(allocation.take((PERSON, GRAIN), 1).unwrap(), 3);
    assert!(allocation.take((PERSON, GRAIN), 3).is_err());
    assert!(allocation.take((PERSON, GRAIN), 0).is_err());
    assert_eq!(allocation.take((PERSON, GRAIN), 2).unwrap(), 7);
}

#[test]
fn representable_cost_shares_do_not_fail_on_intermediate_multiplication() {
    let inventory = Inventory(BTreeMap::from([(
        (PERSON, GRAIN),
        Holding {
            quantity: 3,
            cost: i128::MAX,
        },
    )]));
    let mut allocation = CostAllocation::new(&inventory);
    let first = allocation.take((PERSON, GRAIN), 1).unwrap();
    let last = allocation.take((PERSON, GRAIN), 2).unwrap();
    assert_eq!(first, i128::MAX / 3);
    assert_eq!(first.checked_add(last), Some(i128::MAX));
}
