use economics_compute_smoke::{
    acceptance::{Run, WORKSHOP, continuation as c, governed as g},
    agency::expectations::{Policy, Snapshot},
    compute::Backend,
    forecast::ForecastContext,
    minting::{COIN, ISSUER, MINT},
    model::*,
    opportunities::Action,
    simulation::Simulation,
};
const POLICY: Policy = Policy::RecentSuccessfulStartsAndBallots { memory_months: 6 };

#[test]
fn projected_demand_cannot_replace_money_permission_market_access_or_enabled_recipes() {
    let (w, s) = g::scenario(c::Case::Baseline).unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(6).unwrap();
    let snapshot = Snapshot::observe(&run.sim.world, &run.sim.state, WORKSHOP, POLICY).unwrap();
    for control in [
        "normal",
        "no wealth",
        "no permission",
        "no venue",
        "disabled",
    ] {
        let (mut w, mut s) = ForecastContext::new(&run.sim.world, &run.sim.state).into_parts();
        let kind = w.transaction_policy.as_ref().unwrap().agent_types[&ISSUER];
        match control {
            "no wealth" => {
                for ((agent, _), amount) in &mut s.balances {
                    if *agent == ISSUER {
                        *amount = 0;
                    }
                }
            }
            "no permission" => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(kind, Action::Process(MINT)));
            }
            "no venue" => {
                w.marketplaces[0].allowed_types.remove(&kind);
            }
            "disabled" => {
                w.definitions
                    .iter_mut()
                    .find(|d| d.id == MINT)
                    .unwrap()
                    .enabled = false;
            }
            _ => {}
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        let mut starts = vec![];
        while sim.state.month < 11 {
            snapshot.apply_ballots(&mut sim.world, &sim.state).unwrap();
            starts.extend(
                snapshot
                    .apply_hypotheses(&mut sim.world, &sim.state)
                    .unwrap(),
            );
            sim.step().unwrap();
        }
        let completed = sim
            .state
            .processes
            .values()
            .filter(|p| {
                p.operator == ISSUER
                    && p.definition == MINT
                    && p.start > snapshot.as_of
                    && p.status == Status::Completed
            })
            .count();
        assert_eq!(completed > 0, control == "normal", "{control}");
        if matches!(control, "no permission" | "disabled") {
            assert!(starts.is_empty());
        }
        if matches!(control, "no wealth" | "no venue") {
            assert!(!starts.is_empty(), "test must exercise failed execution");
        }
        assert!(sim.state.balance(ISSUER, COIN) >= 0);
    }
}

#[test]
fn peer_incumbent_preferences_bound_demand_instead_of_overridden_collective_goals() {
    use economics_compute_smoke::agency::{
        self,
        objectives::{Metric, Objective, Scope},
    };
    let (w, s) = g::scenario(c::Case::Baseline).unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(6).unwrap();
    let snapshot = Snapshot::observe(&run.sim.world, &run.sim.state, WORKSHOP, POLICY).unwrap();
    let (mut w, mut s) = ForecastContext::new(&run.sim.world, &run.sim.state).into_parts();
    s.month = 8;
    let governor = agency::governor(&w, &s, ISSUER).unwrap();
    w.agency
        .get_mut(&ISSUER)
        .unwrap()
        .config
        .preferences
        .insert(
            governor,
            vec![Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: COIN,
                    target: 0,
                },
            }],
        );
    assert!(snapshot.apply_hypotheses(&mut w, &s).unwrap().is_empty());
    w.agency
        .get_mut(&ISSUER)
        .unwrap()
        .config
        .preferences
        .remove(&governor);
    assert!(!snapshot.apply_hypotheses(&mut w, &s).unwrap().is_empty());
}
