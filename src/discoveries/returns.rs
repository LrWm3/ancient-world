//! Typed botanical collections share the original finite organic source and cargo ledger.
use super::*;
pub const NAMES: [&str; 3] = [
    "silver bast fibers",
    "ironberry pigment material",
    "marsh-thread planting samples",
];
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Use {
    Store,
    Study,
    #[default]
    Apply,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Botanicals {
    pub policy: [Use; 3],
    pub stock: [f64; 3],
    pub received: [f64; 3],
    pub used: [f64; 3],
    pub studied: [f64; 3],
    pub output: [f64; 3],
    pub attempts: [u32; 3],
    pub causes: [Option<u64>; 3],
    pub sources: [Option<u32>; 3],
}
impl Botanicals {
    pub fn validate(&self, h: &History) -> Result<()> {
        for k in 0..3 {
            ensure!(
                [
                    self.stock[k],
                    self.received[k],
                    self.used[k],
                    self.studied[k],
                    self.output[k]
                ]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.)
                    && (self.received[k] - self.stock[k] - self.used[k]).abs()
                        < 1e-6 * self.received[k].max(1.)
                    && self.studied[k] <= self.used[k] + 1e-8
                    && self.studied[k] <= 0.25 + 1e-8
                    && self.output[k] <= (self.used[k] - self.studied[k]) * 0.5 + 1e-6,
                "botanical collection ledger mismatch"
            );
            ensure!(
                self.causes[k].is_none_or(|id| h
                    .events
                    .get(id as usize)
                    .is_some_and(|e| e.kind == "specimens_delivered"))
                    && (self.received[k] == 0. || self.causes[k].is_some())
                    && self.sources[k]
                        .is_none_or(|cell| cell < 6 * h.terrain_resolution * h.terrain_resolution),
                "invalid botanical provenance"
            );
        }
        Ok(())
    }
}
pub(super) fn limit(b: &Botanicals, k: usize) -> f64 {
    if b.policy[k] == Use::Store {
        return 0.;
    }
    if b.studied[k] < 0.25 - 1e-8 {
        return b.stock[k].min(0.25 - b.studied[k]).min(0.1);
    }
    if b.policy[k] == Use::Study {
        return 0.;
    }
    b.stock[k].min(0.25)
}
/// Conservatively bound all output constituents by the input composition.
fn yield_fraction(output: [f32; 3]) -> f64 {
    output
        .into_iter()
        .zip(CNP[0])
        .fold(0.5_f64, |fraction, (out, input)| {
            if out > 0. {
                fraction.min(input / out as f64)
            } else {
                fraction
            }
        })
}
pub(super) fn process(h: &mut History, w: &mut Workshop, cells: &[Cell], labor: &mut f64) -> f64 {
    let mut completed = 0.;
    for (k, name) in NAMES.iter().enumerate() {
        let Some(plan) = w
            .work_plan
            .as_ref()
            .filter(|p| p.receipt.month == h.month && !p.receipt.settled)
        else {
            continue;
        };
        let s = &h.sites[w.site as usize];
        let studying = w.botanicals.studied[k] < 0.25 - 1e-8;
        let kg = limit(&w.botanicals, k)
            .min(plan.botanical_kg[k])
            .min(*labor / 2.)
            .min(s.economy.goods[3] as f64 / 0.1)
            .min(s.economy.goods[6] as f64 / 0.2);
        if kg <= 1e-8 {
            continue;
        }
        let crop = h
            .economy_catalog
            .as_ref()
            .and_then(|c| c.agriculture.as_ref())
            .and_then(|a| a.crops.iter().enumerate().find(|(_, c)| c.good == "flax"));
        let crop_index = crop.map(|(i, _)| i);
        let habitat = crop.is_some_and(|(_, crop)| {
            cells.get(s.cell as usize).is_some_and(|cell| {
                cell.climate[0] > crop.temperature[0] + 5.
                    && cell.climate[0] < crop.temperature[1] - 5.
                    && cell.hydro[2] >= crop.rainfall_mm * 0.5
            })
        }) && s.economy.management[0] > 0.
            && crop_index.is_some_and(|i| s.economy.crops[i][0] > 0.)
            && s.economy.soil[1] >= 0.001
            && s.economy.soil[2] >= 0.0001
            && s.economy.water[0] as f64 >= kg * 0.2;
        let good = h
            .economy_catalog
            .as_ref()
            .and_then(|c| c.index(["fiber", "writing_material", "flax"][k]));
        // Absent catalog entries leave the collection untouched for later work.
        if !studying && good.is_none() {
            continue;
        }
        let composition = good
            .map(|g| h.economy_catalog.as_ref().unwrap().composition(g))
            .unwrap_or([0.; 3]);
        let output = if studying || (k == 2 && !habitat) {
            0.
        } else {
            kg * yield_fraction(composition)
        };
        if let Some(outcomes) = w.work_plan.as_mut().and_then(|p| p.outcomes.as_mut()) {
            outcomes.botanical_actual[k + if studying { 0 } else { 3 }] += kg;
        }
        w.botanicals.stock[k] -= kg;
        w.botanicals.used[k] += kg;
        w.botanicals.output[k] += output;
        if studying {
            w.botanicals.studied[k] += kg;
        } else {
            w.botanicals.attempts[k] += 1;
        }
        *labor = (*labor - 2. * kg).max(0.);
        completed += 2. * kg;
        let e = &mut h.sites[w.site as usize].economy;
        let tools = (kg * 0.1) as f32;
        let fuel = (kg * 0.2) as f32;
        e.goods[3] -= tools;
        e.used[3] += tools;
        e.reserves[3] += tools;
        e.goods[6] -= fuel;
        e.used[6] += fuel;
        e.external[0] -= fuel;
        if output > 0. {
            if k == 2 {
                e.crops[crop_index.unwrap()][2] += output as f32;
                e.water[0] -= (kg * 0.2) as f32;
                e.water[3] += (kg * 0.2) as f32;
            } else {
                let good = good.unwrap();
                e.goods[good] += output as f32;
                e.made[good] += output as f32;
            }
        }
        for (j, ratio) in composition.into_iter().enumerate() {
            e.detritus[j] += (kg * CNP[0][j] - output * ratio as f64).max(0.) as f32;
        }
        if studying && w.botanicals.studied[k] >= 0.25 - 1e-8 {
            event(h,w.site,w.botanicals.causes[k],"botanical_trial_method",format!("Destructive trials established preparation of {}; further applications still consume material and work",name));
        } else if !studying
            && (w.botanicals.attempts[k] == 1 || w.botanicals.attempts[k].is_multiple_of(12))
        {
            event(
                h,
                w.site,
                w.botanicals.causes[k],
                "botanical_application",
                format!(
                    "{}: consumed {kg:.3} kg and {:.3} worker-months; {output:.3} kg {}. {}",
                    name,
                    2. * kg,
                    [
                        "usable fiber",
                        "writing preparation",
                        "compatible fiber-crop seed"
                    ][k],
                    if k == 2 {
                        "Habitat-gated trial; subsequent growth uses ordinary GPU land, nutrient and water budgets."
                    } else {
                        "Output enters ordinary workshop and market inventories; residues remain in detritus."
                    }
                ),
            );
        }
    }
    completed
}

impl Generator {
    pub fn set_botanical_use(&mut self, site: u32, kind: usize, policy: Use) -> Result<()> {
        ensure!(kind < 3, "unknown botanical collection kind");
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let w = h
            .expeditions
            .as_mut()
            .and_then(|x| x.discoveries.as_mut())
            .and_then(|d| d.workshops.iter_mut().find(|w| w.site == site))
            .ok_or_else(|| anyhow::anyhow!("no research workshop"))?;
        ensure!(
            w.work_plan
                .as_ref()
                .is_none_or(|p| p.receipt.settled || p.receipt.month != h.month),
            "finish reserved research before changing use"
        );
        w.botanicals.policy[kind] = policy;
        h.event(
            "botanical_use_policy",
            Some(site),
            None,
            format!("{} assigned to {policy:?}", NAMES[kind]),
        );
        Ok(())
    }
}

/// A proven local use can justify revisiting its actual, nondepleted collection site.
pub(crate) fn followup(h: &History, d: &Discoveries, site: u32, cell: u32) -> Option<u64> {
    let w = d.workshops.iter().find(|w| w.site == site && w.enabled)?;
    if !d
        .sources
        .iter()
        .any(|s| s.cell == cell && s.remaining[0] > 0.1)
    {
        return None;
    }
    for k in 0..3 {
        if w.botanicals.policy[k] != Use::Apply
            || w.botanicals.sources[k] != Some(cell)
            || w.botanicals.output[k] <= 0.
            || w.botanicals.stock[k] >= 0.25
        {
            continue;
        }
        let good = h
            .economy_catalog
            .as_ref()?
            .index(["fiber", "writing_material", "flax"][k])?;
        let town = &h.sites[site as usize];
        let target = town.stocks.stock[0].max(1.) * 0.05;
        if town.economy.goods[good] >= target {
            continue;
        }
        return h
            .events
            .iter()
            .rev()
            .find(|e| {
                e.kind == "botanical_application"
                    && e.site == Some(site)
                    && w.botanicals.causes[k].is_some_and(|cause| e.causes.contains(&cause))
            })
            .map(|e| e.id);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_yield_and_collection_policies() {
        assert!((yield_fraction([0.45, 0.02, 0.003]) - 0.5).abs() < 1e-6);
        assert!(yield_fraction([0.45, 0.5, 0.003]) <= 0.04 + 1e-6);
        let mut b = Botanicals::default();
        b.stock[0] = 1.;
        assert_eq!(limit(&b, 0), 0.1);
        b.policy[0] = Use::Store;
        assert_eq!(limit(&b, 0), 0.);
        b.policy[0] = Use::Study;
        b.studied[0] = 0.25;
        assert_eq!(limit(&b, 0), 0.);
        b.policy[0] = Use::Apply;
        assert_eq!(limit(&b, 0), 0.25);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn botanical_trials_transfer_material_and_require_habitat_and_work() {
        use crate::{catalog::Catalog, config::Config, gpu::ContextGpu};
        let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
        for seed in [17, 81, 256] {
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    resolution: 32,
                    ecology_resolution: 16,
                    seed,
                    ..Default::default()
                },
                Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.found_civilizations(3).unwrap();
            g.enable_society().unwrap();
            g.set_diversified_farming(true).unwrap();
            let mut cells = g.snapshot().unwrap();
            let h = g.civilizations.as_mut().unwrap();
            h.month = 1;
            // Exercise collection -> cargo -> delivery using finite synthetic frontier patches.
            let mut collection_h = h.clone();
            let mut collections = Discoveries::new(1);
            let mut voyages = vec![];
            for profile in 0..3 {
                let source = cells
                    .iter()
                    .enumerate()
                    .find(|(i, c)| {
                        c.meta[0] == 3
                            && c.water[0] < 0.25
                            && ((*i as u32).wrapping_add(seed) % 3) as usize == profile
                    })
                    .unwrap()
                    .0;
                cells[source].life[0] = 0.8;
                cells[source].climate[0] = 20.;
                cells[source].geology[0] = 0.8;
                collection_h.event(
                    "expedition_departure",
                    Some(0),
                    None,
                    "Controlled frontier collection".into(),
                );
                let mut expedition = Expedition {
                    planned_cells: Some(vec![source as u32]),
                    institution: None,
                    heritage: None,
                    id: profile as u32,
                    origin: 0,
                    sponsor: 0,
                    public_funding: false,
                    route: 0,
                    objective: Objective::Ecology,
                    rescue: None,
                    crew: vec![crate::expeditions::Crew {
                        person: None,
                        identified_from_cohort: false,
                        expertise: Some(1.),
                        name: "Fixture collector".into(),
                        role: "naturalist".into(),
                        alive: true,
                    }],
                    phase: crate::expeditions::Phase::Camp,
                    departed: 1,
                    due: 2,
                    ended: None,
                    food: 20.,
                    timber: 10.,
                    tools: 12.,
                    purse: 0.,
                    spent: 0.,
                    findings: 0.,
                    confirmed: false,
                    exposure: 0.,
                    skill: 1.,
                    cause: collection_h.events.last().unwrap().id,
                    field_months: 1,
                    samples: [0.; 2],
                    botanicals: [0.; 3],
                    botanical_sources: [None; 3],
                };
                collections.collect(
                    &mut collection_h,
                    &mut expedition,
                    source as u32,
                    &cells[source],
                );
                assert_eq!(expedition.botanicals[profile], expedition.samples[0] * 0.5);
                if profile == 2 {
                    let mut rescue = expedition.clone();
                    rescue.samples = [0.; 2];
                    rescue.botanicals = [0.; 3];
                    rescue.botanical_sources = [None; 3];
                    rescue.take_collections(&mut expedition);
                    assert_eq!(expedition.samples, [0.; 2]);
                    assert_eq!(expedition.botanicals, [0.; 3]);
                    assert_eq!(rescue.botanical_sources[profile], Some(source as u32));
                    expedition = rescue;
                }
                collections.deliver(&mut collection_h, &mut expedition);
                let delivered = collections.workshops[0].botanicals.received;
                collections.deliver(&mut collection_h, &mut expedition);
                assert_eq!(collections.workshops[0].botanicals.received, delivered);
                voyages.push(expedition);
            }
            let x = Expeditions {
                version: 1,
                started: 1,
                rules: Default::default(),
                surveyed_ports: 0,
                routes: vec![],
                voyages,
                knowledge: vec![0.; 3],
                next_launch: vec![0; 3],
                discoveries: None,
            };
            let mut lost = x.voyages[0].clone();
            let source = lost.planned_cells.as_ref().unwrap()[0];
            collections.collect(
                &mut collection_h,
                &mut lost,
                source,
                &cells[source as usize],
            );
            assert!(lost.samples[0] > 0.);
            collections.discard(&mut collection_h, &mut lost);
            assert_eq!(lost.botanicals, [0.; 3]);
            assert_eq!(lost.samples, [0.; 2]);
            collections.validate(&collection_h, &x, &cells).unwrap();
            assert!(collections.residuals(&x).iter().all(|v| v.abs() < 1e-10));
            for month in 1..=4 {
                collection_h.month = month;
                let e = &mut collection_h.sites[0].economy;
                e.external[3] = 2.;
                e.labor[3] = 2.;
                e.goods[3] = 10.;
                e.goods[6] = 10.;
                collection_h.sites[0].economy.external[3] = 0.;
                let plan = super::plan_work(
                    &collection_h,
                    &collections.workshops[0],
                    &collections.workshops,
                );
                assert!(plan.botanical_kg.iter().any(|kg| *kg > 0.));
                let expected = plan.outcomes.as_ref().unwrap().botanical_expected;
                assert!(expected.iter().sum::<f64>() > 0.);
                assert_eq!(plan.outcomes.as_ref().unwrap().botanical_actual, [0.; 6]);
                collection_h.sites[0].economy.external[3] = plan.receipt.granted as f32;
                collections.workshops[0].work_plan = Some(plan);
                collections.month_in_environment(&mut collection_h, &cells);
                let actual = collections.workshops[0]
                    .work_plan
                    .as_ref()
                    .unwrap()
                    .outcomes
                    .as_ref()
                    .unwrap()
                    .botanical_actual;
                for (a, e) in actual.into_iter().zip(expected) {
                    assert!((a - e).abs() < 1e-7, "botanical request/execution mismatch");
                }
                collections.validate(&collection_h, &x, &cells).unwrap();
            }
            assert!(collections.workshops[0].botanicals.output[0] > 0.);
            assert!(collections.workshops[0].botanicals.output[1] > 0.);

            h.event(
                "specimens_delivered",
                Some(0),
                None,
                "Controlled botanical delivery".into(),
            );
            let cause = h.events.last().unwrap().id;
            let mut w = Workshop {
                botanicals: Botanicals {
                    stock: [1.75; 3],
                    received: [2.; 3],
                    used: [0.25; 3],
                    studied: [0.25; 3],
                    causes: [Some(cause); 3],
                    ..Default::default()
                },
                work_plan: Some(ResearchPlan {
                    botanical_kg: [0.25; 3],
                    receipt: crate::labor::WorkReceipt {
                        month: 1,
                        requested: 1.5,
                        granted: 1.5,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                site: 0,
                enabled: true,
                processed: [0.; 2],
                curated: [0.; 2],
                samples: [0.; 2],
                studied: [0.; 2],
                learned: [None; 2],
                remedy: 0.,
                delivered: [0.; 2],
                causes: [None; 2],
                batches: [0; 2],
            };
            let crop = h
                .economy_catalog
                .as_ref()
                .unwrap()
                .agriculture
                .as_ref()
                .unwrap()
                .crops
                .iter()
                .position(|c| c.good == "flax")
                .unwrap();
            let s = &mut h.sites[0];
            s.economy.detritus = [0.; 4]; // Isolate sub-kilogram transfers from a large f32 opening stock.
            s.economy.goods[3] = 10.;
            s.economy.goods[6] = 10.;
            s.economy.water[0] = 10.;
            s.economy.soil[1] = 1.;
            s.economy.soil[2] = 1.;
            s.economy.crops[crop][0] = 0.1;
            cells[s.cell as usize].climate[0] = 20.;
            cells[s.cell as usize].hydro[2] = 1000.;
            let cell_index = s.cell as usize;
            let opening = h.clone();
            // Forecasting applications without their destination catalog must request no work.
            let mut missing_catalog = h.clone();
            missing_catalog.economy_catalog = None;
            let missing_plan = super::plan_work(&missing_catalog, &w, &[]);
            assert_eq!(missing_plan.botanical_kg, [0.; 3]);
            let mut legacy = serde_json::to_value(&missing_plan).unwrap();
            legacy["outcomes"]
                .as_object_mut()
                .unwrap()
                .remove("botanical_captured");
            let legacy: ResearchPlan = serde_json::from_value(legacy).unwrap();
            assert!(!legacy.outcomes.unwrap().botanical_captured);
            let original = w.clone();
            assert_eq!(process(h, &mut w, &cells, &mut 0.), 0.);
            assert_eq!(w.botanicals.stock, original.botanicals.stock);
            let mut cold = cells.clone();
            cold[cell_index].climate[0] = -40.;
            let mut failed = original.clone();
            let mut failed_h = opening.clone();
            process(&mut failed_h, &mut failed, &cold, &mut 1.5);
            assert_eq!(failed.botanicals.output[2], 0.);
            assert!(failed.botanicals.used[2] > 0.25);
            let mut resumed_h: History =
                serde_json::from_value(serde_json::to_value(&opening).unwrap()).unwrap();
            let mut resumed: Workshop =
                serde_json::from_value(serde_json::to_value(&w).unwrap()).unwrap();
            assert!((process(h, &mut w, &cells, &mut 1.5) - 1.5).abs() < 1e-6);
            process(&mut resumed_h, &mut resumed, &cells, &mut 1.5);
            assert_eq!(
                serde_json::to_value(&*h).unwrap(),
                serde_json::to_value(&resumed_h).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&w).unwrap(),
                serde_json::to_value(&resumed).unwrap()
            );
            assert!(w.botanicals.output.iter().all(|v| *v > 0.));
            w.botanicals.validate(h).unwrap();
            let catalog = h.economy_catalog.as_ref().unwrap();
            for j in 0..3 {
                let product: f64 = ["fiber", "writing_material"]
                    .iter()
                    .enumerate()
                    .map(|(k, id)| {
                        w.botanicals.output[k]
                            * catalog.composition(catalog.index(id).unwrap())[j] as f64
                    })
                    .sum::<f64>()
                    + w.botanicals.output[2]
                        * catalog.composition(catalog.index("flax").unwrap())[j] as f64;
                let residue =
                    (h.sites[0].economy.detritus[j] - opening.sites[0].economy.detritus[j]) as f64;
                assert!((product + residue - 0.75 * CNP[0][j]).abs() < 1e-5);
            }
            let mut d = Discoveries::new(1);
            w.botanicals.stock[0] = 0.;
            w.botanicals.sources[0] = Some(0);
            d.workshops.push(w);
            d.sources.push(super::super::Source {
                cell: 0,
                initial: [10., 0.],
                remaining: [8., 0.],
                collected: [2., 0.],
            });
            let fiber = h.economy_catalog.as_ref().unwrap().index("fiber").unwrap();
            h.sites[0].economy.goods[fiber] = 0.;
            assert!(followup(h, &d, 0, 0).is_some());
            d.sources[0].remaining[0] = 0.;
            assert!(followup(h, &d, 0, 0).is_none());
        }
    }
}
