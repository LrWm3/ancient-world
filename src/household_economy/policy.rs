//! Toy political platforms, not historical estimates. Spending still uses finite budgets.
use super::HouseholdEconomy;
use crate::civilization::History;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DistributionPolicy {
    pub common: f32,
    pub payroll: f32,
    pub dividends: f32,
    pub relief: f32,
    pub food_target: f32,
}
impl DistributionPolicy {
    pub fn baseline(e: &HouseholdEconomy) -> Self {
        Self {
            common: e.common_share,
            payroll: e.payroll_share,
            dividends: e.dividend_share,
            relief: e.relief_share,
            food_target: e.relief_target,
        }
    }
    pub fn valid(self) -> bool {
        [
            self.common,
            self.payroll,
            self.dividends,
            self.relief,
            self.food_target,
        ]
        .iter()
        .all(|x| x.is_finite() && (0. ..=1.).contains(x))
    }
    /// Move gradually toward a faction platform; hardship changes the platform, not stocks.
    fn revise(self, interest: usize, hunger: f32) -> Self {
        let platforms = [
            [0.65, 0.25, 0.01, 0.08, 0.90],  // growers
            [0.35, 0.20, 0.04, 0.04, 0.80],  // merchants
            [0.40, 0.25, 0.02, 0.04, 0.80],  // retainers
            [0.50, 0.35, 0.01, 0.07, 0.90],  // artisans
            [0.50, 0.25, 0.01, 0.06, 0.85],  // scholars
            [0.65, 0.20, 0.005, 0.12, 0.95], // congregations
            [0.80, 0.30, 0.0, 0.18, 1.0],    // bread leagues
            [0.70, 0.20, 0.005, 0.12, 0.95], // revivalists
            [0.40, 0.30, 0.02, 0.03, 0.80],  // warbands
        ];
        let mut target = platforms[interest];
        let pressure = (hunger * 4.).clamp(0., 1.);
        target[0] = (target[0] + 0.15 * pressure).min(0.95);
        target[2] *= 1. - pressure;
        target[3] += 0.08 * pressure;
        target[4] = (target[4] + 0.15 * pressure).min(1.);
        let current = [
            self.common,
            self.payroll,
            self.dividends,
            self.relief,
            self.food_target,
        ];
        let limits = [0.05, 0.05, 0.005, 0.02, 0.05];
        let next: [f32; 5] = std::array::from_fn(|i| {
            current[i] + (target[i] - current[i]).clamp(-limits[i], limits[i])
        });
        Self {
            common: next[0],
            payroll: next[1],
            dividends: next[2],
            relief: next[3],
            food_target: next[4],
        }
    }
    pub fn description(self) -> String {
        format!("common food {:.1}%, municipal payroll {:.1}%, dividends {:.1}%, council relief {:.1}%, food target {:.1}%",
            self.common*100.,self.payroll*100.,self.dividends*100.,self.relief*100.,self.food_target*100.)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingDistribution {
    pub policy: DistributionPolicy,
    pub decided: u32,
    pub effective: u32,
    pub cause: u64,
}
impl History {
    /// Explicit user configuration overrides this group of fields for every council.
    pub(crate) fn override_distribution(&mut self, relief: bool) {
        let Some(s) = &mut self.society else { return };
        let Some(e) = &s.household_economy else {
            return;
        };
        let baseline = DistributionPolicy::baseline(e);
        for c in &mut s.councils {
            let mut policy = c.distribution.unwrap_or(baseline);
            if relief {
                policy.relief = baseline.relief;
                policy.food_target = baseline.food_target;
            } else {
                policy.common = baseline.common;
                policy.payroll = baseline.payroll;
                policy.dividends = baseline.dividends;
            }
            c.distribution = Some(policy);
            c.pending_distribution = None;
        }
    }

    pub(crate) fn propose_distribution(&mut self, civ: usize, interest: usize) {
        let Some(s) = &self.society else { return };
        let Some(e) = &s.household_economy else {
            return;
        };
        let council = &s.councils[civ];
        if council.pending_distribution.is_some() || council.distribution_review == Some(self.month)
        {
            return;
        }
        let mut need = 0.;
        let mut gap = 0.;
        for a in &e.accounts {
            if a.food_site
                .is_some_and(|site| self.controller(site) as usize == civ)
                && a.need > 0.
            {
                need += a.need;
                gap += a.need * a.hunger;
            }
        }
        if need <= 0. {
            return;
        }
        let hunger = (gap / need) as f32;
        let old = council
            .distribution
            .unwrap_or_else(|| DistributionPolicy::baseline(e));
        let next = old.revise(interest, hunger);
        self.society.as_mut().unwrap().councils[civ].distribution_review = Some(self.month);
        if old == next {
            return;
        }
        self.event(
            "distribution_policy_scheduled",
            None,
            None,
            format!(
                "{}: {} council responding to {:.1}% household food deficit; {} effective month {}",
                self.civilizations[civ].name,
                crate::faction_interests::NAMES[interest],
                hunger * 100.,
                next.description(),
                self.month + 1
            ),
        );
        let event = self.events.last_mut().unwrap();
        event.subjects.push(("civilization".into(), civ as u32));
        self.society.as_mut().unwrap().councils[civ].pending_distribution =
            Some(PendingDistribution {
                policy: next,
                decided: self.month,
                effective: self.month + 1,
                cause: event.id,
            });
    }
    pub(crate) fn activate_distribution(&mut self) {
        let mut changes = vec![];
        if let Some(s) = &mut self.society {
            for c in &mut s.councils {
                if c.pending_distribution
                    .as_ref()
                    .is_some_and(|p| p.effective <= self.month)
                {
                    let p = c.pending_distribution.take().unwrap();
                    c.distribution = Some(p.policy);
                    changes.push((c.civilization, p));
                }
            }
        }
        for (c, p) in changes {
            self.event(
                "distribution_policy_effective",
                None,
                None,
                format!("Civilization {c}: {}", p.policy.description()),
            );
            let event = self.events.last_mut().unwrap();
            event.subjects.push(("civilization".into(), c));
            event.causes.push(p.cause);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn council_policy_changes_access_only_after_boundary_and_preserves_cash() {
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
        g.enable_politics().unwrap();
        let mut h = g.civilizations.as_ref().unwrap().clone();
        h.month = 12;
        let foreign = h
            .sites
            .iter()
            .find(|s| s.civilization != h.sites[0].civilization)
            .unwrap()
            .civilization;
        h.politics.as_mut().unwrap().controllers[0] = foreign;
        let e = h
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap();
        e.founding_access = None;
        e.payroll_share = 0.;
        e.dividend_share = 0.;
        e.relief_share = 0.;
        h.prepare_household_retail();
        let civ = h.controller(0) as usize;
        // Remove new spending opportunities so the only change is food entitlement.
        for site in &mut h.sites {
            site.economy.finance[0] = 0.;
        }
        for a in &mut h
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .accounts
        {
            a.cash = 0.;
        }
        let old = h.clone();
        h.propose_distribution(civ, 6);
        assert!(h.society.as_ref().unwrap().councils[civ]
            .distribution
            .is_none());
        let events = h.events.len();
        h.propose_distribution(civ, 1);
        assert_eq!(h.events.len(), events);
        h.activate_monthly_policies();
        assert!(h.society.as_ref().unwrap().councils[civ]
            .distribution
            .is_none());
        let mut resumed: History =
            serde_json::from_slice(&serde_json::to_vec(&h).unwrap()).unwrap();
        h.month += 1;
        resumed.month += 1;
        h.activate_monthly_policies();
        resumed.activate_monthly_policies();
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let events = h.events.len();
        h.activate_monthly_policies();
        assert_eq!(h.events.len(), events);
        let mut base = old;
        base.month += 1;
        let baseline = base.prepare_household_retail();
        let before_money = h.economy_residuals()[4];
        let plans = h.prepare_household_retail();
        assert!((h.economy_residuals()[4] - before_money).abs() < 1e-7);
        let mut affected = 0;
        let mut controls = 0;
        for (a, b) in baseline.iter().zip(&plans) {
            assert_eq!(a.site, b.site);
            if h.controller(a.site as u32) as usize == civ {
                assert!(b.free > a.free);
                affected += 1;
            } else {
                assert_eq!(a.free, b.free);
                controls += 1;
            }
        }
        assert!(affected > 0 && controls > 0);
        // Explicit configuration can override a political policy.
        h.society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .common_share = 0.2;
        h.override_distribution(false);
        assert_eq!(
            h.society.as_ref().unwrap().councils[civ]
                .distribution
                .unwrap()
                .common,
            0.2
        );
    }

    #[test]
    fn platforms_and_pressure_change_shares_without_unbounded_steps() {
        let base = DistributionPolicy {
            common: 0.5,
            payroll: 0.2,
            dividends: 0.01,
            relief: 0.05,
            food_target: 0.75,
        };
        let bread = base.revise(6, 0.);
        let merchant = base.revise(1, 0.);
        assert!(bread.common > merchant.common && bread.relief > merchant.relief);
        assert!(bread.dividends < merchant.dividends);
        let crisis = base.revise(1, 0.5);
        assert!(crisis.relief > merchant.relief && crisis.dividends < merchant.dividends);
        for interest in 0..9 {
            let mut p = base;
            for _ in 0..500 {
                p = p.revise(interest, 0.5);
                assert!(p.valid());
            }
        }
    }
}
