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
    practice: f32,
    pressure: f32,
    reference_wage: f64,
}
impl Candidate {
    pub(crate) fn at_wage(self, wage: f64) -> Offer {
        // Toy response in food-price-relative currency units, bounded independently
        // of employer funding and the person's remaining time.
        let relative = (wage / self.reference_wage).max(0.);
        let pay_response = (relative / (1. + relative)) as f32;
        Offer {
            person: self.person,
            household: self.household,
            score: self.familiarity + self.practice + 0.25 * self.ambition,
            fraction: if wage <= 0. {
                0.
            } else {
                ((0.25 + 0.25 * self.ambition + 0.5 * self.pressure) * (0.5 + pay_response))
                    .clamp(0., 1.)
            },
        }
    }
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
/// Revision of the reserved execution inputs, recomputed before settlement.
pub(crate) fn staffing_revision(f: &crate::enterprises::Firm) -> u64 {
    crate::resolution::revision(
        [
            f.last_requested_work.to_bits(),
            f.last_funded_work.to_bits(),
            f.wage_rate.to_bits(),
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
            let practice = (p.workshop_completed / (12. + p.workshop_completed)) as f32;
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
                "workshop execution exceeded its committed time"
            );
            let used = f.last_completed_work.min(s.granted).max(0.);
            if s.mode == Mode::Individual {
                let pool = self
                    .participation
                    .as_mut()
                    .ok_or_else(|| anyhow::anyhow!("missing workshop participants"))?;
                for &id in &s.commitments {
                    let c = &pool.commitments[id as usize];
                    let contribution = if s.granted > 0. {
                        c.granted as f64 * used / s.granted
                    } else {
                        0.
                    };
                    pool.settle(id, contribution as f32)?;
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
    #[test]
    fn offers_respond_to_pay_need_and_completed_practice() {
        let secure = Candidate {
            person: 0,
            household: 0,
            ambition: 0.5,
            familiarity: 0.,
            practice: 0.,
            pressure: household_pressure(1000., 18., 0., 2.),
            reference_wage: 36.,
        };
        let strained = Candidate {
            pressure: household_pressure(0., 18., 1., 2.),
            ..secure
        };
        assert!(strained.at_wage(36.).fraction > secure.at_wage(36.).fraction);
        assert!(secure.at_wage(72.).fraction > secure.at_wage(18.).fraction);
        assert_eq!(strained.at_wage(0.).fraction, 0.);
        let skilled = Candidate {
            practice: 0.8,
            ..secure
        };
        assert!(skilled.at_wage(36.).score > secure.at_wage(36.).score);
        assert_eq!(skilled.at_wage(36.).fraction, secure.at_wage(36.).fraction);
        for wage in [0., 0.01, 36., 1e9] {
            for candidate in [secure, strained, skilled] {
                assert!((0. ..=1.).contains(&candidate.at_wage(wage).fraction));
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
