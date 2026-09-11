//! Annual daughter founding refines a population target into whole households.
use super::{Candidate, History, LIMIT};
use crate::population_registry::age_band;
use std::collections::BTreeSet;

impl History {
    /// Completed-month movement, like the legacy regional founding abstraction.
    /// No travel month or production is retroactively credited to the new site.
    pub(super) fn found_resident_daughter(
        &mut self,
        from: usize,
        candidate: &Candidate,
        target: f32,
    ) -> bool {
        if !self.individual_demography_enabled()
            || self.sites.len() >= LIMIT
            || !target.is_finite()
            || target < 40.
            || self.sites[from].abandoned
            || candidate.island != self.sites[from].island
            || self.sites.iter().any(|s| s.cell == candidate.cell)
        {
            return false;
        }
        let Some(society) = &self.society else {
            return false;
        };
        let mut households = BTreeSet::new();
        let mut people = vec![];
        let mut ages = [0f32; 3];
        for household in &society.households {
            if household.site != from as u32
                || !self.household_available_for_relocation(household.id)
                || self
                    .civilizations
                    .iter()
                    .any(|c| c.leader == household.head)
            {
                continue;
            }
            let roster = self.household_resident_roster(household.id);
            if roster.is_empty() || people.len() + roster.len() > target.min(90.) as usize {
                continue;
            }
            for &id in &roster {
                ages[age_band(self.month, self.people[id as usize].born).unwrap()] += 1.;
            }
            people.extend(roster);
            households.insert(household.id);
        }
        let population = people.len() as f32;
        let source = &self.sites[from];
        let food = population * 18. * 12.;
        if population < 40.
            || ages[1] < 1.
            || population >= source.stocks.stock[0]
            || source.stocks.stock[1] < food
            || ages.iter().zip(source.demography.ages).any(|(n, a)| *n > a)
        {
            return false;
        }
        let origin_share: f64 = society
            .households
            .iter()
            .filter(|h| h.site == from as u32 && !households.contains(&h.id))
            .map(|h| h.share)
            .sum();
        let moved_share: f64 = society
            .households
            .iter()
            .filter(|h| households.contains(&h.id))
            .map(|h| h.share)
            .sum();
        if origin_share <= 0. || moved_share <= 0. {
            return false;
        }
        let fraction = population / source.stocks.stock[0];
        let civilization = source.civilization;
        let cash = if self.version == 2 {
            source.economy.finance[0] * fraction
        } else {
            0.
        };
        let source = &mut self.sites[from];
        source.stocks.stock[0] -= population;
        source.stocks.people[3] += population;
        source.stocks.stock[1] -= food;
        source.economy.finance[0] -= cash;
        for (a, n) in source.demography.ages.iter_mut().zip(ages) {
            *a -= n;
        }
        self.found(candidate, civilization, population, food);
        let to = self.sites.len() - 1;
        let site = &mut self.sites[to];
        site.stocks.people[2] += population;
        site.demography.ages[..3].copy_from_slice(&ages);
        site.economy.finance[0] = cash;
        // Existing-household arrivals bypass the anonymous household initializer;
        // seed the crop inventory by transferring existing founding provisions.
        let seeds = site.stocks.stock[1].min(population);
        site.stocks.stock[1] -= seeds;
        let south = crate::grid::cell_direction(site.cell, self.terrain_resolution)[1] < 0.;
        site.demography.crops = [0., seeds, if south { 2. } else { 8. }, 1.];
        for household in &mut self.society.as_mut().unwrap().households {
            if households.contains(&household.id) {
                household.site = to as u32;
                household.share /= moved_share;
            } else if household.site == from as u32 {
                household.share /= origin_share;
            }
        }
        self.event("migration", Some(from as u32), Some(to as u32), format!(
            "{} whole households founded a daughter village with {population:.0} existing residents, {food:.1} kg provisions and {cash:.2} communal money; ownership interests moved, household wallets, ancestry and faith retained",
            households.len()));
        let event = self.events.last_mut().unwrap();
        event
            .subjects
            .extend(households.into_iter().map(|id| ("household".into(), id)));
        event
            .subjects
            .extend(people.into_iter().map(|id| ("person".into(), id)));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::Catalog,
        config::Config,
        gpu::{ContextGpu, Generator},
        participation::Presence,
    };

    #[test]
    #[ignore = "requires hardware GPU"]
    fn founding_moves_whole_rosters_and_preserves_continuation() {
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                seed: 17,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.enable_individual_demography().unwrap();
        let c = h
            .candidates
            .iter()
            .find(|c| c.island == h.sites[0].island && h.sites.iter().all(|s| s.cell != c.cell))
            .unwrap()
            .clone();
        let before = h.clone();
        let mut unfunded = before.clone();
        unfunded.sites[0].stocks.stock[1] = 0.;
        let unfunded_before = serde_json::to_value(&unfunded).unwrap();
        assert!(!unfunded.found_resident_daughter(0, &c, 40.));
        assert_eq!(unfunded_before, serde_json::to_value(&unfunded).unwrap());
        let mut absent = before.clone();
        for f in &before.society.as_ref().unwrap().households {
            absent.person_duties.insert(
                f.head,
                crate::participation::TravelDuty {
                    voyage: 0,
                    origin: f.site,
                    household: Some(f.id),
                },
            );
        }
        let absent_before = serde_json::to_value(&absent).unwrap();
        assert!(!absent.found_resident_daughter(0, &c, 40.));
        assert_eq!(absent_before, serde_json::to_value(&absent).unwrap());
        assert!(h.found_resident_daughter(0, &c, 40.));
        let to = before.sites.len();
        let moved = h.sites[to].stocks.stock[0];
        assert!((40. ..=90.).contains(&moved));
        assert_eq!(h.people.len(), before.people.len());
        assert_eq!(
            serde_json::to_value(&h.people).unwrap(),
            serde_json::to_value(&before.people).unwrap()
        );
        assert_eq!(
            h.society.as_ref().unwrap().households.len(),
            before.society.as_ref().unwrap().households.len()
        );
        assert_eq!(
            h.sites[0].stocks.stock[0] + moved,
            before.sites[0].stocks.stock[0]
        );
        assert!((h.food_residual() - before.food_residual()).abs() < 1e-6);
        for band in 0..3 {
            assert_eq!(
                h.sites[0].demography.ages[band] + h.sites[to].demography.ages[band],
                before.sites[0].demography.ages[band]
            );
        }
        let cash = |h: &History| {
            h.sites
                .iter()
                .map(|s| s.economy.finance[0] as f64)
                .sum::<f64>()
        };
        assert!((cash(h) - cash(&before)).abs() < 0.01);
        for f in &h.society.as_ref().unwrap().households {
            let original = &before.society.as_ref().unwrap().households[f.id as usize];
            let roster = before.household_resident_roster(f.id);
            for id in roster {
                assert_eq!(h.person_presence(id).1, Presence::Resident(f.site));
            }
            assert_eq!(f.head, original.head);
        }
        for site in [0, to] {
            let shares: f64 = h
                .society
                .as_ref()
                .unwrap()
                .households
                .iter()
                .filter(|f| f.site == site as u32)
                .map(|f| f.share)
                .sum();
            assert!((shares - 1.).abs() < 1e-9);
        }
        let mut replay: History =
            serde_json::from_value(serde_json::to_value(before).unwrap()).unwrap();
        assert!(replay.found_resident_daughter(0, &c, 40.));
        assert_eq!(
            serde_json::to_value(&replay).unwrap(),
            serde_json::to_value(&*h).unwrap()
        );
        // Newly populated sites must survive normal initialization without inventing
        // household heads, then reproduce both checkpoint and monthly/batch schedules.
        g.advance_history(1).unwrap();
        let path =
            std::env::temp_dir().join(format!("daughter-roster-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        g.advance_history(12).unwrap();
        for _ in 0..12 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
        );
    }
}
