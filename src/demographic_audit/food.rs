//! Optional food observations; net unclassified flow is not proof of conservation.
use crate::civilization::Site;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const RETAINED_MONTHS: u32 = 60;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Boundary {
    pub food_kg: f64,
    /// Cumulative edible production, recorded consumption, recorded spoilage.
    pub ledger_kg: [f64; 3],
}
impl Boundary {
    pub fn from_site(site: &Site) -> Self {
        Self {
            food_kg: site.stocks.stock[1] as f64,
            ledger_kg: std::array::from_fn(|i| site.stocks.ledger[i] as f64),
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Month {
    pub month: u32,
    pub site: u32,
    pub opening: Option<Boundary>,
    pub before_gpu: Boundary,
    pub after_gpu: Boundary,
    pub closing: Option<Boundary>,
    pub need_kg: f64,
    pub eaten_kg: f64,
    pub physically_available_kg: f64,
    pub cultivated_ha: f32,
    pub tool_multiplier: f32,
    pub limiting_resource: f32,
    pub crop_harvest_kg: [f32; 6],
    /// Exact managed-growth dispatch inputs, not ending soil stocks.
    #[serde(default)]
    pub crop_probe: [[f32; 4]; 3],
    #[serde(default)]
    pub phosphorus_probe: [f32; 4],
    #[serde(default)]
    pub farm_workers: [f32; 4],
    #[serde(default)]
    pub land_capacity_ha: f32,
    #[serde(default)]
    pub illness: f32,
    #[serde(default)]
    pub funded_entitlement_kg: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Trace {
    pub months: Vec<Month>,
}
impl Month {
    /// Ordered diagnostic decomposition: satisfy need from local physical food
    /// first, then attribute the remaining unmet ration to entitlement/access.
    /// This is an accounting identity, not two independent intervention effects.
    pub fn shortfalls_kg(&self) -> [f64; 2] {
        let unmet = (self.need_kg - self.eaten_kg).max(0.);
        let physical = (self.need_kg - self.physically_available_kg)
            .max(0.)
            .min(unmet);
        [physical, unmet - physical]
    }
}
impl Trace {
    pub fn record(&mut self, month: u32, site: &Site, before_gpu: Boundary) {
        self.months.push(Month {
            month,
            site: site.id,
            before_gpu,
            after_gpu: Boundary::from_site(site),
            need_kg: site.demography.ration_need[..3]
                .iter()
                .map(|x| *x as f64)
                .sum(),
            eaten_kg: site.demography.ration_eaten[..3]
                .iter()
                .map(|x| *x as f64)
                .sum(),
            physically_available_kg: site.demography.household_food[2] as f64,
            cultivated_ha: site.economy.production_probe[1],
            tool_multiplier: site.economy.production_probe[0],
            limiting_resource: site.economy.diagnostics[0],
            crop_harvest_kg: site.economy.crops.map(|c| c[3]),
            crop_probe: site.economy.crop_probe,
            phosphorus_probe: site.economy.phosphorus_probe,
            farm_workers: site.economy.farm_workers,
            land_capacity_ha: site.stocks.habitat[1],
            illness: site.demography.health[0],
            funded_entitlement_kg: site.demography.household_food[0],
            ..Default::default()
        });
    }
    pub fn finish(&mut self, month: u32, opening: &BTreeMap<u32, Boundary>, sites: &[Site]) {
        let closing: BTreeMap<_, _> = sites
            .iter()
            .map(|s| (s.id, Boundary::from_site(s)))
            .collect();
        self.months
            .retain(|r| month.saturating_sub(r.month) < RETAINED_MONTHS);
        for row in self
            .months
            .iter_mut()
            .rev()
            .take_while(|r| r.month == month)
        {
            row.opening = opening.get(&row.site).copied();
            row.closing = closing.get(&row.site).copied();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortfalls_distinguish_stock_from_access() {
        for (available, eaten, expected) in [
            (120., 100., [0., 0.]),
            (120., 40., [0., 60.]),
            (60., 60., [40., 0.]),
            (60., 40., [40., 20.]),
            (0., 0., [100., 0.]),
        ] {
            let row = Month {
                need_kg: 100.,
                eaten_kg: eaten,
                physically_available_kg: available,
                ..Default::default()
            };
            assert_eq!(row.shortfalls_kg(), expected);
            assert_eq!(expected.iter().sum::<f64>(), 100. - eaten);
        }
    }

    #[test]
    fn retention_identity_and_missing_boundaries_survive_resume() {
        let mut trace = Trace {
            months: vec![
                Month {
                    month: 1,
                    site: 3,
                    ..Default::default()
                },
                Month {
                    month: 2,
                    site: 3,
                    ..Default::default()
                },
                Month {
                    month: 61,
                    site: 8,
                    ..Default::default()
                },
                Month {
                    month: 61,
                    site: 3,
                    ..Default::default()
                },
            ],
        };
        let opening = BTreeMap::from([(
            3,
            Boundary {
                food_kg: 125.,
                ledger_kg: [0.; 3],
            },
        )]);
        trace.finish(61, &opening, &[]);
        assert_eq!(trace.months.len(), 3);
        assert_eq!(trace.months[0].month, 2);
        assert!(trace.months[1].opening.is_none());
        assert_eq!(trace.months[2].opening.unwrap().food_kg, 125.);
        assert!(trace.months[2].closing.is_none());
        let mut resumed: Trace =
            serde_json::from_value(serde_json::to_value(&trace).unwrap()).unwrap();
        for t in [&mut trace, &mut resumed] {
            t.finish(62, &BTreeMap::new(), &[]);
        }
        assert_eq!(
            serde_json::to_value(trace).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
    }

    #[test]
    fn old_audit_has_no_fabricated_food_observations() {
        let old: super::super::Audit =
            serde_json::from_str(r#"{"last_month":12,"years":[]}"#).unwrap();
        assert!(old.food.months.is_empty());
    }
}
