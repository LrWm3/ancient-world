//! Compare captured work expectations with reservations and completed work.
//! These receipts observe existing execution; they never apply a second output.
use crate::resolution::{Boundary, Metric, Mode, Receipt, System};
use anyhow::{ensure, Result};

fn outcome(
    month: u32,
    site: u32,
    system: System,
    mode: Mode,
    work: [f64; 4], // requested, allocated, reserved, completed
    compare: bool,
) -> Result<Receipt> {
    ensure!(
        work.iter().all(|v| v.is_finite() && *v >= 0.),
        "invalid learning work"
    );
    ensure!(
        work.windows(2).all(|v| v[1] <= v[0] + 1e-4),
        "learning work exceeds its grant"
    );
    let boundary = Boundary {
        month,
        system,
        site,
        subject: site,
        revision: crate::resolution::revision(work[..3].iter().map(|v| v.to_bits())),
    };
    let metrics = if compare {
        [
            ("allocated_work", "allocation shortfall", work[0], work[1]),
            ("reserved_work", "reservation shortfall", work[1], work[2]),
            ("completed_work", "unused reserved work", work[2], work[3]),
        ]
        .into_iter()
        .map(|(name, reason, expected, actual)| Metric {
            name: name.into(),
            unit: "worker-months".into(),
            expected,
            actual,
            explained: vec![(reason.into(), actual - expected)],
        })
        .collect()
    } else {
        vec![]
    };
    Ok(Receipt {
        boundary,
        mode,
        metrics,
        demographic_snapshot: None,
    })
}

impl crate::civilization::History {
    pub(crate) fn settle_learning_resolutions(&mut self) -> Result<()> {
        let Some(mut resolution) = self.resolution.clone() else {
            return Ok(());
        };
        // Stage all receipts before replacing state; failure cannot leave a partial batch.
        for r in &self.service_allocation.receipts {
            let Some(mode) = r.resolution_mode else {
                continue;
            };
            ensure!(r.month == self.month, "stale learning allocation");
            for (k, system) in [System::Research, System::Culture].into_iter().enumerate() {
                if r.requested[k] <= 0. {
                    continue;
                }
                let (requested, granted, used, month) = if k == 0 {
                    let p = self
                        .expeditions
                        .as_ref()
                        .and_then(|x| x.discoveries.as_ref())
                        .and_then(|d| d.workshops.iter().find(|w| w.site == r.site))
                        .and_then(|w| w.work_plan.as_ref())
                        .ok_or_else(|| anyhow::anyhow!("missing research result"))?;
                    ensure!(p.receipt.settled, "research work is not settled");
                    (
                        p.receipt.requested,
                        p.receipt.granted,
                        p.receipt.used,
                        p.receipt.month,
                    )
                } else {
                    let p = self
                        .culture
                        .as_ref()
                        .and_then(|c| c.work_plans.iter().find(|p| p.site == r.site))
                        .ok_or_else(|| anyhow::anyhow!("missing cultural result"))?;
                    (
                        p.actions.iter().map(|(_, w)| *w).sum::<f32>().min(0.5) as f64,
                        p.granted as f64,
                        p.completed as f64,
                        p.month,
                    )
                };
                ensure!(
                    month == r.month
                        && (requested - r.requested[k] as f64).abs() <= 1e-4
                        && (granted - r.reserved[k] as f64).abs() <= 1e-4,
                    "learning result no longer matches reservation"
                );
                let mut receipt = outcome(
                    r.month,
                    r.site,
                    system,
                    mode,
                    [r.requested[k] as f64, r.allocated[k] as f64, granted, used],
                    resolution.compare,
                )?;
                if k == 1 && resolution.compare {
                    if let Some(lesson) = self
                        .culture
                        .as_ref()
                        .and_then(|c| c.work_plans.iter().find(|p| p.site == r.site))
                        .and_then(|p| p.successor_expectation.as_ref())
                    {
                        receipt.metrics.extend([
                            Metric {
                                name: "successor_learning_gain".into(),
                                unit: "topic fraction".into(),
                                expected: lesson.expected_gain as f64,
                                actual: lesson.actual_gain as f64,
                                explained: vec![],
                            },
                            Metric {
                                name: "successor_acquisition".into(),
                                unit: "topics".into(),
                                expected: f64::from(lesson.expected_acquisition),
                                actual: f64::from(lesson.actual_acquisition),
                                explained: vec![],
                            },
                        ]);
                    }
                }
                let boundary = receipt.boundary.clone();
                resolution.commit(receipt, &boundary)?;
            }
        }
        self.resolution = Some(resolution);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expectations_separate_policy_matching_and_execution() {
        let r = outcome(
            12,
            0,
            System::Culture,
            Mode::Individual,
            [1., 0.8, 0.5, 0.2],
            true,
        )
        .unwrap();
        assert_eq!(r.metrics.len(), 3);
        for m in &r.metrics {
            assert!(m.unexplained().abs() < 1e-12);
        }
        assert_eq!(r.metrics[1].expected, 0.8);
        assert_eq!(r.metrics[1].actual, 0.5);
        let no_people = outcome(
            12,
            0,
            System::Research,
            Mode::Individual,
            [1., 0.8, 0., 0.],
            true,
        )
        .unwrap();
        assert_eq!(no_people.metrics[2].actual, 0.);
        let aggregate = outcome(
            12,
            0,
            System::Research,
            Mode::Aggregate,
            [1., 0.8, 0.8, 0.2],
            true,
        )
        .unwrap();
        assert_eq!(aggregate.metrics[1].unexplained(), 0.);
    }
    #[test]
    fn bounded_results_duplicate_guards_and_serialization() {
        assert!(outcome(
            1,
            0,
            System::Culture,
            Mode::Individual,
            [1., 0.5, 0.6, 0.],
            true
        )
        .is_err());
        assert!(outcome(
            1,
            0,
            System::Culture,
            Mode::Individual,
            [1., 0.5, 0.5, f64::NAN],
            true
        )
        .is_err());
        let r = outcome(
            1,
            0,
            System::Research,
            Mode::Aggregate,
            [1., 0.5, 0.5, 0.2],
            false,
        )
        .unwrap();
        assert!(r.metrics.is_empty());
        let b = r.boundary.clone();
        let mut state = crate::resolution::ResolutionState::default();
        state.commit(r.clone(), &b).unwrap();
        let mut restored: crate::resolution::ResolutionState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert!(restored.commit(r, &b).is_err());
        restored.validate(1, 1).unwrap();
    }
}
