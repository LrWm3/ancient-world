//! First production participation adapter. Existing recipes still own physical output.
use crate::{
    participation::{Activity, Participation, Presence},
    resolution::{Boundary, Metric, Mode, Receipt, System},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Staffing {
    pub boundary: Boundary,
    pub mode: Mode,
    pub expected: f64,
    pub granted: f64,
    /// Existing personal time commitments; one grant cannot pay several households twice.
    pub commitments: Vec<u32>,
    pub wages: Vec<(u32, f64)>,
    pub settled: bool,
}
#[derive(Clone, Copy)]
pub(crate) struct Offer {
    pub person: u32,
    pub household: u32,
    pub score: f32,
    pub fraction: f32,
}
/// An opening-state candidate; evaluated against the employer's actual wage.
#[derive(Clone, Copy)]
pub(crate) struct Candidate {
    pub person: u32,
    pub household: u32,
    ambition: f32,
    familiarity: f32,
    practice: [f32; 4],
    pressure: f32,
    reference_wage: f64,
}
impl Candidate {
    pub(crate) fn at_wage(self, wage: f64, family: u32) -> Offer {
        // Toy response in food-price-relative currency units, bounded independently
        // of employer funding and the person's remaining time.
        let relative = (wage / self.reference_wage).max(0.);
        let pay_response = (relative / (1. + relative)) as f32;
        Offer {
            person: self.person,
            household: self.household,
            score: self.familiarity + self.practice[family as usize] + 0.25 * self.ambition,
            fraction: if wage <= 0. {
                0.
            } else {
                ((0.25 + 0.25 * self.ambition + 0.5 * self.pressure) * (0.5 + pay_response))
                    .clamp(0., 1.)
            },
        }
    }
}
fn practice_scores(total: f64, by_family: [f64; 4]) -> [f32; 4] {
    std::array::from_fn(|family| {
        // Includes untyped old work; this is a hiring score, not extra experience.
        let relevant = by_family[family] + 0.2 * (total - by_family[family]).max(0.);
        (relevant / (12. + relevant)) as f32
    })
}
/// Learning by observation during real shared production, not extra teaching work.
/// Each mentor's completed work is shared across learners; no remote/idle mentors.
fn peer_learning(crew: &[(u32, f64, f32, f32)]) -> Vec<(u32, f32)> {
    // Entries: identity, completed work, opening family experience score, prior learning.
    let mut ordered = crew.to_vec();
    ordered.sort_by_key(|entry| entry.0);
    let crew = &ordered;
    let demand: Vec<f64> = crew
        .iter()
        .map(|mentor| {
            crew.iter()
                .filter(|learner| learner.0 != mentor.0 && learner.2 < mentor.2)
                .map(|learner| learner.1)
                .sum()
        })
        .collect();
    crew.iter()
        .map(|learner| {
            let exposure: f64 = crew
                .iter()
                .zip(&demand)
                .filter(|(mentor, demand)| {
                    mentor.0 != learner.0 && mentor.2 > learner.2 && **demand > 0.
                })
                .map(|(mentor, demand)| {
                    (mentor.1 * learner.1 / demand) * (mentor.2 - learner.2) as f64
                })
                .sum();
            let gain = (0.05 * exposure.min(learner.1)) as f32 * (1. - learner.3);
            (learner.0, gain)
        })
        .collect()
}
fn household_pressure(cash: f64, food_need: f64, hunger: f64, food_price: f64) -> f32 {
    // A two-month gross food bill is a buffer target, not a new inventory.
    let buffer = 2. * food_need.max(0.) * food_price;
    let cash_pressure = if buffer > 0. {
        (1. - cash.max(0.) / buffer).clamp(0., 1.)
    } else {
        0.
    };
    (0.7 * cash_pressure + 0.3 * hunger.clamp(0., 1.)) as f32
}
/// Explicit stable hiring priority; storage order never determines who gets the shift.
pub(crate) fn resolve(
    pool: &mut Participation,
    boundary: Boundary,
    expected: f32,
    offers: &[Offer],
) -> Staffing {
    let mut order = offers.to_vec();
    order.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.person.cmp(&b.person))
    });
    let mut result = Staffing {
        boundary,
        mode: Mode::Individual,
        expected: expected as f64,
        granted: 0.,
        commitments: vec![],
        wages: vec![],
        settled: false,
    };
    for offer in order {
        let available = pool.available(offer.person) * offer.fraction.clamp(0., 1.);
        let wanted = ((expected as f64 - result.granted).max(0.) as f32).min(available);
        if let Some(id) = pool.reserve(
            result.boundary.month,
            result.boundary.site,
            Activity::Workshop,
            &[offer.person],
            wanted,
        ) {
            let work = pool.commitments[id as usize].granted as f64;
            result.granted += work;
            result.commitments.push(id);
            result.wages.push((offer.household, work));
        }
    }
    result
}
/// An already affordable job quote. Market matching cannot expand its work grant.
pub(crate) struct Job {
    pub boundary: Boundary,
    pub expected: f32,
    pub wage: f64,
    pub family: u32,
}
/// Bounded proposal rounds. Each resident applies to at most one employer per round;
/// employers choose among applicants using family skill, then stable identity ties.
pub(crate) fn resolve_market(
    pool: &mut Participation,
    jobs: &[Job],
    candidates: &[Candidate],
) -> Vec<Staffing> {
    let mut jobs: Vec<_> = jobs.iter().collect();
    jobs.sort_by_key(|j| (j.boundary.site, j.boundary.subject));
    let mut candidates = candidates.to_vec();
    candidates.sort_by_key(|c| c.person);
    let mut tried = vec![std::collections::BTreeSet::new(); candidates.len()];
    let mut results: Vec<_> = jobs
        .iter()
        .map(|job| Staffing {
            boundary: job.boundary.clone(),
            mode: Mode::Individual,
            expected: job.expected as f64,
            granted: 0.,
            commitments: vec![],
            wages: vec![],
            settled: false,
        })
        .collect();
    // Each person can try every local job once, including after a partial acceptance.
    for _ in 0..jobs.len() {
        let mut proposals = vec![Vec::new(); jobs.len()];
        let mut any = false;
        for (person_index, candidate) in candidates.iter().enumerate() {
            if pool.available(candidate.person) <= 1e-6 {
                continue;
            }
            let mut best: Option<(usize, f64)> = None;
            for (index, job) in jobs.iter().enumerate() {
                if tried[person_index].contains(&index)
                    || job.wage <= 0.
                    || job.expected as f64 - results[index].granted <= 1e-6
                    || pool
                        .residents
                        .get(&candidate.person)
                        .is_none_or(|r| r.presence != Presence::Resident(job.boundary.site))
                {
                    continue;
                }
                let preference = job.wage / candidate.reference_wage
                    + 0.25 * candidate.practice[job.family as usize] as f64;
                // Jobs are in stable identity order, which resolves exact ties.
                if best.is_none_or(|(_, score)| preference > score) {
                    best = Some((index, preference));
                }
            }
            if let Some((index, _)) = best {
                tried[person_index].insert(index);
                proposals[index].push(candidate.at_wage(jobs[index].wage, jobs[index].family));
                any = true;
            }
        }
        if !any {
            break;
        }
        for (index, applicants) in proposals.iter().enumerate() {
            let result = &mut results[index];
            let remaining = (result.expected - result.granted).max(0.) as f32;
            let assigned = resolve(pool, result.boundary.clone(), remaining, applicants);
            result.granted += assigned.granted;
            result.commitments.extend(assigned.commitments);
            result.wages.extend(assigned.wages);
        }
    }
    results
}
/// Revision of the reserved execution inputs, recomputed before settlement.
pub(crate) fn staffing_revision(f: &crate::enterprises::Firm) -> u64 {
    crate::resolution::revision(
        [
            f.last_requested_work.to_bits(),
            f.last_funded_work.to_bits(),
            f.wage_rate.to_bits(),
            f.service_rate.unwrap_or(f.wage_rate * 1.25).to_bits(),
            u64::from(f.family),
        ]
        .into_iter()
        .chain(
            f.staffing
                .iter()
                .flat_map(|s| [s.expected.to_bits(), s.granted.to_bits()]),
        )
        .chain(
            f.staffing
                .iter()
                .flat_map(|s| s.commitments.iter().map(|id| u64::from(*id))),
        ),
    )
}
impl Staffing {
    pub(crate) fn receipt(&self, used: f64, compare: bool) -> Receipt {
        Receipt {
            demographic_snapshot: None,
            boundary: self.boundary.clone(),
            mode: self.mode,
            metrics: if compare {
                vec![
                    Metric {
                        name: "paid_time".into(),
                        unit: "worker-months".into(),
                        expected: self.expected,
                        actual: self.granted,
                        explained: vec![(
                            "unfilled individual offers".into(),
                            self.granted - self.expected,
                        )],
                    },
                    Metric {
                        name: "completed_work".into(),
                        unit: "worker-months".into(),
                        expected: self.granted,
                        actual: used,
                        explained: vec![("unused paid capacity".into(), used - self.granted)],
                    },
                ]
            } else {
                vec![]
            },
        }
    }
}
impl crate::civilization::History {
    pub(crate) fn check_workshop_reservation_boundary(&self) -> anyhow::Result<()> {
        if self
            .resolution
            .as_ref()
            .is_some_and(|r| r.workshop_individual)
        {
            anyhow::ensure!(
                self.individual_demography_enabled() && self.participation.is_some(),
                "workshop authority lost its resident provider"
            );
            anyhow::ensure!(
                self.enterprises
                    .as_ref()
                    .is_none_or(|e| e.firms.iter().all(|f| f
                        .staffing
                        .as_ref()
                        .is_none_or(|s| s.settled && s.boundary.month < self.month))),
                "duplicate or unfinished workshop reservation"
            );
        }
        Ok(())
    }
    pub fn set_workshop_refinement(&mut self, enabled: bool) -> anyhow::Result<()> {
        anyhow::ensure!(
            enabled || !self.agriculture_refinement_enabled(),
            "disable agriculture refinement before workshops"
        );
        if !enabled && self.resolution.is_none() {
            return Ok(());
        }
        anyhow::ensure!(
            !enabled || (self.individual_demography_enabled() && self.participation.is_some()),
            "workshop refinement requires individual residents and participation"
        );
        anyhow::ensure!(
            self.enterprises.as_ref().is_none_or(|e| e
                .firms
                .iter()
                .all(|f| f.staffing.as_ref().is_none_or(|s| s.settled))),
            "workshop assignments remain unsettled"
        );
        self.resolution
            .get_or_insert_with(Default::default)
            .workshop_individual = enabled;
        Ok(())
    }
    pub(crate) fn workshop_offers(&self) -> Vec<Vec<Candidate>> {
        let mut offers = vec![vec![]; self.sites.len()];
        if !self
            .resolution
            .as_ref()
            .is_some_and(|r| r.workshop_individual)
        {
            return offers;
        }
        let Some(pool) = &self.participation else {
            return offers;
        };
        for p in pool.residents.values() {
            let (Some(household), Presence::Resident(site)) = (p.household, p.presence) else {
                continue;
            };
            if pool.available(p.person) <= 0. {
                continue;
            }
            let agent = self
                .culture
                .as_ref()
                .and_then(|c| c.agents.iter().find(|a| a.person == p.person));
            let interest = agent.map_or(0.5, |a| a.traits[0]);
            let familiar = agent.map_or(0., |a| f32::from(a.occupation.contains("craft")));
            let food_price =
                self.sites[site as usize].economy.prices[crate::economy::FOOD].max(0.01) as f64;
            let account = self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .and_then(|e| e.accounts.get(household as usize));
            // Do not carry old settlement food exposure across a relocation.
            let pressure = account
                .filter(|a| a.food_site == Some(site))
                .map_or(0., |a| {
                    household_pressure(a.cash, a.need, a.hunger, food_price)
                });
            // Completed work, not merely time paid for, develops hiring familiarity.
            let base = practice_scores(p.workshop_completed, p.workshop_practice);
            let practice = std::array::from_fn(|family| {
                base[family] + 0.25 * p.workshop_learning[family] * (1. - base[family])
            });
            offers[site as usize].push(Candidate {
                person: p.person,
                household,
                ambition: interest.clamp(0., 1.),
                familiarity: familiar,
                practice,
                pressure,
                reference_wage: 18. * food_price,
            });
        }
        offers
    }
    pub(crate) fn settle_workshop_resolutions(&mut self) -> anyhow::Result<()> {
        let Some(state) = &mut self.resolution else {
            return Ok(());
        };
        let Some(firms) = self.enterprises.as_mut().map(|e| &mut e.firms) else {
            return Ok(());
        };
        // Capture before any firm settles: new work cannot become same-month expertise.
        let opening: std::collections::BTreeMap<_, _> = self
            .participation
            .iter()
            .flat_map(|p| p.residents.iter())
            .map(|(&id, r)| (id, (r.workshop_practice, r.workshop_learning)))
            .collect();
        for f in firms {
            let revision = staffing_revision(f);
            let Some(s) = f.staffing.as_mut().filter(|s| !s.settled) else {
                continue;
            };
            let current = Boundary {
                month: self.month,
                system: System::Workshop,
                site: f.site,
                subject: f.id,
                revision,
            };
            state.check(&s.boundary, &current)?;
            anyhow::ensure!(
                f.last_completed_work.is_finite()
                    && f.last_completed_work >= 0.
                    && f.last_completed_work <= s.granted + 1e-5,
                "workshop {} site {} month {} invalid execution {} for committed {} (funded {}, closed {:?})", f.id, f.site, self.month, f.last_completed_work, s.granted, f.last_funded_work, f.closed
            );
            let used = f.last_completed_work.min(s.granted).max(0.);
            if s.mode == Mode::Individual {
                let pool = self
                    .participation
                    .as_mut()
                    .ok_or_else(|| anyhow::anyhow!("missing workshop participants"))?;
                let mut assignments = Vec::new();
                let mut crew = Vec::new();
                for &id in &s.commitments {
                    let c = &pool.commitments[id as usize];
                    let contribution = if s.granted > 0. {
                        (c.granted as f64 * used / s.granted) as f32
                    } else {
                        0.
                    };
                    for &(person, share) in &c.people {
                        let (practice, learning) = &opening[&person];
                        let experience = practice[f.family as usize];
                        let completed = (share * contribution.min(c.granted) / c.granted) as f64;
                        crew.push((
                            person,
                            completed,
                            (experience / (12. + experience)) as f32,
                            learning[f.family as usize],
                        ));
                    }
                    assignments.push((id, contribution));
                }
                let gains = peer_learning(&crew);
                for (id, contribution) in assignments {
                    pool.settle_workshop(id, contribution, f.family)?;
                }
                for (person, gain) in gains {
                    let learning = &mut pool.residents.get_mut(&person).unwrap().workshop_learning
                        [f.family as usize];
                    *learning = (*learning + gain).min(1.);
                }
            }
            state.commit(s.receipt(used, state.compare), &current)?;
            s.settled = true;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn market_fixture() -> (Participation, Candidate, Vec<Job>) {
        let mut pool = Participation {
            month: Some(1),
            ..Default::default()
        };
        pool.residents.insert(
            0,
            crate::participation::Resident {
                person: 0,
                household: Some(0),
                presence: Presence::Resident(0),
                care: 0.,
                capacity: 0.1,
                committed: 0.,
                completed: [0.; 2],
                workshop_completed: 0.,
                merchant_completed: 0.,
                workshop_practice: [0.; 4],
                workshop_learning: [0.; 4],
            },
        );
        let worker = Candidate {
            person: 0,
            household: 0,
            ambition: 1.,
            familiarity: 0.,
            practice: [0.; 4],
            pressure: 1.,
            reference_wage: 36.,
        };
        let jobs = (0..2)
            .map(|id| Job {
                boundary: Boundary {
                    month: 1,
                    site: 0,
                    subject: id,
                    system: System::Workshop,
                    revision: 0,
                },
                expected: 0.1,
                wage: if id == 0 { 36. } else { 72. },
                family: id,
            })
            .collect();
        (pool, worker, jobs)
    }
    #[test]
    fn local_job_choice_responds_to_pay_skill_access_and_funding() {
        let (pool, worker, mut jobs) = market_fixture();
        let mut branch = pool.clone();
        let result = resolve_market(&mut branch, &jobs, &[worker]);
        assert_eq!(result[0].granted, 0.);
        assert!((result[1].granted - 0.1).abs() < 1e-6);
        jobs[1].expected = 0.;
        let result = resolve_market(&mut pool.clone(), &jobs, &[worker]);
        assert!((result[0].granted - 0.1).abs() < 1e-6);
        assert_eq!(result[1].granted, 0.);
        jobs[1].expected = 0.1;
        jobs[1].wage = 39.6;
        let specialist = Candidate {
            practice: [1., 0., 0., 0.],
            ..worker
        };
        let result = resolve_market(&mut pool.clone(), &jobs, &[specialist]);
        assert!((result[0].granted - 0.1).abs() < 1e-6);
        assert_eq!(result[1].granted, 0.);
        let mut absent = pool;
        absent.residents.get_mut(&0).unwrap().presence = Presence::Resident(1);
        assert!(resolve_market(&mut absent, &jobs, &[worker])
            .iter()
            .all(|s| s.granted == 0.));
    }
    #[test]
    fn rejected_applicants_try_alternatives_without_order_or_time_leaks() {
        let (mut pool, worker, mut jobs) = market_fixture();
        pool.residents.get_mut(&0).unwrap().capacity = 0.2;
        let mut second = pool.residents[&0].clone();
        second.person = 1;
        second.household = Some(1);
        pool.residents.insert(1, second);
        let other = Candidate {
            person: 1,
            household: 1,
            ..worker
        };
        jobs[0].expected = 0.4;
        let mut reordered = pool.clone();
        let result = resolve_market(&mut pool, &jobs, &[worker, other]);
        assert!(result[0]
            .wages
            .iter()
            .any(|(household, work)| *household == 1 && *work > 0.));
        assert!(result.iter().all(|s| s.granted <= s.expected + 1e-6));
        assert!(pool
            .residents
            .values()
            .all(|r| r.committed <= r.capacity + 1e-6));
        jobs.reverse();
        let again = resolve_market(&mut reordered, &jobs, &[other, worker]);
        assert_eq!(
            serde_json::to_value(&result).unwrap(),
            serde_json::to_value(again).unwrap()
        );
        assert_eq!(
            serde_json::to_value(pool).unwrap(),
            serde_json::to_value(reordered).unwrap()
        );
    }
    #[test]
    fn peer_learning_requires_shared_completed_work_and_bounds_mentor_capacity() {
        let mentor = (0, 0.2, 0.8, 0.);
        let novice = (1, 0.4, 0., 0.);
        let gains = peer_learning(&[mentor, novice]);
        assert_eq!(gains[0].1, 0.);
        assert!(gains[1].1 > 0.);
        assert_eq!(peer_learning(&[novice])[0].1, 0.);
        assert_eq!(peer_learning(&[(0, 0., 0.8, 0.), novice])[1].1, 0.);
        assert_eq!(peer_learning(&[mentor, (1, 0., 0., 0.)])[1].1, 0.);
        assert_eq!(peer_learning(&[mentor, (1, 0.4, 0.8, 0.)])[1].1, 0.);
        let crowded = [mentor, novice, (2, 0.4, 0., 0.)];
        let gains = peer_learning(&crowded);
        assert!(gains[1].1 < peer_learning(&[mentor, novice])[1].1);
        assert!(gains.iter().map(|(_, g)| *g).sum::<f32>() <= 0.05 * 0.2);
        let mut reversed = peer_learning(&[crowded[2], crowded[1], crowded[0]]);
        reversed.sort_by_key(|(id, _)| *id);
        assert_eq!(gains, reversed);
        assert!(
            peer_learning(&[mentor, (1, 0.4, 0., 0.9)])[1].1
                < peer_learning(&[mentor, novice])[1].1
        );
    }
    #[test]
    fn specialization_changes_hiring_without_changing_time_or_inventing_old_trades() {
        let base = Candidate {
            person: 0,
            household: 0,
            ambition: 0.5,
            familiarity: 0.,
            practice: practice_scores(12., [12., 0., 0., 0.]),
            pressure: 0.,
            reference_wage: 36.,
        };
        let other = Candidate {
            person: 1,
            household: 1,
            practice: practice_scores(12., [0., 12., 0., 0.]),
            ..base
        };
        let mut pool = Participation {
            month: Some(1),
            ..Default::default()
        };
        for person in 0..2 {
            pool.residents.insert(
                person,
                crate::participation::Resident {
                    person,
                    household: Some(person),
                    presence: Presence::Resident(0),
                    care: 0.,
                    capacity: 0.8,
                    committed: 0.,
                    completed: [0.; 2],
                    workshop_completed: 0.,
                    merchant_completed: 0.,
                    workshop_practice: [0.; 4],
                    workshop_learning: [0.; 4],
                },
            );
        }
        for family in 0..2 {
            let mut branch = pool.clone();
            let result = resolve(
                &mut branch,
                Boundary {
                    month: 1,
                    site: 0,
                    subject: family,
                    system: System::Workshop,
                    revision: 0,
                },
                0.1,
                &[other.at_wage(36., family), base.at_wage(36., family)],
            );
            assert_eq!(
                branch.commitments[result.commitments[0] as usize].people[0].0,
                family
            );
            assert!((result.granted - 0.1).abs() < 1e-6);
        }
        assert_eq!(base.at_wage(36., 0).fraction, base.at_wage(36., 1).fraction);
        let legacy = practice_scores(12., [0.; 4]);
        assert!(legacy.iter().all(|v| *v == legacy[0]));
        assert!(legacy[0] > 0. && legacy[0] < base.practice[0]);
    }
    #[test]
    fn learning_uses_completed_family_work_once_and_old_archives_stay_untyped() {
        let mut pool = Participation {
            month: Some(1),
            ..Default::default()
        };
        let old = serde_json::json!({
            "person":0, "household":0, "presence":{"Resident":0},
            "care":0., "capacity":0.8, "committed":0., "completed":[0.,0.],
            "workshop_completed":2.
        });
        let resident: crate::participation::Resident = serde_json::from_value(old).unwrap();
        assert_eq!(resident.workshop_practice, [0.; 4]);
        pool.residents.insert(0, resident);
        let id = pool.reserve(1, 0, Activity::Workshop, &[0], 0.4).unwrap();
        let before = serde_json::to_value(&pool).unwrap();
        assert!(pool.settle_workshop(id, 0.2, 4).is_err());
        assert_eq!(before, serde_json::to_value(&pool).unwrap());
        pool.settle_workshop(id, 0.2, 2).unwrap();
        let learned = pool.residents[&0].workshop_practice;
        assert!((learned[2] - 0.2).abs() < 1e-6);
        assert_eq!([learned[0], learned[1], learned[3]], [0.; 3]);
        assert!((pool.residents[&0].workshop_completed - 2.2).abs() < 1e-6);
        let before = serde_json::to_value(&pool).unwrap();
        assert!(pool.settle_workshop(id, 0.2, 2).is_err());
        assert_eq!(before, serde_json::to_value(&pool).unwrap());
        let id = pool.reserve(1, 0, Activity::Workshop, &[0], 0.1).unwrap();
        pool.settle_workshop(id, 0., 1).unwrap();
        assert_eq!(learned, pool.residents[&0].workshop_practice);
        let resumed: Participation =
            serde_json::from_value(serde_json::to_value(&pool).unwrap()).unwrap();
        assert_eq!(resumed.residents[&0].workshop_practice, learned);
    }
    #[test]
    fn offers_respond_to_pay_need_and_completed_practice() {
        let secure = Candidate {
            person: 0,
            household: 0,
            ambition: 0.5,
            familiarity: 0.,
            practice: [0.; 4],
            pressure: household_pressure(1000., 18., 0., 2.),
            reference_wage: 36.,
        };
        let strained = Candidate {
            pressure: household_pressure(0., 18., 1., 2.),
            ..secure
        };
        assert!(strained.at_wage(36., 0).fraction > secure.at_wage(36., 0).fraction);
        assert!(secure.at_wage(72., 0).fraction > secure.at_wage(18., 0).fraction);
        assert_eq!(strained.at_wage(0., 0).fraction, 0.);
        let skilled = Candidate {
            practice: [0.8; 4],
            ..secure
        };
        assert!(skilled.at_wage(36., 0).score > secure.at_wage(36., 0).score);
        assert_eq!(
            skilled.at_wage(36., 0).fraction,
            secure.at_wage(36., 0).fraction
        );
        for wage in [0., 0.01, 36., 1e9] {
            for candidate in [secure, strained, skilled] {
                assert!((0. ..=1.).contains(&candidate.at_wage(wage, 0).fraction));
            }
        }
    }
    #[test]
    fn staffing_uses_remaining_time_and_stable_priority() {
        let mut pool = Participation {
            month: Some(1),
            ..Default::default()
        };
        for id in 0..2 {
            pool.residents.insert(
                id,
                crate::participation::Resident {
                    person: id,
                    household: Some(id),
                    presence: Presence::Resident(0),
                    care: 0.,
                    capacity: 0.8,
                    committed: 0.,
                    completed: [0.; 2],
                    workshop_completed: 0.,
                    merchant_completed: 0.,
                    workshop_practice: [0.; 4],
                    workshop_learning: [0.; 4],
                },
            );
        }
        pool.reserve(1, 0, Activity::Research, &[0], 0.6).unwrap();
        let b = Boundary {
            month: 1,
            system: System::Workshop,
            site: 0,
            subject: 0,
            revision: 1,
        };
        let offers = [
            Offer {
                person: 0,
                household: 0,
                score: 1.,
                fraction: 1.,
            },
            Offer {
                person: 1,
                household: 1,
                score: 0.,
                fraction: 1.,
            },
        ];
        let mut other = pool.clone();
        let s = resolve(&mut pool, b.clone(), 1.2, &offers);
        let t = resolve(&mut other, b, 1.2, &[offers[1], offers[0]]);
        assert!((s.granted - 1.).abs() < 1e-6);
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(t).unwrap()
        );
        assert!(pool.residents.values().all(|r| r.committed <= r.capacity));
    }
}
