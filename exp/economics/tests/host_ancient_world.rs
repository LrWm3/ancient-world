#![cfg(feature = "ancient-world-host")]

use ancient_world::{
    civilization::{History, Person, Site, Stocks},
    participation::{Activity, Participation, Presence, Resident},
};
use economics_compute_smoke::{
    compute::Backend,
    financial_reporting::Audit,
    host::{
        Boundary, Settlement,
        ancient_world::{Binding, LaborBindings, LaborReplay, PersonBinding, ReplayOutcome},
    },
    model::*,
    scenario::{self, GRAIN, GROW, LABOR, PERSON, PLOT, SEED, STATE_AGENT, TOKEN},
    simulation::Simulation,
};

fn history(month: u32, workshop: f32, completed: f32) -> History {
    let mut h: History = serde_json::from_value(serde_json::json!({
        "version": 2, "seed": 17, "terrain_resolution": 16, "source_epoch": 0,
        "month": month, "civilizations": [], "sites": [], "people": [], "events": [],
        "shipments": [], "candidates": [], "initial_food": 0., "initial_population": 1.
    }))
    .unwrap();
    h.sites.push(Site {
        id: 0,
        civilization: 0,
        island: 0,
        cell: 0,
        name: "Host settlement".into(),
        founded: 0,
        abandoned: false,
        lifecycle: Default::default(),
        stocks: Stocks {
            stock: [1., 0., 0., 0.],
            habitat: [0.; 4],
            ledger: [0.; 4],
            people: [0.; 4],
        },
        economy: Default::default(),
        demography: Default::default(),
    });
    h.people.push(Person {
        id: 0,
        name: "Host worker".into(),
        civilization: 0,
        born: -240,
        died: None,
        predecessor: None,
    });
    let mut participation = Participation {
        month: Some(month),
        ..Default::default()
    };
    participation.residents.insert(
        0,
        Resident {
            person: 0,
            household: None,
            presence: Presence::Resident(0),
            care: 0.,
            capacity: 0.75,
            committed: 0.,
            completed: [0.; 2],
            workshop_completed: 0.,
            merchant_completed: 0.,
            production_practice: [0.; 4],
            workshop_practice: [0.; 4],
            workshop_learning: [0.; 4],
        },
    );
    // Exercise the main engine's reservation/settlement implementation, sharing
    // one person's finite endowment with an unrelated service first.
    let culture = participation
        .reserve(month, 0, Activity::Culture, &[0], 0.25)
        .unwrap();
    participation.settle(culture, 0.25).unwrap();
    if workshop > 0. {
        let job = participation
            .reserve(month, 0, Activity::Workshop, &[0], workshop)
            .unwrap();
        participation.settle(job, completed).unwrap();
    }
    h.participation = Some(participation);
    h.participation.as_ref().unwrap().validate(&h).unwrap();
    h
}

fn economy(month: u32, backend: Backend) -> Settlement {
    let (mut w, s) = scenario::baseline();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    w.participants.push(Participant {
        agent: STATE_AGENT,
        capacity: Amount::new(LABOR, 0),
        needs: vec![],
    });
    w.definitions
        .retain(|d| d.execution == Execution::Consumption);
    w.definitions.push(ProcessDefinition {
        id: GROW,
        name: "two-tick productive lot".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "work".into(),
            months: 1,
            entry_inputs: vec![],
            monthly_services: vec![Amount::new(LABOR, 2)],
        }],
        outputs: vec![Amount::new(GRAIN, 1)],
    });
    w.scheduled_starts.push(ScheduledStart {
        month: 1,
        agent: PERSON,
        definition: GROW,
    });
    let audit = Audit::with_inventory(
        &w,
        &s,
        TOKEN,
        [(PLOT, 0)].into(),
        s.balances
            .iter()
            .map(|(key, q)| (*key, i128::from(*q)))
            .collect(),
    )
    .unwrap()
    .with_process_policy(&w, Default::default())
    .unwrap();
    Settlement::attach(
        Boundary { site: 0, month },
        Simulation::new(w, s, backend).unwrap(),
        audit,
    )
    .unwrap()
}

fn bindings() -> LaborBindings {
    LaborBindings {
        site: 0,
        capacity_resource: LABOR,
        ticks_per_worker_month: 4,
        people: vec![
            Binding {
                agent: PERSON,
                source: Some(PersonBinding {
                    person: 0,
                    activity: Activity::Workshop,
                }),
            },
            Binding {
                agent: STATE_AGENT,
                source: None,
            },
        ],
    }
}

fn unchanged(a: &Settlement, b: &Settlement) {
    assert_eq!(a.boundary(), b.boundary());
    assert_eq!(a.stage(), b.stage());
    assert_eq!(a.simulation().world, b.simulation().world);
    assert_eq!(a.simulation().state, b.simulation().state);
    assert_eq!(a.simulation().ledger, b.simulation().ledger);
    assert_eq!(a.audit(), b.audit());
}

fn replay(
    bindings: LaborBindings,
    history: &History,
    economy: &mut Settlement,
) -> Result<ReplayOutcome, String> {
    let mut session = LaborReplay::attach(bindings, history, economy.clone())?;
    let result = session.advance(history)?;
    *economy = session.economy().clone();
    Ok(result)
}

#[test]
fn real_host_reservations_bound_completed_work_without_mutating_the_source() {
    let h = history(41, 0.5, 0.);
    let before = serde_json::to_value(&h).unwrap();
    let mut reference = economy(41, Backend::Reference);
    let mut cpu = economy(41, Backend::CubeCpu);
    let r = replay(bindings(), &h, &mut reference).unwrap();
    assert_eq!(r, replay(bindings(), &h, &mut cpu).unwrap());
    unchanged(&reference, &cpu);
    let labor = r.input.labor.iter().find(|r| r.agent == PERSON).unwrap();
    assert_eq!(labor.capacity_ticks, 2);
    assert_eq!(labor.reserved_worker_months, 0.5);
    assert_eq!(labor.sources.len(), 1); // Culture is not silently reassigned.
    assert_eq!(labor.sources[0].commitment, 1);
    assert_eq!(labor.sources[0].completed_worker_months, 0.);
    assert!(r.economy.completions.iter().any(|p| p.definition == GROW));
    assert_eq!(r.economy.boundary.month, 41);
    assert_eq!(r.economy.local_month, 1);
    assert_eq!(serde_json::to_value(&h).unwrap(), before);
    let closed = reference.clone();
    assert!(replay(bindings(), &h, &mut reference).is_err());
    unchanged(&reference, &closed);
}

#[test]
fn unfilled_indivisible_work_and_inactive_demand_do_not_gain_capacity() {
    for reserved in [0., 0.25] {
        let h = history(1, reserved, 0.);
        let mut s = economy(1, Backend::Reference);
        let r = replay(bindings(), &h, &mut s).unwrap();
        assert!(!r.economy.completions.iter().any(|p| p.definition == GROW));
        let granted = r
            .input
            .opening
            .capacities
            .iter()
            .find(|c| c.agent == PERSON)
            .unwrap()
            .quantity;
        assert_eq!(granted, (reserved * 4.) as i32);
    }
}

#[test]
fn conversion_rounds_down_and_does_not_bank_fractional_or_excess_grants() {
    for (work, expected) in [
        (f32::from_bits(0.5_f32.to_bits() - 1), 1),
        (0.5, 2),
        (0.375, 1),
    ] {
        let h = history(7, work, 0.);
        let r = bindings()
            .capture(&h, &economy(7, Backend::Reference))
            .unwrap();
        let receipt = r.labor.iter().find(|r| r.agent == PERSON).unwrap();
        assert_eq!(receipt.capacity_ticks, expected);
        assert_eq!(
            f64::from(receipt.capacity_ticks) / 4. + receipt.unused_worker_months,
            receipt.reserved_worker_months
        );
    }
    let mut high_scale = bindings();
    high_scale.ticks_per_worker_month = 1 << 31;
    let r = high_scale
        .capture(&history(7, 0.5, 0.), &economy(7, Backend::Reference))
        .unwrap();
    assert_eq!(
        r.opening
            .capacities
            .iter()
            .find(|c| c.agent == PERSON)
            .unwrap()
            .quantity,
        2
    );
    for scale in [0, 3, 1000] {
        high_scale.ticks_per_worker_month = scale;
        assert!(
            high_scale
                .capture(&history(7, 0.5, 0.), &economy(7, Backend::Reference))
                .is_err()
        );
    }
}

#[test]
fn duplicate_or_incomplete_identity_bindings_are_rejected_atomically() {
    let h = history(1, 0.5, 0.5);
    let base = bindings();
    let mut cases = Vec::new();
    let mut b = base.clone();
    b.people.push(b.people[0].clone());
    cases.push(b);
    let mut b = base.clone();
    b.people.pop();
    cases.push(b);
    let mut b = base.clone();
    b.people[1].source = b.people[0].source.clone();
    cases.push(b);
    let mut b = base.clone();
    b.people[0].source = None;
    cases.push(b);
    let mut b = base.clone();
    b.people[0].source.as_mut().unwrap().person = 99;
    cases.push(b);
    let mut b = base.clone();
    b.people[0].agent = u32::MAX;
    cases.push(b);
    let mut b = base.clone();
    b.site = 1;
    cases.push(b);
    for b in cases {
        let mut s = economy(1, Backend::Reference);
        let before = s.clone();
        assert!(replay(b, &h, &mut s).is_err());
        unchanged(&s, &before);
    }
    let mut reversed = base.clone();
    reversed.people.reverse();
    let s = economy(1, Backend::Reference);
    assert_eq!(
        base.capture(&h, &s).unwrap(),
        reversed.capture(&h, &s).unwrap()
    );
}

#[test]
fn stale_unsettled_malformed_or_absent_host_work_is_not_a_replay_grant() {
    let base = history(1, 0.5, 0.);
    let mut cases = Vec::new();
    let mut h = base.clone();
    h.month = 2;
    cases.push(h);
    let mut h = base.clone();
    h.participation.as_mut().unwrap().month = Some(2);
    cases.push(h);
    let mut h = base.clone();
    h.participation.as_mut().unwrap().commitments[1].month = 2;
    cases.push(h);
    let mut h = base.clone();
    h.participation.as_mut().unwrap().commitments[1].settled = false;
    cases.push(h);
    let mut h = base.clone();
    h.participation.as_mut().unwrap().commitments[1].people[0].1 = 1.;
    cases.push(h);
    let mut h = base.clone();
    h.participation
        .as_mut()
        .unwrap()
        .residents
        .get_mut(&0)
        .unwrap()
        .presence = Presence::Traveling(1);
    cases.push(h);
    let mut h = base.clone();
    h.participation
        .as_mut()
        .unwrap()
        .residents
        .get_mut(&0)
        .unwrap()
        .capacity = f32::NAN;
    cases.push(h);
    let mut h = base.clone();
    h.sites[0].founded = 2;
    cases.push(h);
    let mut h = base.clone();
    h.people[0].died = Some(0);
    cases.push(h);
    let mut h = base.clone();
    h.participation = None;
    cases.push(h);
    for h in cases {
        let mut s = economy(1, Backend::Reference);
        let before = s.clone();
        assert!(replay(bindings(), &h, &mut s).is_err());
        unchanged(&s, &before);
    }
    // Same-month Respond outcomes do not rewrite the recorded reservation.
    let mut later_death = base.clone();
    later_death.people[0].died = Some(1);
    later_death.sites[0].abandoned = true;
    let s = economy(1, Backend::Reference);
    assert_eq!(
        bindings().capture(&base, &s).unwrap(),
        bindings().capture(&later_death, &s).unwrap()
    );
}

#[test]
fn ordinary_history_json_round_trip_and_consecutive_frames_preserve_the_bridge() {
    let first = history(5, 0.5, 0.);
    let mut a = LaborReplay::attach(bindings(), &first, economy(5, Backend::Reference)).unwrap();
    let mut b = a.clone();
    for month in 5..=7 {
        let h = history(month, if month == 6 { 0. } else { 0.5 }, 0.);
        let roundtrip: History = serde_json::from_slice(&serde_json::to_vec(&h).unwrap()).unwrap();
        assert_eq!(a.advance(&h).unwrap(), b.advance(&roundtrip).unwrap());
        unchanged(a.economy(), b.economy());
    }
}

#[test]
fn a_team_cannot_exceed_its_recorded_grant_after_high_resolution_conversion() {
    let mut h = history(1, 0., 0.);
    let mut part = Participation {
        month: Some(1),
        ..Default::default()
    };
    let resident = h.participation.as_ref().unwrap().residents[&0].clone();
    for id in 0..3 {
        if id > 0 {
            h.people.push(Person {
                id,
                ..h.people[0].clone()
            });
        }
        part.residents.insert(
            id,
            Resident {
                person: id,
                committed: 0.,
                completed: [0.; 2],
                ..resident.clone()
            },
        );
    }
    let job = part
        .reserve(1, 0, Activity::Workshop, &[0, 1, 2], 0.5)
        .unwrap();
    part.settle(job, 0.).unwrap();
    h.participation = Some(part);
    h.participation.as_ref().unwrap().validate(&h).unwrap();
    let mut s = economy(1, Backend::Reference).simulation().clone();
    s.world.participants.clear();
    s.world.agents.clear();
    s.world.assets.clear();
    s.world.rights.clear();
    s.world.scheduled_starts.clear();
    s.state.balances.clear();
    for id in 0..3 {
        s.world.agents.push(Agent {
            id,
            name: format!("worker {id}"),
        });
        s.world.participants.push(Participant {
            agent: id,
            capacity: Amount::new(LABOR, 1_000_000_000),
            needs: vec![],
        });
    }
    let audit = Audit::with_inventory(
        &s.world,
        &s.state,
        TOKEN,
        Default::default(),
        Default::default(),
    )
    .unwrap();
    let economy = Settlement::attach(Boundary { site: 0, month: 1 }, s, audit).unwrap();
    let binding = LaborBindings {
        site: 0,
        capacity_resource: LABOR,
        ticks_per_worker_month: 1 << 31,
        people: (0..3)
            .map(|id| Binding {
                agent: id,
                source: Some(PersonBinding {
                    person: id,
                    activity: Activity::Workshop,
                }),
            })
            .collect(),
    };
    let r = binding.capture(&h, &economy).unwrap();
    let total: i64 = r
        .opening
        .capacities
        .iter()
        .map(|c| i64::from(c.quantity))
        .sum();
    assert_eq!(total, 1 << 30); // Exactly 0.5 worker-month, not 32 excess ticks.
    for receipt in r.labor {
        assert!(receipt.capacity_ticks >= 0);
        assert!(receipt.unused_worker_months >= 0.);
    }
    // Unmapped people still consume their shares of the common source budget.
    let mut one = economy.simulation().clone();
    one.world.participants.retain(|p| p.agent == 2);
    let one = Settlement::attach(economy.boundary(), one, economy.audit().clone()).unwrap();
    let mut subset = binding;
    subset.people.retain(|b| b.agent == 2);
    assert_eq!(
        subset.capture(&h, &one).unwrap().opening.capacities[0].quantity,
        357_913_920
    );
}

#[test]
fn source_identity_is_pinned_and_non_working_ages_cannot_supply_positive_grants() {
    let first = history(1, 0.5, 0.);
    let mut session =
        LaborReplay::attach(bindings(), &first, economy(1, Backend::Reference)).unwrap();
    session.advance(&first).unwrap();
    let before = session.clone();
    let mut changed = history(2, 0.5, 0.);
    changed.seed += 1;
    assert!(session.advance(&changed).is_err());
    unchanged(session.economy(), before.economy());
    changed.seed = first.seed;
    changed.people[0].born -= 1;
    assert!(session.advance(&changed).is_err());
    unchanged(session.economy(), before.economy());
    for born in [-100, -800] {
        let mut h = first.clone();
        h.people[0].born = born;
        assert!(
            bindings()
                .capture(&h, &economy(1, Backend::Reference))
                .is_err()
        );
        let mut inactive = history(1, 0., 0.);
        inactive.people[0].born = born;
        assert!(
            bindings()
                .capture(&inactive, &economy(1, Backend::Reference))
                .is_ok()
        );
    }
    let mut wrong_resource = bindings();
    wrong_resource.capacity_resource = GRAIN;
    assert!(
        wrong_resource
            .capture(&first, &economy(1, Backend::Reference))
            .is_err()
    );
}

const WALLET_AGENT: AgentId = 700;

#[path = "host_ancient_world/claims.rs"]
mod claim_tests;

fn stock_history(month: u32, food: f32) -> History {
    use ancient_world::{
        economy::EconomyCatalog,
        household_economy::{HouseholdAccount, HouseholdEconomy},
    };
    let mut h = history(month, 0.5, 0.5);
    h.economy_catalog = Some(EconomyCatalog::bundled().unwrap());
    h.sites[0].stocks.stock[1] = food;
    h.sites[0].economy.goods[0] = 3.375;
    h.sites[0].economy.finance[0] = 9.375;
    let mut wallets = HouseholdEconomy::new(month);
    wallets.accounts.push(HouseholdAccount {
        cash: 2.375,
        wages: 2.375,
        ..Default::default()
    });
    h.society = Some(
        serde_json::from_value(serde_json::json!({
            "version": 1, "started": 0,
            "households": [{"id":0,"site":0,"name":"family","share":1.,"head":0,
                "founded":0,"parent":null,"generation":0}],
            "councils": [{"civilization":0,"treasury":500.,"tax_rate":0.,"relief_paid":0.}],
            "routes":[], "routed_sites":0,"raids":[],"next_raid":0,
            "household_economy":wallets
        }))
        .unwrap(),
    );
    h
}

fn stock_bindings() -> economics_compute_smoke::host::ancient_world::stocks::StockBindings {
    use economics_compute_smoke::host::ancient_world::stocks::*;
    StockBindings {
        site: 0,
        town_agent: STATE_AGENT,
        currency: TOKEN,
        ticks_per_money_unit: 4,
        goods: vec![
            GoodBinding {
                good: "food".into(),
                resource: GRAIN,
                ticks_per_kg: 1,
                carrying_cost_per_tick: 3,
            },
            GoodBinding {
                good: "wood".into(),
                resource: SEED,
                ticks_per_kg: 4,
                carrying_cost_per_tick: 2,
            },
        ],
        wallets: vec![WalletBinding {
            household: 0,
            agent: WALLET_AGENT,
        }],
    }
}

fn stock_world() -> World {
    let mut w = economy(1, Backend::Reference).simulation().world.clone();
    w.resources.iter_mut().find(|r| r.id == SEED).unwrap().name = "imported wood".into();
    w.assets.clear();
    w.rights.clear();
    w.scheduled_starts.clear();
    w.definitions
        .retain(|d| d.execution == Execution::Consumption);
    w.condition_rules.clear();
    w.participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .needs
        .clear();
    // One abstract collective food requirement; no claim of per-person calibration.
    w.participants
        .iter_mut()
        .find(|p| p.agent == STATE_AGENT)
        .unwrap()
        .needs = vec![Requirement {
        resource: scenario::NUTRITION,
        quantity: 1,
        priority: 0,
    }];
    w.agents.push(Agent {
        id: WALLET_AGENT,
        name: "host household wallet".into(),
    });
    w
}

#[test]
fn imported_portfolio_has_real_custody_explicit_costs_and_no_fixture_endowment() {
    use economics_compute_smoke::accounting::Account as BookAccount;
    let previous = stock_history(10, 2.75);
    let first = stock_history(11, 999.); // Later stocks must not overwrite the opening.
    let before = serde_json::to_value(&previous).unwrap();
    let (mut a, receipt) = stock_bindings()
        .start(
            &previous,
            &first,
            bindings(),
            stock_world(),
            Backend::Reference,
        )
        .unwrap();
    let (mut b, same) = stock_bindings()
        .start(
            &previous,
            &first,
            bindings(),
            stock_world(),
            Backend::CubeCpu,
        )
        .unwrap();
    assert_eq!(receipt, same);
    assert_eq!(receipt.completed_boundary.month, 10);
    assert_eq!(receipt.first_replay_boundary.month, 11);
    let s = &a.economy().simulation().state;
    assert_eq!(s.month, 1);
    assert_eq!(s.balance(STATE_AGENT, GRAIN), 2);
    assert_eq!(s.balance(STATE_AGENT, SEED), 13);
    assert_eq!(s.balance(STATE_AGENT, TOKEN), 37);
    assert_eq!(s.balance(WALLET_AGENT, TOKEN), 9);
    assert_eq!(s.balance(PERSON, GRAIN), 0); // Baseline's five grain never imported.
    assert_eq!(s.balance(PERSON, SEED), 0);
    assert_eq!(s.balances.len(), 4);
    assert!(a.economy().simulation().ledger.is_empty());
    let book = a.economy().audit().book().balances();
    assert_eq!(book[&(STATE_AGENT, BookAccount::Inventory(GRAIN))], 6);
    assert_eq!(book[&(STATE_AGENT, BookAccount::Inventory(SEED))], 26);
    assert_eq!(book[&(STATE_AGENT, BookAccount::Cash)], 37);
    assert_eq!(book[&(STATE_AGENT, BookAccount::OpeningEquity)], -69);
    assert_eq!(book[&(WALLET_AGENT, BookAccount::Cash)], 9);
    for r in &receipt.balances {
        assert_eq!(
            f64::from(r.quantity) / f64::from(r.ticks_per_source_unit) + r.remainder,
            r.source_quantity
        );
        assert!(r.remainder >= 0.);
    }
    let first_outcome = a.advance(&first).unwrap();
    assert_eq!(first_outcome, b.advance(&first).unwrap());
    assert_eq!(first_outcome.economy.needs[0].fulfilled, 1);
    let mut resumed = a.clone();
    for month in 12..=13 {
        let next = stock_history(month, 999.);
        let decoded: History = serde_json::from_slice(&serde_json::to_vec(&next).unwrap()).unwrap();
        assert_eq!(
            a.advance(&next).unwrap(),
            resumed.advance(&decoded).unwrap()
        );
        unchanged(a.economy(), resumed.economy());
    }
    // Two imported food ticks pay for exactly two months. No monthly re-import.
    assert_eq!(
        a.economy().simulation().state.balance(STATE_AGENT, GRAIN),
        0
    );
    assert_eq!(
        a.economy()
            .simulation()
            .reports
            .iter()
            .rev()
            .find(|r| r.agent == STATE_AGENT)
            .unwrap()
            .needs[&scenario::NUTRITION]
            .deficit,
        1
    );
    assert_eq!(serde_json::to_value(&previous).unwrap(), before);
}

#[test]
fn empty_and_fractional_source_food_cannot_fund_consumption() {
    for food in [0., 0.75, f32::from_bits(1_f32.to_bits() - 1)] {
        let h = stock_history(1, food);
        let next = stock_history(2, 999.);
        let (mut replay, _) = stock_bindings()
            .start(&h, &next, bindings(), stock_world(), Backend::Reference)
            .unwrap();
        let result = replay.advance(&next).unwrap();
        assert_eq!(result.economy.needs[0].fulfilled, 0);
        assert_eq!(result.economy.needs[0].deficit, 1);
    }
}

#[test]
fn stock_import_rejects_aliases_missing_balances_and_merged_custodians() {
    let h = stock_history(1, 2.);
    let next = stock_history(2, 2.);
    let base = stock_bindings();
    let mut stale_world = stock_world();
    let mut controller = economics_compute_smoke::agency::Controller::new(
        economics_compute_smoke::agency::scenario::config(vec![], Default::default()),
    );
    controller.applied_through = 1;
    stale_world.agency.insert(STATE_AGENT, controller);
    assert!(
        base.start(&h, &next, bindings(), stale_world, Backend::Reference)
            .unwrap_err()
            .contains("requires fresh controllers")
    );
    let mut cases = Vec::new();
    let mut b = base.clone();
    b.goods.pop();
    cases.push(b);
    let mut b = base.clone();
    b.goods.push(b.goods[0].clone());
    cases.push(b);
    let mut b = base.clone();
    b.goods[1].resource = GRAIN;
    cases.push(b);
    let mut b = base.clone();
    b.goods[1].resource = TOKEN;
    cases.push(b);
    let mut b = base.clone();
    b.goods[1].resource = LABOR;
    cases.push(b);
    let mut b = base.clone();
    b.goods[1].good = "unknown".into();
    cases.push(b);
    let mut b = base.clone();
    b.goods[1].ticks_per_kg = 3;
    cases.push(b);
    let mut b = base.clone();
    b.ticks_per_money_unit = 0;
    cases.push(b);
    let mut b = base.clone();
    b.wallets.clear();
    cases.push(b);
    let mut b = base.clone();
    b.wallets.push(b.wallets[0].clone());
    cases.push(b);
    let mut b = base.clone();
    b.wallets[0].agent = STATE_AGENT;
    cases.push(b);
    let mut b = base.clone();
    b.wallets[0].agent = PERSON;
    cases.push(b);
    let mut b = base.clone();
    b.wallets[0].household = 99;
    cases.push(b);
    let mut b = base.clone();
    b.town_agent = PERSON;
    cases.push(b);
    let mut b = base.clone();
    b.site = 1;
    cases.push(b);
    for b in cases {
        assert!(
            b.start(&h, &next, bindings(), stock_world(), Backend::Reference)
                .is_err()
        );
    }
    let mut reversed = base.clone();
    reversed.goods.reverse();
    assert_eq!(
        base.start(&h, &next, bindings(), stock_world(), Backend::Reference)
            .unwrap()
            .1,
        reversed
            .start(&h, &next, bindings(), stock_world(), Backend::Reference)
            .unwrap()
            .1
    );
    // Stock import takes a catalog and creates a fresh state; there is no API to
    // top up a live economy or silently keep material assets from a fixture.
    assert!(
        base.start(
            &h,
            &next,
            bindings(),
            economy(2, Backend::Reference).simulation().world.clone(),
            Backend::Reference
        )
        .is_err()
    );
}

#[test]
fn malformed_or_wrong_boundary_stock_import_fails_without_changing_source() {
    let base = stock_history(1, 2.);
    let next = stock_history(2, 2.);
    // JSON's null representation cannot conceal a non-finite next-frame catalog.
    let mut prior = base.clone();
    let mut invalid_next = next.clone();
    prior.economy_catalog.as_mut().unwrap().goods[0].delay_spoilage = None;
    invalid_next.economy_catalog.as_mut().unwrap().goods[0].delay_spoilage = Some(f32::NAN);
    assert_eq!(
        serde_json::to_value(&prior.economy_catalog).unwrap(),
        serde_json::to_value(&invalid_next.economy_catalog).unwrap()
    );
    assert!(
        stock_bindings()
            .start(
                &prior,
                &invalid_next,
                bindings(),
                stock_world(),
                Backend::Reference
            )
            .is_err()
    );
    let mut unbacked = base.clone();
    unbacked
        .society
        .as_mut()
        .unwrap()
        .household_economy
        .as_mut()
        .unwrap()
        .accounts[0]
        .cash += 1.;
    let error = stock_bindings()
        .start(
            &unbacked,
            &next,
            bindings(),
            stock_world(),
            Backend::Reference,
        )
        .unwrap_err();
    assert!(error.contains("household cash ledger does not reconcile"));
    let mut cases = Vec::new();
    let mut h = base.clone();
    h.month = 2;
    cases.push(h);
    let mut h = base.clone();
    h.seed += 1;
    cases.push(h);
    let mut h = base.clone();
    h.sites[0].stocks.stock[1] = f32::NAN;
    cases.push(h);
    let mut h = base.clone();
    h.sites[0].economy.finance[0] = -1.;
    cases.push(h);
    let mut h = base.clone();
    h.sites[0].economy.goods[0] = f32::INFINITY;
    cases.push(h);
    let mut h = base.clone();
    h.sites[0].economy.goods[ancient_world::economy::FOOD] = 1.;
    cases.push(h);
    let mut h = base.clone();
    h.sites[0].stocks.stock[1] = i32::MAX as f32;
    cases.push(h);
    let mut h = base.clone();
    h.society
        .as_mut()
        .unwrap()
        .household_economy
        .as_mut()
        .unwrap()
        .accounts[0]
        .cash = f64::NAN;
    cases.push(h);
    let mut h = base.clone();
    h.society.as_mut().unwrap().households[0].id = 99;
    cases.push(h);
    let mut h = base.clone();
    h.society.as_mut().unwrap().households[0].site = 1;
    cases.push(h);
    let mut h = base.clone();
    h.society
        .as_mut()
        .unwrap()
        .household_economy
        .as_mut()
        .unwrap()
        .observed = 2;
    cases.push(h);
    let mut h = base.clone();
    h.participation.as_mut().unwrap().commitments[0].settled = false;
    cases.push(h);
    let mut h = base.clone();
    h.economy_catalog.as_mut().unwrap().goods[0].base_price += 1.;
    cases.push(h);
    for h in cases {
        let before = serde_json::to_value(&h).unwrap();
        assert!(
            stock_bindings()
                .start(&h, &next, bindings(), stock_world(), Backend::Reference)
                .is_err()
        );
        assert_eq!(serde_json::to_value(&h).unwrap(), before);
    }
}

#[test]
fn household_shares_do_not_duplicate_town_stock_or_other_wallets() {
    use economics_compute_smoke::host::ancient_world::stocks::WalletBinding;
    let mut h = stock_history(1, 2.);
    let society = h.society.as_mut().unwrap();
    society.households[0].share = 0.4;
    let mut second = society.households[0].clone();
    second.id = 1;
    second.share = 0.6;
    second.head = 1;
    society.households.push(second);
    society.household_economy.as_mut().unwrap().accounts.push(
        ancient_world::household_economy::HouseholdAccount {
            cash: 5.25,
            wages: 5.25,
            ..Default::default()
        },
    );
    h.people.push(Person {
        id: 1,
        ..h.people[0].clone()
    });
    let mut first = stock_history(2, 2.);
    first.society = h.society.clone();
    first.people = h.people.clone();
    let mut binding = stock_bindings();
    binding.wallets.push(WalletBinding {
        household: 1,
        agent: WALLET_AGENT + 1,
    });
    let mut w = stock_world();
    w.agents.push(Agent {
        id: WALLET_AGENT + 1,
        name: "second household".into(),
    });
    let (replay, receipt) = binding
        .start(&h, &first, bindings(), w.clone(), Backend::Reference)
        .unwrap();
    let s = &replay.economy().simulation().state;
    assert_eq!(s.balance(STATE_AGENT, GRAIN), 2);
    assert_eq!(s.balance(WALLET_AGENT, GRAIN), 0);
    assert_eq!(s.balance(WALLET_AGENT + 1, GRAIN), 0);
    assert_eq!(s.balance(WALLET_AGENT, TOKEN), 9);
    assert_eq!(s.balance(WALLET_AGENT + 1, TOKEN), 21);
    assert_eq!(s.balance(STATE_AGENT, TOKEN), 37);
    binding.wallets.reverse();
    assert_eq!(
        receipt,
        binding
            .start(&h, &first, bindings(), w.clone(), Backend::Reference)
            .unwrap()
            .1
    );
    binding.wallets[0].agent = WALLET_AGENT;
    assert!(
        binding
            .start(&h, &first, bindings(), w, Backend::Reference)
            .is_err()
    );
}

#[test]
fn fresh_signed_organization_is_permitted_but_prior_review_is_not() {
    use economics_compute_smoke::{
        agency::{
            self,
            objectives::{Metric, Objective, Scope},
            organization::{self, Agreement},
        },
        governance::Leadership,
        opportunities::{Action, HOUSEHOLD_TYPE, PERSON_TYPE, Policy, STATE_TYPE},
    };
    let mut w = stock_world();
    w.transaction_policy = Some(Policy {
        authority: STATE_AGENT,
        laws: vec![],
        agreement_forms: None,
        agreement_limits: Default::default(),
        membership_offers: vec![],
        membership_permissions: Default::default(),
        agent_types: [
            (STATE_AGENT, STATE_TYPE),
            (PERSON, PERSON_TYPE),
            (WALLET_AGENT, HOUSEHOLD_TYPE),
        ]
        .into(),
        permissions: [(PERSON_TYPE, Action::GovernOrganization(HOUSEHOLD_TYPE))].into(),
    });
    let agreement = Agreement {
        organization: WALLET_AGENT,
        members: [PERSON].into(),
        founder: PERSON,
        leadership: Leadership::FixedFounder,
        term_months: 1,
        processes: Default::default(),
        forwards: Default::default(),
        stock_markets: Default::default(),
    };
    let config = agency::scenario::config(
        vec![Objective {
            scope: Scope::Organization,
            metric: Metric::Reserve {
                resource: TOKEN,
                target: 1,
            },
        }],
        Default::default(),
    );
    organization::install_initial(
        &mut w,
        &State::empty(),
        agreement.clone(),
        &agreement.members,
        config,
    )
    .unwrap();
    assert!(w.agency[&WALLET_AGENT].is_unreviewed());
    assert!(w.agency[&WALLET_AGENT].operating_agreement().is_some());
    let h = stock_history(1, 2.);
    let first = stock_history(2, 2.);
    let (replay, _) = stock_bindings()
        .start(&h, &first, bindings(), w.clone(), Backend::Reference)
        .unwrap();
    assert_eq!(
        replay
            .economy()
            .simulation()
            .state
            .balance(WALLET_AGENT, TOKEN),
        9
    );
    w.agency.get_mut(&WALLET_AGENT).unwrap().applied_through = 1;
    assert!(!w.agency[&WALLET_AGENT].is_unreviewed());
    assert!(
        stock_bindings()
            .start(&h, &first, bindings(), w, Backend::Reference)
            .unwrap_err()
            .contains("requires fresh controllers")
    );
}
