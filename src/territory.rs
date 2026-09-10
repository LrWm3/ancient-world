//! Change-only monthly political footprints, independent of present-day controllers.
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SiteControl {
    pub site: u32,
    pub cell: u32,
    pub controller: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snapshot {
    pub month: u32,
    pub politics_enabled: bool,
    pub sites: Vec<SiteControl>,
    /// Cell -> all claiming controllers. Overlap is intentionally retained.
    pub claims: BTreeMap<u32, BTreeSet<u32>>,
}
impl History {
    pub(crate) fn record_territory(&mut self) {
        let mut snapshot = Snapshot {
            month: self.month,
            politics_enabled: self.politics.is_some(),
            sites: vec![],
            claims: BTreeMap::new(),
        };
        if let Some(p) = &self.politics {
            snapshot.sites = self
                .sites
                .iter()
                .filter_map(|s| {
                    p.controllers
                        .get(s.id as usize)
                        .map(|&controller| SiteControl {
                            site: s.id,
                            cell: s.cell,
                            controller,
                        })
                })
                .collect();
            for c in &p.claims {
                snapshot
                    .claims
                    .entry(c.cell)
                    .or_default()
                    .extend(p.claim_owners(c));
            }
        }
        // A monthly boundary records its final state, not sub-month ordering.
        if self
            .territorial_history
            .last()
            .is_some_and(|s| s.month == self.month)
        {
            self.territorial_history.pop();
        }
        if self.territorial_history.last().is_none_or(|s| {
            s.politics_enabled != snapshot.politics_enabled
                || s.sites != snapshot.sites
                || s.claims != snapshot.claims
        }) {
            self.territorial_history.push(snapshot);
        }
    }
    /// None before the first recorded baseline, never reconstructed from current control.
    pub fn territory_at(&self, month: u32) -> Option<&Snapshot> {
        if month > self.month {
            return None;
        }
        self.territorial_history
            .iter()
            .rev()
            .find(|s| s.month <= month)
    }
    pub(crate) fn validate_territory(&self) -> Result<()> {
        let cells = 6 * self.terrain_resolution as u64 * self.terrain_resolution as u64;
        ensure!(
            self.territorial_history
                .windows(2)
                .all(|w| w[0].month < w[1].month),
            "unordered territory snapshots"
        );
        for s in &self.territorial_history {
            ensure!(
                s.month <= self.month
                    && (s.politics_enabled || (s.sites.is_empty() && s.claims.is_empty())),
                "invalid territory clock or disabled state"
            );
            let mut sites = BTreeSet::new();
            ensure!(
                s.sites.iter().all(|v| sites.insert(v.site)
                    && (v.site as usize) < self.sites.len()
                    && (v.cell as u64) < cells
                    && (v.controller as usize) < self.civilizations.len()),
                "invalid historical site control"
            );
            ensure!(
                s.claims.iter().all(|(&cell, owners)| (cell as u64) < cells
                    && !owners.is_empty()
                    && owners
                        .iter()
                        .all(|&id| (id as usize) < self.civilizations.len())),
                "invalid historical claim"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::Catalog,
        config::Config,
        gpu::{ContextGpu, Generator},
        politics::Claim,
    };
    #[test]
    #[ignore = "requires hardware GPU"]
    fn territorial_revisions_preserve_overlap_and_prior_control() {
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.territorial_history.clear();
        h.month = 10;
        let cell = h.sites[0].cell;
        let p = h.politics.as_mut().unwrap();
        p.controllers[0] = 0;
        p.controllers[1] = 1;
        p.claims = vec![Claim {
            cell,
            sites: vec![0, 1],
        }];
        h.record_territory();
        let baseline = h.territory_at(10).unwrap().clone();
        assert_eq!(baseline.claims[&cell], BTreeSet::from([0, 1]));
        assert!(h.territory_at(9).is_none());
        h.month = 11;
        h.record_territory();
        assert_eq!(h.territorial_history.len(), 1);
        h.politics.as_mut().unwrap().controllers[0] = 1;
        h.record_territory();
        assert_eq!(h.territorial_history.len(), 2);
        assert_eq!(h.territory_at(10).unwrap(), &baseline);
        assert_eq!(
            h.territory_at(11).unwrap().claims[&cell],
            BTreeSet::from([1])
        );
        h.politics.as_mut().unwrap().controllers[0] = 0;
        h.record_territory();
        assert_eq!(h.territorial_history.len(), 1); // no phantom intra-month revision
        h.month = 12;
        h.politics.as_mut().unwrap().controllers[0] = 1;
        h.record_territory();
        h.validate_territory().unwrap();
        let bytes = serde_json::to_value(&h).unwrap();
        let loaded: History = serde_json::from_value(bytes.clone()).unwrap();
        assert_eq!(loaded.territory_at(10), Some(&baseline));
        let mut legacy = bytes;
        legacy
            .as_object_mut()
            .unwrap()
            .remove("territorial_history");
        let legacy: History = serde_json::from_value(legacy).unwrap();
        assert!(legacy.territory_at(12).is_none());
        let old = g.spatial_territory(10).unwrap();
        assert_eq!(
            old.features
                .iter()
                .filter(|f| f.role == "claimed_cells")
                .count(),
            2
        );
        assert_eq!(
            g.spatial_territory(12)
                .unwrap()
                .features
                .iter()
                .filter(|f| f.role == "claimed_cells")
                .count(),
            1
        );
        old.geojson().unwrap();
        assert!(g.spatial_territory(9).is_err());
        assert!(g.spatial_territory(13).is_err());
    }
}
