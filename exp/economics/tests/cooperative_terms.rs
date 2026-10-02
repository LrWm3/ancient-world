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
        independent: false,
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

#[test]
fn common_views_keep_conditional_exchange_terms_and_terminal_outcomes() {
    use economics_compute_smoke::agreements::{self, Identity, Status as AgreementStatus, View};
    for failed in [false, true] {
        let initial = fixture();
        let mut w = initial.world;
        let (_, mut s) = calibration::scenario(true);
        let Policy::Agreement(c) = &mut w.production_market.as_mut().unwrap().policy else {
            unreachable!()
        };
        c.deliveries[0].month = 1;
        if failed {
            s.balances.insert((WOOD_PERSON, TOKEN), 0);
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            assert!(
                !agreements::for_agent(&w, &s, WOOD_PERSON)
                    .unwrap()
                    .iter()
                    .any(|v| matches!(v, View::Exchange(_)))
            );
            sim.run_months(1).unwrap();
            let accepted = sim.state.clone();
            let mut observations = vec![];
            for agent in [CROP_PERSON, WOOD_PERSON] {
                let view = agreements::for_agent(&w, &sim.state, agent)
                    .unwrap()
                    .into_iter()
                    .find(|v| matches!(v, View::Exchange(_)))
                    .unwrap();
                assert_eq!(view.identity(), Identity::CooperativeExchange(1));
                assert_eq!(view.parties(), vec![CROP_PERSON, WOOD_PERSON]);
                assert_eq!(view.grantor(), None);
                assert_eq!(view.holder(), None);
                assert!(view.claims().unwrap().is_empty());
                let View::Exchange(v) = view else {
                    unreachable!()
                };
                assert_eq!(v.terms.deliveries[0].payment.amount.quantity, 4);
                assert_eq!(
                    v.status,
                    if failed {
                        AgreementStatus::Failed
                    } else {
                        AgreementStatus::Active
                    }
                );
                assert_eq!(v.completed.len(), usize::from(!failed));
                observations.push(*v);
            }
            assert_eq!(observations[0], observations[1]);
            assert_eq!(sim.state, accepted); // inspection cannot settle or accrue
            assert!(
                !agreements::for_agent(&w, &sim.state, 0)
                    .unwrap()
                    .iter()
                    .any(|v| matches!(v, View::Exchange(_)))
            );
            let mut resumed = Simulation::new(w.clone(), sim.state.clone(), backend).unwrap();
            sim.run_months(5).unwrap();
            resumed.run_months(5).unwrap();
            assert_eq!(sim.state, resumed.state);
            let view = agreements::for_agent(&w, &sim.state, WOOD_PERSON)
                .unwrap()
                .into_iter()
                .find_map(|v| {
                    if let View::Exchange(v) = v {
                        Some(*v)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(
                view.status,
                if failed {
                    AgreementStatus::Failed
                } else {
                    AgreementStatus::Completed
                }
            );
            assert_eq!(view.completed.len(), usize::from(!failed));
            assert_eq!(view.failure.is_some(), failed);
            // Initial failure also keeps the full accepted schedule, even though
            // no work grant or exchange leg ever became active.
            assert_eq!(view.terms.deliveries.len(), 1);
            let mut corrupt = accepted.clone();
            corrupt.town_market.history[0]
                .cooperation
                .as_mut()
                .unwrap()
                .terms = None;
            assert!(Simulation::new(w.clone(), corrupt, backend).is_err());
            (sim.state, sim.ledger, view)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
