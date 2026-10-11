use economics_compute_smoke::{
    acceptance::{TOWN, WORKSHOP, continuation as c},
    agency::{markets::Options, organization},
    marketplace::{Market, Side},
    minting::{COIN, ORE, SUPPLIER, orders::Quote},
    model::*,
};
use std::collections::BTreeMap;
fn options(side: Side) -> Options {
    Options {
        side,
        limits: [2].into(),
        holdings: [0].into(),
        max_lots: 2,
    }
}
fn install(
    w: &mut World,
    s: &State,
    agent: AgentId,
    markets: BTreeMap<u32, Options>,
) -> Result<(), String> {
    let previous = w.agency[&agent].clone();
    let mut a = previous.operating_agreement().unwrap().clone();
    a.stock_markets = markets;
    // Reinstall only before the first Open, retaining the original on failure.
    let mut candidate = w.clone();
    candidate.agency.remove(&agent);
    organization::install_initial(&mut candidate, s, a.clone(), &a.members, previous.config)?;
    *w = candidate;
    Ok(())
}

#[test]
fn separate_empty_books_cannot_authorize_conflicting_listings_for_the_same_good() {
    let (mut w, s) = c::spot_scenario(c::Case::Baseline).unwrap();
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
    let mut alias = w.marketplaces[0]
        .markets
        .iter()
        .find(|m| m.id == ORE)
        .unwrap()
        .clone();
    alias.id = 900;
    w.marketplaces[0].markets.push(alias);
    install(&mut w, &s, WORKSHOP, [(ORE, options(Side::Buy))].into()).unwrap();
    let before = w.clone();
    assert!(install(&mut w, &s, TOWN, [(900, options(Side::Sell))].into()).is_err());
    assert_eq!(w, before);
    install(&mut w, &s, TOWN, [(ORE, options(Side::Sell))].into()).unwrap();
}

#[test]
fn combined_signed_slots_and_current_quotes_must_fit_the_book_limit() {
    let (mut w, s) = c::spot_scenario(c::Case::Baseline).unwrap();
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks
        .as_mut()
        .unwrap()
        .quotes = vec![Quote {
        agent: SUPPLIER,
        market: ORE,
        side: Side::Sell,
        limit: 2,
        holding: 0,
        max_lots: Some(1),
    }];
    let mut markets = vec![ORE];
    for id in 900..915 {
        let mut resource = w.resources.iter().find(|r| r.id == ORE).unwrap().clone();
        resource.id = id;
        resource.name = format!("test resource {id}");
        w.resources.push(resource);
        w.marketplaces[0].markets.push(Market {
            id,
            goods: Amount::new(id, 1),
            payment: COIN,
            price_tick: 1,
        });
        markets.push(id);
    }
    install(
        &mut w,
        &s,
        WORKSHOP,
        markets.iter().map(|id| (*id, options(Side::Buy))).collect(),
    )
    .unwrap();
    let before = w.clone();
    assert!(
        install(
            &mut w,
            &s,
            TOWN,
            markets
                .iter()
                .map(|id| (*id, options(Side::Sell)))
                .collect()
        )
        .is_err()
    );
    assert_eq!(w, before);
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
    install(
        &mut w,
        &s,
        TOWN,
        markets
            .iter()
            .map(|id| (*id, options(Side::Sell)))
            .collect(),
    )
    .unwrap();
}
