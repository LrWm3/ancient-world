//! Bounded social observations and pressure memory. Population remains in Demography/Journey.
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const NO_TRADITION: u32 = u32::MAX;
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SocialCell {
    /// Age (child/adult/elder) × farming, forestry, mining, craft, other/care. Derived people equivalents.
    pub livelihoods: [[f32; 5]; 3],
    /// Household counts in relative-ownership bins (<1/8,1/4,1/2,1,2,4,8,>=8 times mean).
    pub ownership: [f32; 8],
    /// Resident households by cash purchasing power: <0.125,0.25,0.5,1,2,4,8,>=8 months of food.
    #[serde(default)]
    pub cash_buffer: [f32; 8],
    /// Households with latest observed shortages <=2%, <=10%, <=30%, >30%.
    #[serde(default)]
    pub food_security: [f32; 4],
    /// Observed mean hunger, severe share, remembered severe share, observed household count.
    #[serde(default)]
    pub household_stress: [f32; 4],
    /// High months, recovery months, episode active, last monthly sample.
    #[serde(default)]
    pub distribution_status: [u32; 4],
    /// Sustained hunger, disease, disruption, ownership inequality: 0–1.
    pub pressure: [f32; 4],
    /// Shelter capacity, current unsheltered fraction, remembered crowding, episode active.
    #[serde(default)]
    pub housing: [f32; 4],
    /// Four most prevalent tradition IDs; remaining traditions are grouped in faith[4].
    pub traditions: [u32; 4],
    pub faith: [f32; 5],
    /// Existing three faction interests; a descriptive local share, not extra votes.
    pub factions: [f32; 3],
    /// Latest measured shortage, disease burden, disruption, ownership Gini.
    pub exposure: [f32; 4],
    /// Consecutive high/low pressure months, crisis flag, latest observation month.
    pub status: [u32; 4],
}
impl Default for SocialCell {
    fn default() -> Self {
        Self {
            livelihoods: [[0.; 5]; 3],
            ownership: [0.; 8],
            cash_buffer: [0.; 8],
            food_security: [0.; 4],
            household_stress: [0.; 4],
            distribution_status: [0; 4],
            pressure: [0.; 4],
            housing: [0.; 4],
            traditions: [NO_TRADITION; 4],
            faith: [0.; 5],
            factions: [0.; 3],
            exposure: [0.; 4],
            status: [0; 4],
        }
    }
}
impl SocialCell {
    pub fn migration_pressure(&self) -> f32 {
        self.pressure[0]
            .max(self.household_stress[2])
            .max(self.housing[2])
            * 0.7
            + self.pressure[2] * 0.3
    }
    pub fn morale(&self) -> f32 {
        1. - (self.pressure[0] * 0.45
            + self.pressure[1] * 0.2
            + self.pressure[2] * 0.25
            + self.pressure[3] * 0.1)
    }
    fn accumulate(&mut self) {
        for k in 0..4 {
            let rate = if self.exposure[k] > self.pressure[k] {
                0.18
            } else {
                0.08
            };
            self.pressure[k] =
                (self.pressure[k] + (self.exposure[k] - self.pressure[k]) * rate).clamp(0., 1.);
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocialState {
    pub started: u32,
    pub observed: u32,
    pub sites: Vec<SocialCell>,
}
impl SocialState {
    pub fn validate(&self, h: &History) -> Result<()> {
        ensure!(
            self.started <= self.observed
                && self.observed <= h.month
                && self.sites.len() <= h.sites.len(),
            "invalid social observation clock"
        );
        for c in &self.sites {
            ensure!(
                c.livelihoods
                    .iter()
                    .flatten()
                    .chain(&c.ownership)
                    .chain(&c.cash_buffer)
                    .chain(&c.food_security)
                    .chain(&c.household_stress)
                    .chain(&c.housing)
                    .all(|x| x.is_finite() && *x >= 0.),
                "invalid social distribution"
            );
            ensure!(
                c.housing[1..].iter().all(|v| (0. ..=1.).contains(v)),
                "invalid housing pressure"
            );
            ensure!(
                c.pressure
                    .iter()
                    .chain(&c.exposure)
                    .chain(&c.faith)
                    .chain(&c.factions)
                    .all(|x| x.is_finite() && (0. ..=1.).contains(x)),
                "invalid social pressure/share"
            );
            ensure!(
                c.household_stress[..3].iter().all(|v| *v <= 1.)
                    && c.distribution_status[2] <= 1
                    && c.distribution_status[3] <= self.observed
                    && (c.food_security.iter().sum::<f32>() - c.household_stress[3]).abs() < 0.01,
                "invalid household deprivation summary"
            );
            ensure!(
                c.status[2] <= 1 && c.status[3] <= self.observed,
                "invalid social status"
            );
            ensure!(
                c.faith.iter().sum::<f32>() <= 1.00001 && c.factions.iter().sum::<f32>() <= 1.00001,
                "invalid social prevalence totals"
            );
            ensure!(
                c.traditions.iter().all(|&id| id == NO_TRADITION
                    || h.culture
                        .as_ref()
                        .is_some_and(|culture| (id as usize) < culture.traditions.len())),
                "unknown observed tradition"
            );
        }
        Ok(())
    }
}
impl History {
    pub fn social_indicators(&self, site: u32) -> Option<&SocialCell> {
        self.society
            .as_ref()?
            .indicators
            .as_ref()?
            .sites
            .get(site as usize)
    }
    pub(crate) fn social_indicators_month(&mut self) {
        let Some(mut state) = self.society.as_mut().and_then(|s| s.indicators.take()) else {
            return;
        };
        // Repeated boundary reads refresh projections but do not age pressure a second time.
        let elapsed = self.month > state.observed;
        state.sites.resize(self.sites.len(), SocialCell::default());
        let society = self.society.as_ref().unwrap();
        let mut shares = vec![vec![]; self.sites.len()];
        let mut accounts = vec![vec![]; self.sites.len()];
        let mut faith = vec![BTreeMap::<u32, f64>::new(); self.sites.len()];
        let mut factions = vec![[0f64; 3]; self.sites.len()];
        for hh in &society.households {
            if society.relocation.away(hh.id) {
                continue;
            }
            let site = hh.site as usize;
            shares[site].push(hh.share);
            if let Some(a) = society
                .household_economy
                .as_ref()
                .and_then(|e| e.accounts.get(hh.id as usize))
            {
                accounts[site].push(a);
            }
            if let Some(id) = self
                .culture
                .as_ref()
                .and_then(|c| c.household_faith.get(hh.id as usize))
            {
                *faith[site].entry(*id).or_default() += 1.;
            }
            if let Some(id) = self
                .politics
                .as_ref()
                .and_then(|p| p.household_factions.get(hh.id as usize))
            {
                factions[site][*id as usize % 3] += hh.share;
            }
        }
        let mut events = vec![];
        for (i, site) in self.sites.iter().enumerate() {
            let c = &mut state.sites[i];
            c.livelihoods = [[0.; 5]; 3];
            for age in 0..3 {
                c.livelihoods[age][4] = site.demography.ages[age];
            }
            let adults = site.demography.ages[1];
            let labor = site.economy.labor;
            let total = labor.iter().sum::<f32>();
            let scale = (adults / total.max(0.000001)).min(1.);
            for (k, workers) in labor.iter().enumerate() {
                c.livelihoods[1][k] = workers * scale;
            }
            c.livelihoods[1][4] = (adults - c.livelihoods[1][..4].iter().sum::<f32>()).max(0.);
            c.ownership = [0.; 8];
            shares[i].sort_by(f64::total_cmp);
            let n = shares[i].len();
            let sum = shares[i].iter().sum::<f64>();
            let mut gini = 0.;
            for (j, share) in shares[i].iter().enumerate() {
                let relative = share * n as f64 / sum.max(1e-12);
                let bin = [0.125, 0.25, 0.5, 1., 2., 4., 8.]
                    .iter()
                    .position(|b| relative < *b)
                    .unwrap_or(7);
                c.ownership[bin] += 1.;
                gini += (2. * (j + 1) as f64 - n as f64 - 1.) * share;
            }
            c.cash_buffer = [0.; 8];
            c.food_security = [0.; 4];
            c.household_stress[0] = 0.;
            c.household_stress[1] = 0.;
            c.household_stress[3] = 0.;
            let monthly_need = site.demography.ages[..3]
                .iter()
                .zip([10., 18., 14.])
                .map(|(a, r)| *a as f64 * r)
                .sum::<f64>()
                / (n.max(1) as f64);
            let cost = monthly_need * site.economy.prices[crate::economy::FOOD].max(0.01) as f64;
            for a in &accounts[i] {
                if cost > 0. {
                    let months = a.cash / cost;
                    let bin = [0.125, 0.25, 0.5, 1., 2., 4., 8.]
                        .iter()
                        .position(|v| months < *v)
                        .unwrap_or(7);
                    c.cash_buffer[bin] += 1.;
                }
                // Arrival after consumption must not relabel an origin-town food observation.
                if a.food_site == Some(site.id)
                    && a.need > 0.
                    && society
                        .household_economy
                        .as_ref()
                        .is_some_and(|e| e.observed == self.month)
                {
                    let bin = [0.02, 0.1, 0.3]
                        .iter()
                        .position(|v| a.hunger <= *v)
                        .unwrap_or(3);
                    c.food_security[bin] += 1.;
                    c.household_stress[0] += a.hunger as f32;
                    c.household_stress[3] += 1.;
                }
            }
            if c.household_stress[3] > 0. {
                c.household_stress[0] =
                    (c.household_stress[0] / c.household_stress[3]).clamp(0., 1.);
                c.household_stress[1] = c.food_security[3] / c.household_stress[3];
            }
            if elapsed && c.household_stress[3] > 0. {
                let severe = c.household_stress[1];
                let rate = if severe > c.household_stress[2] {
                    0.18
                } else {
                    0.08
                };
                c.household_stress[2] =
                    (c.household_stress[2] + rate * (severe - c.household_stress[2])).clamp(0., 1.);
                let d = &mut c.distribution_status;
                d[0] = if c.household_stress[2] >= 0.25 {
                    d[0].saturating_add(1)
                } else {
                    0
                };
                d[1] = if c.household_stress[2] < 0.1 {
                    d[1].saturating_add(1)
                } else {
                    0
                };
                if d[2] == 0 && d[0] >= 3 {
                    d[2] = 1;
                    events.push((i,"household_deprivation",format!("Persistent household deprivation: {:.0} of {:.0} observed households have more than 30% unmet food need; mean household shortage {:.0}%",c.food_security[3],c.household_stress[3],c.household_stress[0]*100.)));
                } else if d[2] == 1 && d[1] >= 6 {
                    d[2] = 0;
                    events.push((i,"household_deprivation_recovery","Six observed months below the persistent household-deprivation threshold; this does not imply every household is secure".into()));
                }
                d[3] = self.month;
            }
            c.faith = [0.; 5];
            c.traditions = [NO_TRADITION; 4];
            let mut ranked = faith[i]
                .iter()
                .map(|(&id, &share)| (id, share))
                .collect::<Vec<_>>();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            let faith_total = ranked.iter().map(|x| x.1).sum::<f64>();
            for (j, (id, share)) in ranked.iter().enumerate() {
                if j < 4 {
                    c.traditions[j] = *id;
                }
                c.faith[j.min(4)] += (*share / faith_total.max(1e-12)) as f32;
            }
            // Roundoff cannot make a prevalence exceed one.
            for x in &mut c.faith {
                *x = x.clamp(0., 1.);
            }
            let faction_total = factions[i].iter().sum::<f64>();
            for k in 0..3 {
                c.factions[k] = (factions[i][k] / faction_total.max(1e-12)) as f32;
            }
            c.housing[0] = site.economy.housing_capacity();
            c.housing[1] = site.economy.crowding(site.stocks.stock[0]);
            if elapsed {
                let rate = if c.housing[1] > c.housing[2] {
                    0.18
                } else {
                    0.08
                };
                c.housing[2] += (c.housing[1] - c.housing[2]) * rate;
                if c.housing[2] > 0.15 && c.housing[3] < 0.5 {
                    c.housing[3] = 1.;
                    events.push((i,"housing_pressure",format!("Persistent crowding: {:.0}% lack shelter capacity; {:.1} residents, {:.1} places",c.housing[1]*100.,site.stocks.stock[0],c.housing[0])));
                } else if c.housing[2] < 0.05 && c.housing[3] > 0.5 {
                    c.housing[3] = 0.;
                    events.push((
                        i,
                        "housing_recovery",
                        format!(
                            "Remembered crowding below 5%; {:.1} residents, {:.1} shelter places",
                            site.stocks.stock[0], c.housing[0]
                        ),
                    ));
                }
            }
            c.exposure = [
                site.stocks.stock[3].clamp(0., 1.),
                site.demography.health[0].clamp(0., 1.),
                site.economy.soil[3].clamp(0., 1.),
                (gini / (n as f64 * sum).max(1e-12)).clamp(0., 1.) as f32,
            ];
            if elapsed {
                c.accumulate();
                let distress = c.pressure[0].max(c.pressure[2]);
                c.status[0] = if distress > 0.55 {
                    c.status[0].saturating_add(1)
                } else {
                    0
                };
                c.status[1] = if distress < 0.25 {
                    c.status[1].saturating_add(1)
                } else {
                    0
                };
                if site.stocks.stock[0] > 0. && c.status[2] == 0 && c.status[0] >= 3 {
                    c.status[2] = 1;
                    events.push((i,"social_strain",format!("Sustained hardship: hunger pressure {:.0}%, disruption {:.0}%; current food shortage {:.0}%",c.pressure[0]*100.,c.pressure[2]*100.,c.exposure[0]*100.)));
                } else if c.status[2] == 1 && c.status[1] >= 6 {
                    c.status[2] = 0;
                    if site.stocks.stock[0] > 0. {
                        events.push((i,"social_recovery",format!("Six months below the recovery threshold; hunger pressure {:.0}%, disruption {:.0}%",c.pressure[0]*100.,c.pressure[2]*100.)));
                    }
                }
            }
            c.status[3] = self.month;
        }
        state.observed = self.month;
        self.society.as_mut().unwrap().indicators = Some(state);
        for (site, kind, detail) in events {
            let cause = self
                .events
                .iter()
                .rev()
                .find(|e| {
                    if kind.starts_with("housing_") {
                        return e.site == Some(site as u32) && e.kind.starts_with("housing_");
                    }
                    e.site == Some(site as u32)
                        && matches!(
                            e.kind.as_str(),
                            "famine"
                                | "shortage"
                                | "flood"
                                | "social_strain"
                                | "social_recovery"
                                | "food_access_crisis"
                                | "food_access_recovery"
                                | "household_deprivation"
                                | "household_deprivation_recovery"
                        )
                })
                .map(|e| e.id);
            self.event(kind, Some(site as u32), None, detail);
            if let Some(cause) = cause {
                self.events.last_mut().unwrap().causes.push(cause);
            }
        }
    }
}
impl crate::gpu::Generator {
    pub fn enable_social_indicators(&mut self) -> Result<()> {
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "social baseline requires a completed boundary"
        );
        self.validate_living_boundary()?;
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let society = h
            .society
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable society first"))?;
        ensure!(
            society.indicators.is_none(),
            "social indicators already enabled"
        );
        society.indicators = Some(SocialState {
            started: h.month,
            observed: h.month,
            sites: vec![],
        });
        h.social_indicators_month();
        h.event("social_indicators_baseline",None,None,"Social pressure observation begins; population remains in existing cohorts and journeys".into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pressure_accumulates_and_recovers_without_instant_reset() {
        let mut c = SocialCell {
            exposure: [1., 0., 0., 0.],
            ..Default::default()
        };
        c.accumulate();
        assert!((c.pressure[0] - 0.18).abs() < 1e-6);
        for _ in 0..11 {
            c.accumulate();
        }
        assert!(c.pressure[0] > 0.9);
        c.exposure = [0.; 4];
        c.accumulate();
        assert!(c.pressure[0] > 0.8);
        for _ in 0..48 {
            c.accumulate();
        }
        assert!(c.pressure[0] < 0.02);
        assert!((0. ..=1.).contains(&c.morale()));
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn social_observations_conserve_projections_and_debounce_events() {
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
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let original = h
            .sites
            .iter()
            .map(|s| bytemuck::bytes_of(&s.stocks).to_vec())
            .collect::<Vec<_>>();
        h.social_indicators_month();
        let once = serde_json::to_vec(&h.society.as_ref().unwrap().indicators).unwrap();
        h.social_indicators_month();
        assert_eq!(
            once,
            serde_json::to_vec(&h.society.as_ref().unwrap().indicators).unwrap()
        );
        for (site, stock) in h.sites.iter().zip(original) {
            assert_eq!(stock, bytemuck::bytes_of(&site.stocks));
            let c = h.social_indicators(site.id).unwrap();
            for age in 0..3 {
                assert!(
                    (c.livelihoods[age].iter().sum::<f32>() - site.demography.ages[age]).abs()
                        < 0.0001
                );
            }
            let households = h
                .society
                .as_ref()
                .unwrap()
                .households
                .iter()
                .filter(|f| f.site == site.id)
                .count();
            assert_eq!(c.ownership.iter().sum::<f32>(), households as f32);
        }
        for _ in 0..12 {
            h.month += 1;
            h.sites[0].stocks.stock[3] = 1.;
            h.social_indicators_month();
        }
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "social_strain" && e.site == Some(0))
                .count(),
            1
        );
        let pressure = h.social_indicators(0).unwrap().pressure[0];
        assert!(pressure > 0.9);
        for _ in 0..36 {
            h.month += 1;
            h.sites[0].stocks.stock[3] = 0.;
            h.social_indicators_month();
        }
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "social_recovery" && e.site == Some(0))
                .count(),
            1
        );
        let recovery = h
            .events
            .iter()
            .find(|e| e.kind == "social_recovery" && e.site == Some(0))
            .unwrap();
        assert!(recovery
            .causes
            .iter()
            .any(|&id| h.events[id as usize].kind == "social_strain"));
        // A deprived minority is visible even when aggregate town shortage is low.
        let stocks = h.sites.iter().map(|s| s.stocks).collect::<Vec<_>>();
        for _ in 0..18 {
            h.month += 1;
            let society = h.society.as_mut().unwrap();
            let e = society.household_economy.as_mut().unwrap();
            e.observed = h.month;
            e.accounts
                .resize(society.households.len(), Default::default());
            for hh in &society.households {
                let a = &mut e.accounts[hh.id as usize];
                a.need = 100.;
                a.food_site = Some(hh.site);
                a.hunger = if hh.id % 3 == 0 { 0.4 } else { 0. };
                a.common_food = 100. * (1. - a.hunger);
            }
            h.social_indicators_month();
        }
        let c = h.social_indicators(0).unwrap();
        assert!(c.food_security[3] > 0. && c.household_stress[0] < 0.2);
        assert_eq!(c.cash_buffer.iter().sum::<f32>(), c.household_stress[3]);
        assert_eq!(c.food_security.iter().sum::<f32>(), c.household_stress[3]);
        assert!(h
            .events
            .iter()
            .any(|e| e.kind == "household_deprivation" && e.site == Some(0)));
        let before = serde_json::to_vec(&h.society.as_ref().unwrap().indicators).unwrap();
        h.social_indicators_month();
        assert_eq!(
            before,
            serde_json::to_vec(&h.society.as_ref().unwrap().indicators).unwrap()
        );
        for (s, old) in h.sites.iter().zip(stocks) {
            assert_eq!(bytemuck::bytes_of(&s.stocks), bytemuck::bytes_of(&old));
        }
        for _ in 0..42 {
            h.month += 1;
            let e = h
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap();
            e.observed = h.month;
            for a in &mut e.accounts {
                a.hunger = 0.;
                a.common_food = a.need;
            }
            h.social_indicators_month();
        }
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "household_deprivation_recovery" && e.site == Some(0))
                .count(),
            1
        );
        // A post-consumption arrival cannot bring its origin meal into destination statistics.
        let hh = h.society.as_ref().unwrap().households[0].clone();
        let before = h.social_indicators(hh.site).unwrap().household_stress[3];
        h.society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .accounts[0]
            .food_site = Some((hh.site + 1) % h.sites.len() as u32);
        h.social_indicators_month();
        assert_eq!(
            h.social_indicators(hh.site).unwrap().household_stress[3],
            before - 1.
        );
        let mut legacy = serde_json::to_value(SocialCell::default()).unwrap();
        for key in [
            "cash_buffer",
            "food_security",
            "household_stress",
            "distribution_status",
        ] {
            legacy.as_object_mut().unwrap().remove(key);
        }
        let legacy: SocialCell = serde_json::from_value(legacy).unwrap();
        assert_eq!(legacy.household_stress, [0.; 4]);
        let mut old = serde_json::to_value(h.society.as_ref().unwrap()).unwrap();
        old.as_object_mut().unwrap().remove("indicators");
        let old: crate::society::Society = serde_json::from_value(old).unwrap();
        assert!(old.indicators.is_none());
    }
}
