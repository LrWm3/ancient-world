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
