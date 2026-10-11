use super::*;
use crate::{
    acceptance::{WORKSHOP, continuation as c},
    agency::organization,
    minting::{HOURS, METAL, ORE},
};

fn fixture(market: u32, side: Side) -> Result<World, String> {
    let (mut w, s) = c::spot_scenario(c::Case::Baseline)?;
    let previous = w.agency.remove(&WORKSHOP).unwrap();
    let mut a = previous.operating_agreement().unwrap().clone();
    a.stock_markets.insert(
        market,
        Options {
            side,
            limits: [4].into(),
            holdings: [0].into(),
            max_lots: 1,
        },
    );
    organization::install_initial(&mut w, &s, a.clone(), &a.members, previous.config)?;
    Ok(w)
}
#[test]
fn issuer_stock_sale_options_route_and_stop_without_touching_unmanaged_quotes() {
    let mut w = fixture(METAL, Side::Sell).unwrap();
    let before = w.minting.as_ref().unwrap().order_policy.clone().unwrap();
    let q = Quote {
        agent: WORKSHOP,
        market: METAL,
        side: Side::Sell,
        limit: 4,
        holding: 0,
        max_lots: Some(1),
    };
    apply(&mut w, WORKSHOP, std::slice::from_ref(&q)).unwrap();
    let p = w.minting.as_ref().unwrap().order_policy.as_ref().unwrap();
    assert!(p.quotes.contains(&q));
    assert!(
        p.peer_stocks
            .as_ref()
            .unwrap()
            .quotes
            .iter()
            .all(|q| q.agent != WORKSHOP)
    );
    apply(&mut w, WORKSHOP, &[]).unwrap();
    let p = w.minting.as_ref().unwrap().order_policy.as_ref().unwrap();
    assert!(
        !p.quotes
            .iter()
            .any(|q| q.agent == WORKSHOP && q.market == METAL)
    );
    let mut expected: Vec<_> = before
        .quotes
        .into_iter()
        .filter(|q| q.agent != WORKSHOP || q.market != METAL)
        .collect();
    expected.sort_by_key(|q| (q.market, q.agent));
    assert_eq!(p.quotes, expected);
    assert_eq!(p.input_limits, before.input_limits);
    assert_eq!(p.additional_months, before.additional_months);
}
#[test]
fn issuer_input_authority_rejects_buy_capacity_and_unsigned_changes_atomically() {
    assert!(fixture(METAL, Side::Buy).is_err());
    assert!(fixture(HOURS, Side::Sell).is_err());
    let mut w = fixture(METAL, Side::Sell).unwrap();
    for (market, limit) in [(METAL, 5), (ORE, 4)] {
        let before = w.clone();
        let q = Quote {
            agent: WORKSHOP,
            market,
            side: Side::Sell,
            limit,
            holding: 0,
            max_lots: Some(1),
        };
        assert!(apply(&mut w, WORKSHOP, &[q]).is_err());
        assert_eq!(w, before);
    }
}
