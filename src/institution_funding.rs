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
    /// Independent member assignments settle after upkeep and generic cultural actions.
    /// New institutional knowledge must not invalidate their captured opening plans.
    pub(crate) fn execute_institution_administration(
        &mut self,
        h: &mut crate::civilization::History,
    ) {
        for site in 0..self.work_plans.len() {
            if self.work_plans[site].month != h.month || h.sites[site].abandoned {
                continue;
            }
            let count = self.work_plans[site]
                .administration
                .as_ref()
                .map_or(0, Vec::len);
            for index in 0..count {
                let p = &self.work_plans[site].administration.as_ref().unwrap()[index];
                let Some(n) = self.institutions.get(p.institution as usize) else {
                    continue;
                };
                if !n.active
                    || n.site as usize != site
                    || p.used > 0.
                    || p.granted < 0.05
                    || h.personal_grant_live(p.commitment) < 0.05
                {
                    continue;
                }
                let actor = p
                    .commitment
                    .and_then(|id| h.participation.as_ref()?.commitments.get(id as usize))
                    .and_then(|c| c.people.first())
                    .map(|(id, _)| *id);
                let Some(actor) = actor.filter(|id| {
                    n.members.contains(id) && self.site_people(h, site as u32).contains(id)
                }) else {
                    continue;
                };
                let ni = p.institution as usize;
                if self
                    .collect_institution_funding(h, site as u32, ni)
                    .is_none()
                {
                    continue;
                }
                let n = &mut self.institutions[ni];
                // Legacy organizations without capacity retain their immediate fee rule.
                if n.capacity.is_none() {
                    let fee = n.treasury.min(0.5);
                    n.treasury -= fee;
                    n.expenses += fee;
                    h.sites[site].economy.finance[0] += fee as f32;
                }
                n.knowledge
                    .extend(self.agents[actor as usize].knowledge.iter().copied());
                self.work_plans[site].administration.as_mut().unwrap()[index].used = 0.05;
                self.labor_spent += 0.05f32 as f64;
                crate::culture::work_requests::record_work(
                    &mut self.work_plans,
                    site as u32,
                    h.month,
                    0.05,
                );
            }
        }
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
        if plan.month != h.month || (plan.cancellation.is_some() && plan.administration.is_none()) {
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
    #[ignore = "requires hardware GPU"]
    fn named_administration_uses_member_work_once_despite_unrelated_cancellation() {
        use crate::{
            catalog::Catalog,
            config::Config,
            culture::{Institution, InstitutionKind},
            gpu::{ContextGpu, Generator},
            institution_capacity::Capacity,
        };
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
        let h = g.civilizations.as_mut().unwrap();
        h.set_individual_participation(true).unwrap();
        h.month = 3;
        h.begin_service_reservations();
        h.sync_culture();
        let member = h.culture.as_ref().unwrap().site_people(h, 0)[0];
        let c = h.culture.as_mut().unwrap();
        c.named_administration = true;
        c.institution_funding = Policy::Operating;
        c.agents[member as usize].knowledge.insert(11);
        c.institutions = vec![Institution {
            capacity: Some(Capacity::new(0)),
            id: 0,
            name: "Named administration fixture".into(),
            kind: InstitutionKind::Scholarly,
            site: 0,
            tradition: None,
            members: vec![member],
            leader: member,
            treasury: 0.,
            active: true,
            founded: 0,
            knowledge: Default::default(),
            property: vec![],
            dues: 0.,
            expenses: 0.,
        }];
        h.sites[0].economy.finance[0] = 100.;
        let opening = h.clone();
        // Normal, too-small shared grant, no grant, prior employment, revoked membership,
        // death after reservation, and stale captured month.
        for scenario in 0..7 {
            let mut case = opening.clone();
            if scenario == 3 {
                let state = case.participation.as_mut().unwrap();
                state
                    .reserve(
                        case.month,
                        0,
                        crate::participation::Activity::Research,
                        &[member],
                        state.available(member),
                    )
                    .unwrap();
            }
            let cap = match scenario {
                1 => 0.074,
                2 => 0.,
                _ => 0.075,
            };
            let plans = case.cultural_work_plans();
            case.reserve_cultural_plans(plans, &vec![cap; case.sites.len()]);
            let u = &case.culture.as_ref().unwrap().work_plans[0]
                .administration
                .as_ref()
                .unwrap()[0];
            assert_eq!(u.granted > 0., !matches!(scenario, 1..=3));
            if u.granted > 0. {
                assert!((u.granted - 0.05).abs() < 1e-6);
            }
            let mut resumed: crate::civilization::History =
                serde_json::from_slice(&serde_json::to_vec(&case).unwrap()).unwrap();
            for run in [&mut case, &mut resumed] {
                run.release_cultural_work();
                let mut c = run.culture.take().unwrap();
                c.work_plans[0].cancellation = Some("unrelated cultural actor unavailable".into());
                c.labor_budget.fill(0.);
                if scenario == 4 {
                    c.institutions[0].members.clear();
                }
                if scenario == 5 {
                    run.people[member as usize].died = Some(run.month);
                }
                if scenario == 6 {
                    c.work_plans[0].month -= 3;
                }
                c.maintain_institutions(run);
                c.execute_institution_administration(run);
                let performed = scenario == 0;
                let p = &c.work_plans[0];
                let u = &p.administration.as_ref().unwrap()[0];
                assert_eq!(u.used > 0., performed);
                assert_eq!(c.institutions[0].treasury, if performed { 0.5 } else { 0. });
                assert_eq!(c.institutions[0].knowledge.contains(&11), performed);
                assert_eq!(
                    run.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury,
                    100.
                );
                let after = serde_json::to_value(&c).unwrap();
                let cash = run.sites[0].economy.finance[0];
                c.execute_institution_administration(run);
                assert_eq!(after, serde_json::to_value(&c).unwrap());
                assert_eq!(cash, run.sites[0].economy.finance[0]);
                run.culture = Some(c);
                run.settle_participation().unwrap();
                if scenario != 6 {
                    run.validate_service_work().unwrap();
                }
                if performed {
                    let c = run.culture.as_ref().unwrap();
                    let u = &c.work_plans[0].administration.as_ref().unwrap()[0];
                    let grant = &run.participation.as_ref().unwrap().commitments
                        [u.commitment.unwrap() as usize];
                    assert!((grant.used - 0.05).abs() < 1e-6);
                    assert_eq!(grant.people[0].0, member);
                    let mut invalid = run.clone();
                    invalid.culture.as_mut().unwrap().work_plans[0]
                        .administration
                        .as_mut()
                        .unwrap()[0]
                        .used = 0.;
                    assert!(invalid.validate_service_work().is_err());
                }
            }
            assert_eq!(
                serde_json::to_value(&case).unwrap(),
                serde_json::to_value(&resumed).unwrap()
            );
        }
        let mut old = serde_json::to_value(opening.culture.as_ref().unwrap()).unwrap();
        old.as_object_mut().unwrap().remove("named_administration");
        let old: crate::culture::Culture = serde_json::from_value(old).unwrap();
        assert!(!old.named_administration);
        let plans = opening.cultural_work_plans();
        let mut old = serde_json::to_value(&plans[0]).unwrap();
        old.as_object_mut().unwrap().remove("administration");
        let old: crate::culture::work_requests::WorkPlan = serde_json::from_value(old).unwrap();
        assert!(old.administration.is_none());
    }

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
