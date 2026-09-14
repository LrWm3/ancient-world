//! Opt-in local food solidarity: finite wallet transfers followed by ordinary retail.
use super::{apportion_relief, HouseholdEconomy, RetailPlan};
use serde::{Deserialize, Serialize};

const PROTECTED_FOOD_MONTHS: f64 = 3.;
const MONTHLY_SURPLUS_SHARE: f64 = 0.05;
const NEED_FLOOR_KG: f64 = 1e-12;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Policy {
    pub enabled: bool,
    pub month: Option<u32>,
    pub receipts: Vec<Receipt>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub site: u32,
    pub requested: f64,
    pub donor_allowance: f64,
    pub stock_backed_budget: f64,
    pub transferred: f64,
}

pub(super) fn settle(e: &mut HouseholdEconomy, plans: &[RetailPlan], food: &[f64], month: u32) {
    if !e.solidarity.enabled || e.solidarity.month == Some(month) {
        return;
    }
    e.solidarity.month = Some(month);
    e.solidarity.receipts.clear();
    for p in plans {
        if p.need <= NEED_FLOOR_KG {
            continue;
        }
        let private_share = (1. - p.free / p.need).clamp(0., 1.);
        let requested: Vec<_> = p
            .ids
            .iter()
            .zip(&p.needs)
            .map(|(&id, &need)| (need * private_share * p.price - e.accounts[id].cash).max(0.))
            .collect();
        let donors: Vec<_> = p
            .ids
            .iter()
            .zip(&p.needs)
            .map(|(&id, &need)| {
                if need <= NEED_FLOOR_KG {
                    return 0.;
                }
                (e.accounts[id].cash - PROTECTED_FOOD_MONTHS * need * p.price).max(0.)
                    * MONTHLY_SURPLUS_SHARE
            })
            .collect();
        let funded = p.free
            + p.ids
                .iter()
                .zip(&p.needs)
                .map(|(&id, &need)| (need * private_share).min(e.accounts[id].cash / p.price))
                .sum::<f64>();
        // Only opening local food supports the guarantee. Future harvests and
        // incoming cargo are not assumed available. No cross-town cash pooling.
        let stock_backed_budget = (food[p.site].min(p.need) - funded).max(0.) * p.price;
        let request = requested.iter().sum::<f64>();
        let allowance = donors.iter().sum::<f64>();
        let budget = request.min(allowance).min(stock_backed_budget);
        let mut paid = 0.;
        for (&id, amount) in p.ids.iter().zip(apportion_relief(&donors, budget)) {
            let a = &mut e.accounts[id];
            let before = a.cash;
            a.cash = (before - amount).max(0.);
            let actual = before - a.cash;
            a.solidarity_sent += actual;
            paid += actual;
        }
        for (&id, amount) in p.ids.iter().zip(apportion_relief(&requested, paid)) {
            e.accounts[id].cash += amount;
            e.accounts[id].solidarity_received += amount;
        }
        e.solidarity.receipts.push(Receipt {
            site: p.site as u32,
            requested: request,
            donor_allowance: allowance,
            stock_backed_budget,
            transferred: paid,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (HouseholdEconomy, Vec<RetailPlan>) {
        let mut e = HouseholdEconomy::new(0);
        e.solidarity.enabled = true;
        e.accounts = vec![Default::default(); 3];
        e.accounts[0].cash = 1000.;
        (
            e,
            vec![RetailPlan {
                site: 0,
                ids: vec![0, 1, 2],
                needs: vec![100.; 3],
                demand: vec![100., 0., 0.],
                need: 300.,
                free: 0.,
                price: 1.,
            }],
        )
    }
    #[test]
    fn surplus_funds_real_gaps_and_preserves_cash_and_donor_reserve() {
        let (mut e, p) = fixture();
        settle(&mut e, &p, &[300.], 1);
        assert_eq!(e.accounts[0].cash, 965.);
        assert_eq!(e.accounts[1].cash, 17.5);
        assert_eq!(e.accounts[2].cash, 17.5);
        assert_eq!(e.accounts.iter().map(|a| a.cash).sum::<f64>(), 1000.);
        assert_eq!(e.solidarity.receipts[0].transferred, 35.);
        let once = serde_json::to_value(&e).unwrap();
        settle(&mut e, &p, &[300.], 1);
        assert_eq!(once, serde_json::to_value(&e).unwrap());
        let mut restored: HouseholdEconomy = serde_json::from_value(once).unwrap();
        settle(&mut restored, &p, &[300.], 2);
        settle(&mut e, &p, &[300.], 2);
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::to_value(restored).unwrap()
        );
    }
    #[test]
    fn unavailable_food_and_no_gap_never_collect() {
        let (mut e, p) = fixture();
        settle(&mut e, &p, &[100.], 1); // All stock already covered by existing demand.
        assert_eq!(e.accounts[0].cash, 1000.);
        let (mut e, mut p) = fixture();
        p[0].free = p[0].need;
        settle(&mut e, &p, &[300.], 1);
        assert_eq!(e.accounts[0].cash, 1000.);
        let (mut e, p) = fixture();
        e.accounts[0].cash = 300.; // Exactly the protected reserve.
        settle(&mut e, &p, &[300.], 1);
        assert_eq!(e.accounts[0].cash, 300.);
    }
    #[test]
    fn households_without_residents_do_not_contribute() {
        let (mut e, mut p) = fixture();
        p[0].needs[0] = 0.;
        p[0].need = 200.;
        settle(&mut e, &p, &[300.], 1);
        assert_eq!(e.accounts[0].cash, 1000.);
        assert_eq!(e.solidarity.receipts[0].transferred, 0.);
    }
    #[test]
    fn food_ceiling_and_household_order_bound_the_same_transfer() {
        let (mut e, p) = fixture();
        let mut reordered = e.clone();
        settle(&mut e, &p, &[110.], 1);
        assert_eq!(e.solidarity.receipts[0].transferred, 10.);
        let (_, mut q) = fixture();
        q[0].ids.reverse();
        settle(&mut reordered, &q, &[110.], 1);
        assert_eq!(
            serde_json::to_value(e.accounts).unwrap(),
            serde_json::to_value(reordered.accounts).unwrap()
        );
    }
}
