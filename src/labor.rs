//! Shared CPU reservation ceiling, in effective worker-months.
//! Match workforce modifiers in society.wgsl::workers and ecological_production.
use crate::civilization::Site;

fn service_capacity(
    population: f32,
    adults: f32,
    illness: f32,
    recovery: f32,
    society: bool,
    living: bool,
) -> f32 {
    let workers = if society {
        adults.max(0.) * 0.8 * (1. - 0.5 * illness.clamp(0., 0.5))
    } else {
        population.max(0.) * 0.5
    };
    let recovery = if living { recovery.clamp(0., 1.) } else { 0. };
    workers * (1. - 0.4 * recovery) * 0.2
}

fn remaining(capacity: f32, external: f32, enterprise: [f32; 4]) -> f32 {
    (capacity - external.max(0.) - enterprise.iter().sum::<f32>()).max(0.)
}

/// Existing reservations have priority. This is a ceiling, not a promise that
/// materials, orders, cash or the GPU craft allocation will permit all work.
pub(crate) fn available(site: &Site, society: bool, living: bool) -> f32 {
    if site.abandoned {
        return 0.;
    }
    remaining(
        service_capacity(
            site.stocks.stock[0],
            site.demography.ages[1],
            site.demography.health[0],
            site.economy.soil[3],
            society,
            living,
        ),
        site.economy.external[3],
        site.economy.enterprise_plan,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workforce_flags_match_production_contract() {
        assert_eq!(service_capacity(200., 100., 0., 0., true, true), 16.);
        assert_eq!(service_capacity(200., 100., 0.5, 0., true, true), 12.);
        assert!((service_capacity(200., 100., 0.5, 1., true, true) - 7.2).abs() < 1e-6);
        assert_eq!(service_capacity(200., 100., 0.5, 1., true, false), 12.);
        assert_eq!(service_capacity(200., 100., 0.5, 1., false, false), 20.);
        assert_eq!(service_capacity(200., 0., 0., 0., true, true), 0.);
    }
    #[test]
    fn earlier_reservations_reduce_later_grants() {
        let capacity = service_capacity(200., 100., 0., 0., true, true);
        assert_eq!(remaining(capacity, 2.5, [3., 4., 0., 0.]), 6.5);
        assert_eq!(remaining(capacity, 20., [0.; 4]), 0.);
        assert_eq!(remaining(capacity, 2.5, [8.; 4]), 0.);
        // Current illness closes capacity even if last month's craft pool was larger.
        let sick = service_capacity(200., 100., 0.5, 0., true, true);
        assert_eq!(remaining(sick, 2.5, [3., 4., 0., 0.]), 2.5);
    }
}

/// Last boundary's work ledger. Released work expires; it is not backdated into production.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct WorkReceipt {
    pub month: u32,
    pub requested: f64,
    pub granted: f64,
    pub used: f64,
    pub released: f64,
    pub settled: bool,
}
impl WorkReceipt {
    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            [self.requested, self.granted, self.used, self.released]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.)
                && self.granted <= self.requested + 1e-5
                && self.used <= self.granted + 1e-5
                && (!self.settled || (self.granted - self.used - self.released).abs() <= 1e-5),
            "invalid work reservation receipt: {self:?}"
        );
        Ok(())
    }
    pub(crate) fn settle(&mut self, used: f64) {
        self.used = used;
        self.released = (self.granted - used).max(0.);
        self.settled = true;
    }
}

impl crate::civilization::History {
    pub fn service_work_report(&self) -> serde_json::Value {
        serde_json::json!({
            "month": self.month,
            "participation": self.participation_report(),
            "learning_allocation": self.service_allocation,
            "office_service": self.offices.as_ref().and_then(|o| o.service.as_ref()),
            "culture": self.culture.as_ref().map(|c| (&c.work_receipt, &c.work_plans)),
            "research": self.expeditions.as_ref().and_then(|x| x.discoveries.as_ref()).map(|d| d.workshops.iter().map(|w| (w.site, &w.work_plan)).collect::<Vec<_>>()),
            "workshops": self.enterprises.as_ref().map(|e| e.firms.iter().map(|f| serde_json::json!({"firm":f.id,"site":f.site,"requested":f.last_requested_work,"funded":f.last_funded_work,"completed":f.last_completed_work,"idle_paid":(f.last_funded_work-f.last_completed_work).max(0.)})).collect::<Vec<_>>()),
            "crews": self.shipping.as_ref().map(|s| s.ports.iter().map(|p| (p.site, p.fleet.as_ref().map(|f| serde_json::json!({"requested":f.requested_work,"vessels":f.vessels,"funded":f.work(),"unfunded":(f.requested_work-f.work()).max(0.)})))).collect::<Vec<_>>()),
            "note": "Crew work is paid employment, including standby; unused late grants expire rather than rerunning production."
        })
    }
    pub(crate) fn validate_service_work(&self) -> anyhow::Result<()> {
        self.service_allocation.policy.allocate(0., [0.; 2])?;
        self.military.validate(self)?;
        if let Some(r) = &self.resolution {
            r.validate(self.month, self.sites.len())?;
        }
        if let Some(d) = &self.named_demography {
            d.validate(self)?;
        }
        if let Some(d) = &self.domestic {
            d.validate(self)?;
        }
        for (&person, duty) in &self.person_duties {
            anyhow::ensure!(
                self.people
                    .get(person as usize)
                    .is_some_and(|p| p.died.is_none())
                    && self.sites.get(duty.origin as usize).is_some()
                    && duty
                        .household
                        .is_none_or(|id| self.society.as_ref().is_some_and(|s| s
                            .households
                            .get(id as usize)
                            .is_some_and(|hh| hh.site == duty.origin)
                            && !s.relocation.away(id)))
                    && self.expeditions.as_ref().is_some_and(|x| x
                        .voyages
                        .get(duty.voyage as usize)
                        .is_some_and(|e| e.phase.active()
                            && e.origin == duty.origin
                            && e.crew.iter().any(|c| c.alive && c.person == Some(person)))),
                "orphaned or invalid personal travel duty"
            );
        }
        if let Some(p) = &self.participation {
            p.validate(self)?;
        }
        for cargo in &self.cargo {
            anyhow::ensure!(
                cargo
                    .voyage_clock
                    .as_ref()
                    .is_none_or(|c| c.month <= self.month
                        && c.remaining.is_finite()
                        && c.remaining >= 0.),
                "invalid cargo travel clock"
            );
        }
        if let Some(c) = &self.culture {
            c.work_receipt.validate()?;
            anyhow::ensure!(
                c.work_receipt.month <= self.month,
                "future cultural work receipt"
            );
            for (i, p) in c.work_plans.iter().enumerate() {
                if let Some(plans) = &p.upkeep {
                    anyhow::ensure!(
                        plans.len() <= c.institutions.len(),
                        "unbounded upkeep plans"
                    );
                    for (j, u) in plans.iter().enumerate() {
                        anyhow::ensure!(
                            (u.institution as usize) < c.institutions.len()
                                && c.institutions[u.institution as usize].site == p.site
                                && !plans[..j].iter().any(|v| v.institution == u.institution)
                                && [u.requested, u.granted, u.used]
                                    .iter()
                                    .all(|v| v.is_finite() && *v >= 0.)
                                && u.used <= u.granted + 1e-6
                                && u.granted <= u.requested + 1e-6
                                && u.members
                                    .iter()
                                    .all(|&id| (id as usize) < self.people.len())
                                && u.commitment.is_none_or(|id| self
                                    .participation
                                    .as_ref()
                                    .is_some_and(|state| state
                                        .commitments
                                        .get(id as usize)
                                        .is_some_and(|a| a.month == p.month
                                            && a.site == p.site
                                            && a.activity
                                                == crate::participation::Activity::Culture
                                            && a.people.len() == 1
                                            && u.members.contains(&a.people[0].0)
                                            && (a.granted - u.granted).abs() < 1e-6)))
                                && (u.commitment.is_some() || u.granted == 0.),
                            "invalid institutional work plan"
                        );
                    }
                    anyhow::ensure!(
                        plans.iter().map(|u| u.granted).sum::<f32>() <= p.granted + 1e-5
                            && plans.iter().map(|u| u.used).sum::<f32>() <= p.completed + 1e-5,
                        "upkeep exceeds cultural work"
                    );
                }
                anyhow::ensure!(
                    p.site as usize == i
                        && i < self.sites.len()
                        && p.month <= self.month
                        && p.commitment
                            .is_none_or(|id| self.participation.as_ref().is_some_and(|state| {
                                state.commitments.get(id as usize).is_some_and(|c| {
                                    c.month == p.month
                                        && c.site == p.site
                                        && c.activity == crate::participation::Activity::Culture
                                })
                            }))
                        && p.actor.is_none_or(|a| (a as usize) < self.people.len())
                        && p.participants.as_ref().is_none_or(|ids| ids
                            .iter()
                            .all(|&id| (id as usize) < self.people.len()))
                        && p.institution_lesson
                            .is_none_or(|(topic, teacher, institution)| topic < 12
                                && (teacher as usize) < self.people.len()
                                && (institution as usize) < c.institutions.len())
                        && [p.granted, p.cancelled_work, p.completed]
                            .iter()
                            .all(|v| v.is_finite() && *v >= 0.)
                        && (p.commitment.is_none() || p.completed <= p.granted + 1e-5)
                        && p.actions.iter().all(|(_, w)| w.is_finite() && *w >= 0.),
                    "invalid cultural work plan"
                );
            }
        }
        if let Some(d) = self
            .expeditions
            .as_ref()
            .and_then(|x| x.discoveries.as_ref())
        {
            for w in &d.workshops {
                if let Some(p) = &w.work_plan {
                    p.receipt.validate()?;
                    anyhow::ensure!(
                        p.receipt.month <= self.month
                            && p.commitment.is_none_or(|id| self
                                .participation
                                .as_ref()
                                .is_some_and(|state| state
                                    .commitments
                                    .get(id as usize)
                                    .is_some_and(|c| c.month == p.receipt.month
                                        && c.site == w.site
                                        && c.activity
                                            == crate::participation::Activity::Research)))
                            && p.teachers
                                .iter()
                                .all(|t| t.is_none_or(|id| (id as usize) < self.sites.len()))
                            && p.processing_kg.iter().all(|v| v.is_finite() && *v >= 0.),
                        "invalid research work plan"
                    );
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod receipt_tests {
    use super::*;
    #[test]
    fn unused_work_expires_and_overspending_is_rejected() {
        let mut r = WorkReceipt {
            month: 12,
            requested: 0.5,
            granted: 0.2,
            ..Default::default()
        };
        r.settle(0.05);
        r.validate().unwrap();
        assert!((r.released - 0.15).abs() < 1e-9);
        r.settle(0.3);
        assert!(r.validate().is_err());
    }
}
