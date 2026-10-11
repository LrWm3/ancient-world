use economics_compute_smoke::{
    acceptance::{WORKSHOP, continuation as c, expectations},
    agency::{
        self, Command,
        markets::Options,
        objectives::{Metric, Objective, Scope},
        organization,
    },
    marketplace::Side,
    minting::{COIN, METAL, REFINE},
    model::*,
};
fn fixture(floor: i32) -> Result<(World, State), String> {
    let (mut w, s) = expectations::scenario(c::Case::Baseline)?;
    let previous = w.agency.remove(&WORKSHOP).unwrap();
    let mut a = previous.operating_agreement().unwrap().clone();
    a.stock_markets.insert(
        METAL,
        Options {
            side: Side::Sell,
            limits: [4].into(),
            holdings: [floor].into(),
            max_lots: 1,
        },
    );
    let mut config = previous.config;
    config.objectives = vec![Objective {
        scope: Scope::Organization,
        metric: Metric::Reserve {
            resource: COIN,
            target: 1000,
        },
    }];
    organization::install_discovering_initial(&mut w, &s, a.clone(), &a.members, config)?;
    Ok((w, s))
}
#[test]
fn cash_only_catalog_derives_recurring_work_and_trade_bundles_from_signed_sales() {
    let (mut w, mut s) = fixture(0).unwrap();
    let evidence = agency::discovery_sales::opportunities(&w, WORKSHOP).unwrap();
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].definition, REFINE);
    assert_eq!(evidence[0].target, 2);
    let catalog = agency::discovery::programs(&w, WORKSHOP).unwrap();
    assert!(catalog.values().any(|p| {
        p.commands.len() == 2
            && p.commands
                .iter()
                .any(|cmd| matches!(cmd,Command::WorkTargets(v) if v[0].definition==REFINE))
    }));
    assert!(!catalog.values().any(|p| {
        p.commands
            .iter()
            .any(|cmd| matches!(cmd, Command::StartProcess(REFINE)))
    }));
    for p in catalog.values() {
        agency::validate_program(&w, WORKSHOP, p).unwrap();
    }
    s.balances.clear();
    w.definitions.reverse();
    w.marketplaces[0].markets.reverse();
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.quotes.clear();
    p.peer_stocks.as_mut().unwrap().quotes.clear();
    assert_eq!(agency::discovery::programs(&w, WORKSHOP).unwrap(), catalog);
    assert_eq!(
        agency::discovery_sales::opportunities(&w, WORKSHOP).unwrap(),
        evidence
    );
}
#[test]
fn sale_routes_require_payment_goals_and_target_arithmetic_is_checked() {
    assert!(fixture(i32::MAX).is_err());
    let (mut w, _) = fixture(3).unwrap();
    assert_eq!(
        agency::discovery_sales::opportunities(&w, WORKSHOP).unwrap()[0].target,
        5
    );
    w.agency.get_mut(&WORKSHOP).unwrap().config.objectives[0].metric = Metric::Reserve {
        resource: COIN,
        target: 0,
    };
    assert!(
        agency::discovery_sales::opportunities(&w, WORKSHOP)
            .unwrap()
            .is_empty()
    );
    w.agency.get_mut(&WORKSHOP).unwrap().config.objectives[0].metric = Metric::Reserve {
        resource: METAL,
        target: 10,
    };
    assert!(
        agency::discovery_sales::opportunities(&w, WORKSHOP)
            .unwrap()
            .is_empty()
    );
}
