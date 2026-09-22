use economics_compute_smoke::{
    allocation::{Claim, Context, Outcome, Policy},
    resolution::{self, Mechanism, Request},
};
use std::collections::BTreeMap;

fn request(id: u64, lots: u32, inputs: &[((u32, u32), u32)]) -> Request {
    Request {
        claim: Claim {
            id,
            priority: 0,
            requested: lots,
            minimum: 1,
        },
        inputs: inputs.iter().copied().collect(),
    }
}
const CONTEXT: Context = Context {
    seed: 7,
    pool: 1,
    round: 2,
};

#[test]
fn same_requests_different_mechanisms_and_no_partial_bundle_hold() {
    let stock = BTreeMap::from([((0, 1), 3), ((0, 2), 1), ((1, 3), 2), ((2, 3), 1)]);
    let requests = vec![
        request(1, 2, &[((0, 1), 1), ((0, 2), 1), ((1, 3), 1)]),
        request(2, 1, &[((0, 1), 2), ((0, 2), 1), ((2, 3), 1)]),
    ];
    let run = |mechanism| {
        resolution::resolve(
            CONTEXT,
            &Policy::StablePriority,
            mechanism,
            &stock,
            &requests,
            &Vec::new(),
            |plan, c, n| {
                plan.push((c.id, n));
                Ok(())
            },
        )
        .unwrap()
    };
    let (immediate, accepted) = run(Mechanism::Immediate);
    assert_eq!(accepted, vec![(1, 1)]);
    assert_eq!(immediate.remaining[&(0, 1)], 2);
    let (bundle, accepted) = run(Mechanism::ConditionalBundle);
    assert_eq!(accepted, vec![(2, 1)]);
    assert_eq!(bundle.remaining[&(1, 3)], 2); // losing plan retained no labor
    assert_eq!(bundle.remaining[&(0, 1)], 1);
    assert_eq!(bundle.shortfalls[&1][0].account, (0, 2));
    assert_eq!(bundle.shortfalls[&1][0].required, 2);
    assert_eq!(bundle.receipts[0].outcome, Outcome::InsufficientCapacity);
}

#[test]
fn rejected_domain_check_discards_candidate_and_all_prerequisites() {
    let stock = BTreeMap::from([((0, 1), 2), ((0, 2), 1)]);
    let requests = vec![
        request(1, 1, &[((0, 1), 2), ((0, 2), 1)]),
        request(2, 1, &[((0, 1), 2), ((0, 2), 1)]),
    ];
    let (r, accepted) = resolution::resolve(
        CONTEXT,
        &Policy::StablePriority,
        Mechanism::ConditionalBundle,
        &stock,
        &requests,
        &vec![],
        |plan, c, _| {
            plan.push(c.id);
            if c.id == 1 {
                Err("permission denied".into())
            } else {
                Ok(())
            }
        },
    )
    .unwrap();
    assert_eq!(accepted, vec![2]);
    assert!(matches!(r.receipts[0].outcome, Outcome::Rejected(_)));
    assert!(r.remaining.values().all(|q| *q == 0));
}

#[test]
fn missing_dependency_empty_demand_and_order_are_explicit() {
    let stock = BTreeMap::from([((0, 1), 2)]);
    let mut requests = vec![request(1, 1, &[((0, 2), 1)]), request(2, 1, &[((0, 1), 2)])];
    let run = |rs: &[Request]| {
        resolution::resolve(
            CONTEXT,
            &Policy::Lottery,
            Mechanism::ConditionalBundle,
            &stock,
            rs,
            &(),
            |_, _, _| Ok(()),
        )
        .unwrap()
    };
    let expected = run(&requests);
    requests.reverse();
    assert_eq!(expected, run(&requests));
    assert_eq!(expected.0.shortfalls[&1][0].available, 0);
    assert_eq!(run(&[]).0.remaining, stock);
    requests.push(requests[0].clone());
    assert!(
        resolution::resolve(
            CONTEXT,
            &Policy::Lottery,
            Mechanism::Immediate,
            &stock,
            &requests,
            &(),
            |_, _, _| Ok(())
        )
        .is_err()
    );
}
