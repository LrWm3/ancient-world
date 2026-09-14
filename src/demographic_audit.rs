//! Optional aggregate GPU-boundary observations. Never used to resolve outcomes.
use crate::society::{self, Demography};
use serde::{Deserialize, Serialize};

const MONTHS_PER_YEAR: u32 = 12;
const RETAINED_YEARS: usize = 200;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Audit {
    pub last_month: Option<u32>,
    pub years: Vec<Year>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Year {
    pub year: u32,
    pub months: u32,
    pub person_months: [f64; 3],
    pub base_deaths: f64,
    pub nutrition_deaths: f64,
    pub illness_deaths: f64,
    pub potential_births: f64,
    pub hunger_suppressed_births: f64,
    pub illness_suppressed_births: f64,
    pub observed_births: f64,
    pub observed_deaths: f64,
    pub food_need_kg: f64,
    pub physical_shortfall_kg: f64,
    pub access_shortfall_kg: f64,
}
impl Year {
    fn observe(&mut self, d: Demography, actual: [f64; 2]) {
        let need = d.ration_need.map(f64::from);
        let hunger: [f64; 3] = std::array::from_fn(|b| {
            if need[b] > 0. {
                (1. - d.ration_eaten[b] as f64 / need[b]).clamp(0., 1.)
            } else {
                0.
            }
        });
        let ages: [f64; 3] =
            std::array::from_fn(|b| need[b] / society::AGE_RATIONS_KG_PER_MONTH[b]);
        let illness = d.health[0].clamp(0., society::MAX_DISEASE_BURDEN) as f64;
        for b in 0..3 {
            self.person_months[b] += ages[b];
            let stress = if d.nutrition[3] > 0.5 {
                (d.nutrition[b] as f64).powi(2).max(
                    ((hunger[b] - society::ACUTE_HUNGER_THRESHOLD as f64)
                        * society::ACUTE_HUNGER_SCALE as f64)
                        .max(0.),
                )
            } else {
                hunger[b]
            };
            self.base_deaths += ages[b] * society::BASE_MONTHLY_MORTALITY[b];
            self.nutrition_deaths += ages[b] * stress * society::HUNGER_MORTALITY[b];
            self.illness_deaths += ages[b] * illness * society::DISEASE_MORTALITY;
        }
        // Ordered decomposition of multiplicative suppression, not independent causal effects.
        let potential = ages[1] * society::MONTHLY_BIRTH_RATE_PER_ADULT;
        self.potential_births += potential;
        self.hunger_suppressed_births += potential * hunger[1];
        self.illness_suppressed_births += potential * (1. - hunger[1]) * illness;
        self.observed_births += actual[0];
        self.observed_deaths += actual[1];
        let total_need = need[..3].iter().sum::<f64>();
        let unmet =
            (total_need - d.ration_eaten[..3].iter().map(|v| *v as f64).sum::<f64>()).max(0.);
        let physical = (total_need - d.household_food[2] as f64).max(0.).min(unmet);
        self.food_need_kg += total_need;
        self.physical_shortfall_kg += physical;
        self.access_shortfall_kg += unmet - physical;
    }
}
impl Audit {
    pub(crate) fn record(
        &mut self,
        month: u32,
        samples: impl Iterator<Item = (Demography, [f64; 2])>,
    ) {
        if self.last_month.is_some_and(|m| month <= m) {
            return;
        }
        self.last_month = Some(month);
        let year = month.saturating_sub(1) / MONTHS_PER_YEAR + 1;
        if self.years.last().is_none_or(|y| y.year != year) {
            if self.years.len() >= RETAINED_YEARS {
                self.years.remove(0);
            }
            self.years.push(Year {
                year,
                ..Default::default()
            });
        }
        let row = self.years.last_mut().unwrap();
        row.months += 1;
        for (d, actual) in samples {
            row.observe(d, actual);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn analytical_healthy_and_starved_exposures() {
        let mut d = Demography {
            ration_need: [100., 180., 140., 420.],
            ..Default::default()
        }; // Ten people in each band.
        d.ration_eaten = d.ration_need;
        d.household_food[2] = 420.;
        let mut r = Year::default();
        r.observe(d, [0.04, 0.041]);
        assert!((r.base_deaths - 0.041).abs() < 1e-12);
        assert_eq!(r.person_months, [10.; 3]);
        assert_eq!(r.potential_births, 0.04);
        assert_eq!(r.nutrition_deaths, 0.);
        d.ration_eaten = [0.; 4];
        d.nutrition = [0., 0., 0., 1.];
        let mut r = Year::default();
        r.observe(d, [0., 1.391]);
        assert!((r.nutrition_deaths - 1.35).abs() < 1e-12);
        assert_eq!(r.hunger_suppressed_births, 0.04);
        assert_eq!(r.access_shortfall_kg, 420.);
        assert_eq!(r.physical_shortfall_kg, 0.);
        d.household_food[2] = 100.;
        let mut r = Year::default();
        r.observe(d, [0., 0.]);
        assert_eq!(r.physical_shortfall_kg, 320.);
        assert_eq!(r.access_shortfall_kg, 100.);
    }
    #[test]
    fn annual_boundaries_and_resume_do_not_duplicate_months() {
        let mut a = Audit::default();
        a.record(12, std::iter::empty());
        let mut b: Audit = serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
        for x in [&mut a, &mut b] {
            x.record(12, std::iter::empty());
            x.record(13, std::iter::empty());
        }
        assert_eq!(a.years.len(), 2);
        assert_eq!(a.years[0].months, 1);
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
    }
}
