//! Scoped allocation policy for research/culture service work. No execution or payment here.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum Policy {
    /// Compatibility policy for archives lacking this field.
    #[default]
    ResearchFirst,
    /// Positive relative weights; unused entitlement is redistributed up to demand.
    Weighted { research: f32, culture: f32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub month: u32,
    pub site: u32,
    pub capacity: f32,
    /// Opening participation mode; absent in older receipts or without the framework.
    #[serde(default)]
    pub resolution_mode: Option<crate::resolution::Mode>,
    pub policy: Policy,
    /// Research, culture. Requests are captured before either reserves work.
    pub requested: [f32; 2],
    pub allocated: [f32; 2],
    /// Actual reservations can be smaller due to individual availability.
    pub reserved: [f32; 2],
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Allocation {
    pub policy: Policy,
    pub receipts: Vec<Receipt>,
}

impl Policy {
    pub fn allocate(&self, capacity: f32, requested: [f32; 2]) -> anyhow::Result<[f32; 2]> {
        anyhow::ensure!(
            capacity.is_finite()
                && capacity >= 0.
                && requested.iter().all(|x| x.is_finite() && *x >= 0.),
            "invalid service allocation request"
        );
        let [a, b] = requested.map(f64::from);
        let cap = f64::from(capacity);
        let grants = match self {
            Self::ResearchFirst => {
                let first = a.min(cap);
                [first, b.min(cap - first)]
            }
            Self::Weighted { research, culture } => {
                anyhow::ensure!(
                    [research, culture]
                        .iter()
                        .all(|x| x.is_finite() && **x > 0.),
                    "service allocation weights must be finite and positive"
                );
                let first =
                    cap * f64::from(*research) / (f64::from(*research) + f64::from(*culture));
                // Weighted water filling with demand caps; no iterations or actor ordering.
                let ga = a.min(first.max(cap - b));
                [ga, b.min(cap - ga)]
            }
        };
        Ok(grants.map(|x| x as f32))
    }
}

impl crate::civilization::History {
    pub(crate) fn reserve_learning_services(&mut self) -> anyhow::Result<()> {
        self.service_allocation.policy.allocate(0., [0.; 2])?;
        self.open_participation();
        self.sync_culture();
        let research = self.discovery_work_plans();
        let culture = self.cultural_work_plans();
        let mut receipts: Vec<_> = self
            .sites
            .iter()
            .map(|s| Receipt {
                resolution_mode: self.resolution.as_ref().map(|_| {
                    if self.participation.is_some() {
                        crate::resolution::Mode::Individual
                    } else {
                        crate::resolution::Mode::Aggregate
                    }
                }),
                policy: self.service_allocation.policy.clone(),
                month: self.month,
                site: s.id,
                capacity: crate::labor::available(s, self.society.is_some(), self.living.is_some()),
                requested: [0., 0.],
                allocated: [0.; 2],
                reserved: [0.; 2],
            })
            .collect();
        for (site, plan) in &research {
            receipts[*site as usize].requested[0] += plan.receipt.requested as f32;
        }
        for plan in &culture {
            receipts[plan.site as usize].requested[1] =
                plan.actions.iter().map(|(_, w)| *w).sum::<f32>().min(0.5);
        }
        for r in &mut receipts {
            r.allocated = self
                .service_allocation
                .policy
                .allocate(r.capacity, r.requested)?;
        }
        let research_caps: Vec<_> = receipts.iter().map(|r| r.allocated[0]).collect();

        let before: Vec<_> = self.sites.iter().map(|s| s.economy.external[3]).collect();
        self.reserve_discovery_plans(research, &research_caps);
        for (r, s) in receipts.iter_mut().zip(&self.sites) {
            r.reserved[0] = (s.economy.external[3] - before[s.id as usize]).max(0.);
        }
        // Strict priority lends unused research grants to culture, preserving the
        // old remaining-capacity behavior even when personal matching falls short.
        if matches!(self.service_allocation.policy, Policy::ResearchFirst) {
            for r in &mut receipts {
                r.allocated[1] = r.requested[1].min((r.capacity - r.reserved[0]).max(0.));
                r.allocated[0] = r.reserved[0];
            }
        }
        let culture_caps: Vec<_> = receipts.iter().map(|r| r.allocated[1]).collect();
        self.reserve_cultural_plans(culture, &culture_caps);
        for (r, s) in receipts.iter_mut().zip(&self.sites) {
            r.reserved[1] = (s.economy.external[3] - before[s.id as usize] - r.reserved[0]).max(0.);
        }
        self.service_allocation.receipts = receipts;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires hardware GPU"]
    fn matched_learning_claims_change_shares_without_spending_materials() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
        for seed in [17, 81, 256] {
            let mut g = Generator::new(
                pollster::block_on(ContextGpu::headless()).unwrap(),
                Config {
                    resolution: 32,
                    ecology_resolution: 16,
                    seed,
                    ..Default::default()
                },
                Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.found_civilizations(5).unwrap();
            g.enable_society().unwrap();
            g.enable_politics().unwrap();
            g.enable_governance().unwrap();
            g.enable_shipping().unwrap();
            g.enable_expeditions().unwrap();
            g.enable_discoveries().unwrap();
            let h = g.civilizations.as_mut().unwrap();
            // Isolate site-pool policy from personal matching and domestic care.
            let named_participation = h.participation.clone();
            h.participation = None;
            h.domestic = None;
            h.month = 12;
            h.resolution = Some(crate::resolution::ResolutionState {
                compare: true,
                ..Default::default()
            });
            // Ensure a real teaching opportunity rather than hoping a freshly
            // founded town happens to request cultural work this quarter.
            h.sync_culture();
            let mut c = h.culture.take().unwrap();
            let people = c.site_people(h, 0);
            assert!(people.len() >= 2);
            for a in &mut c.agents {
                a.knowledge.clear();
                a.traits = [0.; 6];
            }
            let actor = people[(h.month / 3) as usize % people.len()];
            c.agents[actor as usize].knowledge.insert(0);
            h.culture = Some(c);

            h.expeditions
                .as_mut()
                .unwrap()
                .discoveries
                .as_mut()
                .unwrap()
                .workshops = vec![crate::discoveries::Workshop {
                botanicals: Default::default(),
                work_plan: None,
                site: 0,
                enabled: true,
                samples: [1., 0.],
                studied: [0.; 2],
                processed: [0.; 2],
                curated: [0.; 2],
                learned: [None; 2],
                remedy: 0.,
                delivered: [1., 0.],
                causes: [None; 2],
                batches: [0; 2],
            }];
            h.sites[0].economy.goods[3] = 10.;
            h.sites[0].economy.goods[6] = 10.;
            h.begin_service_reservations();
            let capacity = crate::labor::available(&h.sites[0], true, false);
            assert!(capacity > 0.25);
            h.sites[0].economy.external[3] = capacity - 0.25;
            let base = h.clone();
            let mut equal = base.clone();
            equal.service_allocation.policy = Policy::Weighted {
                research: 1.,
                culture: 1.,
            };
            let mut restored: crate::civilization::History =
                serde_json::from_str(&serde_json::to_string(&equal).unwrap()).unwrap();
            let mut priority = base.clone();
            priority.reserve_learning_services().unwrap();
            equal.reserve_learning_services().unwrap();
            restored.reserve_learning_services().unwrap();
            assert_eq!(
                serde_json::to_value(&equal.service_allocation).unwrap(),
                serde_json::to_value(&restored.service_allocation).unwrap()
            );
            let p = &priority.service_allocation.receipts[0];
            let e = &equal.service_allocation.receipts[0];
            assert!(e.requested[1] > 0., "fixture must contain cultural demand");
            assert!(e.reserved[1] > p.reserved[1] + 1e-5);
            assert!(e.reserved[0] < p.reserved[0] - 1e-5);
            for outcome in [&priority, &equal] {
                for r in &outcome.service_allocation.receipts {
                    assert!(r.reserved.iter().sum::<f32>() <= r.capacity + 1e-5);
                    for k in 0..2 {
                        assert!(r.reserved[k] <= r.allocated[k] + 1e-5);
                    }
                }
                for (before, after) in base.sites.iter().zip(&outcome.sites) {
                    assert_eq!(before.economy.goods, after.economy.goods);
                    assert_eq!(before.economy.finance, after.economy.finance);
                }
            }
            // No culture demand: changing the policy cannot invent an allocation.
            let mut inactive = base.clone();
            inactive.month = 13;
            inactive.service_allocation.policy = Policy::Weighted {
                research: 1.,
                culture: 1.,
            };
            inactive.reserve_learning_services().unwrap();
            let r = &inactive.service_allocation.receipts[0];
            assert_eq!(r.allocated[1], 0.);
            assert!((r.reserved[0] - r.requested[0]).abs() < 1e-5);
            // An entitlement cannot manufacture available people. Both claims
            // retain their unmet demand when all residents are already committed.
            let mut blocked = base.clone();
            blocked.participation = named_participation;
            blocked.open_participation();
            for resident in blocked
                .participation
                .as_mut()
                .unwrap()
                .residents
                .values_mut()
            {
                resident.committed = resident.capacity;
            }
            blocked.service_allocation.policy = Policy::Weighted {
                research: 1.,
                culture: 1.,
            };
            blocked.reserve_learning_services().unwrap();
            let r = &blocked.service_allocation.receipts[0];
            assert!(r.allocated.iter().sum::<f32>() > 0.);
            assert_eq!(r.reserved, [0.; 2]);

            // Explicit priority should match the former sequential path's grants.
            let mut legacy = base.clone();
            legacy.prepare_discoveries();
            legacy.reserve_cultural_work();
            for (old, new) in legacy.sites.iter().zip(&priority.sites) {
                assert!((old.economy.external[3] - new.economy.external[3]).abs() < 1e-5);
            }
            println!(
                "seed {seed}: priority {:?}, weighted {:?}",
                p.reserved, e.reserved
            );
            // Execute both systems once, then compare receipts on/off against
            // the same resulting work and material state.
            let mut without_comparison = equal.clone();
            without_comparison.resolution.as_mut().unwrap().compare = false;
            for h in [&mut equal, &mut without_comparison] {
                h.sites[0].economy.labor[3] = 10.;
                let mut d = h.expeditions.as_mut().unwrap().discoveries.take().unwrap();
                d.month(h);
                h.expeditions.as_mut().unwrap().discoveries = Some(d);
                h.culture_month();
                h.settle_participation().unwrap();
                h.settle_learning_resolutions().unwrap();
                h.resolution
                    .as_ref()
                    .unwrap()
                    .validate(h.month, h.sites.len())
                    .unwrap();
            }
            let receipts = &equal.resolution.as_ref().unwrap().receipts;
            for system in [
                crate::resolution::System::Research,
                crate::resolution::System::Culture,
            ] {
                let r = receipts
                    .iter()
                    .find(|r| r.boundary.system == system && r.boundary.site == 0)
                    .unwrap();
                assert_eq!(r.metrics.len(), 3);
                assert!(r.metrics[2].actual > 0., "fixture must complete real work");
                assert!(r.metrics.iter().all(|m| m.unexplained().abs() < 1e-8));
            }
            let previous = serde_json::to_value(&equal.resolution).unwrap();
            assert!(equal.settle_learning_resolutions().is_err());
            assert_eq!(previous, serde_json::to_value(&equal.resolution).unwrap());
            // Comparison bookkeeping cannot alter inventory, people, events or learning.
            equal.resolution = None;
            without_comparison.resolution = None;
            assert_eq!(
                serde_json::to_value(&equal).unwrap(),
                serde_json::to_value(&without_comparison).unwrap()
            );
        }
    }
    #[test]
    fn policies_change_shares_and_redistribute_unused_demand() {
        let balanced = Policy::Weighted {
            research: 1.,
            culture: 1.,
        };
        assert_eq!(
            Policy::ResearchFirst.allocate(1., [1., 1.]).unwrap(),
            [1., 0.]
        );
        assert_eq!(balanced.allocate(1., [1., 1.]).unwrap(), [0.5, 0.5]);
        assert_eq!(balanced.allocate(1., [0.1, 1.]).unwrap(), [0.1, 0.9]);
        assert_eq!(balanced.allocate(1., [1., 0.1]).unwrap(), [0.9, 0.1]);
        assert_eq!(balanced.allocate(3., [1., 1.]).unwrap(), [1., 1.]);
        assert_eq!(balanced.allocate(0., [1., 1.]).unwrap(), [0., 0.]);
    }
    #[test]
    fn bounded_and_symmetric_under_scarcity() {
        for cap in [0., 0.001, 0.1, 1., 100.] {
            for a in [0., 0.01, 1., 99.] {
                for b in [0., 0.3, 2., 101.] {
                    let g = Policy::Weighted {
                        research: 3.,
                        culture: 1.,
                    }
                    .allocate(cap, [a, b])
                    .unwrap();
                    let r = Policy::Weighted {
                        research: 1.,
                        culture: 3.,
                    }
                    .allocate(cap, [b, a])
                    .unwrap();
                    assert!(g[0] <= a && g[1] <= b && g[0] + g[1] <= cap + 1e-5);
                    assert!((g[0] - r[1]).abs() < 1e-5 && (g[1] - r[0]).abs() < 1e-5);
                }
            }
        }
    }
    #[test]
    fn invalid_inputs_rejected_and_policy_roundtrips() {
        for weight in [0., -1., f32::NAN, f32::INFINITY] {
            assert!(Policy::Weighted {
                research: weight,
                culture: 1.
            }
            .allocate(1., [1.; 2])
            .is_err());
        }
        assert!(Policy::ResearchFirst.allocate(-1., [1.; 2]).is_err());
        assert!(Policy::ResearchFirst.allocate(1., [f32::NAN, 1.]).is_err());
        let p = Policy::Weighted {
            research: 2.,
            culture: 3.,
        };
        let restored: Policy = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(
            p.allocate(1., [1.; 2]).unwrap(),
            restored.allocate(1., [1.; 2]).unwrap()
        );
    }
}
