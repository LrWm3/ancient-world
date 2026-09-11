//! Dated requests carried by arriving households; never global shortage detection.
use crate::{
    civilization::{History, Shipment},
    relocation::Journey,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Appeal {
    pub origin: u32,
    pub host: u32,
    pub household: u32,
    pub reported: u32,
    pub received: u32,
    pub route: u32,
    pub population: f32,
    pub cause: u64,
    pub response: Option<u64>,
}
/// Appeal evidence is captured together; goods and carriers remain live reservations.
#[derive(Clone, Debug)]
pub(crate) struct ReliefObservations {
    month: u32,
    pending: Vec<(usize, Appeal, f32, bool, f32)>,
}
/// Simultaneous secular claims: food belongs to the donor; freight is shared
/// by incoming and outgoing journeys. No float atomics or history mutation here.
#[derive(Clone, Copy)]
struct ReliefClaim {
    host: usize,
    origin: usize,
    requested: f32,
}

fn allocate_relief(claims: &[ReliefClaim], food: &[f32], freight: &[f32]) -> Vec<f32> {
    let mut food_demand = vec![0f64; food.len()];
    let mut freight_demand = vec![0f64; freight.len()];
    for c in claims {
        food_demand[c.host] += c.requested as f64;
        freight_demand[c.host] += c.requested as f64;
        if c.origin != c.host {
            freight_demand[c.origin] += c.requested as f64;
        }
    }
    let factor = |available: f32, demand: f64| {
        if demand > 0. {
            (available as f64 / demand).clamp(0., 1.)
        } else {
            1.
        }
    };
    claims
        .iter()
        .map(|c| {
            let scale = factor(food[c.host], food_demand[c.host])
                .min(factor(freight[c.host], freight_demand[c.host]))
                .min(factor(freight[c.origin], freight_demand[c.origin]));
            let exact = c.requested as f64 * scale;
            let mut grant = exact as f32;
            if grant as f64 > exact {
                grant = f32::from_bits(grant.to_bits().saturating_sub(1));
            }
            // Below the existing minimum journey load: release it, do not send a token shipment.
            if grant >= 18. {
                grant
            } else {
                0.
            }
        })
        .collect()
}

impl History {
    pub(crate) fn relief_affinity(&self, from: u32, to: u32) -> f32 {
        let a = self.controller(from);
        let b = self.controller(to);
        if a == b {
            return 1.;
        }
        let diplomatic = self
            .governance
            .as_ref()
            .and_then(|g| {
                g.relations
                    .iter()
                    .find(|r| r.parties.contains(&a) && r.parties.contains(&b))
            })
            .map_or(0.25, |r| {
                (r.trust / 100. + (r.trade_contacts as f32 / 100.).min(0.2)).min(1.)
            });
        let kin = self.society.as_ref().is_some_and(|s| {
            s.households.iter().any(|hh| {
                hh.site == to
                    && hh
                        .parent
                        .is_some_and(|id| s.households[id as usize].site == from)
            })
        });
        let received = self
            .culture
            .as_ref()
            .filter(|c| c.religious_relief.enabled)
            .map_or(0., |c| c.religious_relief.received_kg(from, to));
        let trust =
            0.15 * received / (received + self.sites[from as usize].stocks.stock[0].max(1.) * 18.);
        let reciprocity = self.culture.as_ref().map_or(0., |c| {
            c.religious_relief.memory.reciprocity(to, from, self.month)
        });
        (diplomatic + if kin { 0.2 } else { 0. } + trust + reciprocity).min(1.)
    }
    pub(crate) fn receive_appeal(&mut self, j: &Journey) {
        if !j.seek_help || j.returning {
            return;
        }
        let Some(s) = &self.society else {
            return;
        };
        if !s.relocation.witnessed_relief {
            return;
        }
        // One origin crisis has one response opportunity per two years, across hosts.
        if s.relocation
            .appeals
            .iter()
            .rev()
            .any(|a| a.origin == j.from && self.month < a.received + 24)
        {
            return;
        }
        self.event("relief_appeal", Some(j.to), Some(j.from), format!(
            "Arriving household {} requests food and evacuation provisions for {:.1} residents left at home; report from month {}, {} months old", j.household, j.report_population, j.departed, self.month - j.departed));
        let e = self.events.last_mut().unwrap();
        e.causes.push(j.cause);
        e.subjects.push(("household".into(), j.household));
        let cause = e.id;
        self.society
            .as_mut()
            .unwrap()
            .relocation
            .appeals
            .push(Appeal {
                origin: j.from,
                host: j.to,
                household: j.household,
                reported: j.departed,
                received: self.month,
                route: j.route,
                population: j.report_population,
                cause,
                response: None,
            });
    }
    pub(crate) fn observe_relief(&self) -> ReliefObservations {
        let pending = self.society.as_ref().map_or_else(Vec::new, |s| {
            s.relocation
                .appeals
                .iter()
                .enumerate()
                .filter(|(_, a)| a.response.is_none() && a.received < self.month)
                .map(|(i, a)| {
                    let hostile = self.politics.as_ref().is_some_and(|p| {
                        p.wars.iter().any(|w| {
                            w.ended.is_none()
                                && ((w.attacker == self.controller(a.origin)
                                    && w.defender == self.controller(a.host))
                                    || (w.defender == self.controller(a.origin)
                                        && w.attacker == self.controller(a.host)))
                        })
                    });
                    (
                        i,
                        a.clone(),
                        self.relief_affinity(a.origin, a.host),
                        hostile,
                        self.sites[a.host as usize].stocks.stock[3],
                    )
                })
                .collect()
        });
        ReliefObservations {
            month: self.month,
            pending,
        }
    }

    pub(crate) fn answer_appeals_observed(
        &mut self,
        observations: &ReliefObservations,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            observations.month == self.month,
            "stale relief observations"
        );
        for (i, a, _, _, _) in &observations.pending {
            anyhow::ensure!(
                self.society
                    .as_ref()
                    .and_then(|s| s.relocation.appeals.get(*i))
                    .is_some_and(|live| live == a),
                "relief appeal changed after observation"
            );
        }
        self.answer_appeals_inner(observations);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn answer_appeals(&mut self) {
        let observations = self.observe_relief();
        self.answer_appeals_observed(&observations).unwrap();
    }

    fn answer_appeals_inner(&mut self, observations: &ReliefObservations) {
        let mut pending: Vec<_> = observations.pending.iter().collect();
        pending.sort_by_key(|(_, a, _, _, _)| (a.host, a.origin, a.cause, a.household));
        let food: Vec<_> = self
            .sites
            .iter()
            .map(|s| (s.stocks.stock[1] - s.stocks.stock[0] * 18. * 12.).max(0.))
            .collect();
        let freight: Vec<_> = self
            .sites
            .iter()
            .map(|s| self.land_freight_capacity(s.id))
            .collect();
        let claims: Vec<_> = pending
            .iter()
            .map(|&&(_, ref a, affinity, hostile, shortage)| {
                let r = &self.society.as_ref().unwrap().routes[a.route as usize];
                let months = (r.cost_km / 150.).ceil().max(1.) as u32;
                let allowed = affinity >= 0.2 + months as f32 * 0.025
                    && !hostile
                    && !self.sites[a.host as usize].abandoned
                    && r.open
                    && r.flood_months == 0
                    && self.month - a.reported <= 18
                    && shortage <= 0.01;
                ReliefClaim {
                    host: a.host as usize,
                    origin: a.origin as usize,
                    requested: if allowed {
                        (a.population * 18. * 3.)
                            .min(3000. / months as f32)
                            .min(food[a.host as usize])
                            .min(freight[a.host as usize])
                            .min(freight[a.origin as usize])
                    } else {
                        0.
                    },
                }
            })
            .collect();
        let grants = allocate_relief(&claims, &food, &freight);
        // Commit all funded secular shipments before any religious fallback may reserve resources.
        let mut order: Vec<_> = (0..pending.len()).collect();
        order.sort_by_key(|&k| grants[k] < 18.);
        for k in order {
            let &(i, ref a, affinity, hostile, shortage) = pending[k];
            let r = self.society.as_ref().unwrap().routes[a.route as usize].clone();
            let months = (r.cost_km / 150.).ceil().max(1.) as u32;
            let host = &self.sites[a.host as usize];
            let surplus = (host.stocks.stock[1] - host.stocks.stock[0] * 18. * 12.).max(0.);
            let willing = affinity >= 0.2 + months as f32 * 0.025;
            // Live checks protect against f32 subtraction rounding at the commit boundary.
            let amount = grants[k]
                .min(surplus)
                .min(self.land_freight_capacity(a.host))
                .min(self.land_freight_capacity(a.origin));
            if amount < 18. && grants[k] < 18. && self.sponsor_religious_relief(a) {
                let response = self.events.last().unwrap().id;
                self.society.as_mut().unwrap().relocation.appeals[i].response = Some(response);
                continue;
            }
            let host = &self.sites[a.host as usize];
            let reason = if host.abandoned {
                "host community failed"
            } else if hostile {
                "hostile political relations"
            } else if !r.open || r.flood_months > 0 {
                "route unavailable"
            } else if self.month - a.reported > 18 {
                "report too old"
            } else if !willing {
                "ties too weak for this distance"
            } else if amount < 18. || shortage > 0.01 {
                "insufficient safe host surplus"
            } else {
                "host can afford a limited shipment"
            };
            let kind = if amount >= 18. {
                "appeal_relief_sent"
            } else {
                "relief_appeal_declined"
            };
            self.event(
                kind,
                Some(a.host),
                Some(a.origin),
                format!(
                    "Dated household appeal: {:.1} kg relief, {} months travel, affinity {:.2}; {}",
                    amount, months, affinity, reason
                ),
            );
            let e = self.events.last_mut().unwrap();
            e.causes.push(a.cause);
            let response = e.id;
            self.society.as_mut().unwrap().relocation.appeals[i].response = Some(response);
            if amount >= 18. {
                self.sites[a.host as usize].stocks.stock[1] -= amount;
                self.shipments.push(Shipment {
                    from: a.host,
                    to: a.origin,
                    food_kg: amount,
                    arrives: self.month + months,
                    weather_delay_months: 0,
                    appeal_cause: Some(response),
                    relief_route: Some(a.route),
                });
            }
        }
    }
}

#[cfg(test)]
mod allocation_tests {
    use super::*;
    #[test]
    fn competing_appeals_share_food_without_first_claimant_advantage() {
        let claims = [
            ReliefClaim {
                host: 0,
                origin: 1,
                requested: 120.,
            },
            ReliefClaim {
                host: 0,
                origin: 2,
                requested: 60.,
            },
        ];
        let food = [90., 0., 0.];
        let freight = [f32::INFINITY; 3];
        assert_eq!(allocate_relief(&claims, &food, &freight), [60., 30.]);
        assert_eq!(
            allocate_relief(&[claims[1], claims[0]], &food, &freight),
            [30., 60.]
        );
        assert_eq!(allocate_relief(&claims, &[0.; 3], &freight), [0., 0.]);
    }
    #[test]
    fn incoming_and_outgoing_relief_share_endpoint_capacity() {
        let claims = [
            ReliefClaim {
                host: 0,
                origin: 1,
                requested: 100.,
            },
            ReliefClaim {
                host: 1,
                origin: 2,
                requested: 100.,
            },
        ];
        assert_eq!(
            allocate_relief(&claims, &[100.; 3], &[200., 80., 200.]),
            [40., 40.]
        );
        // Below-minimum allocations are released; the algorithm deliberately does not refill.
        assert_eq!(
            allocate_relief(&claims, &[100.; 3], &[200., 20., 200.]),
            [0., 0.]
        );
    }
    #[test]
    fn combined_constraints_never_overdraw() {
        for available in [0., 18., 31.7, 100., 1000.] {
            let claims: Vec<_> = (0..20)
                .map(|i| ReliefClaim {
                    host: i % 4,
                    origin: (i + 1) % 4,
                    requested: 18. + i as f32 * 7.3,
                })
                .collect();
            let food = [available; 4];
            let freight = [available * 1.5; 4];
            let grants = allocate_relief(&claims, &food, &freight);
            for site in 0..4 {
                let used_food: f64 = claims
                    .iter()
                    .zip(&grants)
                    .filter(|(c, _)| c.host == site)
                    .map(|(_, &v)| v as f64)
                    .sum();
                let used_freight: f64 = claims
                    .iter()
                    .zip(&grants)
                    .filter(|(c, _)| c.host == site || c.origin == site)
                    .map(|(_, &v)| v as f64)
                    .sum();
                assert!(used_food <= food[site] as f64);
                assert!(used_freight <= freight[site] as f64);
            }
        }
    }
}
