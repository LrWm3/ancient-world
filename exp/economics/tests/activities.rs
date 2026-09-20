use economics_compute_smoke::{
    activities::CoinPayment,
    compute::Backend,
    crafts::*,
    equipment::DurableAsset,
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

fn fixture() -> Simulation {
    let (mut w, s) = economics_compute_smoke::crafts::scenario().unwrap();
    w.activities.orders.clear();
    w.agreements.clear();
    w.issuance.clear();
    w.activities.coin_payments.clear();
    w.condition_rules.clear();
    for p in &mut w.participants {
        for n in &mut p.needs {
            n.quantity = 0;
        }
    }
    w.priority = Priority::ContinuingFirst;
    Simulation::new(w, s, Backend::Reference).unwrap()
}
fn schedule(sim: &mut Simulation, month: u32, definitions: &[u32]) {
    sim.world.scheduled_starts = definitions
        .iter()
        .map(|&definition| ScheduledStart {
            agent: PERSON,
            month,
            definition,
        })
        .collect();
}
fn gear(sim: &mut Simulation, id: u32, kind: u32, uses: u32) {
    sim.state.equipment.insert(
        id,
        DurableAsset {
            id,
            owner: PERSON,
            kind,
            remaining_uses: uses,
            last_used_month: None,
            attached_to: sim.world.activities.kinds[&kind].attached.then_some(PLOT),
        },
    );
}
fn completed(sim: &Simulation, d: u32) -> usize {
    sim.state
        .processes
        .values()
        .filter(|p| p.definition == d && p.status == Status::Completed)
        .count()
}

#[test]
fn completed_crafting_consumes_inputs_and_new_tools_wait_for_next_boundary() {
    let mut sim = fixture();
    schedule(&mut sim, 1, &[craft(HAMMER_STONE), craft(PUNCH)]);
    sim.run_months(1).unwrap();
    assert_eq!(completed(&sim, craft(HAMMER_STONE)), 1);
    assert_eq!(completed(&sim, craft(PUNCH)), 1);
    assert_eq!(sim.state.balance(PERSON, LABOR), 0); // Both pay manual cost, no free same-batch tool.
    assert_eq!(sim.state.balance(PERSON, STONE), 7);
    assert_eq!(sim.state.balance(PERSON, COPPER), 1);
    assert_eq!(sim.state.balance(PERSON, RAW_WOOD), 58);
    schedule(&mut sim, 2, &[craft(KNIFE)]);
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.balance(PERSON, LABOR), 5);
    assert_eq!(
        sim.state
            .equipment
            .values()
            .find(|a| a.kind == HAMMER_STONE)
            .unwrap()
            .remaining_uses,
        23
    );
    assert_eq!(completed(&sim, craft(KNIFE)), 1);
    let mut none = fixture();
    none.state.balances.insert((PERSON, STONE), 0);
    schedule(&mut none, 1, &[craft(HAMMER_STONE)]);
    none.run_months(1).unwrap();
    assert!(none.state.equipment.is_empty());
}

#[test]
fn repair_is_capped_exclusive_and_works_with_labor_or_a_separate_grinding_stone() {
    for equipped in [false, true] {
        let mut sim = fixture();
        gear(&mut sim, 9000, PICK, 1);
        if equipped {
            gear(&mut sim, 9001, GRINDING_STONE, 24);
        }
        schedule(&mut sim, 1, &[repair(PICK), repair(PICK)]);
        sim.run_months(1).unwrap();
        assert_eq!(completed(&sim, repair(PICK)), 1);
        assert_eq!(sim.state.equipment[&9000].remaining_uses, 13);
        assert_eq!(
            sim.state.balance(PERSON, LABOR),
            if equipped { 5 } else { 2 }
        );
        schedule(&mut sim, 2, &[repair(PICK)]);
        sim.run_months(1).unwrap();
        assert_eq!(sim.state.equipment[&9000].remaining_uses, 24);
    }
    let mut sim = fixture();
    gear(&mut sim, 9000, GRINDING_STONE, 1);
    schedule(&mut sim, 1, &[repair(GRINDING_STONE)]);
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.balance(PERSON, LABOR), 2); // Cannot use the asset being repaired.
}

#[test]
fn homes_attach_to_plots_allow_farming_and_produce_only_current_month_shelter() {
    let mut sim = fixture();
    schedule(&mut sim, 1, &[BUILD_HOME]);
    sim.run_months(2).unwrap();
    assert_eq!(completed(&sim, BUILD_HOME), 1);
    let home = sim
        .state
        .equipment
        .values()
        .find(|a| a.kind == HOUSE)
        .unwrap()
        .id;
    assert_eq!(sim.state.equipment[&home].attached_to, Some(PLOT));
    assert_eq!(sim.state.equipment[&home].remaining_uses, 120);
    assert_eq!(sim.state.balance(PERSON, STONE), 6);
    assert_eq!(sim.state.balance(PERSON, RAW_WOOD), 56);
    let mut expired = sim.clone();
    expired
        .world
        .rights
        .iter_mut()
        .find(|r| r.holder == PERSON)
        .unwrap()
        .through = 2;
    schedule(&mut expired, 3, &[OCCUPY_HOME]);
    expired.run_months(1).unwrap();
    assert_eq!(completed(&expired, OCCUPY_HOME), 0);
    schedule(&mut sim, 3, &[GROW, OCCUPY_HOME]);
    sim.run_months(1).unwrap();
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.definition == GROW && p.status == Status::Active)
    );
    assert_eq!(sim.state.equipment[&home].remaining_uses, 119);
    assert_eq!(sim.state.balance(PERSON, SHELTER_TICKET), 1);
    sim.step().unwrap(); // Open expires the unconsumed ticket.
    assert_eq!(sim.state.balance(PERSON, SHELTER_TICKET), 0);
}

#[test]
fn home_maintenance_restores_condition_but_cannot_also_occupy_the_same_month() {
    let mut sim = fixture();
    gear(&mut sim, 9000, HOUSE, 0);
    schedule(&mut sim, 1, &[repair(HOUSE), OCCUPY_HOME]);
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.equipment[&9000].remaining_uses, 24);
    assert_eq!(completed(&sim, OCCUPY_HOME), 0);
    assert_eq!(sim.state.balance(PERSON, CLAY), 7);
    schedule(&mut sim, 2, &[OCCUPY_HOME]);
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.equipment[&9000].remaining_uses, 23);
}

#[test]
fn livestock_requires_herd_and_feed_and_has_idle_upkeep_and_comb_benefit() {
    for (has_herd, has_feed, comb) in [
        (false, true, false),
        (true, false, true),
        (true, true, false),
        (true, true, true),
    ] {
        let mut sim = fixture();
        if has_herd {
            gear(&mut sim, 9000, HERD, 24);
        }
        if comb {
            gear(&mut sim, 9001, COMB, 24);
        }
        if !has_feed {
            sim.state.balances.insert((PERSON, HAY), 0);
        }
        schedule(&mut sim, 1, &[HUSBANDRY]);
        sim.run_months(1).unwrap();
        let works = has_herd && has_feed;
        assert_eq!(sim.state.balance(PERSON, MILK), if works { 2 } else { 0 });
        assert_eq!(sim.state.balance(PERSON, WOOL), i32::from(works));
        if has_herd {
            assert_eq!(
                sim.state.equipment[&9000].remaining_uses,
                if works { 22 } else { 23 }
            );
        }
        if works {
            assert_eq!(sim.state.balance(PERSON, LABOR), if comb { 4 } else { 2 });
        }
    }
    let mut sim = fixture();
    gear(&mut sim, 9000, HERD, 2);
    schedule(&mut sim, 1, &[TEND_HERD]);
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.equipment[&9000].remaining_uses, 13);
    assert_eq!(sim.state.balance(PERSON, HAY), 7);
}

#[test]
fn extraction_is_depletable_and_refining_does_not_mint_currency() {
    for index in 0..6 {
        let mut sim = fixture();
        let id = MINE_START + index;
        let account = sim
            .world
            .pool_inputs
            .iter()
            .find(|i| i.definition == id)
            .unwrap()
            .account;
        sim.state.balances.insert(account, 1);
        let output = sim.world.definition(id).outputs[0].resource;
        let initial = sim.state.balance(PERSON, output);
        schedule(&mut sim, 1, &[id]);
        sim.run_months(1).unwrap();
        assert_eq!(sim.state.balance(account.0, account.1), 0);
        assert_eq!(sim.state.balance(PERSON, output), initial + 2);
        schedule(&mut sim, 2, &[id]);
        sim.run_months(1).unwrap();
        assert_eq!(completed(&sim, id), 1);
    }
    for index in 0..4 {
        let mut sim = fixture();
        let id = REFINE_START + index;
        let ore = sim.world.definition(id).stages[0].entry_inputs[0].resource;
        sim.state.balances.insert((PERSON, ore), 1);
        schedule(&mut sim, 1, &[id]);
        sim.run_months(1).unwrap();
        assert_eq!(completed(&sim, id), 1);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 4);
        assert_eq!(sim.state.balance(PERSON, FUEL), 39);
    }
}

#[test]
fn commodity_or_coin_taxes_respect_prices_partial_payment_and_no_double_issuance() {
    for (grain, coins, rate, capacity, paid, native, treasury) in [
        (2, 2, 1, 0, 2, 0, 2),
        (1, 1, 1, 32, 2, 1, 1),
        (0, 3, 2, 32, 1, 0, 2),
        (0, 0, 1, 32, 0, 0, 0),
        (2, 0, 1, 32, 2, 2, 1),
    ] {
        let (mut w, mut s) = named("storage-exchange").unwrap();
        w.activities.coin_payments.insert(
            1,
            CoinPayment {
                resource: TOKEN,
                coins_per_unit: rate,
            },
        );
        w.storage.capacities.insert(STATE_AGENT, capacity);
        s.month = 13;
        s.phase = Phase::Due;
        s.balances.insert((PERSON, GRAIN), grain);
        s.balances.insert((PERSON, TOKEN), coins);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.step().unwrap();
        assert_eq!(sim.state.obligations[&(1, 13)].paid, paid);
        assert_eq!(sim.state.obligations[&(1, 13)].in_kind_paid, native);
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), treasury);
        let treasury_before = sim.state.balance(STATE_AGENT, TOKEN);
        sim.state.phase = Phase::ClearArrears;
        sim.step().unwrap();
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), treasury_before);
    }
}

#[test]
fn durable_completion_replay_is_atomic_and_tampered_work_publishes_nothing() {
    let mut sim = fixture();
    schedule(&mut sim, 1, &[craft(HAMMER_STONE)]);
    sim.step().unwrap();
    let before = sim.state.clone();
    sim.step().unwrap();
    let batch = sim.ledger.last().unwrap().clone();
    let mut replay = before.clone();
    let mut bad = batch.clone();
    bad.transactions[0].effects[0].delta = 0;
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
    commit(
        &sim.world,
        &mut replay,
        &batch,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    assert_eq!(replay, sim.state);
    assert!(
        commit(
            &sim.world,
            &mut replay,
            &batch,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, sim.state);
}

#[test]
fn all_activity_families_activate_on_cpu_with_replay_reordering_and_continuation() {
    check_population("specialized-activities");
}

#[test]
fn thirty_two_people_cpu_replay_reordering_and_continuation() {
    check_population("specialized-32");
}

fn check_population(name: &str) {
    let (w, s) = named(name).unwrap();
    let count = if name == "specialized-32" { 32 } else { 4 };
    assert_eq!(w.participants.len(), count);
    assert_eq!(w.agents.len(), count + 1);
    assert_eq!(w.rights.len(), count);
    assert_eq!(w.agreements.len(), count);
    assert_eq!(w.issuance.len(), count / 4);
    for p in &w.participants {
        assert_eq!(s.balance(p.agent, SEED), 1);
        assert_eq!(s.balance(p.agent, TOKEN), 4);
        assert!(
            w.rights
                .iter()
                .any(|r| r.holder == p.agent && r.asset == PLOT + p.agent - PERSON)
        );
    }
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    cpu.run_months(ACTIVITY_MONTHS).unwrap();
    let mut reversed = w.clone();
    reversed.agents.reverse();
    reversed.participants.reverse();
    reversed.definitions.reverse();
    reversed.activities.orders.reverse();
    reversed.techniques.reverse();
    reversed.rights.reverse();
    reversed.assets.reverse();
    reversed.agreements.reverse();
    reversed.pools.reverse();
    reversed.pool_inputs.reverse();
    reversed.condition_rules.reverse();
    let mut reference = Simulation::new(reversed, s.clone(), Backend::Reference).unwrap();
    for _ in 0..ACTIVITY_MONTHS {
        reference.run_months(1).unwrap();
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    for id in (MINE_START..MINE_START + 6)
        .chain(REFINE_START..REFINE_START + 4)
        .chain([
            BUILD_HOME,
            OCCUPY_HOME,
            RAISE_HERD,
            HUSBANDRY,
            FISHING,
            TEND_HERD,
        ])
    {
        assert!(completed(&cpu, id) > 0, "inactive process {id}");
    }
    for kind in HAMMER_STONE..=GRINDING_STONE {
        assert!(completed(&cpu, craft(kind)) > 0);
    }
    let mut replay = s;
    let mut checkpoint = None;
    for b in &cpu.ledger {
        commit(&w, &mut replay, b, Backend::Reference, DEFAULT_EFFECT_LIMIT).unwrap();
        if b.month == 1 && b.phase == Phase::Productive {
            checkpoint = Some(replay.clone());
        }
    }
    assert_eq!(replay, cpu.state);
    let mut resumed = Simulation::new(w, checkpoint.unwrap(), Backend::Reference).unwrap();
    while resumed.state.month < cpu.state.month {
        resumed.step().unwrap();
    }
    assert_eq!(resumed.state, cpu.state);
    assert!(cpu.state.terminal.is_empty());
    assert!(cpu.state.obligations.values().all(|o| o.paid == o.owed));
    assert_eq!(completed(&cpu, BUILD_HOME), count);
    for p in &cpu.world.participants {
        let reports: Vec<_> = cpu.reports.iter().filter(|r| r.agent == p.agent).collect();
        assert_eq!(reports.len(), ACTIVITY_MONTHS as usize);
        assert!(
            reports
                .iter()
                .all(|r| r.deficit(NUTRITION) == 0 && r.deficit(WARMTH) == 0)
        );
    }
}
