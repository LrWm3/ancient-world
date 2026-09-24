use economics_compute_smoke::{
    accounting::Account as A, compute::Backend, crafts::*, equipment::DurableAsset,
    financial_reporting::Audit, model::*, scenario::*, simulation::Simulation,
};
use std::collections::BTreeMap;

fn fixture() -> (World, State) {
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
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    let costs = s
        .balances
        .iter()
        .filter(|((_, r), q)| {
            *r != TOKEN
                && **q > 0
                && w.resources
                    .iter()
                    .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
        })
        .map(|(k, q)| (*k, i128::from(*q)))
        .collect();
    let mut assets: BTreeMap<_, _> = w.assets.iter().map(|a| (a.id, 0)).collect();
    assets.extend(
        s.equipment
            .iter()
            .map(|(id, a)| (*id, i128::from(a.remaining_uses))),
    );
    Audit::with_processes(w, s, TOKEN, assets, costs, BTreeMap::new()).unwrap()
}
fn schedule(w: &mut World, month: u32, ds: &[u32]) {
    w.scheduled_starts = ds
        .iter()
        .map(|d| ScheduledStart {
            agent: PERSON,
            month,
            definition: *d,
        })
        .collect();
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
fn basis(a: &Audit, id: u32) -> i128 {
    a.book()
        .balances()
        .get(&(PERSON, A::Tangible(id)))
        .copied()
        .unwrap_or(0)
}
#[test]
fn manufacture_capitalizes_materials_and_resumes_identically_on_cpu() {
    let (mut w, s) = fixture();
    w.activities.kinds.get_mut(&HOUSE).unwrap().monthly_decay = 1;
    schedule(&mut w, 1, &[BUILD_HOME]);
    let mut a = audit(&w, &s);
    let mut b = a.clone();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut cpu, 1);
    through(&mut b, &mut reference, 1);
    assert_eq!(a, b);
    assert!(
        a.book()
            .balances()
            .iter()
            .any(|((_, k), v)| matches!(k, A::WorkInProgress(_)) && *v == 8)
    );
    let mut resumed = a.clone();
    let mut checkpoint = cpu.clone();
    through(&mut a, &mut cpu, 2);
    through(&mut b, &mut reference, 2);
    through(&mut resumed, &mut checkpoint, 2);
    assert_eq!(a, b);
    assert_eq!(a, resumed);
    let home = cpu
        .state
        .equipment
        .values()
        .find(|v| v.kind == HOUSE)
        .unwrap()
        .id;
    assert_eq!(basis(&a, home), 8);
    assert_eq!(a.book().statements(PERSON, 1, 2).unwrap().net_income, 0);
    through(&mut a, &mut cpu, 122);
    assert_eq!(basis(&a, home), 0);
    assert_eq!(
        a.book().statements(PERSON, 1, 122).unwrap().expenses[&A::Depreciation],
        8
    );
}
#[test]
fn repair_restores_capacity_without_revaluing_and_expenses_helper_wear() {
    let (mut w, mut s) = fixture();
    for (id, kind, uses) in [(9000, PICK, 1), (9001, GRINDING_STONE, 24)] {
        s.equipment.insert(
            id,
            DurableAsset {
                id,
                owner: PERSON,
                attached_to: None,
                kind,
                remaining_uses: uses,
                last_used_month: None,
            },
        );
    }
    schedule(&mut w, 1, &[repair(PICK)]);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 1);
    assert_eq!(sim.state.equipment[&9000].remaining_uses, 13);
    assert_eq!(basis(&a, 9000), 1);
    assert_eq!(basis(&a, 9001), 23);
    assert_eq!(
        a.book().statements(PERSON, 1, 1).unwrap().expenses[&A::ProductionExpense],
        1
    );
}
#[test]
fn required_equipment_cost_flows_into_output_and_missing_material_creates_no_asset() {
    let (mut w, mut s) = fixture();
    s.equipment.insert(
        9000,
        DurableAsset {
            id: 9000,
            owner: PERSON,
            attached_to: None,
            kind: HAMMER_STONE,
            remaining_uses: 24,
            last_used_month: None,
        },
    );
    w.techniques.retain(|t| t.definition != craft(KNIFE));
    w.activities.required.insert(craft(KNIFE), HAMMER_STONE);
    schedule(&mut w, 1, &[craft(KNIFE)]);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 1);
    let knife = sim
        .state
        .equipment
        .values()
        .find(|a| a.kind == KNIFE)
        .unwrap()
        .id;
    assert_eq!(basis(&a, 9000), 23);
    assert_eq!(basis(&a, knife), 3);
    let (mut w, mut s) = fixture();
    s.balances.insert((PERSON, STONE), 0);
    schedule(&mut w, 1, &[craft(HAMMER_STONE)]);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 1);
    assert!(sim.state.equipment.is_empty());
}

#[test]
fn joint_stock_and_durable_output_without_policy_is_rejected_atomically() {
    let (mut w, s) = fixture();
    w.definitions
        .iter_mut()
        .find(|d| d.id == craft(HAMMER_STONE))
        .unwrap()
        .outputs
        .push(Amount::new(GRAIN, 1));
    schedule(&mut w, 1, &[craft(HAMMER_STONE)]);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    loop {
        let old = a.clone();
        let state = sim.state.clone();
        if let Err(e) = a.step(&mut sim) {
            assert!(e.contains("joint durable/stock"), "{e}");
            assert_eq!(a, old);
            assert_eq!(sim.state, state);
            break;
        }
        assert!(sim.state.month <= 1);
    }
}

#[test]
fn joint_outputs_split_wip_cost_with_exact_rounding_on_cpu_and_resume() {
    use economics_compute_smoke::process_accounting::Output;
    let (mut w, s) = fixture();
    let d = w
        .definitions
        .iter_mut()
        .find(|d| d.id == BUILD_HOME)
        .unwrap();
    d.outputs = vec![Amount::new(GRAIN, 2), Amount::new(FUEL, 1)];
    schedule(&mut w, 1, &[BUILD_HOME]);
    let weights = BTreeMap::from([(
        BUILD_HOME,
        BTreeMap::from([
            (Output::Stock(GRAIN), 1),
            (Output::Stock(FUEL), 1),
            (Output::Durable(HOUSE), 1),
        ]),
    )]);
    let mut a = audit(&w, &s).with_output_cost_policy(&w, weights).unwrap();
    let mut b = a.clone();
    let opening = a.book().balances().clone();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut cpu, 1);
    through(&mut b, &mut reference, 1);
    assert_eq!(a, b);
    assert!(
        a.book()
            .balances()
            .iter()
            .any(|((_, k), v)| matches!(k, A::WorkInProgress(_)) && *v == 8)
    );
    let mut resumed = a.clone();
    let mut checkpoint = cpu.clone();
    through(&mut a, &mut cpu, 2);
    through(&mut b, &mut reference, 2);
    through(&mut resumed, &mut checkpoint, 2);
    assert_eq!(a, b);
    assert_eq!(a, resumed);
    assert_eq!(cpu.state, reference.state);
    let home = cpu
        .state
        .equipment
        .values()
        .find(|v| v.kind == HOUSE)
        .unwrap()
        .id;
    assert_eq!(basis(&a, home), 3);
    // Shares apply to each complete output lot, not each physical unit.
    assert_eq!(
        a.book().balances()[&(PERSON, A::Inventory(GRAIN))]
            - opening[&(PERSON, A::Inventory(GRAIN))],
        2
    );
    assert_eq!(
        a.book().balances()[&(PERSON, A::Inventory(FUEL))] - opening[&(PERSON, A::Inventory(FUEL))],
        3
    );
    assert_eq!(a.book().statements(PERSON, 1, 2).unwrap().net_income, 0);
    assert!(
        !a.book()
            .balances()
            .iter()
            .any(|((_, k), v)| matches!(k, A::WorkInProgress(_)) && *v != 0)
    );
    assert!(
        a.clone()
            .with_output_cost_policy(&cpu.world, BTreeMap::new())
            .is_err()
    );
}

#[test]
fn joint_cost_policy_rejects_missing_extra_zero_and_wrong_kind_shares() {
    use economics_compute_smoke::process_accounting::Output;
    let (mut w, s) = fixture();
    w.definitions
        .iter_mut()
        .find(|d| d.id == BUILD_HOME)
        .unwrap()
        .outputs = vec![Amount::new(GRAIN, 1)];
    for shares in [
        vec![(Output::Stock(GRAIN), 1)],
        vec![(Output::Durable(HOUSE), 1)],
        vec![(Output::Stock(GRAIN), 0), (Output::Durable(HOUSE), 1)],
        vec![(Output::Stock(GRAIN), 1), (Output::Durable(PICK), 1)],
        vec![
            (Output::Stock(GRAIN), 1),
            (Output::Durable(HOUSE), 1),
            (Output::Stock(SEED), 1),
        ],
    ] {
        assert!(
            audit(&w, &s)
                .with_output_cost_policy(
                    &w,
                    BTreeMap::from([(BUILD_HOME, shares.into_iter().collect())])
                )
                .is_err()
        );
    }
}

#[test]
fn actual_outputs_must_still_match_the_opening_cost_policy() {
    use economics_compute_smoke::process_accounting::Output;
    for (changed_outputs, remove_durable) in [
        (vec![Amount::new(SEED, 1)], false),
        (vec![], false),
        (vec![Amount::new(GRAIN, 1)], true),
    ] {
        let (mut w, s) = fixture();
        w.definitions
            .iter_mut()
            .find(|d| d.id == BUILD_HOME)
            .unwrap()
            .outputs = vec![Amount::new(GRAIN, 1)];
        schedule(&mut w, 1, &[BUILD_HOME]);
        let mut a = audit(&w, &s)
            .with_output_cost_policy(
                &w,
                BTreeMap::from([(
                    BUILD_HOME,
                    BTreeMap::from([(Output::Stock(GRAIN), 1), (Output::Durable(HOUSE), 1)]),
                )]),
            )
            .unwrap();
        // Catalog change after opening cannot silently move all cost into a different stock.
        w.definitions
            .iter_mut()
            .find(|d| d.id == BUILD_HOME)
            .unwrap()
            .outputs = changed_outputs;
        if remove_durable {
            w.activities.outcomes.remove(&BUILD_HOME);
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        loop {
            let old = a.clone();
            let state = sim.state.clone();
            if let Err(e) = a.step(&mut sim) {
                assert!(e.contains("output cost shares"), "{e}");
                assert_eq!(a, old);
                assert_eq!(sim.state, state);
                break;
            }
            assert!(sim.state.month <= 2);
        }
    }
}
