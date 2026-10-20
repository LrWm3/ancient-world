use super::*;
use crate::{
    acceptance::{WORKSHOP, continuation::Case, financing::Stocks, work_discovery},
    minting::{COIN, ORE},
};
fn fixture() -> (World, State, Config, u32) {
    let (w, s) = work_discovery::scenario(
        Case::Baseline,
        Stocks::Neither,
        work_requests::Policy::Upstream,
    )
    .unwrap();
    let mut c = w.agency[&WORKSHOP].config.clone();
    let snapshot = work_requests::Snapshot::observe(&w, &s, WORKSHOP).unwrap();
    c.programs = discovery::programs_from_requests(&w, WORKSHOP, &snapshot).unwrap();
    let id = *c.programs.iter().find(|(_, p)| matches!(p.commands.as_slice(),
        [Command::PeerStockQuotes(q)] if q.len() == 1 && q[0].market == ORE && q[0].side == Side::Buy)).unwrap().0;
    (w, s, c, id)
}
fn alt(program: Option<u32>, loss: i128) -> Alternative {
    Alternative {
        program,
        losses: Some(vec![loss]),
        preferences: BTreeMap::new(),
        failure: None,
        projection: None,
    }
}
#[test]
fn input_bid_publication_is_opt_in_and_only_breaks_an_exact_waiting_tie() {
    let (w, s, mut c, id) = fixture();
    let wait = alt(None, 1);
    let choices = [alt(Some(id), 1)];
    assert!(
        select(&w, &s, WORKSHOP, &c, None, &wait, &choices)
            .unwrap()
            .is_none()
    );
    c.publication = Policy::InputBidsOnEqualProjection;
    assert_eq!(
        select(&w, &s, WORKSHOP, &c, None, &wait, &choices).unwrap(),
        Some(id)
    );
    for loss in [0, 2] {
        assert!(
            select(&w, &s, WORKSHOP, &c, None, &wait, &[alt(Some(id), loss)])
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn stocked_sales_precede_input_exploration_in_either_candidate_order() {
    let (w, mut s, mut c, bid) = fixture();
    s.balances.insert((WORKSHOP, crate::minting::METAL), 2);
    c.publication = Policy::InputBidsOnEqualProjection;
    let ask = *c
        .programs
        .iter()
        .find(|(_, p)| {
            matches!(p.commands.as_slice(),
        [Command::PeerStockQuotes(q)] if q.len() == 1 && q[0].side == Side::Sell)
        })
        .unwrap()
        .0;
    for ids in [[bid, ask], [ask, bid]] {
        let alternatives = ids.map(|id| alt(Some(id), 1));
        assert_eq!(
            select(&w, &s, WORKSHOP, &c, None, &alt(None, 1), &alternatives).unwrap(),
            Some(ask)
        );
    }
}
#[test]
fn inputs_need_a_real_shortage_and_available_payment_not_assumed_sales() {
    let (w, mut s, c, id) = fixture();
    let p = &c.programs[&id];
    assert!(input_bids(&w, &s, WORKSHOP, p).unwrap());
    s.balances.insert((WORKSHOP, COIN), 0);
    assert!(!input_bids(&w, &s, WORKSHOP, p).unwrap());
    s.balances.insert((WORKSHOP, COIN), 3); // The full signed four-ore target costs four coins.
    assert!(!input_bids(&w, &s, WORKSHOP, p).unwrap());
    s.balances.insert((WORKSHOP, COIN), 100);
    s.balances.insert((WORKSHOP, ORE), 100);
    assert!(!input_bids(&w, &s, WORKSHOP, p).unwrap());
}
#[test]
fn publishing_inputs_cannot_withdraw_existing_quotes_or_add_work() {
    let (mut w, s, c, id) = fixture();
    let p = &c.programs[&id];
    let mut work = p.clone();
    work.commands
        .push(Command::StartProcess(crate::minting::REFINE));
    assert!(!input_bids(&w, &s, WORKSHOP, &work).unwrap());
    let ask = crate::minting::orders::Quote {
        agent: WORKSHOP,
        market: crate::minting::METAL,
        side: Side::Sell,
        holding: 0,
        limit: 4,
        max_lots: Some(1),
    };
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes
        .push(ask);
    assert!(!input_bids(&w, &s, WORKSHOP, p).unwrap());
}

#[test]
fn accepted_payment_claims_reduce_the_publication_budget() {
    let (w, mut s, c, id) = fixture();
    let p = &c.programs[&id];
    s.month = 2;
    s.balances.insert((WORKSHOP, COIN), 4);
    assert!(input_bids(&w, &s, WORKSHOP, p).unwrap());
    // An already accepted delivery denominated in coins protects one of them.
    let t = crate::forward::direct::Terms {
        id: 9900,
        seller: WORKSHOP,
        buyer: crate::minting::ISSUER,
        month: 1,
        due: 2,
        goods: Amount::new(COIN, 1),
        prepayment: Amount::new(ORE, 1),
    };
    s.exchange.forwards.insert(t.id, t.contract());
    assert!(!input_bids(&w, &s, WORKSHOP, p).unwrap());
}

#[test]
fn existing_bids_share_the_same_payment_budget_as_new_bids() {
    let (mut w, mut s, c, id) = fixture();
    let mut p = c.programs[&id].clone();
    let market = 9900;
    let mut listing = crate::marketplace::venue(&w, w.minting.as_ref().unwrap().venue)
        .unwrap()
        .markets
        .iter()
        .find(|m| m.id == ORE)
        .unwrap()
        .clone();
    listing.id = market;
    // A second stock's retained bid competes for the same payment resource.
    let mut resource = w.resources.iter().find(|r| r.id == ORE).unwrap().clone();
    resource.id = market;
    listing.goods.resource = market;
    w.resources.push(resource);
    w.marketplaces
        .iter_mut()
        .find(|v| v.agent == w.minting.as_ref().unwrap().venue)
        .unwrap()
        .markets
        .push(listing);
    let authority = w.agency[&WORKSHOP]
        .operating_agreement()
        .unwrap()
        .stock_markets[&ORE]
        .clone();
    w.agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .organization
        .as_mut()
        .unwrap()
        .stock_markets
        .insert(market, authority);
    let Command::PeerStockQuotes(quotes) = &mut p.commands[0] else {
        panic!()
    };
    let mut retained = quotes[0].clone();
    retained.market = market;
    quotes.push(retained.clone());
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
        .push(retained);
    s.balances.insert((WORKSHOP, COIN), 4);
    // Each four-coin target fits alone; the replaced portfolio costs eight.
    assert!(!input_bids(&w, &s, WORKSHOP, &p).unwrap());
    s.balances.insert((WORKSHOP, COIN), 8);
    validate_program(&w, WORKSHOP, &p).unwrap();
    assert!(input_bids(&w, &s, WORKSHOP, &p).unwrap());
}

#[test]
fn upstream_opt_in_and_pending_forward_activation_gate_publication() {
    let (mut w, s, c, id) = fixture();
    let p = &c.programs[&id];
    w.agency.get_mut(&WORKSHOP).unwrap().config.work_requests =
        work_requests::Policy::RespondWithWorkQuotes;
    assert!(!input_bids(&w, &s, WORKSHOP, p).unwrap());
    w.agency.get_mut(&WORKSHOP).unwrap().config.work_requests = work_requests::Policy::Upstream;
    assert!(input_bids(&w, &s, WORKSHOP, p).unwrap());
    let proposal = forwards::candidates(&w, &s)
        .unwrap()
        .into_iter()
        .find(|p| p.terms.buyer == WORKSHOP)
        .unwrap();
    w.agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .startup_forwards
        .push(forwards::execution::Agreement {
            month: s.month,
            proposal,
            governors: BTreeMap::new(),
            programs: BTreeMap::new(),
            status: forwards::execution::Status::Pending,
        });
    assert!(!input_bids(&w, &s, WORKSHOP, p).unwrap());
}
