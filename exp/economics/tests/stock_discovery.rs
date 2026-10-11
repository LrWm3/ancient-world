use economics_compute_smoke::{
    acceptance::{WORKSHOP, continuation as c},
    agency::{self, Command, markets::Options, organization},
    marketplace::Side,
    minting::ORE,
    model::*,
};
fn fixture(limits: &[i32], holdings: &[i32]) -> Result<(World, State), String> {
    let (mut w, s) = c::spot_scenario(c::Case::Baseline)?;
    let previous = w.agency.remove(&WORKSHOP).unwrap();
    let mut a = previous.operating_agreement().unwrap().clone();
    a.stock_markets.insert(
        ORE,
        Options {
            side: Side::Buy,
            limits: limits.iter().copied().collect(),
            holdings: holdings.iter().copied().collect(),
            max_lots: 2,
        },
    );
    let mut config = previous.config;
    config.programs.clear();
    organization::install_discovering_initial(&mut w, &s, a.clone(), &a.members, config)?;
    Ok((w, s))
}
#[test]
fn static_portfolios_include_stopping_and_signed_price_target_choices() {
    let (mut w, _) = fixture(&[2, 3], &[2, 4]).unwrap();
    let catalog = agency::discovery::programs(&w, WORKSHOP).unwrap();
    let quotes: Vec<_> = catalog
        .values()
        .flat_map(|p| &p.commands)
        .filter_map(|cmd| match cmd {
            Command::PeerStockQuotes(q) => Some(q),
            _ => None,
        })
        .collect();
    assert_eq!(quotes.len(), 5);
    assert!(quotes.iter().any(|q| q.is_empty()));
    for q in quotes {
        agency::markets::validate_quotes(&w, WORKSHOP, q).unwrap();
    }
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks
        .as_mut()
        .unwrap()
        .quotes
        .clear();
    w.marketplaces[0].markets.reverse();
    assert_eq!(agency::discovery::programs(&w, WORKSHOP).unwrap(), catalog);
}
#[test]
fn impossible_or_excessive_signed_choices_are_rejected_instead_of_truncated() {
    assert!(fixture(&[2, 0], &[4]).is_err());
    assert!(fixture(&[2], &[4, -1]).is_err());
    assert!(fixture(&(1..=16).collect::<Vec<_>>(), &[4]).is_err());
}

#[test]
fn portfolio_discovery_covers_partial_stops_and_bounds_joint_expansion() {
    use economics_compute_smoke::{
        marketplace::Market,
        minting::{COIN, FIREWOOD},
    };
    for wide in [false, true] {
        let (mut w, s) = fixture(&[2], &[4]).unwrap();
        w.marketplaces[0].markets.push(Market {
            id: FIREWOOD,
            goods: Amount::new(FIREWOOD, 1),
            payment: COIN,
            price_tick: 1,
        });
        let previous = w.agency.remove(&WORKSHOP).unwrap();
        let mut a = previous.operating_agreement().unwrap().clone();
        if wide {
            a.stock_markets.get_mut(&ORE).unwrap().limits = (1..=4).collect();
        }
        a.stock_markets.insert(
            FIREWOOD,
            Options {
                side: Side::Buy,
                limits: if wide { (1..=4).collect() } else { [2].into() },
                holdings: [4].into(),
                max_lots: 2,
            },
        );
        let before = w.clone();
        let result = organization::install_discovering_initial(
            &mut w,
            &s,
            a.clone(),
            &a.members,
            previous.config,
        );
        if wide {
            assert!(result.is_err());
            assert_eq!(w, before);
            continue;
        }
        result.unwrap();
        let portfolios = agency::markets::discovery::portfolios(&w, WORKSHOP).unwrap();
        let keys: std::collections::BTreeSet<Vec<_>> = portfolios
            .iter()
            .map(|q| q.iter().map(|q| q.market).collect())
            .collect();
        assert_eq!(
            keys,
            [vec![], vec![ORE], vec![FIREWOOD], vec![FIREWOOD, ORE]].into()
        );
    }
}
