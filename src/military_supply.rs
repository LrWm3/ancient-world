//! Opening travel forecasts; bounded receipts observe existing transfers, never apply them.
use crate::resolution::{Boundary, Metric, Mode, Receipt, System};
use crate::{civilization::History, society::Raid};
use anyhow::{ensure, Result};
use std::collections::BTreeMap;

struct Row {
    boundary: Boundary,
    expected: [f64; 3],
    actual: [f64; 3],
}
pub(crate) struct SupplyComparison(BTreeMap<(u32, bool), Row>);
impl SupplyComparison {
    pub fn capture(h: &History) -> Result<Self> {
        let mut rows: BTreeMap<(u32, bool), Row> = BTreeMap::new();
        if let Some(state) = &h.resolution {
            ensure!(
                state
                    .receipts
                    .iter()
                    .all(|r| r.boundary.system != System::MilitarySupply
                        || r.boundary.month != h.month),
                "military supply already observed this month"
            );
            for j in h.society.iter().flat_map(|s| &s.raids) {
                let individual = j.members.is_some();
                let need = j.soldiers * crate::military::SOLDIER_FOOD_KG_PER_MONTH;
                let food = j.food.min(need);
                let rate = if food + crate::military::SUPPLY_SHORTFALL_TOLERANCE_KG < need {
                    crate::military::MONTHLY_STARVATION_LOSS_FRACTION
                } else {
                    0.
                };
                let pop = j.soldiers;
                let row = rows.entry((j.origin, individual)).or_insert_with(|| Row {
                    boundary: Boundary {
                        month: h.month,
                        system: System::MilitarySupply,
                        site: j.origin,
                        subject: u32::from(individual),
                        revision: h.seed as u64,
                    },
                    expected: [0.; 3],
                    actual: [0.; 3],
                });
                row.boundary.revision = crate::resolution::revision([
                    row.boundary.revision,
                    j.id as u64,
                    j.arrives as u64,
                    j.food.to_bits() as u64,
                    j.soldiers.to_bits() as u64,
                    j.loss_remainder.to_bits() as u64,
                ]);
                for &person in j.members.iter().flatten() {
                    row.boundary.revision = crate::resolution::revision([
                        row.boundary.revision,
                        person as u64,
                        u64::from(h.people[person as usize].died.is_some()),
                    ]);
                }
                row.expected[0] += food as f64;
                row.expected[1] += pop as f64 * rate;
                row.expected[2] += pop as f64 * (1. - rate);
            }
            for row in rows.values() {
                h.resolution
                    .as_ref()
                    .unwrap()
                    .check(&row.boundary, &row.boundary)?;
            }
        }
        Ok(Self(rows))
    }
    pub fn observe(&mut self, j: &Raid, individual: bool, before: f32, eaten: f32) {
        if let Some(row) = self.0.get_mut(&(j.origin, individual)) {
            let remaining = j.soldiers;
            row.actual[0] += eaten as f64;
            row.actual[1] += before as f64 - remaining as f64;
            row.actual[2] += remaining as f64;
        }
    }
    pub fn settle(self, h: &mut History) -> Result<()> {
        for ((_, individual), row) in self.0 {
            let current = row.boundary.clone();
            h.resolution.as_mut().unwrap().commit(
                Receipt {
                    boundary: row.boundary,
                    mode: if individual {
                        Mode::Individual
                    } else {
                        Mode::Aggregate
                    },
                    metrics: ["food_consumed", "supply_deaths", "soldiers_after_supply"]
                        .into_iter()
                        .enumerate()
                        .map(|(i, name)| Metric {
                            name: name.into(),
                            unit: if i == 0 {
                                "kg food equivalent"
                            } else {
                                "people"
                            }
                            .into(),
                            expected: row.expected[i],
                            actual: row.actual[i],
                            explained: vec![],
                        })
                        .collect(),
                    demographic_snapshot: None,
                },
                &current,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn verify_supply_comparison(baseline: &History) {
    for hungry in [false, true] {
        let mut h = baseline.clone();
        h.month += 1;
        h.resolution = Some(crate::resolution::ResolutionState {
            compare: true,
            ..Default::default()
        });
        let raid = &mut h.society.as_mut().unwrap().raids[0];
        raid.arrives = h.month + 100;
        raid.occupation_until = None;
        raid.returning = false;
        if hungry {
            h.sites[raid.origin as usize].stocks.stock[1] += raid.food;
            raid.food = 0.;
        }
        let before = raid.soldiers as f64;
        let food = raid.food.min(raid.soldiers * 18.) as f64;
        let individual = raid.members.is_some();
        let population_residual = h.population_residual();
        let food_residual = h.food_residual();
        let mut resumed: History =
            serde_json::from_slice(&serde_json::to_vec(&h).unwrap()).unwrap();
        let mut ablated = h.clone();
        ablated.resolution = None;
        h.social_month().unwrap();
        resumed.social_month().unwrap();
        ablated.social_month().unwrap();
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let r = h
            .resolution
            .as_ref()
            .unwrap()
            .receipts
            .iter()
            .find(|r| r.boundary.system == System::MilitarySupply)
            .unwrap();
        assert_eq!(
            r.mode,
            if individual {
                Mode::Individual
            } else {
                Mode::Aggregate
            }
        );
        assert_eq!(r.metrics[0].expected, food);
        assert_eq!(r.metrics[0].actual, food);
        assert!((r.metrics[1].expected - if hungry { before * 0.1 } else { 0. }).abs() < 1e-5);
        assert!((r.metrics[1].actual + r.metrics[2].actual - before).abs() < 1e-5);
        if individual {
            assert_eq!(r.metrics[1].actual.fract(), 0.);
        } else {
            assert!((r.metrics[1].expected - r.metrics[1].actual).abs() < 1e-5);
        }
        let mut physical = h.clone();
        physical.resolution = None;
        assert_eq!(
            serde_json::to_value(&physical).unwrap(),
            serde_json::to_value(&ablated).unwrap()
        );
        assert!((h.population_residual() - population_residual).abs() < 1e-4);
        assert!((h.food_residual() - food_residual).abs() < 1e-4);
        let closed = serde_json::to_value(&h).unwrap();
        assert!(h.social_month().is_err());
        assert_eq!(closed, serde_json::to_value(&h).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn campaign_supply_checkpoint_and_batch_equivalence() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 64,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(16).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        let h = g.civilizations.as_ref().unwrap();
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .find(|r| r.cost_km < 900. && h.controller(r.from) != h.controller(r.to))
            .unwrap()
            .clone();
        g.declare_war(route.from, route.to).unwrap();
        g.civilizations
            .as_mut()
            .unwrap()
            .set_demographic_resolution(Mode::Aggregate, true)
            .unwrap();
        let path = std::path::PathBuf::from(format!(
            "output/military-supply-{}.world",
            std::process::id()
        ));
        g.save(&path).unwrap();
        let mut single = Generator::load(g.gpu.clone(), &path).unwrap();
        g.advance_history(12).unwrap();
        for _ in 0..12 {
            single.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&single.civilizations).unwrap()
        );
        let h = g.civilizations.as_ref().unwrap();
        assert!(h
            .resolution
            .as_ref()
            .unwrap()
            .summaries
            .iter()
            .any(|s| s.system == System::MilitarySupply && s.samples > 0));
        let food = h
            .resolution
            .as_ref()
            .unwrap()
            .summaries
            .iter()
            .find(|s| s.system == System::MilitarySupply && s.name == "food_consumed")
            .unwrap();
        assert!(
            (food.expected - food.actual).abs() < 1e-5,
            "combat or return must not observe supply twice"
        );
        assert!(h.population_residual().abs() < 0.001);
        assert!(h.food_residual().abs() < 0.001);
        std::fs::remove_file(path).unwrap();
    }
}
