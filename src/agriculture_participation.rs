//! Shared farming/extraction pilot: GPU request -> named attendance -> bounded production.
use crate::{
    civilization::{History, ProductionLaborForecast},
    participation::{Activity, Presence},
    resolution::{Boundary, Metric, Mode, Receipt, System},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Agriculture {
    #[serde(default)]
    pub extraction: bool,
    #[serde(default)]
    pub construction: bool,
    pub plans: Vec<FarmPlan>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FarmPlan {
    /// 0 farming, 1 forestry, 2 mining, 3 construction; legacy plans are farming.
    #[serde(default)]
    pub sector: usize,
    pub month: u32,
    pub site: u32,
    pub requested: f32,
    pub assignments: Vec<FarmAssignment>,
    pub settled: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FarmAssignment {
    pub person: u32,
    pub household: u32,
    pub commitment: u32,
    pub granted: f32,
    pub used: f32,
}
impl FarmPlan {
    pub fn granted(&self) -> f32 {
        self.assignments.iter().map(|a| a.granted).sum()
    }
}
impl History {
    pub fn agriculture_refinement_enabled(&self) -> bool {
        self.resolution
            .as_ref()
            .is_some_and(|r| r.agriculture.is_some())
    }
    pub fn extraction_refinement_enabled(&self) -> bool {
        self.resolution
            .as_ref()
            .and_then(|r| r.agriculture.as_ref())
            .is_some_and(|a| a.extraction)
    }
    pub fn set_construction_refinement(&mut self, enabled: bool) -> Result<()> {
        ensure!(
            !enabled || self.extraction_refinement_enabled(),
            "construction participation requires extraction participation"
        );
        if let Some(a) = self
            .resolution
            .as_mut()
            .and_then(|r| r.agriculture.as_mut())
        {
            ensure!(
                a.plans.iter().all(|p| p.settled),
                "production work remains unsettled"
            );
            a.construction = enabled;
        }
        for site in &mut self.sites {
            site.economy.construction_workers = [0.; 4];
        }
        Ok(())
    }
    pub fn set_extraction_refinement(&mut self, enabled: bool) -> Result<()> {
        let a = self
            .resolution
            .as_mut()
            .and_then(|r| r.agriculture.as_mut());
        let Some(a) = a else {
            ensure!(
                !enabled,
                "extraction participation requires agriculture participation"
            );
            return Ok(());
        };
        ensure!(
            a.plans.iter().all(|p| p.settled),
            "production work remains unsettled"
        );
        ensure!(
            enabled || !a.construction,
            "disable construction before extraction"
        );
        a.extraction = enabled;
        for s in &mut self.sites {
            s.economy.extraction_workers = [0.; 4];
            s.economy.farm_workers[0] = 0.;
        }
        Ok(())
    }
    pub fn set_agriculture_refinement(&mut self, enabled: bool) -> Result<()> {
        ensure!(
            !enabled
                || (self.individual_demography_enabled()
                    && self.participation.is_some()
                    && self
                        .resolution
                        .as_ref()
                        .is_some_and(|r| r.workshop_individual)),
            "agriculture refinement requires individual residents and refined workshops"
        );
        ensure!(
            self.resolution
                .as_ref()
                .and_then(|r| r.agriculture.as_ref())
                .is_none_or(|a| a.plans.iter().all(|p| p.settled)),
            "agriculture work remains unsettled"
        );
        if enabled {
            self.resolution
                .as_mut()
                .unwrap()
                .agriculture
                .get_or_insert_with(Default::default);
        } else if let Some(r) = &mut self.resolution {
            r.agriculture = None;
        }
        for s in &mut self.sites {
            s.economy.farm_workers = [0.; 4];
            s.economy.construction_workers = [0.; 4];
            s.economy.extraction_workers = [0.; 4];
        }
        Ok(())
    }
    pub(crate) fn reserve_agriculture(
        &mut self,
        forecasts: &[ProductionLaborForecast],
    ) -> Result<()> {
        ensure!(
            self.agriculture_refinement_enabled(),
            "agriculture refinement disabled"
        );
        ensure!(
            self.resolution
                .as_ref()
                .unwrap()
                .agriculture
                .as_ref()
                .unwrap()
                .plans
                .iter()
                .all(|p| p.settled),
            "unsettled agriculture plan"
        );
        ensure!(
            forecasts.len() == self.sites.len()
                && forecasts.iter().enumerate().all(|(i, f)| f.site == i as u32
                    && f.month == self.month
                    && f.sectors.iter().all(|x| x.is_finite() && *x >= 0.)),
            "stale agriculture forecast"
        );
        let mut plans = Vec::new();
        let extraction = self.extraction_refinement_enabled();
        let construction = self
            .resolution
            .as_ref()
            .unwrap()
            .agriculture
            .as_ref()
            .unwrap()
            .construction;
        for site in &mut self.sites {
            site.economy.extraction_workers = [0.; 4];
            site.economy.construction_workers = [0.; 4];
        }
        for f in forecasts {
            for sector in 0..if construction {
                4
            } else if extraction {
                3
            } else {
                1
            } {
                let site = &self.sites[f.site as usize];
                let wanted = if site.abandoned {
                    0.
                } else if sector == 0 {
                    f.sectors[0].min(site.stocks.habitat[1].max(0.) / 1.5)
                } else if sector == 3 {
                    (f.sectors[3] - f.services - f.enterprises).max(0.) * 0.2
                } else {
                    f.sectors[sector]
                };
                let pool = self.participation.as_ref().unwrap();
                let people: Vec<_> = pool
                    .residents
                    .values()
                    .filter_map(|r| {
                        let hh = r.household?;
                        let household = self.society.as_ref()?.households.get(hh as usize)?;
                        (r.presence == Presence::Resident(f.site)
                            && household.site == f.site
                            && pool.available(r.person) > 1e-6)
                            .then_some((r.person, hh, pool.available(r.person)))
                    })
                    .collect();
                let available: f32 = people.iter().map(|r| r.2).sum();
                let fraction = (wanted / available.max(1e-6)).min(1.);
                let mut plan = FarmPlan {
                    sector,
                    month: self.month,
                    site: f.site,
                    requested: wanted,
                    assignments: vec![],
                    settled: false,
                };
                let mut left = wanted;
                for (person, household, capacity) in people {
                    let grant = (capacity * fraction).min(left);
                    if let Some(commitment) = self.participation.as_mut().unwrap().reserve(
                        self.month,
                        f.site,
                        activity(sector),
                        &[person],
                        grant,
                    ) {
                        let granted = self.participation.as_ref().unwrap().commitments
                            [commitment as usize]
                            .granted;
                        plan.assignments.push(FarmAssignment {
                            person,
                            household,
                            commitment,
                            granted,
                            used: 0.,
                        });
                        left = (left - granted).max(0.);
                    }
                }
                if sector == 0 {
                    self.sites[f.site as usize].economy.farm_workers =
                        [if extraction { 2. } else { 1. }, plan.granted(), 0., wanted];
                } else if sector == 3 {
                    self.sites[f.site as usize].economy.construction_workers =
                        [1., plan.granted(), 0., wanted];
                } else {
                    self.sites[f.site as usize].economy.extraction_workers[sector - 1] =
                        plan.granted();
                }
                plans.push(plan);
            }
        }
        self.resolution
            .as_mut()
            .unwrap()
            .agriculture
            .as_mut()
            .unwrap()
            .plans = plans;
        Ok(())
    }
    /// Prepaid attendance weights. A later production shortfall does not claw back food already bought.
    pub(crate) fn agricultural_earnings(&self) -> Option<BTreeMap<usize, f64>> {
        self.production_earnings(0)
    }
    pub(crate) fn production_earnings(&self, sector: usize) -> Option<BTreeMap<usize, f64>> {
        let a = self.resolution.as_ref()?.agriculture.as_ref()?;
        if (sector > 0 && !a.extraction) || (sector == 3 && !a.construction) {
            return None;
        }
        Some(
            a.plans
                .iter()
                .filter(|p| p.month == self.month && !p.settled && p.sector == sector)
                .flat_map(|p| &p.assignments)
                .fold(BTreeMap::new(), |mut m, a| {
                    *m.entry(a.household as usize).or_default() += a.granted as f64;
                    m
                }),
        )
    }
    pub(crate) fn settle_agriculture(&mut self) -> Result<()> {
        let Some(mut a) = self.resolution.as_mut().and_then(|r| r.agriculture.take()) else {
            return Ok(());
        };
        let result = (|| {
            for p in &mut a.plans {
                if p.settled {
                    continue;
                }
                ensure!(p.month == self.month, "stale agriculture settlement");
                let granted = p.granted();
                let economy = &self.sites[p.site as usize].economy;
                let used = if p.sector == 0 {
                    economy.farm_workers[2]
                } else if p.sector == 3 {
                    economy.construction_workers[2]
                } else {
                    economy.extraction_workers[p.sector + 1]
                };
                ensure!(
                    used.is_finite() && used >= 0. && used <= granted + 1e-4,
                    "agriculture exceeded attendance"
                );
                let fraction = used.min(granted) / granted.max(1e-6);
                for row in &mut p.assignments {
                    row.used = row.granted * fraction;
                    self.participation
                        .as_mut()
                        .unwrap()
                        .settle(row.commitment, row.used)?;
                }
                let state = self.resolution.as_mut().unwrap();
                let boundary = Boundary {
                    month: self.month,
                    system: [
                        System::Agriculture,
                        System::Forestry,
                        System::Mining,
                        System::Construction,
                    ][p.sector],
                    site: p.site,
                    subject: p.site,
                    revision: crate::resolution::revision([
                        p.requested.to_bits() as u64,
                        granted.to_bits() as u64,
                    ]),
                };
                let metrics = if state.compare {
                    vec![
                        Metric {
                            name: if p.sector == 0 {
                                "agriculture_attendance"
                            } else if p.sector == 3 {
                                "construction_attendance"
                            } else {
                                "extraction_attendance"
                            }
                            .into(),
                            unit: "worker-months".into(),
                            expected: p.requested as f64,
                            actual: granted as f64,
                            explained: vec![],
                        },
                        Metric {
                            name: if p.sector == 0 {
                                "cultivation_work"
                            } else if p.sector == 3 {
                                "construction_work"
                            } else {
                                "extraction_work"
                            }
                            .into(),
                            unit: "worker-months".into(),
                            expected: granted as f64,
                            actual: used as f64,
                            explained: vec![(
                                "unused production attendance".into(),
                                (used - granted) as f64,
                            )],
                        },
                    ]
                } else {
                    vec![]
                };
                state.commit(
                    Receipt {
                        boundary: boundary.clone(),
                        mode: Mode::Individual,
                        metrics,
                        demographic_snapshot: None,
                    },
                    &boundary,
                )?;
                p.settled = true;
            }
            Ok(())
        })();
        self.resolution.as_mut().unwrap().agriculture = Some(a);
        result
    }
}

impl History {
    pub(crate) fn validate_agriculture(&self) -> Result<()> {
        let Some(a) = self
            .resolution
            .as_ref()
            .and_then(|r| r.agriculture.as_ref())
        else {
            return Ok(());
        };
        ensure!(
            self.individual_demography_enabled()
                && self.participation.is_some()
                && self.resolution.as_ref().unwrap().workshop_individual
                && (!a.construction || a.extraction),
            "agriculture lost individual authority"
        );
        let mut sites = std::collections::BTreeSet::new();
        let mut commitments = std::collections::BTreeSet::new();
        for p in &a.plans {
            ensure!(
                p.month <= self.month
                    && (p.site as usize) < self.sites.len()
                    && p.sector < 4
                    && sites.insert((p.site, p.sector))
                    && p.requested.is_finite()
                    && p.requested >= 0.
                    && p.granted() <= p.requested + 1e-4,
                "invalid agriculture plan"
            );
            for row in &p.assignments {
                ensure!(
                    (row.person as usize) < self.people.len()
                        && (row.household as usize)
                            < self.society.as_ref().unwrap().households.len()
                        && row.granted.is_finite()
                        && row.granted > 0.
                        && row.used.is_finite()
                        && row.used >= 0.
                        && row.used <= row.granted + 1e-5
                        && commitments.insert(row.commitment),
                    "invalid agricultural assignment"
                );
                let pool = self.participation.as_ref().unwrap();
                if pool.month == Some(p.month) {
                    let c = pool
                        .commitments
                        .get(row.commitment as usize)
                        .ok_or_else(|| anyhow::anyhow!("missing agricultural commitment"))?;
                    ensure!(
                        c.activity == activity(p.sector)
                            && c.site == p.site
                            && c.people == vec![(row.person, row.granted)]
                            && c.settled == p.settled
                            && (c.used - row.used).abs() < 1e-5,
                        "agriculture commitment mismatch"
                    );
                }
            }
        }
        Ok(())
    }
}

fn activity(sector: usize) -> Activity {
    [
        Activity::Agriculture,
        Activity::Forestry,
        Activity::Mining,
        Activity::Construction,
    ][sector]
}
