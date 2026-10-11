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

#[test]
fn oversized_sale_catalog_is_rejected_and_unsigned_recipes_are_not_added() {
    let (mut w, s) = fixture(0).unwrap();
    let previous = w.agency.remove(&WORKSHOP).unwrap();
    let a = previous.operating_agreement().unwrap().clone();
    let mut unrelated = w
        .definitions
        .iter()
        .find(|d| d.id == REFINE)
        .unwrap()
        .clone();
    unrelated.id = 9000;
    w.definitions.push(unrelated);
    organization::install_discovering_initial(
        &mut w,
        &s,
        a.clone(),
        &a.members,
        previous.config.clone(),
    )
    .unwrap();
    assert!(
        agency::discovery_sales::opportunities(&w, WORKSHOP)
            .unwrap()
            .iter()
            .all(|o| o.definition == REFINE)
    );
    let previous = w.agency.remove(&WORKSHOP).unwrap();
    let mut a = previous.operating_agreement().unwrap().clone();
    for id in 9001..9018 {
        let mut d = w
            .definitions
            .iter()
            .find(|d| d.id == REFINE)
            .unwrap()
            .clone();
        d.id = id;
        w.definitions.push(d);
        a.processes.insert(id);
    }
    let before = w.clone();
    assert!(
        organization::install_discovering_initial(
            &mut w,
            &s,
            a.clone(),
            &a.members,
            previous.config
        )
        .is_err()
    );
    assert_eq!(w, before);
}

#[test]
fn explicit_inventory_objectives_keep_their_original_work_target() {
    use economics_compute_smoke::activities::Target;
    let (mut w, _) = fixture(0).unwrap();
    w.agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .config
        .objectives
        .insert(
            0,
            Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: METAL,
                    target: 7,
                },
            },
        );
    let catalog = agency::discovery::programs(&w, WORKSHOP).unwrap();
    let orders: Vec<_> = catalog
        .values()
        .flat_map(|p| &p.commands)
        .filter_map(|c| {
            if let Command::WorkTargets(o) = c {
                Some(o)
            } else {
                None
            }
        })
        .flatten()
        .filter(|o| o.definition == REFINE)
        .collect();
    assert!(!orders.is_empty());
    assert!(
        orders
            .iter()
            .all(|o| o.target == Target::Stock(Amount::new(METAL, 7)))
    );
}
