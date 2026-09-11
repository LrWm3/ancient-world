//! The aggregate production planner remains on GPU. Individual matching can consume
//! this bounded projection rather than reimplementing worker_shares on the CPU.
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProductionLaborForecast {
    /// Snapshot date, not a claim that a later month's inputs remain unchanged.
    pub month: u32,
    pub site: u32,
    /// Effective worker-months: farming, forestry, mining, crafting.
    pub sectors: [f32; 4],
    /// Already reserved public services and endpoint crew work within crafting.
    pub services: f32,
    /// Already requested private workshop work within crafting.
    pub enterprises: f32,
    /// Fishing from the input snapshot, not a fresh catch or fishing proposal.
    pub prior_fishing: f32,
    /// Current aggregate workforce before waterlogging and sector allocation.
    pub workforce: f32,
}
impl Engine {
    /// Call with the same completed Reserve inputs that matching will use.
    /// Reuses the production output buffer as scratch; execute must overwrite it
    /// before reading actual Stocks. Copies only 64 bytes per settlement.
    pub(super) fn forecast_labor(
        &self,
        g: &Generator,
        h: &History,
    ) -> Result<Vec<ProductionLaborForecast>> {
        if h.sites.is_empty() {
            return Ok(vec![]);
        }
        ensure!(h.sites.len() <= LIMIT, "production forecast site limit");
        self.upload(g, h);
        let mut encoder = g.gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.labor_forecast);
            pass.set_bind_group(0, &self.group, &[]);
            pass.dispatch_workgroups((h.sites.len() as u32).div_ceil(64), 1, 1);
        }
        g.gpu.queue.submit(Some(encoder.finish()));
        let bytes = read_buffer(&g.gpu, &self.output, 0, h.sites.len() as u64 * 64)?;
        bytemuck::cast_slice::<u8, Stocks>(&bytes)
            .iter()
            .zip(&h.sites)
            .map(|(row, site)| {
                ensure!(
                    row.stock
                        .iter()
                        .chain(&row.habitat)
                        .all(|x| x.is_finite() && *x >= 0.),
                    "invalid GPU labor forecast at site {}",
                    site.id
                );
                Ok(ProductionLaborForecast {
                    month: h.month,
                    site: site.id,
                    sectors: row.stock,
                    services: row.habitat[0],
                    enterprises: row.habitat[1],
                    prior_fishing: row.habitat[2],
                    workforce: row.habitat[3],
                })
            })
            .collect()
    }
}
impl Generator {
    /// Read-only labor projection from the current history snapshot. This does not
    /// advance history or reserve workers. It intentionally does not run fishing,
    /// weather wear or ecological claims; later changes can revise the projection.
    /// Interactive callers pay for a temporary engine; monthly integration can
    /// reuse Engine::forecast_labor on the existing cached engine.
    pub fn production_labor_forecast(&self) -> Result<Vec<ProductionLaborForecast>> {
        let h = self
            .civilizations
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(h.version == 2, "labor forecast requires managed production");
        Engine::new(self)?.forecast_labor(self, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn forecast_is_read_only_and_matches_fixed_policy_execution() {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        let mut h = g.civilizations.as_ref().unwrap().clone();
        h.month = 1;
        h.society = None;
        h.living = None;
        for s in &mut h.sites {
            s.stocks.stock[0] = 100.;
            s.economy = Economy::default();
            s.economy.claim = [1., 1000., 1000., 1.];
            // Fixed 62/8/10/20 allocation has an independent analytical answer.
            s.economy.logistics[3] = 3.;
        }
        let before = serde_json::to_value(&h).unwrap();
        g.civilizations = Some(h.clone());
        let engine = Engine::new(&g).unwrap();
        let projected = engine.forecast_labor(&g, &h).unwrap();
        assert_eq!(serde_json::to_value(&h).unwrap(), before);
        assert_eq!(g.production_labor_forecast().unwrap(), projected);
        assert_eq!(engine.forecast_labor(&g, &h).unwrap(), projected);
        let mut unchanged = h.clone();
        engine.read(&g, &mut unchanged, false).unwrap();
        assert_eq!(serde_json::to_value(&unchanged).unwrap(), before);
        for row in &projected {
            for (actual, expected) in row.sectors.iter().zip([31., 4., 5., 10.]) {
                assert!((actual - expected).abs() < 1e-5);
            }
            assert_eq!(row.workforce, 50.);
        }
        engine.dispatch(&g, false, h.sites.len() as u32);
        engine.read(&g, &mut h, true).unwrap();
        for (s, p) in h.sites.iter().zip(&projected) {
            assert_eq!(s.economy.labor, p.sectors);
        }
        // Forecasting does not alter the ensuing complete production result.
        let mut direct = g.civilizations.as_ref().unwrap().clone();
        engine.upload(&g, &direct);
        engine.dispatch(&g, false, direct.sites.len() as u32);
        engine.read(&g, &mut direct, true).unwrap();
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(&direct).unwrap()
        );
        direct.sites[0].stocks.stock[0] = 0.;
        let zero = engine.forecast_labor(&g, &direct).unwrap();
        assert_eq!(zero[0].sectors, [0.; 4]);
    }
}

#[cfg(test)]
mod agriculture_tests {
    use super::*;
    use crate::{
        participation::{Activity, Presence},
        resolution::Mode,
    };
    #[test]
    #[ignore = "requires hardware GPU"]
    fn agricultural_attendance_controls_cultivation_income_and_continuation() {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                seed: 17,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.set_demographic_resolution(Mode::Individual, true)
            .unwrap();
        h.set_workshop_refinement(true).unwrap();
        h.set_agriculture_refinement(true).unwrap();
        assert!(h.set_workshop_refinement(false).is_err());
        let mut ready = h.clone();
        ready.month += 1;
        ready.begin_service_reservations();
        // Declared standing wheat fixture: this month's harvest must also require attendance.
        for site in &mut ready.sites {
            site.demography.crops[2] = ready.month as f32;
            site.economy.crops[0][1] = 100.;
        }
        let engine = Engine::new(&g).unwrap();
        let forecast = engine.forecast_labor(&g, &ready).unwrap();
        let mut busy = ready.clone();
        let ids: Vec<_> = busy
            .participation
            .as_ref()
            .unwrap()
            .residents
            .values()
            .filter_map(|r| {
                if let Presence::Resident(site) = r.presence {
                    Some((r.person, site))
                } else {
                    None
                }
            })
            .collect();
        for (id, site) in ids {
            let pool = busy.participation.as_mut().unwrap();
            let spare = pool.available(id);
            pool.reserve(busy.month, site, Activity::Research, &[id], spare);
        }
        ready.reserve_agriculture(&forecast).unwrap();
        busy.reserve_agriculture(&forecast).unwrap();
        assert!(ready.sites.iter().any(|s| s.economy.farm_workers[1] > 0.));
        assert!(busy.sites.iter().all(|s| s.economy.farm_workers[1] == 0.));
        assert!(ready.set_agriculture_refinement(false).is_err());
        assert!(ready.reserve_agriculture(&forecast).is_err());
        let farm_weights = ready.agricultural_earnings().unwrap();
        let money = |h: &History| {
            h.sites
                .iter()
                .map(|s| s.economy.finance[0] as f64)
                .sum::<f64>()
                + h.society
                    .as_ref()
                    .unwrap()
                    .household_economy
                    .as_ref()
                    .unwrap()
                    .accounts
                    .iter()
                    .map(|a| a.cash)
                    .sum::<f64>()
        };
        let before = money(&ready);
        ready.prepare_household_retail();
        assert!((money(&ready) - before).abs() < 1e-6);
        let wallets = &ready
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .accounts;
        assert!(wallets.iter().any(|a| a.sector_wages[0] > 0.));
        for (id, a) in wallets.iter().enumerate() {
            if !farm_weights.contains_key(&id) {
                assert!(a.sector_wages[0] < 1e-6);
            }
        }
        engine.upload(&g, &ready);
        engine.dispatch(&g, false, ready.sites.len() as u32);
        engine.read(&g, &mut ready, true).unwrap();
        engine.upload(&g, &busy);
        engine.dispatch(&g, false, busy.sites.len() as u32);
        engine.read(&g, &mut busy, true).unwrap();
        assert!(ready
            .sites
            .iter()
            .any(|s| s.economy.production_probe[1] > 0.));
        assert!(busy
            .sites
            .iter()
            .all(|s| s.economy.production_probe[1] == 0.));
        assert!(ready.sites.iter().any(|s| s.economy.crops[0][3] > 0.));
        assert!(busy.sites.iter().all(|s| s.economy.crops[0][3] == 0.));
        ready.settle_agriculture().unwrap();
        let settled = serde_json::to_value(&ready.resolution).unwrap();
        ready.settle_agriculture().unwrap();
        assert_eq!(serde_json::to_value(&ready.resolution).unwrap(), settled);
        ready.validate_agriculture().unwrap();
        // Normal monthly pipeline, plus persistence and execution-speed equivalence.
        g.advance_history(2).unwrap();
        let path =
            std::env::temp_dir().join(format!("farm-participation-{}.world", std::process::id()));
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
