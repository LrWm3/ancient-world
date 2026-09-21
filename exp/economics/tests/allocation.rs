use economics_compute_smoke::allocation::*;
fn context() -> Context {
    Context {
        seed: 7,
        pool: 1,
        round: 1,
    }
}
fn claim(id: u64, priority: u32, requested: u32, minimum: u32) -> Claim {
    Claim {
        id,
        priority,
        requested,
        minimum,
    }
}
#[test]
fn priority_lottery_is_order_independent_seeded_and_demand_capped() {
    let claims = vec![claim(8, 1, 1, 1), claim(9, 0, 1, 1), claim(10, 0, 1, 1)];
    let resolve = |c: Context, rows: &[Claim]| allocation(c, rows);
    let first = resolve(context(), &claims);
    let mut reversed = claims.clone();
    reversed.reverse();
    assert_eq!(first, resolve(context(), &reversed));
    assert_eq!(first[0].claim.priority, 0);
    assert_eq!(
        first
            .iter()
            .filter(|r| matches!(r.outcome, Outcome::Reserved(_)))
            .count(),
        1
    );
    let winners: std::collections::BTreeSet<_> = (0..64)
        .map(|seed| resolve(Context { seed, ..context() }, &claims)[0].claim.id)
        .collect();
    assert_eq!(winners, [9, 10].into_iter().collect());
}
fn allocation(c: Context, rows: &[Claim]) -> Vec<Receipt> {
    resolve(c, &Policy::PriorityLottery, 1, rows, |_, _| Ok(())).unwrap()
}
#[test]
fn unusable_grants_and_failed_reservations_leave_capacity_for_next_claim() {
    let rows = [claim(1, 0, 4, 4), claim(2, 0, 2, 1), claim(3, 0, 2, 1)];
    let result = resolve(context(), &Policy::StablePriority, 3, &rows, |c, _| {
        if c.id == 2 {
            Err("participant unavailable".into())
        } else {
            Ok(())
        }
    })
    .unwrap();
    assert_eq!(result[0].outcome, Outcome::InsufficientCapacity);
    assert!(matches!(result[1].outcome, Outcome::Rejected(_)));
    assert_eq!(result[2].outcome, Outcome::Reserved(2));
    assert!(
        resolve(
            context(),
            &Policy::Lottery,
            1,
            &[rows[0].clone(), rows[0].clone()],
            |_, _| Ok(())
        )
        .is_err()
    );
    assert!(
        resolve(context(), &Policy::Lottery, 0, &[], |_, _| Ok(()))
            .unwrap()
            .is_empty()
    );
}
#[test]
fn callers_can_supply_a_different_policy_without_domain_dependencies() {
    struct Descending;
    impl RankingPolicy for Descending {
        fn key(&self, _: Context, c: &Claim) -> (u32, u64) {
            (0, u64::MAX - c.id)
        }
    }
    let result = resolve(
        context(),
        &Descending,
        2,
        &[claim(1, 0, 4, 1), claim(2, 0, 3, 1)],
        |_, _| Ok(()),
    )
    .unwrap();
    assert_eq!(result[0].claim.id, 2);
    assert_eq!(result[0].outcome, Outcome::Reserved(2));
}

#[test]
fn alternatives_reassign_flexible_claimants_without_losing_priority_or_double_awarding() {
    let choices = vec![
        Alternatives {
            claim: claim(1, 0, 1, 1),
            slots: vec![10, 20],
        },
        Alternatives {
            claim: claim(2, 0, 1, 1),
            slots: vec![10],
        },
        Alternatives {
            claim: claim(3, 1, 1, 1),
            slots: vec![10, 20],
        },
    ];
    let (receipts, awards) = assign(context(), &Policy::StablePriority, &choices).unwrap();
    assert_eq!(awards, [(1, 20), (2, 10)].into_iter().collect());
    assert_eq!(receipts[2].outcome, Outcome::InsufficientCapacity);
    let mut reversed = choices;
    reversed.reverse();
    for a in &mut reversed {
        a.slots.reverse();
    }
    assert_eq!(
        (receipts, awards),
        assign(context(), &Policy::StablePriority, &reversed).unwrap()
    );
}
