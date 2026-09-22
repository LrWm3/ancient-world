use economics_compute_smoke::{
    allocation::{Outcome, Policy},
    compute::Backend,
    model::*,
    offers::{self, Id, Request, Terms},
    pool_market,
    scenario::*,
    settlement,
    simulation::Simulation,
};

fn sim(supply: &str, backend: Backend) -> Simulation {
    let (w, s) = pool_market::scenario(supply, Policy::PriorityLottery).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn productive(s: &mut Simulation) {
    while s.state.phase != Phase::Acquire {
        s.step().unwrap();
    }
    offers::accept(s, &[]).unwrap();
    assert_eq!(s.state.phase, Phase::Productive);
}
fn requests(s: &Simulation, lots: u32) -> Vec<Request> {
    let mut r: Vec<_> = s
        .state
        .processes
        .values()
        .filter(|p| p.status == Status::Active)
        .map(|p| Request {
            offer: Id::Process(p.definition),
            agent: p.operator,
            continuing: Some(p.id),
            need: p.goal,
        })
        .collect();
    for agent in [PERSON, PERSON + 1] {
        for _ in 0..lots {
            r.push(Request::new(Id::Process(PREPARE_FUEL), agent));
        }
    }
    r
}
fn granted(b: &Batch) -> u32 {
    b.pool_market
        .as_ref()
        .unwrap()
        .receipts
        .iter()
        .map(|r| {
            if let Outcome::Reserved(n) = r.outcome {
                n
            } else {
                0
            }
        })
        .sum()
}

#[test]
fn quantities_share_one_stock_and_labor_budget_and_replay_atomically() {
    let mut s = sim("ample", Backend::CubeCpu);
    productive(&mut s);
    let opening = s.state.clone();
    let visible = offers::discover(&s.world, &s.state, PERSON);
    assert!(visible.iter().any(|o| matches!(&o.terms, Terms::Collection { supply, .. } if supply.available_stock == 4 && supply.units_per_lot == 2)));
    let r = requests(&s, 2);
    let batch = pool_market::prepare(&s, &r).unwrap();
    assert_eq!(s.state, opening);
    assert_eq!(granted(&batch), 2);
    assert_eq!(
        batch
            .pool_market
            .as_ref()
            .unwrap()
            .demands
            .iter()
            .map(|d| d.requested)
            .sum::<u32>(),
        4
    );
    assert_eq!(
        batch
            .pool_market
            .as_ref()
            .unwrap()
            .demands
            .iter()
            .map(|d| d.feasible)
            .sum::<u32>(),
        4
    );
    let mut forged = batch.clone();
    forged.pool_market.as_mut().unwrap().available_stock += 2;
    assert!(
        settlement::commit(&s.world, &mut s.state, &forged, s.backend, s.effect_limit).is_err()
    );
    assert_eq!(s.state, opening);
    let mut stripped = batch.clone();
    stripped.pool_market = None;
    assert!(
        settlement::commit(&s.world, &mut s.state, &stripped, s.backend, s.effect_limit).is_err()
    );
    settlement::commit(&s.world, &mut s.state, &batch, s.backend, s.effect_limit).unwrap();
    assert_eq!(s.state.balance(STATE_AGENT, RAW_WOOD), 0);
    assert_eq!(
        [PERSON, PERSON + 1]
            .iter()
            .map(|a| s.state.balance(*a, LABOR))
            .sum::<i32>(),
        2
    ); // Two crop-hours + two collection-hours.
    assert!(
        s.state
            .processes
            .values()
            .filter(|p| p.definition == GROW)
            .all(|p| p.status == Status::Active && p.elapsed == 1)
    );
    let after = s.state.clone();
    assert!(settlement::commit(&s.world, &mut s.state, &batch, s.backend, s.effect_limit).is_err());
    assert_eq!(s.state, after);
}

#[test]
fn urgency_changes_recipient_for_identical_opening_requests() {
    let mut s = sim("sufficient", Backend::Reference);
    productive(&mut s);
    s.state.balances.insert((PERSON, FUEL), 4);
    s.state.balances.insert((PERSON + 1, FUEL), 0);
    let r = requests(&s, 1);
    let urgent = pool_market::prepare(&s, &r).unwrap();
    let winner = |b: &Batch| {
        b.pool_market
            .as_ref()
            .unwrap()
            .receipts
            .iter()
            .find(|r| matches!(r.outcome, Outcome::Reserved(_)))
            .unwrap()
            .claim
            .id
    };
    assert_eq!(winner(&urgent), u64::from(PERSON + 1));
    s.world.pool_market.as_mut().unwrap().policy = Policy::Lottery;
    let mut changed = false;
    for seed in 0..16 {
        s.world.pool_market.as_mut().unwrap().seed = seed;
        let random = pool_market::prepare(&s, &r).unwrap();
        assert_eq!(
            urgent.pool_market.as_ref().unwrap().demands,
            random.pool_market.as_ref().unwrap().demands
        );
        assert_eq!(granted(&urgent), granted(&random));
        changed |= winner(&random) != winner(&urgent);
    }
    assert!(changed);
}

#[test]
fn unavailable_labor_inactive_demand_and_sub_lot_stock_do_not_reserve() {
    let mut s = sim("ample", Backend::Reference);
    productive(&mut s);
    s.state.balances.insert((PERSON, LABOR), 1); // Committed crop needs this hour.
    let r = requests(&s, 1);
    let b = pool_market::prepare(&s, &r).unwrap();
    assert_eq!(b.pool_market.as_ref().unwrap().demands[0].feasible, 0);
    assert_eq!(granted(&b), 1);
    s.state.balances.insert((STATE_AGENT, RAW_WOOD), 1);
    let b = pool_market::prepare(&s, &r).unwrap();
    assert_eq!(granted(&b), 0);
    assert!(
        b.transactions
            .iter()
            .flat_map(|t| &t.effects)
            .all(|e| e.account != (STATE_AGENT, RAW_WOOD))
    );
    let empty = pool_market::prepare(&s, &[]).unwrap();
    assert!(empty.pool_market.unwrap().demands.is_empty());
}

#[test]
fn cpu_reference_monthly_checkpoint_and_reversed_participants_agree() {
    let mut cpu = sim("sufficient", Backend::CubeCpu);
    let mut reference = sim("sufficient", Backend::Reference);
    reference.world.participants.reverse();
    reference.world.agents.reverse();
    cpu.run_months(12).unwrap();
    reference.run_months(12).unwrap();
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert!(
        cpu.reports
            .iter()
            .all(|r| r.deficit(WARMTH) == 0 && r.deficit(NUTRITION) == 0)
    );
    let mut resumed = cpu.clone();
    cpu.run_months(6).unwrap();
    for _ in 0..6 {
        resumed.run_months(1).unwrap();
    }
    assert_eq!(cpu.state, resumed.state);
    assert_eq!(cpu.ledger, resumed.ledger);
}

#[test]
fn scarcity_reduces_capacity_before_crop_failure_and_receipts_reconcile() {
    let mut s = sim("scarce", Backend::CubeCpu);
    s.run_months(12).unwrap();
    let (month, agent) = s
        .ledger
        .iter()
        .find_map(|b| {
            b.transactions
                .iter()
                .filter_map(|t| t.process.as_ref())
                .find(|p| p.after.definition == GROW && p.after.status == Status::Aborted)
                .map(|p| (b.month, p.after.operator))
        })
        .unwrap();
    let previous = s
        .reports
        .iter()
        .find(|r| r.month + 1 == month && r.agent == agent)
        .unwrap();
    let threshold = s
        .world
        .condition_rules
        .iter()
        .find(|r| r.subject == agent && r.provision == WARMTH)
        .unwrap()
        .impaired_at;
    assert!(previous.conditions[&WARMTH].deprivation >= threshold);
    let failed = s
        .ledger
        .iter()
        .find(|b| b.month == month && b.phase == Phase::Productive)
        .unwrap();
    assert!(failed.receipts.iter().any(|r| r.agent == agent
        && r.definition == Some(GROW)
        && r.reason == Reason::InsufficientCapacity));
    for b in s.ledger.iter().filter(|b| b.phase == Phase::Productive) {
        let round = b.pool_market.as_ref().unwrap();
        let completed = b
            .transactions
            .iter()
            .filter_map(|t| t.process.as_ref())
            .filter(|p| p.after.definition == PREPARE_FUEL && p.after.status == Status::Completed)
            .count();
        assert_eq!(completed, granted(b) as usize);
        let removed: i32 = b
            .transactions
            .iter()
            .flat_map(|t| &t.effects)
            .filter(|e| e.account == (STATE_AGENT, RAW_WOOD))
            .map(|e| -e.delta)
            .sum();
        assert_eq!(removed, granted(b) as i32 * round.units_per_lot);
        assert!(removed <= round.available_stock);
    }
}
