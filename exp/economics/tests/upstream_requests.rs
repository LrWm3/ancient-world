use economics_compute_smoke::{
    acceptance::{TOWN, WORKSHOP, continuation::Case, financing::Stocks, work_discovery},
    agency::{
        self,
        objectives::{Metric, Objective, Scope},
        work_requests::{Policy, Snapshot, Source},
    },
    marketplace::Side,
    minting::{COIN, ORE, REFINE},
    model::*,
};
fn fixture() -> (World, State) {
    work_discovery::scenario(Case::Baseline, Stocks::Neither, Policy::Upstream).unwrap()
}
fn requests(w: &World, s: &State) -> Vec<agency::work_requests::Request> {
    let snapshot = Snapshot::observe(w, s, TOWN).unwrap();
    snapshot.validate(w, TOWN, s.month).unwrap();
    snapshot
        .requests
        .into_iter()
        .filter(|r| matches!(r.source, Source::ProspectiveInput(_)))
        .collect()
}
#[test]
fn explicit_production_goal_exposes_a_conditional_input_window_without_mutation() {
    let (w, mut s) = fixture();
    s.balances.insert((WORKSHOP, COIN), 0);
    let before = (w.clone(), s.clone());
    let rows = requests(&w, &s);
    assert_eq!(rows.len(), 1);
    let r = &rows[0];
    assert_eq!((r.requester, r.resource, r.lots), (WORKSHOP, ORE, 1));
    let Source::ProspectiveInput(d) = &r.source else {
        panic!()
    };
    assert_eq!(d.definition, REFINE);
    assert_eq!(d.output_by, s.month + w.agency[&WORKSHOP].config.horizon);
    assert_eq!(r.due, Some(d.output_by - w.definition(REFINE).duration()));
    let snapshot = Snapshot::observe(&w, &s, TOWN).unwrap();
    assert!(
        !agency::work_requests::sales(&w, &snapshot)
            .unwrap()
            .is_empty()
    );
    assert_eq!((w, s), before);
}
#[test]
fn cash_only_inactive_governor_and_inadequate_windows_do_not_invent_input_demand() {
    let (w, s) = fixture();
    let cash = vec![Objective {
        scope: Scope::Organization,
        metric: Metric::Reserve {
            resource: COIN,
            target: 10,
        },
    }];
    let mut no_goal = w.clone();
    let governor = agency::governor(&w, &s, WORKSHOP).unwrap();
    // An active personal cash preference overrides the fallback production goal.
    no_goal
        .agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .config
        .preferences
        .insert(governor, cash);
    assert!(requests(&no_goal, &s).is_empty());
    let mut too_short = w.clone();
    too_short.agency.get_mut(&WORKSHOP).unwrap().config.horizon = w.definition(REFINE).duration();
    assert!(requests(&too_short, &s).is_empty());
    let mut denied = w.clone();
    denied
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(
            economics_compute_smoke::acceptance::WORKSHOP_TYPE,
            economics_compute_smoke::opportunities::Action::Process(REFINE),
        ));
    assert!(requests(&denied, &s).is_empty());
    let mut stocked = s.clone();
    stocked.balances.insert((WORKSHOP, ORE), 100);
    assert!(requests(&w, &stocked).is_empty());
}
#[test]
fn duplicate_inputs_add_once_but_duplicate_goals_do_not_multiply_demand() {
    let (mut w, s) = fixture();
    let recipe = w.definitions.iter_mut().find(|d| d.id == REFINE).unwrap();
    recipe.stages[0].entry_inputs.push(Amount::new(ORE, 2));
    let config = &mut w.agency.get_mut(&WORKSHOP).unwrap().config;
    let goal = config
        .objectives
        .iter()
        .find(|g| matches!(g.metric, Metric::ProcessShortfall { .. }))
        .unwrap()
        .clone();
    config.objectives.push(goal);
    let rows = requests(&w, &s);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].lots, 2);
    let Source::ProspectiveInput(d) = &rows[0].source else {
        panic!()
    };
    assert_eq!(d.needed, 4);
}
#[test]
fn standing_interest_suppresses_duplicate_request_and_snapshots_keep_their_dates() {
    let (mut w, mut s) = fixture();
    let before = Snapshot::observe(&w, &s, TOWN).unwrap();
    let options = &w.agency[&WORKSHOP]
        .operating_agreement()
        .unwrap()
        .stock_markets[&ORE];
    let quote = economics_compute_smoke::minting::orders::Quote {
        agent: WORKSHOP,
        market: ORE,
        side: Side::Buy,
        holding: *options.holdings.last().unwrap(),
        limit: *options.limits.last().unwrap(),
        max_lots: Some(options.max_lots),
    };
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
        .push(quote);
    assert!(requests(&w, &s).is_empty());
    before.validate(&w, TOWN, s.month).unwrap();
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
        .clear();
    s.month += 1;
    let renewed = requests(&w, &s);
    let old = before
        .requests
        .iter()
        .find(|r| matches!(r.source, Source::ProspectiveInput(_)))
        .unwrap();
    assert_eq!(renewed[0].due, old.due.map(|d| d + 1));
    before.validate(&w, TOWN, 1).unwrap();
    let mut invalid = before.clone();
    invalid
        .requests
        .iter_mut()
        .find(|r| matches!(r.source, Source::ProspectiveInput(_)))
        .unwrap()
        .due = Some(1);
    assert!(invalid.validate(&w, TOWN, 1).is_err());
}

#[test]
fn concrete_procurement_wins_an_identical_prospective_request_key() {
    let (mut w, s) = fixture();
    let due = s.month + w.agency[&WORKSHOP].config.horizon - w.definition(REFINE).duration();
    let c = w.minting.as_mut().unwrap();
    c.issuer = WORKSHOP;
    c.definition = REFINE;
    let p = c.order_policy.as_mut().unwrap();
    p.month = due;
    p.additional_months.clear();
    p.input_limits = [(ORE, 2)].into();
    let snapshot = Snapshot::observe(&w, &s, TOWN).unwrap();
    snapshot.validate(&w, TOWN, s.month).unwrap();
    let ore: Vec<_> = snapshot
        .requests
        .iter()
        .filter(|r| r.requester == WORKSHOP && r.resource == ORE)
        .collect();
    assert_eq!(ore.len(), 1);
    assert!(matches!(ore[0].source, Source::ScheduledInput { .. }));
}
