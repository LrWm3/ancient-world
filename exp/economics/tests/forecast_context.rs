use economics_compute_smoke::{
    compute::Backend,
    credit,
    finance::Transfer,
    forecast::ForecastContext,
    model::*,
    resale,
    scenario::{GROW, PERSON, STATE_AGENT, TOKEN},
    search::SearchContext,
    simulation::Simulation,
    work_choice,
};

#[test]
fn observations_and_known_commitments_survive_but_future_fixture_events_do_not() {
    let (w, s) = credit::scenario("repaid").unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.world.capacity_overrides.insert((3, PERSON), 0);
    sim.world.scheduled_starts = vec![
        ScheduledStart {
            month: 2,
            agent: PERSON,
            definition: GROW,
        },
        ScheduledStart {
            month: 3,
            agent: PERSON,
            definition: GROW,
        },
    ];
    let before = sim.clone();
    let context = ForecastContext::new(&sim.world, &sim.state);
    assert_eq!(context.boundary(), (2, Phase::Open));
    assert_eq!(context.state(), &sim.state);
    assert!(context.world().capacity_overrides.is_empty());
    assert_eq!(context.world().scheduled_starts.len(), 1);
    let c = context.world().credit.as_ref().unwrap();
    assert_eq!(c.transfers.len(), 1);
    assert_eq!(c.transfers[0].month, 2);
    assert_eq!(
        c.application,
        sim.world.credit.as_ref().unwrap().application
    );
    assert_eq!(context.state().credit.loans[&1].due(3).unwrap(), 4000);
    assert_eq!(context, SearchContext::new(&sim.world, &sim.state));
    assert_eq!(
        context,
        ForecastContext::new(context.world(), context.state())
    );
    let (mut hypothetical_world, mut hypothetical_state) = context.into_parts();
    hypothetical_world.credit.as_mut().unwrap().offers.clear();
    hypothetical_state.credit.loans.clear();
    assert_eq!(sim.world, before.world);
    assert_eq!(sim.state, before.state);
    let (world, state) = economics_compute_smoke::scenario::named("annual-access").unwrap();
    let context = ForecastContext::new(&world, &state);
    assert_eq!(context.world().agreements, world.agreements);
    assert_eq!(context.world().rights, world.rights);
    assert_eq!(context.state(), &state);
}

fn future_gift(sim: &mut Simulation, recipient: u32) {
    sim.world
        .credit
        .as_mut()
        .unwrap()
        .transfers
        .push(credit::ScheduledTransfer {
            month: sim.state.month + 1,
            transfer: Transfer {
                from: STATE_AGENT,
                to: recipient,
                amount: Amount::new(TOKEN, 1000),
            },
        });
}

#[test]
fn decisions_ignore_hidden_transfers_but_respond_to_current_observations() {
    let (w, s) = work_choice::scenario("mature").unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    while sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    let expected = work_choice::evaluate(&sim).unwrap();
    future_gift(&mut sim, PERSON);
    let live = sim.clone();
    assert_eq!(expected, work_choice::evaluate(&sim).unwrap());
    assert_eq!(sim.world, live.world);
    assert_eq!(sim.state, live.state);
    let (w, s) = resale::scenario("funded").unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(3).unwrap();
    sim.step().unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Acquire);
    let expected = credit::evaluate(&sim.world, &sim.state).unwrap();
    future_gift(&mut sim, resale::BUYER);
    let observed = ForecastContext::new(&sim.world, &sim.state);
    assert!(
        observed
            .world()
            .credit
            .as_ref()
            .unwrap()
            .resale_buyer
            .is_none()
    );
    assert!(sim.world.credit.as_ref().unwrap().resale_buyer.is_some());
    assert_eq!(expected, credit::evaluate(&sim.world, &sim.state).unwrap());
    sim.state.balances.insert((resale::BUYER, TOKEN), 8000);
    assert_ne!(expected, credit::evaluate(&sim.world, &sim.state).unwrap());
}
