//! Local recognition of a witnessed recovery, separate from ownership and theology.
use crate::{civilization::History, culture::Culture};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recognition {
    pub artifact: u32,
    pub expedition: u32,
    pub event: u64,
    pub month: u32,
    pub origin: u32,
    pub civilization: u32,
    pub tradition: u32,
    pub institution: Option<u32>,
    pub people: Vec<u32>,
    pub survival: f32,
    /// First receipt per settlement. Repeated rumors cannot refresh the achievement.
    pub witnesses: Vec<(u32, u32)>,
}

impl Recognition {
    pub fn weight(&self, observer: u32, month: u32) -> f32 {
        if !self
            .witnesses
            .iter()
            .any(|&(site, received)| site == observer && received <= month)
        {
            return 0.;
        }
        self.survival / (1. + month.saturating_sub(self.month) as f32 / 120.)
    }
}

/// Completed-snapshot rumor diffusion through this month's delivered commerce.
/// Traffic supplies a social contact opportunity, not a simulated onboard messenger.
pub fn spread(h: &History, c: &mut Culture) {
    for r in &mut c.heritage_renown {
        let mut next = vec![];
        for (a, b) in h.trade_contact.links(h.month) {
            for (from, to) in [(a, b), (b, a)] {
                if !h.sites[from as usize].abandoned
                    && !h.sites[to as usize].abandoned
                    && r.witnesses
                        .iter()
                        .any(|&(site, month)| site == from && month < h.month)
                    && !r.witnesses.iter().any(|&(site, _)| site == to)
                {
                    next.push(to);
                }
            }
        }
        next.sort_unstable();
        next.dedup();
        r.witnesses
            .extend(next.into_iter().map(|site| (site, h.month)));
    }
}

/// Saturating recognition prevents repeated similar objects producing unbounded status.
pub fn score(
    c: &Culture,
    observer: u32,
    month: u32,
    matches: impl Fn(&Recognition) -> bool,
) -> f32 {
    let total: f32 = c
        .heritage_renown
        .iter()
        .filter(|r| matches(r))
        .map(|r| r.weight(observer, month))
        .sum();
    total / (1. + total)
}

/// Visitors require actual accessible custody and maintained scholarly/religious space.
pub fn destination(
    c: &Culture,
    h: &History,
    observer: u32,
    tradition: u32,
    work: f32,
) -> Option<(u32, u32)> {
    c.heritage_renown
        .iter()
        .filter(|r| r.tradition == tradition && r.weight(observer, h.month) > 0.1)
        .filter_map(|r| {
            let a = c.artifacts.get(r.artifact as usize)?;
            let site = a.site?;
            if a.lost
                || a.destroyed
                || site == observer
                || h.sites[site as usize].abandoned
                || !c.institutions.iter().any(|n| {
                    n.site == site
                        && n.operational()
                        && matches!(
                            n.kind,
                            crate::culture::InstitutionKind::Religious
                                | crate::culture::InstitutionKind::Scholarly
                        )
                })
            {
                return None;
            }
            let route = h.society.as_ref()?.routes.iter().find(|q| {
                q.passable()
                    && ((q.from == observer && q.to == site)
                        || (q.to == observer && q.from == site))
            })?;
            if route.cost_km * 2. / 1200. > work.min(1.) {
                return None;
            }
            Some((r, site))
        })
        .max_by(|(a, _), (b, _)| {
            a.weight(observer, h.month)
                .total_cmp(&b.weight(observer, h.month))
                .then_with(|| b.artifact.cmp(&a.artifact))
        })
        .map(|(r, site)| (site, r.artifact))
}

pub fn validate(c: &Culture, h: &History) -> anyhow::Result<()> {
    let mut artifacts = std::collections::BTreeSet::new();
    for r in &c.heritage_renown {
        anyhow::ensure!(
            artifacts.insert(r.artifact)
                && (r.artifact as usize) < c.artifacts.len()
                && r.month <= h.month
                && (r.origin as usize) < h.sites.len()
                && (r.civilization as usize) < h.civilizations.len()
                && (r.tradition as usize) < c.traditions.len()
                && r.institution
                    .is_none_or(|i| (i as usize) < c.institutions.len())
                && r.people.iter().all(|&p| (p as usize) < h.people.len())
                && r.survival.is_finite()
                && (0. ..=1.).contains(&r.survival)
                && h.events
                    .get(r.event as usize)
                    .is_some_and(|e| e.kind == "heritage_fragment_received" && e.month == r.month),
            "invalid heritage recognition"
        );
        let mut sites = std::collections::BTreeSet::new();
        anyhow::ensure!(
            r.witnesses.iter().all(|&(site, month)| sites.insert(site)
                && (site as usize) < h.sites.len()
                && month >= r.month
                && month <= h.month),
            "invalid heritage witnesses"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognition_requires_witnesses_and_fades_without_repeated_awards() {
        let mut r = Recognition {
            artifact: 0,
            expedition: 0,
            event: 0,
            month: 12,
            origin: 0,
            civilization: 0,
            tradition: 0,
            institution: None,
            people: vec![0],
            survival: 0.5,
            witnesses: vec![(0, 12)],
        };
        assert_eq!(r.weight(1, 12), 0.);
        assert_eq!(r.weight(0, 12), 0.5);
        assert_eq!(r.weight(0, 132), 0.25);
        r.witnesses.push((1, 132));
        assert_eq!(r.weight(1, 132), 0.25, "late news does not rejuvenate fame");
    }
}
