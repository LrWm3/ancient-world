//! Opt-in named office attendance; funding remains owned by governance.
use super::*;
use crate::{labor::WorkReceipt, participation::Activity};

const OFFICE_WORK_MONTHS: f32 = 0.1;
const MAX_REQUESTED_WORK_MONTHS: f64 = 0.10001;
const SERVICE_MEMORY_MONTHS: f32 = 60.0;
const SERVICE_CREDIT_PER_WORK: f32 = 0.25;
const MAX_SERVICE_CREDIT: f32 = 0.30;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub person: u32,
    pub site: u32,
    pub month: u32,
    pub credit: f32,
}
impl Record {
    pub(crate) fn score(&self, month: u32) -> f32 {
        self.credit * (-(month.saturating_sub(self.month) as f32) / SERVICE_MEMORY_MONTHS).exp()
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Service {
    pub plans: Vec<Plan>,
    #[serde(default)]
    pub records: Vec<Record>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub site: u32,
    pub controller: u32,
    pub holder: u32,
    pub commitment: Option<u32>,
    pub work: WorkReceipt,
    /// Opening town allowance, before matching the holder. Absent in older plans.
    #[serde(default)]
    pub allowance: Option<f64>,
}
impl History {
    pub fn set_office_service(&mut self, enabled: bool) -> Result<()> {
        ensure!(
            self.participation
                .as_ref()
                .is_none_or(|p| p.commitments.iter().all(|c| c.settled)),
            "finish personal work before changing office service"
        );
        ensure!(
            !enabled || self.participation.is_some(),
            "office service requires personal participation"
        );
        let offices = self
            .offices
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable offices first"))?;
        ensure!(
            offices
                .service
                .as_ref()
                .is_none_or(|s| s.plans.iter().all(|p| p.work.settled)),
            "finish office work before changing service"
        );
        if enabled {
            offices.service.get_or_insert_with(Default::default);
        } else {
            offices.service = None;
        }
        Ok(())
    }
    pub(crate) fn reserve_office_service(&mut self) -> Result<()> {
        let Some(service) = self.offices.as_ref().and_then(|o| o.service.as_ref()) else {
            return Ok(());
        };
        ensure!(
            self.participation.is_some(),
            "office service lost personal participation"
        );
        ensure!(
            service
                .plans
                .iter()
                .all(|p| p.work.settled && p.work.month < self.month),
            "stale office reservation"
        );
        let holders: Vec<_> = self
            .offices
            .as_ref()
            .unwrap()
            .seats
            .iter()
            .filter_map(|o| {
                let holder = o.holder()?;
                (o.controller == self.controller(o.site)
                    && self
                        .office_candidates(o.site, o.controller, o.selection)
                        .contains(&holder))
                .then_some((o.site, o.controller, holder))
            })
            .collect();
        let mut plans = vec![];
        for (site, controller, holder) in holders {
            let wanted = OFFICE_WORK_MONTHS;
            let allowance = crate::labor::available(
                &self.sites[site as usize],
                self.society.is_some(),
                self.living.is_some(),
            )
            .min(wanted);
            let state = self.participation.as_mut().unwrap();
            let commitment =
                state.reserve(self.month, site, Activity::Governance, &[holder], allowance);
            let grant = commitment.map_or(0., |id| state.commitments[id as usize].granted);
            self.sites[site as usize].economy.external[3] += grant;
            plans.push(Plan {
                site,
                controller,
                holder,
                commitment,
                allowance: Some(allowance as f64),
                work: WorkReceipt {
                    month: self.month,
                    requested: wanted as f64,
                    granted: grant as f64,
                    ..Default::default()
                },
            });
        }
        self.offices
            .as_mut()
            .unwrap()
            .service
            .as_mut()
            .unwrap()
            .plans = plans;
        Ok(())
    }
    pub(crate) fn settle_office_resolutions(&mut self) -> Result<()> {
        let Some(mut resolution) = self.resolution.clone() else {
            return Ok(());
        };
        let Some(service) = self.offices.as_ref().and_then(|o| o.service.as_ref()) else {
            return Ok(());
        };
        // Observe completed work only. Stage the entire batch before publishing it.
        for p in &service.plans {
            let Some(allowance) = p.allowance else {
                continue;
            };
            ensure!(
                p.work.month == self.month && p.work.settled,
                "office comparison requires current completed work"
            );
            let mut receipt = crate::learning_resolution::outcome(
                p.work.month,
                p.site,
                crate::resolution::System::OfficeService,
                crate::resolution::Mode::Individual,
                [p.work.requested, allowance, p.work.granted, p.work.used],
                resolution.compare,
            )?;
            receipt.boundary.subject = p.holder;
            receipt.boundary.revision = crate::resolution::revision([
                receipt.boundary.revision,
                p.holder as u64,
                p.controller as u64,
            ]);
            let boundary = receipt.boundary.clone();
            resolution.commit(receipt, &boundary)?;
        }
        resolution.validate(self.month, self.sites.len())?;
        self.resolution = Some(resolution);
        Ok(())
    }
    pub(crate) fn settle_office_service(&mut self) -> Result<()> {
        let Some(service) = self.offices.as_ref().and_then(|o| o.service.as_ref()) else {
            return Ok(());
        };
        let plans = service.plans.clone();
        // Validate the entire batch before consuming any commitments.
        service.validate(self)?;
        ensure!(
            plans
                .iter()
                .all(|p| p.work.month == self.month && !p.work.settled),
            "stale office settlement"
        );
        for (index, p) in plans.iter().enumerate() {
            ensure!(
                p.work.month == self.month && !p.work.settled,
                "stale office settlement"
            );
            let office = &self.offices.as_ref().unwrap().seats[p.site as usize];
            let live = office.holder() == Some(p.holder)
                && office.controller == p.controller
                && self.controller(p.site) == p.controller;
            let used = if live {
                self.personal_grant_live(p.commitment)
                    .min(p.work.granted as f32)
            } else {
                0.
            };
            if let Some(id) = p.commitment {
                self.participation
                    .as_mut()
                    .ok_or_else(|| anyhow::anyhow!("missing office participants"))?
                    .settle(id, used)?;
            }
            // GPU production has already honored the reserved allowance. Expired
            // time cannot be recycled into this month's production.
            self.sites[p.site as usize].economy.external[3] =
                (self.sites[p.site as usize].economy.external[3] - p.work.granted as f32).max(0.);
            self.offices
                .as_mut()
                .unwrap()
                .service
                .as_mut()
                .unwrap()
                .plans[index]
                .work
                .settle(used as f64);
            if used > 0. {
                let records = &mut self
                    .offices
                    .as_mut()
                    .unwrap()
                    .service
                    .as_mut()
                    .unwrap()
                    .records;
                if let Some(r) = records
                    .iter_mut()
                    .find(|r| r.person == p.holder && r.site == p.site)
                {
                    r.credit = (r.score(self.month) + used * SERVICE_CREDIT_PER_WORK)
                        .min(MAX_SERVICE_CREDIT);
                    r.month = self.month;
                } else {
                    records.push(Record {
                        person: p.holder,
                        site: p.site,
                        month: self.month,
                        credit: (used * SERVICE_CREDIT_PER_WORK).min(MAX_SERVICE_CREDIT),
                    });
                }
            }
        }
        Ok(())
    }
}
impl Service {
    pub(super) fn validate(&self, h: &History) -> Result<()> {
        let mut keys = std::collections::BTreeSet::new();
        for r in &self.records {
            ensure!(
                (r.person as usize) < h.people.len()
                    && (r.site as usize) < h.sites.len()
                    && r.month <= h.month
                    && r.credit.is_finite()
                    && (0. ..=MAX_SERVICE_CREDIT).contains(&r.credit)
                    && keys.insert((r.person, r.site)),
                "invalid personal office service memory"
            );
        }
        ensure!(
            h.participation.is_some() && self.plans.len() <= h.sites.len(),
            "invalid office service state"
        );
        for (i, p) in self.plans.iter().enumerate() {
            p.work.validate()?;
            if let Some(allowance) = p.allowance {
                ensure!(
                    allowance.is_finite()
                        && allowance >= 0.
                        && allowance <= p.work.requested + 1e-6
                        && p.work.granted <= allowance + 1e-6,
                    "invalid opening office allowance"
                );
            }
            if p.work.month == h.month {
                if let Some(id) = p.commitment {
                    let c = h
                        .participation
                        .as_ref()
                        .and_then(|s| s.commitments.get(id as usize))
                        .ok_or_else(|| anyhow::anyhow!("missing office commitment"))?;
                    ensure!(
                        c.month == p.work.month
                            && c.site == p.site
                            && c.activity == Activity::Governance
                            && c.people.len() == 1
                            && c.people[0].0 == p.holder
                            && (c.granted as f64 - p.work.granted).abs() < 1e-6
                            && c.settled == p.work.settled,
                        "office commitment mismatch"
                    );
                } else {
                    ensure!(p.work.granted == 0., "unassigned office grant");
                }
            }
            ensure!(
                (p.site as usize) < h.sites.len()
                    && (p.holder as usize) < h.people.len()
                    && p.work.month <= h.month
                    && p.work.requested <= MAX_REQUESTED_WORK_MONTHS
                    && !self.plans[..i].iter().any(|other| other.site == p.site),
                "invalid office service plan"
            );
        }
        Ok(())
    }
    pub(super) fn delivered(&self, month: u32, site: u32, holder: u32, controller: u32) -> f32 {
        self.plans
            .iter()
            .find(|p| {
                p.site == site
                    && p.holder == holder
                    && p.controller == controller
                    && p.work.month == month
                    && p.work.settled
            })
            .map_or(0., |p| {
                (p.work.used / p.work.requested.max(1e-12)).clamp(0., 1.) as f32
            })
    }
}
