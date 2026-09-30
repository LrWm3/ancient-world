//! Supplied estate bids assign an entire eligible loan at face units times its
//! explicit custody-coin quote or agreed floor. Partial/onward assignment is deferred.
use crate::{credit, finance, model::*, opportunities, recovery};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listing {
    /// Custody coins per native claim unit; coin claims retain a one-to-one quote.
    pub coins_per_unit: i32,
    pub id: u32,
    pub proceeding: u32,
    pub loan: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bid {
    pub id: u32,
    pub listing: u32,
    pub buyer: AgentId,
    pub month: u32,
    pub price: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignment {
    pub listing: u32,
    pub bid: u32,
    pub month: u32,
    pub buyer: AgentId,
    pub principal: i32,
    pub interest: i32,
    pub price: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuaranteeCoverage {
    pub terms: recovery::Guarantee,
    pub accepted_month: u32,
    /// Native claim units still covered; neither escrow nor a funding forecast.
    pub remaining_cap: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    /// Current servicing terms and security; discovery is not an underwriting promise.
    pub loan: credit::Loan,
    pub guarantees: Vec<GuaranteeCoverage>,
    pub listing: Listing,
    pub seller: AgentId,
    pub debtor: AgentId,
    pub remaining: Amount,
    pub payment_resource: ResourceId,
}

pub fn discover(world: &World, state: &State, buyer: AgentId) -> Vec<Offer> {
    if !recovery::market::eligible_buyer_for(world, state, buyer, opportunities::Action::AssetTrade)
    {
        return vec![];
    }
    let mut offers = vec![];
    for l in &world.recovery.receivable_listings {
        let Some(p) = world
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == l.proceeding)
        else {
            continue;
        };
        let Some(loan) = state.credit.loans.get(&l.loan) else {
            continue;
        };
        if loan.creditor == p.debtor
            && ![p.debtor, p.estate, loan.debtor].contains(&buyer)
            && !world
                .recovery
                .guarantees
                .iter()
                .any(|g| g.claim == recovery::GuaranteedClaim::Loan(l.loan) && g.guarantor == buyer)
            && loan.debt().unwrap_or(0) > 0
            && !state.credit.recovery.assignments.contains_key(&l.loan)
            && state
                .credit
                .recovery
                .proceedings
                .get(&p.id)
                .is_some_and(|c| c.stage == recovery::Stage::Active)
        {
            let mut guarantees: Vec<_> = world
                .recovery
                .guarantees
                .iter()
                .filter(|g| g.claim == recovery::GuaranteedClaim::Loan(l.loan))
                .filter_map(|g| {
                    let accepted_month =
                        recovery::admission::accepted_month(world, &state.credit, g)?;
                    let paid = state
                        .credit
                        .recovery
                        .paid_guarantees
                        .get(&g.id)
                        .copied()
                        .unwrap_or(0);
                    let remaining_cap = g.cap.checked_sub(paid)?;
                    (g.through >= state.month && remaining_cap > 0).then(|| GuaranteeCoverage {
                        terms: g.clone(),
                        accepted_month,
                        remaining_cap,
                    })
                })
                .collect();
            guarantees.sort_by_key(|g| g.terms.id);
            offers.push(Offer {
                loan: loan.clone(),
                guarantees,
                listing: l.clone(),
                seller: p.debtor,
                debtor: loan.debtor,
                remaining: Amount::new(loan.denomination, loan.debt().unwrap()),
                payment_resource: p.denomination,
            });
        }
    }
    offers.sort_by_key(|o| (o.listing.proceeding, o.listing.id));
    offers
}

fn accepts_price(
    world: &World,
    listing: &Listing,
    principal: i32,
    interest: i32,
    price: i32,
) -> bool {
    if let Some(minimum) = world.recovery.receivable_price_floors.get(&listing.id) {
        principal > 0 && interest == 0 && price >= *minimum
    } else {
        (i64::from(principal) + i64::from(interest)) * i64::from(listing.coins_per_unit)
            == i64::from(price)
    }
}

pub(crate) fn validate(world: &World, state: &State) -> Result<(), String> {
    if world
        .recovery
        .receivable_price_floors
        .iter()
        .any(|(id, price)| {
            *price <= 0
                || !world
                    .recovery
                    .receivable_listings
                    .iter()
                    .any(|l| l.id == *id)
        })
    {
        return Err("invalid receivable price floor".into());
    }

    let mut ids = BTreeSet::new();
    let mut loans = BTreeSet::new();
    for l in &world.recovery.receivable_listings {
        let p = world
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == l.proceeding)
            .ok_or("receivable listing without authorized estate")?;
        let a = credit::offered_loan(world, l.loan)
            .ok_or("receivable assignment requires configured loan terms")?;
        if !ids.insert(l.id)
            || !loans.insert(l.loan)
            || a.terms.creditor != p.debtor
            || (world.recovery.receivable_price_floors.contains_key(&l.id)
                && (a.terms.monthly_rate_bps != 0 || a.terms.denomination != p.denomination))
            || l.coins_per_unit <= 0
            || (a.terms.denomination == p.denomination && l.coins_per_unit != 1)
            || (a.collateral.is_some() && a.terms.denomination != p.denomination)
            || a.collateral.is_some_and(|c| {
                c.settlement != credit::CollateralSettlement::AuthorizedLiquidation
            })
            || world.recovery.guarantees.iter().any(|g| {
                (g.claim == recovery::GuaranteedClaim::Loan(l.loan) && !g.follows_assignment)
                    || g.recourse == l.loan
            })
        {
            return Err(
                "receivable assignment requires a positive native-unit quote, transferable guarantees and compatible liquidation security"
                    .into(),
            );
        }
        if let Some(loan) = state.credit.loans.get(&l.loan) {
            let expected = state
                .credit
                .recovery
                .assignments
                .get(&l.loan)
                .map_or(p.debtor, |a| a.buyer);
            if loan.creditor != expected {
                return Err("receivable owner differs from accepted assignment".into());
            }
        }
    }
    ids.clear();
    for b in &world.recovery.receivable_bids {
        let l = world
            .recovery
            .receivable_listings
            .iter()
            .find(|l| l.id == b.listing)
            .ok_or("receivable bid without listing")?;
        let p = world
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == l.proceeding)
            .unwrap();
        let loan = credit::offered_loan(world, l.loan).unwrap();
        if !ids.insert(b.id)
            || b.month < p.opening_month
            || b.price <= 0
            || !world.agents.iter().any(|a| a.id == b.buyer)
            || [p.debtor, p.estate, loan.debtor].contains(&b.buyer)
            || world.recovery.guarantees.iter().any(|g| {
                g.claim == recovery::GuaranteedClaim::Loan(l.loan) && g.guarantor == b.buyer
            })
        {
            return Err("invalid receivable bid".into());
        }
    }
    for (&loan, a) in &state.credit.recovery.assignments {
        let l = world
            .recovery
            .receivable_listings
            .iter()
            .find(|l| l.id == a.listing && l.loan == loan)
            .ok_or("assignment without listed claim")?;
        let b = world
            .recovery
            .receivable_bids
            .iter()
            .find(|b| b.id == a.bid)
            .ok_or("assignment without consented bid")?;
        if b.listing != l.id
            || b.buyer != a.buyer
            || b.month != a.month
            || b.price != a.price
            || a.month > state.month
            || (a.month == state.month
                && matches!(state.phase, Phase::Open | Phase::Due | Phase::Acquire))
            || a.principal < 0
            || a.interest < 0
            || !accepts_price(world, l, a.principal, a.interest, a.price)
            || !state.credit.loans.contains_key(&loan)
            || !state
                .credit
                .recovery
                .proceedings
                .contains_key(&l.proceeding)
        {
            return Err("invalid receivable assignment history".into());
        }
    }
    Ok(())
}

pub(crate) fn sales(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    execution: &mut finance::Execution,
) -> Result<(), String> {
    let mut bids: Vec<_> = world
        .recovery
        .receivable_bids
        .iter()
        .filter(|b| b.month == state.month)
        .collect();
    bids.sort_by_key(|b| (b.listing, std::cmp::Reverse(b.price), b.id));
    for b in bids {
        let l = world
            .recovery
            .receivable_listings
            .iter()
            .find(|l| l.id == b.listing)
            .unwrap();
        let p = world
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == l.proceeding)
            .unwrap();
        let loan = out.after.loans.get(&l.loan);
        let eligible = out
            .after
            .recovery
            .proceedings
            .get(&p.id)
            .is_some_and(|c| c.stage == recovery::Stage::Active)
            && !out.after.recovery.assignments.contains_key(&l.loan)
            && loan.is_some_and(|loan| {
                loan.creditor == p.debtor
                    && accepts_price(world, l, loan.principal, loan.interest, b.price)
            })
            && recovery::market::eligible_buyer_for(
                world,
                state,
                b.buyer,
                opportunities::Action::AssetTrade,
            )
            && recovery::active(world, &out.after, b.buyer).is_none();
        let payment = finance::Transfer {
            from: b.buyer,
            to: p.estate,
            amount: Amount::new(p.denomination, b.price),
        };
        if !eligible
            || execution
                .exchange(world, std::slice::from_ref(&payment))
                .is_err()
        {
            out.recovery
                .push(recovery::Receipt::ReceivableSaleRejected { bid: b.id });
            continue;
        }
        let loan = out.after.loans.get_mut(&l.loan).unwrap();
        let assignment = Assignment {
            listing: l.id,
            bid: b.id,
            buyer: b.buyer,
            month: state.month,
            principal: loan.principal,
            interest: loan.interest,
            price: b.price,
        };
        loan.creditor = b.buyer;
        out.after.recovery.assignments.insert(l.loan, assignment);
        let case = out.after.recovery.proceedings.get_mut(&p.id).unwrap();
        case.cash = case
            .cash
            .checked_add(b.price)
            .ok_or("receivable proceeds overflow")?;
        out.transactions.push(credit::tx(
            format!("estate receivable bid {}", b.id),
            payment.effects()?,
        ));
        out.recovery.push(recovery::Receipt::ReceivableSold {
            proceeding: p.id,
            listing: l.id,
            bid: b.id,
            loan: l.loan,
            buyer: b.buyer,
            proceeds: b.price,
        });
    }
    Ok(())
}
