//! Diagnostic pooled-care counterfactual; actual family assignments remain authoritative.
use super::*;
use crate::resolution::{Boundary, Metric, Mode, Receipt, System};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CareProjection {
    pub site: u32,
    pub demand: f64,
    pub pooled_capacity: f64,
    pub labor: f64,
}
impl CareProjection {
    fn outcome(&self, month: u32, granted: f64, used: f64, compare: bool) -> Result<Receipt> {
        ensure!(
            [self.demand, self.pooled_capacity, self.labor, granted, used]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.),
            "invalid care resolution inputs"
        );
        let pooled = self.demand.min(self.pooled_capacity).min(self.labor);
        ensure!(
            granted <= pooled + 1e-4 && used <= granted + 1e-4,
            "care result exceeds opening capacity"
        );
        Ok(Receipt {
            boundary: Boundary {
                month,
                system: System::DomesticCare,
                site: self.site,
                subject: self.site,
                revision: crate::resolution::revision(
                    [self.demand, self.pooled_capacity, self.labor].map(f64::to_bits),
                ),
            },
            // Actual care always uses known people, even with aggregate demography.
            mode: Mode::Individual,
            metrics: if compare {
                [
                    (
                        "pooled_care",
                        self.demand,
                        pooled,
                        "opening capacity shortfall",
                    ),
                    (
                        "reserved_care",
                        pooled,
                        granted,
                        "family matching shortfall",
                    ),
                    ("completed_care", granted, used, "unused care reservation"),
                ]
                .into_iter()
                .map(|(name, expected, actual, reason)| Metric {
                    name: name.into(),
                    unit: "worker-months".into(),
                    expected,
                    actual,
                    explained: vec![(reason.into(), actual - expected)],
                })
                .collect()
            } else {
                vec![]
            },
            demographic_snapshot: None,
        })
    }
}
impl History {
    pub(crate) fn settle_care_resolutions(&mut self) -> Result<()> {
        let Some(mut state) = self.resolution.clone() else {
            return Ok(());
        };
        let Some(plan) = self.domestic.as_ref().and_then(|d| d.care.as_ref()) else {
            return Ok(());
        };
        if plan.projections.is_empty() {
            return Ok(());
        }
        ensure!(
            plan.receipt.month == self.month && plan.receipt.settled && plan.reserved,
            "care comparison requires a settled current plan"
        );
        let mut sites = BTreeSet::new();
        for p in &plan.projections {
            ensure!(
                (p.site as usize) < self.sites.len() && sites.insert(p.site),
                "invalid care projection site"
            );
            let mut totals = [0.; 3];
            for row in plan.rows.iter().filter(|r| r.site == p.site) {
                ensure!(
                    [row.need, row.granted, row.used]
                        .iter()
                        .all(|v| v.is_finite() && *v >= 0.)
                        && row.granted <= row.need + 1e-4
                        && row.used <= row.granted + 1e-4,
                    "invalid care row outcome"
                );
                totals[0] += row.need;
                totals[1] += row.granted;
                totals[2] += row.used;
            }
            ensure!(
                (totals[0] - p.demand).abs() <= 1e-4,
                "care demand changed after reservation"
            );
            let receipt = p.outcome(self.month, totals[1], totals[2], state.compare)?;
            if p.demand > 0. {
                let boundary = receipt.boundary.clone();
                state.commit(receipt, &boundary)?;
            }
        }
        ensure!(
            plan.rows.iter().all(|r| sites.contains(&r.site)),
            "missing care projection"
        );
        self.resolution = Some(state);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn care_comparison_separates_pool_family_and_execution_shortfalls() {
        let p = CareProjection {
            site: 0,
            demand: 1.,
            pooled_capacity: 0.8,
            labor: 0.6,
        };
        let r = p.outcome(12, 0.4, 0.3, true).unwrap();
        assert_eq!(
            r.metrics
                .iter()
                .map(|m| (m.expected, m.actual))
                .collect::<Vec<_>>(),
            vec![(1., 0.6), (0.6, 0.4), (0.4, 0.3)]
        );
        assert!(r.metrics.iter().all(|m| m.unexplained().abs() < 1e-12));
        assert!(p.outcome(12, 0.7, 0.3, true).is_err());
        assert!(p.outcome(12, 0.4, 0.5, true).is_err());
        assert!(p.outcome(12, f64::NAN, 0., true).is_err());
        assert!(p.outcome(12, 0.4, 0.3, false).unwrap().metrics.is_empty());
        let mut state = crate::resolution::ResolutionState::default();
        let boundary = r.boundary.clone();
        state.commit(r.clone(), &boundary).unwrap();
        let mut resumed: crate::resolution::ResolutionState =
            serde_json::from_value(serde_json::to_value(state).unwrap()).unwrap();
        assert!(resumed.commit(r, &boundary).is_err());
    }
}
