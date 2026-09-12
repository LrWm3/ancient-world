//! Opening travel forecasts; bounded receipts observe existing transfers, never apply them.
use super::*;
use crate::resolution::{Boundary, Metric, Mode, Receipt, System};
use std::collections::BTreeMap;

struct Row {
    boundary: Boundary,
    expected: [f64; 3],
    actual: [f64; 3],
}
pub(super) struct TravelComparison(BTreeMap<(u32, bool), Row>);
impl TravelComparison {
    pub fn capture(h: &History) -> Result<Self> {
        let mut rows: BTreeMap<(u32, bool), Row> = BTreeMap::new();
        if h.resolution.is_some() {
            for j in h.society.iter().flat_map(|s| &s.relocation.journeys) {
                let individual = h.individual_demography_enabled() && j.roster.is_some();
                let need = j
                    .cohorts
                    .iter()
                    .zip([10., 18., 14.])
                    .map(|(n, r)| n * r)
                    .sum::<f32>();
                let food = j.food.min(need);
                let rate = if food + 0.001 < need {
                    0.08 * (1. - food / need.max(0.001))
                } else {
                    0.
                };
                let pop = j.population();
                let row = rows.entry((j.from, individual)).or_insert_with(|| Row {
                    boundary: Boundary {
                        month: h.month,
                        system: System::RelocationTravel,
                        site: j.from,
                        subject: u32::from(individual),
                        revision: h.seed as u64,
                    },
                    expected: [0.; 3],
                    actual: [0.; 3],
                });
                row.boundary.revision = crate::resolution::revision([
                    row.boundary.revision,
                    j.household as u64,
                    j.departed as u64,
                    j.food.to_bits() as u64,
                    j.cohorts[0].to_bits() as u64,
                    j.cohorts[1].to_bits() as u64,
                    j.cohorts[2].to_bits() as u64,
                ]);
                if let Some(roster) = &j.roster {
                    for p in &roster.passengers {
                        row.boundary.revision = crate::resolution::revision([
                            row.boundary.revision,
                            p.person as u64,
                            p.band as u64,
                            u64::from(h.people[p.person as usize].died.is_some()),
                        ]);
                    }
                }
                row.expected[0] += food as f64;
                row.expected[1] += pop as f64 * rate as f64;
                row.expected[2] += pop as f64 * (1. - rate as f64);
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
    pub fn observe(&mut self, j: &Journey, individual: bool, before: f32, eaten: f32) {
        if let Some(row) = self.0.get_mut(&(j.from, individual)) {
            // Existing extinction cleanup also removes tiny residual cohorts.
            let remaining = if j.population() < 0.01 {
                0.
            } else {
                j.population()
            };
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
                    metrics: ["food_consumed", "travel_deaths", "travelers_after_losses"]
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
