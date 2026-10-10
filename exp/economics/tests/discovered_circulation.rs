use economics_compute_smoke::{
    agency::{integration::GROWER, objectives::Metric},
    compute::Backend,
    discovery::scenario,
    financial_reporting::Audit,
    minting::*,
    model::*,
    simulation::Simulation,
};

fn run(w: World, s: State, backend: Backend, resume: bool) -> (Simulation, Audit) {
    let mut audit = scenario::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, backend).unwrap();
    while sim.state.month <= scenario::RUN_MONTHS {
        audit.step(&mut sim).unwrap();
        if resume {
            let mut next = Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
            next.ledger = sim.ledger;
            next.reports = sim.reports;
            sim = next;
        }
    }
    (sim, audit)
}
fn accepted(sim: &Simulation) -> Vec<&Deal> {
    sim.ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .flat_map(|b| {
            b.deals.iter().filter(|d| {
                b.receipts
                    .iter()
                    .any(|r| r.accepted && r.deals.contains(&d.id))
            })
        })
        .collect()
}
fn deficit(sim: &Simulation, agent: AgentId) -> i32 {
    sim.reports
        .iter()
        .filter(|r| r.agent == agent)
        .map(|r| r.deficit(NUTRITION))
        .sum()
}
fn show(label: &str, sim: &Simulation) {
    println!(
        "{label}: deficits {:?}, accepted {:?}, final {:?}",
        [ISSUER, SUPPLIER, GROWER, WORKER].map(|a| (a, deficit(sim, a))),
        accepted(sim),
        sim.state.balances
    );
}

#[test]
fn discovered_wages_buy_later_food_with_cpu_books_and_phase_continuation() {
    let (w, s) = scenario::circulation().unwrap();
    assert_eq!(s.balance(WORKER, COIN), 0);
    assert!(s.memberships.is_empty());
    assert!(w.agency.is_empty() && w.state_governance.is_none());
    assert!(w.agreements.is_empty() && w.rights.is_empty() && w.access_offers.is_empty());
    assert!(w.lending.is_empty() && w.prepaid_deliveries.is_empty() && w.employment.is_empty());
    assert!(w.scheduled_starts.is_empty() && w.activities.orders.is_empty());
    let mint = w.minting.as_ref().unwrap();
    assert!(mint.deals.is_empty());
    assert!(mint.order_policy.as_ref().unwrap().quotes.is_empty());
    let (reference, book) = run(w.clone(), s.clone(), Backend::Reference, false);
    let (cpu, cpu_book) = run(w, s, Backend::CubeCpu, true);
    assert_eq!(cpu.world, reference.world);
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    assert_eq!(cpu_book, book);
    show("circulation", &cpu);
    let deals = accepted(&cpu);
    let wages: Vec<_> = deals
        .iter()
        .filter(|d| d.seller == WORKER && d.market == HOURS)
        .collect();
    let purchases: Vec<_> = deals
        .iter()
        .filter(|d| d.buyer == WORKER && d.seller == ISSUER && d.market == WHEAT)
        .collect();
    assert!(!wages.is_empty() && !purchases.is_empty());
    assert!(wages[0].month < purchases[0].month);
    assert_eq!((wages[0].month, purchases[0].month), (3, 4));
    assert!(cpu.state.credit.loans.is_empty() && cpu.state.exchange.forwards.is_empty());
    let earned: i32 = wages.iter().map(|d| d.price).sum();
    let spent: i32 = purchases.iter().map(|d| d.price).sum();
    assert_eq!((earned, spent), (4, 3));
    assert_eq!(cpu.state.balance(WORKER, COIN), earned - spent);
    let fulfilled: i32 = cpu
        .reports
        .iter()
        .filter(|r| r.agent == WORKER)
        .map(|r| r.fulfilled(NUTRITION))
        .sum();
    let food_lot = cpu
        .world
        .marketplaces
        .iter()
        .find(|v| v.agent == VENUE)
        .unwrap()
        .markets
        .iter()
        .find(|m| m.id == WHEAT)
        .unwrap()
        .goods
        .quantity;
    assert_eq!(
        fulfilled,
        3 + purchases.len() as i32 * food_lot - cpu.state.balance(WORKER, WHEAT)
    );
    assert_eq!(fulfilled, 6);
    assert!(
        cpu.state
            .processes
            .values()
            .any(|p| p.definition == MINT && p.status == Status::Completed)
    );
    assert!(
        cpu.reports
            .iter()
            .filter(|r| r.agent == ISSUER)
            .all(|r| r.balances[&WHEAT] >= 4)
    );
}

#[test]
fn finite_stock_competition_and_reserves_limit_wage_circulation() {
    for case in ["normal", "legacy", "scarce", "reserved"] {
        let (mut w, mut s) = scenario::circulation().unwrap();
        if case == "legacy" {
            w.discovery.as_mut().unwrap().public_sales = false;
        }
        if case == "scarce" {
            s.balances.insert((ISSUER, WHEAT), 10);
        }
        if case == "reserved" {
            for o in &mut w
                .discovery
                .as_mut()
                .unwrap()
                .state
                .as_mut()
                .unwrap()
                .objectives
            {
                if let Metric::Reserve { resource, target } = &mut o.metric
                    && *resource == WHEAT
                {
                    *target = 16;
                }
            }
        }
        let (sim, _) = run(w, s, Backend::Reference, false);
        show(case, &sim);
        let deals = accepted(&sim);
        let wages: Vec<_> = deals
            .iter()
            .filter(|d| d.seller == WORKER && d.market == HOURS)
            .collect();
        let first_wage = wages.first().unwrap();
        assert_eq!((first_wage.month, first_wage.price), (3, 4), "{case}");
        let earned: i32 = wages.iter().map(|d| d.price).sum();
        let bought: usize = deals
            .iter()
            .filter(|d| d.buyer == WORKER && d.market == WHEAT)
            .count();
        let mints = sim
            .state
            .processes
            .values()
            .filter(|p| p.definition == MINT && p.status == Status::Completed)
            .count();
        let expected = match case {
            // earnings, food lots, completed mints, ending public wheat, all deficits
            "normal" => (4, 1, 1, 7, [0, 8, 14, 8]),
            "scarce" => (4, 0, 1, 4, [0, 8, 14, 11]),
            "legacy" | "reserved" => (8, 0, 2, 16, [0, 14, 14, 11]),
            _ => unreachable!(),
        };
        assert_eq!(
            (
                earned,
                bought,
                mints,
                sim.state.balance(ISSUER, WHEAT),
                [ISSUER, SUPPLIER, GROWER, WORKER].map(|a| deficit(&sim, a))
            ),
            expected,
            "{case}"
        );
        if case == "scarce" || case == "reserved" {
            // Buyers have money and submit bids, but protected stock leaves no ask.
            assert!(
                sim.ledger
                    .iter()
                    .filter_map(|b| b.minting.as_ref())
                    .filter(|b| b.month >= 4)
                    .flat_map(|b| &b.plan.as_ref().unwrap().purchases)
                    .any(|p| p.agent == WORKER
                        && p.opening_cash >= 4
                        && p.submitted_lots > 0
                        && p.matched_lots == 0)
            );
            assert!(
                sim.reports
                    .iter()
                    .filter(|r| r.agent == ISSUER)
                    .all(|r| r.balances[&WHEAT] >= if case == "reserved" { 16 } else { 4 })
            );
        }
        if case == "scarce" {
            let prior_sales: Vec<_> = deals.iter().filter(|d| d.market == WHEAT).collect();
            assert_eq!(prior_sales.len(), 2);
            assert!(
                prior_sales
                    .iter()
                    .all(|d| d.buyer == SUPPLIER && d.month < first_wage.month)
            );
        }
        if case == "normal" {
            // Remaining public grain cannot feed a buyer unable to afford another lot.
            assert!(
                sim.ledger
                    .iter()
                    .filter_map(|b| b.minting.as_ref())
                    .filter(|b| b.month > 6)
                    .flat_map(|b| &b.plan.as_ref().unwrap().purchases)
                    .any(|p| p.agent == WORKER
                        && p.requested_lots > 0
                        && p.opening_cash == 1
                        && p.affordable_lots == 0)
            );
        }
    }
}

#[test]
fn financed_circulation_keeps_household_farming_and_exposes_finite_wage_limits() {
    use economics_compute_smoke::{agency::integration::GROW, credit};
    for case in ["financed", "two-hour lots", "no ore", "no lender cash"] {
        let (mut w, mut s) = scenario::financed_circulation().unwrap();
        assert_eq!(s.balance(ISSUER, COIN), 0);
        assert_eq!(s.balance(WORKER, COIN), 0);
        assert!(w.households.is_empty() && s.memberships.is_empty());
        assert!(w.agency.is_empty() && w.state_governance.is_none());
        assert!(w.agreements.is_empty() && w.rights.is_empty() && w.access_offers.is_empty());
        assert!(w.lending.is_empty() && w.prepaid_deliveries.is_empty());
        assert!(w.scheduled_starts.is_empty() && w.activities.orders.is_empty());
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
        match case {
            "two-hour lots" => {
                w.participants
                    .iter_mut()
                    .find(|p| p.agent == WORKER)
                    .unwrap()
                    .capacity
                    .quantity = 2;
                w.definitions
                    .iter_mut()
                    .find(|d| d.id == MINT)
                    .unwrap()
                    .stages[0]
                    .monthly_services
                    .iter_mut()
                    .find(|a| a.resource == HOURS)
                    .unwrap()
                    .quantity = 2;
                w.marketplaces
                    .iter_mut()
                    .find(|v| v.agent == VENUE)
                    .unwrap()
                    .markets
                    .iter_mut()
                    .find(|m| m.goods.resource == HOURS)
                    .unwrap()
                    .goods
                    .quantity = 2;
            }
            "no ore" => {
                s.balances.insert((SUPPLIER, METAL), 0);
            }
            "no lender cash" => {
                s.balances.insert((SUPPLIER, COIN), 0);
                s.balances.insert((GROWER, COIN), 0);
            }
            _ => {}
        }
        let (reference, book) = run(w.clone(), s.clone(), Backend::Reference, false);
        let (cpu, cpu_book) = run(w, s, Backend::CubeCpu, true);
        assert_eq!(cpu.world, reference.world);
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
        assert_eq!(cpu_book, book);
        show(case, &cpu);
        println!(
            "{case}: loans {:?}, obligations {:?}",
            cpu.state.credit.loans, cpu.state.obligations
        );
        assert_eq!(cpu.world.households.len(), 1);
        assert_eq!(cpu.state.accepted_agreements.len(), 1);
        assert_eq!(cpu.state.obligations.len(), 1);
        assert!(
            cpu.state
                .obligations
                .values()
                .all(|o| o.paid == 1 && o.outstanding() == 0)
        );
        let completed = |id| {
            cpu.state
                .processes
                .values()
                .filter(|p| p.definition == id && p.status == Status::Completed)
                .count()
        };
        assert_eq!(completed(GROW), 10, "{case}");
        assert_eq!(
            completed(MINT),
            if case == "financed" || case == "two-hour lots" {
                2
            } else {
                0
            },
            "{case}"
        );
        assert_eq!(
            cpu.state.credit.loans.len(),
            usize::from(case != "no lender cash")
        );
        assert!(
            cpu.state
                .credit
                .loans
                .values()
                .all(|l| l.status == credit::Status::Repaid
                    && l.original_principal == 6
                    && l.term_months == 8)
        );
        assert_eq!(
            [SUPPLIER, GROWER, WORKER].map(|a| deficit(&cpu, a)),
            [1, 1, if case == "financed" { 5 } else { 11 }],
            "{case}"
        );
        let deals = accepted(&cpu);
        let wages: Vec<_> = deals
            .iter()
            .filter(|d| d.seller == WORKER && d.market == HOURS)
            .collect();
        let purchases: Vec<_> = deals
            .iter()
            .filter(|d| d.buyer == WORKER && d.market == WHEAT)
            .collect();
        if case == "financed" {
            assert_eq!(wages.len(), 2);
            assert_eq!(purchases.len(), 2);
            assert_eq!((wages[0].month, purchases[0].month), (3, 4));
            let earned: i32 = wages.iter().map(|d| d.price).sum();
            let spent: i32 = purchases.iter().map(|d| d.price).sum();
            assert_eq!((earned, spent), (8, 6));
            assert_eq!(cpu.state.balance(WORKER, COIN), earned - spent);
            let food_lot = cpu
                .world
                .marketplaces
                .iter()
                .find(|v| v.agent == VENUE)
                .unwrap()
                .markets
                .iter()
                .find(|m| m.id == WHEAT)
                .unwrap()
                .goods
                .quantity;
            let fulfilled: i32 = cpu
                .reports
                .iter()
                .filter(|r| r.agent == WORKER)
                .map(|r| r.fulfilled(NUTRITION))
                .sum();
            assert_eq!(
                3 + purchases.len() as i32 * food_lot - cpu.state.balance(WORKER, WHEAT),
                fulfilled
            );
        } else {
            assert!(wages.is_empty() && purchases.is_empty(), "{case}");
        }
    }
}
