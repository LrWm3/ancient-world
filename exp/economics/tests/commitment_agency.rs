use economics_compute_smoke::{
    agency::{self, integration as economy},
    compute::Backend,
    financial_reporting::Audit,
    forward, household_governance as h, households,
    minting::*,
    model::*,
    settlement::DEFAULT_EFFECT_LIMIT,
    simulation::Simulation,
    telemetry::{Config, Observer},
};

fn through(sim: &mut Simulation, audit: &mut Audit, month: u32) {
    while sim.state.month <= month {
        audit.step(sim).unwrap();
    }
}
fn run(w: World, s: State, months: u32) -> (Simulation, Audit) {
    let mut audit = economy::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut sim, &mut audit, months);
    (sim, audit)
}
fn deliveries(sim: &Simulation) -> Vec<(u32, i32)> {
    sim.ledger
        .iter()
        .flat_map(|b| {
            b.transactions.iter().filter_map(move |t| match t.forward {
                Some(forward::Event::Delivery {
                    contract: 20,
                    quantity,
                }) => Some((b.month, quantity)),
                _ => None,
            })
        })
        .collect()
}
fn deficits(sim: &Simulation) -> i32 {
    sim.reports.iter().map(|r| r.deficit(NUTRITION)).sum()
}
fn policy(w: &mut World, p: h::Policy) {
    w.households[0]
        .governance
        .constitution
        .permitted_policies
        .insert(p);
    w.agency
        .get_mut(&economy::HOUSEHOLD)
        .unwrap()
        .config
        .programs
        .get_mut(&1)
        .unwrap()
        .commands = vec![agency::Command::HouseholdPolicy(p)];
}

#[test]
fn commitment_preparation_delivers_on_time_with_cpu_replay_and_separate_books() {
    let (w, s) = economy::commitment_scenario().unwrap();
    let (reference, a) = run(w.clone(), s.clone(), economy::RUN_MONTHS);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut audit = economy::audit(&sim.world, &sim.state).unwrap();
    let mut observer = Observer::new(
        vec![],
        "commitments",
        Config {
            settlement: true,
            ..Default::default()
        },
    )
    .unwrap();
    while sim.state.month <= economy::RUN_MONTHS {
        observer.step_audited(&mut sim, &mut audit).unwrap();
        let mut restarted =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
        restarted.ledger = sim.ledger;
        restarted.reports = sim.reports;
        sim = restarted;
    }
    assert_eq!(sim.world, reference.world);
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    assert_eq!(sim.reports, reference.reports);
    assert_eq!(audit, a);
    assert_eq!(deliveries(&sim), [(4, 2)]);
    assert_eq!(deficits(&sim), 2);
    assert_eq!(sim.state.credit.loans[&10].principal, 0);
    assert_eq!(sim.state.obligations[&(77, 13)].outstanding(), 0);
    assert_eq!(sim.state.balance(ISSUER, COIN), 18);
    for agent in &sim.world.agents {
        let statement = audit
            .book()
            .statements(agent.id, 1, economy::RUN_MONTHS)
            .unwrap();
        assert_eq!(statement.assets, statement.liabilities + statement.equity);
        assert_eq!(
            statement.closing_cash,
            i128::from(sim.state.balance(agent.id, COIN))
        );
    }
    assert_eq!(
        audit
            .book()
            .statements(ISSUER, 1, economy::RUN_MONTHS)
            .unwrap()
            .issuance_change,
        40
    );
    // A real decision for accepted delivery cover when current needs are equal.
    assert!(
        sim.ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .any(|d| {
                d.recipient == Some(economy::GROWER)
                    && d.baseline_needs == d.projected_needs
                    && d.baseline_funding
                        .as_ref()
                        .zip(d.projected_funding.as_ref())
                        .is_some_and(|(base, next)| {
                            base.gaps
                                .iter()
                                .zip(&next.gaps)
                                .any(|(b, n)| b.resource == WHEAT && n.shortfall < b.shortfall)
                        })
            })
    );
    let logs = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert!(logs.lines().any(|line| line.contains("household_labor")
        && line.contains("\"months\":4")
        && line.contains("projected_funding")));
    assert!(
        logs.lines()
            .any(|line| line.contains("accepted prospective voluntary payment funding"))
    );
}

#[test]
fn policy_and_consent_are_separate_and_the_old_control_stays_late() {
    let (w, s) = economy::commitment_scenario().unwrap();
    let (normal, _) = run(w.clone(), s.clone(), 6);
    assert_eq!(deliveries(&normal), [(4, 2)]);
    let mut no_consent = w.clone();
    no_consent.households[0].support.clear();
    let (no_consent, _) = run(no_consent, s.clone(), 6);
    assert!(
        no_consent.state.exchange.forwards[&20].delivered < 2
            || deliveries(&no_consent).iter().any(|(m, _)| *m > 4)
    );
    assert!(
        no_consent
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .all(|h| h.support.is_empty())
    );
    let mut disabled = w;
    disabled.households[0]
        .governance
        .charter
        .accept_payment_support = false;
    let (disabled, _) = run(disabled, s, 6);
    assert_eq!(deliveries(&disabled), deliveries(&no_consent));
    let (w, s) = economy::scenario().unwrap();
    let (baseline, _) = run(w, s, economy::RUN_MONTHS);
    assert_eq!(deliveries(&baseline), [(10, 1), (11, 1)]);
    assert_eq!(deficits(&normal), deficits(&baseline));
}

#[test]
fn horizon_controls_advance_preparation_and_unsigned_offers_create_no_demand() {
    let (mut w, s) = economy::commitment_scenario().unwrap();
    policy(&mut w, h::Policy::NeedsThenCommitments { months: 1 });
    let (short, _) = run(w.clone(), s.clone(), 4);
    assert!(short.state.exchange.forwards[&20].claim().outstanding() > 0);
    w.prepaid_deliveries.clear();
    let (unsigned, _) = run(w, s, 4);
    assert!(unsigned.state.exchange.forwards.is_empty());
    assert!(
        unsigned
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .filter_map(|d| d.projected_funding.as_ref())
            .flat_map(|f| &f.gaps)
            .all(|g| g.resource != WHEAT || g.required == 0)
    );
}

#[test]
fn seed_shortage_preserves_claim_and_does_not_fabricate_production_or_support() {
    let (mut w, mut s) = economy::commitment_scenario().unwrap();
    w.households[0].governance.charter.initial_policy =
        h::Policy::NeedsThenCommitments { months: 4 };
    s.balances.insert((economy::GROWER, economy::SEED), 0);
    let (sim, _) = run(w, s, 6);
    assert_eq!(sim.state.exchange.forwards[&20].claim().outstanding(), 2);
    assert!(deliveries(&sim).is_empty());
    assert!(
        !sim.state
            .processes
            .values()
            .any(|p| p.definition == economy::GROW && p.status == Status::Completed)
    );
    assert!(
        sim.ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .filter_map(|d| d.projected_funding.as_ref())
            .flat_map(|f| &f.gaps)
            .any(|g| g.resource == WHEAT && g.shortfall == 2)
    );
    assert!(deficits(&sim) > 2);
    assert!(sim.state.balances.values().all(|q| *q >= 0));
}

#[test]
fn support_precedes_due_collection_runs_once_and_protects_member_food() {
    let (w, s) = economy::commitment_scenario().unwrap();
    let mut audit = economy::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while (sim.state.month, sim.state.phase) != (4, Phase::Acquire) {
        audit.step(&mut sim).unwrap();
    }
    let opening = sim.clone();
    audit.step(&mut sim).unwrap();
    let b = sim.ledger.last().unwrap();
    let support = &b.household.as_ref().unwrap().support;
    let donor = support.iter().find(|r| r.accepted > 0).unwrap();
    assert_eq!(donor.mandate.member, economy::GROWER);
    assert_eq!(donor.accepted, 1);
    assert_eq!(donor.protected, 1);
    assert_eq!(sim.state.balance(economy::GROWER, WHEAT), 1);
    assert_eq!(sim.state.exchange.forwards[&20].delivered, 2);
    // Replay rejects fabricated support evidence before touching either balance sheet.
    let mut forged = b.clone();
    forged
        .household
        .as_mut()
        .unwrap()
        .support
        .iter_mut()
        .find(|r| r.accepted > 0)
        .unwrap()
        .accepted += 1;
    let mut untouched = opening.state.clone();
    assert!(
        households::commit(
            &opening.world,
            &mut untouched,
            &forged,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(untouched, opening.state);
    through(&mut sim, &mut audit, 6);
    let mut totals = std::collections::BTreeMap::new();
    for b in &sim.ledger {
        for r in b.household.iter().flat_map(|h| &h.support) {
            if b.month > 1 {
                assert_eq!(b.phase, Phase::Acquire);
            }
            *totals
                .entry((b.month, r.mandate.member, r.mandate.resource))
                .or_insert(0) += r.accepted;
            assert!(
                totals[&(b.month, r.mandate.member, r.mandate.resource)] <= r.mandate.monthly_limit
            );
        }
    }
    assert_eq!(deficits(&sim), 2);
}

#[test]
fn withdrawal_of_surplus_consent_is_honored_before_delivery() {
    let (w, s) = economy::commitment_scenario().unwrap();
    let mut audit = economy::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut sim, &mut audit, 2);
    households::support::revoke(
        &mut sim.world,
        &sim.state,
        economy::HOUSEHOLD,
        economy::GROWER,
        WHEAT,
        1,
    )
    .unwrap();
    through(&mut sim, &mut audit, 4);
    assert_eq!(sim.state.exchange.forwards[&20].delivered, 1);
    assert!(
        sim.ledger
            .iter()
            .filter(|b| b.month >= 4)
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.support)
            .all(|r| r.mandate.member != economy::GROWER)
    );
}

#[test]
fn preparation_requires_valid_terms_and_cannot_expand_the_work_mandate() {
    for months in [0, 25] {
        let (mut w, s) = economy::commitment_scenario().unwrap();
        policy(&mut w, h::Policy::NeedsThenCommitments { months });
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
    let (mut w, s) = economy::commitment_scenario().unwrap();
    w.households[0].governance.constitution.activities = Some([GATHER].into());
    w.households[0].governance.charter.initial_policy =
        h::Policy::NeedsThenCommitments { months: 4 };
    let (sim, _) = run(w, s, 4);
    assert!(
        !sim.state
            .processes
            .values()
            .any(|p| p.definition == economy::GROW && p.status == Status::Completed)
    );
    assert!(
        sim.ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .all(|d| d.recipient != Some(economy::GROWER))
    );
}

#[test]
fn funding_forecasts_are_validated_as_part_of_the_committed_boundary() {
    let (w, s) = economy::commitment_scenario().unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while (sim.state.month, sim.state.phase) != (3, Phase::Productive) {
        sim.step().unwrap();
    }
    let opening = sim.clone();
    sim.step().unwrap();
    let mut forged = sim.ledger.last().unwrap().clone();
    forged.household.as_mut().unwrap().labor[0]
        .projected_funding
        .as_mut()
        .unwrap()
        .gaps
        .iter_mut()
        .find(|g| g.resource == WHEAT)
        .unwrap()
        .shortfall += 1;
    let mut untouched = opening.state.clone();
    assert!(
        households::commit(
            &opening.world,
            &mut untouched,
            &forged,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(untouched, opening.state);
}

#[test]
fn catalog_order_does_not_replace_explicit_household_priorities() {
    let (w, s) = economy::commitment_scenario().unwrap();
    let mut reversed = w.clone();
    reversed.agents.reverse();
    reversed.participants.reverse();
    reversed.definitions.reverse();
    reversed.resources.reverse();
    let (normal, audit) = run(w, s.clone(), 6);
    let (permuted, other) = run(reversed, s, 6);
    assert_eq!(normal.state, permuted.state);
    assert_eq!(normal.ledger, permuted.ledger);
    assert_eq!(normal.reports, permuted.reports);
    assert_eq!(audit, other);
}

#[test]
fn an_unfilled_member_need_blocks_early_payment_support() {
    let (w, s) = economy::commitment_scenario().unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while (sim.state.month, sim.state.phase) != (4, Phase::Acquire) {
        sim.step().unwrap();
    }
    // Controlled stock-loss boundary: the grower has surplus, but the other
    // member's opening food reservation cannot be filled. No future crop is cover.
    sim.state.balances.insert((economy::HOUSEHOLD, WHEAT), 0);
    sim.state.balances.insert((SUPPLIER, WHEAT), 0);
    sim.step().unwrap();
    assert_eq!(sim.state.exchange.forwards[&20].delivered, 0);
    let receipt = sim
        .ledger
        .last()
        .unwrap()
        .household
        .as_ref()
        .unwrap()
        .support
        .iter()
        .find(|r| r.mandate.member == economy::GROWER)
        .unwrap();
    assert_eq!(receipt.offered, 1);
    assert_eq!(receipt.accepted, 0);
    assert_eq!(receipt.reason, "current member needs lack stock cover");
    assert_eq!(sim.state.balance(economy::GROWER, WHEAT), 2);
}
