//! Dated requests carried by arriving households; never global shortage detection.
use crate::{
    civilization::{History, Shipment},
    relocation::Journey,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
        (diplomatic + if kin { 0.2 } else { 0. } + trust).min(1.)
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
    pub(crate) fn answer_appeals(&mut self) {
        let Some(s) = &self.society else {
            return;
        };
        let pending = s
            .relocation
            .appeals
            .iter()
            .enumerate()
            .filter(|(_, a)| a.response.is_none() && a.received < self.month)
            .map(|(i, a)| (i, a.clone()))
            .collect::<Vec<_>>();
        for (i, a) in pending {
            let r = self.society.as_ref().unwrap().routes[a.route as usize].clone();
            let months = (r.cost_km / 150.).ceil().max(1.) as u32;
            let affinity = self.relief_affinity(a.origin, a.host);
            let hostile = self.politics.as_ref().is_some_and(|p| {
                p.wars.iter().any(|w| {
                    w.ended.is_none()
                        && ((w.attacker == self.controller(a.origin)
                            && w.defender == self.controller(a.host))
                            || (w.defender == self.controller(a.origin)
                                && w.attacker == self.controller(a.host)))
                })
            });
            let host = &self.sites[a.host as usize];
            let surplus = (host.stocks.stock[1] - host.stocks.stock[0] * 18. * 12.).max(0.);
            let willing = affinity >= 0.2 + months as f32 * 0.025;
            let allowed = willing
                && !hostile
                && !host.abandoned
                && r.open
                && r.flood_months == 0
                && self.month - a.reported <= 18
                && host.stocks.stock[3] <= 0.01;
            let amount = if allowed {
                surplus
                    .min(a.population * 18. * 3.)
                    .min(3000. / months as f32)
            } else {
                0.
            };
            if amount < 18. && self.sponsor_religious_relief(&a) {
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
            } else if amount < 18. || host.stocks.stock[3] > 0.01 {
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
