use economics_compute_smoke::{
    acceptance::{self, Run},
    compute::Backend,
};

#[test]
fn acceptance_baseline_reconciles_actual_cross_agent_outcomes() {
    let (w, s) = acceptance::scenario().unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    assert!(run.evidence().is_err());
    run.through(acceptance::RUN_MONTHS).unwrap();
    let e = run.evidence().unwrap();
    assert_eq!(e.months, 14);
    assert_eq!(e.household_on_time_delivery, 2);
    assert!(e.state_loan_repaid);
    assert_eq!(e.state_loan_outstanding, Some(0));
    assert_eq!(e.household_delivered, 2);
    assert_eq!(e.land_dues_paid, 1);
    assert!(e.completed_mints > 0);
    assert_eq!(e.issuance, 10 * e.completed_mints as i128);
}

#[test]
fn workshop_requires_real_inputs_and_hours_to_supply_the_state() {
    use economics_compute_smoke::{
        acceptance::{SMITH, WORKSHOP},
        minting::{COIN, ORE},
    };
    let (w, s) = acceptance::scenario().unwrap();
    let mut run = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    run.through(acceptance::RUN_MONTHS).unwrap();
    let e = run.evidence().unwrap();
    assert_eq!(e.workshop_batches, 4);
    assert_eq!(e.workshop_wages_paid, 8);
    assert_eq!(
        run.sim.state.balance(SMITH, COIN),
        1 + e.workshop_wages_paid + e.municipal_wages_paid - 3 * e.smith_food_purchases as i32
    );
    assert_eq!(
        run.sim.state.balance(WORKSHOP, COIN),
        2 * e.workshop_metal_sales as i32
    );
    assert_eq!(e.reconciled_agents, 9);
    for missing_labor in [false, true] {
        let (mut w, mut s) = (w.clone(), s.clone());
        if missing_labor {
            w.participants
                .iter_mut()
                .find(|p| p.agent == SMITH)
                .unwrap()
                .capacity
                .quantity = 0;
        } else {
            s.balances.insert((WORKSHOP, ORE), 0);
        }
        let mut control = Run::new(w, s, Backend::Reference).unwrap();
        control.through(acceptance::RUN_MONTHS).unwrap();
        let c = control.evidence().unwrap();
        assert_eq!(c.workshop_batches, 0);
        assert_eq!(c.completed_mints, 0);
        assert_eq!(c.issuance, 0);
    }
}

#[test]
fn municipality_uses_only_residual_worker_hours_and_receives_real_land_dues() {
    use economics_compute_smoke::{
        acceptance::{SMITH, TOWN, TOWN_EMPLOYMENT, WORKSHOP_EMPLOYMENT},
        agency::integration,
        minting::{COIN, FIREWOOD},
    };
    let (w, s) = acceptance::scenario().unwrap();
    assert_eq!(
        w.assets
            .iter()
            .find(|a| a.id == integration::PLOT)
            .unwrap()
            .owner,
        TOWN
    );
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(4).unwrap();
    assert_eq!(run.evidence().unwrap().municipal_batches, 0);
    assert_eq!(run.sim.state.balance(TOWN, COIN), 10);
    run.through(acceptance::RUN_MONTHS).unwrap();
    let e = run.evidence().unwrap();
    assert_eq!(e.municipal_batches, 10);
    assert_eq!(e.municipal_wages_paid, 10);
    assert_eq!(e.municipal_cash, 1);
    assert_eq!(run.sim.state.balance(TOWN, FIREWOOD), 10);
    assert_eq!(
        run.sim.state.balance(SMITH, COIN),
        1 + e.workshop_wages_paid + e.municipal_wages_paid - 3 * e.smith_food_purchases as i32
    );
    for month in 1..=acceptance::RUN_MONTHS {
        let delivered: i32 = run
            .sim
            .state
            .employment
            .earned
            .iter()
            .filter(|((id, m), _)| {
                *m == month && [WORKSHOP_EMPLOYMENT, TOWN_EMPLOYMENT].contains(id)
            })
            .map(|(_, e)| e.delivered)
            .sum();
        assert!(delivered <= 2, "duplicated worker hours in month {month}");
    }
}

#[test]
fn worker_wages_purchase_market_food_and_missing_asks_expose_shortages() {
    use economics_compute_smoke::{agency::integration, minting::WHEAT};
    let (w, s) = acceptance::scenario().unwrap();
    let mut run = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    run.through(acceptance::RUN_MONTHS).unwrap();
    let e = run.evidence().unwrap();
    assert!(e.household_food_sales > 0);
    assert!(e.smith_food_purchases > 0);
    assert_eq!(e.smith_food_deficit, 0);
    assert!(acceptance::accepted_deals(&run.sim).any(|d| {
        [integration::HOUSEHOLD, integration::GROWER].contains(&d.seller)
            && d.buyer == acceptance::SMITH
            && d.market == WHEAT
    }));
    let mut unavailable = w;
    unavailable
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes
        .retain(|q| ![integration::HOUSEHOLD, integration::GROWER].contains(&q.agent));
    let mut control = Run::new(unavailable, s, Backend::Reference).unwrap();
    control.through(acceptance::RUN_MONTHS).unwrap();
    let c = control.evidence().unwrap();
    assert_eq!(c.household_food_sales, 0);
    assert!(c.smith_food_deficit > e.smith_food_deficit);
}

#[test]
fn accepted_deal_evidence_does_not_count_uncommitted_packages() {
    let (w, s) = acceptance::scenario().unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(acceptance::RUN_MONTHS).unwrap();
    assert!(acceptance::accepted_deals(&run.sim).count() > 0);
    // Corrupt only a diagnostic clone: forecasts/deal proposals must not count as fills.
    let mut proposed_only = run.sim.clone();
    for b in &mut proposed_only.ledger {
        if let Some(m) = &mut b.minting {
            for r in &mut m.receipts {
                r.accepted = false;
            }
        }
    }
    assert_eq!(acceptance::accepted_deals(&proposed_only).count(), 0);
}

#[test]
fn acceptance_controls_expose_authority_liquidity_and_contention_failures() {
    use acceptance::{Case, TOWN_EMPLOYMENT};
    let run = |case| {
        let (w, s) = acceptance::scenario_for(case).unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(acceptance::RUN_MONTHS).unwrap();
        run
    };
    let base = run(Case::Baseline).evidence().unwrap();
    for case in [
        Case::WorkshopExcluded,
        Case::WorkshopProhibited,
        Case::NoOre,
        Case::NoSmithHours,
    ] {
        let r = run(case);
        let e = r.evidence().unwrap();
        assert_eq!(e.completed_mints, 0, "{case:?}");
        assert_eq!(e.workshop_metal_sales, 0, "{case:?}");
        assert_eq!(e.issuance, 0, "{case:?}");
        if case == Case::WorkshopExcluded {
            assert!(e.workshop_batches > 0);
        } else {
            assert_eq!(e.workshop_batches, 0);
        }
    }
    let priority = run(Case::TownFirst);
    let e = priority.evidence().unwrap();
    assert_eq!(e.workshop_batches, 0);
    assert_eq!(e.completed_mints, 0);
    assert!(
        priority
            .sim
            .state
            .employment
            .earned
            .contains_key(&(TOWN_EMPLOYMENT, 1))
    );
    // Earlier delivered work remains due even when its employer later runs out of cash.
    let broke = run(Case::UnfundedTown);
    let e = broke.evidence().unwrap();
    assert!(e.municipal_batches < base.municipal_batches);
    assert!(e.wage_arrears > 0);
    assert!(e.municipal_batches > 0);
    let e = run(Case::UnfundedWorkshop).evidence().unwrap();
    assert!(e.workshop_batches < base.workshop_batches);
    assert!(e.workshop_wages_paid < base.workshop_wages_paid);
}

#[test]
fn supply_controls_distinguish_private_offers_production_and_storage() {
    use acceptance::Case;
    let run = |case| {
        let (w, s) = acceptance::scenario_for(case).unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(acceptance::RUN_MONTHS).unwrap();
        run.evidence().unwrap()
    };
    let base = run(Case::Baseline);
    let closed = run(Case::NoPrivateAsks);
    assert_eq!(closed.household_food_sales + closed.grower_food_sales, 0);
    assert!(closed.people_food_deficit > base.people_food_deficit);
    let wood = run(Case::CrowdedWood);
    assert!(wood.grower_storage_rejections > base.grower_storage_rejections);
    assert!(wood.people_food_deficit > base.people_food_deficit);
    assert_eq!(base.people_food_deficit, 0);
    let food = run(Case::TightFood);
    assert!(food.people_food_deficit > base.people_food_deficit);
    let needs_only = run(Case::NoSalesTarget);
    assert!(needs_only.people_food_deficit >= base.people_food_deficit);
}

#[test]
fn all_acceptance_cases_match_cpu_with_reconstruction_at_every_phase() {
    use economics_compute_smoke::simulation::Simulation;
    for case in acceptance::Case::ALL {
        let (w, s) = acceptance::scenario_for(case).unwrap();
        let mut reference = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
        reference.through(acceptance::RUN_MONTHS).unwrap();
        let mut cpu = Run::new(w, s, Backend::CubeCpu).unwrap();
        while cpu.sim.state.month <= acceptance::RUN_MONTHS {
            cpu.audit.step(&mut cpu.sim).unwrap();
            let mut next = Simulation::new(
                cpu.sim.world.clone(),
                cpu.sim.state.clone(),
                Backend::CubeCpu,
            )
            .unwrap();
            next.ledger = cpu.sim.ledger;
            next.reports = cpu.sim.reports;
            next.effect_limit = cpu.sim.effect_limit;
            cpu.sim = next;
        }
        assert_eq!(cpu.sim.world, reference.sim.world, "{case:?}");
        assert_eq!(cpu.sim.state, reference.sim.state, "{case:?}");
        assert_eq!(cpu.sim.ledger, reference.sim.ledger, "{case:?}");
        assert_eq!(cpu.sim.reports, reference.sim.reports, "{case:?}");
        assert_eq!(cpu.audit, reference.audit, "{case:?}");
        assert_eq!(
            cpu.evidence().unwrap(),
            reference.evidence().unwrap(),
            "{case:?}"
        );
    }
}

#[test]
fn mixed_economy_replays_with_dated_governance_context_and_survives_catalog_reordering() {
    use economics_compute_smoke::{demography, model::Phase, settlement::DEFAULT_EFFECT_LIMIT};
    let (w, s) = acceptance::scenario().unwrap();
    let mut baseline = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut replay_state = s.clone();
    let mut replay_audit = baseline.audit.clone();
    while baseline.sim.state.month <= acceptance::RUN_MONTHS {
        baseline.audit.step(&mut baseline.sim).unwrap();
        let batch = baseline.sim.ledger.last().unwrap();
        // Governance lives in World and may change during Open. These dated
        // contexts are required inputs, not reconstructed from ledger alone.
        let mut context = baseline.sim.world.clone();
        let before = replay_state.clone();
        demography::replay(
            &mut context,
            &mut replay_state,
            batch,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT,
        )
        .unwrap();
        replay_audit
            .record(&context, &before, batch, &replay_state)
            .unwrap();
        assert_eq!(replay_state, baseline.sim.state);
    }
    assert_eq!(replay_audit, baseline.audit);
    let mut reordered = w;
    reordered.agents.reverse();
    reordered.participants.reverse();
    reordered.definitions.reverse();
    reordered.resources.reverse();
    reordered.employment.reverse();
    let mut monthly = Run::new(reordered, s, Backend::CubeCpu).unwrap();
    for month in 1..=acceptance::RUN_MONTHS {
        monthly.through(month).unwrap();
    }
    assert_eq!(monthly.sim.world.agency, baseline.sim.world.agency);
    assert_eq!(
        monthly.sim.world.state_governance,
        baseline.sim.world.state_governance
    );
    assert_eq!(monthly.sim.world.households, baseline.sim.world.households);
    assert_eq!(monthly.sim.state, baseline.sim.state);
    assert_eq!(monthly.sim.ledger, baseline.sim.ledger);
    assert_eq!(monthly.sim.reports, baseline.sim.reports);
    assert_eq!(monthly.audit, baseline.audit);
    assert_eq!(monthly.evidence().unwrap(), baseline.evidence().unwrap());
    assert_eq!(baseline.sim.state.phase, Phase::Open);
    assert_eq!(baseline.evidence().unwrap().people_food_deficit, 0);
    // Completed production first becomes tradeable at a subsequent Acquire.
    assert!(
        acceptance::accepted_deals(&baseline.sim)
            .filter(|d| d.seller == acceptance::WORKSHOP)
            .all(|d| d.month > 1)
    );
    assert!(
        Run::new(
            baseline.sim.world.clone(),
            baseline.sim.state.clone(),
            Backend::Reference
        )
        .is_err()
    );
}
