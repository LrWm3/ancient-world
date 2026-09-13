//! Named monthly port-service assignments, backing the existing prepaid sea capacity.
use super::{funded_work, Fleet};
use crate::{
    civilization::History,
    household_economy::withdraw,
    labor::WorkReceipt,
    participation::{Activity, Presence},
    shipping::Shipping,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

const MAX_CREW_PRODUCTIVITY_BONUS: f32 = 0.50;
const CREW_PRACTICE_HALF_SATURATION_WORKER_MONTHS: f64 = 12.;

fn productivity_bonus(practice: f64) -> f32 {
    let practice = if practice.is_finite() {
        practice.max(0.)
    } else {
        0.
    };
    MAX_CREW_PRODUCTIVITY_BONUS
        * (practice / (CREW_PRACTICE_HALF_SATURATION_WORKER_MONTHS + practice)) as f32
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrewWork {
    pub person: u32,
    pub household: u32,
    pub commitment: Option<u32>,
    pub wages: f64,
    /// Frozen at reservation from previously completed port-service work. This is
    /// a service-rate bonus, never additional paid or personally committed time.
    #[serde(default)]
    pub productivity_bonus: f32,
    pub receipt: WorkReceipt,
}

impl CrewWork {
    pub(super) fn extra_service(&self) -> f32 {
        let time = if self.receipt.settled {
            self.receipt.used
        } else {
            self.receipt.granted
        };
        time as f32 * self.productivity_bonus
    }
}

impl History {
    pub(super) fn reserve_named_vessels(
        &mut self,
        site: u32,
        fleet: &mut Fleet,
        hulls: usize,
        target: f32,
    ) {
        let Some(society) = &self.society else { return };
        if society.household_economy.is_none() {
            return;
        }
        let pool = self.participation.as_ref().unwrap();
        let mut candidates: Vec<_> = pool
            .residents
            .values()
            .filter_map(|r| {
                let household = r.household?;
                let hh = society.households.get(household as usize)?;
                (r.presence == Presence::Resident(site)
                    && pool.available(r.person) > 1e-6
                    && hh.site == site
                    && !society.relocation.away(household)
                    && !society.relocation.lost_households.contains(&household))
                .then_some((r.person, household, r.merchant_completed))
            })
            .collect();
        // Experienced available residents are preferred. Stable identity resolves ties;
        // experience changes hiring priority and service rate, never paid time or cargo inventory.
        candidates.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
        let wage =
            18. * self.sites[site as usize].economy.prices[crate::economy::FOOD].max(0.01) as f64;
        let mut left =
            crate::labor::available(&self.sites[site as usize], true, self.living.is_some())
                .min((target - fleet.work()).max(0.));
        for vessel in fleet.vessels.iter_mut().take(hulls) {
            for &(person, household, practice) in &candidates {
                let wanted = (0.25 - vessel.funded_work)
                    .max(0.)
                    .min(left)
                    .min((self.sites[site as usize].economy.finance[0] as f64 / wage) as f32);
                if wanted <= 1e-6 {
                    break;
                }
                // f32 cash can leave an unpayable sub-cent remainder. Do not
                // reserve personal time when the actual debit would round to zero.
                let payable_work =
                    wanted.min(self.participation.as_ref().unwrap().available(person));
                let mut cash_probe = self.sites[site as usize].economy.finance[0];
                if withdraw(&mut cash_probe, payable_work as f64 * wage) <= 0. {
                    continue;
                }
                let Some(id) = self.participation.as_mut().unwrap().reserve(
                    self.month,
                    site,
                    Activity::MerchantCrew,
                    &[person],
                    wanted,
                ) else {
                    continue;
                };
                let grant = self.participation.as_ref().unwrap().commitments[id as usize].granted;
                let paid = withdraw(
                    &mut self.sites[site as usize].economy.finance[0],
                    grant as f64 * wage,
                );
                let actual = funded_work(paid, wage, grant);
                let society = self.society.as_mut().unwrap();
                let wallets = society.household_economy.as_mut().unwrap();
                wallets
                    .accounts
                    .resize(society.households.len(), Default::default());
                let account = &mut wallets.accounts[household as usize];
                account.cash += paid;
                account.wages += paid;
                account.employer_income += paid;
                vessel.household.get_or_insert(household); // legacy summary; crew is authoritative
                vessel.funded_work += actual;
                vessel.wages_paid += paid;
                vessel.crew.push(CrewWork {
                    person,
                    household,
                    commitment: Some(id),
                    wages: paid,
                    productivity_bonus: productivity_bonus(practice),
                    receipt: WorkReceipt {
                        month: self.month,
                        requested: wanted as f64,
                        granted: actual as f64,
                        ..Default::default()
                    },
                });
                self.sites[site as usize].economy.external[3] += actual;
                left = (left - actual).max(0.);
            }
        }
    }

    /// Settle before new market dispatches. The previous interval's cargo already
    /// advanced at Open; this work supports current dispatch and next month's travel.
    pub(crate) fn settle_vessel_crews(&mut self) -> Result<()> {
        let Some(mut shipping) = self.shipping.take() else {
            return Ok(());
        };
        let result = (|| {
            for port in &mut shipping.ports {
                let Some(fleet) = &mut port.fleet else {
                    continue;
                };
                for vessel in &mut fleet.vessels {
                    for work in &mut vessel.crew {
                        if work.receipt.settled {
                            continue;
                        }
                        ensure!(
                            work.receipt.month == self.month,
                            "stale merchant crew assignment"
                        );
                        let used = self
                            .personal_grant_live(work.commitment)
                            .min(work.receipt.granted as f32);
                        let pool = self.participation.as_mut().ok_or_else(|| {
                            anyhow::anyhow!("merchant crew lost participation ledger")
                        })?;
                        let commitment = work
                            .commitment
                            .ok_or_else(|| anyhow::anyhow!("missing merchant commitment"))?;
                        pool.settle(commitment, used)?;
                        if used + 1e-6 < work.receipt.granted as f32 {
                            pool.commitments[commitment as usize].cancellation = Some(
                                "assigned crew no longer present; prepaid wages retained".into(),
                            );
                        }
                        work.receipt.settle(used as f64);
                        let lost = work.receipt.released as f32;
                        vessel.funded_work = (vessel.funded_work - lost).max(0.);
                        let external = &mut self.sites[port.site as usize].economy.external[3];
                        *external = (*external - lost).max(0.);
                    }
                }
                let used = fleet.work() as f64;
                if let (Some(projection), Some(state)) =
                    (&mut fleet.projection, &mut self.resolution)
                {
                    if !projection.settled {
                        let receipt =
                            projection.outcome(self.month, port.site, used, state.compare)?;
                        let boundary = receipt.boundary.clone();
                        state.commit(receipt, &boundary)?;
                        projection.settled = true;
                    }
                }
            }
            Ok(())
        })();
        self.shipping = Some(shipping);
        result
    }
}

pub(crate) fn validate_crews(shipping: &Shipping, h: &History) -> Result<()> {
    let mut ids = std::collections::BTreeSet::new();
    for port in &shipping.ports {
        if let Some(fleet) = &port.fleet {
            if let Some(projection) = &fleet.projection {
                ensure!(projection.month <= h.month, "future crew projection");
                let mut checked = projection.clone();
                checked.settled = false;
                checked.outcome(projection.month, port.site, fleet.work() as f64, false)?;
            }
            for vessel in &fleet.vessels {
                for work in &vessel.crew {
                    work.receipt.validate()?;
                    ensure!(
                        work.receipt.month <= h.month
                            && (work.person as usize) < h.people.len()
                            && h.society
                                .as_ref()
                                .is_some_and(|s| (work.household as usize) < s.households.len())
                            && work.wages.is_finite()
                            && work.wages >= 0.
                            && work.productivity_bonus.is_finite()
                            && (0. ..=MAX_CREW_PRODUCTIVITY_BONUS)
                                .contains(&work.productivity_bonus)
                            && work
                                .commitment
                                .is_none_or(|id| ids.insert((work.receipt.month, id)))
                            && (work.commitment.is_some() || work.receipt.settled)
                            && (h.participation.is_some() || work.receipt.settled),
                        "invalid merchant crew receipt"
                    );
                    if let Some(pool) = h.participation.as_ref().filter(|p| {
                        p.month == Some(work.receipt.month) && work.commitment.is_some()
                    }) {
                        let c = pool
                            .commitments
                            .get(work.commitment.unwrap() as usize)
                            .ok_or_else(|| anyhow::anyhow!("missing merchant crew commitment"))?;
                        ensure!(
                            c.activity == Activity::MerchantCrew
                                && c.site == port.site
                                && c.people.len() == 1
                                && c.people[0].0 == work.person
                                && work.receipt.granted <= c.granted as f64 + 1e-6
                                && c.settled == work.receipt.settled
                                && (c.used as f64 - work.receipt.used).abs() < 1e-6,
                            "merchant crew commitment mismatch"
                        );
                    }
                }
                if !vessel.crew.is_empty() {
                    let work: f64 = vessel
                        .crew
                        .iter()
                        .map(|c| {
                            if c.receipt.settled {
                                c.receipt.used
                            } else {
                                c.receipt.granted
                            }
                        })
                        .sum();
                    ensure!(
                        (work - vessel.funded_work as f64).abs() < 1e-5,
                        "merchant crew capacity mismatch"
                    );
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::Catalog,
        config::Config,
        gpu::{ContextGpu, Generator},
    };

    #[test]
    fn productivity_is_bounded_frozen_and_only_backs_existing_hulls() {
        assert_eq!(productivity_bonus(0.), 0.);
        assert_eq!(productivity_bonus(12.), 0.25);
        assert!(productivity_bonus(1e12) <= 0.5);
        let crew = CrewWork {
            person: 0,
            household: 0,
            commitment: None,
            wages: 1.,
            productivity_bonus: 0.5,
            receipt: WorkReceipt {
                month: 0,
                requested: 0.25,
                granted: 0.25,
                ..Default::default()
            },
        };
        let mut fleet = Fleet {
            vessels: vec![super::super::Vessel {
                id: 0,
                name: "Test".into(),
                commissioned: 0,
                household: Some(0),
                funded_work: 0.25,
                wages_paid: 1.,
                crew: vec![crew.clone()],
            }],
            ..Default::default()
        };
        assert_eq!(fleet.capacity(), 250.); // Skilled full crew cannot enlarge a hull.
        fleet.vessels[0].crew[0].receipt.settle(0.);
        fleet.vessels[0].funded_work = 0.;
        assert_eq!(fleet.capacity(), 0.); // Absent crew retains pay, not service capacity.
        let mut legacy = serde_json::to_value(&crew).unwrap();
        legacy.as_object_mut().unwrap().remove("productivity_bonus");
        let legacy: CrewWork = serde_json::from_value(legacy).unwrap();
        assert_eq!(legacy.extra_service(), 0.);
    }

    fn cash(h: &History) -> f64 {
        h.sites
            .iter()
            .map(|s| s.economy.finance[0] as f64)
            .sum::<f64>()
            + h.society
                .as_ref()
                .unwrap()
                .household_economy
                .as_ref()
                .unwrap()
                .accounts
                .iter()
                .map(|a| a.cash)
                .sum::<f64>()
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn named_crews_compete_receive_pay_and_settle_once() {
        let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
        for seed in [17, 81, 256] {
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    resolution: 64,
                    ecology_resolution: 16,
                    seed,
                    ..Default::default()
                },
                Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.run_epochs(1).unwrap();
            g.found_civilizations(5).unwrap();
            g.enable_society().unwrap();
            g.enable_politics().unwrap();
            g.enable_shipping().unwrap();
            let h = g.civilizations.as_mut().unwrap();
            // Declared fixture assets/cash isolate matching from harbor construction.
            let site = h.shipping.as_ref().unwrap().ports[0].site as usize;
            let prior_assets = h.shipping.as_ref().unwrap().ports[0].assets;
            for (k, good) in crate::shipping::MATERIALS.into_iter().enumerate() {
                let imported = crate::shipping::TARGET[k] - prior_assets[k];
                h.sites[site].economy.initial[good] += imported;
                if k == 0 {
                    for (j, ratio) in [0.5, 0.002, 0.0002].into_iter().enumerate() {
                        h.sites[site].economy.external[j] += imported * ratio;
                    }
                }
            }
            h.shipping.as_mut().unwrap().ports[0].assets = crate::shipping::TARGET;
            h.shipping.as_mut().unwrap().ports[0].commissioned = Some(h.month);
            h.sites[site].economy.finance[1] += 10000. - h.sites[site].economy.finance[0];
            h.sites[site].economy.finance[0] = 10000.;
            h.set_demographic_resolution(crate::resolution::Mode::Aggregate, true)
                .unwrap();
            h.begin_service_reservations();
            let base = h.clone();
            let before = cash(h);
            h.prepare_vessels();
            assert!((cash(h) - before).abs() < 1e-8);
            let vessel = &h.shipping.as_ref().unwrap().ports[0]
                .fleet
                .as_ref()
                .unwrap()
                .vessels[0];
            assert!(vessel.funded_work > 0.);
            assert!(!vessel.crew.is_empty());
            let hire = vessel.crew[0].clone();
            let pool = h.participation.as_ref().unwrap();
            assert_eq!(
                pool.commitments[hire.commitment.unwrap() as usize].activity,
                Activity::MerchantCrew
            );
            assert!(pool.residents[&hire.person].committed >= hire.receipt.granted as f32);
            let wages_before = base
                .society
                .as_ref()
                .unwrap()
                .household_economy
                .as_ref()
                .unwrap()
                .accounts
                .get(hire.household as usize)
                .map_or(0., |a| a.wages);
            assert!(
                h.society
                    .as_ref()
                    .unwrap()
                    .household_economy
                    .as_ref()
                    .unwrap()
                    .accounts[hire.household as usize]
                    .wages
                    > wages_before
            );
            let paid = cash(h);
            let commitments = h.participation.as_ref().unwrap().commitments.len();
            h.prepare_vessels();
            assert_eq!(cash(h), paid);
            assert_eq!(
                h.participation.as_ref().unwrap().commitments.len(),
                commitments
            );
            h.settle_vessel_crews().unwrap();
            let resolution_before = serde_json::to_value(&h.resolution).unwrap();
            assert!(h
                .resolution
                .as_ref()
                .unwrap()
                .receipts
                .iter()
                .any(|r| r.boundary.system == crate::resolution::System::MerchantCrew));
            let experience =
                h.participation.as_ref().unwrap().residents[&hire.person].merchant_completed;
            assert!(experience > 0.);
            h.settle_vessel_crews().unwrap();
            assert_eq!(
                h.participation.as_ref().unwrap().residents[&hire.person].merchant_completed,
                experience
            );
            assert_eq!(
                serde_json::to_value(&h.resolution).unwrap(),
                resolution_before
            );
            validate_crews(h.shipping.as_ref().unwrap(), h).unwrap();
            h.participation.as_ref().unwrap().validate(h).unwrap();

            let mut switched = h.clone();
            let prepaid = switched.vessel_work(site as u32);
            switched.set_individual_participation(false).unwrap();
            assert_eq!(switched.vessel_work(site as u32), prepaid);
            validate_crews(switched.shipping.as_ref().unwrap(), &switched).unwrap();
            switched.set_individual_participation(true).unwrap();
            validate_crews(switched.shipping.as_ref().unwrap(), &switched).unwrap();

            let mut unfunded = base.clone();
            unfunded.sites[site].economy.finance[0] = 0.;
            unfunded.prepare_vessels();
            assert_eq!(unfunded.vessel_work(site as u32), 0.);

            // A more experienced eligible resident wins over the stable ID tie break.
            let mut skilled = base.clone();
            let veteran = skilled
                .participation
                .as_ref()
                .unwrap()
                .residents
                .values()
                .filter(|r| {
                    r.presence == Presence::Resident(site as u32)
                        && r.household.is_some()
                        && r.capacity > 0.1
                })
                .map(|r| r.person)
                .max()
                .unwrap();
            skilled
                .participation
                .as_mut()
                .unwrap()
                .residents
                .get_mut(&veteran)
                .unwrap()
                .merchant_completed = 12.;
            skilled.prepare_vessels();
            assert_eq!(
                skilled.shipping.as_ref().unwrap().ports[0]
                    .fleet
                    .as_ref()
                    .unwrap()
                    .vessels[0]
                    .crew[0]
                    .person,
                veteran
            );

            // Equal scarce time and money; only accumulated practice differs.
            let mut novice = base.clone();
            for r in novice
                .participation
                .as_mut()
                .unwrap()
                .residents
                .values_mut()
            {
                if r.presence == Presence::Resident(site as u32) {
                    r.capacity = if r.person == veteran { 0.05 } else { 0. };
                    r.merchant_completed = 0.;
                }
            }
            let mut expert = novice.clone();
            expert
                .participation
                .as_mut()
                .unwrap()
                .residents
                .get_mut(&veteran)
                .unwrap()
                .merchant_completed = 12.;
            for run in [&mut novice, &mut expert] {
                run.prepare_vessels();
                run.settle_vessel_crews().unwrap();
                validate_crews(run.shipping.as_ref().unwrap(), run).unwrap();
                assert!((cash(run) - before).abs() < 1e-8);
            }
            let nf = novice.shipping.as_ref().unwrap().ports[0]
                .fleet
                .as_ref()
                .unwrap();
            let ef = expert.shipping.as_ref().unwrap().ports[0]
                .fleet
                .as_ref()
                .unwrap();
            assert!(nf.capacity() > 0.);
            assert!((ef.capacity() / nf.capacity() - 1.25).abs() < 1e-5);
            assert_eq!(ef.work(), nf.work());
            assert_eq!(ef.vessels[0].wages_paid, nf.vessels[0].wages_paid);
            let capacity = ef.capacity();
            let mut restored: History =
                serde_json::from_value(serde_json::to_value(&expert).unwrap()).unwrap();
            restored.settle_vessel_crews().unwrap();
            assert_eq!(
                restored.shipping.as_ref().unwrap().ports[0]
                    .fleet
                    .as_ref()
                    .unwrap()
                    .capacity(),
                capacity
            );
            assert_eq!(
                serde_json::to_value(&restored).unwrap(),
                serde_json::to_value(&expert).unwrap()
            );

            // The same opening resources cannot buy crew time already committed elsewhere.
            let mut busy = base.clone();
            let pool = busy.participation.as_mut().unwrap();
            let people: Vec<_> = pool
                .residents
                .values()
                .filter(|r| r.presence == Presence::Resident(site as u32))
                .map(|r| r.person)
                .collect();
            for person in people {
                let available = pool.available(person);
                pool.reserve(
                    busy.month,
                    site as u32,
                    Activity::Research,
                    &[person],
                    available,
                );
            }
            let cash_before = cash(&busy);
            busy.prepare_vessels();
            assert_eq!(busy.vessel_work(site as u32), 0.);
            assert_eq!(cash(&busy), cash_before);
            let mut aggregate = busy.clone();
            // Explicit comparison control: no personal matching, same site cash and ceilings.
            aggregate.participation = None;
            for p in &mut aggregate.shipping.as_mut().unwrap().ports {
                if let Some(f) = &mut p.fleet {
                    f.projection = None;
                }
            }
            aggregate.prepare_vessels();
            assert!(aggregate.vessel_work(site as u32) > 0.);
            busy.settle_vessel_crews().unwrap();
            aggregate.settle_vessel_crews().unwrap();
            let crew_receipt = |h: &History| {
                h.resolution
                    .as_ref()
                    .unwrap()
                    .receipts
                    .iter()
                    .find(|r| {
                        r.boundary.system == crate::resolution::System::MerchantCrew
                            && r.boundary.site == site as u32
                    })
                    .unwrap()
                    .clone()
            };
            let unavailable = crew_receipt(&busy);
            assert!(unavailable.metrics[0].expected > 0.);
            assert_eq!(unavailable.metrics[0].actual, 0.);
            let pooled = crew_receipt(&aggregate);
            assert_eq!(pooled.mode, crate::resolution::Mode::Aggregate);
            assert!(pooled.metrics[0].actual > 0.);

            // Loss after reservation cannot fund new dispatch or earn completed experience.
            let mut absent = base.clone();
            absent.prepare_vessels();
            let person = absent.shipping.as_ref().unwrap().ports[0]
                .fleet
                .as_ref()
                .unwrap()
                .vessels[0]
                .crew[0]
                .person;
            absent.people[person as usize].died = Some(absent.month);
            absent.settle_vessel_crews().unwrap();
            let crew = &absent.shipping.as_ref().unwrap().ports[0]
                .fleet
                .as_ref()
                .unwrap()
                .vessels[0]
                .crew[0];
            assert_eq!(crew.receipt.used, 0.);
            assert!(crew.receipt.released > 0.);
            assert_eq!(
                absent.participation.as_ref().unwrap().residents[&person].merchant_completed,
                0.
            );
            let loss = crew_receipt(&absent);
            let completed = loss.metrics.last().unwrap();
            assert!(completed.actual < completed.expected);
            assert!(completed.unexplained().abs() < 1e-9);
            validate_crews(absent.shipping.as_ref().unwrap(), &absent).unwrap();

            // Settled prepaid capacity survives save/reload and the next monthly reset.
            h.release_vessel_work();
            let path = std::env::temp_dir().join(format!(
                "merchant-crews-{}-{seed}.world",
                std::process::id()
            ));
            g.save(&path).unwrap();
            let mut resumed = Generator::load(gpu.clone(), &path).unwrap();
            std::fs::remove_file(path).unwrap();
            g.advance_history(15).unwrap();
            for _ in 0..15 {
                resumed.advance_history(1).unwrap();
            }
            assert_eq!(
                serde_json::to_value(&g.civilizations).unwrap(),
                serde_json::to_value(&resumed.civilizations).unwrap()
            );
        }
    }
}
