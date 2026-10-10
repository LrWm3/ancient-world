use economics_compute_smoke::{
    agency::integration::{GROW, GROWER, SEED},
    compute::Backend,
    credit,
    discovery::scenario,
    financial_reporting::Audit,
    minting::*,
    model::*,
    opportunities::{Action, PERSON_TYPE},
    settlement,
    simulation::Simulation,
};

fn run(w: World, s: State, backend: Backend, months: u32) -> (Simulation, Audit) {
    let mut audit = scenario::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, backend).unwrap();
    while sim.state.month <= months {
        audit
            .step(&mut sim)
            .unwrap_or_else(|e| panic!("{} {:?}: {e}", sim.state.month, sim.state.phase));
    }
    (sim, audit)
}
fn completed(s: &State, definition: DefinitionId) -> usize {
    s.processes
        .values()
        .filter(|p| p.definition == definition && p.status == Status::Completed)
        .count()
}

#[test]
fn empty_agreements_and_menus_bootstrap_into_an_audited_cpu_economy() {
    let (w, s) = scenario::scenario().unwrap();
    assert!(w.agency.is_empty() && w.households.is_empty() && w.state_governance.is_none());
    assert!(w.agreements.is_empty() && w.access_offers.is_empty() && w.rights.is_empty());
    assert!(w.lending.is_empty() && w.prepaid_deliveries.is_empty() && w.employment.is_empty());
    assert!(w.scheduled_starts.is_empty() && w.activities.orders.is_empty());
    assert!(s.memberships.is_empty() && s.accepted_agreements.is_empty());
    assert!(
        w.transaction_policy
            .as_ref()
            .unwrap()
            .membership_offers
            .is_empty()
    );
    assert!(
        w.minting
            .as_ref()
            .unwrap()
            .order_policy
            .as_ref()
            .unwrap()
            .quotes
            .is_empty()
    );
    let (reference, expected_book) = run(
        w.clone(),
        s.clone(),
        Backend::Reference,
        scenario::RUN_MONTHS,
    );
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut audit = scenario::audit(&sim.world, &sim.state).unwrap();
    while sim.state.month <= scenario::RUN_MONTHS {
        audit.step(&mut sim).unwrap();
        // Reconstruct at every barrier, retaining the exact issued decisions.
        let mut resumed =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
        resumed.ledger = sim.ledger;
        resumed.reports = sim.reports;
        sim = resumed;
    }
    assert_eq!(sim.world, reference.world);
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    assert_eq!(sim.reports, reference.reports);
    assert_eq!(audit, expected_book);
    assert_eq!(sim.state.memberships.len(), 3);
    assert_eq!(sim.world.households.len(), 1);
    assert_eq!(sim.state.accepted_agreements.len(), 1);
    assert_eq!(sim.state.accepted_agreements[&1].debtor, GROWER);
    assert!(completed(&sim.state, GROW) > 1);
    assert!(completed(&sim.state, MINT) > 1);
    assert!(
        sim.state
            .credit
            .loans
            .values()
            .any(|l| l.status == credit::Status::Repaid)
    );
    assert_eq!(sim.state.obligations[&(1, 14)].outstanding(), 0);
    assert_eq!(
        sim.reports
            .iter()
            .map(|r| r.deficit(NUTRITION))
            .sum::<i32>(),
        2
    );
    assert!(
        sim.reports
            .iter()
            .filter(|r| r.month > 1)
            .all(|r| r.deficit(NUTRITION) == 0)
    );
    assert!(
        sim.world
            .agency
            .values()
            .all(|c| c.discover_programs && c.config.programs.is_empty())
    );
    assert!(
        sim.world.agency[&ISSUER]
            .history
            .iter()
            .any(|d| d.accepted.is_some())
    );
}

#[test]
fn catalogue_iteration_order_does_not_select_signatories_or_work() {
    let (w, s) = scenario::scenario().unwrap();
    let (reference, book) = run(w.clone(), s.clone(), Backend::Reference, 6);
    let mut shuffled = w;
    shuffled.agents.reverse();
    shuffled.participants.reverse();
    shuffled.definitions.reverse();
    shuffled.assets.reverse();
    shuffled.resources.reverse();
    let (other, other_book) = run(shuffled, s, Backend::Reference, 6);
    assert_eq!(reference.state, other.state);
    assert_eq!(reference.ledger, other.ledger);
    assert_eq!(reference.world.discovery, other.world.discovery);
    assert_eq!(reference.world.agency, other.world.agency);
    assert_eq!(book, other_book);
}

#[test]
fn missing_seed_does_not_turn_an_offer_into_a_feasible_lease() {
    let (w, mut s) = scenario::scenario().unwrap();
    s.balances.remove(&(GROWER, SEED));
    let (sim, _) = run(w, s, Backend::Reference, 4);
    assert!(sim.state.accepted_agreements.is_empty());
    assert_eq!(completed(&sim.state, GROW), 0);
    assert!(sim.reports.iter().any(|r| r.deficit(NUTRITION) > 0));
}

#[test]
fn law_can_deny_citizenship_without_hidden_governance_or_households() {
    let (mut w, s) = scenario::scenario().unwrap();
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(PERSON_TYPE, Action::Membership));
    let (sim, _) = run(w, s, Backend::Reference, 4);
    assert!(sim.state.memberships.is_empty());
    assert!(sim.world.state_governance.is_none());
    assert!(sim.world.households.is_empty());
    assert!(sim.state.accepted_agreements.is_empty());
    assert_eq!(completed(&sim.state, MINT), 0);
}

#[test]
fn no_household_forms_if_everyone_is_already_materially_covered() {
    let (mut w, s) = scenario::scenario().unwrap();
    for p in &mut w.participants {
        p.needs.clear();
    }
    let (sim, _) = run(w, s, Backend::Reference, 3);
    assert!(sim.world.households.is_empty());
    assert!(
        sim.world
            .discovery
            .unwrap()
            .receipts
            .iter()
            .filter(|r| r.description.starts_with("household proposal"))
            .all(|r| !r.accepted)
    );
}

#[test]
fn law_can_deny_formation_and_prevent_an_unfulfillable_land_commitment() {
    let (mut w, s) = scenario::scenario().unwrap();
    w.transaction_policy
        .as_mut()
        .unwrap()
        .membership_permissions
        .remove(&(
            economics_compute_smoke::membership::CITIZEN,
            Action::FoundHousehold,
        ));
    let (sim, _) = run(w, s, Backend::Reference, 4);
    assert!(sim.world.households.is_empty());
    assert!(sim.state.accepted_agreements.is_empty());
}

#[test]
fn no_lender_funds_means_no_invented_advance() {
    let (w, mut s) = scenario::scenario().unwrap();
    s.balances.retain(|(_, resource), _| *resource != COIN);
    let (sim, _) = run(w, s, Backend::Reference, 4);
    assert!(sim.world.lending.is_empty());
    assert!(sim.state.credit.loans.is_empty());
    assert_eq!(completed(&sim.state, MINT), 0);
}

#[test]
fn generated_prepaid_delivery_can_be_selected_and_settled_without_supplied_terms() {
    let (w, s) = scenario::surplus().unwrap();
    let (sim, _) = run(w, s, Backend::Reference, 8);
    assert!(
        sim.world.prepaid_deliveries.len() >= 2,
        "{:?}",
        sim.world.discovery.unwrap().receipts
    );
    assert!(
        sim.state
            .exchange
            .forwards
            .values()
            .filter(|f| f.delivered == f.goods.quantity)
            .count()
            >= 2
    );
}

#[test]
fn altered_acceptance_and_failed_open_do_not_publish_resources_or_relationships() {
    let (w, s) = scenario::scenario().unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let before_world = sim.world.clone();
    let before_state = sim.state.clone();
    sim.effect_limit = 0;
    assert!(sim.step().is_err());
    assert_eq!(sim.world, before_world);
    assert_eq!(sim.state, before_state);
    sim.effect_limit = settlement::DEFAULT_EFFECT_LIMIT;
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    let opening = sim.state.clone();
    sim.step().unwrap();
    let mut altered = sim.ledger.last().unwrap().clone();
    altered.additional_memberships.clear();
    let mut replay = opening.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut replay,
            &altered,
            Backend::Reference,
            settlement::DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, opening);
}

#[test]
fn discovered_program_snapshot_cannot_be_rewritten_to_another_policy() {
    let (w, s) = scenario::scenario().unwrap();
    let (mut sim, _) = run(w, s, Backend::Reference, 3);
    let c = sim.world.agency.get_mut(&ISSUER).unwrap();
    let d = c.history.iter_mut().find(|d| d.chosen.is_some()).unwrap();
    d.catalog.get_mut(&d.chosen.unwrap()).unwrap().name = "rewritten".into();
    assert!(Simulation::new(sim.world, sim.state, Backend::Reference).is_err());
}

#[test]
fn expired_land_is_reoffered_and_renewed_without_dated_fixture_intervention() {
    let (w, s) = scenario::scenario().unwrap();
    let (sim, _) = run(w, s, Backend::Reference, 40);
    assert_eq!(sim.state.accepted_agreements.len(), 2);
    assert_eq!(sim.state.accepted_agreements[&2].activated, 26);
    assert_eq!(sim.state.obligations[&(2, 38)].outstanding(), 0);
    assert!(
        sim.reports
            .iter()
            .filter(|r| r.month > 1)
            .all(|r| r.deficit(NUTRITION) == 0)
    );
}

#[test]
fn two_feasible_applicants_cannot_double_book_household_labor() {
    let (mut w, mut s) = scenario::scenario().unwrap();
    s.balances.insert((SUPPLIER, SEED), 4);
    w.assets.push(Asset {
        id: 778,
        owner: ISSUER,
        kind: 1,
    });
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.month < 2 || sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    assert_eq!(sim.world.households.len(), 1);
    let opening = sim.state.clone();
    sim.step().unwrap();
    let b = sim.ledger.last().unwrap();
    assert_eq!(b.discovery_allocation.len(), 2);
    assert_eq!(b.additional_access.len(), 1);
    assert!(matches!(
        b.discovery_allocation[0].outcome,
        economics_compute_smoke::allocation::Outcome::Reserved(1)
    ));
    assert!(matches!(
        b.discovery_allocation[1].outcome,
        economics_compute_smoke::allocation::Outcome::Rejected(_)
    ));
    let mut altered = b.clone();
    altered.discovery_allocation[1].outcome =
        economics_compute_smoke::allocation::Outcome::Reserved(1);
    let mut replay = opening.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut replay,
            &altered,
            Backend::Reference,
            settlement::DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, opening);
}

#[test]
fn land_admission_preserves_unpaid_process_inputs_but_not_consumed_inputs() {
    for elapsed in [0, 1] {
        let (mut w, s) = scenario::scenario().unwrap();
        let definition = ProcessDefinition {
            id: 901,
            name: "prior seed commitment".into(),
            execution: Execution::Productive,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "waiting".into(),
                months: 2,
                entry_inputs: vec![Amount::new(SEED, 4)],
                monthly_services: vec![],
            }],
            outputs: vec![Amount::new(METAL, 1)],
        };
        w.definitions.push(definition);
        w.transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .insert((PERSON_TYPE, Action::Process(901)));
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        while sim.state.month < 2 || sim.state.phase != Phase::Acquire {
            sim.step().unwrap();
        }
        sim.state.processes.insert(
            901,
            ProcessInstance {
                id: 901,
                definition: 901,
                operator: GROWER,
                beneficiary: GROWER,
                goal: None,
                asset: None,
                right: None,
                start: 2 - elapsed,
                reserved_through: 3 - elapsed,
                stage: 0,
                elapsed,
                status: Status::Active,
            },
        );
        let mut cpu =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
        sim.step().unwrap();
        cpu.step().unwrap();
        assert_eq!(sim.state, cpu.state);
        assert_eq!(sim.ledger.last(), cpu.ledger.last());
        assert_eq!(
            sim.state.accepted_agreements.len(),
            usize::from(elapsed > 0)
        );
        // Existing work still executes after the admission decision.
        sim.run_months(2).unwrap();
        assert_eq!(
            sim.state.processes[&901].status,
            Status::Completed,
            "elapsed={elapsed}"
        );
    }
}

#[test]
fn unrepresentable_discovered_demand_rejects_open_atomically() {
    let (mut w, s) = scenario::scenario().unwrap();
    w.participants
        .iter_mut()
        .find(|p| p.agent == SUPPLIER)
        .unwrap()
        .needs[0]
        .quantity = i32::MAX;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let before = (sim.world.clone(), sim.state.clone(), sim.ledger.clone());
    let error = sim.step().unwrap_err();
    assert!(error.contains("food target overflow"), "{error}");
    assert_eq!((sim.world, sim.state, sim.ledger), before);
}

#[test]
fn discovered_land_policy_changes_winners_without_changing_joint_capacity() {
    use economics_compute_smoke::allocation::Policy;
    let (mut w, mut s) = scenario::scenario().unwrap();
    s.balances.insert((SUPPLIER, SEED), 4);
    w.assets.push(Asset {
        id: 778,
        owner: ISSUER,
        kind: 1,
    });
    let mut opening = Simulation::new(w, s, Backend::Reference).unwrap();
    while opening.state.month < 2 || opening.state.phase != Phase::Acquire {
        opening.step().unwrap();
    }
    let mut winners = std::collections::BTreeSet::new();
    let mut requests = None;
    for (policy, seed) in std::iter::once((Policy::StablePriority, 0))
        .chain((0..8).map(|seed| (Policy::Lottery, seed)))
    {
        let mut world = opening.world.clone();
        world.discovery.as_mut().unwrap().land_allocation = policy;
        world.discovery.as_mut().unwrap().land_seed = seed;
        let mut reference =
            Simulation::new(world.clone(), opening.state.clone(), Backend::Reference).unwrap();
        world.participants.reverse();
        let mut cpu = Simulation::new(world, opening.state.clone(), Backend::CubeCpu).unwrap();
        reference.step().unwrap();
        cpu.step().unwrap();
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        let b = reference.ledger.last().unwrap();
        let mut claims: Vec<_> = b
            .discovery_allocation
            .iter()
            .map(|r| (r.claim.id, r.claim.requested, r.claim.minimum))
            .collect();
        claims.sort_unstable();
        assert_eq!(claims.len(), 2);
        if let Some(expected) = &requests {
            assert_eq!(&claims, expected);
        } else {
            requests = Some(claims);
        }
        assert_eq!(b.additional_access.len(), 1);
        let winner = b.additional_access[0].1;
        winners.insert(winner);
        if policy == Policy::StablePriority {
            assert_eq!(b.additional_access[0].1, SUPPLIER.min(GROWER));
        }
        reference.run_months(1).unwrap();
        cpu.run_months(1).unwrap();
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        assert!(
            reference
                .state
                .processes
                .values()
                .any(|p| p.operator == winner
                    && p.definition == GROW
                    && p.status == Status::Completed),
            "policy {policy:?} seed {seed} winner {winner}"
        );
    }
    assert_eq!(winners, [SUPPLIER, GROWER].into());
    // Priority cannot make a seedless applicant feasible.
    let mut world = opening.world.clone();
    world.discovery.as_mut().unwrap().land_allocation = Policy::Lottery;
    let mut state = opening.state.clone();
    state.balances.remove(&(SUPPLIER, SEED));
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.ledger.last().unwrap().additional_access[0].1, GROWER);
}

#[test]
fn land_admission_protects_delivery_claims_after_current_boundary_changes() {
    use economics_compute_smoke::forward::direct::Terms;
    let (w, s) = scenario::scenario().unwrap();
    let mut opening = Simulation::new(w, s, Backend::Reference).unwrap();
    while opening.state.month < 2 || opening.state.phase != Phase::Acquire {
        opening.step().unwrap();
    }
    // issue, due, opening seed, expected delivered now, expected land acceptance
    for (issue, due, seed, delivered, admitted) in [
        (1, 3, 4, 0, false),
        (1, 2, 5, 4, true),
        (2, 3, 4, 0, false),
        (1, 6, 4, 0, true),
    ] {
        let mut w = opening.world.clone();
        w.discovery.as_mut().unwrap().horizon = 4;
        let terms = Terms {
            id: 900,
            seller: GROWER,
            buyer: ISSUER,
            month: issue,
            due,
            goods: Amount::new(SEED, 4),
            prepayment: Amount::new(COIN, 1),
        };
        let mut s = opening.state.clone();
        s.balances.insert((GROWER, SEED), seed);
        s.balances.insert((ISSUER, COIN), 10);
        if issue < s.month {
            s.exchange.forwards.insert(terms.id, terms.contract());
        }
        w.prepaid_deliveries.push(terms);
        let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
        let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        reference.step().unwrap();
        cpu.step().unwrap();
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        assert_eq!(reference.state.exchange.forwards[&900].delivered, delivered);
        assert_eq!(
            reference.ledger.last().unwrap().additional_access.len(),
            usize::from(admitted),
            "issue={issue} due={due}"
        );
        reference.run_months(1).unwrap();
        cpu.run_months(1).unwrap();
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        if admitted {
            assert!(
                reference
                    .state
                    .processes
                    .values()
                    .any(|p| p.operator == GROWER
                        && p.definition == GROW
                        && p.status == Status::Completed)
            );
        } else {
            while reference.state.month < 3 || reference.state.phase != Phase::Productive {
                reference.step().unwrap();
            }
            assert_eq!(reference.state.exchange.forwards[&900].delivered, 4);
        }
    }
}
