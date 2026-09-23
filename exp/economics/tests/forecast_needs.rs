use economics_compute_smoke::{forecast::needs, model::Requirement};
use std::collections::BTreeMap;

#[test]
fn multiple_needs_keep_units_priorities_and_absolute_limits_separate() {
    let mut requirements = vec![
        Requirement {
            resource: 4,
            quantity: 1,
            priority: 1,
        },
        Requirement {
            resource: 9,
            quantity: 1,
            priority: 0,
        },
        Requirement {
            resource: 2,
            quantity: 1,
            priority: 1,
        },
    ];
    let limits = BTreeMap::from([(2, 3), (4, 5), (9, 0)]);
    assert!(needs::valid_limits(&requirements, &limits));
    let mut deficits = BTreeMap::new();
    needs::accumulate(&mut deficits, [(4, 2), (9, 1), (2, 3)]);
    needs::accumulate(&mut deficits, [(4, 4), (9, 0), (2, 0)]);
    assert_eq!(needs::score(&requirements, &deficits), vec![1, 3, 6]);
    assert!(!needs::within_limits(&deficits, &limits));
    assert_eq!(
        needs::first_violation(&requirements, &deficits, &limits),
        Some((9, 1, 0))
    );
    requirements.reverse();
    assert_eq!(needs::score(&requirements, &deficits), vec![1, 3, 6]);
    assert_eq!(
        needs::first_violation(&requirements, &deficits, &limits),
        Some((9, 1, 0))
    );
    deficits.insert(9, 0);
    assert_eq!(
        needs::first_violation(&requirements, &deficits, &limits),
        Some((4, 6, 5))
    );
    deficits.insert(4, 5);
    assert!(needs::within_limits(&deficits, &limits));
    assert_eq!(
        needs::first_violation(&requirements, &deficits, &limits),
        None
    );
    assert!(!needs::valid_limits(
        &requirements,
        &BTreeMap::from([(5, 0)])
    ));
    assert!(!needs::valid_limits(
        &requirements,
        &BTreeMap::from([(2, -1)])
    ));
    // Empty-limit acceptance is domain policy; the common constraint is vacuous.
    assert!(needs::valid_limits(&requirements, &BTreeMap::new()));
    assert!(needs::within_limits(&deficits, &BTreeMap::new()));
}

#[test]
fn cumulative_needs_use_wide_units() {
    let mut deficits = BTreeMap::new();
    for _ in 0..24 {
        needs::accumulate(&mut deficits, [(1, i32::MAX)]);
    }
    assert_eq!(deficits[&1], 24 * i64::from(i32::MAX));
}
