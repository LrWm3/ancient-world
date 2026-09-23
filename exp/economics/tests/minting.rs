use economics_compute_smoke::{
    compute::Backend,
    minting::{self, *},
    model::*,
    opportunities::{self, Action},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};
fn sim(case: &str, backend: Backend) -> Simulation {
    let (w, s) = minting::scenario(case).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn supply(s: &Simulation) -> i32 {
    s.state
        .balances
        .iter()
        .filter(|((_, r), _)| *r == COIN)
        .map(|(_, v)| v)
        .sum()
}
fn minted(s: &Simulation) -> usize {
    s.state
        .processes
        .values()
        .filter(|p| p.definition == MINT && p.status == Status::Completed)
        .count()
}
#[test]
fn cpu_wheat_funds_real_inputs_then_production_is_the_only_coin_source() {
    let mut s = sim("normal", Backend::CubeCpu);
    assert_eq!(supply(&s), 12);
    s.run_months(1).unwrap();
    assert_eq!(supply(&s), 12);
    assert_eq!(s.state.balance(ISSUER, COIN), 6);
    assert_eq!(s.state.balance(ISSUER, WHEAT), 0);
    assert_eq!(s.state.balance(SUPPLIER, WHEAT), 3);
    assert_eq!(s.state.balance(WORKER, WHEAT), 3);
    s.step().unwrap(); // month two Open
    assert_eq!(s.state.phase, Phase::Acquire);
    s.step().unwrap(); // paid metal and hours
    assert_eq!(supply(&s), 12);
    assert_eq!(s.state.balance(ISSUER, COIN), 0);
    assert_eq!(s.state.balance(ISSUER, METAL), 2);
    assert_eq!(s.state.balance(ISSUER, HOURS), 2);
    assert_eq!(s.state.balance(WORKER, HOURS), 0);
    s.run_months(1).unwrap();
    assert_eq!(supply(&s), 22);
    assert_eq!(minted(&s), 1);
    assert_eq!(s.state.balance(ISSUER, METAL), 0);
    assert_eq!(s.state.balance(ISSUER, HOURS), 0);
    assert_eq!(s.state.balance(WORKER, FIREWOOD), 0);
    assert_eq!(
        (
            s.state.balance(ISSUER, COIN),
            s.state.balance(SUPPLIER, COIN),
            s.state.balance(WORKER, COIN)
        ),
        (10, 5, 7)
    );
    assert!(
        s.ledger
            .iter()
            .flat_map(|b| &b.receipts)
            .any(|r| r.agent == WORKER && r.reason == Reason::InsufficientCapacity)
    );
    s.run_months(2).unwrap();
    assert_eq!(supply(&s), 22);
    assert_eq!(minted(&s), 1);
    assert_eq!(s.state.balance(WORKER, HOURS), 2);
}
#[test]
fn missing_treasury_metal_or_labor_rejects_the_whole_package() {
    for case in ["treasury", "metal", "labor"] {
        let mut s = sim(case, Backend::CubeCpu);
        s.run_months(2).unwrap();
        assert_eq!(supply(&s), 12, "{case}");
        assert_eq!(minted(&s), 0, "{case}");
        assert_eq!(s.state.balance(ISSUER, METAL), 0);
        let b = s
            .ledger
            .iter()
            .find(|b| b.month == 2 && b.phase == Phase::Acquire)
            .unwrap();
        assert!(b.transactions.is_empty());
        let r = &b.minting.as_ref().unwrap().receipts[0];
        assert!(!r.accepted);
        assert!(r.reason.as_ref().unwrap().contains("opening available"));
        if case != "labor" {
            assert_eq!(s.state.balance(WORKER, FIREWOOD), 1);
        }
    }
}
#[test]
fn incoming_wheat_revenue_cannot_fund_its_own_batch() {
    let mut s = sim("normal", Backend::CubeCpu);
    for d in &mut s.world.minting.as_mut().unwrap().deals {
        d.month = 1;
    }
    s.run_months(1).unwrap();
    let b = s.ledger.iter().find_map(|b| b.minting.as_ref()).unwrap();
    assert!(b.receipts[0].accepted && b.receipts[1].accepted);
    assert!(!b.receipts[2].accepted);
    assert_eq!(supply(&s), 12);
    assert_eq!(s.state.balance(ISSUER, METAL), 0);
}
#[test]
fn phases_checkpoints_and_catalog_reordering_match_reference() {
    for case in ["normal", "treasury", "metal", "labor"] {
        let mut reference = sim(case, Backend::Reference);
        reference.run_months(3).unwrap();
        let mut cpu = sim(case, Backend::CubeCpu);
        cpu.world.agents.reverse();
        cpu.world.participants.reverse();
        cpu.world.resources.reverse();
        cpu.world.definitions.reverse();
        cpu.world.minting.as_mut().unwrap().deals.reverse();
        cpu.world.marketplaces[0].markets.reverse();
        while cpu.state.month <= 3 {
            cpu.step().unwrap();
            cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
                .unwrap()
                .state;
        }
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        assert_eq!(reference.reports, cpu.reports);
    }
}
#[test]
fn forged_labor_receipts_unbacked_issuance_and_replays_are_atomic() {
    let mut s = sim("normal", Backend::CubeCpu);
    s.run_months(1).unwrap();
    s.step().unwrap();
    let mut valid = Batch::empty(&s.state);
    valid.minting = minting::evaluate(&s.world, &s.state).unwrap();
    valid.transactions = valid.minting.as_ref().unwrap().transactions.clone();
    let before = s.state.clone();
    let mut forged = valid.clone();
    forged.transactions[0].effects[0].delta = 0;
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &forged,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, s.state);
    settlement::commit(
        &s.world,
        &mut s.state,
        &valid,
        Backend::CubeCpu,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    let before = s.state.clone();
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &valid,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, s.state);
    let mut counterfeit = Batch::empty(&s.state);
    let mut t = valid.transactions[0].clone();
    t.effects = vec![Effect {
        account: (ISSUER, COIN),
        delta: 10,
    }];
    counterfeit.transactions.push(t);
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &counterfeit,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, s.state);
    counterfeit.transactions[0].effects = vec![Effect {
        account: (ISSUER, HOURS),
        delta: 2,
    }];
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &counterfeit,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, s.state);
}
#[test]
fn market_admission_and_labor_and_mint_permissions_are_independent() {
    for denied in ["admission", "labor", "mint"] {
        let mut s = sim("normal", Backend::CubeCpu);
        if denied == "admission" {
            s.world.marketplaces[0]
                .allowed_types
                .remove(&opportunities::STATE_TYPE);
        }
        if denied == "labor" {
            s.world
                .transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .remove(&(opportunities::PERSON_TYPE, Action::CapacityTrade));
        }
        if denied == "mint" {
            s.world
                .transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .remove(&(opportunities::STATE_TYPE, Action::Process(MINT)));
        }
        s.run_months(2).unwrap();
        assert_eq!(minted(&s), 0);
        assert_eq!(supply(&s), 12);
    }
}
#[test]
fn observer_records_fulfilled_inputs_and_physical_issuance_without_mutation() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let mut observed = sim("normal", Backend::CubeCpu);
    let mut observer = Observer::new(
        Vec::new(),
        "minting",
        Config {
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut observed, 3).unwrap();
    let bytes = observer.finish().unwrap();
    let rows: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let issuance: Vec<_> = rows
        .iter()
        .filter(|r| r["kind"] == "physical_coin_issuance")
        .collect();
    assert_eq!(issuance.len(), 1);
    assert_eq!(issuance[0]["quantity"], 10);
    assert!(
        rows.iter()
            .any(|r| r["kind"] == "physical_minting_market" && r["accepted"] == true)
    );
    let mut plain = sim("normal", Backend::CubeCpu);
    plain.run_months(3).unwrap();
    assert_eq!(observed.state, plain.state);
    assert_eq!(observed.ledger, plain.ledger);
}

#[test]
fn double_selling_hours_fails_and_opening_cannot_invent_metal() {
    let mut s = sim("normal", Backend::CubeCpu);
    let mut duplicate = s.world.minting.as_ref().unwrap().deals[3].clone();
    duplicate.id = 5;
    duplicate.package = 4;
    s.world.minting.as_mut().unwrap().deals.push(duplicate);
    s.run_months(2).unwrap();
    let boundary = s
        .ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .find(|b| b.month == 2)
        .unwrap();
    assert!(boundary.receipts[0].accepted);
    assert!(!boundary.receipts[1].accepted);
    assert_eq!(minted(&s), 1);
    let mut s = sim("normal", Backend::CubeCpu);
    let before = s.state.clone();
    let mut fake = Batch::empty(&s.state);
    let mut trade = minting::evaluate(
        &s.world,
        &State {
            phase: Phase::Acquire,
            ..s.state.clone()
        },
    )
    .unwrap()
    .unwrap()
    .transactions[0]
        .clone();
    trade.effects = vec![Effect {
        account: (ISSUER, METAL),
        delta: 2,
    }];
    fake.transactions.push(trade);
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &fake,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, s.state);
}
