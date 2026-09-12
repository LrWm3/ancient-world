//! A scoped household-relief policy, not escrow or a change to payment timing.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Policy {
    #[default]
    Existing,
    ProtectAdministration,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub month: u32,
    pub council: u32,
    pub policy: Policy,
    pub treasury: f64,
    pub administration_forecast: f64,
    pub relief_requested: f64,
    pub relief_ceiling: f64,
    pub relief_granted: f64,
    pub relief_paid: f64,
}
impl Receipt {
    pub(crate) fn quote(
        month: u32,
        council: u32,
        policy: Policy,
        treasury: f64,
        administration_forecast: f64,
        relief_requested: f64,
        share: f64,
    ) -> Self {
        let relief_ceiling = (treasury * share).min(relief_requested);
        let relief_granted = match policy {
            Policy::Existing => relief_ceiling,
            Policy::ProtectAdministration => {
                relief_ceiling.min((treasury - administration_forecast).max(0.))
            }
        };
        Self {
            month,
            council,
            policy,
            treasury,
            administration_forecast,
            relief_requested,
            relief_ceiling,
            relief_granted,
            relief_paid: 0.,
        }
    }
}
pub(super) fn validate(
    receipts: &[Receipt],
    h: &crate::civilization::History,
) -> anyhow::Result<()> {
    let councils = h.society.as_ref().map_or(0, |s| s.councils.len());
    anyhow::ensure!(
        receipts.len() <= councils,
        "incomplete council allocation receipts"
    );
    for (id, r) in receipts.iter().enumerate() {
        anyhow::ensure!(
            r.council as usize == id && r.month <= h.month,
            "invalid council allocation boundary"
        );
        anyhow::ensure!(
            [
                r.treasury,
                r.administration_forecast,
                r.relief_requested,
                r.relief_ceiling,
                r.relief_granted,
                r.relief_paid
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.),
            "invalid council allocation amounts"
        );
        anyhow::ensure!(
            r.relief_paid <= r.relief_granted + 1e-7
                && r.relief_granted <= r.relief_ceiling + 1e-7
                && r.relief_ceiling <= r.treasury + 1e-7
                && r.relief_ceiling <= r.relief_requested + 1e-7,
            "council allocation exceeds budget"
        );
        if r.policy == Policy::ProtectAdministration {
            anyhow::ensure!(
                r.relief_granted <= (r.treasury - r.administration_forecast).max(0.) + 1e-7,
                "relief spends protected administration allowance"
            );
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scarce_cash_and_inactive_claims() {
        let quote = |p, cash, admin, need| Receipt::quote(7, 0, p, cash, admin, need, 0.5);
        assert_eq!(quote(Policy::Existing, 10., 8., 100.).relief_granted, 5.);
        assert_eq!(
            quote(Policy::ProtectAdministration, 10., 8., 100.).relief_granted,
            2.
        );
        assert_eq!(
            quote(Policy::ProtectAdministration, 10., 12., 100.).relief_granted,
            0.
        );
        assert_eq!(
            quote(Policy::ProtectAdministration, 10., 0., 100.).relief_granted,
            5.
        );
        assert_eq!(
            quote(Policy::ProtectAdministration, 10., 8., 1.).relief_granted,
            1.
        );
        assert_eq!(
            quote(Policy::ProtectAdministration, 0., 8., 100.).relief_granted,
            0.
        );
        assert_eq!(
            quote(Policy::ProtectAdministration, 10., 8., 0.).relief_granted,
            0.
        );
    }
}

#[cfg(test)]
mod integration {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn protection_trades_relief_for_actual_administration_and_resumes() {
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
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.enable_governance().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        for site in &mut h.sites {
            site.economy.finance[0] = 0.;
        }
        let forecast = h.administration_forecast();
        let society = h.society.as_mut().unwrap();
        for council in &mut society.councils {
            council.treasury = forecast[council.civilization as usize] * 1.5;
            council.distribution = None;
        }
        let e = society.household_economy.as_mut().unwrap();
        e.founding_access = None;
        e.common_share = 0.;
        e.payroll_share = 0.;
        e.dividend_share = 0.;
        e.relief_share = 1.;
        e.relief_target = 1.;
        for a in &mut e.accounts {
            a.cash = 0.;
        }
        let mut control = h.clone();
        let mut protected = h.clone();
        protected
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .council_allocation = Policy::ProtectAdministration;
        let total = |h: &crate::civilization::History| h.economy_residuals()[4];
        let opening = total(&protected);
        control.prepare_household_retail();
        protected.prepare_household_retail();
        let e = protected
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        let c = control
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        assert!(e
            .council_allocations
            .iter()
            .zip(&c.council_allocations)
            .any(|(a, b)| a.relief_paid < b.relief_paid));
        validate(&e.council_allocations, &protected).unwrap();
        assert!((total(&protected) - opening).abs() < 1e-6);
        let mut restored: crate::civilization::History =
            serde_json::from_value(serde_json::to_value(&protected).unwrap()).unwrap();
        control.governance_month();
        protected.governance_month();
        restored.governance_month();
        assert_eq!(
            serde_json::to_value(&protected).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        let funding = |h: &crate::civilization::History| {
            h.society
                .as_ref()
                .unwrap()
                .council_funding
                .administration
                .paid
        };
        assert!(funding(&protected) > funding(&control));
        assert!((total(&protected) - opening).abs() < 1e-5);
        // No governance means no protected claim, so the policy is inert.
        let mut absent = h.clone();
        absent.governance = None;
        let mut absent_control = absent.clone();
        absent
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .council_allocation = Policy::ProtectAdministration;
        absent.prepare_household_retail();
        absent_control.prepare_household_retail();
        let grants = |h: &crate::civilization::History| {
            h.society
                .as_ref()
                .unwrap()
                .household_economy
                .as_ref()
                .unwrap()
                .council_allocations
                .iter()
                .map(|r| r.relief_paid)
                .collect::<Vec<_>>()
        };
        assert_eq!(grants(&absent), grants(&absent_control));
    }
}
