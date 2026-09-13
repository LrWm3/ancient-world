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
    /// Feasible building work from opening stocks and targets, before new extraction.
    pub construction: f32,
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
            pass.dispatch_workgroups(
                (h.sites.len() as u32).div_ceil(super::HISTORY_WORKGROUP_SIZE),
                1,
                1,
            );
        }
        g.gpu.queue.submit(Some(encoder.finish()));
        let bytes = read_buffer(
            &g.gpu,
            &self.output,
            0,
            h.sites.len() as u64 * std::mem::size_of::<super::Stocks>() as u64,
        )?;
        bytemuck::cast_slice::<u8, Stocks>(&bytes)
            .iter()
            .zip(&h.sites)
            .map(|(row, site)| {
                ensure!(
                    row.stock
                        .iter()
                        .chain(&row.habitat)
                        .chain(&row.ledger)
                        .all(|x| x.is_finite() && *x >= 0.),
                    "invalid GPU labor forecast at site {}",
                    site.id
                );
                Ok(ProductionLaborForecast {
                    month: h.month,
                    site: site.id,
                    sectors: row.stock,
                    construction: row.ledger[0],
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
        assert_eq!(zero[0].construction, 0.);

        // Analytical housing fixture: two timber + three bricks build one place
        // using 0.2 worker-month; an unfunded target alone requests no builders.
        let mut housing = g.civilizations.as_ref().unwrap().clone();
        for site in &mut housing.sites {
            site.economy = Economy::default();
            site.economy.logistics[3] = 3.;
            site.economy.housing_plan = [10., 0., 0., 1.];
            site.economy.goods = [0.; 64];
            site.economy.goods[0] = 2.;
            site.economy.goods[5] = 3.;
        }
        let before = serde_json::to_value(&housing).unwrap();
        let built = engine.forecast_labor(&g, &housing).unwrap();
        assert!(built.iter().all(|r| (r.construction - 0.2).abs() < 1e-5));
        let mut read = housing.clone();
        engine.read(&g, &mut read, false).unwrap();
        assert_eq!(serde_json::to_value(&read).unwrap(), before);
        for site in &mut housing.sites {
            site.economy.goods[5] = 0.;
        }
        assert!(engine
            .forecast_labor(&g, &housing)
            .unwrap()
            .iter()
            .all(|r| r.construction == 0.));
        for site in &mut housing.sites {
            site.economy.goods[5] = 3.;
            site.economy.housing_plan[0] = 0.;
        }
        assert!(engine
            .forecast_labor(&g, &housing)
            .unwrap()
            .iter()
            .all(|r| r.construction == 0.));
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
        attendance_fixture(false, false);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn extraction_attendance_controls_output_income_and_continuation() {
        attendance_fixture(true, false);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn builders_limit_assets_without_stopping_contracted_workshops() {
        attendance_fixture(true, true);
    }
    fn attendance_fixture(extraction: bool, construction: bool) {
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
        h.set_extraction_refinement(extraction).unwrap();
        h.set_construction_refinement(construction).unwrap();
        assert!(h.set_workshop_refinement(false).is_err());
        let mut ready = h.clone();
        ready.month += 1;
        ready.begin_service_reservations();
        // Declared standing wheat fixture: this month's harvest must also require attendance.
        for site in &mut ready.sites {
            site.demography.crops[2] = ready.month as f32;
            site.economy.crops[0][1] = 100.;
            if extraction {
                site.economy.forest = [5000., 20., 2., 0.];
                site.economy.reserves[1] = 100.;
                site.economy.reserves[2] = 100.;
                site.economy.logistics[3] = 0.; // no stock-target suppression
            }
        }
        if construction {
            for site in &mut ready.sites {
                site.economy.goods[0] = 1000.;
                site.economy.goods[5] = 1000.;
                site.economy.housing = [0., 0., 0., 0.];
                site.economy.housing_plan = [200., 0., 0., 1.];
                site.economy.waterworks[3] = 0.;
            }
        }
        let engine = Engine::new(&g).unwrap();
        let forecast = engine.forecast_labor(&g, &ready).unwrap();
        if construction {
            // Demand-limited matched intervention: same effective work and stocks,
            // different completed practice. Use a small common request so matching
            // itself does not obscure the conversion for later sectors.
            let mut requests = forecast.clone();
            for f in &mut requests {
                for work in &mut f.sectors {
                    *work *= 0.1;
                }
                f.construction *= 0.1;
            }
            let mut novice = ready.clone();
            for r in novice
                .participation
                .as_mut()
                .unwrap()
                .residents
                .values_mut()
            {
                r.production_practice = [0.; 4];
            }
            let mut expert = novice.clone();
            for r in expert
                .participation
                .as_mut()
                .unwrap()
                .residents
                .values_mut()
            {
                r.production_practice = [12.; 4];
            }
            for case in [&mut novice, &mut expert] {
                case.reserve_agriculture(&requests).unwrap();
                engine.upload(&g, case);
                engine.dispatch(&g, false, case.sites.len() as u32);
                engine.read(&g, case, true).unwrap();
                case.settle_agriculture().unwrap();
                case.validate_agriculture().unwrap();
            }
            let plans = |h: &History| {
                h.resolution
                    .as_ref()
                    .unwrap()
                    .agriculture
                    .as_ref()
                    .unwrap()
                    .plans
                    .clone()
            };
            let np = plans(&novice);
            let ep = plans(&expert);
            for sector in 0..4 {
                let totals = |ps: &[crate::agriculture_participation::FarmPlan]| {
                    ps.iter()
                        .filter(|p| p.sector == sector)
                        .fold([0f64; 2], |mut t, p| {
                            t[0] += p.effective_granted() as f64;
                            t[1] += p.assignments.iter().map(|a| a.used as f64).sum::<f64>();
                            t
                        })
                };
                let n = totals(&np);
                let e = totals(&ep);
                assert!(n[0] > 0. && n[1] > 0., "sector {sector}: {n:?}");
                assert!((e[0] - n[0]).abs() < 1e-4, "sector {sector}: {n:?} {e:?}");
                assert!(
                    (e[1] / n[1] - 0.8).abs() < 0.002,
                    "sector {sector}: {n:?} {e:?}"
                );
            }
            for (n, e) in novice.sites.iter().zip(&expert.sites) {
                for (a, b) in n.economy.goods.iter().zip(e.economy.goods.iter()) {
                    assert!(
                        (a - b).abs() < 0.002,
                        "same productive work must not multiply materials"
                    );
                }
                assert!(
                    (n.economy.production_probe[1] - e.economy.production_probe[1]).abs() < 0.002
                );
                assert!((n.economy.housing_plan[2] - e.economy.housing_plan[2]).abs() < 0.002);
            }
            let mut restored: History =
                serde_json::from_value(serde_json::to_value(&expert).unwrap()).unwrap();
            restored.settle_agriculture().unwrap();
            assert_eq!(
                serde_json::to_value(&restored).unwrap(),
                serde_json::to_value(&expert).unwrap()
            );
        }
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
        let extraction_weights = [ready.production_earnings(1), ready.production_earnings(2)];
        if extraction {
            assert!(ready.set_extraction_refinement(false).is_err());
            assert!(ready
                .sites
                .iter()
                .any(|s| s.economy.extraction_workers[0] > 0.
                    && s.economy.extraction_workers[1] > 0.));
            assert!(busy
                .sites
                .iter()
                .all(|s| s.economy.extraction_workers == [0.; 4]));
        }
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
        if extraction {
            for (sector, weights) in extraction_weights.iter().enumerate() {
                let weights = weights.as_ref().unwrap();
                assert!(wallets.iter().any(|a| a.sector_wages[sector + 1] > 0.));
                for (id, account) in wallets.iter().enumerate() {
                    if !weights.contains_key(&id) {
                        assert!(account.sector_wages[sector + 1] < 1e-6);
                    }
                }
            }
        }
        // Aggregate staffing remains the full-attendance control; it does not
        // borrow the busy residents' named commitments or pay another wage.
        let mut aggregate = busy.clone();
        for site in &mut aggregate.sites {
            site.economy.farm_workers = [0.; 4];
        }
        engine.upload(&g, &aggregate);
        engine.dispatch(&g, false, aggregate.sites.len() as u32);
        engine.read(&g, &mut aggregate, true).unwrap();
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
        let herd_products = |h: &History| {
            h.sites
                .iter()
                .flat_map(|s| s.economy.herds)
                .map(|a| a[3] as f64)
                .sum::<f64>()
        };
        assert!(herd_products(&ready) > 0.);
        assert_eq!(
            herd_products(&busy),
            0.,
            "absent attendants cannot collect animal products"
        );
        assert!(
            herd_products(&aggregate) >= herd_products(&ready),
            "aggregate attendance remains the unrestricted control"
        );
        assert_eq!(
            busy.sites
                .iter()
                .map(|s| s.economy.agriculture[1])
                .sum::<f32>(),
            0.,
            "nobody delivers stored feed without farm attendance"
        );
        assert!(ready.sites.iter().any(|s| s.economy.agriculture[1] > 0.));
        assert!(
            busy.sites
                .iter()
                .flat_map(|s| s.economy.herds)
                .any(|a| a[2] > 0.),
            "unattended herds still experience biological mortality"
        );
        assert!(ready.sites.iter().any(|s| s.economy.crops[0][3] > 0.));
        assert!(busy.sites.iter().all(|s| s.economy.crops[0][3] == 0.));
        if extraction {
            assert!(ready
                .sites
                .iter()
                .any(|s| s.economy.extraction_workers[2] > 0.));
            assert!(ready
                .sites
                .iter()
                .any(|s| s.economy.extraction_workers[3] > 0.));
            assert!(busy
                .sites
                .iter()
                .all(|s| s.economy.extraction_workers == [0.; 4]));
            assert!(busy
                .sites
                .iter()
                .all(|s| s.economy.reserves[1] == 100. && s.economy.reserves[2] == 100.));
        }
        if construction {
            assert!(ready
                .sites
                .iter()
                .any(|s| s.economy.construction_workers[2] > 0.));
            assert!(ready.sites.iter().any(|s| s.economy.housing_plan[2] > 0.));
            assert!(busy.sites.iter().all(
                |s| s.economy.construction_workers[2] == 0. && s.economy.housing_plan[2] == 0.
            ));
            // Synthetic, already contracted workshop inputs: no new builders available.
            let mut contracted = busy.clone();
            for site in &mut contracted.sites {
                let e = &mut site.economy;
                e.goods = [0.; 64];
                e.goods[0] = 1000.;
                e.goods[1] = 1000.;
                e.goods[2] = 1000.;
                e.goods[4] = 1000.;
                e.workshop = [200., 300., 20., 1.];
                e.workshop_types = [[2., 2., 0., 1.]; 4];
                e.enterprise_lease = [1.; 4];
                e.enterprise_plan = [1.; 4];
                e.logistics[3] = 0.;
            }
            engine.upload(&g, &contracted);
            engine.dispatch(&g, false, contracted.sites.len() as u32);
            engine.read(&g, &mut contracted, true).unwrap();
            assert!(contracted
                .sites
                .iter()
                .all(|s| s.economy.construction_workers[2] == 0.));
            assert!(contracted
                .sites
                .iter()
                .any(|s| s.economy.enterprise_used.iter().sum::<f32>() > 0.));
        }
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

#[cfg(test)]
mod recipe_allocation_tests {
    use super::*;
    use crate::economy::{EconomyCatalog, Recipe, GOODS};

    #[test]
    #[ignore = "requires hardware GPU"]
    fn recovered_inputs_enable_only_matching_funded_work_after_delivery() {
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
        g.enable_society().unwrap();
        let mut baseline = g.civilizations.as_ref().unwrap().clone();
        baseline.politics = None;
        baseline.living = None;
        baseline.month = 3;
        baseline.cargo.clear();
        let mut recipe = Recipe {
            input: [0.; GOODS],
            output: [0.; GOODS],
            work: [1., 0., 1., 0.],
        };
        recipe.input[2] = 1.;
        recipe.output[3] = 1.;
        let mut catalog = EconomyCatalog::bundled().unwrap();
        catalog.recipes = vec![recipe];
        baseline.economy_catalog = Some(catalog);
        for site in &mut baseline.sites {
            site.economy = Economy {
                logistics: [1000., 0., 0., 3.],
                policy: [0., 0., 0., 1.],
                ..Default::default()
            };
            site.economy.targets.fill(100.);
            site.economy.orders[0] = 100.;
            site.economy.workshop = [100., 150., 10., 1.];
            site.economy.workshop_types[0][3] = 1.;
            site.economy.workshop_types[1] = [5., 5., 0., 0.];
            site.economy.enterprise_lease[1] = 5.;
            site.economy.enterprise_plan[1] = 3.;
            site.stocks.stock[0] = 100.;
        }
        baseline.sites[1].civilization = baseline.sites[0].civilization;
        baseline.sites[1].island = baseline.sites[0].island;
        baseline.sites[1].abandoned = true;
        baseline.sites[1].stocks.stock[0] = 0.;
        baseline.sites[1].economy.goods[2] = 20.;
        baseline.sites[1].economy.goods[0] = 20.;
        baseline.sites[1].economy.prices.fill(2.);
        baseline.sites[0].economy.finance[0] = 1000.;
        baseline.society.as_mut().unwrap().routes = vec![crate::society::Route {
            upkeep: None,
            id: 0,
            from: 0,
            to: 1,
            cells: vec![baseline.sites[0].cell, baseline.sites[1].cell],
            cost_km: 150.,
            open: true,
            flood_months: 0,
            road_bricks: 0.,
        }];
        // Recovery off, cargo not yet due, useful delivery, wrong material,
        // and useful delivery without prepaid operator attendance.
        for (recover, due, good, prepaid) in [
            (false, true, 2, 3.),
            (true, false, 2, 3.),
            (true, true, 2, 3.),
            (true, true, 0, 3.),
            (true, true, 2, 0.),
        ] {
            let mut h = baseline.clone();
            let cash = h.money_residual();
            if recover {
                assert_eq!(h.recover_abandoned_stock(0, 1, good, 20.).unwrap(), 20.);
            }
            assert!((h.money_residual() - cash).abs() < 1e-6);
            assert_eq!(h.sites[0].economy.goods[2], 0.);
            if due {
                h.month = 5;
                h.market_arrivals();
            }
            let available = recover && due && good == 2;
            assert_eq!(
                h.sites[0].economy.goods[2],
                if available { 20. } else { 0. }
            );
            // Isolate the existing production dispatch from other monthly social
            // changes. Recovery has already used the real cargo arrival path.
            h.society = None;
            h.sites[0].economy.enterprise_plan[1] = prepaid;
            g.civilizations = Some(h.clone());
            let engine = Engine::new(&g).unwrap();
            engine.upload(&g, &h);
            engine.dispatch(&g, false, h.sites.len() as u32);
            engine.read(&g, &mut h, true).unwrap();
            let e = &h.sites[0].economy;
            assert_eq!(e.made[3] > 0., available);
            assert_eq!(e.enterprise_used[1] > 0., available && prepaid > 0.);
            assert!(e.enterprise_used[1] <= prepaid + 1e-5);
            assert!(e.used[2] <= 20.);
            assert!((e.used[2] - e.made[3]).abs() < 1e-5);
        }
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn unrelated_food_recipes_cannot_spend_prepaid_industrial_time() {
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
        let mut recipe = Recipe {
            input: [0.; GOODS],
            output: [0.; GOODS],
            work: [1., 0., 1., 0.],
        };
        recipe.input[2] = 1.;
        recipe.output[3] = 1.;
        let mut competitor = recipe;
        competitor.input.fill(0.);
        competitor.output.fill(0.);
        competitor.input[8] = 1.;
        competitor.output[14] = 1.; // Household flour processing, not industrial service.
        let baseline = g.civilizations.as_ref().unwrap().clone();
        for (competing, construction) in [(false, 0), (true, 0), (false, 1), (false, 2)] {
            let prepaid = if construction > 0 { 9.5 } else { 8. };
            let mut h = baseline.clone();
            h.month = 0;
            h.society = None;
            h.living = None;
            let mut catalog = EconomyCatalog::bundled().unwrap();
            catalog.recipes = if competing {
                vec![competitor, recipe]
            } else {
                vec![recipe]
            };
            h.economy_catalog = Some(catalog);
            for s in &mut h.sites {
                s.stocks.stock[0] = 100.;
                let mut e = Economy {
                    logistics: [1000., 0., 0., 3.], // Fixed ten craft worker-months.
                    ..Default::default()
                };
                e.goods[2] = 100.;
                e.goods[8] = 100.;
                e.targets.fill(100.);
                e.orders[..2].fill(100.);
                e.workshop = [100., 150., 10., 1.];
                e.workshop_types[0][3] = 1.;
                e.workshop_types[1] = [5., 5., 0., 0.];
                e.enterprise_lease[1] = 5.;
                e.enterprise_productivity[1] = 0.;
                e.enterprise_plan[1] = prepaid; // Already funded attendance, not extra population.
                if construction > 0 {
                    e.goods[0] = 10000.;
                    e.goods[5] = 10000.;
                    e.housing_plan = [1000., 0., 0., 1.];
                    if construction == 2 {
                        e.construction_workers = [1., 2., 0., 0.];
                    }
                }
                s.economy = e;
            }
            g.civilizations = Some(h.clone());
            let engine = Engine::new(&g).unwrap();
            engine.upload(&g, &h);
            engine.dispatch(&g, false, h.sites.len() as u32);
            engine.read(&g, &mut h, true).unwrap();
            let e = &h.sites[0].economy;
            assert!(
                (e.enterprise_used[1] - prepaid).abs() < 1e-5,
                "competing={competing} construction={construction}: prepaid={} flour={}",
                e.enterprise_used[1],
                e.made[14]
            );
            assert!(e.made[3] + e.made[14] + e.housing[3] <= 10. + 1e-5);
            if construction > 0 {
                assert!(e.housing[3] > 0. && e.housing[3] <= 10. - prepaid + 1e-5);
            }
            assert!((e.used[2] - e.made[3]).abs() < 1e-5);
        }
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn blocked_recipes_do_not_strand_usable_prepaid_work() {
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
        let mut recipe = Recipe {
            input: [0.; GOODS],
            output: [0.; GOODS],
            work: [1., 0., 1., 0.],
        };
        recipe.input[2] = 1.;
        recipe.output[3] = 1.;
        let mut blocked = recipe;
        blocked.input[6] = 1.; // Additional charcoal is absent; material identities remain valid.
        let baseline = g.civilizations.as_ref().unwrap().clone();
        let mut outcomes = vec![];
        for (count, metal, order, experience) in [
            (1, 100., 100., 0.),
            (6, 100., 100., 0.),
            (6, 0., 100., 0.),
            (6, 1., 100., 0.),
            (6, 100., 1., 0.),
            (6, 100., 100., 0.5),
        ] {
            let mut h = baseline.clone();
            h.month = 0;
            h.society = None;
            h.living = None;
            let mut catalog = EconomyCatalog::bundled().unwrap();
            catalog.recipes = vec![blocked; count];
            catalog.recipes[0] = recipe;
            h.economy_catalog = Some(catalog);
            for s in &mut h.sites {
                s.stocks.stock[0] = 100.;
                let mut e = Economy {
                    logistics: [1000., 0., 0., 3.], // Fixed ten craft worker-months.
                    ..Default::default()
                };
                e.goods[2] = metal;
                e.targets.fill(100.);
                e.orders[..count].fill(order);
                e.workshop = [100., 150., 10., 1.];
                e.workshop_types[0][3] = 1.;
                e.workshop_types[1] = [5., 5., 0., 0.];
                e.enterprise_lease[1] = 5.;
                e.enterprise_productivity[1] = experience;
                e.enterprise_plan[1] = 3.; // Already funded attendance, not extra population.
                s.economy = e;
            }
            g.civilizations = Some(h.clone());
            let engine = Engine::new(&g).unwrap();
            engine.upload(&g, &h);
            engine.dispatch(&g, false, h.sites.len() as u32);
            engine.read(&g, &mut h, true).unwrap();
            let e = &h.sites[0].economy;
            assert_eq!(e.enterprise_used[1] > 0., metal > 0.);
            assert!(e.enterprise_used[1] <= 3. + 1e-5);
            assert!(e.used[2] <= metal + 1e-5);
            assert!(e.made[3] <= order + 1e-5);
            assert!(e.enterprise_used[1] <= e.made[3] / (1. + experience) + 1e-5);
            assert!((e.used[2] - e.made[3]).abs() < 1e-5);
            outcomes.push(e.enterprise_used[1]);
        }
        assert!(
            (outcomes[0] - outcomes[1]).abs() < 1e-5,
            "irrelevant blocked recipes reduced paid work: {outcomes:?}"
        );
    }
}
