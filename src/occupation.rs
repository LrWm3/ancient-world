//! Finite occupation uses the campaign's existing people, equipment and food.
use crate::{
    civilization::History,
    society::{Raid, Society},
};

pub(crate) const DEFAULT_OCCUPATION_MONTHS: u32 = 3;
pub(crate) const MAX_OCCUPATION_MONTHS: u32 = 12;
const FULL_COERCION_SOLDIERS_PER_RESIDENT: f32 = 0.1;
impl Raid {
    pub(crate) fn return_duration(&self, society: &Society) -> u32 {
        if self.travel_months > 0 {
            return self.travel_months;
        }
        // Old archives did not retain a multi-hop itinerary; retain their documented fallback.
        society
            .routes
            .iter()
            .find(|r| {
                (r.from == self.origin && r.to == self.target)
                    || (r.to == self.origin && r.from == self.target)
            })
            .map_or(1, |r| {
                (r.cost_km / crate::society::LAND_TRAVEL_KM_PER_MONTH)
                    .ceil()
                    .max(1.) as u32
            })
    }
}
impl History {
    pub(crate) fn begin_occupation(&mut self, raid: &mut Raid) {
        let Some(p) = &self.politics else { return };
        let Some(w) = raid.war.and_then(|id| p.wars.get(id as usize)) else {
            return;
        };
        if p.occupation_months == 0 || w.outcome != "conquest" || raid.soldiers <= 0. {
            return;
        }
        raid.occupation_until = Some(self.month + p.occupation_months);
        raid.arrives = raid.occupation_until.unwrap();
        raid.returning = false;
        let peace = self.events.last().unwrap().id;
        self.event("occupation_started", Some(raid.target), Some(raid.origin),
            format!("Army {} stationed {:.1} soldiers until month {}; {:.1} kg carried provisions, no resident population or stock duplicated",raid.id,raid.soldiers,raid.arrives,raid.food));
        let e = self.events.last_mut().unwrap();
        e.causes.push(peace);
        raid.cause = e.id;
    }
    /// Called after the ordinary monthly campaign ration debit.
    pub(crate) fn occupation_month(&mut self, raid: &mut Raid, society: &Society) -> bool {
        let Some(until) = raid.occupation_until else {
            return false;
        };
        let months = raid.return_duration(society);
        let allegiance = raid
            .war
            .and_then(|id| self.politics.as_ref()?.wars.get(id as usize))
            .map(|w| w.attacker);
        let reason = if self.month >= until {
            Some("assignment completed")
        } else if self.sites[raid.target as usize].abandoned {
            Some("settlement abandoned")
        } else if allegiance != Some(self.controller(raid.target))
            || allegiance != Some(self.controller(raid.origin))
        {
            Some("political access lost")
        } else if raid.food
            < raid.soldiers
                * crate::military::SOLDIER_FOOD_KG_PER_MONTH
                * (months + crate::military::RETURN_PROVISION_MARGIN_MONTHS) as f32
        {
            Some("provisions reserved for withdrawal")
        } else {
            None
        };
        if let Some(reason) = reason {
            raid.occupation_until = None;
            raid.returning = true;
            raid.arrives = self.month + months;
            self.event("occupation_ended",Some(raid.target),Some(raid.origin),
                format!("Army {} withdrew: {reason}; {:.1} soldiers and {:.1} kg provisions begin the return journey",raid.id,raid.soldiers,raid.food));
            let e = self.events.last_mut().unwrap();
            e.causes.push(raid.cause);
            raid.cause = e.id;
        }
        true
    }
    /// Coercive presence can delay separation, but is not popular legitimacy.
    pub fn occupation_strength(&self, site: u32) -> f32 {
        let Some(s) = self.sites.get(site as usize) else {
            return 0.;
        };
        if s.abandoned {
            return 0.;
        }
        let soldiers = self.society.as_ref().map_or(0., |soc| {
            soc.raids
                .iter()
                .filter(|r| {
                    r.target == site
                        && r.occupation_until.is_some_and(|m| m > self.month)
                        && self.controller(r.origin) == self.controller(site)
                        && r.war.is_some_and(|id| {
                            self.politics.as_ref().is_some_and(|p| {
                                p.wars[id as usize].attacker == self.controller(site)
                            })
                        })
                })
                .map(|r| r.soldiers)
                .sum::<f32>()
        });
        (soldiers / (s.stocks.stock[0] * FULL_COERCION_SOLDIERS_PER_RESIDENT).max(1.)).clamp(0., 1.)
    }
}

impl crate::gpu::Generator {
    /// Changes future occupation assignments; existing troops keep their recorded end date.
    pub fn set_occupation_months(&mut self, months: u32) -> anyhow::Result<()> {
        self.validate_living_boundary()?;
        anyhow::ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "occupation policy requires a completed boundary"
        );
        anyhow::ensure!(
            months <= MAX_OCCUPATION_MONTHS,
            "occupation duration must be 0–12 months"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let p = h
            .politics
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable politics first"))?;
        let prior = p.occupation_months;
        if prior != months {
            p.occupation_months = months;
            h.event("occupation_policy",None,None,format!("Future occupation assignments changed from {prior} to {months} months; carried supplies still constrain actual service"));
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
    };
    #[test]
    #[ignore = "requires hardware GPU"]
    fn finite_occupation_withdraws_without_duplicating_troops_or_supplies() {
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 64,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(16).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        let h = g.civilizations.as_ref().unwrap();
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .find(|r| r.cost_km < 900. && h.controller(r.from) != h.controller(r.to))
            .unwrap()
            .clone();
        g.declare_war(route.from, route.to).unwrap();
        let arrives = g
            .civilizations
            .as_ref()
            .unwrap()
            .society
            .as_ref()
            .unwrap()
            .raids[0]
            .arrives;
        g.advance_history(arrives).unwrap();
        g.enable_governance().unwrap();
        let h = g.civilizations.as_ref().unwrap();
        let raid = &h.society.as_ref().unwrap().raids[0];
        assert!(raid.occupation_until.is_some());
        assert!(!raid.returning);
        assert!(h.occupation_strength(route.to) > 0.);
        assert!(h.population_residual().abs() < 0.001);
        let mut governed = h.clone();
        let mut without = governed.clone();
        without.society.as_mut().unwrap().raids[0].occupation_until = None;
        governed.governance_month();
        without.governance_month();
        let a = &governed.governance.as_ref().unwrap().administrations[route.to as usize];
        let b = &without.governance.as_ref().unwrap().administrations[route.to as usize];
        assert!(
            a.unrest > b.unrest && a.loyalty < b.loyalty,
            "coercive presence must not count as consent"
        );
        assert_eq!(
            governed.sites[route.to as usize].economy.finance[0],
            without.sites[route.to as usize].economy.finance[0]
        );

        let mut continued: History =
            serde_json::from_slice(&serde_json::to_vec(h).unwrap()).unwrap();
        let mut uninterrupted = h.clone();
        for _ in 0..12 {
            uninterrupted.month += 1;
            uninterrupted.social_month().unwrap();
            continued.month += 1;
            continued.social_month().unwrap();
        }
        assert_eq!(
            serde_json::to_value(&uninterrupted).unwrap(),
            serde_json::to_value(&continued).unwrap()
        );
        assert!(uninterrupted.society.as_ref().unwrap().raids.is_empty());
        assert!(uninterrupted
            .events
            .iter()
            .any(|e| e.kind == "occupation_ended" && !e.causes.is_empty()));
        assert!(uninterrupted.population_residual().abs() < 0.001);
        assert!(uninterrupted.food_residual().abs() < 0.001);
        assert!(uninterrupted
            .economy_residuals()
            .iter()
            .all(|r| r.abs() < 0.001));
        // A shortage is declared as consumed food, with the same external ledger debit.
        let mut shortage = h.clone();
        let r = &mut shortage.society.as_mut().unwrap().raids[0];
        let removed = r.food;
        r.food = 0.;
        shortage.sites[route.from as usize].stocks.ledger[1] += removed;
        for (k, v) in crate::economy::FOOD_CNP.iter().enumerate() {
            shortage.sites[route.from as usize].economy.external[k] -= removed * *v as f32;
        }
        shortage.month += 1;
        shortage.social_month().unwrap();
        let r = &shortage.society.as_ref().unwrap().raids[0];
        assert!(r.returning && r.occupation_until.is_none());
        assert!(r.soldiers < raid.soldiers);
        assert_eq!(shortage.occupation_strength(route.to), 0.);
        assert!(shortage
            .events
            .iter()
            .rev()
            .find(|e| e.kind == "occupation_ended")
            .unwrap()
            .detail
            .contains("provisions reserved"));
        assert!(shortage.food_residual().abs() < 0.001);
        assert!(shortage.population_residual().abs() < 0.001);
        // Changing political access triggers withdrawal even with full food reserves.
        let mut recaptured = h.clone();
        let foreign = h.sites[route.to as usize].civilization;
        recaptured.politics.as_mut().unwrap().controllers[route.from as usize] = foreign;
        recaptured.politics.as_mut().unwrap().controllers[route.to as usize] = foreign;
        assert_eq!(
            recaptured.occupation_strength(route.to),
            0.,
            "two conquered towns do not change the army's allegiance"
        );
        let mut displaced = h.clone();
        displaced.politics.as_mut().unwrap().controllers[route.to as usize] =
            h.sites[route.to as usize].civilization;
        displaced.month += 1;
        displaced.social_month().unwrap();
        assert!(displaced
            .events
            .iter()
            .rev()
            .find(|e| e.kind == "occupation_ended")
            .unwrap()
            .detail
            .contains("political access lost"));
    }
}
