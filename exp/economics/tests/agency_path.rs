use economics_compute_smoke::{
    acceptance::{self, WORKSHOP, continuation::Case, financing},
    agency::path::Path,
    minting::COIN,
    model::Phase,
};

#[test]
fn completed_month_path_keeps_temporary_stock_losses_and_requires_exact_dates() {
    let (w, mut s) = financing::scenario(Case::Baseline).unwrap();
    let mut path = Path::new(s.month);
    assert!(path.record(&w, &s).is_err());
    s.month = 2;
    s.balances.insert((WORKSHOP, COIN), 0);
    path.record(&w, &s).unwrap();
    assert!(path.record(&w, &s).is_err());
    s.month = 3;
    s.phase = Phase::Acquire;
    assert!(path.record(&w, &s).is_err());
    s.phase = Phase::Open;
    s.balances.insert((WORKSHOP, COIN), 10);
    path.record(&w, &s).unwrap();
    assert_eq!(path.minimum(WORKSHOP, COIN), Some(0));
    assert_eq!(path.closes[1].balance(WORKSHOP, COIN), 10);
    assert_eq!(
        path.closes.iter().map(|c| c.month).collect::<Vec<_>>(),
        [1, 2]
    );
    assert!(serde_json::to_string(&path).is_ok());
    assert_eq!(
        path.minimum(acceptance::TOWN, COIN),
        Some(s.balance(acceptance::TOWN, COIN))
    );
}

#[test]
fn paid_later_wages_remain_in_the_earlier_completed_month() {
    use economics_compute_smoke::{
        employment::Earned,
        finance::{Condition, FailureRule, Obligation, Transfer},
        model::Amount,
    };
    let (w, mut s) = financing::scenario(Case::Baseline).unwrap();
    let mut path = Path::new(1);
    s.employment.earned.insert(
        (99, 1),
        Earned {
            delivered: 1,
            relief: vec![],
            claim: Obligation {
                transfer: Transfer {
                    from: WORKSHOP,
                    to: acceptance::SMITH,
                    amount: Amount::new(COIN, 2),
                },
                settled: 0,
                condition: Condition::OnOrAfterMonth(2),
                failure: FailureRule::CarryArrears,
            },
        },
    );
    s.month = 2;
    path.record(&w, &s).unwrap();
    assert!(
        path.closes[0].unpaid.is_empty(),
        "not due in completed month 1"
    );
    s.month = 3;
    path.record(&w, &s).unwrap();
    s.employment.earned.get_mut(&(99, 1)).unwrap().claim.settled = 2;
    s.month = 4;
    path.record(&w, &s).unwrap();
    assert_eq!(path.closes[1].unpaid[0].quantity, 2);
    assert_eq!(path.closes[1].unpaid[0].resource, COIN);
    assert!(path.closes[2].unpaid.is_empty());
    assert!(serde_json::to_string(&path).unwrap().contains("earned"));
}

#[test]
fn exposure_distinguishes_recovered_shortage_from_terminal_reserve() {
    use economics_compute_smoke::agency::objectives::{self, Metric, Objective, Scope};
    let (w, mut s) = financing::scenario(Case::Baseline).unwrap();
    let goals = vec![
        Objective {
            scope: Scope::Organization,
            metric: Metric::Reserve {
                resource: COIN,
                target: 10,
            },
        },
        Objective {
            scope: Scope::Organization,
            metric: Metric::ReserveExposure {
                resource: COIN,
                target: 10,
            },
        },
    ];
    objectives::validate(&w, &goals).unwrap();
    let mut path = Path::new(1);
    s.month = 2;
    s.balances.insert((WORKSHOP, COIN), 0);
    path.record(&w, &s).unwrap();
    s.month = 3;
    s.balances.insert((WORKSHOP, COIN), 10);
    path.record(&w, &s).unwrap();
    assert_eq!(
        objectives::measure(&w, &s, &[], WORKSHOP, &goals).unwrap(),
        [0, 0]
    );
    assert_eq!(
        objectives::measure_projection(&w, &s, &[], WORKSHOP, &goals, &path).unwrap(),
        [0, 10]
    );
    let invalid = [Objective {
        scope: Scope::Organization,
        metric: Metric::ReserveExposure {
            resource: COIN,
            target: -1,
        },
    }];
    assert!(objectives::validate(&w, &invalid).is_err());
    assert!(objectives::measure_projection(&w, &s, &[], WORKSHOP, &goals, &Path::new(1)).is_err());
}
