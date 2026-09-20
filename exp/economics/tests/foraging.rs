use economics_compute_smoke::{
    commitments::{self, PaymentPolicy},
    compute::Backend,
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
use std::collections::BTreeMap;

fn new(name: &str, backend: Backend) -> Simulation {
    let (w, s) = named(name).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn completions(sim: &Simulation, definition: DefinitionId) -> usize {
    sim.ledger
        .iter()
        .flat_map(|b| &b.transactions)
        .filter_map(|t| t.process.as_ref())
        .filter(|p| p.after.definition == definition && p.after.status == Status::Completed)
        .count()
}

#[test]
fn substitutes_cover_one_need_without_double_counting_aliases_or_shared_stock() {
    let mut sim = new("forage-abundant", Backend::Reference);
    sim.state.phase = Phase::Consumption;
    sim.world.participants[0].needs[0].quantity = 2;
    sim.state.balances.insert((PERSON, GRAIN), 1);
    sim.state.balances.insert((PERSON, WILD_FOOD), 1);
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, NUTRITION), 2);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    assert_eq!(sim.state.balance(PERSON, WILD_FOOD), 0);

    let mut sim = new("forage-abundant", Backend::Reference);
    sim.state.balances.insert((PERSON, GRAIN), 0);
    sim.state.balances.insert((PERSON, WILD_FOOD), 6);
    sim.run_months(1).unwrap();
    assert_eq!(completions(&sim, FORAGE), 0);
    assert!(
        sim.ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter_map(|t| t.process.as_ref())
            .all(|p| p.after.definition != GROW)
    );
    assert_eq!(sim.reports[0].deficit(NUTRITION), 0);

    let (mut w, _) = named("forage-abundant").unwrap();
    let mut alias = w.definition(CONSUME).clone();
    alias.id = 99;
    w.definitions.push(alias);
    let mut stocks = BTreeMap::from([(GRAIN, 1i128)]);
    let (_, unmet) = economics_compute_smoke::substitution::allocate(
        &w,
        NUTRITION,
        2,
        &mut stocks,
        &BTreeMap::new(),
    );
    assert_eq!(unmet, 1);
    w.definitions
        .iter_mut()
        .find(|d| d.id == USE_FUEL)
        .unwrap()
        .stages[0]
        .entry_inputs[0]
        .resource = GRAIN;
    let (_, unmet) = economics_compute_smoke::substitution::allocate(
        &w,
        WARMTH,
        1,
        &mut stocks,
        &BTreeMap::new(),
    );
    assert_eq!(unmet, 1);
}

#[test]
fn alternative_food_preserves_payment_grain_but_does_not_force_starvation() {
    let mut sim = new("forage-abundant", Backend::Reference);
    let (annual, _) = named("annual-access").unwrap();
    sim.world.agreements = annual.agreements;
    sim.state.month = 12;
    sim.state.phase = Phase::Consumption;
    sim.state.balances.insert((PERSON, GRAIN), 1);
    sim.state.balances.insert((PERSON, WILD_FOOD), 1);
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 1);
    assert_eq!(sim.state.balance(PERSON, WILD_FOOD), 0);
    assert_eq!(sim.state.balance(PERSON, NUTRITION), 1);

    sim.state.phase = Phase::Consumption;
    sim.state.balances.insert((PERSON, NUTRITION), 0);
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    assert_eq!(sim.state.balance(PERSON, NUTRITION), 1);

    sim.state.month = 13;
    sim.state.phase = Phase::Due;
    sim.state.balances.insert((PERSON, NUTRITION), 0);
    sim.state.balances.insert((PERSON, GRAIN), 1);
    sim.state.balances.insert((PERSON, WILD_FOOD), 1);
    sim.world.payment_policy = PaymentPolicy::ProtectEssentials;
    let protected = commitments::protected_stock(&sim.world, &sim.state).unwrap();
    assert_eq!(protected.get(&(PERSON, GRAIN)), None);
    assert_eq!(protected.get(&(PERSON, WILD_FOOD)), Some(&1));
    sim.step().unwrap();
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 1);
    assert_eq!(sim.state.balance(PERSON, WILD_FOOD), 1);
}

#[test]
fn shared_pool_is_demand_capped_regenerates_once_and_rejects_forged_sources() {
    let mut sim = new("forage-abundant", Backend::CubeCpu);
    sim.world.priority = Priority::NeedFirst;
    sim.world.condition_rules.clear();
    sim.world.participants[0]
        .needs
        .retain(|n| n.resource == NUTRITION);
    sim.state.balances.insert((PERSON, GRAIN), 0);
    sim.state.balances.insert((PERSON, SEED), 0);
    sim.world.agents.push(Agent {
        id: 91,
        name: "second claimant".into(),
    });
    let mut second = sim.world.participants[0].clone();
    second.agent = 91;
    sim.world.participants.push(second);
    sim.world.pools[0].capacity = 1;
    sim.world.pools[0].monthly_regeneration = 0;
    sim.state.balances.insert((STATE_AGENT, WILD_SUPPLY), 1);
    let mut reordered = sim.clone();
    reordered.world.participants.reverse();
    reordered.world.agents.reverse();
    reordered.run_months(1).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state, reordered.state);
    assert_eq!(sim.ledger, reordered.ledger);
    assert_eq!(completions(&sim, FORAGE), 1);
    assert_eq!(sim.state.balance(STATE_AGENT, WILD_SUPPLY), 0);
    assert_eq!(
        sim.reports
            .iter()
            .map(|r| r.fulfilled(NUTRITION))
            .sum::<i32>(),
        1
    );
    assert!(
        sim.ledger
            .iter()
            .flat_map(|b| &b.receipts)
            .any(|r| r.definition == Some(FORAGE) && r.reason == Reason::MissingStock)
    );
    sim.world.pools[0].monthly_regeneration = 1;
    let before = sim.state.clone();
    sim.step().unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, WILD_SUPPLY), 1);
    let mut replay = before.clone();
    let mut bad = sim.ledger.last().unwrap().clone();
    bad.transactions
        .iter_mut()
        .flat_map(|t| &mut t.effects)
        .find(|e| e.account == (STATE_AGENT, WILD_SUPPLY))
        .unwrap()
        .delta = 2;
    assert!(
        commit(
            &sim.world,
            &mut replay,
            &bad,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, before);
    let good = sim.ledger.last().unwrap();
    commit(
        &sim.world,
        &mut replay,
        good,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    assert!(
        commit(
            &sim.world,
            &mut replay,
            good,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, sim.state);
}

#[test]
fn foresight_exposes_immediate_food_versus_losing_the_crop() {
    let mut abundant = new("forage-abundant", Backend::Reference);
    abundant.run_months(9).unwrap();
    assert_eq!(completions(&abundant, FORAGE), 0);
    let mut bridge = new("forage-bridge", Backend::Reference);
    bridge.run_months(2).unwrap();
    assert_eq!(completions(&bridge, FORAGE), 1);
    assert_eq!(completions(&bridge, GROW), 1);
    assert!(bridge.reports.iter().all(|r| r.deficit(NUTRITION) == 0));
    let mut no_bridge = new("forage-bridge", Backend::Reference);
    no_bridge
        .world
        .definitions
        .iter_mut()
        .find(|d| d.id == FORAGE)
        .unwrap()
        .enabled = false;
    no_bridge.run_months(2).unwrap();
    assert_eq!(no_bridge.reports[0].deficit(NUTRITION), 1);

    let mut short = new("forage-conflict", Backend::Reference);
    let mut long = new("forage-conflict-long", Backend::Reference);
    assert_eq!(short.state, long.state);
    short.run_months(9).unwrap();
    long.run_months(9).unwrap();
    assert_eq!(short.reports[0].deficit(NUTRITION), 0);
    assert_eq!(completions(&short, GROW), 0);
    assert!(short.reports.iter().any(|r| r.deficit(WARMTH) > 0));
    assert_eq!(long.reports[0].deficit(NUTRITION), 1);
    assert_eq!(completions(&long, GROW), 1);
    assert!(long.reports.iter().all(|r| r.deficit(WARMTH) == 0));
    let mut harvest = new("forage-harvest", Backend::Reference);
    harvest.run_months(1).unwrap();
    assert_eq!(completions(&harvest, GROW), 1);
    assert_eq!(completions(&harvest, FORAGE), 0);
    assert_eq!(harvest.reports[0].deficit(NUTRITION), 0);
}

#[test]
fn urgent_need_can_justify_sacrificing_the_crop() {
    let mut sim = new("forage-urgent", Backend::Reference);
    let mut disabled = sim.clone();
    disabled
        .world
        .definitions
        .iter_mut()
        .find(|d| d.id == FORAGE)
        .unwrap()
        .enabled = false;
    sim.run_months(1).unwrap();
    disabled.run_months(1).unwrap();
    assert_eq!(completions(&sim, FORAGE), 1);
    assert!(sim.state.terminal.is_empty());
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.definition == GROW && p.status == Status::Aborted)
    );
    assert!(disabled.state.terminal.contains_key(&PERSON));
}

#[test]
fn cpu_reference_forecasts_replay_checkpoints_and_storage_order_agree() {
    for &name in FORAGING_SCENARIOS {
        let mut cpu = new(name, Backend::CubeCpu);
        let initial = cpu.state.clone();
        cpu.run_months(9).unwrap();
        let mut reference = new(name, Backend::Reference);
        reference.world.definitions.reverse();
        reference.world.resources.reverse();
        reference.world.participants[0].needs.reverse();
        for _ in 0..9 {
            reference.run_months(1).unwrap();
        }
        assert_eq!(cpu.state, reference.state, "{name}");
        assert_eq!(cpu.reports, reference.reports, "{name}");
        assert_eq!(cpu.ledger, reference.ledger, "{name}");
        let mut replay = initial;
        let mut checkpoints = Vec::new();
        for b in &cpu.ledger {
            if b.month == cpu.ledger[0].month {
                checkpoints.push(replay.clone());
            }
            if let Some(d) = &b.decision {
                for r in d.alternatives[d.selected]
                    .outcomes
                    .iter()
                    .filter(|r| r.month == b.month)
                {
                    assert_eq!(
                        Some(r),
                        cpu.reports
                            .iter()
                            .find(|row| row.month == r.month && row.agent == r.agent)
                    );
                }
            }
            commit(
                &cpu.world,
                &mut replay,
                b,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT,
            )
            .unwrap();
        }
        assert_eq!(replay, cpu.state);
        for state in checkpoints {
            let mut resumed =
                Simulation::new(cpu.world.clone(), state, Backend::Reference).unwrap();
            while resumed.state.month < cpu.state.month {
                resumed.step().unwrap();
            }
            assert_eq!(resumed.state, cpu.state, "{name}");
        }
    }
}

#[test]
fn indivisible_food_lots_and_fulfillment_expiration_are_respected() {
    let mut sim = new("forage-abundant", Backend::Reference);
    sim.state.phase = Phase::Consumption;
    sim.world.participants[0].needs[0].quantity = 2;
    let d = sim
        .world
        .definitions
        .iter_mut()
        .find(|d| d.id == CONSUME)
        .unwrap();
    d.stages[0].entry_inputs[0].quantity = 3;
    d.outputs[0].quantity = 2;
    sim.state.balances.insert((PERSON, GRAIN), 2);
    sim.state.balances.insert((PERSON, WILD_FOOD), 1);
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, NUTRITION), 1);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 2);
    assert_eq!(sim.state.balance(PERSON, WILD_FOOD), 0);
    // Surplus provisions expire, unlike food stocks; a large meal cannot be
    // credited again as next month's nutrition in a buffer forecast.
    let mut sim = new("forage-abundant", Backend::Reference);
    sim.state.phase = Phase::Consumption;
    sim.world
        .definitions
        .iter_mut()
        .find(|d| d.id == CONSUME)
        .unwrap()
        .outputs[0]
        .quantity = 3;
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, NUTRITION), 3);
    sim.step().unwrap(); // Close
    sim.step().unwrap(); // Open
    assert_eq!(sim.state.balance(PERSON, NUTRITION), 0);
}
