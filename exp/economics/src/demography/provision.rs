//! Dated material support pressure, not a financial claim or a reservation.
use crate::model::*;
use std::collections::BTreeMap;

/// Project top-ups for known living children using their own finite stocks and
/// permitted consumption recipes. Future births are deliberately absent.
pub fn requests(
    w: &World,
    s: &State,
    parent: AgentId,
    resource: ResourceId,
    horizon: u64,
) -> BTreeMap<u64, i128> {
    let mut requests = BTreeMap::new();
    let Some(c) = &w.demography else {
        return requests;
    };
    if !c.plan_support || c.support.resource != resource || s.terminal.contains_key(&parent) {
        return requests;
    }
    for child in super::dependents(s, parent) {
        let birth = &s.demography.births[&child];
        let Some(p) = w.participants.iter().find(|p| p.agent == child) else {
            continue;
        };
        let mut stocks = crate::substitution::stocks(s, child);
        let mut preview = s.clone();
        let end = (u64::from(s.month) + horizon).min(u64::from(birth.adult_at));
        for month in u64::from(s.month)..end {
            preview.month = month as u32;
            if month > u64::from(s.month) || s.phase == Phase::Open {
                let stock = stocks.entry(resource).or_default();
                let requested = (i128::from(c.support.quantity) - *stock).max(0);
                *requests.entry(month).or_default() += requested;
                *stock += requested;
            }
            if month == u64::from(s.month) && s.phase == Phase::Close {
                continue;
            }
            for need in crate::forecast::needs::ordered(&p.needs) {
                let fulfilled = if month == u64::from(s.month) && s.phase != Phase::Open {
                    s.balance(child, need.resource)
                } else {
                    0
                };
                let recipes = crate::substitution::recipes_for(w, &preview, child, need.resource);
                crate::substitution::allocate_recipes(
                    &recipes,
                    i128::from((need.quantity - fulfilled).max(0)),
                    &mut stocks,
                    &BTreeMap::new(),
                );
            }
        }
    }
    requests.retain(|_, q| *q > 0);
    requests
}

/// Apply a hypothetical Open request, preserving the reusable donor stock floor.
/// Return unmet support; do not turn that floor into a recurring consumption cost.
pub(crate) fn debit(w: &World, stock: &mut i128, requested: i128) -> i128 {
    let floor = w
        .demography
        .as_ref()
        .map_or(0, |c| i128::from(c.parent_reserve));
    let paid = (*stock - floor).max(0).min(requested);
    *stock -= paid;
    requested - paid
}

/// Conservative voluntary-sale protection: future top-ups plus the donor floor
/// once, in addition to personal consumption. This is not a minimal dated budget,
/// debt, or escrow; retaining extra stock avoids assuming consumption timing away.
pub(crate) fn protection(
    w: &World,
    s: &State,
    parent: AgentId,
    months: u32,
) -> BTreeMap<ResourceId, i128> {
    let Some(c) = &w.demography else {
        return BTreeMap::new();
    };
    let quantity: i128 = requests(w, s, parent, c.support.resource, u64::from(months))
        .values()
        .sum();
    if quantity == 0 {
        BTreeMap::new()
    } else {
        [(c.support.resource, quantity + i128::from(c.parent_reserve))].into()
    }
}
