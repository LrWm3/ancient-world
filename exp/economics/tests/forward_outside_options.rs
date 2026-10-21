use economics_compute_smoke::{
    acceptance::{
        TOWN, WORKSHOP,
        continuation::Case,
        financing::{Liquidity, Preferences, Stocks},
        production_preferences::{self, Priority},
    },
    agency::{
        self, Command,
        forwards::{self, assessment},
    },
    minting::{ORE, REFINE},
};

#[test]
fn forward_must_beat_independent_spot_work_but_can_still_enable_startup() {
    for stocked_spot_seller in [false, true] {
        let (mut world, mut state) = production_preferences::scenario(
            Case::Baseline,
            Stocks::Neither,
            Preferences {
                workshop: Liquidity::MonthlyExposure,
                town: Liquidity::MonthlyExposure,
            },
            Priority::ProductionFirst,
        )
        .unwrap();
        if stocked_spot_seller {
            state.balances.insert((TOWN, ORE), 4);
            let catalog = agency::discovery::programs(&world, TOWN).unwrap();
            let quotes = catalog
                .values()
                .flat_map(|p| &p.commands)
                .find_map(|c| match c {
                    Command::PeerStockQuotes(q) => Some(q.clone()),
                    _ => None,
                })
                .unwrap();
            let book = world
                .minting
                .as_mut()
                .unwrap()
                .order_policy
                .as_mut()
                .unwrap()
                .peer_stocks
                .as_mut()
                .unwrap();
            book.quotes.retain(|q| q.agent != TOWN);
            book.quotes.extend(quotes);
        }
        let before = (world.clone(), state.clone());
        let proposal = forwards::candidates(&world, &state).unwrap().remove(0);
        let result = assessment::assess(&world, &state, &proposal).unwrap();
        // The old retain-only comparison approves both proposals.
        assert!(result.comparisons.values().all(|c| {
            c.admitted
                && c.delivered_on_time == proposal.terms.goods.quantity
                && c.proposed <= c.baseline
        }));
        assert!(result.comparisons.values().any(|c| c.proposed < c.baseline));
        assert_eq!(result.mutually_beneficial(), !stocked_spot_seller);
        let buyer = &result.comparisons[&WORKSHOP];
        let outside = buyer.self_funded.as_ref().unwrap();
        if stocked_spot_seller {
            assert!(outside.losses < buyer.proposed);
            assert!(outside.losses < buyer.baseline);
            // The delayed independent start is better than an immediate one;
            // checking only the next ordinary Open misses this alternative.
            assert_eq!(outside.month, state.month + 2);
            assert!(outside.completions.iter().any(|c| {
                c.operator == WORKSHOP && c.definition == REFINE && c.month == outside.month
            }));
        } else {
            assert!(buyer.proposed < outside.losses);
            assert!(outside.completions.is_empty());
        }
        for (&actor, comparison) in &result.comparisons {
            if let Some(option) = &comparison.self_funded {
                assert!(option.losses <= comparison.baseline);
                assert!(option.completions.iter().all(|c| c.operator == actor));
            }
        }
        // No speculative program, counterparty action or prepayment escapes.
        assert_eq!((world, state), before);
    }
}
