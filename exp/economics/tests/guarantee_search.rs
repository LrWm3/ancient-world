use economics_compute_smoke::{
    compute::Backend,
    credit::{Advance, LoanOffer},
    model::*,
    offers::{self, Id, Request},
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    recovery::{
        Guarantee, GuaranteeTender, GuaranteedClaim, RecourseSecurity, admission::Application,
    },
    scenario::*,
    settlement,
    simulation::Simulation,
};
const GUARANTOR: AgentId = 99;

#[test]
fn guaranteed_seed_credit_uses_common_productive_acceptance_and_survives_failed_work() {
    for permitted in [false, true] {
        for funded in [false, true] {
            let (mut w, mut s) = named("opportunity-farming").unwrap();
            w.agents.push(Agent {
                id: GUARANTOR,
                name: "seed guarantor".into(),
            });
            w.lending.push(Advance {
                id: 10,
                debtor: PERSON,
                principal: 1,
                month: 1,
                collateral: None,
                priority: 0,
                terms: LoanOffer {
                    creditor: STATE_AGENT,
                    denomination: SEED,
                    max_principal: 1,
                    monthly_rate_bps: 0,
                    term_months: 6,
                    grace_months: 12,
                },
            });
            let policy = w.transaction_policy.as_mut().unwrap();
            policy.agent_types.insert(GUARANTOR, PERSON_TYPE);
            policy.permissions.insert((PERSON_TYPE, Action::Borrow));
            policy.permissions.insert((STATE_TYPE, Action::Lend));
            if permitted {
                policy.permissions.insert((PERSON_TYPE, Action::Guarantee));
            }
            s.balances.insert((PERSON, SEED), 0);
            s.balances.insert((PERSON, GRAIN), 5);
            s.balances.insert((STATE_AGENT, SEED), i32::from(funded));
            s.balances.insert((GUARANTOR, SEED), 1);
            w.recovery.guarantees.push(Guarantee {
                id: 1,
                follows_assignment: false,
                tender: GuaranteeTender::Native,
                security: RecourseSecurity::Unsecured,
                claim: GuaranteedClaim::Loan(10),
                guarantor: GUARANTOR,
                cap: 1,
                from: 1,
                through: 12,
                delay_months: 0,
                recourse: 11,
                priority: 0,
            });
            w.recovery.posted_guarantees.insert(1);
            w.recovery.guarantee_applications.push(Application {
                guarantee: 1,
                month: 1,
            });
            let run = |backend| {
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                while sim.state.phase != Phase::Acquire {
                    sim.step().unwrap();
                }
                let requests = [
                    Request::new(Id::Advance(10), PERSON),
                    Request::new(Id::Guarantee(1), GUARANTOR),
                    Request::new(Id::Membership(1), PERSON),
                    Request::new(Id::Land(1), PERSON),
                    Request::new(Id::Process(GROW), PERSON),
                ];
                let before = sim.state.clone();
                let prepared = offers::prepare(&sim, &requests);
                assert_eq!(sim.state, before);
                if !permitted || !funded {
                    assert!(prepared.is_err());
                    assert!(offers::accept(&mut sim, &requests).is_err());
                    assert_eq!(sim.state, before);
                    assert!(
                        sim.ledger
                            .iter()
                            .all(|b| b.credit.as_ref().is_none_or(|c| c.after.loans.is_empty()))
                    );
                    return (sim.state, sim.ledger);
                }
                let prepared = prepared.unwrap();
                let mut forged = prepared.clone();
                forged
                    .credit
                    .as_mut()
                    .unwrap()
                    .after
                    .recovery
                    .accepted_guarantees
                    .clear();
                let mut unchanged = before.clone();
                assert!(
                    settlement::commit(
                        &sim.world,
                        &mut unchanged,
                        &forged,
                        backend,
                        sim.effect_limit
                    )
                    .is_err()
                );
                assert_eq!(unchanged, before);
                offers::accept(&mut sim, &requests).unwrap();
                assert_eq!(sim.state.credit.recovery.accepted_guarantees[&1], 1);
                assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
                assert!(sim.state.processes.is_empty());
                sim.step().unwrap(); // Consume seed only at the reserved Productive boundary.
                assert_eq!(sim.state.balance(PERSON, SEED), 0);
                assert!(sim.state.processes.values().any(|p| p.definition == GROW));
                // An observed capacity interruption after acceptance is not a
                // retroactive failure of supplied consent or of the dated plan.
                sim.world
                    .transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(PERSON_TYPE, Action::Guarantee));
                for month in 2..=8 {
                    sim.world.capacity_overrides.insert((month, PERSON), 0);
                }
                let mut resumed =
                    Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
                while sim.state.month < 9 {
                    sim.step().unwrap();
                }
                while resumed.state.month < 9 {
                    resumed.step().unwrap();
                }
                assert_eq!(sim.state, resumed.state);
                assert_eq!(sim.state.credit.loans[&10].principal, 0);
                assert_eq!(sim.state.credit.loans[&11].principal, 1);
                assert_eq!(sim.state.credit.loans[&11].denomination, SEED);
                assert_eq!(sim.state.credit.loans[&11].creditor, GUARANTOR);
                assert_eq!(sim.state.credit.loans[&11].debtor, PERSON);
                assert_eq!(sim.state.balance(GUARANTOR, SEED), 0);
                assert_eq!(sim.state.balance(STATE_AGENT, SEED), 1);
                assert!(
                    sim.state
                        .processes
                        .values()
                        .any(|p| p.definition == GROW && p.status != Status::Completed)
                );
                (sim.state, sim.ledger)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}
