//! Sparse staffing forecast; physical stock transfers remain in GPU production.
use crate::economy::{Economy, EconomyCatalog};

/// Ordinary work follows the lease share. A due customer commitment can claim
/// more of the same feasible pool, but never more than that pool contains.
pub(super) fn contracted_share(total: f64, lease_share: f64, contracted: f64) -> f64 {
    (total * lease_share.clamp(0., 1.))
        .max(contracted)
        .min(total)
}

/// One ordered pass through current stock. Shared inputs are counted once and
/// later recipes may use forecast intermediates. New extraction and undelivered
/// cargo are deliberately not spendable inputs. This can delay private hiring
/// until next month; it must not suppress the town's upstream production orders.
/// Equipment, labor arbitration, tool-priority waves and later consumption can
/// still reduce actual work. Never commit this scratch inventory to the world.
pub(super) fn stocked_work(catalog: &EconomyCatalog, economy: &Economy, month: u32) -> [f64; 4] {
    let mut stock = economy.goods.map(f64::from);
    let mut work = [0.; 4];
    let mut residue = f64::from(economy.residue[0]);
    for step in 0..catalog.recipes.len() {
        let index = (step + month as usize) % catalog.recipes.len();
        let recipe = &catalog.recipes[index];
        if recipe.work[1] > 0.
            && (economy.management[3] as u32 & (1 << (recipe.work[1] as u32 - 1))) == 0
        {
            continue;
        }
        let mut batches = f64::from(economy.orders[index]);
        let mut expansion = 0.;
        let mut stored = 0.;
        let mut household = false;
        for (k, held) in stock.iter().enumerate() {
            let input = f64::from(recipe.input[k]);
            let output = f64::from(recipe.output[k]);
            if input > 0. {
                batches = batches.min(*held / input);
            }
            if output > 0. {
                batches = batches.min((f64::from(economy.targets[k]) - held).max(0.) / output);
                household |= catalog.goods[k].food_energy > 0.;
            }
            if k < crate::economy::FOOD && catalog.goods[k].food_energy <= 0. {
                stored += held;
                expansion += output - input;
            }
        }
        if expansion > 0. {
            batches = batches.min(
                (f64::from(economy.logistics[0] - economy.logistics[1]) - stored).max(0.)
                    / expansion,
            );
        }
        if economy.extraction[1] > 0.5 && recipe.work[3] > 0. {
            batches = batches
                .min((f64::from(economy.residue[2]) - residue).max(0.) / f64::from(recipe.work[3]));
            residue += batches * f64::from(recipe.work[3]);
        }
        for (k, held) in stock.iter_mut().enumerate() {
            *held = (*held - f64::from(recipe.input[k]) * batches).max(0.)
                + f64::from(recipe.output[k]) * batches;
        }
        if !household {
            work[recipe.work[2] as usize] += batches * f64::from(recipe.work[0]);
        }
    }
    work
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::{Recipe, GOODS};

    #[test]
    fn funded_work_can_use_capacity_beyond_ordinary_market_share() {
        assert_eq!(contracted_share(10., 0.25, 0.), 2.5);
        assert_eq!(contracted_share(10., 0.25, 6.), 6.);
        assert_eq!(contracted_share(3., 0.25, 6.), 3.);
        assert_eq!(contracted_share(0., 0.25, 6.), 0.);
        assert_eq!(contracted_share(10., 0.25, 1.), 2.5);
        // Demand and input feasibility both bound the commitment independently.
        assert_eq!(
            contracted_share(10., 0.25, 6.).min(contracted_share(4., 0.25, 6.)),
            4.
        );
    }

    #[test]
    fn stocked_forecast_accounts_for_shared_inputs_and_intermediates() {
        let mut catalog = EconomyCatalog::bundled().unwrap();
        let mut first = Recipe {
            input: [0.; GOODS],
            output: [0.; GOODS],
            work: [1., 0., 0., 0.],
        };
        first.input[0] = 1.;
        first.output[6] = 1.;
        let mut competitor = first;
        competitor.work[2] = 1.;
        competitor.output[6] = 0.;
        competitor.output[3] = 1.;
        let mut downstream = competitor;
        downstream.input[0] = 0.;
        downstream.input[6] = 1.;
        downstream.work[2] = 2.;
        catalog.recipes = vec![first, competitor, downstream];
        let mut e = Economy::default();
        e.orders[..3].fill(10.);
        e.targets.fill(100.);
        e.logistics[0] = 1000.;
        assert_eq!(stocked_work(&catalog, &e, 0), [0.; 4]);
        e.goods[0] = 2.;
        let before = e.goods;
        assert_eq!(stocked_work(&catalog, &e, 0), [2., 0., 2., 0.]);
        assert_eq!(e.goods, before);
        // Rotating the same finite requests changes who can use timber first.
        assert_eq!(stocked_work(&catalog, &e, 1), [0., 2., 0., 0.]);
        // Output room and processing waste constrain the very first producer.
        e.targets[6] = 1.;
        assert_eq!(stocked_work(&catalog, &e, 0), [1., 1., 1., 0.]);
        e.extraction[1] = 1.;
        e.residue[2] = 0.5;
        catalog.recipes[0].work[3] = 1.;
        assert_eq!(stocked_work(&catalog, &e, 0), [0.5, 1.5, 0.5, 0.]);
        // Expansion is blocked in a full yard; non-expanding alternatives remain.
        let mut full = e;
        full.logistics[0] = 2.;
        let mut expansion = catalog.clone();
        expansion.recipes[0].output[6] = 2.;
        assert_eq!(stocked_work(&expansion, &full, 0), [0., 2., 0., 0.]);
        // A knowledge requirement cannot be fulfilled by stock alone.
        catalog.recipes[0].work[1] = 1.;
        assert_eq!(stocked_work(&catalog, &e, 0), [0., 2., 0., 0.]);
        e.management[3] = 1.;
        assert_eq!(stocked_work(&catalog, &e, 0), [0.5, 1.5, 0.5, 0.]);
    }
}
