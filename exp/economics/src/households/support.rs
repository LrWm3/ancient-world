//! Voluntary private-surplus offers, evaluated before contributed work. These
//! grants transfer ownership once; they are neither wages nor household claims.
use super::*;
use crate::household_governance::Policy;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Mandate {
    pub member: AgentId,
    pub resource: ResourceId,
    pub from: u32,
    pub through: u32,
    /// Withdrawal takes effect at a future monthly boundary; original terms survive.
    pub revoked_from: Option<u32>,
    pub reserve_months: u32,
    pub private_reserve: i32,
    pub household_target: i32,
    pub monthly_limit: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Receipt {
    pub household: AgentId,
    pub mandate: Mandate,
    pub protected: i32,
    pub offered: i32,
    pub accepted: i32,
    pub reason: String,
    pub baseline_income: Option<income::Forecast>,
    pub projected_income: Option<income::Forecast>,
    /// Funding comparison only; actual claims remain collectible at their boundary.
    pub payment_funding: Option<PaymentFunding>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct PaymentFunding {
    /// Funding units of the mandate resource, never summed across denominations.
    /// Land alternatives use their accepted rate; the bill remains native.
    pub due: i128,
    pub shortfall: i128,
    pub projected_shortfall: i128,
}

/// An explicit member instruction, independent of the governor's authority. A
/// caller supplies the signatory just as with the other agreement admission APIs.
pub fn authorize(
    world: &mut World,
    state: &State,
    household: AgentId,
    signed_by: AgentId,
    mandate: Mandate,
) -> Result<(), String> {
    if signed_by != mandate.member
        || parent(world, state, signed_by) != Some(household)
        || mandate.revoked_from.is_some()
        || mandate.from < state.month
        || (mandate.from == state.month && state.phase != Phase::Open)
    {
        return Err("surplus support requires a member's prospective authorization".into());
    }
    let mut candidate = world.clone();
    candidate
        .households
        .iter_mut()
        .find(|h| h.agent == household)
        .ok_or("unknown support household")?
        .support
        .push(mandate);
    crate::settlement::validate_world(&candidate, state)?;
    *world = candidate;
    Ok(())
}

/// Stop an existing mandate next month without rewriting its accepted terms.
pub fn revoke(
    world: &mut World,
    state: &State,
    household: AgentId,
    signed_by: AgentId,
    resource: ResourceId,
    authorized_from: u32,
) -> Result<(), String> {
    if parent(world, state, signed_by) != Some(household) {
        return Err("only a current member may withdraw their surplus mandate".into());
    }
    let mut candidate = world.clone();
    let mandate = candidate
        .households
        .iter_mut()
        .find(|h| h.agent == household)
        .and_then(|h| {
            h.support.iter_mut().find(|m| {
                m.member == signed_by && m.resource == resource && m.from == authorized_from
            })
        })
        .ok_or("unknown member surplus mandate")?;
    if mandate.revoked_from.is_some() || mandate.through < state.month {
        return Err("surplus mandate is already withdrawn or expired".into());
    }
    mandate.revoked_from = Some(
        state
            .month
            .checked_add(1)
            .ok_or("withdrawal date overflow")?,
    );
    crate::settlement::validate_world(&candidate, state)?;
    *world = candidate;
    Ok(())
}

fn effective_end(mandate: &Mandate) -> u32 {
    mandate.through.min(
        mandate
            .revoked_from
            .map_or(u32::MAX, |date| date.saturating_sub(1)),
    )
}

pub(super) fn validate(world: &World) -> Result<(), String> {
    for h in &world.households {
        for (i, m) in h.support.iter().enumerate() {
            if !membership::ever_member(h, m.member)
                || m.from < h.formed
                || m.through < m.from
                || m.revoked_from == Some(0)
                || !(1..=crate::need_orders::MAX_RESERVE_MONTHS).contains(&m.reserve_months)
                || m.private_reserve < 0
                || m.household_target <= 0
                || m.monthly_limit <= 0
                || !world
                    .resources
                    .iter()
                    .any(|r| r.id == m.resource && r.kind == ResourceKind::Stock)
                || world.activities.perishable.contains(&m.resource)
                || h.support[..i].iter().any(|other| {
                    other.member == m.member
                        && other.resource == m.resource
                        && m.from <= effective_end(m)
                        && other.from <= effective_end(other)
                        && other.from <= effective_end(m)
                        && m.from <= effective_end(other)
                })
            {
                return Err("invalid or overlapping voluntary household support mandate".into());
            }
        }
    }
    Ok(())
}

pub(super) fn prepare(
    world: &World,
    opening: &State,
    allocated: &State,
) -> Result<(Vec<Effect>, Vec<Receipt>), String> {
    let mut staged = allocated.clone();
    let mut effects = vec![];
    let mut receipts = vec![];
    let mut households: Vec<_> = world.households.iter().collect();
    households.sort_by_key(|h| h.agent);
    for h in households {
        // Stable policy ordering also resolves competing surplus offers. No
        // alternative may allocate resources already accepted from another offer.
        for member in crate::household_governance::ordered(h, &staged) {
            let mut mandates: Vec<_> = h
                .support
                .iter()
                .filter(|m| {
                    m.member == member
                        && m.from <= staged.month
                        && staged.month <= m.through
                        && m.revoked_from.is_none_or(|month| staged.month < month)
                })
                .collect();
            mandates.sort_by_key(|m| m.resource);
            for m in mandates {
                let mut accepted_terms = m.clone();
                // Future withdrawal is not part of this historical offer receipt.
                accepted_terms.revoked_from = None;
                let mut r = Receipt {
                    household: h.agent,
                    mandate: accepted_terms,
                    protected: 0,
                    offered: 0,
                    accepted: 0,
                    reason: "inactive membership or policy".into(),
                    baseline_income: None,
                    projected_income: None,
                    payment_funding: None,
                };
                if parent(world, &staged, member) != Some(h.agent)
                    || !matches!(
                        h.governance.policy(staged.month),
                        Policy::NeedsFirst | Policy::NeedsThenIncome
                    )
                    || !market::active(world, &staged, h.agent)
                {
                    receipts.push(r);
                    continue;
                }
                // Protect accepted obligations/entry inputs and personal needs
                // before considering the member's additional private stock floor.
                let protected =
                    crate::need_orders::protected_stock(world, &staged, member, m.reserve_months)?;
                let held = staged.balance(member, m.resource);
                let floor = protected
                    .get(&m.resource)
                    .copied()
                    .unwrap_or(0)
                    .max(i128::from(m.private_reserve))
                    .min(i128::from(held));
                let surplus = held
                    - i32::try_from(floor).map_err(|_| "household support protection overflow")?;
                r.protected = held - surplus;
                // Goods received during this reservation boundary cannot be
                // donated back or finance another outgoing reservation.
                r.offered = surplus
                    .min((opening.balance(member, m.resource) - r.protected).max(0))
                    .min(m.monthly_limit)
                    .min((m.household_target - staged.balance(h.agent, m.resource)).max(0));
                r.reason = "no authorized surplus or collective stock gap".into();
                if r.offered == 0 {
                    receipts.push(r);
                    continue;
                }
                let feasible = crate::storage::transferable(
                    world,
                    &crate::storage::usage(world, &staged.balances),
                    member,
                    h.agent,
                    m.resource,
                    r.offered,
                )?;
                if feasible == 0 {
                    r.reason = "collective storage unavailable".into();
                    receipts.push(r);
                    continue;
                }
                let proposed = transfer(member, h.agent, m.resource, feasible);
                let baseline = probe(world, &staged)?;
                let people: BTreeSet<_> = members(h, &staged).collect();
                let base_needs = needs::project(world, &staged, &baseline, &people)?;
                let base_private = needs::project(world, &staged, &baseline, &[member].into())?;
                let monetary = h.governance.policy(staged.month) == Policy::NeedsThenIncome;
                let base_income = monetary
                    .then(|| income::project(world, &staged, &baseline, h.agent))
                    .transpose()?;
                let mut trial = staged.clone();
                apply(world, &mut trial, &proposed, Backend::Reference)?;
                let plan = probe(world, &trial)?;
                let next_needs = needs::project(world, &trial, &plan, &people)?;
                let next_private = needs::project(world, &trial, &plan, &[member].into())?;
                let mut next_income = monetary
                    .then(|| income::project(world, &trial, &plan, h.agent))
                    .transpose()?;
                // Personal fulfillment cannot be sacrificed to aggregate income.
                let no_private_harm = next_private
                    .iter()
                    .zip(&base_private)
                    .all(|(after, before)| after.unmet <= before.unmet);
                let improves = next_needs < base_needs
                    || (next_needs == base_needs
                        && next_income
                            .as_ref()
                            .zip(base_income.as_ref())
                            .is_some_and(|(next, base)| income::improves(h, base, next)));
                r.reason = if no_private_harm && improves {
                    r.accepted = feasible;
                    staged = trial;
                    effects.extend(proposed);
                    "accepted useful voluntary surplus"
                } else {
                    "no protected need or market-income improvement"
                }
                .into();
                // Preserve the existing needs/income choice. When that rejects,
                // an explicit charter may accept a smaller payment-funding offer.
                // Do not capture the entire mandate merely because one unit helps.
                if r.accepted == 0 && h.governance.charter.accept_payment_support {
                    let wages = crate::employment::claims(&staged, h.agent)?;
                    let loans = crate::credit::current_dues(world, &staged, h.agent)?;
                    let land = crate::commitments::funding_dues(world, &staged, h.agent)?;
                    let due = wages
                        .get(&m.resource)
                        .copied()
                        .unwrap_or(0)
                        .checked_add(loans.get(&m.resource).copied().unwrap_or(0))
                        .and_then(|q| q.checked_add(land.get(&m.resource).copied().unwrap_or(0)))
                        .ok_or("household payment funding overflow")?;
                    let shortfall = (due - i128::from(staged.balance(h.agent, m.resource))).max(0);
                    let quantity = i128::from(feasible).min(shortfall) as i32;
                    r.payment_funding = Some(PaymentFunding {
                        due,
                        shortfall,
                        projected_shortfall: shortfall - i128::from(quantity),
                    });
                    if quantity > 0 {
                        let payment = transfer(member, h.agent, m.resource, quantity);
                        let mut funded = staged.clone();
                        apply(world, &mut funded, &payment, Backend::Reference)?;
                        let plan = probe(world, &funded)?;
                        let funded_needs = needs::project(world, &funded, &plan, &people)?;
                        let funded_private =
                            needs::project(world, &funded, &plan, &[member].into())?;
                        next_income = monetary
                            .then(|| income::project(world, &funded, &plan, h.agent))
                            .transpose()?;
                        if funded_needs <= base_needs
                            && funded_private
                                .iter()
                                .zip(&base_private)
                                .all(|(after, before)| after.unmet <= before.unmet)
                        {
                            r.accepted = quantity;
                            staged = funded;
                            effects.extend(payment);
                            r.reason = "accepted voluntary payment funding".into();
                        } else {
                            r.reason = "payment funding would worsen protected needs".into();
                        }
                    }
                }
                r.baseline_income = base_income;
                r.projected_income = next_income;
                receipts.push(r);
            }
        }
    }
    Ok((effects, receipts))
}
