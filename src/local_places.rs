//! Local actions reference canonical historical objects; surveys never mint discoveries.
use crate::{
    civilization::History,
    culture::Culture,
    gpu::{Generator, Stage},
    region::Region,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoveryRequest {
    pub site: u32,
    pub person: u32,
    pub artifact: u32,
    pub requested: u32,
    pub cause: u64,
}
impl Culture {
    pub(crate) fn validate_local_recoveries(&self, h: &History) -> Result<()> {
        let mut sites = BTreeSet::new();
        let mut objects = BTreeSet::new();
        for r in &self.local_recoveries {
            ensure!(
                (r.site as usize) < h.sites.len()
                    && (r.person as usize) < h.people.len()
                    && (r.artifact as usize) < self.artifacts.len()
                    && r.requested <= h.month
                    && sites.insert(r.site)
                    && objects.insert(r.artifact)
                    && h.events.get(r.cause as usize).is_some_and(|e| e.kind
                        == "local_recovery_requested"
                        && e.site == Some(r.site)
                        && e.month == r.requested
                        && e.subjects.contains(&("artifact".into(), r.artifact))
                        && e.subjects.contains(&("person".into(), r.person))),
                "invalid local recovery request"
            );
        }
        Ok(())
    }
    pub(crate) fn process_local_recoveries(&mut self, h: &mut History) -> BTreeSet<u32> {
        let mut handled = BTreeSet::new();
        for r in std::mem::take(&mut self.local_recoveries) {
            let a = &self.artifacts[r.artifact as usize];
            let available = !h.sites[r.site as usize].abandoned
                && self.site_people(h, r.site).contains(&r.person)
                && a.lost
                && !a.destroyed
                && a.site
                    .is_some_and(|s| h.sites[s as usize].cell == h.sites[r.site as usize].cell);
            if !available {
                h.event("local_recovery_cancelled",Some(r.site),None,"The resident, occupied site or surviving local object is no longer available; no recovery work or new object was credited".into());
                let e = h.events.last_mut().unwrap();
                e.causes.push(r.cause);
                e.subjects
                    .extend([("artifact".into(), r.artifact), ("person".into(), r.person)]);
                continue;
            }
            let work = self
                .labor_budget
                .get(r.site as usize)
                .copied()
                .unwrap_or(0.);
            if work < 0.1 {
                self.local_recoveries.push(r);
                continue;
            }
            if self.recover_specific(h, r.site, r.person, r.artifact as usize) {
                self.labor_spent += work as f64;
                h.events.last_mut().unwrap().causes.push(r.cause);
                handled.insert(r.site);
            }
        }
        handled
    }
}
impl Generator {
    /// Queue a selected local object for the next affordable quarterly cultural action.
    pub fn request_local_recovery(
        &mut self,
        region: &Region,
        site: u32,
        person: u32,
        artifact: u32,
    ) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "local recovery requires a completed boundary"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no historical world"))?;
        ensure!(
            region.seed == self.config.seed
                && region.epoch == self.progress.epoch
                && region.ecological_month == self.ecology.clock.month
                && region.history_month == Some(h.month),
            "stale or foreign regional survey"
        );
        let s = h
            .sites
            .get(site as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown site"))?;
        ensure!(
            !s.abandoned && region.cells.iter().any(|c| c.route[2] == s.cell),
            "recovery requires a resident settlement within the survey"
        );
        let c = h
            .culture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no cultural history"))?;
        ensure!(
            c.site_people(h, site).contains(&person),
            "recovery requires a living adult resident"
        );
        let a = c
            .artifacts
            .get(artifact as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown artifact"))?;
        ensure!(
            a.lost && !a.destroyed && a.site.is_some_and(|id| h.sites[id as usize].cell == s.cell),
            "object is not recoverable at this place"
        );
        let observed = region
            .historical_artifacts
            .iter()
            .find(|a| a.id == artifact)
            .ok_or_else(|| anyhow::anyhow!("artifact absent from survey"))?;
        ensure!(
            serde_json::to_value(observed)? == serde_json::to_value(a)?,
            "object changed since survey"
        );
        ensure!(
            !c.local_recoveries
                .iter()
                .any(|r| r.site == site || r.artifact == artifact),
            "site or object already has a recovery request"
        );
        h.event("local_recovery_requested",Some(site),None,"Regional recovery requested; existing cultural work must be reserved before the canonical object changes custody".into());
        let e = h.events.last_mut().unwrap();
        e.subjects
            .extend([("artifact".into(), artifact), ("person".into(), person)]);
        let request = RecoveryRequest {
            site,
            person,
            artifact,
            requested: h.month,
            cause: e.id,
        };
        h.culture.as_mut().unwrap().local_recoveries.push(request);
        Ok(())
    }
}
