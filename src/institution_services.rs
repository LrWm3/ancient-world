//! Dated room-capacity reservations for institutional services.
//!
//! Capacity is abstract usable room units multiplied by a fraction of one month,
//! not square metres or worker time. This ledger cannot grant personnel, funds or
//! materials. Callers must establish those before reserving a service and recheck
//! them before execution. Allocation order is explicit in the request sequence.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Service {
    Lesson {
        student: u32,
        teacher: u32,
        topic: u32,
    },
    HeritageStudy {
        artifact: u32,
        author: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    pub service: Service,
    pub occupants: f64,
    pub duration: f64,
    pub requested: f64,
    pub granted: f64,
    pub used: f64,
    pub settled: bool,
}
impl Receipt {
    pub fn released(&self) -> f64 {
        if self.settled {
            self.granted - self.used
        } else {
            0.
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub month: u32,
    pub site: u32,
    pub institution: u32,
    pub opening_space: f64,
    pub receipts: Vec<Receipt>,
    pub closed: bool,
}
impl Plan {
    pub fn new(month: u32, site: u32, institution: u32, usable_space: f64) -> Self {
        Self {
            month,
            site,
            institution,
            opening_space: if usable_space.is_finite() {
                usable_space.max(0.)
            } else {
                0.
            },
            receipts: Vec::new(),
            closed: false,
        }
    }

    /// One working group must fit at once, as well as within monthly room time.
    /// Requests are indivisible. An unfunded request leaves capacity for others.
    pub fn reserve(&mut self, service: Service, occupants: f64, duration: f64) -> Option<usize> {
        if self.closed
            || self.receipts.iter().any(|r| r.settled)
            || !occupants.is_finite()
            || occupants <= 0.
            || !duration.is_finite()
            || duration <= 0.
            || duration > 1.
            || self.receipts.iter().any(|r| r.service == service)
        {
            return None;
        }
        let requested = occupants * duration;
        if !requested.is_finite() {
            return None;
        }
        let reserved: f64 = self.receipts.iter().map(|r| r.granted).sum();
        let granted = if occupants <= self.opening_space
            && requested <= (self.opening_space - reserved).max(0.)
        {
            requested
        } else {
            0.
        };
        let id = self.receipts.len();
        self.receipts.push(Receipt {
            service,
            occupants,
            duration,
            requested,
            granted,
            used: 0.,
            settled: false,
        });
        Some(id)
    }

    /// Consume a grant only at its captured boundary, after live eligibility checks.
    /// Lost space can invalidate later execution; newly built space cannot enlarge
    /// opening grants. Failure releases this grant for reporting, never reallocation.
    pub fn settle(
        &mut self,
        boundary: (u32, u32, u32),
        id: usize,
        live_space: f64,
        eligible: bool,
    ) -> bool {
        if self.closed || boundary != (self.month, self.site, self.institution) {
            return false;
        }
        let used: f64 = self.receipts.iter().map(|r| r.used).sum();
        let Some(r) = self.receipts.get_mut(id).filter(|r| !r.settled) else {
            return false;
        };
        r.settled = true;
        if eligible
            && live_space.is_finite()
            && r.granted > 0.
            && r.occupants <= live_space
            && r.granted <= (live_space.min(self.opening_space) - used).max(0.)
        {
            r.used = r.granted;
            true
        } else {
            false
        }
    }

    pub fn close(&mut self, month: u32) {
        if month == self.month {
            for r in &mut self.receipts {
                r.settled = true;
            }
            self.closed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn lesson(student: u32) -> Service {
        Service::Lesson {
            student,
            teacher: 9,
            topic: 2,
        }
    }
    #[test]
    fn groups_compete_for_finite_room_time_and_physical_space() {
        let mut small = Plan::new(12, 1, 3, 2.);
        let a = small.reserve(lesson(1), 2., 0.75).unwrap();
        let b = small.reserve(lesson(2), 2., 0.5).unwrap();
        assert_eq!(small.receipts[a].granted, 1.5);
        assert_eq!(small.receipts[b].granted, 0.);
        // A short meeting still cannot fit an oversized group.
        let c = small.reserve(lesson(3), 3., 0.01).unwrap();
        assert_eq!(small.receipts[c].granted, 0.);
        let d = small
            .reserve(
                Service::HeritageStudy {
                    artifact: 5,
                    author: 4,
                },
                1.,
                0.5,
            )
            .unwrap();
        assert_eq!(small.receipts[d].granted, 0.5);
        assert!(small.settle((12, 1, 3), a, 2., true));
        assert!(small.settle((12, 1, 3), d, 2., true));
        assert_eq!(small.receipts.iter().map(|r| r.used).sum::<f64>(), 2.);
        let mut large = Plan::new(12, 1, 3, 4.);
        large.reserve(lesson(1), 2., 0.75);
        large.reserve(lesson(2), 2., 0.5);
        assert_eq!(large.receipts.iter().map(|r| r.granted).sum::<f64>(), 2.5);
    }
    #[test]
    fn absence_damage_stale_and_duplicate_execution_do_not_create_capacity() {
        let mut p = Plan::new(12, 1, 3, 2.);
        let a = p.reserve(lesson(1), 2., 0.5).unwrap();
        let b = p.reserve(lesson(2), 2., 0.5).unwrap();
        assert!(p.reserve(lesson(1), 2., 0.5).is_none());
        assert!(!p.settle((13, 1, 3), a, 2., true));
        assert!(!p.settle((12, 2, 3), a, 2., true));
        assert!(!p.settle((12, 1, 4), a, 2., true));
        assert!(!p.receipts[a].settled);
        assert!(!p.settle((12, 1, 3), a, 2., false));
        assert!(!p.settle((12, 1, 3), a, 2., true));
        assert_eq!(p.receipts[a].released(), 1.);
        assert!(p.reserve(lesson(3), 1., 0.5).is_none());
        assert!(!p.settle((12, 1, 3), b, 1., true));
        assert_eq!(p.receipts.iter().map(|r| r.used).sum::<f64>(), 0.);
    }
    #[test]
    fn continuation_and_close_preserve_receipts() {
        let mut p = Plan::new(12, 1, 3, 2.);
        p.reserve(lesson(1), 2., 0.5);
        p.reserve(lesson(2), 2., 0.5);
        assert!(p.settle((12, 1, 3), 0, 2., true));
        let mut restored: Plan = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        for plan in [&mut p, &mut restored] {
            assert!(plan.settle((12, 1, 3), 1, 2., true));
            plan.close(12);
            assert!(plan.reserve(lesson(4), 1., 0.1).is_none());
        }
        assert_eq!(p, restored);
        let mut invalid = Plan::new(0, 0, 0, f64::NAN);
        assert_eq!(invalid.opening_space, 0.);
        assert!(invalid.reserve(lesson(1), f64::INFINITY, 1.).is_none());
        assert!(invalid.reserve(lesson(1), 1., -1.).is_none());
    }
}
