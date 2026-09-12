//! Evidence about an actually encountered route closure, delivered by survivors.
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encounter {
    pub from: u32,
    pub to: u32,
    pub observed: u32,
    pub cause: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Warning {
    pub observer: u32,
    pub received: u32,
    pub arrival: u64,
    pub encounter: Encounter,
}
impl Warning {
    pub fn penalty(&self, month: u32) -> f32 {
        if month < self.received || month < self.encounter.observed {
            return 1.;
        }
        1. - 0.6 / (1. + (month - self.encounter.observed) as f32 / 6.)
    }
}
impl crate::social_memory::LocalMemory {
    pub fn route_warning(&mut self, report: Warning) {
        if report.encounter.observed > report.received
            || report.encounter.from == report.encounter.to
        {
            return;
        }
        let key = |r: &Warning| {
            (
                r.observer,
                r.encounter.from.min(r.encounter.to),
                r.encounter.from.max(r.encounter.to),
            )
        };
        if let Some(old) = self.warnings.iter_mut().find(|r| key(r) == key(&report)) {
            if (report.encounter.observed, report.arrival) > (old.encounter.observed, old.arrival) {
                *old = report;
            }
        } else {
            self.warnings.push(report);
        }
    }
    pub fn route_preference(&self, observer: u32, destination: u32, month: u32) -> f32 {
        self.warnings
            .iter()
            .filter(|r| {
                r.observer == observer
                    && ((r.encounter.from == observer && r.encounter.to == destination)
                        || (r.encounter.to == observer && r.encounter.from == destination))
            })
            .map(|r| r.penalty(month))
            .fold(1., f32::min)
    }
}
pub(crate) fn validate(h: &History, warnings: &[Warning]) -> Result<()> {
    let mut keys = std::collections::BTreeSet::new();
    for w in warnings {
        let e = &w.encounter;
        ensure!(
            [w.observer, e.from, e.to]
                .iter()
                .all(|s| (*s as usize) < h.sites.len())
                && e.from != e.to
                && e.observed <= w.received
                && w.received <= h.month
                && (e.cause as usize) < h.events.len()
                && (w.arrival as usize) < h.events.len()
                && keys.insert((w.observer, e.from.min(e.to), e.from.max(e.to))),
            "invalid route warning"
        );
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn warning_is_local_delayed_aging_and_idempotent() {
        let mut m = crate::social_memory::LocalMemory::default();
        let report = Warning {
            observer: 2,
            received: 8,
            arrival: 12,
            encounter: Encounter {
                from: 1,
                to: 2,
                observed: 5,
                cause: 10,
            },
        };
        m.route_warning(report.clone());
        m.route_warning(report);
        assert_eq!(m.warnings.len(), 1);
        assert_eq!(m.route_preference(2, 1, 7), 1.);
        assert_eq!(m.route_preference(1, 2, 8), 1.);
        assert!(m.route_preference(2, 1, 8) < 1.);
        assert!(m.route_preference(2, 1, 80) > m.route_preference(2, 1, 8));
        let restored: crate::social_memory::LocalMemory =
            serde_json::from_slice(&serde_json::to_vec(&m).unwrap()).unwrap();
        assert_eq!(
            m.route_preference(2, 1, 10),
            restored.route_preference(2, 1, 10)
        );
    }
}
