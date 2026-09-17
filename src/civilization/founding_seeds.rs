//! Finite planting supplies committed with annual daughter founding in Respond.
use super::History;
use crate::economy::{Economy, EconomyCatalog};

const DONOR_SEED_SHARE: f32 = 0.2;
const CROP_SEED_TARGET_KG: f32 = 2.;
// A toy establishment packet, safely above the shader's 0.001 kg live-crop threshold.
const MIN_EDIBLE_SEED_PACKET_KG: f32 = 0.1;

#[derive(Clone, Copy, Default)]
struct Transfer {
    good: usize,
    seed_kg: f32,
    stock_kg: f32,
}
#[derive(Clone, Default)]
pub(super) struct SeedPlan([Transfer; 6]);

impl SeedPlan {
    fn prepare(source: &Economy, catalog: &EconomyCatalog) -> Option<Self> {
        let Some(agriculture) = &catalog.agriculture else {
            return Some(Self::default()); // Legacy farming has its own seed accounting.
        };
        let mut plan = Self::default();
        let mut available_stock = source.goods.map(|v| v.max(0.) * DONOR_SEED_SHARE);
        let mut edible_established = false;
        for (i, crop) in agriculture.crops.iter().enumerate() {
            let good = catalog.index(&crop.good)?;
            let seed_kg = (source.crops[i][2].max(0.) * DONOR_SEED_SHARE).min(CROP_SEED_TARGET_KG);
            let stock_kg = available_stock[good].min(CROP_SEED_TARGET_KG - seed_kg);
            available_stock[good] -= stock_kg;
            plan.0[i] = Transfer {
                good,
                seed_kg,
                stock_kg,
            };
            edible_established |= catalog.goods[good].food_energy > 0.
                && seed_kg + stock_kg >= MIN_EDIBLE_SEED_PACKET_KG;
        }
        edible_established.then_some(plan)
    }

    fn commit(&self, source: &mut Economy, destination: &mut Economy) -> f32 {
        let mut total = 0.;
        for (i, t) in self.0.iter().enumerate() {
            source.crops[i][2] -= t.seed_kg;
            source.goods[t.good] -= t.stock_kg;
            destination.crops[i][2] += t.seed_kg + t.stock_kg;
            total += t.seed_kg + t.stock_kg;
        }
        total
    }
}

impl History {
    pub(super) fn daughter_seed_plan(&self, from: usize) -> Option<SeedPlan> {
        if self.version != 2 || self.farming_mode == Some(false) {
            return Some(SeedPlan::default());
        }
        SeedPlan::prepare(&self.sites[from].economy, self.economy_catalog.as_ref()?)
    }

    pub(super) fn commit_daughter_seeds(&mut self, from: usize, to: usize, plan: SeedPlan) {
        let (older, new) = self.sites.split_at_mut(to);
        let total = plan.commit(&mut older[from].economy, &mut new[0].economy);
        if total > 0. {
            self.event("founding_seeds", Some(from as u32), Some(to as u32),
                format!("Founders carried {total:.2} kg of planting material from their parent town's seed and unprocessed crop stores"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_plan_uses_real_stocks_and_requires_an_edible_crop() {
        let c = EconomyCatalog::bundled().unwrap();
        let a = c.agriculture.as_ref().unwrap();
        let wheat = c.index(&a.crops[0].good).unwrap();
        let flax = c.index(&a.crops[5].good).unwrap();
        let mut donor = Economy::default();
        donor.goods[flax] = 100.;
        donor.crops[0][1] = 100.; // Standing biomass is not harvested seed.
        assert!(SeedPlan::prepare(&donor, &c).is_none());
        donor.goods[wheat] = 0.1;
        assert!(SeedPlan::prepare(&donor, &c).is_none());
        donor.goods[wheat] = 5.;
        donor.crops[0][2] = 5.;
        let before = donor;
        let plan = SeedPlan::prepare(&donor, &c).unwrap();
        let mut daughter = Economy::default();
        plan.commit(&mut donor, &mut daughter);
        assert_eq!(donor.goods[wheat], 4.);
        assert_eq!(donor.crops[0][2], 4.);
        assert_eq!(daughter.crops[0][2], 2.);
        for (i, crop) in a.crops.iter().enumerate() {
            let good = c.index(&crop.good).unwrap();
            assert_eq!(
                before.goods[good] + before.crops[i][2],
                donor.goods[good] + donor.crops[i][2] + daughter.crops[i][2]
            );
        }
        assert_eq!(donor.external, before.external);
        assert_eq!(daughter.external, [0.; 4]);
    }
}
