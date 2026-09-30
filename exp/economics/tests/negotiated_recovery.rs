use economics_compute_smoke::{
    compute::Backend,
    credit::{Advance, LoanOffer},
    employment::{ArrearsPolicy, Terms},
    financial_reporting::{Audit, Opening},
    model::*,
    negotiation::{self, Outcome},
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    recovery::{ProceedingTerms, Stage},
    scenario::{GRAIN, LABOR, PERSON, STATE_AGENT, TOKEN},
    simulation::Simulation,
};

const SELLER: AgentId = 89;
const ESTATE: AgentId = 20000;
const WORKER: AgentId = 20001;
const CUSTOMER: AgentId = 20002;

#[test]
fn negotiated_markets_respect_buyer_and_seller_stays_and_reopen_after_closure() {
    for debtor in [PERSON, SELLER] {
        for trade_month in [3, 5] {
            let (mut w, mut s) = negotiation::scenario();
            w.negotiation.as_mut().unwrap().month = trade_month;
            w.resources.push(Resource {
                id: LABOR,
                name: "labor hours".into(),
                kind: ResourceKind::Capacity,
            });
            for id in [ESTATE, WORKER, CUSTOMER] {
                w.agents.push(Agent {
                    id,
                    name: format!("counterparty {id}"),
                });
            }
            for id in [debtor, WORKER] {
                w.participants.push(Participant {
                    agent: id,
                    capacity: Amount::new(LABOR, 1),
                    needs: vec![],
                });
            }
            s.balances.insert((debtor, TOKEN), 0);
            s.balances.insert((STATE_AGENT, TOKEN), 4);
            s.balances.insert((CUSTOMER, TOKEN), 100);
            let law = w.transaction_policy.as_mut().unwrap();
            law.agent_types
                .extend([(WORKER, PERSON_TYPE), (CUSTOMER, PERSON_TYPE)]);
            law.permissions.extend([
                (PERSON_TYPE, Action::Borrow),
                (STATE_TYPE, Action::Lend),
                (PERSON_TYPE, Action::CapacityTrade),
            ]);
            w.lending.push(Advance {
                id: 1,
                debtor,
                principal: 4,
                month: 1,
                collateral: None,
                priority: 0,
                terms: LoanOffer {
                    creditor: STATE_AGENT,
                    denomination: TOKEN,
                    max_principal: 4,
                    monthly_rate_bps: 0,
                    term_months: 1,
                    grace_months: 12,
                },
            });
            // Spend borrowed coins on work, miss the next installment, then earn
            // saleable-service income. Prior arrears authorize the dated case.
            for (id, employer, worker, month, wage) in
                [(1, debtor, WORKER, 1, 4), (2, CUSTOMER, debtor, 2, 100)]
            {
                w.employment.push(Terms {
                    id,
                    employer,
                    worker,
                    from: month,
                    through: month,
                    capacity: Amount::new(LABOR, 1),
                    wage_per_unit: Amount::new(TOKEN, wage),
                    on_arrears: ArrearsPolicy::Continue,
                    rank: 0,
                });
            }
            w.recovery.proceedings.push(ProceedingTerms {
                id: 1,
                debtor,
                authority: STATE_AGENT,
                estate: ESTATE,
                denomination: TOKEN,
                opening_month: 3,
                earliest_close: 5,
                assets: vec![],
                discharge_deficiency: true,
            });
            let mut invalid = w.clone();
            invalid.negotiation.as_mut().unwrap().buyer.agent = ESTATE;
            assert!(Simulation::new(invalid, s.clone(), Backend::Reference).is_err());
            let run = |backend| {
                let mut audit = Audit::with_opening(
                    &w,
                    &s,
                    TOKEN,
                    Opening {
                        inventory: [((SELLER, GRAIN), 2)].into(),
                        services: Some(Default::default()),
                        ..Opening::default()
                    },
                )
                .unwrap();
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                while sim.state.month <= 3 {
                    audit.step(&mut sim).unwrap();
                }
                assert_eq!(
                    sim.state.credit.recovery.proceedings[&1].stage,
                    Stage::Active
                );
                assert!(!economics_compute_smoke::marketplace::eligible(
                    &sim.world,
                    &sim.state,
                    negotiation::MARKETPLACE,
                    debtor
                ));
                let unaffected = if debtor == PERSON { SELLER } else { PERSON };
                assert!(economics_compute_smoke::marketplace::eligible(
                    &sim.world,
                    &sim.state,
                    negotiation::MARKETPLACE,
                    unaffected
                ));
                let (mut resumed, mut ra) = (sim.clone(), audit.clone());
                while sim.state.month <= 5 {
                    audit.step(&mut sim).unwrap();
                }
                while resumed.state.month <= 5 {
                    ra.step(&mut resumed).unwrap();
                }
                assert_eq!(
                    (&sim.state, &sim.ledger, &audit),
                    (&resumed.state, &resumed.ledger, &ra)
                );
                let round = sim
                    .ledger
                    .iter()
                    .find_map(|b| b.negotiation.as_ref())
                    .unwrap();
                if trade_month == 3 {
                    assert_eq!(round.outcome, Outcome::Ineligible);
                    assert!(round.quotes.is_empty());
                } else {
                    assert!(matches!(round.outcome, Outcome::Traded { .. }));
                }
                assert_eq!(
                    sim.state.credit.recovery.proceedings[&1].stage,
                    Stage::Closed
                );
                assert_eq!(sim.state.credit.loans[&1].principal, 0);
                assert!(economics_compute_smoke::marketplace::eligible(
                    &sim.world,
                    &sim.state,
                    negotiation::MARKETPLACE,
                    debtor
                ));
                assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
                for a in &sim.world.agents {
                    let report = audit.book().statements(a.id, 1, 5).unwrap();
                    assert_eq!(report.assets, report.liabilities + report.equity);
                }
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}
