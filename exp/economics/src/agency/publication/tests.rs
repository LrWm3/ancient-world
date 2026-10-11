use super::*;
use crate::{
    acceptance::{WORKSHOP, continuation, sales},
    minting::METAL,
};

fn fixture() -> (World, State, Config, u32) {
    let (mut w, s) = sales::with_startup(
        continuation::Case::Baseline,
        sales::Startup::SupplyChainInventory,
    )
    .unwrap();
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes
        .retain(|q| q.agent != WORKSHOP);
    let mut c = w.agency[&WORKSHOP].config.clone();
    c.programs = discovery::programs(&w, WORKSHOP).unwrap();
    let id = *c.programs.iter().find(|(_, p)| matches!(p.commands.as_slice(), [Command::PeerStockQuotes(q)] if q.len() == 1 && q[0].market == METAL)).unwrap().0;
    (w, s, c, id)
}

fn alternative(program: Option<u32>, losses: Option<Vec<i128>>) -> Alternative {
    Alternative {
        program,
        losses,
        preferences: BTreeMap::new(),
        failure: None,
        projection: None,
    }
}

#[test]
fn publication_is_opt_in_and_requires_exact_valid_retain_loss() {
    let (w, s, mut c, id) = fixture();
    let retain = alternative(None, Some(vec![10]));
    let mut choices = vec![alternative(Some(id), Some(vec![10]))];
    assert_eq!(
        select(&w, &s, WORKSHOP, &c, None, &retain, &choices).unwrap(),
        None
    );
    c.publication = Policy::StockedAsksOnEqualProjection;
    assert_eq!(
        select(&w, &s, WORKSHOP, &c, None, &retain, &choices).unwrap(),
        Some(id)
    );
    for loss in [Some(vec![11]), Some(vec![9]), None] {
        choices[0].losses = loss;
        assert_eq!(
            select(&w, &s, WORKSHOP, &c, None, &retain, &choices).unwrap(),
            None
        );
    }
    let invalid = alternative(None, None);
    assert_eq!(
        select(&w, &s, WORKSHOP, &c, None, &invalid, &choices).unwrap(),
        None
    );
}

#[test]
fn publication_preserves_existing_quotes_and_adds_neither_buys_nor_work() {
    use crate::minting::{ORE, orders::Quote};
    let (mut w, s, c, id) = fixture();
    let ask = c.programs[&id].clone();
    let buy = Quote {
        agent: WORKSHOP,
        market: ORE,
        side: Side::Buy,
        limit: 2,
        holding: 4,
        max_lots: Some(2),
    };
    let mut combined = ask.clone();
    if let Command::PeerStockQuotes(q) = &mut combined.commands[0] {
        q.push(buy.clone());
    }
    assert!(!eligible(&w, &s, WORKSHOP, &combined, &c.objectives).unwrap());
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
        .push(buy);
    assert!(!eligible(&w, &s, WORKSHOP, &ask, &c.objectives).unwrap());
    assert!(eligible(&w, &s, WORKSHOP, &combined, &c.objectives).unwrap());
    let mut changed = combined.clone();
    if let Command::PeerStockQuotes(q) = &mut changed.commands[0] {
        q[1].holding = 0;
    }
    assert!(!eligible(&w, &s, WORKSHOP, &changed, &c.objectives).unwrap());
    combined
        .commands
        .push(Command::StartProcess(crate::minting::REFINE));
    assert!(!eligible(&w, &s, WORKSHOP, &combined, &c.objectives).unwrap());
}

#[test]
fn stock_floors_claims_law_and_actual_payment_goals_bound_publication() {
    use crate::{
        acceptance::WORKSHOP_TYPE,
        minting::{COIN, ISSUER},
        opportunities::Action,
    };
    let (w, s, c, id) = fixture();
    let ask = &c.programs[&id];
    let mut short = s.clone();
    short.balances.insert((WORKSHOP, METAL), 1);
    assert!(!eligible(&w, &short, WORKSHOP, ask, &c.objectives).unwrap());
    let mut satisfied = s.clone();
    satisfied.balances.insert((WORKSHOP, COIN), 1000);
    assert!(!eligible(&w, &satisfied, WORKSHOP, ask, &c.objectives).unwrap());
    let mut illegal = w.clone();
    illegal
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(WORKSHOP_TYPE, Action::StockTrade));
    assert!(!eligible(&illegal, &s, WORKSHOP, ask, &c.objectives).unwrap());
    let mut claimed = s.clone();
    claimed.month = 2;
    let t = crate::forward::direct::Terms {
        id: 9900,
        seller: WORKSHOP,
        buyer: ISSUER,
        month: 1,
        due: 2,
        goods: Amount::new(METAL, 4),
        prepayment: Amount::new(COIN, 1),
    };
    claimed.exchange.forwards.insert(t.id, t.contract());
    assert!(!eligible(&w, &claimed, WORKSHOP, ask, &c.objectives).unwrap());
}

#[test]
fn governing_preferences_control_both_tie_comparison_and_payment_interest() {
    let (w, s, mut c, id) = fixture();
    c.publication = Policy::StockedAsksOnEqualProjection;
    let person = crate::acceptance::SMITH;
    c.preferences.insert(person, c.objectives.clone());
    let mut retain = alternative(None, Some(vec![100]));
    retain.preferences.insert(person, vec![8]);
    let mut a = alternative(Some(id), Some(vec![100]));
    a.preferences.insert(person, vec![9]);
    assert_eq!(
        select(
            &w,
            &s,
            WORKSHOP,
            &c,
            Some(person),
            &retain,
            std::slice::from_ref(&a)
        )
        .unwrap(),
        None
    );
    a.preferences.insert(person, vec![8]);
    assert_eq!(
        select(
            &w,
            &s,
            WORKSHOP,
            &c,
            Some(person),
            &retain,
            std::slice::from_ref(&a)
        )
        .unwrap(),
        Some(id)
    );
    c.preferences.get_mut(&person).unwrap()[0].metric = objectives::Metric::Reserve {
        resource: METAL,
        target: 10,
    };
    assert_eq!(
        select(&w, &s, WORKSHOP, &c, Some(person), &retain, &[a]).unwrap(),
        None
    );
}

#[test]
fn signed_floor_and_vacant_office_can_refuse_a_stocked_offer() {
    let (mut w, s, c, id) = fixture();
    let mut ask = c.programs[&id].clone();
    w.agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .organization
        .as_mut()
        .unwrap()
        .stock_markets
        .get_mut(&METAL)
        .unwrap()
        .holdings
        .insert(3);
    if let Command::PeerStockQuotes(q) = &mut ask.commands[0] {
        q[0].holding = 3;
    }
    assert!(!eligible(&w, &s, WORKSHOP, &ask, &c.objectives).unwrap());
    let mut dead = s.clone();
    dead.terminal.insert(
        crate::acceptance::SMITH,
        crate::maintenance::TerminalTransition {
            month: 1,
            subject: crate::acceptance::SMITH,
            reason: crate::minting::NUTRITION,
            state: "dead".into(),
        },
    );
    dead.month = 2;
    assert!(!eligible(&w, &dead, WORKSHOP, &c.programs[&id], &c.objectives).unwrap());
}
