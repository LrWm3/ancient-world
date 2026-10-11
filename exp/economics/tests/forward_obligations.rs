use economics_compute_smoke::{
    acceptance::{
        TOWN, WORKSHOP,
        continuation::Case,
        financing::{self, Stocks},
    },
    agency::forwards::{self, assessment, execution},
    compute::Backend,
    forward::direct::Terms,
    minting::{COIN, METAL, SUPPLIER, WHEAT},
    model::*,
    simulation::Simulation,
};

#[test]
fn prior_prepayment_and_outstanding_seller_delivery_are_not_hidden_in_forecasts() {
    for shared_buyer_cash in [true, false] {
        let (mut w, s) = financing::with_stocks(Case::Baseline, true, Stocks::MetalOnly).unwrap();
        let previous = Terms {
            id: 100,
            month: 2,
            due: 6,
            buyer: if shared_buyer_cash {
                WORKSHOP
            } else {
                SUPPLIER
            },
            seller: if shared_buyer_cash { SUPPLIER } else { TOWN },
            goods: Amount::new(if shared_buyer_cash { WHEAT } else { METAL }, 1),
            prepayment: Amount::new(
                COIN,
                if shared_buyer_cash {
                    s.balance(WORKSHOP, COIN) - 2
                } else {
                    1
                },
            ),
        };
        w.prepaid_deliveries.push(previous.clone());
        let p = forwards::candidates(&w, &s).unwrap().remove(0);
        let a = assessment::assess(&w, &s, &p).unwrap();
        assert!(
            !a.mutually_beneficial(),
            "shared_buyer_cash={shared_buyer_cash}: {a:?}"
        );
        assert!(a.comparisons.values().all(|c| !c.admitted));
        let before = w.clone();
        assert!(execution::publish(&mut w, &s, &p).is_err());
        assert_eq!(before, w);
        // Observe real admission with discretionary controllers frozen: prior
        // consent survives the refusal and uses the ordinary settlement path.
        for c in w.agency.values_mut() {
            c.autonomous = false;
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(2).unwrap();
        assert!(sim.state.exchange.forwards.contains_key(&previous.id));
        assert!(!sim.state.exchange.forwards.contains_key(&p.terms.id));
        if !shared_buyer_cash {
            // This second assessment starts with a genuinely admitted, unsettled claim.
            for agent in [WORKSHOP, TOWN] {
                sim.world.agency.get_mut(&agent).unwrap().autonomous = true;
            }
            assert_eq!(
                sim.state.exchange.forwards[&previous.id]
                    .claim()
                    .outstanding(),
                1
            );
            let next = forwards::candidates(&sim.world, &sim.state)
                .unwrap()
                .remove(0);
            let assessed = assessment::assess(&sim.world, &sim.state, &next).unwrap();
            assert!(assessed.comparisons.values().all(|c| !c.admitted));
            assert!(!assessed.mutually_beneficial());
        }
        assert_eq!(
            sim.state.exchange.forwards[&previous.id].goods,
            previous.goods
        );
    }
}
