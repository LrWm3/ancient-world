//! Negotiated title settlements heard within already-completed local office work.
use crate::{civilization::History, culture::Owner};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Remedy {
    Return,
    Compensation(f64),
    Unresolved,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Petition {
    pub artifact: u32,
    pub claimant: Owner,
    pub owner: Owner,
    pub site: u32,
    pub controller: u32,
    pub opened: u32,
    pub cause: u64,
    pub remedy: Remedy,
    pub consent: Vec<Owner>,
    pub resolved: Option<u32>,
    pub outcome: String,
    pub paid: f64,
}
fn local(h: &History, owner: &Owner, site: u32) -> bool {
    let Some(c) = &h.culture else { return false };
    match owner {
        Owner::Person(p) => c.site_people(h, site).contains(p),
        Owner::Institution(i) => c
            .institutions
            .get(*i as usize)
            .is_some_and(|n| n.active && n.site == site),
        Owner::Community(s) => *s == site && !h.sites[site as usize].abandoned,
    }
}
impl History {
    /// Consent records an explicit negotiated agreement, not a declaration that a claim is true.
    pub fn petition_artifact(
        &mut self,
        artifact: u32,
        claimant: Owner,
        remedy: Remedy,
        consent: Vec<Owner>,
    ) -> Result<()> {
        let c = self
            .culture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no culture"))?;
        let a = c
            .artifacts
            .get(artifact as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown artifact"))?;
        let site = a
            .site
            .ok_or_else(|| anyhow::anyhow!("object is not locally accessible"))?;
        ensure!(
            !a.destroyed && !a.lost && a.claims.contains(&claimant) && claimant != a.owner,
            "no actionable claim"
        );
        ensure!(
            local(self, &claimant, site),
            "claimant needs local representation"
        );
        ensure!(
            self.offices.as_ref().is_some_and(|o| o
                .seats
                .get(site as usize)
                .is_some_and(|o| o.holder().is_some())),
            "petition requires a local office"
        );
        if let Remedy::Compensation(p) = remedy {
            ensure!(p.is_finite() && p > 0. && p <= 1e9, "invalid compensation");
        }
        let g = self
            .governance
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no governance"))?;
        ensure!(
            !g.artifact_petitions
                .iter()
                .any(|p| p.artifact == artifact && p.resolved.is_none()),
            "object already petitioned"
        );
        let owner = a.owner.clone();
        self.event(
            "artifact_petition",
            Some(site),
            None,
            format!("A claimant petitioned for title settlement of object {artifact}"),
        );
        let cause = self.events.last().unwrap().id;
        self.events
            .last_mut()
            .unwrap()
            .subjects
            .push(("artifact".into(), artifact));
        let controller = self.controller(site);
        self.governance
            .as_mut()
            .unwrap()
            .artifact_petitions
            .push(Petition {
                artifact,
                claimant,
                owner,
                site,
                controller,
                opened: self.month,
                cause,
                remedy,
                consent,
                resolved: None,
                outcome: String::new(),
                paid: 0.,
            });
        Ok(())
    }
    pub(crate) fn resolve_artifact_petitions(&mut self) {
        let Some(mut g) = self.governance.take() else {
            return;
        };
        let mut used: std::collections::BTreeSet<_> = g
            .artifact_petitions
            .iter()
            .filter(|p| p.resolved == Some(self.month))
            .map(|p| p.site)
            .collect();
        for p in &mut g.artifact_petitions {
            if p.resolved.is_some() || p.opened >= self.month {
                continue;
            }
            let office = self
                .offices
                .as_ref()
                .and_then(|o| o.seats.get(p.site as usize));
            // One hearing occupies up to half a normal 0.1-month completed office service.
            // It is a use of that service, not another reservation or labor grant.
            let funded = office
                .is_some_and(|o| o.holder().is_some() && o.controller == p.controller)
                && self
                    .offices
                    .as_ref()
                    .and_then(|o| o.service.as_ref())
                    .is_some_and(|s| {
                        s.plans.iter().any(|plan| {
                            plan.site == p.site
                                && plan.controller == p.controller
                                && Some(plan.holder) == office.and_then(|o| o.holder())
                                && plan.work.month == self.month
                                && plan.work.settled
                                && plan.work.used >= 0.05
                        })
                    });
            if !funded || used.contains(&p.site) {
                if self.month < p.opened + 12 {
                    continue;
                }
                p.outcome = "unresolved: no funded hearing within twelve months".into();
            } else {
                used.insert(p.site);
                p.outcome = "unresolved: access, consent, title or funding changed".into();
                let current_controller = self
                    .politics
                    .as_ref()
                    .map_or(self.sites[p.site as usize].civilization, |x| {
                        x.controllers[p.site as usize]
                    });
                let a = &self.culture.as_ref().unwrap().artifacts[p.artifact as usize];
                let agreed = a.owner == p.owner
                    && a.claims.contains(&p.claimant)
                    && a.site == Some(p.site)
                    && !a.lost
                    && !a.destroyed
                    && current_controller == p.controller
                    && p.consent.contains(&a.owner)
                    && a.claims.iter().all(|o| p.consent.contains(o))
                    && a.custodian
                        .is_none_or(|i| p.consent.contains(&Owner::Person(i)))
                    && local(self, &p.owner, p.site)
                    && local(self, &p.claimant, p.site);
                if agreed {
                    match p.remedy {
                        Remedy::Return => {
                            let c = self.culture.as_mut().unwrap();
                            for n in &mut c.institutions {
                                n.property.retain(|&i| i != p.artifact);
                            }
                            if let Owner::Institution(i) = p.claimant {
                                c.institutions[i as usize].property.push(p.artifact);
                            }
                            let a = &mut c.artifacts[p.artifact as usize];
                            a.owner = p.claimant.clone();
                            a.custodian = if let Owner::Person(i) = p.claimant {
                                Some(i)
                            } else {
                                None
                            };
                            a.claims.clear();
                            p.outcome = "returned by agreement".into();
                        }
                        Remedy::Compensation(price) => {
                            // Institutions can buy out claims using their existing treasury.
                            if let Owner::Institution(payer) = p.owner {
                                let credit = match p.claimant {
                                    Owner::Institution(_) => price,
                                    _ => {
                                        let before = self.sites[p.site as usize].economy.finance[0];
                                        (before as f64 + price) as f32 as f64 - before as f64
                                    }
                                };
                                let c = self.culture.as_mut().unwrap();
                                if credit > 0. && c.institutions[payer as usize].treasury >= credit
                                {
                                    c.institutions[payer as usize].treasury -= credit;
                                    c.institutions[payer as usize].expenses += credit;
                                    match p.claimant {
                                        Owner::Institution(i) => {
                                            c.institutions[i as usize].treasury += credit
                                        }
                                        _ => {
                                            self.sites[p.site as usize].economy.finance[0] =
                                                (self.sites[p.site as usize].economy.finance[0]
                                                    as f64
                                                    + credit)
                                                    as f32
                                        }
                                    }
                                    c.artifacts[p.artifact as usize].claims.clear();
                                    p.paid = credit;
                                    p.outcome = "claims released for paid compensation".into();
                                }
                            }
                        }
                        Remedy::Unresolved => p.outcome = "unresolved by request".into(),
                    }
                }
            }
            p.resolved = Some(self.month);
            self.event(
                "artifact_judgment",
                Some(p.site),
                None,
                format!("Object {}: {}; paid {:.2}", p.artifact, p.outcome, p.paid),
            );
            let e = self.events.last_mut().unwrap();
            e.causes.push(p.cause);
            e.subjects.push(("artifact".into(), p.artifact));
            e.subjects.push(("office".into(), p.site));
            self.culture.as_mut().unwrap().artifacts[p.artifact as usize]
                .events
                .push(e.id);
        }
        self.governance = Some(g);
    }
}
pub(crate) fn validate(h: &History, petitions: &[Petition]) -> Result<()> {
    for p in petitions {
        ensure!(
            h.culture
                .as_ref()
                .is_some_and(|c| (p.artifact as usize) < c.artifacts.len())
                && (p.site as usize) < h.sites.len()
                && (p.controller as usize) < h.civilizations.len()
                && p.opened <= h.month
                && p.resolved.is_none_or(|m| m > p.opened && m <= h.month)
                && (p.cause as usize) < h.events.len()
                && p.paid.is_finite()
                && p.paid >= 0.,
            "invalid artifact petition"
        );
        if let Remedy::Compensation(p) = p.remedy {
            ensure!(p.is_finite() && p > 0. && p <= 1e9, "invalid compensation");
        }
    }
    Ok(())
}
impl crate::gpu::Generator {
    pub fn petition_artifact(
        &mut self,
        artifact: u32,
        claimant: Owner,
        remedy: Remedy,
        consent: Vec<Owner>,
    ) -> Result<()> {
        self.validate_living_boundary()?;
        self.civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?
            .petition_artifact(artifact, claimant, remedy, consent)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn title_settlement_requires_consent_and_completed_office_work() {
        let mut g = crate::continuity_fixture::world();
        g.civilizations
            .as_mut()
            .unwrap()
            .set_office_service(true)
            .unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let person = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.site == 0)
            .unwrap()
            .head;
        let c = h.culture.as_mut().unwrap();
        let id = c.artifacts.len() as u32;
        c.artifacts.push(crate::culture::Artifact {
            id,
            name: "Disputed keepsake".into(),
            kind: "keepsake".into(),
            creator: None,
            owner: Owner::Person(person),
            claims: vec![Owner::Community(0)],
            site: Some(0),
            custodian: Some(person),
            materials: vec![],
            topic: None,
            tradition: None,
            events: vec![],
            destroyed: false,
            lost: false,
        });
        h.petition_artifact(
            id,
            Owner::Community(0),
            Remedy::Return,
            vec![Owner::Person(person), Owner::Community(0)],
        )
        .unwrap();
        h.resolve_artifact_petitions();
        assert_eq!(
            h.culture.as_ref().unwrap().artifacts[id as usize].owner,
            Owner::Person(person)
        );
        let mut refused = g.civilizations.as_ref().unwrap().clone();
        refused.governance.as_mut().unwrap().artifact_petitions[0]
            .consent
            .clear();
        // Identical completed service with an unsigned agreement must retain title.
        g.advance_history(1).unwrap();
        refused.month = g.civilizations.as_ref().unwrap().month;
        refused.offices = g.civilizations.as_ref().unwrap().offices.clone();
        refused.resolve_artifact_petitions();
        assert_eq!(
            refused.culture.as_ref().unwrap().artifacts[id as usize].owner,
            Owner::Person(person)
        );
        assert!(refused.governance.as_ref().unwrap().artifact_petitions[0]
            .outcome
            .starts_with("unresolved"));

        let h = g.civilizations.as_ref().unwrap();
        assert_eq!(
            h.culture.as_ref().unwrap().artifacts[id as usize].owner,
            Owner::Community(0)
        );
        assert_eq!(
            h.governance.as_ref().unwrap().artifact_petitions[0].paid,
            0.
        );
        let terrain = g.snapshot().unwrap();
        h.validate(&terrain).unwrap();
    }
}
