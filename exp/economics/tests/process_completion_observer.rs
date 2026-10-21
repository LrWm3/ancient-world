use economics_compute_smoke::{
    acceptance::{continuation::Case, financing},
    agency::completions::observe,
    model::*,
};

fn batch() -> Batch {
    let (_, s) = financing::scenario(Case::Baseline).unwrap();
    Batch::empty(&s)
}
fn transaction(id: u64, status: Status) -> Transaction {
    Transaction {
        cause: "observer fixture".into(),
        effects: vec![],
        forward: None,
        delivery: None,
        royalty: None,
        technique_use: None,
        trade: None,
        stock_trade: None,
        process: Some(ProcessChange {
            before: None,
            after: ProcessInstance {
                id,
                definition: 3,
                operator: 7,
                beneficiary: 8,
                goal: None,
                asset: None,
                right: None,
                start: 1,
                reserved_through: 10,
                stage: 0,
                elapsed: 1,
                status,
            },
        }),
    }
}
#[test]
fn counts_committed_completions_once_at_completion_date_for_operator() {
    let mut b = batch();
    b.month = 5;
    b.transactions = vec![
        transaction(2, Status::Completed),
        transaction(1, Status::Completed),
    ];
    let mut earlier = b.clone();
    earlier.month = 4;
    earlier.transactions.truncate(1);
    let rows = observe(&[b.clone(), earlier.clone(), b.clone()], 7, 1, 5);
    assert_eq!(
        rows.iter()
            .map(|r| (r.month, r.process))
            .collect::<Vec<_>>(),
        [(4, 2), (5, 1)]
    );
    assert_eq!(rows, observe(&[earlier.clone(), b.clone()], 7, 1, 5));
    assert_eq!(observe(&[b.clone(), earlier], 7, 5, 5).len(), 1);
    assert!(observe(&[b.clone()], 8, 1, 5).is_empty());
    assert!(observe(&[b], 7, 6, 10).is_empty());
}
#[test]
fn planned_failed_and_already_completed_work_are_not_new_completions() {
    let mut plan = batch();
    plan.transactions.push(transaction(1, Status::Completed));
    let mut b = batch();
    b.production_plan = Some(Box::new(plan));
    b.transactions = vec![
        transaction(2, Status::Active),
        transaction(3, Status::Aborted),
    ];
    let mut old = transaction(4, Status::Completed);
    let change = old.process.as_mut().unwrap();
    change.before = Some(change.after.clone());
    b.transactions.push(old);
    assert!(observe(&[b], 7, 1, 10).is_empty());
}
#[test]
fn employment_transactions_are_included() {
    let mut b = batch();
    b.employment = Some(economics_compute_smoke::employment::Boundary {
        supply: vec![],
        recruitment: vec![],
        after: Default::default(),
        receipts: vec![],
        transactions: vec![transaction(1, Status::Completed)],
    });
    assert_eq!(observe(&[b], 7, 1, 10).len(), 1);
}
