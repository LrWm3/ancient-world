use economics_compute_smoke::{
    acceptance::{self, continuation as c, publication},
    agency::{forwards::Options, organization},
    minting::ORE,
    model::*,
};

fn authorize(w: &mut World, s: &State, agent: AgentId, option: Options) -> Result<(), String> {
    let controller = w.agency.remove(&agent).unwrap();
    let mut agreement = controller.operating_agreement().unwrap().clone();
    agreement.forwards.insert(ORE, option);
    let signatures = agreement.members.clone();
    organization::install_discovering_initial(w, s, agreement, &signatures, controller.config)
}

#[test]
fn forward_authority_is_separate_and_bounded() {
    let (w, s) = publication::scenario(c::Case::Baseline).unwrap();
    assert!(
        w.agency[&acceptance::WORKSHOP]
            .operating_agreement()
            .unwrap()
            .forwards
            .is_empty()
    );
    for (price, months, valid) in [(2, 3, true), (0, 2, false), (2, 1, false), (2, 13, false)] {
        let mut candidate = w.clone();
        assert_eq!(
            authorize(
                &mut candidate,
                &s,
                acceptance::WORKSHOP,
                Options {
                    prepayment: price,
                    delivery_months: months
                }
            )
            .is_ok(),
            valid
        );
    }
}

#[test]
fn productive_zero_stock_supplier_requires_two_signed_forward_authorities() {
    use economics_compute_smoke::{
        acceptance::sales::Startup,
        agency::{forwards, publication::Policy},
    };
    let (mut w, mut s) = publication::with_startup(
        c::Case::Baseline,
        Startup::MetalInventory,
        Policy::StockedAsksOnEqualProjection,
    )
    .unwrap();
    s.balances.insert((acceptance::WORKSHOP, ORE), 0);
    let option = Options {
        prepayment: 2,
        delivery_months: 3,
    };
    authorize(&mut w, &s, acceptance::WORKSHOP, option.clone()).unwrap();
    assert!(forwards::candidates(&w, &s).unwrap().is_empty());
    authorize(&mut w, &s, acceptance::TOWN, option).unwrap();
    let before = (w.clone(), s.clone());
    let offers = forwards::candidates(&w, &s).unwrap();
    assert!(!offers.is_empty());
    assert_eq!(s.balance(acceptance::TOWN, ORE), 0);
    for p in offers {
        assert_eq!(
            (p.terms.buyer, p.terms.seller),
            (acceptance::WORKSHOP, acceptance::TOWN)
        );
        assert_eq!(
            (p.terms.month, p.terms.due, p.terms.goods.quantity),
            (2, 4, 2)
        );
    }
    assert_eq!(before, (w, s));
}

#[test]
fn candidates_respect_matching_terms_funding_and_finite_startup_use() {
    use economics_compute_smoke::agency::forwards;
    let (mut w, mut s) = publication::scenario(c::Case::Baseline).unwrap();
    s.balances.insert((acceptance::WORKSHOP, ORE), 0);
    for agent in [acceptance::WORKSHOP, acceptance::TOWN] {
        authorize(
            &mut w,
            &s,
            agent,
            Options {
                prepayment: 2,
                delivery_months: 3,
            },
        )
        .unwrap();
    }
    let offers = forwards::candidates(&w, &s).unwrap();
    assert!(!offers.is_empty());
    let mut poor = s.clone();
    poor.balances.insert(
        (acceptance::WORKSHOP, economics_compute_smoke::minting::COIN),
        1,
    );
    assert!(forwards::candidates(&w, &poor).unwrap().is_empty());
    let mut stocked = s.clone();
    stocked.balances.insert((acceptance::WORKSHOP, ORE), 4);
    assert!(forwards::candidates(&w, &stocked).unwrap().is_empty());
    w.prepaid_deliveries.push(offers[0].terms.clone());
    assert!(forwards::candidates(&w, &s).unwrap().is_empty());
}
