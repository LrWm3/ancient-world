//! Funded institutional moves. Buildings remain at their original physical site.
use crate::{civilization::History, culture::Owner, gpu::Generator};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Move {
    pub institution: u32,
    pub from: u32,
    pub to: u32,
    pub route: u32,
    pub departed: u32,
    pub due: u32,
    pub arrived: Option<u32>,
    pub portable: Vec<u32>,
    pub paid: f64,
    pub cause: u64,
}
impl History {
    pub fn relocate_institution(&mut self, id: u32, to: u32) -> Result<()> {
        let c = self
            .culture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no culture"))?;
        let n = c
            .institutions
            .get(id as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown institution"))?;
        let destination = self
            .sites
            .get(to as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown destination"))?;
        ensure!(
            n.site != to && n.active && !destination.abandoned,
            "move requires an active institution and occupied destination"
        );
        ensure!(
            !c.relocations
                .iter()
                .any(|m| m.institution == id && m.arrived.is_none()),
            "institution already moving"
        );
        ensure!(
            c.work_receipt.settled || c.work_plans.is_empty(),
            "finish cultural work before moving"
        );
        let present = c.site_people(self, to);
        ensure!(
            n.members.iter().filter(|p| present.contains(p)).count() >= 2,
            "destination needs two present members"
        );
        let route = self
            .society
            .as_ref()
            .and_then(|s| {
                s.routes.iter().find(|r| {
                    r.passable()
                        && ((r.from == n.site && r.to == to) || (r.to == n.site && r.from == to))
                })
            })
            .ok_or_else(|| anyhow::anyhow!("no open direct relocation route"))?;
        let duration = (route.cost_km / 150.).ceil().max(1.) as u32;
        let quote = 10. + duration as f64 * 5.;
        let from = n.site;
        // Pay the origin's existing transport account with its representable increment.
        let before = self.sites[from as usize].economy.finance[0];
        let after = (before as f64 + quote) as f32;
        let paid = after as f64 - before as f64;
        ensure!(
            paid > 0. && n.treasury >= paid,
            "insufficient relocation funding"
        );
        let building = n
            .capacity
            .as_ref()
            .and_then(|x| x.building.as_ref())
            .map(|b| b.artifact);
        let portable: Vec<_> = c
            .artifacts
            .iter()
            .filter(|a| {
                a.owner == Owner::Institution(id)
                    && a.site == Some(from)
                    && !a.destroyed
                    && !a.lost
                    && Some(a.id) != building
                    && a.kind != "institutional foundation"
            })
            .map(|a| a.id)
            .collect();
        let route_id = route.id;
        self.event("institution_departure", Some(from), Some(to),
            format!("Institution {id} paid {paid:.2} to move portable property; its treasury remains the same institutional account"));
        let cause = self.events.last().unwrap().id;
        self.events
            .last_mut()
            .unwrap()
            .subjects
            .push(("institution".into(), id));
        self.sites[from as usize].economy.finance[0] = after;
        let c = self.culture.as_mut().unwrap();
        let n = &mut c.institutions[id as usize];
        n.treasury -= paid;
        n.expenses += paid;
        n.active = false; // No simultaneous services at either end during transit.
        for &a in &portable {
            c.artifacts[a as usize].site = None;
            c.artifacts[a as usize].custodian = None;
            c.artifacts[a as usize].events.push(cause);
        }
        c.relocations.push(Move {
            institution: id,
            from,
            to,
            route: route_id,
            departed: self.month,
            due: self.month + duration,
            arrived: None,
            portable,
            paid,
            cause,
        });
        Ok(())
    }
    pub(crate) fn institution_arrivals(&mut self) {
        let Some(mut c) = self.culture.take() else {
            return;
        };
        for m in &mut c.relocations {
            if m.arrived.is_some() || m.due > self.month {
                continue;
            }
            // A disrupted destination delays arrival; goods and money are not duplicated.
            if self.sites[m.to as usize].abandoned
                || self
                    .society
                    .as_ref()
                    .is_none_or(|s| s.routes.get(m.route as usize).is_none_or(|r| !r.passable()))
            {
                continue;
            }
            let n = &mut c.institutions[m.institution as usize];
            n.site = m.to;
            n.active = true;
            if let Some(cap) = &mut n.capacity {
                cap.building = None;
                cap.readiness = 0.;
                cap.mandate = None;
            }
            self.event(
                "institution_arrival",
                Some(m.to),
                Some(m.from),
                format!(
                    "{} re-established its seat; the old meeting place remains behind",
                    n.name
                ),
            );
            let e = self.events.last_mut().unwrap();
            e.causes.push(m.cause);
            e.subjects.push(("institution".into(), m.institution));
            for &id in &m.portable {
                let a = &mut c.artifacts[id as usize];
                if !a.destroyed && a.site.is_none() {
                    a.site = Some(m.to);
                    a.events.push(e.id);
                }
            }
            m.arrived = Some(self.month);
        }
        self.culture = Some(c);
    }
}
impl Generator {
    pub fn relocate_institution(&mut self, id: u32, to: u32) -> Result<()> {
        self.validate_living_boundary()?;
        self.civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?
            .relocate_institution(id, to)
    }
}
pub(crate) fn validate(c: &crate::culture::Culture, h: &History) -> Result<()> {
    let mut active = std::collections::BTreeSet::new();
    for m in &c.relocations {
        ensure!(
            (m.institution as usize) < c.institutions.len()
                && (m.from as usize) < h.sites.len()
                && (m.to as usize) < h.sites.len()
                && m.from != m.to
                && m.departed <= h.month
                && m.due > m.departed
                && m.arrived.is_none_or(|t| t >= m.due && t <= h.month)
                && m.paid.is_finite()
                && m.paid > 0.
                && (m.cause as usize) < h.events.len()
                && h.society
                    .as_ref()
                    .is_some_and(|s| (m.route as usize) < s.routes.len())
                && m.portable.iter().all(|a| (*a as usize) < c.artifacts.len()),
            "invalid institutional move"
        );
        if m.arrived.is_none() {
            ensure!(
                active.insert(m.institution) && !c.institutions[m.institution as usize].active,
                "duplicate active institutional move"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn funded_move_preserves_identity_property_and_checkpoint() {
        let mut g = crate::continuity_fixture::world();
        let h = g.civilizations.as_mut().unwrap();
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .find(|r| r.passable())
            .unwrap()
            .clone();
        let from = route.from;
        let to = route.to;
        let mut c = h.culture.take().unwrap();
        let members = c.site_people(h, to);
        assert!(members.len() >= 2);
        let id = c.institutions.len() as u32;
        let artifact = c.artifacts.len() as u32;
        c.artifacts.push(crate::culture::Artifact {
            id: artifact,
            name: "Travel manuscript".into(),
            kind: "manuscript".into(),
            creator: None,
            owner: Owner::Institution(id),
            claims: vec![],
            site: Some(from),
            custodian: None,
            materials: vec![],
            topic: Some(0),
            tradition: None,
            events: vec![],
            destroyed: false,
            lost: false,
        });
        let funds = h.sites[from as usize].economy.finance[0].min(1000.);
        h.sites[from as usize].economy.finance[0] -= funds;
        c.institutions.push(crate::culture::Institution {
            id,
            name: "Displaced school".into(),
            kind: crate::culture::InstitutionKind::Scholarly,
            site: from,
            tradition: None,
            leader: members[0],
            members,
            treasury: funds as f64,
            active: true,
            founded: h.month,
            knowledge: [0].into(),
            property: vec![artifact],
            dues: 0.,
            expenses: 0.,
            capacity: Some(crate::institution_capacity::Capacity::new(h.month)),
        });
        h.culture = Some(c);
        let treasury = h.culture.as_ref().unwrap().institutions[id as usize].treasury;
        h.relocate_institution(id, to).unwrap();
        assert!(h.relocate_institution(id, to).is_err());
        let c = h.culture.as_ref().unwrap();
        assert_eq!(c.artifacts[artifact as usize].site, None);
        assert!(c.institutions[id as usize].treasury < treasury);
        let mut resumed: History = serde_json::from_slice(&serde_json::to_vec(h).unwrap()).unwrap();
        let due = c.relocations.last().unwrap().due;
        h.month = due;
        resumed.month = due;
        h.institution_arrivals();
        resumed.institution_arrivals();
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let c = h.culture.as_ref().unwrap();
        assert_eq!(c.artifacts[artifact as usize].site, Some(to));
        assert_eq!(c.institutions[id as usize].site, to);
        validate(c, h).unwrap();
    }
}
