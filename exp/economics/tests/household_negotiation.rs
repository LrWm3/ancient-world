use economics_compute_smoke::{
    compute::Backend,
    financial_reporting::{Audit, Opening},
    household_governance::{Governance, Policy, Purchasing},
    households::{self, Agreement},
    model::*,
    need_orders,
    negotiation::Outcome,
    scenario::{GRAIN, PERSON, TOKEN},
    simulation::Simulation,
};
const HOME: AgentId = 10000;
const MEMBER: AgentId = 91;
const SELLER: AgentId = 89;
const ORE: ResourceId = 777;

#[test]
fn negotiated_member_purchases_pool_once_and_reserve_contributed_storage() {
    for (full, loan) in [(false, false), (false, true), (true, true)] {
        let (mut w, mut s) = need_orders::scenario();
        let mut member = w.participants[0].clone();
        member.agent = MEMBER;
        member.needs.clear();
        w.participants.push(member);
        w.agents.push(Agent {
            id: MEMBER,
            name: "second household member".into(),
        });
        // Two private and two contributed grain slots. A full household has no
        // room for the mandatory half-share even though the buyer has space.
        w.storage.capacities.insert(PERSON, 4);
        w.storage.capacities.insert(MEMBER, 0);
        let law = w.transaction_policy.as_mut().unwrap();
        law.agent_types
            .insert(MEMBER, economics_compute_smoke::opportunities::PERSON_TYPE);
        law.permissions.insert((
            economics_compute_smoke::opportunities::PERSON_TYPE,
            economics_compute_smoke::opportunities::Action::FoundHousehold,
        ));
        if loan {
            use economics_compute_smoke::{
                credit::{Advance, LoanOffer},
                opportunities::{Action, PERSON_TYPE, STATE_TYPE},
                scenario::STATE_AGENT,
            };
            law.permissions
                .extend([(PERSON_TYPE, Action::Borrow), (STATE_TYPE, Action::Lend)]);
            s.balances.insert((STATE_AGENT, TOKEN), 10);
            w.lending.push(Advance {
                id: 1,
                debtor: PERSON,
                principal: 10,
                month: 1,
                collateral: None,
                priority: 0,
                terms: LoanOffer {
                    creditor: STATE_AGENT,
                    denomination: TOKEN,
                    max_principal: 10,
                    monthly_rate_bps: 0,
                    term_months: 2,
                    grace_months: 12,
                },
            });
        }
        let mut governance = Governance::contributed(PERSON);
        governance.charter.initial_policy = Policy::NeedsFirst;
        governance.charter.purchasing = Purchasing::Members;
        households::form(
            &mut w,
            &s,
            Agreement {
                id: 1,
                agent: HOME,
                adults: vec![PERSON, MEMBER],
                governance,
                formed: s.month,
                dwelling_process: None,
                admission: None,
                membership: vec![],
                asset_sales: vec![],
                equipment_retirements: vec![],
                support: vec![],
            },
        )
        .unwrap();
        w.resources.push(Resource {
            id: ORE,
            name: "stored ore".into(),
            kind: ResourceKind::Stock,
        });
        w.storage.weights.insert(ORE, 1);
        s.balances.insert((HOME, ORE), if full { 2 } else { 0 });
        if !full {
            let mut collective = w.clone();
            collective.households[0].governance.charter.purchasing = Purchasing::Collective;
            let mut blocked =
                Simulation::new(collective.clone(), s.clone(), Backend::Reference).unwrap();
            while blocked.state.phase != Phase::Productive {
                blocked.step().unwrap();
            }
            assert_eq!(
                blocked
                    .ledger
                    .last()
                    .unwrap()
                    .negotiation
                    .as_ref()
                    .unwrap()
                    .outcome,
                Outcome::PurchasePolicy
            );
            assert_eq!(blocked.state.balance(SELLER, TOKEN), 0);
            collective.negotiation.as_mut().unwrap().buyer.agent = HOME;
            collective.marketplaces[0]
                .allowed_types
                .insert(economics_compute_smoke::opportunities::HOUSEHOLD_TYPE);
            collective
                .transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .insert((
                    economics_compute_smoke::opportunities::HOUSEHOLD_TYPE,
                    economics_compute_smoke::opportunities::Action::StockTrade,
                ));
            let mut opening = s.clone();
            opening.balances.insert((PERSON, TOKEN), 0);
            opening.balances.insert((HOME, TOKEN), 200);
            let mut collective_audit = Audit::with_opening(
                &collective,
                &opening,
                TOKEN,
                Opening {
                    inventory: [((SELLER, GRAIN), 10)].into(),
                    processes: Some(Default::default()),
                    ..Opening::default()
                },
            )
            .unwrap();
            let mut collective_sim =
                Simulation::new(collective, opening, Backend::CubeCpu).unwrap();
            while collective_sim.state.phase != Phase::Productive {
                collective_audit.step(&mut collective_sim).unwrap();
            }
            assert!(matches!(
                collective_sim
                    .ledger
                    .last()
                    .unwrap()
                    .negotiation
                    .as_ref()
                    .unwrap()
                    .outcome,
                Outcome::Traded { price: 40 }
            ));
            assert_eq!(collective_sim.state.balance(HOME, GRAIN), 2);
            assert_eq!(collective_sim.state.balance(PERSON, GRAIN), 0);
            assert_eq!(collective_sim.state.balance(HOME, TOKEN), 160);
            while collective_sim.state.month == 1 {
                collective_audit.step(&mut collective_sim).unwrap();
            }
        }
        let run = |backend| {
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    inventory: s
                        .balances
                        .iter()
                        .filter(|((_, r), q)| [GRAIN, ORE].contains(r) && **q > 0)
                        .map(|(k, q)| (*k, i128::from(*q)))
                        .collect(),
                    processes: Some(Default::default()),
                    ..Opening::default()
                },
            )
            .unwrap();
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.phase != Phase::Acquire {
                audit.step(&mut sim).unwrap();
            }
            let (mut resumed, mut ra) = (sim.clone(), audit.clone());
            audit.step(&mut sim).unwrap();
            assert_eq!(sim.state.balance(HOME, TOKEN), 0);
            if loan {
                assert_eq!(sim.state.credit.loans[&1].principal, 10);
            }
            let round = sim.ledger.last().unwrap().negotiation.as_ref().unwrap();
            if full {
                assert_eq!(round.outcome, Outcome::InsufficientStorage);
                assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
                assert_eq!(sim.state.balance(HOME, GRAIN), 0);
                assert_eq!(sim.state.balance(HOME, ORE), 2);
                assert_eq!(sim.state.balance(SELLER, TOKEN), 0);
            } else {
                assert!(matches!(round.outcome, Outcome::Traded { price: 40 }));
                assert_eq!(sim.state.balance(PERSON, GRAIN), 1);
                assert_eq!(sim.state.balance(HOME, GRAIN), 1);
                assert_eq!(sim.state.balance(SELLER, TOKEN), 40);
            }
            while sim.state.month <= 2 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month <= 2 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &ra)
            );
            for a in &sim.world.agents {
                let report = audit.book().statements(a.id, 1, 2).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
