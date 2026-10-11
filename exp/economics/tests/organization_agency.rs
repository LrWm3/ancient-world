use economics_compute_smoke::{
    acceptance::{self, SMITH, WORKSHOP, WORKSHOP_TYPE},
    activities::{Target, WorkOrder},
    agency::{
        self, Command, Program,
        objectives::{Metric, Objective, Scope},
        organization::{self, Agreement},
    },
    compute::Backend,
    governance::Leadership,
    maintenance::TerminalTransition,
    minting::{METAL, NUTRITION, REFINE, SUPPLIER},
    model::*,
    opportunities::{Action, PERSON_TYPE},
    simulation::Simulation,
};
use std::collections::BTreeSet;

fn fixture() -> (World, State, Agreement, agency::Config) {
    let (mut w, s) = acceptance::scenario().unwrap();
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::GovernOrganization(WORKSHOP_TYPE)));
    w.activities.orders.retain(|o| o.agent != WORKSHOP);
    let a = Agreement {
        organization: WORKSHOP,
        members: [SMITH, SUPPLIER].into(),
        founder: SMITH,
        leadership: Leadership::Rotating,
        term_months: 2,
        processes: [REFINE].into(),
        stock_markets: Default::default(),
    };
    let c = agency::scenario::config(
        vec![Objective {
            scope: Scope::Organization,
            metric: Metric::Reserve {
                resource: METAL,
                target: 2,
            },
        }],
        [(
            1,
            Program {
                name: "supply metal".into(),
                commands: vec![Command::WorkTargets(vec![WorkOrder {
                    agent: WORKSHOP,
                    definition: REFINE,
                    priority: 0,
                    target: Target::Stock(Amount::new(METAL, 2)),
                }])],
            },
        )]
        .into(),
    );
    (w, s, a, c)
}

#[test]
fn initial_operating_terms_require_exact_lawful_consent_and_do_not_transfer_property() {
    let (mut w, s, a, c) = fixture();
    for signatures in [
        BTreeSet::new(),
        [SMITH].into(),
        [SMITH, SUPPLIER, 89].into(),
    ] {
        let old = w.clone();
        assert!(
            organization::install_initial(&mut w, &s, a.clone(), &signatures, c.clone()).is_err()
        );
        assert_eq!(w, old);
    }
    let mut denied = w.clone();
    denied
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(PERSON_TYPE, Action::GovernOrganization(WORKSHOP_TYPE)));
    let old = denied.clone();
    assert!(
        organization::install_initial(&mut denied, &s, a.clone(), &a.members, c.clone()).is_err()
    );
    assert_eq!(denied, old);
    let old = w.clone();
    let before = s.clone();
    organization::install_initial(&mut w, &s, a.clone(), &a.members, c.clone()).unwrap();
    assert_eq!(s, before);
    assert_eq!(w.assets, old.assets);
    assert_eq!(w.participants, old.participants);
    assert_eq!(w.agents, old.agents);
    assert_eq!(w.agency[&WORKSHOP].operating_agreement(), Some(&a));
    assert_eq!(agency::objectives::members(&w, &s, WORKSHOP), a.members);
    let old = w.clone();
    assert!(organization::install_initial(&mut w, &s, a.clone(), &a.members, c).is_err());
    assert_eq!(w, old);
}

#[test]
fn constitution_and_legal_process_permission_remain_separate() {
    let (mut w, s, mut a, c) = fixture();
    a.processes.clear();
    let old = w.clone();
    assert!(organization::install_initial(&mut w, &s, a.clone(), &a.members, c.clone()).is_err());
    assert_eq!(w, old);
    a.processes.insert(REFINE);
    a.leadership = Leadership::Elected;
    assert!(organization::install_initial(&mut w, &s, a.clone(), &a.members, c.clone()).is_err());
    assert_eq!(w, old);
    a.leadership = Leadership::FixedFounder;
    organization::install_initial(&mut w, &s, a.clone(), &a.members, c).unwrap();
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(WORKSHOP_TYPE, Action::Process(REFINE)));
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(3).unwrap();
    assert!(
        sim.world.agency[&WORKSHOP]
            .history
            .iter()
            .all(|d| d.chosen.is_none())
    );
    assert!(sim.state.processes.values().all(|p| p.operator != WORKSHOP));
}

#[test]
fn elected_adapters_are_unchanged_and_operating_authority_rotates_without_rewriting_history() {
    let (mut w, mut s, a, c) = fixture();
    organization::install_initial(&mut w, &s, a.clone(), &a.members, c).unwrap();
    for (month, expected) in [
        (1, SMITH),
        (2, SMITH),
        (3, SUPPLIER),
        (4, SUPPLIER),
        (5, SMITH),
    ] {
        s.month = month;
        assert_eq!(agency::governor(&w, &s, WORKSHOP), Some(expected));
    }
    s.terminal.insert(
        SMITH,
        TerminalTransition {
            month: 4,
            subject: SMITH,
            reason: NUTRITION,
            state: "dead".into(),
        },
    );
    assert_eq!(agency::governor(&w, &s, WORKSHOP), None);
    s.month = 7;
    assert_eq!(agency::governor(&w, &s, WORKSHOP), Some(SUPPLIER));
    let (mut w, s, a, c) = fixture();
    organization::install_initial(&mut w, &s, a.clone(), &a.members, c).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    let accepted = sim.world.agency[&WORKSHOP].history[0].clone();
    assert_eq!(accepted.authorized_by, Some(SMITH));
    assert!(accepted.chosen.is_some());
    assert!(
        sim.world
            .activities
            .orders
            .iter()
            .all(|o| o.agent != WORKSHOP)
    );
    sim.step().unwrap();
    assert!(
        sim.world
            .activities
            .orders
            .iter()
            .any(|o| o.agent == WORKSHOP)
    );
    assert_eq!(sim.world.agency[&WORKSHOP].history[0], accepted);
    sim.state.terminal.insert(
        SMITH,
        TerminalTransition {
            month: 1,
            subject: SMITH,
            reason: NUTRITION,
            state: "dead".into(),
        },
    );
    agency::validate_history(&sim.world, &sim.state).unwrap();
}

#[test]
fn stock_authority_is_signed_static_and_validated_against_market_units() {
    use economics_compute_smoke::{
        acceptance::continuation as c, agency::markets::Options, marketplace::Side, minting::ORE,
    };
    for invalid in [false, true] {
        let (mut w, s) = c::spot_scenario(c::Case::Baseline).unwrap();
        let old = w.agency.remove(&WORKSHOP).unwrap();
        let mut a = old.operating_agreement().unwrap().clone();
        a.stock_markets.insert(
            ORE,
            Options {
                side: Side::Buy,
                limits: [if invalid { 0 } else { 2 }].into(),
                holdings: [4].into(),
                max_lots: 2,
            },
        );
        let before = w.clone();
        let result = organization::install_initial(&mut w, &s, a.clone(), &a.members, old.config);
        assert_eq!(result.is_err(), invalid);
        if invalid {
            assert_eq!(w, before);
        } else {
            assert_eq!(w.agency[&WORKSHOP].operating_agreement(), Some(&a));
            assert_eq!(w.minting, before.minting, "signing does not publish quotes");
        }
    }
}

#[test]
fn stock_program_cannot_change_peers_side_units_or_unsigned_terms() {
    use economics_compute_smoke::{
        acceptance::continuation as c,
        agency::markets::{self, Options},
        marketplace::Side,
        minting::{ORE, orders::Quote},
    };
    let (mut w, s) = c::spot_scenario(c::Case::Baseline).unwrap();
    let old = w.agency.remove(&WORKSHOP).unwrap();
    let mut a = old.operating_agreement().unwrap().clone();
    a.stock_markets.insert(
        ORE,
        Options {
            side: Side::Buy,
            limits: [2].into(),
            holdings: [4].into(),
            max_lots: 2,
        },
    );
    organization::install_initial(&mut w, &s, a.clone(), &a.members, old.config).unwrap();
    let q = Quote {
        agent: WORKSHOP,
        market: ORE,
        side: Side::Buy,
        limit: 2,
        holding: 4,
        max_lots: Some(2),
    };
    markets::validate_quotes(&w, WORKSHOP, &[q.clone()]).unwrap();
    markets::validate_quotes(&w, WORKSHOP, &[]).unwrap();
    for bad in [
        Quote {
            agent: SUPPLIER,
            ..q.clone()
        },
        Quote {
            side: Side::Sell,
            ..q.clone()
        },
        Quote {
            limit: 3,
            ..q.clone()
        },
        Quote {
            holding: 5,
            ..q.clone()
        },
        Quote {
            max_lots: None,
            ..q.clone()
        },
        Quote {
            max_lots: Some(3),
            ..q.clone()
        },
    ] {
        assert!(markets::validate_quotes(&w, WORKSHOP, &[bad]).is_err());
    }
    assert!(markets::validate_quotes(&w, WORKSHOP, &[q.clone(), q]).is_err());
    assert!(markets::validate_quotes(&w, SUPPLIER, &[]).is_err());
}
