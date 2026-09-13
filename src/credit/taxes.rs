//! Completed annual receipts bound forecasts; forecasts never credit cash.
use super::{underwriting::Evidence, Account, RepaymentSource};
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

const TAX_INTERVAL_MONTHS: u32 = 12;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    pub month: u32,
    pub council: u32,
    pub collected: f64,
    /// Keep requested support: unavailable funds do not erase operating needs.
    pub support_requested: f64,
    /// None in archives predating annual road evidence; wait for a new collection.
    #[serde(default)]
    pub road_requested: Option<f64>,
}
impl Observation {
    pub fn validate(&self, month: u32) -> Result<()> {
        ensure!(
            self.month <= month && self.month % TAX_INTERVAL_MONTHS == 0,
            "invalid tax observation date"
        );
        ensure!(
            [self.collected, self.support_requested]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.),
            "invalid tax observation amount"
        );
        ensure!(
            self.road_requested.is_none_or(|x| x.is_finite() && x >= 0.),
            "invalid annual road demand"
        );
        Ok(())
    }
    fn forecast(&self, month: u32, current_collectible: f64) -> Option<Evidence> {
        if month < self.month
            || month - self.month >= TAX_INTERVAL_MONTHS
            || !current_collectible.is_finite()
            || current_collectible < 0.
        {
            return None;
        }
        Some(Evidence {
            work_funding: None,
            source: RepaymentSource::AnnualTax {
                council: self.council,
                collection_month: self.month.checked_add(TAX_INTERVAL_MONTHS)?,
            },
            beneficiary: Account::Council(self.council),
            observed_month: self.month,
            expected_receipts: self.collected.min(current_collectible),
            operating_costs: self.support_requested + self.road_requested?,
            // Current-base haircut is already in receipts; do not invent a
            // calibrated probability of default from one annual observation.
            expected_loss_fraction: 0.,
        })
    }
}

impl History {
    /// Respond, immediately after annual tax and support transfers. Idempotent;
    /// old archives gain evidence only after their next real collection.
    pub(crate) fn observe_credit_taxes(&mut self) {
        let Some(society) = &self.society else {
            return;
        };
        for council in &society.councils {
            if self
                .credit
                .tax_observations
                .iter()
                .any(|o| o.month == self.month && o.council == council.civilization)
            {
                continue;
            }
            let mut observation = Observation {
                month: self.month,
                council: council.civilization,
                collected: 0.,
                support_requested: 0.,
                road_requested: Some(
                    society
                        .council_funding
                        .road_payments
                        .iter()
                        .filter(|r| r.month == self.month && r.council == council.civilization)
                        .map(|r| r.requested)
                        .sum(),
                ),
            };
            for receipt in &society.council_funding.taxes {
                if receipt.month == self.month && receipt.council == council.civilization {
                    observation.collected += f64::from(receipt.paid);
                    observation.support_requested += receipt.support_requested;
                }
            }
            self.credit.tax_observations.push(observation);
        }
    }

    /// Read current ownership, cash, policy and administrative ability. There is
    /// no assumption that production, treasury cash or previously paid taxes are
    /// new future income. Losing the tax base immediately reduces this ceiling.
    pub fn council_credit_evidence(&self, council: u32) -> Option<Evidence> {
        let government = self
            .society
            .as_ref()?
            .councils
            .iter()
            .find(|c| c.civilization == council)?;
        let observed = self
            .credit
            .tax_observations
            .iter()
            .filter(|o| o.council == council)
            .max_by_key(|o| o.month)?;
        let current = self
            .sites
            .iter()
            .filter(|s| !s.abandoned && self.controller(s.id) == council)
            .map(|s| {
                let autonomy = self
                    .governance
                    .as_ref()
                    .and_then(|g| g.administrations.get(s.id as usize))
                    .map_or(0., |a| a.autonomy);
                f64::from(crate::society::collectible_tax(
                    s.economy.finance[0],
                    government.tax_rate,
                    autonomy,
                    self.office_capacity(s.id),
                ))
            })
            .sum();
        observed.forecast(self.month, current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn taxes_forecast_receipts_not_production_and_expire_at_next_collection() {
        let observation = Observation {
            month: 12,
            council: 2,
            collected: 100.,
            support_requested: 30.,
            road_requested: Some(20.),
        };
        let normal = observation.forecast(13, 200.).unwrap();
        assert_eq!(normal.expected_receipts, 100.);
        assert_eq!(normal.operating_costs, 50.);
        let mut old = observation.clone();
        old.road_requested = None;
        assert!(old.forecast(13, 100.).is_none());
        assert_eq!(
            normal.source,
            RepaymentSource::AnnualTax {
                council: 2,
                collection_month: 24
            }
        );
        assert_eq!(
            observation.forecast(13, 20.).unwrap().expected_receipts,
            20.
        );
        assert_eq!(observation.forecast(13, 0.).unwrap().expected_receipts, 0.);
        assert!(observation.forecast(11, 100.).is_none());
        assert!(observation.forecast(24, 100.).is_none());
        assert!(observation.forecast(13, f64::NAN).is_none());
        let resumed: Observation =
            serde_json::from_str(&serde_json::to_string(&observation).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(observation.forecast(23, 80.)).unwrap(),
            serde_json::to_value(resumed.forecast(23, 80.)).unwrap()
        );
    }
    #[test]
    fn current_tax_ceiling_tracks_cash_policy_autonomy_and_staffing() {
        use crate::society::collectible_tax;
        let base = collectible_tax(1000., 0.1, 0., 1.);
        assert_eq!(base, 100.);
        assert_eq!(collectible_tax(500., 0.1, 0., 1.), base / 2.);
        assert_eq!(collectible_tax(1000., 0.05, 0., 1.), base / 2.);
        assert_eq!(collectible_tax(1000., 0.1, 0., 0.), 0.);
        assert!(collectible_tax(1000., 0.1, 1., 1.) < base);
    }
}
