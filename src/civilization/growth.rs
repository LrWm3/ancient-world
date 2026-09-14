//! Opt-in growth experiments and exact annual founding admission diagnostics.
use super::{
    distance, Candidate, History, DAUGHTER_DISTANCE_SCALE_KM, DAUGHTER_MAX_DISTANCE_KM,
    DAUGHTER_MIN_ORIGIN_POPULATION, DAUGHTER_SURPLUS_THRESHOLD, DAUGHTER_YIELD_RATIO_RANGE,
    FOUNDING_PROVISION_KG_PER_PERSON_MONTH, MAX_SETTLEMENTS,
};
use crate::gpu::Cell;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const DEFAULT_RESERVE_MONTHS: f32 = 12.;
const MIN_EXPERIMENT_SCALE: f64 = 0.1;
const MAX_EXPERIMENT_SCALE: f64 = 2.;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Growth {
    pub background_mortality_scale: f64,
    pub founding_population_scale: f32,
    pub founding_reserve_months: f32,
    /// Experimental soft limit; never exceeds the currently allocated GPU capacity.
    pub settlement_limit: usize,
    pub audit_enabled: bool,
    pub review: Option<Review>,
}
impl Default for Growth {
    fn default() -> Self {
        Self {
            background_mortality_scale: 1.,
            founding_population_scale: 1.,
            founding_reserve_months: DEFAULT_RESERVE_MONTHS,
            settlement_limit: MAX_SETTLEMENTS,
            audit_enabled: false,
            review: None,
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Review {
    pub month: u32,
    pub outcomes: BTreeMap<String, usize>,
    pub sites: Vec<(u32, String)>,
}
impl Growth {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (MIN_EXPERIMENT_SCALE..=MAX_EXPERIMENT_SCALE)
                .contains(&self.background_mortality_scale)
                && (MIN_EXPERIMENT_SCALE as f32..=MAX_EXPERIMENT_SCALE as f32)
                    .contains(&self.founding_population_scale)
                && (1. ..=24.).contains(&self.founding_reserve_months)
                && (1..=MAX_SETTLEMENTS).contains(&self.settlement_limit),
            "invalid growth experiment settings"
        );
        Ok(())
    }
    pub fn mortality(&self, observed: f64, base: f64) -> f64 {
        // Keep the default arithmetic untouched. Hunger and illness increments are unchanged.
        if self.background_mortality_scale == 1. {
            observed
        } else {
            (observed + base * (self.background_mortality_scale - 1.)).clamp(0., 1.)
        }
    }
    pub(super) fn record(&mut self, site: u32, outcome: &str) {
        if let Some(r) = self.review.as_mut().filter(|_| self.audit_enabled) {
            *r.outcomes.entry(outcome.into()).or_default() += 1;
            r.sites.push((site, outcome.into()));
        }
    }
}
impl History {
    pub(super) fn daughter_opportunity(
        &self,
        i: usize,
        radius: f32,
        terrain: &[Cell],
    ) -> std::result::Result<Candidate, &'static str> {
        let s = &self.sites[i];
        if s.abandoned {
            return Err("abandoned_origin");
        }
        if self
            .living
            .as_ref()
            .and_then(|l| l.floods.get(&s.id))
            .is_some_and(|f| f.persistent)
        {
            return Err("flooded_origin");
        }
        if s.stocks.stock[0]
            < DAUGHTER_MIN_ORIGIN_POPULATION * self.growth.founding_population_scale
        {
            return Err("population");
        }
        if s.stocks.stock[1]
            < s.stocks.stock[0]
                * FOUNDING_PROVISION_KG_PER_PERSON_MONTH
                * self.growth.founding_reserve_months
        {
            return Err("provisions");
        }
        let nearby: Vec<_> = self
            .candidates
            .iter()
            .filter(|c| {
                c.island == s.island
                    && distance(c.cell, s.cell, self.terrain_resolution) * radius
                        < DAUGHTER_MAX_DISTANCE_KM
            })
            .collect();
        if nearby.is_empty() {
            return Err("no_reachable_candidates");
        }
        let unused: Vec<_> = nearby
            .into_iter()
            .filter(|c| self.sites.iter().all(|s| s.cell != c.cell))
            .collect();
        if unused.is_empty() {
            return Err("local_candidates_occupied");
        }
        let candidate = unused
            .into_iter()
            .filter(|c| c.available_for_founding(terrain))
            .max_by(|a, b| {
                let score = |c: &Candidate| {
                    c.yield_kg
                        / (1.
                            + distance(c.cell, s.cell, self.terrain_resolution) * radius
                                / DAUGHTER_DISTANCE_SCALE_KM)
                };
                score(a).total_cmp(&score(b))
            })
            .ok_or("candidates_currently_unsafe")?;
        let threshold = (DAUGHTER_MIN_ORIGIN_POPULATION
            + DAUGHTER_SURPLUS_THRESHOLD
                * (s.stocks.habitat[0] / candidate.yield_kg.max(1.)).clamp(
                    *DAUGHTER_YIELD_RATIO_RANGE.start(),
                    *DAUGHTER_YIELD_RATIO_RANGE.end(),
                ))
            * self.growth.founding_population_scale;
        if s.stocks.stock[0] < threshold {
            return Err("population");
        }
        if self.sites.len() >= self.growth.settlement_limit {
            return Err("settlement_cap");
        }
        Ok(candidate.clone())
    }
    /// All currently living population, including military, relocation and expedition travelers.
    pub fn growth_population(&self) -> f64 {
        self.sites
            .iter()
            .map(|s| s.stocks.stock[0] as f64)
            .sum::<f64>()
            + self.society.as_ref().map_or(0., |s| {
                s.raids.iter().map(|r| r.soldiers as f64).sum::<f64>()
                    + s.relocation
                        .journeys
                        .iter()
                        .map(|j| j.population() as f64)
                        .sum::<f64>()
            })
            + self.expeditions.as_ref().map_or(0., |x| {
                x.voyages
                    .iter()
                    .filter(|e| e.phase.active())
                    .map(|e| e.survivors() as f64)
                    .sum::<f64>()
            })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn admission_diagnostics_distinguish_population_provisions_and_cap() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
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
        let terrain = g.snapshot().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        // Use a small radius in this admission fixture to isolate non-distance gates.
        let site = (0..h.sites.len())
            .find(|&i| {
                h.candidates.iter().any(|c| {
                    c.island == h.sites[i].island
                        && h.sites.iter().all(|s| s.cell != c.cell)
                        && c.available_for_founding(&terrain)
                })
            })
            .unwrap();
        h.sites[site].stocks.stock[0] = 500.;
        h.sites[site].stocks.stock[1] = 500. * 18. * 10.;
        assert_eq!(
            h.daughter_opportunity(site, 1., &terrain).unwrap_err(),
            "provisions"
        );
        h.growth.founding_reserve_months = 9.;
        assert!(h.daughter_opportunity(site, 1., &terrain).is_ok());
        let candidates = h.candidates.len();
        h.growth.settlement_limit = h.sites.len();
        assert_eq!(
            h.daughter_opportunity(site, 1., &terrain).unwrap_err(),
            "settlement_cap"
        );
        assert_eq!(h.candidates.len(), candidates);
        h.growth.settlement_limit = MAX_SETTLEMENTS;
        h.sites[site].stocks.stock[0] = 150.;
        assert_eq!(
            h.daughter_opportunity(site, 1., &terrain).unwrap_err(),
            "population"
        );
        h.growth.founding_population_scale = 0.1;
        assert!(h.daughter_opportunity(site, 1., &terrain).is_ok());
    }
    #[test]
    fn favorable_health_preserves_external_stress_and_defaults() {
        let mut g = Growth::default();
        assert_eq!(g.mortality(0.0406, 0.0006), 0.0406);
        g.background_mortality_scale = 0.85;
        assert!((g.mortality(0.0406, 0.0006) - 0.04051).abs() < 1e-12);
        assert!((g.mortality(0.0006, 0.0006) - 0.00051).abs() < 1e-12);
        g.settlement_limit = MAX_SETTLEMENTS + 1;
        assert!(g.validate().is_err());
        let old: Growth = serde_json::from_str("{}").unwrap();
        assert_eq!(old.settlement_limit, MAX_SETTLEMENTS);
    }
}
