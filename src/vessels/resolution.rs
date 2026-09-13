//! Conditional forecasts at the two existing crew reservation windows.
//! These observe pooled budgets; they never reserve money, time or vessel capacity.
use crate::resolution::{Boundary, Metric, Mode, Receipt, System};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

const WORK_TOLERANCE_WORKER_MONTHS: f64 = 1e-5;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrewWindow {
    pub demand: f64,
    pub labor: f64,
    pub affordable: f64,
    pub opening_work: f64,
    pub granted: f64,
}
impl CrewWindow {
    pub fn forecast(&self) -> f64 {
        self.demand.min(self.labor).min(self.affordable)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrewProjection {
    pub month: u32,
    pub mode: Mode,
    /// Committed cargo first, discretionary fleet service second. The latter
    /// observes budgets remaining after intervening services, not a parallel world.
    pub windows: [Option<CrewWindow>; 2],
    pub settled: bool,
}
impl CrewProjection {
    pub fn outcome(&self, month: u32, site: u32, used: f64, compare: bool) -> Result<Receipt> {
        ensure!(
            self.month == month && !self.settled,
            "stale crew projection"
        );
        ensure!(
            used.is_finite() && used >= 0.,
            "invalid completed crew work"
        );
        let mut metrics = Vec::new();
        let mut granted = 0.;
        let mut inputs = vec![month as u64, site as u64, self.mode as u64];
        for (i, w) in self.windows.iter().enumerate() {
            inputs.push(i as u64);
            let Some(w) = w else {
                inputs.push(u64::MAX);
                continue;
            };
            let values = [w.demand, w.labor, w.affordable, w.opening_work, w.granted];
            ensure!(
                values.iter().all(|x| x.is_finite() && *x >= 0.),
                "invalid crew forecast"
            );
            ensure!(
                w.granted <= w.forecast() + WORK_TOLERANCE_WORKER_MONTHS,
                "crew grant exceeds pooled forecast"
            );
            inputs.extend(values[..4].iter().map(|x| x.to_bits()));
            granted += w.granted;
            if compare {
                metrics.push(Metric {
                    name: if i == 0 {
                        "committed_crew_grant"
                    } else {
                        "discretionary_crew_grant"
                    }
                    .into(),
                    unit: "worker-months".into(),
                    expected: w.forecast(),
                    actual: w.granted,
                    // A pooled ceiling does not identify whether a missing match
                    // was caused by absence, prior commitments or account eligibility.
                    explained: vec![],
                });
            }
        }
        ensure!(
            used <= granted + WORK_TOLERANCE_WORKER_MONTHS,
            "crew completion exceeds grants"
        );
        if compare {
            metrics.push(Metric {
                name: "completed_crew_work".into(),
                unit: "worker-months".into(),
                expected: granted,
                actual: used,
                explained: vec![("released prepaid crew work".into(), used - granted)],
            });
        }
        Ok(Receipt {
            boundary: Boundary {
                month,
                system: System::MerchantCrew,
                site,
                subject: site,
                revision: crate::resolution::revision(inputs),
            },
            mode: self.mode,
            metrics,
            demographic_snapshot: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn forecast_separates_matching_from_loss_without_creating_work() {
        let p = CrewProjection {
            month: 12,
            mode: Mode::Individual,
            settled: false,
            windows: [
                Some(CrewWindow {
                    demand: 0.5,
                    labor: 0.4,
                    affordable: 0.3,
                    opening_work: 0.,
                    granted: 0.2,
                }),
                Some(CrewWindow {
                    demand: 0.3,
                    labor: 0.1,
                    affordable: 0.2,
                    opening_work: 0.2,
                    granted: 0.1,
                }),
            ],
        };
        let r = p.outcome(12, 2, 0.25, true).unwrap();
        assert_eq!(r.metrics[0].expected, 0.3);
        assert!((r.metrics[0].unexplained() + 0.1).abs() < 1e-10);
        assert!(r.metrics[2].unexplained().abs() < 1e-10);
        assert!(p.outcome(13, 2, 0.25, true).is_err());
        assert!(p.outcome(12, 2, 0.4, true).is_err());
        let restored: CrewProjection =
            serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(r).unwrap(),
            serde_json::to_value(restored.outcome(12, 2, 0.25, true).unwrap()).unwrap()
        );
        let mut invalid = p.clone();
        invalid.windows[0].as_mut().unwrap().granted = 1.;
        assert!(invalid.outcome(12, 2, 0.25, true).is_err());
        invalid = p;
        invalid.settled = true;
        assert!(invalid.outcome(12, 2, 0.25, true).is_err());
    }
}
