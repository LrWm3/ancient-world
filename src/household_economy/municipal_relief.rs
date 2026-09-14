//! Optional municipal purchasing assistance after council and household help.
use super::{apportion_relief, HouseholdEconomy, RetailPlan};
use crate::civilization::{History, Site};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

const WORKING_CASH_PER_RESIDENT: f64 = 10.;
const MONTHLY_SURPLUS_SHARE: f64 = 0.05;
const NEED_FLOOR_KG: f64 = 1e-12;
const MONEY_TOLERANCE: f64 = 1e-7;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Policy {
    pub enabled: bool,
    pub month: Option<u32>,
    pub receipts: Vec<Receipt>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub site: u32,
    pub opening_cash: f64,
    pub protected_cash: f64,
    pub requested: f64,
    pub food_backed: f64,
    pub allowance: f64,
    pub paid: f64,
}
fn transfer(
    e: &mut HouseholdEconomy,
    p: &RetailPlan,
    cash: &mut f32,
    population: f32,
    food: f64,
) -> Receipt {
    let private = (1. - p.free / p.need.max(NEED_FLOOR_KG)).clamp(0., 1.);
    let requests: Vec<_> = p
        .ids
        .iter()
        .zip(&p.needs)
        .map(|(&id, &need)| (need * private * p.price - e.accounts[id].cash).max(0.))
        .collect();
    let funded = p.free
        + p.ids
            .iter()
            .zip(&p.needs)
            .map(|(&id, &need)| (need * private).min(e.accounts[id].cash / p.price))
            .sum::<f64>();
    let opening_cash = cash.max(0.) as f64;
    let protected_cash = population.max(0.) as f64 * WORKING_CASH_PER_RESIDENT;
    let requested = requests.iter().sum::<f64>();
    let food_backed = (food.min(p.need) - funded).max(0.) * p.price;
    let allowance = (opening_cash - protected_cash).max(0.) * MONTHLY_SURPLUS_SHARE;
    let budget = requested.min(food_backed).min(allowance);
    // Round the remaining f32 account upward, so payment never exceeds its ceiling.
    let mut remaining = (opening_cash - budget) as f32;
    if opening_cash - remaining as f64 > budget {
        remaining = f32::from_bits(remaining.to_bits() + 1).min(*cash);
    }
    let paid = (opening_cash - remaining as f64).max(0.);
    *cash = remaining;
    for (&id, grant) in p.ids.iter().zip(apportion_relief(&requests, paid)) {
        e.accounts[id].cash += grant;
        e.accounts[id].relief += grant;
    }
    Receipt {
        site: p.site as u32,
        opening_cash,
        protected_cash,
        requested,
        food_backed,
        allowance,
        paid,
    }
}
pub(super) fn settle(
    e: &mut HouseholdEconomy,
    plans: &[RetailPlan],
    sites: &mut [Site],
    month: u32,
) {
    if !e.municipal_relief.enabled || e.municipal_relief.month == Some(month) {
        return;
    }
    e.municipal_relief.month = Some(month);
    e.municipal_relief.receipts.clear();
    for p in plans {
        if p.need <= NEED_FLOOR_KG {
            continue;
        }
        let s = &mut sites[p.site];
        let r = transfer(
            e,
            p,
            &mut s.economy.finance[0],
            s.stocks.stock[0],
            s.stocks.stock[1] as f64,
        );
        e.municipal_relief.receipts.push(r);
    }
}
pub(super) fn validate(e: &HouseholdEconomy, h: &History) -> Result<()> {
    let policy = &e.municipal_relief;
    ensure!(
        policy.month.is_none_or(|m| m <= h.month),
        "municipal relief clock"
    );
    let mut previous = None;
    for r in &policy.receipts {
        ensure!(
            policy.month.is_some()
                && (r.site as usize) < h.sites.len()
                && previous.is_none_or(|id| r.site > id),
            "municipal relief site"
        );
        previous = Some(r.site);
        ensure!(
            [
                r.opening_cash,
                r.protected_cash,
                r.requested,
                r.food_backed,
                r.allowance,
                r.paid
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.),
            "municipal relief amounts"
        );
        ensure!(
            (r.allowance - (r.opening_cash - r.protected_cash).max(0.) * MONTHLY_SURPLUS_SHARE)
                .abs()
                <= MONEY_TOLERANCE
                && r.paid <= r.allowance.min(r.requested).min(r.food_backed) + MONEY_TOLERANCE,
            "municipal relief exceeds available budget"
        );
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (HouseholdEconomy, RetailPlan) {
        let mut e = HouseholdEconomy::new(0);
        e.accounts = vec![Default::default(); 2];
        (
            e,
            RetailPlan {
                site: 0,
                ids: vec![0, 1],
                needs: vec![100., 100.],
                demand: vec![0., 0.],
                need: 200.,
                free: 0.,
                price: 1.,
            },
        )
    }
    #[test]
    fn finite_transfer_respects_food_cash_and_existing_entitlements() {
        let (mut e, p) = fixture();
        let mut cash = 2000.;
        let r = transfer(&mut e, &p, &mut cash, 100., 200.);
        assert_eq!(r.paid, 50.);
        assert_eq!(cash, 1950.);
        assert_eq!(e.accounts[0].cash, 25.);
        assert_eq!(
            cash as f64 + e.accounts.iter().map(|a| a.cash).sum::<f64>(),
            2000.
        );
        assert_eq!(e.accounts.iter().map(|a| a.relief).sum::<f64>(), 50.);
        let (mut e, p) = fixture();
        e.accounts[0].cash = 100.;
        let mut cash = 2000.;
        assert_eq!(transfer(&mut e, &p, &mut cash, 100., 100.).paid, 0.);
        assert_eq!(transfer(&mut e, &p, &mut cash, 100., 110.).paid, 10.);
        let (mut e, mut p) = fixture();
        p.free = 200.;
        let mut cash = 2000.;
        assert_eq!(transfer(&mut e, &p, &mut cash, 100., 200.).paid, 0.);
        p.free = 0.;
        cash = 1000.;
        assert_eq!(transfer(&mut e, &p, &mut cash, 100., 200.).paid, 0.);
    }
    #[test]
    fn monthly_guard_disable_and_archive_boundary() {
        let (mut e, p) = fixture();
        let site: Site = serde_json::from_value(serde_json::json!({
            "id":0,"civilization":0,"island":0,"cell":0,"name":"Fixture",
            "founded":0,"abandoned":false,
            "stocks":{"stock":[100.,200.,0.,0.],"habitat":[0.,0.,0.,0.],
                "ledger":[0.,0.,0.,0.],"people":[0.,0.,0.,0.]}
        }))
        .unwrap();
        let mut sites = vec![site];
        sites[0].economy.finance[0] = 2000.;
        e.municipal_relief.enabled = true;
        let plans = vec![p];
        settle(&mut e, &plans, &mut sites, 1);
        let once = serde_json::to_value(&e).unwrap();
        settle(&mut e, &plans, &mut sites, 1);
        assert_eq!(serde_json::to_value(&e).unwrap(), once);
        assert_eq!(sites[0].economy.finance[0], 1950.);
        let mut restored: HouseholdEconomy = serde_json::from_value(once).unwrap();
        let mut copied = sites.clone();
        settle(&mut e, &plans, &mut sites, 2);
        settle(&mut restored, &plans, &mut copied, 2);
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        assert_eq!(sites[0].economy.finance[0], copied[0].economy.finance[0]);
        restored.municipal_relief.enabled = false;
        let before = serde_json::to_value(&restored).unwrap();
        settle(&mut restored, &plans, &mut copied, 3);
        assert_eq!(before, serde_json::to_value(&restored).unwrap());
        let mut old = before;
        old.as_object_mut().unwrap().remove("municipal_relief");
        assert!(
            !serde_json::from_value::<HouseholdEconomy>(old)
                .unwrap()
                .municipal_relief
                .enabled
        );
    }
    #[test]
    fn rounds_within_budget_and_preserves_household_order() {
        let (mut e, p) = fixture();
        let mut other = e.clone();
        let mut cash = 2000.13;
        let mut cash2 = cash;
        let r = transfer(&mut e, &p, &mut cash, 100., 200.);
        assert!(r.paid <= r.allowance);
        let (_, mut q) = fixture();
        q.ids.reverse();
        transfer(&mut other, &q, &mut cash2, 100., 200.);
        assert_eq!(cash, cash2);
        assert_eq!(
            serde_json::to_value(&e.accounts).unwrap(),
            serde_json::to_value(&other.accounts).unwrap()
        );
        let mut restored: HouseholdEconomy =
            serde_json::from_value(serde_json::to_value(&e).unwrap()).unwrap();
        transfer(&mut restored, &p, &mut cash2, 100., 200.);
        transfer(&mut e, &p, &mut cash, 100., 200.);
        assert_eq!(
            serde_json::to_value(e).unwrap(),
            serde_json::to_value(restored).unwrap()
        );
    }
}
