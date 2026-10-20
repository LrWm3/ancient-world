use economics_compute_smoke::{
    acceptance::{TOWN, WORKSHOP, continuation::Case, financing},
    agency::work_requests::{Snapshot, Source},
    marketplace::Side,
    minting::{COIN, ISSUER, METAL, ORE},
    model::Phase,
};
fn fixture() -> (
    economics_compute_smoke::model::World,
    economics_compute_smoke::model::State,
) {
    let (mut w, mut s) = financing::scenario(Case::Baseline).unwrap();
    let options = &w.agency[&WORKSHOP]
        .operating_agreement()
        .unwrap()
        .stock_markets[&ORE];
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
        .retain(|q| q.agent != WORKSHOP);
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
        .push(economics_compute_smoke::minting::orders::Quote {
            agent: WORKSHOP,
            market: ORE,
            side: Side::Buy,
            holding: *options.holdings.first().unwrap(),
            limit: *options.limits.first().unwrap(),
            max_lots: Some(options.max_lots),
        });
    s.balances.insert((WORKSHOP, ORE), 0);
    (w, s)
}
#[test]
fn signed_unfunded_interest_is_visible_without_creating_anything() {
    let (w, mut s) = fixture();
    s.balances.insert((WORKSHOP, COIN), 0);
    let before = (w.clone(), s.clone());
    let x = Snapshot::observe(&w, &s, TOWN).unwrap();
    let r = x
        .requests
        .iter()
        .find(|r| r.requester == WORKSHOP && r.resource == ORE)
        .unwrap();
    assert!(r.lots > 0);
    assert_eq!(r.due, None);
    assert!(matches!(r.source, Source::StandingStockTarget { .. }));
    assert_eq!((w, s), before);
}
#[test]
fn refreshed_requests_follow_withdrawal_stocks_access_and_signed_bounds() {
    let (w, s) = fixture();
    let has = |w: &_, s: &_| {
        Snapshot::observe(w, s, TOWN)
            .unwrap()
            .requests
            .iter()
            .any(|r| r.requester == WORKSHOP)
    };
    assert!(has(&w, &s));
    let mut funded = s.clone();
    funded.balances.insert((WORKSHOP, ORE), 100);
    assert!(!has(&w, &funded));
    let mut withdrawn = w.clone();
    withdrawn
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks
        .as_mut()
        .unwrap()
        .quotes
        .retain(|q| q.agent != WORKSHOP);
    assert!(!has(&withdrawn, &s));
    let mut invalid = w.clone();
    invalid
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks
        .as_mut()
        .unwrap()
        .quotes
        .iter_mut()
        .find(|q| q.agent == WORKSHOP)
        .unwrap()
        .limit += 999;
    assert!(!has(&invalid, &s));
    let mut denied = w.clone();
    denied.marketplaces[0]
        .allowed_types
        .remove(&economics_compute_smoke::acceptance::WORKSHOP_TYPE);
    assert!(!has(&denied, &s));
    let mut closed = s.clone();
    closed.phase = Phase::Close;
    assert!(Snapshot::observe(&w, &closed, TOWN).is_err());
}
#[test]
fn dated_procurement_keeps_deadlines_and_independent_order() {
    let (mut w, mut s) = fixture();
    s.balances.insert((ISSUER, METAL), 0);
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.month = 3;
    p.additional_months.clear();
    let x = Snapshot::observe(&w, &s, WORKSHOP).unwrap();
    let r = x
        .requests
        .iter()
        .find(|r| r.requester == ISSUER && r.resource == METAL)
        .unwrap();
    assert_eq!(r.due, Some(3));
    w.agents.reverse();
    w.definitions.reverse();
    w.resources.reverse();
    assert_eq!(x, Snapshot::observe(&w, &s, WORKSHOP).unwrap());
    s.month = 4;
    assert!(
        Snapshot::observe(&w, &s, WORKSHOP)
            .unwrap()
            .requests
            .iter()
            .all(|r| r.requester != ISSUER)
    );
}
