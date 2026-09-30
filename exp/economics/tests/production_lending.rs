use economics_compute_smoke::{
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::{Audit, Opening},
    household_governance::{Governance, Purchasing},
    households::{self, Agreement},
    model::*,
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    process_accounting::{Costs, Output},
    production_market::{self, Policy},
    scenario::*,
    settlement,
    simulation::Simulation,
};
const HOME: AgentId = 10_000;
const OTHER: AgentId = 89;
const LOAN: u32 = 500;

fn fixture(funded: bool) -> (World, State) {
    let (mut w, mut s) = production_market::scenario(true);
    w.participants
        .retain(|p| [PERSON, OTHER].contains(&p.agent));
    let market = w.town_market.as_mut().unwrap();
    market.order_horizon = economics_compute_smoke::town_market::OrderHorizon::Aligned(3);
    market
        .traders
        .retain(|t| [PERSON, OTHER].contains(&t.trader.agent));
    w.production_market.as_mut().unwrap().horizon = 3;
    w.production_market.as_mut().unwrap().policy = Policy::Plan;
    s.balances.insert((PERSON, TOKEN), 0);
    s.balances.insert((PERSON, GRAIN), 0);
    s.balances.insert((PERSON, FUEL), 0);
    s.balances.insert((OTHER, GRAIN), 12);
    s.balances.insert((OTHER, FUEL), 12);
    s.balances
        .insert((STATE_AGENT, TOKEN), if funded { 12 } else { 11 });
    if let Some(p) = &mut w.transaction_policy {
        p.permissions.extend([
            (PERSON_TYPE, Action::Borrow),
            (PERSON_TYPE, Action::FoundHousehold),
            (STATE_TYPE, Action::Lend),
        ]);
    }
    w.lending.push(Advance {
        id: LOAN,
        debtor: PERSON,
        principal: 12,
        month: 1,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: TOKEN,
            max_principal: 12,
            monthly_rate_bps: 0,
            term_months: 3,
            grace_months: 12,
        },
    });
    let mut governance = Governance::contributed(PERSON);
    governance.charter.purchasing = Purchasing::Members;
    households::form(
        &mut w,
        &s,
        Agreement {
            id: 1,
            agent: HOME,
            adults: vec![PERSON],
            governance,
            formed: 1,
            dwelling_process: None,
            admission: None,
            membership: vec![],
            asset_sales: vec![],
            equipment_retirements: vec![],
            support: vec![],
        },
    )
    .unwrap();
    (w, s)
}

fn opening(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| {
                    *r != TOKEN
                        && **q > 0
                        && w.resources
                            .iter()
                            .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
                })
                .map(|(a, q)| (*a, i128::from(*q)))
                .collect(),
            exchange_values: [GRAIN, SEED, RAW_WOOD, FUEL]
                .into_iter()
                .map(|r| (r, 1))
                .collect(),
            assets: w.assets.iter().map(|a| (a.id, 1)).collect(),
            processes: Some(Costs {
                output_weights: [
                    (
                        GROW,
                        [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
                    ),
                    (PREPARE_FUEL, [(Output::Stock(FUEL), 1)].into()),
                ]
                .into(),
                ..Costs::default()
            }),
            ..Opening::default()
        },
    )
    .unwrap()
}

#[test]
fn household_production_forecasts_include_loans_without_recycling_acquisition_receipts() {
    for funded in [false, true] {
        let (w, s) = fixture(funded);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut audit = opening(&w, &s);
            while sim.state.phase != Phase::Acquire {
                audit.step(&mut sim).unwrap();
            }
            let before = sim.state.clone();
            audit.step(&mut sim).unwrap();
            let accepted = sim.ledger.last().unwrap();
            assert_eq!(sim.state.credit.loans.contains_key(&LOAN), funded);
            assert_eq!(
                sim.state.balance(PERSON, TOKEN),
                if funded { 12 } else { 0 }
            );
            let economics_compute_smoke::town_market::Boundary::Market(town) =
                accepted.town_market.as_ref().unwrap()
            else {
                panic!("missing market round")
            };
            assert!(town.attempts.iter().all(|a| a.session.buyer.agent != PERSON
                || !matches!(
                    a.round.outcome,
                    economics_compute_smoke::negotiation::Outcome::Traded { .. }
                )));
            let decision = town.planning.as_ref().unwrap();
            let person = decision.people.iter().find(|p| p.agent == PERSON).unwrap();
            assert!(
                person
                    .alternatives
                    .iter()
                    .all(|a| a.closing_debt == 0 || funded)
            );
            if funded {
                assert!(person.alternatives.iter().any(|a| a.closing_debt > 0));
                assert!(person.alternatives.iter().all(|a| !a.missed_payment));
                assert!(
                    person
                        .alternatives
                        .iter()
                        .all(|a| a.deficits[&NUTRITION] > 0)
                );
            }
            let mut bad = accepted.clone();
            let economics_compute_smoke::town_market::Boundary::Market(round) =
                bad.town_market.as_mut().unwrap()
            else {
                panic!("missing market round")
            };
            round.planning.as_mut().unwrap().people[0].alternatives[0].closing_debt += 1;
            let mut unchanged = before.clone();
            assert!(
                settlement::commit(&sim.world, &mut unchanged, &bad, backend, sim.effect_limit)
                    .is_err()
            );
            assert_eq!(unchanged, before);
            let prefix = sim.ledger.len();
            let mut resumed =
                Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
            let mut ra = audit.clone();
            while (sim.state.month, sim.state.phase) != (2, Phase::Productive) {
                audit.step(&mut sim).unwrap();
            }
            while (resumed.state.month, resumed.state.phase) != (2, Phase::Productive) {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger[prefix..], &audit),
                (&resumed.state, &resumed.ledger[..], &ra)
            );
            if funded {
                assert_eq!(sim.state.credit.loans[&LOAN].principal, 8);
                let current = sim
                    .state
                    .town_market
                    .history
                    .last()
                    .unwrap()
                    .planning
                    .as_ref()
                    .unwrap();
                let person = current.people.iter().find(|p| p.agent == PERSON).unwrap();
                assert!(person.alternatives.iter().any(|a| a.missed_payment));
                assert!(
                    sim.state
                        .town_market
                        .history
                        .iter()
                        .filter(|r| r.month == 2)
                        .flat_map(|r| &r.attempts)
                        .any(|a| a.session.buyer.agent == PERSON
                            && matches!(
                                a.round.outcome,
                                economics_compute_smoke::negotiation::Outcome::Traded { .. }
                            ))
                );
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn market_currency_check_uses_accepted_loans_after_offer_catalog_changes() {
    let (mut w, mut s) = fixture(true);
    let policy = w.production_market.take();
    w.lending[0].terms.denomination = FUEL;
    s.balances.insert((STATE_AGENT, FUEL), 12);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    assert_eq!(sim.state.credit.loans[&LOAN].denomination, FUEL);
    sim.world.lending[0].terms.denomination = TOKEN;
    sim.world.production_market = policy;
    assert_eq!(
        Simulation::new(sim.world, sim.state, Backend::Reference).unwrap_err(),
        "accepted production-planning loans must use the market currency"
    );
}
