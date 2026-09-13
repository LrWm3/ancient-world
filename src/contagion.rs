//! One toy SEIR infection, partitioning existing people rather than creating a population.
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

const TRANSMISSION_RATE_PER_MONTH: f64 = 0.8;
const EXPOSED_PROGRESSION_FRACTION_PER_MONTH: f64 = 0.6;
const RECOVERY_FRACTION_PER_MONTH: f64 = 0.5;
const IMMUNITY_LOSS_FRACTION_PER_MONTH: f64 = 0.01;
const INFECTIOUS_HEALTH_BURDEN_SCALE: f64 = 0.5;
const INITIAL_EXPOSED_PEOPLE: f64 = 1.0;
const CARGO_CONTACT_PEOPLE: f64 = 2.0;
const CARGO_EXPOSURE_DECAY_PER_MONTH: f64 = 0.35;
const MIN_PARTITION_POPULATION: f64 = 1e-12;
const INVENTORY_RELATIVE_TOLERANCE: f64 = 1e-5;
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pool {
    pub seir: [f64; 4],
    pub added: f64,
    pub removed: f64,
    pub incoming: f64,
    pub outgoing: f64,
}
impl Pool {
    pub fn total(&self) -> f64 {
        self.seir.iter().sum()
    }
    pub fn reconcile(&mut self, population: f64) {
        let old = self.total();
        if population >= old {
            self.seir[0] += population - old;
            self.added += population - old;
        } else if old > 0. {
            for v in &mut self.seir {
                *v *= population / old;
            }
            self.removed += old - population;
        }
    }
    pub fn expose(&mut self, amount: f64) -> f64 {
        let n = amount.max(0.).min(self.seir[0]);
        self.seir[0] -= n;
        self.seir[1] += n;
        n
    }
    pub fn advance(&mut self) {
        // Monthly bounded transitions all use the completed opening snapshot.
        let old = self.seir;
        let exposure =
            old[0] * (1. - (-TRANSMISSION_RATE_PER_MONTH * old[2] / self.total().max(1.)).exp());
        let infectious = old[1] * EXPOSED_PROGRESSION_FRACTION_PER_MONTH;
        let recovered = old[2] * RECOVERY_FRACTION_PER_MONTH;
        let waned = old[3] * IMMUNITY_LOSS_FRACTION_PER_MONTH;
        self.seir = [
            old[0] - exposure + waned,
            old[1] + exposure - infectious,
            old[2] + infectious - recovered,
            old[3] + recovered - waned,
        ];
    }
    pub fn take(&mut self, people: f64) -> Pool {
        let share = (people / self.total().max(MIN_PARTITION_POPULATION)).clamp(0., 1.);
        let mut out = Self::default();
        for k in 0..4 {
            out.seir[k] = self.seir[k] * share;
            self.seir[k] -= out.seir[k];
        }
        self.outgoing += out.total();
        out.incoming = out.total();
        out
    }
    pub fn receive(&mut self, other: &Pool) {
        self.incoming += other.total();
        for k in 0..4 {
            self.seir[k] += other.seir[k];
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Exposure {
    pub observed: u32,
    pub infectious: f64,
    pub exposed: f64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Contagion {
    pub month: Option<u32>,
    pub sites: Vec<Pool>,
    pub seeded: bool,
    pub imported_exposure: f64,
    pub travel_deaths: f64,
}
impl History {
    pub(crate) fn contagion_month(&mut self) {
        let Some(mut d) = self.contagion.take() else {
            return;
        };
        if d.month.is_some_and(|m| m >= self.month) {
            self.contagion = Some(d);
            return;
        }
        d.sites.resize_with(self.sites.len(), Pool::default);
        for (pool, site) in d.sites.iter_mut().zip(&self.sites) {
            pool.reconcile(site.stocks.stock[0] as f64);
        }
        if !d.seeded && !d.sites.is_empty() {
            d.sites[0].expose(INITIAL_EXPOSED_PEOPLE);
            d.seeded = true;
            self.event(
                "infection_baseline",
                Some(0),
                None,
                "A toy infection baseline begins with one exposed resident; population unchanged"
                    .into(),
            );
        }
        for pool in &mut d.sites {
            pool.advance();
        }
        if let Some(s) = &mut self.society {
            for j in &mut s.relocation.journeys {
                if let Some(pool) = &mut j.infection {
                    pool.advance();
                }
            }
        }
        for (pool, site) in d.sites.iter().zip(&mut self.sites) {
            // Lower bound on the existing remembered burden, not a second death/labor debit.
            let burden =
                (INFECTIOUS_HEALTH_BURDEN_SCALE * pool.seir[2] / pool.total().max(1.)) as f32;
            site.demography.health[0] = site.demography.health[0].max(burden);
        }
        d.month = Some(self.month);
        self.contagion = Some(d);
    }
    pub(crate) fn infection_departure(&mut self, from: u32, people: f32) -> Option<Pool> {
        let d = self.contagion.as_mut()?;
        d.sites.resize_with(self.sites.len(), Pool::default);
        let pool = &mut d.sites[from as usize];
        // Called after the authoritative demographic debit.
        pool.reconcile(self.sites[from as usize].stocks.stock[0] as f64 + people as f64);
        Some(pool.take(people as f64))
    }
    pub(crate) fn infection_travel_losses(&mut self, j: &mut crate::relocation::Journey) {
        let pop = j.population() as f64;
        if let (Some(d), Some(pool)) = (&mut self.contagion, &mut j.infection) {
            d.travel_deaths += (pool.total() - pop).max(0.);
            pool.reconcile(pop);
        }
    }
    pub(crate) fn infection_arrival(&mut self, to: u32, mut pool: Pool, pop: f32) {
        let Some(d) = &mut self.contagion else { return };
        pool.reconcile(pop as f64);
        d.sites.resize_with(self.sites.len(), Pool::default);
        // Called after the authoritative arrival; do not count arrivals as new susceptibles too.
        d.sites[to as usize]
            .reconcile((self.sites[to as usize].stocks.stock[0] - pop).max(0.) as f64);
        d.sites[to as usize].receive(&pool);
    }
    pub(crate) fn capture_infection_contacts(&mut self) {
        let Some(d) = &mut self.contagion else { return };
        d.sites.resize_with(self.sites.len(), Pool::default);
        for (pool, site) in d.sites.iter_mut().zip(&self.sites) {
            pool.reconcile(site.stocks.stock[0] as f64);
        }
        for cargo in &mut self.cargo {
            if cargo.infection.is_none() {
                let pool = &d.sites[cargo.from as usize];
                cargo.infection = Some(Exposure {
                    observed: self.month,
                    infectious: pool.seir[2] / pool.total().max(1.),
                    exposed: pool.seir[1] / pool.total().max(1.),
                });
            }
        }
    }
    pub(crate) fn infectious_contact(&mut self, to: u32, exposure: &Exposure) {
        let Some(d) = &mut self.contagion else { return };
        d.sites.resize_with(self.sites.len(), Pool::default);
        let age = self.month.saturating_sub(exposure.observed) as f64;
        // A bounded transport-attendant contact, not organisms carried by the goods.
        // Delay reduces viable exposure; no source state is refreshed on arrival.
        let dose = CARGO_CONTACT_PEOPLE
            * (exposure.infectious + exposure.exposed)
            * (-CARGO_EXPOSURE_DECAY_PER_MONTH * age).exp();
        d.sites[to as usize].reconcile(self.sites[to as usize].stocks.stock[0] as f64);
        d.imported_exposure += d.sites[to as usize].expose(dose);
    }
    pub fn introduce_infection(&mut self, site: u32, exposed: f64) -> Result<()> {
        ensure!(
            (site as usize) < self.sites.len() && exposed.is_finite() && exposed > 0.,
            "invalid outbreak"
        );
        let d = self.contagion.get_or_insert_with(Default::default);
        d.sites.resize_with(self.sites.len(), Pool::default);
        d.sites[site as usize].reconcile(self.sites[site as usize].stocks.stock[0] as f64);
        let actual = d.sites[site as usize].expose(exposed);
        d.seeded = true;
        self.event(
            "infection_introduced",
            Some(site),
            None,
            format!("Scenario exposed {actual:.3} existing residents; no population added"),
        );
        Ok(())
    }
}
impl Contagion {
    pub(crate) fn validate(&self, h: &History) -> Result<()> {
        ensure!(
            self.month.is_none_or(|m| m <= h.month)
                && self.sites.len() <= h.sites.len()
                && self.imported_exposure.is_finite()
                && self.imported_exposure >= 0.,
            "invalid contagion clock"
        );
        ensure!(
            self.travel_deaths.is_finite() && self.travel_deaths >= 0.,
            "invalid travel illness ledger"
        );
        for c in &h.cargo {
            if let Some(e) = &c.infection {
                ensure!(
                    e.observed <= h.month
                        && [e.infectious, e.exposed]
                            .iter()
                            .all(|v| v.is_finite() && (0. ..=1.).contains(v)),
                    "invalid traveler exposure"
                );
            }
        }
        for p in &self.sites {
            validate_pool(p)?;
        }
        for j in h.society.iter().flat_map(|s| &s.relocation.journeys) {
            if let Some(p) = &j.infection {
                validate_pool(p)?;
            }
        }
        Ok(())
    }
}
fn validate_pool(p: &Pool) -> Result<()> {
    ensure!(
        p.seir.iter().all(|v| v.is_finite() && *v >= 0.)
            && p.added.is_finite()
            && p.added >= 0.
            && p.removed.is_finite()
            && p.removed >= 0.
            && p.incoming.is_finite()
            && p.incoming >= 0.
            && p.outgoing.is_finite()
            && p.outgoing >= 0.
            && (p.total() - (p.added - p.removed + p.incoming - p.outgoing)).abs()
                < INVENTORY_RELATIVE_TOLERANCE * p.total().max(1.),
        "invalid infection inventory"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seir_transitions_and_travel_conserve_people() {
        let mut p = Pool::default();
        p.reconcile(100.);
        p.expose(10.);
        let mut traveler = p.take(20.);
        assert!((p.total() + traveler.total() - 100.).abs() < 1e-10);
        for _ in 0..1200 {
            p.advance();
            traveler.advance();
        }
        p.receive(&traveler);
        assert!((p.total() - 100.).abs() < 1e-9);
        p.reconcile(90.);
        assert!((p.total() - 90.).abs() < 1e-9);
        assert!((p.removed - 10.).abs() < 1e-9);
        validate_pool(&p).unwrap();
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn disease_contact_has_a_delayed_mediator_and_exact_continuation() {
        let mut g = crate::continuity_fixture::world();
        let h = g.civilizations.as_mut().unwrap();
        h.introduce_infection(0, 20.).unwrap();
        let pop = h.sites[0].stocks.stock[0];
        h.month += 1;
        h.contagion_month();
        assert_eq!(h.sites[0].stocks.stock[0], pop);
        assert!(h.sites[0].demography.health[0] > 0.);
        let before = h.contagion.as_ref().unwrap().sites[1].seir[1];
        h.infectious_contact(
            1,
            &Exposure {
                observed: h.month - 1,
                infectious: 0.5,
                exposed: 0.,
            },
        );
        assert!(h.contagion.as_ref().unwrap().sites[1].seir[1] > before);
        let mut resumed: History = serde_json::from_slice(&serde_json::to_vec(h).unwrap()).unwrap();
        for _ in 0..24 {
            h.month += 1;
            resumed.month += 1;
            h.contagion_month();
            resumed.contagion_month();
        }
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
    }
}

impl crate::gpu::Generator {
    pub fn set_contagion(&mut self, enabled: bool) -> Result<()> {
        self.validate_living_boundary()?;
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        if enabled == h.contagion.is_some() {
            return Ok(());
        }
        if enabled {
            h.contagion.get_or_insert_with(Default::default);
        } else {
            h.contagion = None;
        }
        // Existing travel is not assigned invented historical exposure on reactivation.
        for c in &mut h.cargo {
            c.infection = None;
        }
        if let Some(s) = &mut h.society {
            for j in &mut s.relocation.journeys {
                j.infection = None;
            }
        }
        Ok(())
    }
    pub fn introduce_infection(&mut self, site: u32, exposed: f64) -> Result<()> {
        self.validate_living_boundary()?;
        self.civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?
            .introduce_infection(site, exposed)
    }
}
#[cfg(test)]
mod integration {
    #[test]
    #[ignore = "requires hardware GPU"]
    fn infectious_cargo_and_monthly_continuation_use_actual_arrivals() {
        let mut g = crate::continuity_fixture::world();
        let h = g.civilizations.as_mut().unwrap();
        h.introduce_infection(0, 20.).unwrap();
        // A dispatched parcel with a previously observed infectious attendant.
        h.sites[0].stocks.stock[1] -= 1.;
        h.sites[1].economy.finance[0] -= 1.;
        h.sites[0].economy.finance[0] += 1.;
        h.cargo.push(crate::economy::Cargo {
            infection: Some(super::Exposure {
                observed: h.month,
                infectious: 0.5,
                exposed: 0.,
            }),
            voyage_clock: None,
            freight_stops: vec![],
            freight_edges: vec![],
            from: 0,
            to: 1,
            good: crate::economy::FOOD as u32,
            kg: 1.,
            paid: 1.,
            arrives: h.month + 1,
            sea_lane: None,
            weather_delay_months: 0,
        });
        let path = std::path::Path::new("output/contagion-continuation.world");
        g.save(path).unwrap();
        let gpu = pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap();
        let mut resumed = crate::gpu::Generator::load(gpu, path).unwrap();
        g.advance_history(12).unwrap();
        for _ in 0..12 {
            resumed.advance_history(1).unwrap();
        }
        let h = g.civilizations.as_ref().unwrap();
        assert!(h.contagion.as_ref().unwrap().imported_exposure > 0.);
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
        );
        h.validate(&g.snapshot().unwrap()).unwrap();
    }
}
