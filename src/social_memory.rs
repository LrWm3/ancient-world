//! Dated, locally received testimony. No remote state is refreshed implicitly.
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub observer: u32,
    pub destination: u32,
    pub observed: u32,
    pub received: u32,
    pub food_months: f32,
    pub cause: u64,
}
impl Report {
    pub fn weight(&self, month: u32) -> f32 {
        1. / (1. + month.saturating_sub(self.observed) as f32 / 12.)
    }
    pub fn preference(&self, month: u32) -> f32 {
        1. + 0.25 * self.weight(month) * ((self.food_months - 6.) / 6.).clamp(-1., 1.)
    }
}
/// Observed delivery outcomes, kept per recipient/donor pair rather than global reputation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AidMemory {
    pub recipient: u32,
    pub donor: u32,
    pub received: u32,
    pub cause: u64,
    pub delivered_kg: f32,
    pub successes: u32,
    pub failures: u32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LocalMemory {
    #[serde(default)]
    pub aid: Vec<AidMemory>,
    pub reports: Vec<Report>,
    /// A practical custom, separate from patron ancestry and religious affiliation.
    pub mutual_aid_sites: Vec<u32>,
}
impl LocalMemory {
    pub fn record_aid(&mut self, recipient: u32, donor: u32, month: u32, cause: u64, kg: f32) {
        let i = self
            .aid
            .iter()
            .position(|a| a.recipient == recipient && a.donor == donor)
            .unwrap_or_else(|| {
                self.aid.push(AidMemory {
                    recipient,
                    donor,
                    received: month,
                    cause,
                    delivered_kg: 0.,
                    successes: 0,
                    failures: 0,
                });
                self.aid.len() - 1
            });
        let a = &mut self.aid[i];
        if a.successes + a.failures > 0 && cause <= a.cause {
            return;
        }
        a.received = month;
        a.cause = cause;
        a.delivered_kg += kg;
        if kg >= 18. {
            a.successes += 1;
        } else {
            a.failures += 1;
        }
        if a.successes >= 2 && !self.mutual_aid_sites.contains(&recipient) {
            self.mutual_aid_sites.push(recipient);
        }
    }
    pub fn reciprocity(&self, recipient: u32, donor: u32, month: u32) -> f32 {
        self.aid
            .iter()
            .find(|a| a.recipient == recipient && a.donor == donor)
            .map_or(0., |a| {
                let confidence = a.successes as f32 / (2. + a.successes as f32 + a.failures as f32);
                0.25 * confidence / (1. + month.saturating_sub(a.received) as f32 / 120.)
            })
    }
    pub fn remember(&mut self, report: Report) {
        if let Some(old) = self
            .reports
            .iter_mut()
            .find(|r| r.observer == report.observer && r.destination == report.destination)
        {
            if report.observed >= old.observed {
                *old = report;
            }
        } else {
            self.reports.push(report);
        }
    }
    pub fn preference(&self, observer: u32, destination: u32, month: u32) -> f32 {
        self.reports
            .iter()
            .find(|r| r.observer == observer && r.destination == destination)
            .map_or(1., |r| r.preference(month))
    }
    pub fn validate(&self, h: &History) -> Result<()> {
        let mut aid_pairs = std::collections::BTreeSet::new();
        for a in &self.aid {
            ensure!(
                (a.recipient as usize) < h.sites.len()
                    && (a.donor as usize) < h.sites.len()
                    && a.recipient != a.donor
                    && a.received <= h.month
                    && a.delivered_kg.is_finite()
                    && a.delivered_kg >= 0.
                    && h.events.get(a.cause as usize).is_some()
                    && aid_pairs.insert((a.recipient, a.donor)),
                "invalid aid memory"
            );
        }
        let mut pairs = std::collections::BTreeSet::new();
        for r in &self.reports {
            ensure!(
                (r.observer as usize) < h.sites.len()
                    && (r.destination as usize) < h.sites.len()
                    && r.observer != r.destination
                    && r.observed <= r.received
                    && r.received <= h.month
                    && r.food_months.is_finite()
                    && (0. ..=24.).contains(&r.food_months)
                    && h.events.get(r.cause as usize).is_some()
                    && pairs.insert((r.observer, r.destination)),
                "invalid local report"
            );
        }
        let mut sites = std::collections::BTreeSet::new();
        ensure!(
            self.mutual_aid_sites
                .iter()
                .all(|&s| (s as usize) < h.sites.len() && sites.insert(s)),
            "invalid learned practice"
        );
        Ok(())
    }
}
impl History {
    pub(crate) fn remember_relief(
        &mut self,
        shipment: &crate::civilization::Shipment,
        delivered: bool,
        cause: u64,
    ) {
        if let Some(c) = &mut self.culture {
            let previous = c
                .religious_relief
                .memory
                .mutual_aid_sites
                .contains(&shipment.to);
            c.religious_relief.memory.record_aid(
                shipment.to,
                shipment.from,
                self.month,
                cause,
                if delivered { shipment.food_kg } else { 0. },
            );
            if !previous
                && c.religious_relief
                    .memory
                    .mutual_aid_sites
                    .contains(&shipment.to)
            {
                self.event("mutual_aid_learned",Some(shipment.to),Some(shipment.from),
                    "Repeated delivered assistance established a local mutual-aid practice; affiliation remains unchanged".into());
                self.events.last_mut().unwrap().causes.push(cause);
            }
        }
    }
    pub(crate) fn remember_arrival(
        &mut self,
        observer: u32,
        destination: u32,
        observed: u32,
        food: Option<f32>,
        cause: u64,
    ) {
        if observer == destination {
            return;
        }
        if let (Some(c), Some(food_months)) = (&mut self.culture, food) {
            c.religious_relief.memory.remember(Report {
                observer,
                destination,
                observed,
                received: self.month,
                food_months,
                cause,
            });
        }
    }
    pub(crate) fn destination_memory(&self, observer: u32, destination: u32) -> f32 {
        self.culture.as_ref().map_or(1., |c| {
            c.religious_relief
                .memory
                .preference(observer, destination, self.month)
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aid_memory_requires_delivery_and_is_local_dated_and_idempotent() {
        let mut m = LocalMemory::default();
        m.record_aid(1, 2, 12, 4, 100.);
        m.record_aid(1, 2, 12, 4, 100.);
        assert_eq!(m.aid[0].delivered_kg, 100.);
        assert!(m.reciprocity(1, 2, 12) > 0.);
        assert_eq!(m.reciprocity(2, 1, 12), 0.);
        assert!(m.reciprocity(1, 2, 120) < m.reciprocity(1, 2, 12));
        m.record_aid(1, 2, 13, 5, 0.);
        assert!(!m.mutual_aid_sites.contains(&1));
        m.record_aid(1, 2, 14, 6, 100.);
        assert!(m.mutual_aid_sites.contains(&1));
        let restored: LocalMemory =
            serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
        assert_eq!(restored.aid[0].successes, 2);
    }
    #[test]
    fn reports_are_local_dated_and_do_not_refresh_from_old_testimony() {
        let mut m = LocalMemory::default();
        let r = Report {
            observer: 0,
            destination: 1,
            observed: 12,
            received: 14,
            food_months: 0.,
            cause: 0,
        };
        m.remember(r.clone());
        assert!(m.preference(0, 1, 14) < 1.);
        assert_eq!(m.preference(2, 1, 14), 1.);
        assert!(m.preference(0, 1, 120) > m.preference(0, 1, 14));
        m.remember(Report {
            observed: 1,
            received: 15,
            food_months: 24.,
            ..r.clone()
        });
        assert_eq!(m.reports[0].observed, 12);
        m.remember(Report {
            observed: 16,
            received: 17,
            food_months: 24.,
            ..r
        });
        assert_eq!(m.reports.len(), 1);
        assert!(m.preference(0, 1, 17) > 1.);
        let restored: LocalMemory =
            serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
        assert_eq!(restored.preference(0, 1, 20), m.preference(0, 1, 20));
    }
}
