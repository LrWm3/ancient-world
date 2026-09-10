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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LocalMemory {
    pub reports: Vec<Report>,
    /// A practical custom, separate from patron ancestry and religious affiliation.
    pub mutual_aid_sites: Vec<u32>,
}
impl LocalMemory {
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
            if c.religious_relief.enabled {
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
    }
    pub(crate) fn destination_memory(&self, observer: u32, destination: u32) -> f32 {
        self.culture
            .as_ref()
            .filter(|c| c.religious_relief.enabled)
            .map_or(1., |c| {
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
