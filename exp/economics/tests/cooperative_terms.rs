use economics_compute_smoke::{
    calibration::{self, CROP_PERSON, WOOD_PERSON},
    compute::Backend,
    cooperation::{Contract, Delivery, Discovery},
    finance::Transfer,
    model::*,
    negotiation::GRAIN_MARKET,
    production_market::{Choice, Policy, Purchases, Work},
    scenario::{GRAIN, TOKEN},
    simulation::Simulation,
};

fn fixture() -> Simulation {
    let (mut w, s) = calibration::scenario(true);
    w.production_market.as_mut().unwrap().policy = Policy::Agreement(Box::new(Contract {
        start: 1,
        through: 6,
        choices: [CROP_PERSON, WOOD_PERSON]
            .into_iter()
            .map(|a| {
                (
                    a,
                    Choice {
                        work: Work::Wait,
                        buy: Purchases::None,
                    },
                )
            })
            .collect(),
        deliveries: vec![Delivery {
            month: 2,
            market: GRAIN_MARKET,
            goods: Transfer {
                from: CROP_PERSON,
                to: WOOD_PERSON,
                amount: Amount::new(GRAIN, 2),
            },
            payment: Transfer {
                from: WOOD_PERSON,
                to: CROP_PERSON,
                amount: Amount::new(TOKEN, 4),
            },
        }],
    }));
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim
}

#[test]
fn accepted_deliveries_cannot_be_rewritten_by_editing_the_planner_configuration() {
    let sim = fixture();
    for edit in 0..3 {
        let mut world = sim.world.clone();
        let Policy::Agreement(c) = &mut world.production_market.as_mut().unwrap().policy else {
            unreachable!()
        };
        match edit {
            0 => c.deliveries[0].payment.amount.quantity = 3,
            1 => c.deliveries[0].month = 3,
            _ => c.choices.get_mut(&CROP_PERSON).unwrap().work = Work::Ordinary,
        }
        assert_eq!(
            Simulation::new(world, sim.state.clone(), Backend::Reference).unwrap_err(),
            "cannot rewrite an active cooperative agreement"
        );
    }
}

#[test]
fn changing_discovery_preserves_accepted_terms_through_cpu_continuation() {
    let mut reference = fixture();
    let mut world = reference.world.clone();
    world.production_market.as_mut().unwrap().policy = Policy::Cooperate(Discovery::Posted);
    let mut cpu = Simulation::new(world, reference.state.clone(), Backend::CubeCpu).unwrap();
    reference.run_months(1).unwrap();
    cpu.run_months(1).unwrap();
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.ledger.last(), cpu.ledger.last());
    assert_eq!(
        cpu.state
            .town_market
            .history
            .last()
            .unwrap()
            .cooperation
            .as_ref()
            .unwrap()
            .completed[0]
            .payment
            .amount
            .quantity,
        4
    );
}
