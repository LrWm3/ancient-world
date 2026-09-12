//! Institution-funded responses to witnessed appeals. Affiliation never changes here.
use crate::{
    civilization::{History, Shipment},
    culture::InstitutionKind,
    relief::Appeal,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReligiousRelief {
    pub enabled: bool,
    pub missions: Vec<Mission>,
    #[serde(default)]
    pub memory: crate::social_memory::LocalMemory,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mission {
    /// Consumed in Open; later cultural reservations see only remaining room-time.
    #[serde(default)]
    pub room: Option<crate::institution_services::OpeningUse>,
    pub institution: u32,
    pub host: u32,
    pub recipient: u32,
    pub dispatched: u32,
    pub response: u64,
    pub promised_kg: f32,
    pub paid: f64,
    /// None while in transit; zero means lost. A loss is not automatically known remotely.
    pub delivered_kg: Option<f32>,
    pub outcome_event: Option<u64>,
    #[serde(default)]
    pub reciprocal: bool,
    /// Actual reverse deliveries credited to this obligation; never a money stock.
    #[serde(default)]
    pub returned_kg: f32,
    /// Portion of this delivery fulfilling older obligations, not creating a new one.
    #[serde(default)]
    pub repayment_kg: f32,
    #[serde(default)]
    pub report_food_months: Option<f32>,
}
impl ReligiousRelief {
    pub fn validate(&self, h: &History, institutions: usize, objects: usize) -> Result<()> {
        self.memory.validate(h)?;
        let mut responses = std::collections::BTreeSet::new();
        let mut room_boundaries = std::collections::BTreeSet::new();
        for m in &self.missions {
            ensure!(
                m.room
                    .as_ref()
                    .is_none_or(|r| r.valid(objects)
                        && room_boundaries.insert((m.institution, m.dispatched))),
                "invalid opening service use"
            );
            ensure!(
                (m.institution as usize) < institutions
                    && (m.host as usize) < h.sites.len()
                    && (m.recipient as usize) < h.sites.len()
                    && m.dispatched <= h.month
                    && m.returned_kg.is_finite()
                    && m.returned_kg >= 0.
                    && m.repayment_kg.is_finite()
                    && m.repayment_kg >= 0.
                    && m.repayment_kg <= m.delivered_kg.unwrap_or(0.)
                    && m.returned_kg
                        <= if m.reciprocal {
                            m.delivered_kg.unwrap_or(0.) - m.repayment_kg
                        } else {
                            0.
                        }
                    && m.report_food_months
                        .is_none_or(|v| v.is_finite() && (0. ..=24.).contains(&v))
                    && m.promised_kg.is_finite()
                    && m.promised_kg >= 18.
                    && m.paid.is_finite()
                    && m.paid > 0.
                    && m.delivered_kg
                        .is_none_or(|v| v.is_finite() && v >= 0. && v <= m.promised_kg)
                    && m.outcome_event.is_some() == m.delivered_kg.is_some()
                    && m.outcome_event
                        .is_none_or(|e| h.events.get(e as usize).is_some())
                    && h.events
                        .get(m.response as usize)
                        .is_some_and(|e| e.kind == "religious_relief_sent")
                    && responses.insert(m.response),
                "invalid religious relief mission"
            );
        }
        let returned: f64 = self.missions.iter().map(|m| m.returned_kg as f64).sum();
        let repaid: f64 = self.missions.iter().map(|m| m.repayment_kg as f64).sum();
        ensure!(
            (returned - repaid).abs() <= 1e-5 * repaid.max(1.),
            "reciprocal credit imbalance"
        );
        for m in &self.missions {
            let pending = h
                .shipments
                .iter()
                .filter(|s| s.appeal_cause == Some(m.response))
                .count();
            ensure!(
                pending == usize::from(m.delivered_kg.is_none()),
                "religious mission/cargo mismatch"
            );
        }
        Ok(())
    }
    pub fn owed_kg(&self, debtor: u32, creditor: u32) -> f32 {
        self.missions
            .iter()
            .filter(|m| m.reciprocal && m.recipient == debtor && m.host == creditor)
            .map(|m| (m.delivered_kg.unwrap_or(0.) - m.repayment_kg - m.returned_kg).max(0.))
            .sum()
    }
    fn settle_delivery(&mut self, index: usize, kg: f32) {
        let (host, recipient) = (self.missions[index].host, self.missions[index].recipient);
        let mut remaining = kg;
        for m in &mut self.missions[..index] {
            if m.reciprocal && m.recipient == host && m.host == recipient {
                let credit = remaining
                    .min((m.delivered_kg.unwrap_or(0.) - m.repayment_kg - m.returned_kg).max(0.));
                m.returned_kg += credit;
                remaining -= credit;
            }
        }
        self.missions[index].repayment_kg = kg - remaining;
    }
    pub fn institution_trust(&self, site: u32, institution: u32, population: f32) -> f32 {
        let received: f32 = self
            .missions
            .iter()
            .filter(|m| m.recipient == site && m.institution == institution)
            .filter_map(|m| m.delivered_kg)
            .sum();
        0.5 + 0.5 * received / (received + population.max(1.) * 18.)
    }
    /// Locally witnessed assistance, bounded and distinct from conversion or global reputation.
    pub fn received_kg(&self, site: u32, host: u32) -> f32 {
        self.missions
            .iter()
            .filter(|m| m.recipient == site && m.host == host)
            .filter_map(|m| m.delivered_kg)
            .sum()
    }
}
impl History {
    pub(crate) fn sponsor_religious_relief(&mut self, appeal: &Appeal) -> bool {
        let Some(c) = &self.culture else { return false };
        if !c.religious_relief.enabled {
            return false;
        }
        let Some(society) = &self.society else {
            return false;
        };
        let Some(route) = society.routes.get(appeal.route as usize) else {
            return false;
        };
        let months = (route.cost_km / 150.).ceil().max(1.) as u32;
        let host = &self.sites[appeal.host as usize];
        let hostile = self.politics.as_ref().is_some_and(|p| {
            p.wars.iter().any(|w| {
                w.ended.is_none()
                    && ((w.attacker == self.controller(appeal.host)
                        && w.defender == self.controller(appeal.origin))
                        || (w.defender == self.controller(appeal.host)
                            && w.attacker == self.controller(appeal.origin)))
            })
        });
        if hostile
            || !route.open
            || route.flood_months > 0
            || months > 12
            || self.month.saturating_sub(appeal.reported) > 18
            || host.abandoned
            || self.sites[appeal.origin as usize].abandoned
            || host.stocks.stock[3] > 0.01
        {
            return false;
        }
        let surplus = (host.stocks.stock[1] - host.stocks.stock[0] * 18. * 6.).max(0.);
        let price = host.economy.prices[crate::economy::FOOD].max(0.01);
        let witness_faith = c.household_faith.get(appeal.household as usize).copied();
        let present = c.site_people(self, appeal.host);
        let candidate = c
            .institutions
            .iter()
            .filter(|n| {
                n.site == appeal.host
                    && n.kind == InstitutionKind::Religious
                    && n.operational()
                    && present.contains(&n.leader)
                    && !c
                        .religious_relief
                        .missions
                        .iter()
                        .any(|m| m.institution == n.id && m.dispatched == self.month)
            })
            .filter_map(|n| {
                let room = crate::institution_services::opening_dispatch(
                    c,
                    n.id,
                    appeal.host,
                    self.month,
                )?;
                let t = n.tradition.and_then(|t| c.traditions.get(t as usize))?;
                let hospitality = t.themes.contains(&0);
                let learned = c
                    .religious_relief
                    .memory
                    .mutual_aid_sites
                    .contains(&appeal.host);
                let returning_help = c.religious_relief.owed_kg(appeal.host, appeal.origin) > 0.;
                if !hospitality && !learned && !returning_help && n.tradition != witness_faith {
                    return None;
                }
                let budget = n.treasury * if hospitality { 0.25 } else { 0.1 };
                let kg = surplus
                    .min(appeal.population * 18. * 3.)
                    .min(3000. / months as f32)
                    .min((budget / price as f64) as f32);
                // Match the exact representable municipal receipt, never overdraw the payer.
                let receipt = (host.economy.finance[0] + kg * price) - host.economy.finance[0];
                if !receipt.is_finite() || receipt <= 0. || receipt as f64 > budget {
                    return None;
                }
                let kg = kg.min(receipt / price);
                (kg >= 18.).then_some((n.id, kg, receipt, room))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1).then_with(|| b.0.cmp(&a.0)));
        let Some((institution, kg, paid, room)) = candidate else {
            return false;
        };
        let reciprocal = c.institutions[institution as usize]
            .tradition
            .is_some_and(|t| c.traditions[t as usize].themes.contains(&5));
        self.sites[appeal.host as usize].stocks.stock[1] -= kg;
        let report_food_months = Some(
            (self.sites[appeal.host as usize].stocks.stock[1]
                / (self.sites[appeal.host as usize].stocks.stock[0].max(1.) * 18.))
                .clamp(0., 24.),
        );
        self.sites[appeal.host as usize].economy.finance[0] += paid;
        let c = self.culture.as_mut().unwrap();
        let n = &mut c.institutions[institution as usize];
        n.treasury -= paid as f64;
        n.expenses += paid as f64;
        self.event("religious_relief_sent",Some(appeal.host),Some(appeal.origin),format!("Institution {institution} purchased {kg:.2} kg provisions for {paid:.2}; {months} months travel; witnessed household appeal; no conversion condition"));
        let e = self.events.last_mut().unwrap();
        e.causes.push(appeal.cause);
        e.subjects.push(("institution".into(), institution));
        e.subjects.push(("household".into(), appeal.household));
        e.detail.push_str(if reciprocal {
            "; reciprocal assistance: future help expected only against actual receipt"
        } else {
            "; gift: no new repayment expectation"
        });
        let response = e.id;
        self.culture
            .as_mut()
            .unwrap()
            .religious_relief
            .missions
            .push(Mission {
                room: Some(room),
                institution,
                host: appeal.host,
                recipient: appeal.origin,
                dispatched: self.month,
                response,
                promised_kg: kg,
                paid: paid as f64,
                delivered_kg: None,
                outcome_event: None,
                reciprocal,
                returned_kg: 0.,
                repayment_kg: 0.,
                report_food_months,
            });
        self.shipments.push(Shipment {
            from: appeal.host,
            to: appeal.origin,
            food_kg: kg,
            arrives: self.month + months,
            weather_delay_months: 0,
            appeal_cause: Some(response),
            relief_route: Some(appeal.route),
        });
        true
    }
    /// Called only after the existing shipment path has delivered or written off its food.
    pub(crate) fn religious_relief_outcome(&mut self, shipment: &Shipment, delivered: bool) {
        let Some(c) = &mut self.culture else { return };
        let Some(index) =
            c.religious_relief.missions.iter().position(|m| {
                Some(m.response) == shipment.appeal_cause && m.outcome_event.is_none()
            })
        else {
            return;
        };
        let m = &mut c.religious_relief.missions[index];
        let event = self.events.last_mut().unwrap();
        let kg = if delivered { shipment.food_kg } else { 0. };
        m.delivered_kg = Some(kg);
        m.outcome_event = Some(event.id);
        event.subjects.push(("institution".into(), m.institution));
        let (host, recipient, observed, food, cause) = (
            m.host,
            m.recipient,
            m.dispatched,
            m.report_food_months,
            event.id,
        );
        c.religious_relief.settle_delivery(index, kg);
        let repaid = c.religious_relief.missions[index].repayment_kg;
        event.detail.push_str(&format!(
            "; {repaid:.2} kg fulfilled prior reciprocal assistance"
        ));
        let learn = c.religious_relief.enabled
            && kg > 0.
            && !c
                .religious_relief
                .memory
                .mutual_aid_sites
                .contains(&recipient)
            && c.religious_relief
                .missions
                .iter()
                .filter(|m| m.recipient == recipient && m.delivered_kg.is_some_and(|v| v >= 18.))
                .count()
                >= 2;
        if learn {
            c.religious_relief.memory.mutual_aid_sites.push(recipient);
        }
        if kg > 0. {
            self.remember_arrival(recipient, host, observed, food, cause);
        }
        if learn {
            self.event("mutual_aid_learned",Some(recipient),Some(host),
                "Repeated witnessed deliveries established a local mutual-aid practice; religious affiliation unchanged; future orders still require funds and safe surplus".into());
            self.events.last_mut().unwrap().causes.push(cause);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mission(host: u32, recipient: u32, delivered: Option<f32>, reciprocal: bool) -> Mission {
        Mission {
            room: None,
            institution: 0,
            host,
            recipient,
            dispatched: 0,
            response: 0,
            promised_kg: 100.,
            paid: 100.,
            delivered_kg: delivered,
            outcome_event: delivered.map(|_| 0),
            reciprocal,
            returned_kg: 0.,
            repayment_kg: 0.,
            report_food_months: None,
        }
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn opening_dispatch_consumes_room_time_before_cultural_reservations() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                seed: 17,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 12;
        h.sync_culture();
        let host = 0;
        let origin = 1;
        let hh = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.site == host)
            .unwrap()
            .clone();
        let route = h.society.as_ref().unwrap().routes.len() as u32;
        h.society
            .as_mut()
            .unwrap()
            .routes
            .push(crate::society::Route {
                id: route,
                from: host,
                to: origin,
                cells: vec![h.sites[0].cell, h.sites[1].cell],
                cost_km: 150.,
                open: true,
                flood_months: 0,
                upkeep: None,
                road_bricks: 0.,
            });
        h.sites[0].stocks.stock[1] = h.sites[0].stocks.stock[0] * 108. + 1000.;
        h.sites[0].stocks.stock[3] = 0.;
        h.sites[0].economy.prices[crate::economy::FOOD] = 2.;
        let c = h.culture.as_mut().unwrap();
        c.religious_relief.enabled = true;
        let faith = c.site_faith[0];
        c.traditions[faith as usize].themes[0] = 0;
        let institution = c.institutions.len() as u32;
        let room = c.artifacts.len() as u32;
        c.artifacts.push(crate::culture::Artifact {
            id: room,
            name: "Declared dispatch room".into(),
            kind: "institutional foundation".into(),
            creator: None,
            owner: crate::culture::Owner::Institution(institution),
            claims: vec![],
            site: Some(host),
            custodian: None,
            materials: vec![(0, 20.)],
            topic: None,
            tradition: None,
            events: vec![],
            lost: false,
            destroyed: false,
        });
        c.institutions.push(crate::culture::Institution {
            id: institution,
            name: "Fixture relief order".into(),
            kind: InstitutionKind::Religious,
            site: host,
            tradition: Some(faith),
            members: vec![hh.head],
            leader: hh.head,
            treasury: 1000.,
            active: true,
            founded: 0,
            knowledge: Default::default(),
            property: vec![],
            dues: 0.,
            expenses: 0.,
            capacity: Some(crate::institution_capacity::Capacity {
                building: Some(crate::institution_capacity::MeetingPlace {
                    condition: 0.25,
                    ..crate::institution_capacity::MeetingPlace::new(room)
                }),
                ..crate::institution_capacity::Capacity::new(h.month)
            }),
        });
        let appeal = Appeal {
            host,
            origin,
            household: hh.id,
            reported: h.month,
            received: h.month,
            route,
            population: 10.,
            cause: 0,
            response: None,
        };
        let mut missing = h.clone();
        missing.culture.as_mut().unwrap().artifacts[room as usize].lost = true;
        let before = serde_json::to_value(&missing).unwrap();
        assert!(!missing.sponsor_religious_relief(&appeal));
        assert_eq!(before, serde_json::to_value(&missing).unwrap());
        let food = h.sites[0].stocks.stock[1];
        assert!(h.sponsor_religious_relief(&appeal));
        let c = h.culture.as_ref().unwrap();
        let mission = c.religious_relief.missions.last().unwrap();
        assert_eq!(mission.room.as_ref().unwrap().used, 0.1);
        assert!((food - h.sites[0].stocks.stock[1] - mission.promised_kg).abs() < 0.001);
        assert_eq!(h.shipments.last().unwrap().food_kg, mission.promised_kg);
        c.religious_relief
            .validate(h, c.institutions.len(), c.artifacts.len())
            .unwrap();
        let (remaining, group) =
            crate::institution_services::remaining_space(c, institution, host, h.month);
        assert_eq!((remaining, group), (0.4, 2.));
        let people = c.site_people(h, host);
        let student = *people.iter().find(|&&p| p != hh.head).unwrap();
        let plans = c.plan_service_space(
            h,
            host,
            Some(student),
            Some((0, hh.head, institution)),
            &[("study".into(), 0.1)],
            None,
        );
        assert_eq!(plans[0].opening_space, 0.4);
        let snapshot = serde_json::to_value(&h).unwrap();
        assert!(!h.sponsor_religious_relief(&appeal));
        assert_eq!(snapshot, serde_json::to_value(&h).unwrap());
        let restored: History = serde_json::from_value(snapshot).unwrap();
        assert_eq!(
            crate::institution_services::remaining_space(
                restored.culture.as_ref().unwrap(),
                institution,
                host,
                restored.month
            ),
            (0.4, 2.)
        );
        assert_eq!(
            crate::institution_services::remaining_space(
                restored.culture.as_ref().unwrap(),
                institution,
                host,
                restored.month + 1
            ),
            (0.5, 2.)
        );
    }

    #[test]
    fn obligations_follow_receipts_and_repayments_do_not_create_debt_cycles() {
        let mut r = ReligiousRelief::default();
        r.missions.push(mission(0, 1, None, true));
        assert_eq!(r.owed_kg(1, 0), 0.);
        r.missions[0].delivered_kg = Some(40.); // remainder was lost
        assert_eq!(r.owed_kg(1, 0), 40.);
        r.missions.push(mission(1, 0, Some(60.), true));
        r.settle_delivery(1, 60.);
        assert_eq!(r.owed_kg(1, 0), 0.);
        assert_eq!(r.owed_kg(0, 1), 20.);
        assert_eq!(r.missions[1].repayment_kg, 40.);
        r.missions.push(mission(0, 1, Some(20.), false));
        r.settle_delivery(2, 20.);
        assert_eq!(r.owed_kg(0, 1), 0.);
        assert_eq!(r.owed_kg(1, 0), 0.);
    }
}
