//! Explicit month-zero provision experiments; storage and daughter founding are unchanged.
use super::{History, FOUNDING_PROVISION_KG_PER_PERSON_MONTH};
use anyhow::{ensure, Result};
use std::collections::BTreeSet;

const MIN_PROVISION_MONTHS: u32 = 12;
const MAX_PROVISION_MONTHS: u32 = 120;

fn additional_food(people: f32, previously_declared: f32, months: u32) -> Result<f32> {
    ensure!(
        (MIN_PROVISION_MONTHS..=MAX_PROVISION_MONTHS).contains(&months),
        "founding food must be 12–120 months"
    );
    ensure!(
        people.is_finite()
            && people > 0.
            && previously_declared.is_finite()
            && previously_declared >= 0.,
        "invalid founding inventory"
    );
    let target = people * FOUNDING_PROVISION_KG_PER_PERSON_MONTH * months as f32;
    ensure!(target.is_finite(), "founding food overflow");
    ensure!(
        target >= previously_declared,
        "founding food override cannot withdraw previously declared provisions"
    );
    Ok(target - previously_declared)
}

impl History {
    /// Increase declared human arrival provisions before history starts. Applying
    /// the same target twice imports nothing twice. Existing seed allocations stay put.
    pub fn set_founding_food_months(&mut self, months: u32) -> Result<()> {
        ensure!(
            self.month == 0 && self.version == 2,
            "founding food requires a month-zero managed history"
        );
        let culture = self
            .culture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("founding food requires recorded patron arrivals"))?;
        ensure!(
            culture.patrons.len() == self.sites.len(),
            "founding food requires one recorded arrival per site"
        );
        let mut seen = BTreeSet::new();
        let mut changes = Vec::new();
        for (patron_index, patron) in culture.patrons.iter().enumerate() {
            ensure!(seen.insert(patron.site), "duplicate founding site");
            let site_index = self
                .sites
                .iter()
                .position(|s| s.id == patron.site)
                .ok_or_else(|| anyhow::anyhow!("missing founding site"))?;
            ensure!(
                self.sites[site_index].founded == 0,
                "founding override cannot provision daughter towns"
            );
            let added = additional_food(patron.initial_people, patron.initial_food, months)?;
            changes.push((patron_index, site_index, added));
        }
        for (patron_index, site_index, added) in changes {
            if added == 0. {
                continue;
            }
            self.sites[site_index].stocks.stock[1] += added;
            self.initial_food += added as f64;
            for (initial, fraction) in self
                .nutrition_initial
                .iter_mut()
                .zip(crate::economy::FOOD_CNP)
            {
                *initial += added as f64 * fraction;
            }
            self.culture.as_mut().unwrap().patrons[patron_index].initial_food += added;
            self.event("founding_provisions", Some(self.sites[site_index].id), None,
                format!("The arrival inventory includes {added:.0} kg additional human provisions, bringing declared supplies to {months} adult-ration months per person. Storage and spoilage still apply."));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archived_granary_default_and_capacity_bounds() {
        let mut catalog = crate::economy::EconomyCatalog::bundled().unwrap();
        let mut old = serde_json::to_value(&catalog).unwrap();
        old["production"]
            .as_object_mut()
            .unwrap()
            .remove("base_granary_months");
        let restored: crate::economy::EconomyCatalog = serde_json::from_value(old).unwrap();
        assert_eq!(restored.production.base_granary_months, 12.);
        catalog.production.base_granary_months = 48.;
        catalog.validate().unwrap();
        for bad in [0., 121., f32::NAN] {
            catalog.production.base_granary_months = bad;
            assert!(catalog.validate().is_err());
        }
    }
    #[test]
    fn four_year_supplies_are_additional_not_duplicated() {
        assert_eq!(additional_food(120., 25920., 48).unwrap(), 77760.);
        assert_eq!(additional_food(120., 103680., 48).unwrap(), 0.);
        assert_eq!(additional_food(120., 25920., 12).unwrap(), 0.);
        assert!(additional_food(120., 103680., 12).is_err());
        assert!(additional_food(120., 25920., 0).is_err());
        assert!(additional_food(f32::NAN, 25920., 48).is_err());
    }
}
