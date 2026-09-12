//! Conditional operating requests. Quotes do not escrow money or promise work.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Policy {
    #[default]
    Legacy,
    Operating,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub institution: u32,
    /// One year of administrative fees and component replacement at opening prices.
    pub target: f64,
    pub requested: f64,
    /// A conditional ceiling, not reserved cash. Execution rechecks the live balance.
    pub ceiling: f64,
    pub paid: f64,
    pub settled: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Budget {
    /// At most 0.5% of opening town cash, shared proportionally across requests.
    pub pool: f64,
    pub requests: Vec<Request>,
}
impl Budget {
    pub(crate) fn allocate(pool: f64, mut requests: Vec<Request>) -> Self {
        requests.sort_by_key(|r| r.institution);
        let total: f64 = requests.iter().map(|r| r.requested).sum();
        let scale = if total > 0. {
            (pool / total).min(1.)
        } else {
            0.
        };
        for r in &mut requests {
            r.ceiling = r.requested * scale;
        }
        Self { pool, requests }
    }
    pub(crate) fn validate(
        &self,
        institutions: &[crate::culture::Institution],
        site: u32,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.pool.is_finite() && self.pool >= 0.,
            "invalid institution funding pool"
        );
        for (i, r) in self.requests.iter().enumerate() {
            anyhow::ensure!(
                institutions
                    .get(r.institution as usize)
                    .is_some_and(|n| n.site == site)
                    && !self.requests[..i]
                        .iter()
                        .any(|q| q.institution == r.institution)
                    && [r.target, r.requested, r.ceiling, r.paid]
                        .iter()
                        .all(|v| v.is_finite() && *v >= 0.)
                    && r.requested <= r.target
                    && r.ceiling <= r.requested + 1e-9
                    && r.paid <= r.ceiling + 1e-9
                    && (r.settled || r.paid == 0.),
                "invalid institution operating request"
            );
        }
        anyhow::ensure!(
            self.requests.iter().map(|r| r.ceiling).sum::<f64>() <= self.pool + 1e-8,
            "institution funding pool oversubscribed"
        );
        Ok(())
    }
}
impl crate::culture::Culture {
    pub(crate) fn plan_institution_funding(
        &self,
        h: &crate::civilization::History,
        site: u32,
    ) -> Option<Budget> {
        if self.institution_funding != Policy::Operating {
            return None;
        }
        let e = &h.sites[site as usize].economy;
        let members = self.site_people(h, site);
        let requests = self
            .institutions
            .iter()
            .filter(|n| {
                n.active
                    && n.site == site
                    && h.month.is_multiple_of(3)
                    && n.members.iter().any(|p| members.contains(p))
            })
            .map(|n| {
                let repairs = n
                    .capacity
                    .as_ref()
                    .and_then(|c| c.building.as_ref())
                    .map_or(0., |b| {
                        let a = &self.artifacts[b.artifact as usize];
                        if a.destroyed
                            || a.lost
                            || a.site != Some(site)
                            || a.owner != crate::culture::Owner::Institution(n.id)
                        {
                            return 0.;
                        }
                        b.facility.as_ref().map_or_else(
                            || {
                                a.materials
                                    .iter()
                                    .filter(|(g, _)| *g == 5)
                                    .map(|(_, kg)| *kg as f64)
                                    .sum::<f64>()
                                    * (0.0025 + 0.08 * e.soil[3].clamp(0., 1.) as f64)
                                    * 4.
                                    * e.prices[5].max(0.01) as f64
                            },
                            |f| {
                                f.rooms
                                    .iter()
                                    .flat_map(|r| &r.components)
                                    .map(|p| {
                                        p.kg as f64
                                            * p.wear_at(e.soil[3]) as f64
                                            * 4.
                                            * e.prices[p.good as usize].max(0.01) as f64
                                    })
                                    .sum()
                            },
                        )
                    });
                let target = 2. + repairs;
                Request {
                    institution: n.id,
                    target,
                    requested: (target - n.treasury).max(0.),
                    ceiling: 0.,
                    paid: 0.,
                    settled: false,
                }
            })
            .collect();
        Some(Budget::allocate(
            e.finance[0].max(0.) as f64 * 0.005,
            requests,
        ))
    }
    /// Called only after the existing administration action passes work/actor guards.
    /// A captured operating request cannot fall back to the legacy donation rule.
    pub(crate) fn collect_institution_funding(
        &mut self,
        h: &mut crate::civilization::History,
        site: u32,
        institution: usize,
    ) -> Option<f64> {
        let n = self.institutions.get(institution)?;
        if !n.active || n.site != site || h.sites[site as usize].abandoned {
            return None;
        }
        let plan = self.work_plans.get_mut(site as usize)?;
        if plan.month != h.month || plan.cancellation.is_some() {
            return None;
        }
        let request = if let Some(budget) = &mut plan.funding {
            let r = budget.requests.iter_mut().find(|r| r.institution == n.id)?;
            if r.settled {
                return None;
            }
            r.settled = true;
            Some(r)
        } else {
            None
        };
        let cash = &mut h.sites[site as usize].economy.finance[0];
        let amount = request
            .as_ref()
            .map_or((*cash as f64 * 0.0005).min(2.), |r| {
                r.ceiling.min((r.target - n.treasury).max(0.))
            })
            .min(cash.max(0.) as f64)
            .max(0.);
        // Credit only the representable source withdrawal, never a larger f64 quote.
        let old = *cash;
        let mut remaining = (old as f64 - amount).max(0.) as f32;
        if old as f64 - remaining as f64 > amount {
            remaining = f32::from_bits(remaining.to_bits() + 1).min(old);
        }
        let paid = (old as f64 - remaining as f64).max(0.);
        *cash = remaining;
        self.institutions[institution].treasury += paid;
        self.institutions[institution].dues += paid;
        if let Some(r) = request {
            r.paid = paid;
        }
        Some(paid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requests_share_a_bounded_pool_without_order_bias() {
        let row = |id, need| Request {
            institution: id,
            target: need,
            requested: need,
            ceiling: 0.,
            paid: 0.,
            settled: false,
        };
        let input = vec![row(2, 6.), row(0, 2.), row(1, 0.)];
        let a = Budget::allocate(2., input.clone());
        let b = Budget::allocate(2., input.into_iter().rev().collect());
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&b).unwrap()
        );
        assert_eq!(
            a.requests.iter().map(|r| r.ceiling).collect::<Vec<_>>(),
            vec![0.5, 0., 1.5]
        );
        assert_eq!(
            Budget::allocate(20., a.requests.clone())
                .requests
                .iter()
                .map(|r| r.ceiling)
                .sum::<f64>(),
            8.
        );
        assert!(Budget::allocate(0., a.requests)
            .requests
            .iter()
            .all(|r| r.ceiling == 0.));
        assert_eq!(Policy::default(), Policy::Legacy);
    }
}
