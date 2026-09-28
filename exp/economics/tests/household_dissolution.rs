use economics_compute_smoke::{
    accounting::{Account as FinancialAccount, ReportingScope},
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::Audit,
    household_governance::{self as governance, Governance},
    households::{self, Agreement, dissolution as d, membership as m},
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
    telemetry::{Config, Observer},
};
const HOME: AgentId = 10000;

fn fixture() -> (World, State) {
    let (mut w, mut s) = baseline();
    w.assets.clear();
    w.rights.clear();
    w.definitions.clear();
    w.participants[0].needs.clear();
    s.balances.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    let mut g = Governance::contributed(PERSON);
    g.constitution.allow_dissolution = true;
    households::form(
        &mut w,
        &s,
        Agreement {
            id: 1,
            agent: HOME,
            adults: vec![PERSON],
            governance: g,
            formed: 1,
            dwelling_process: None,
            admission: None,
            membership: vec![],
        },
    )
    .unwrap();
    s.month = 2;
    s.balances.insert((HOME, GRAIN), 2);
    s.balances.insert((HOME, TOKEN), 5);
    s.balances.insert((STATE_AGENT, TOKEN), 10);
    (w, s)
}
fn run_through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}

#[test]
fn dissolution_requires_founding_permission_last_member_and_month_boundaries() {
    let (mut w, mut s) = fixture();
    w.households[0].governance.constitution.allow_dissolution = false;
    let before = w.clone();
    assert!(d::request(&mut w, &s, HOME, PERSON).is_err());
    assert_eq!(w, before);
    w.households[0].governance.constitution.allow_dissolution = true;
    s.phase = Phase::Productive;
    assert!(d::request(&mut w, &s, HOME, PERSON).is_err());
    s.phase = Phase::Open;
    assert!(d::request(&mut w, &s, HOME, STATE_AGENT).is_err());
    d::request(&mut w, &s, HOME, PERSON).unwrap();
    assert!(d::finish(&mut w, &s, HOME, PERSON).is_err());
    assert!(m::leave(&mut w, &s, HOME, PERSON).is_err());
    assert_eq!(governance::leader(&w.households[0], &s), None);
    assert_eq!(households::members(&w.households[0], &s).count(), 0);
    assert_eq!(m::current(&w.households[0]), vec![PERSON]); // affiliation/storage held through wind-down
    assert!(m::join(&mut w, &s, HOME, PERSON, vec![PERSON]).is_err());
    s.month = 3;
    assert!(d::finish(&mut w, &s, HOME, PERSON).is_err()); // property not yet settled
}

#[test]
fn residual_stock_settles_once_with_replay_evidence_then_releases_membership() {
    let (mut w, s) = fixture();
    let founding = w.households[0].admission.clone();
    d::request(&mut w, &s, HOME, PERSON).unwrap();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.balance(HOME, GRAIN), 0);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 2);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 5);
    let batch = sim.ledger[0].clone();
    assert_eq!(
        batch.household.as_ref().unwrap().dissolution[0].distributed,
        [(GRAIN, 2), (TOKEN, 5)].into()
    );
    let mut replay = s.clone();
    commit(
        &w,
        &mut replay,
        &batch,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    assert_eq!(replay, sim.state);
    let mut forged = batch.clone();
    forged.household.as_mut().unwrap().dissolution[0]
        .distributed
        .clear();
    let mut unchanged = s.clone();
    assert!(
        commit(
            &w,
            &mut unchanged,
            &forged,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(unchanged, s);
    sim.run_months(1).unwrap();
    let s_before = sim.state.clone();
    d::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
    assert_eq!(sim.state, s_before);
    assert!(d::closed_at(&sim.world.households[0], 3));
    assert!(m::current(&sim.world.households[0]).is_empty());
    assert_eq!(sim.world.households[0].admission, founding);
    assert_eq!(m::roster_at(&sim.world.households[0], 1), vec![PERSON]);
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 2);
    assert!(m::join(&mut sim.world, &sim.state, HOME, PERSON, vec![PERSON]).is_err());
}

#[test]
fn static_named_recipient_and_storage_capacity_bound_distribution() {
    let (mut w, s) = fixture();
    w.households[0].governance.charter.residual_recipient = Some(STATE_AGENT);
    w.storage.weights.insert(GRAIN, 1);
    w.storage.capacities.insert(PERSON, 4);
    w.storage.capacities.insert(STATE_AGENT, 1);
    d::request(&mut w, &s, HOME, PERSON).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    assert_eq!(
        sim.ledger[0].household.as_ref().unwrap().dissolution[0].blockers,
        vec![d::Blocker::Storage]
    );
    assert_eq!(sim.state.balance(HOME, TOKEN), 5); // no partial release of unweighted money
    sim.run_months(1).unwrap();
    sim.world.storage.capacities.insert(STATE_AGENT, 2);
    sim.step().unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 2);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 15);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
}

#[test]
fn owned_assets_and_future_contracts_block_distribution() {
    for owned in [true, false] {
        let (mut w, s) = fixture();
        if owned {
            w.assets.push(Asset {
                id: PLOT,
                owner: HOME,
                kind: 1,
            });
        } else {
            w.lending.push(Advance {
                id: 77,
                debtor: HOME,
                terms: LoanOffer {
                    creditor: STATE_AGENT,
                    denomination: TOKEN,
                    max_principal: 2,
                    monthly_rate_bps: 0,
                    term_months: 1,
                    grace_months: 1,
                },
                principal: 2,
                month: 4,
                collateral: None,
                priority: 0,
            });
        }
        if !owned {
            assert!(d::blockers(&w, &s, HOME).contains(&d::Blocker::Loan));
            let before = w.clone();
            assert!(d::request(&mut w, &s, HOME, PERSON).is_err()); // existing unsupported driver remains rejected
            assert_eq!(w, before);
            continue;
        }
        d::request(&mut w, &s, HOME, PERSON).unwrap();
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.step().unwrap();
        let receipt = &sim.ledger[0].household.as_ref().unwrap().dissolution[0];
        assert!(receipt.blockers.contains(&if owned {
            d::Blocker::Asset
        } else {
            d::Blocker::Loan
        }));
        assert!(receipt.distributed.is_empty());
        assert_eq!(sim.state.balance(HOME, GRAIN), 2);
    }
}

#[test]
fn dues_clear_before_surplus_distribution_and_separate_books_reconcile_on_cpu() {
    let (mut w, mut s) = fixture();
    s.month = 13;
    w.assets.push(Asset {
        id: PLOT,
        owner: STATE_AGENT,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: 1,
        holder: HOME,
        output_owner: HOME,
        asset: PLOT,
        from: 1,
        through: 13,
    });
    w.agreements
        .push(economics_compute_smoke::commitments::Agreement {
            id: 1,
            right: 1,
            debtor: HOME,
            creditor: STATE_AGENT,
            activated: 1,
            payment: Amount::new(GRAIN, 1),
        });
    d::request(&mut w, &s, HOME, PERSON).unwrap();
    let audit = Audit::with_dues(
        &w,
        &s,
        TOKEN,
        [(PLOT, 0)].into(),
        [((HOME, GRAIN), 6)].into(),
        [(1, 3)].into(),
    )
    .unwrap();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut reference_audit = audit.clone();
    let mut cpu_audit = audit;
    for (sim, audit) in [
        (&mut reference, &mut reference_audit),
        (&mut cpu, &mut cpu_audit),
    ] {
        run_through(audit, sim, 13);
        assert_eq!(sim.state.obligations[&(1, 13)].paid, 1);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
        assert_eq!(sim.state.balance(HOME, TOKEN), 5);
    }
    let mut resumed = cpu.clone();
    let mut resumed_audit = cpu_audit.clone();
    for (sim, audit) in [
        (&mut reference, &mut reference_audit),
        (&mut cpu, &mut cpu_audit),
        (&mut resumed, &mut resumed_audit),
    ] {
        run_through(audit, sim, 14);
        d::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
        run_through(audit, sim, 15);
        audit.finalize_through(15).unwrap();
        for id in [HOME, PERSON, STATE_AGENT] {
            let report = audit.book().finalized_statements(id, 13, 15).unwrap();
            assert_eq!(report.scope, ReportingScope::Separate { agent: id });
            assert_eq!(report.assets, report.liabilities + report.equity);
            if id == HOME {
                assert_eq!(report.assets, 0);
                assert_eq!(report.liabilities, 0);
                assert_eq!(report.expenses[&FinancialAccount::TransferExpense], 8);
            }
        }
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu_audit, reference_audit);
    assert_eq!(cpu.state, resumed.state);
    assert_eq!(cpu_audit, resumed_audit);
}

#[test]
fn observer_reports_blocked_and_paid_residuals_without_changing_execution() {
    let (mut w, s) = fixture();
    d::request(&mut w, &s, HOME, PERSON).unwrap();
    let mut observed = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut plain = observed.clone();
    let mut observer = Observer::new(
        vec![],
        "dissolution",
        Config {
            settlement: true,
            agents: [PERSON].into(),
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut observed, 2).unwrap();
    plain.run_months(2).unwrap();
    assert_eq!(observed.state, plain.state);
    assert_eq!(observed.ledger, plain.ledger);
    let log = String::from_utf8(observer.finish().unwrap()).unwrap();
    let rows: Vec<serde_json::Value> = log
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|r: &serde_json::Value| r["kind"] == "household_dissolution")
        .collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0]["distributed"],
        serde_json::json!({GRAIN.to_string():2, TOKEN.to_string():5})
    );
    assert_eq!(rows[1]["distributed"], serde_json::json!({}));
}

#[test]
fn law_can_reject_dissolution_templates_without_retroactively_amending_admitted_terms() {
    use economics_compute_smoke::{laws, opportunities as o};
    let (mut w, mut s) = fixture();
    let mut a = w.households.remove(0);
    w.agents.retain(|x| x.id != HOME);
    s.balances.retain(|(id, _), _| *id != HOME);
    s.month = 1;
    a.admission = None;
    w.transaction_policy = Some(o::Policy {
        authority: STATE_AGENT,
        laws: vec![],
        agreement_forms: Some([laws::AgreementForm::Household].into()),
        agreement_limits: laws::AgreementLimits {
            household: Some(laws::households::Rules {
                allow_dissolution: false,
                ..Default::default()
            }),
            ..Default::default()
        },
        membership_offers: vec![],
        membership_permissions: Default::default(),
        agent_types: [(PERSON, o::PERSON_TYPE)].into(),
        permissions: [(o::PERSON_TYPE, o::Action::FoundHousehold)].into(),
    });
    let before = w.clone();
    assert!(households::form(&mut w, &s, a.clone()).is_err());
    assert_eq!(w, before);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .agreement_limits
        .household
        .as_mut()
        .unwrap()
        .allow_dissolution = true;
    households::form(&mut w, &s, a).unwrap();
    w.transaction_policy
        .as_mut()
        .unwrap()
        .agreement_limits
        .household
        .as_mut()
        .unwrap()
        .allow_dissolution = false;
    s.month = 2;
    d::request(&mut w, &s, HOME, PERSON).unwrap(); // accepted founding terms survive subsequent law changes
}

#[test]
fn winding_household_cannot_supply_labor_vote_or_admit_another_member() {
    use governance::elections::{Ballot, cast};
    let (mut w, mut s) = fixture();
    w.households[0].governance.constitution.leadership = governance::Leadership::Elected;
    w.households[0].governance.charter.term_months = 1;
    d::request(&mut w, &s, HOME, PERSON).unwrap();
    assert!(
        cast(
            &mut w,
            &s,
            HOME,
            Ballot {
                voter: PERSON,
                candidate: Some(PERSON),
                term_start: 3
            }
        )
        .is_err()
    );
    s.month = 3;
    assert!(m::leave(&mut w, &s, HOME, PERSON).is_err());
    assert!(
        governance::schedule(
            &mut w,
            &s,
            HOME,
            governance::PolicyChange {
                month: 4,
                authorized_by: PERSON,
                policy: governance::Policy::NetOutput
            }
        )
        .is_err()
    );
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert!(
        sim.ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .all(|h| h
                .labor
                .iter()
                .all(|d| d.granted == 0 && d.contributions.is_empty()))
    );
}

#[test]
fn closure_does_not_orphan_a_remaining_asset_or_revive_a_dissolved_agent() {
    let (mut w, s) = fixture();
    d::request(&mut w, &s, HOME, PERSON).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.world.assets.push(Asset {
        id: PLOT,
        owner: HOME,
        kind: 1,
    });
    let before = sim.world.clone();
    assert!(d::finish(&mut sim.world, &sim.state, HOME, PERSON).is_err());
    assert_eq!(sim.world, before);
    sim.world.assets.clear();
    d::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
    // An external attempted credit to the closed entity must not silently revive it.
    sim.state.balances.insert((HOME, TOKEN), 1);
    assert!(households::validate(&sim.world, &sim.state).is_err());
}

#[test]
fn winding_recipient_cannot_recycle_another_households_residual_distribution() {
    let (mut w, mut s) = fixture();
    w.agents.push(Agent {
        id: PERSON + 1,
        name: "other member".into(),
    });
    let mut p = w.participants[0].clone();
    p.agent = PERSON + 1;
    w.participants.push(p);
    let mut a = w.households[0].clone();
    a.id = 2;
    a.agent = HOME + 1;
    a.adults = vec![PERSON + 1];
    a.governance = Governance::contributed(PERSON + 1);
    a.governance.constitution.allow_dissolution = true;
    a.admission = None;
    s.month = 1;
    households::form(&mut w, &s, a).unwrap();
    w.households[0].governance.charter.residual_recipient = Some(HOME + 1);
    w.households[1].governance.charter.residual_recipient = Some(HOME);
    s.month = 2;
    d::request(&mut w, &s, HOME, PERSON).unwrap();
    d::request(&mut w, &s, HOME + 1, PERSON + 1).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.balance(HOME, TOKEN), 5);
    assert!(
        sim.ledger[0]
            .household
            .as_ref()
            .unwrap()
            .dissolution
            .iter()
            .all(|r| r.blockers.contains(&d::Blocker::RecipientUnavailable)
                && r.distributed.is_empty())
    );
}
