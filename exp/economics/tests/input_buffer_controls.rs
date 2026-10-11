use economics_compute_smoke::{
    acceptance::{
        self, Run, SMITH, WORKSHOP,
        buffers::{self, Policy},
        continuation::{self, Case},
        financing::{Liquidity, Preferences, Stocks},
    },
    compute::Backend,
    minting::{METAL, ORE},
};

#[derive(Clone, Copy, Debug)]
enum Control {
    NoOre,
    NoAccess,
    NoLabor,
    NoStorage,
    NoForwards,
    LowPrice,
    LatePayroll,
}
fn compare(control: Control) {
    for policy in [Policy::Off, Policy::InputsFirst, Policy::CashFirst] {
        let case = match control {
            Control::NoOre => Case::NoOreSupply,
            Control::NoAccess => Case::NoMetalAccess,
            Control::LowPrice => Case::LowMetalPrice,
            Control::LatePayroll => Case::LatePayroll,
            _ => Case::Baseline,
        };
        let (mut w, s) = buffers::scenario(
            case,
            !matches!(control, Control::NoForwards),
            Stocks::MetalOnly,
            Preferences {
                workshop: Liquidity::MonthlyExposure,
                town: Liquidity::TerminalReserve,
            },
            policy,
        )
        .unwrap();
        match control {
            Control::NoLabor => {
                w.participants
                    .iter_mut()
                    .find(|p| p.agent == SMITH)
                    .unwrap()
                    .capacity
                    .quantity = 0
            }
            Control::NoStorage => {
                w.storage.weights.insert(ORE, 1);
                w.storage.weights.insert(METAL, 0);
                w.storage.capacities.insert(WORKSHOP, 0);
            }
            _ => {}
        }
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(48).unwrap();
        let e = run.evidence().unwrap();
        let ready = acceptance::governed::check(&run).unwrap();
        println!(
            "CONTROL {control:?} {policy:?}: ready={} mints={} workshop={} town={} food={} wages={} failures={:?}",
            ready.ready(),
            e.completed_mints,
            e.workshop_batches,
            e.municipal_batches,
            e.people_food_deficit,
            e.wage_arrears,
            ready.failures
        );
        let expected = match (control, policy) {
            (Control::NoAccess, _) => (0, 0, 0, 67, 2),
            (Control::NoLabor, _) => (0, 0, 0, 88, 0),
            (Control::LatePayroll, _) => (2, 0, 0, 52, 41),
            (Control::LowPrice, Policy::InputsFirst) => (5, 3, 5, 40, 2),
            (Control::LowPrice, _) => (2, 0, 0, 58, 2),
            (Control::NoOre | Control::NoStorage | Control::NoForwards, _) => (2, 0, 0, 52, 2),
        };
        assert!(!ready.ready());
        assert_eq!(
            (
                e.completed_mints,
                e.workshop_batches,
                e.municipal_batches,
                e.people_food_deficit,
                e.wage_arrears
            ),
            expected,
            "{control:?} {policy:?}"
        );
        if matches!(
            control,
            Control::NoOre | Control::NoLabor | Control::NoStorage
        ) {
            assert_eq!(e.workshop_batches, 0, "{control:?} {policy:?}");
            assert!(!ready.ready());
        }
        match control {
            Control::NoAccess => assert_eq!(e.workshop_metal_sales, 0),
            Control::NoLabor => {
                assert_eq!(continuation::period(&run, 1, 48).unwrap().workshop_hours, 0)
            }
            Control::NoStorage => {
                assert_eq!(run.sim.state.balance(WORKSHOP, ORE), 0);
                assert!(
                    !run.sim
                        .ledger
                        .iter()
                        .flat_map(|b| b.all_transactions())
                        .flat_map(|t| &t.effects)
                        .any(|e| e.account == (WORKSHOP, ORE) && e.delta > 0)
                );
            }
            Control::NoForwards => {
                assert!(
                    run.sim
                        .world
                        .agency
                        .values()
                        .all(|c| c.startup_forwards.is_empty())
                );
                assert!(
                    !run.sim
                        .state
                        .exchange
                        .forwards
                        .values()
                        .any(|c| c.creditor == WORKSHOP && c.goods.resource == ORE)
                );
            }
            _ => {}
        }
        assert!(run.sim.state.balances.values().all(|b| *b >= 0));
    }
}
#[test]
fn unavailable_ore() {
    compare(Control::NoOre);
}
#[test]
fn unavailable_market_access() {
    compare(Control::NoAccess);
}
#[test]
fn unavailable_labor() {
    compare(Control::NoLabor);
}
#[test]
fn unavailable_input_storage() {
    compare(Control::NoStorage);
}
#[test]
fn no_forward_authority() {
    compare(Control::NoForwards);
}
#[test]
fn lower_sale_price() {
    compare(Control::LowPrice);
}
#[test]
fn competing_startup_payroll() {
    compare(Control::LatePayroll);
}
