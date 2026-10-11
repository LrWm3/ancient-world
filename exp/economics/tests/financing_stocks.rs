use economics_compute_smoke::{
    acceptance::{
        TOWN, WORKSHOP,
        continuation::Case,
        financing::{self, Stocks},
    },
    minting::{METAL, ORE},
};

#[test]
fn ablations_change_only_declared_workshop_balances() {
    for authority in [false, true] {
        let (base_world, base_state) =
            financing::with_authority(Case::Baseline, authority).unwrap();
        for stocks in Stocks::ALL {
            let (world, state) = financing::with_stocks(Case::Baseline, authority, stocks).unwrap();
            let mut expected = base_state.clone();
            if matches!(stocks, Stocks::MetalOnly | Stocks::Neither) {
                expected.balances.insert((WORKSHOP, ORE), 0);
            }
            if matches!(stocks, Stocks::OreOnly | Stocks::Neither) {
                expected.balances.insert((WORKSHOP, METAL), 0);
            }
            assert_eq!(world, base_world);
            assert_eq!(state, expected);
            assert_eq!(state.balance(TOWN, ORE), 0);
        }
    }
}
