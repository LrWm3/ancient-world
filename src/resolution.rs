//! Shared boundary contract. Projections never mutate inventories; one result commits.
pub use crate::individual_demography::{
    DemographicComparison, DemographicOutcome, DemographicProjection, DemographicSnapshot,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Aggregate,
    Individual,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum System {
    Demography,
    Workshop,
    Research,
    Culture,
    DomesticCare,
    MerchantCrew,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boundary {
    pub month: u32,
    pub system: System,
    pub site: u32,
    pub subject: u32,
    pub revision: u64,
}
/// Deterministic numeric input fingerprint; not an identity or a security hash.
pub(crate) fn revision(values: impl IntoIterator<Item = u64>) -> u64 {
    values.into_iter().fold(0xcbf29ce484222325, |h, x| {
        (h ^ x).wrapping_mul(0x100000001b3)
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub unit: String,
    pub expected: f64,
    pub actual: f64,
    /// Mechanically attributable differences, not post-hoc narratives.
    pub explained: Vec<(String, f64)>,
}
impl Metric {
    pub fn unexplained(&self) -> f64 {
        self.actual - self.expected - self.explained.iter().map(|(_, v)| v).sum::<f64>()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub boundary: Boundary,
    pub mode: Mode,
    pub metrics: Vec<Metric>,
    /// Optional replay inputs; retained only with the latest monthly receipts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub demographic_snapshot: Option<DemographicSnapshot>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Summary {
    pub system: System,
    pub mode: Mode,
    pub name: String,
    pub unit: String,
    pub samples: u64,
    pub expected: f64,
    pub actual: f64,
    pub absolute_error: f64,
    pub squared_error: f64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResolutionState {
    #[serde(default)]
    pub summaries: Vec<Summary>,
    pub compare: bool,
    pub workshop_individual: bool,
    /// Only the latest month's receipts, bounded by sites and converted work categories.
    pub receipts: Vec<Receipt>,
}
impl ResolutionState {
    pub(crate) fn check(&self, boundary: &Boundary, current: &Boundary) -> Result<()> {
        ensure!(boundary == current, "stale resolution inputs");
        ensure!(
            self.receipts
                .iter()
                .all(|r| r.boundary.month <= boundary.month),
            "resolution clock moved backwards"
        );
        ensure!(
            !self
                .receipts
                .iter()
                .any(|r| r.boundary.month == boundary.month
                    && r.boundary.system == boundary.system
                    && r.boundary.site == boundary.site
                    && r.boundary.subject == boundary.subject),
            "resolution already committed"
        );
        Ok(())
    }
    pub(crate) fn commit(&mut self, receipt: Receipt, current: &Boundary) -> Result<()> {
        self.check(&receipt.boundary, current)?;
        ensure!(
            receipt.metrics.iter().all(|m| m.expected.is_finite()
                && m.actual.is_finite()
                && m.explained.iter().all(|(_, v)| v.is_finite())),
            "nonfinite resolution result"
        );
        for m in &receipt.metrics {
            let index = self
                .summaries
                .iter()
                .position(|s| {
                    s.system == receipt.boundary.system
                        && s.mode == receipt.mode
                        && s.name == m.name
                        && s.unit == m.unit
                })
                .unwrap_or_else(|| {
                    self.summaries.push(Summary {
                        system: receipt.boundary.system,
                        mode: receipt.mode,
                        name: m.name.clone(),
                        unit: m.unit.clone(),
                        samples: 0,
                        expected: 0.,
                        actual: 0.,
                        absolute_error: 0.,
                        squared_error: 0.,
                    });
                    self.summaries.len() - 1
                });
            let s = &mut self.summaries[index];
            let error = m.actual - m.expected;
            s.samples += 1;
            s.expected += m.expected;
            s.actual += m.actual;
            s.absolute_error += error.abs();
            s.squared_error += error * error;
        }
        self.receipts
            .retain(|r| r.boundary.month == receipt.boundary.month);
        self.receipts.push(receipt);
        Ok(())
    }
    pub(crate) fn validate(&self, month: u32, sites: usize) -> Result<()> {
        ensure!(
            self.receipts.len() <= sites * 9,
            "unbounded resolution receipts"
        );
        ensure!(
            self.summaries.len() <= 64
                && self.summaries.iter().all(|s| [
                    s.expected,
                    s.actual,
                    s.absolute_error,
                    s.squared_error
                ]
                .iter()
                .all(|v| v.is_finite())),
            "invalid comparison summaries"
        );
        for (i, r) in self.receipts.iter().enumerate() {
            if let Some(snapshot) = &r.demographic_snapshot {
                ensure!(
                    r.boundary.system == System::Demography && snapshot.month == r.boundary.month,
                    "snapshot does not belong to receipt"
                );
                snapshot.validate()?;
            }
            ensure!(
                r.boundary.month <= month && (r.boundary.site as usize) < sites,
                "invalid resolution boundary"
            );
            ensure!(
                r.metrics.len() <= 8
                    && r.metrics.iter().all(|m| m.expected.is_finite()
                        && m.actual.is_finite()
                        && m.explained.iter().all(|(_, v)| v.is_finite())),
                "invalid reconciliation metrics"
            );
            ensure!(
                !self.receipts[..i]
                    .iter()
                    .any(|p| p.boundary.month == r.boundary.month
                        && p.boundary.system == r.boundary.system
                        && p.boundary.site == r.boundary.site
                        && p.boundary.subject == r.boundary.subject),
                "duplicate resolution receipt"
            );
        }
        Ok(())
    }
}
impl crate::civilization::History {
    /// Aggregate CPU demographic resolver using the same projection as refinement.
    /// Conversion to individuals is explicit and may reject incompatible old stocks.
    pub fn set_demographic_resolution(&mut self, mode: Mode, compare: bool) -> Result<()> {
        ensure!(
            self.society.is_some() && self.politics.is_some(),
            "demographic resolution needs society and politics"
        );
        ensure!(
            self.participation
                .as_ref()
                .is_none_or(|p| p.commitments.iter().all(|c| c.settled)),
            "unsettled personal work"
        );
        if mode == Mode::Individual {
            self.enable_individual_demography()?;
        } else {
            ensure!(
                !self
                    .resolution
                    .as_ref()
                    .is_some_and(|s| s.workshop_individual),
                "disable workshop refinement before aggregate demography"
            );
            if let Some(n) = &mut self.named_demography {
                n.individual = false;
            }
        }
        self.resolution.get_or_insert_with(Default::default).compare = compare;
        Ok(())
    }
    pub fn resolution_report(&self) -> serde_json::Value {
        serde_json::json!({"state":self.resolution,"demographic_mode":if self.individual_demography_enabled(){Mode::Individual}else{Mode::Aggregate}})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundary_rejects_stale_duplicate_and_backward_commits_after_serialization() {
        let b = Boundary {
            month: 3,
            system: System::Demography,
            site: 0,
            subject: 0,
            revision: 9,
        };
        let r = Receipt {
            boundary: b.clone(),
            mode: Mode::Aggregate,
            metrics: vec![],
            demographic_snapshot: None,
        };
        let mut s = ResolutionState::default();
        let mut stale = b.clone();
        stale.revision += 1;
        assert!(s.commit(r.clone(), &stale).is_err());
        assert!(s.receipts.is_empty());
        s.commit(r.clone(), &b).unwrap();
        let mut s: ResolutionState =
            serde_json::from_value(serde_json::to_value(s).unwrap()).unwrap();
        assert!(s.commit(r, &b).is_err());
        let mut old = b.clone();
        old.month = 2;
        assert!(s.check(&old, &old).is_err());
    }
}
