use economics_compute_smoke::{
    acceptance::{self, Run, WORKSHOP, continuation::Case, financing::Stocks, startup_publication},
    compute::Backend,
};
fn compare(stocks: Stocks, enabled: bool, case: Case) {
    let (w, s) = startup_publication::scenario(case, stocks, enabled).unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(48).unwrap();
    let e = run.evidence().unwrap();
    let ready = acceptance::governed::check(&run).unwrap();
    println!(
        "STARTUP {stocks:?} {enabled} {case:?}: ready={} mints={} workshop={} town={} food={} wages={} forwards={:?} failures={:?}",
        ready.ready(),
        e.completed_mints,
        e.workshop_batches,
        e.municipal_batches,
        e.people_food_deficit,
        e.wage_arrears,
        run.sim.world.agency[&WORKSHOP]
            .startup_forwards
            .iter()
            .map(|a| (a.month, a.status.clone()))
            .collect::<Vec<_>>(),
        ready.failures
    );
    let (expected, forward) = match (stocks, enabled, case) {
        (Stocks::Both, true, Case::Baseline) => ((true, 45, 44, 43, 0, 0), None),
        (Stocks::MetalOnly, true, Case::Baseline) => ((true, 44, 43, 44, 0, 0), Some((3, 5))),
        (Stocks::OreOnly, true, Case::Baseline) => ((true, 45, 46, 46, 0, 0), None),
        (Stocks::OreOnly, false, Case::Baseline) => ((false, 0, 1, 2, 67, 2), None),
        (Stocks::Neither, _, Case::Baseline) => ((false, 0, 1, 3, 67, 2), Some((1, 3))),
        (Stocks::MetalOnly, true, Case::NoOreSupply) => ((false, 2, 0, 0, 52, 2), None),
        (Stocks::OreOnly, true, Case::LowMetalPrice) => ((false, 8, 8, 8, 22, 2), None),
        _ => panic!("unmeasured comparison"),
    };
    assert_eq!(
        (
            ready.ready(),
            e.completed_mints,
            e.workshop_batches,
            e.municipal_batches,
            e.people_food_deficit,
            e.wage_arrears
        ),
        expected
    );
    assert_eq!(
        run.sim.world.agency[&WORKSHOP]
            .startup_forwards
            .iter()
            .map(|a| (a.month, a.status.clone()))
            .collect::<Vec<_>>(),
        forward
            .into_iter()
            .map(|(signed, activated)| (
                signed,
                economics_compute_smoke::agency::forwards::execution::Status::Activated(activated)
            ))
            .collect::<Vec<_>>()
    );
    assert!(run.sim.state.balances.values().all(|q| *q >= 0));
}
#[test]
fn both() {
    compare(Stocks::Both, true, Case::Baseline);
}
#[test]
fn metal() {
    compare(Stocks::MetalOnly, true, Case::Baseline);
}
#[test]
fn ore() {
    compare(Stocks::OreOnly, true, Case::Baseline);
}
#[test]
fn neither() {
    compare(Stocks::Neither, true, Case::Baseline);
}
#[test]
fn ore_control() {
    compare(Stocks::OreOnly, false, Case::Baseline);
}
#[test]
fn neither_control() {
    compare(Stocks::Neither, false, Case::Baseline);
}
#[test]
fn no_ore() {
    compare(Stocks::MetalOnly, true, Case::NoOreSupply);
}
#[test]
fn low_price() {
    compare(Stocks::OreOnly, true, Case::LowMetalPrice);
}

#[test]
fn early_output_offer_breaks_the_measured_founding_tie_without_a_promised_buyer() {
    use economics_compute_smoke::{
        agency::{Command, completions},
        marketplace::Side,
        minting::{ISSUER, METAL, MINT},
    };
    for enabled in [false, true] {
        let (w, s) =
            startup_publication::scenario(Case::Baseline, Stocks::OreOnly, enabled).unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.audit.step(&mut run.sim).unwrap();
        let d = &run.sim.world.agency[&WORKSHOP].history[0];
        let selected = d.chosen.unwrap();
        let a = d
            .alternatives
            .iter()
            .find(|a| a.program == Some(selected))
            .unwrap();
        assert_eq!(a.losses, Some(vec![11, 11972]));
        let has_ask = |p: &economics_compute_smoke::agency::Program| {
            p.commands.iter().any(|c|matches!(c,Command::PeerStockQuotes(q) if q.iter().any(|q|q.market==METAL && q.side==Side::Sell)))
        };
        assert_eq!(has_ask(d.accepted.as_ref().unwrap()), enabled);
        assert!(d.alternatives.iter().any(|other| {
            other
                .program
                .is_some_and(|id| has_ask(&d.catalog[&id]) != enabled)
                && other.losses == a.losses
        }));
        assert_eq!(d.effective_month, 2);
        run.through(8).unwrap();
        let mints = completions::observe(&run.sim.ledger, ISSUER, 1, 8)
            .into_iter()
            .filter(|c| c.definition == MINT)
            .map(|c| c.month)
            .collect::<Vec<_>>();
        assert_eq!(mints, if enabled { vec![3, 4, 6, 7, 8] } else { vec![] });
    }
}

#[test]
fn output_publication_cannot_supply_missing_labor_storage_or_permissions() {
    use economics_compute_smoke::{
        acceptance::{SMITH, WORKSHOP_TYPE},
        minting::{METAL, REFINE},
        opportunities::Action,
    };
    for case in ["labor", "storage", "process law", "market access"] {
        let (mut w, mut s) =
            startup_publication::scenario(Case::Baseline, Stocks::OreOnly, true).unwrap();
        match case {
            "labor" => {
                w.participants
                    .iter_mut()
                    .find(|p| p.agent == SMITH)
                    .unwrap()
                    .capacity
                    .quantity = 0
            }
            "storage" => {
                w.storage.capacities.insert(WORKSHOP, 0);
                s.balances
                    .insert((WORKSHOP, economics_compute_smoke::minting::ORE), 0);
            }
            "process law" => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(WORKSHOP_TYPE, Action::Process(REFINE)));
            }
            "market access" => {
                w.marketplaces[0].allowed_types.remove(&WORKSHOP_TYPE);
            }
            _ => unreachable!(),
        }
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(12).unwrap();
        let e = run.evidence().unwrap();
        println!("PUBLICATION_CONTROL {case}: {e:?}");
        if case != "market access" {
            assert_eq!(e.workshop_batches, 0, "{case}");
        }
        assert_eq!(e.completed_mints, 0, "{case}");
        assert!(
            run.sim
                .ledger
                .iter()
                .filter_map(|b| b.minting.as_ref())
                .flat_map(|b| &b.deals)
                .all(|d| d.seller != WORKSHOP || d.market != METAL),
            "{case}"
        );
    }
}

#[test]
fn early_publication_without_startup_forward_authority_is_measured_separately() {
    use economics_compute_smoke::{acceptance::TOWN, agency::organization};
    for stocks in [Stocks::OreOnly, Stocks::Neither] {
        let (mut w, s) = startup_publication::scenario(Case::Baseline, stocks, true).unwrap();
        for agent in [WORKSHOP, TOWN] {
            let old = w.agency.remove(&agent).unwrap();
            let mut agreement = old.operating_agreement().unwrap().clone();
            agreement.forwards.clear();
            organization::install_discovering_initial(
                &mut w,
                &s,
                agreement.clone(),
                &agreement.members,
                old.config,
            )
            .unwrap();
        }
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(48).unwrap();
        let e = run.evidence().unwrap();
        let ready = acceptance::governed::check(&run).unwrap();
        println!(
            "NO_FORWARD {stocks:?}: ready={} mints={} workshop={} town={} food={} wages={}",
            ready.ready(),
            e.completed_mints,
            e.workshop_batches,
            e.municipal_batches,
            e.people_food_deficit,
            e.wage_arrears
        );
        assert!(
            run.sim
                .world
                .agency
                .values()
                .all(|c| c.startup_forwards.is_empty())
        );
        let expected = match stocks {
            Stocks::OreOnly => (true, 45, 46, 46, 0, 0),
            Stocks::Neither => (false, 0, 0, 0, 67, 2),
            _ => unreachable!(),
        };
        assert_eq!(
            (
                ready.ready(),
                e.completed_mints,
                e.workshop_batches,
                e.municipal_batches,
                e.people_food_deficit,
                e.wage_arrears
            ),
            expected
        );
    }
}

#[test]
fn earlier_publication_requires_actual_crossing_buyer_demand() {
    use economics_compute_smoke::{
        agency::objectives::{Metric, Objective, Scope},
        minting::{COIN, ISSUER, METAL},
    };
    for case in ["satisfied issuer", "noncrossing bid"] {
        let (mut w, s) =
            startup_publication::scenario(Case::Baseline, Stocks::OreOnly, true).unwrap();
        if case == "satisfied issuer" {
            let c = &mut w.agency.get_mut(&ISSUER).unwrap().config;
            c.objectives = vec![Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: COIN,
                    target: 0,
                },
            }];
            c.preferences.clear();
        } else {
            w.minting
                .as_mut()
                .unwrap()
                .order_policy
                .as_mut()
                .unwrap()
                .input_limits
                .insert(METAL, 2);
        }
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(12).unwrap();
        let e = run.evidence().unwrap();
        assert!(
            e.workshop_batches > 0,
            "the negative demand control must still produce metal: {case}"
        );
        assert_eq!(e.completed_mints, 0, "{case}");
        assert!(
            run.sim
                .ledger
                .iter()
                .filter_map(|b| b.minting.as_ref())
                .flat_map(|b| &b.deals)
                .all(|d| d.seller != WORKSHOP || d.market != METAL),
            "{case}"
        );
    }
}
