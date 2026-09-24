//! Consented guarantees and authorized recovery proceedings use the credit book.
//! Asset sales produce real escrow cash; distribution occurs at the next Due boundary.
use crate::{
    credit::{self, Loan, Status},
    finance,
    model::*,
};
use std::collections::{BTreeMap, BTreeSet};

const RECOURSE_TERM_MONTHS: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Guarantee {
    pub id: u32,
    pub loan: u32,
    pub guarantor: AgentId,
    pub cap: i32,
    pub from: u32,
    pub through: u32,
    pub delay_months: u32,
    /// Reserved loan identity for subrogated principal, collectible next month.
    pub recourse: u32,
    pub priority: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listing {
    pub asset: AssetId,
    pub minimum_price: i32,
}
/// Configured authorization/consent, not a cash-shortage heuristic. The authority
/// controls the proceeding; economic ownership remains with the debtor until sale.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProceedingTerms {
    pub id: u32,
    pub debtor: AgentId,
    pub authority: AgentId,
    /// Dedicated non-operating custody agent; never a source of new wealth.
    pub estate: AgentId,
    pub denomination: ResourceId,
    pub opening_month: u32,
    pub earliest_close: u32,
    pub assets: Vec<Listing>,
    /// Applies to loan deficiencies only. Non-loan performance claims block closure.
    pub discharge_deficiency: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bid {
    pub id: u32,
    pub proceeding: u32,
    pub buyer: AgentId,
    pub asset: AssetId,
    pub month: u32,
    pub price: i32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub guarantees: Vec<Guarantee>,
    pub proceedings: Vec<ProceedingTerms>,
    pub bids: Vec<Bid>,
    pub delivery_relief: Vec<crate::delivery_relief::Terms>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Active,
    Closed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proceeding {
    pub opened: u32,
    pub closed: Option<u32>,
    pub stage: Stage,
    pub cash: i32,
    pub sold: BTreeSet<AssetId>,
    /// Actual proceeds reserved for the asset's existing lien, capped at its debt.
    pub secured: BTreeMap<u32, i32>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Book {
    pub paid_guarantees: BTreeMap<u32, i32>,
    pub proceedings: BTreeMap<u32, Proceeding>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Receipt {
    DeliveryRelief {
        proceeding: u32,
        terms: u32,
        contract: AssetId,
        creditor: AgentId,
        applied: bool,
        rejection: Option<crate::delivery_relief::Rejection>,
        due: u32,
        written_off: i32,
        remaining: i32,
    },
    Admitted {
        proceeding: u32,
        claims: Vec<crate::recovery_claims::Claim>,
    },
    ClosureDeferred {
        proceeding: u32,
        claims: Vec<crate::recovery_claims::Claim>,
    },
    LandDistributed {
        proceeding: u32,
        agreement: u32,
        creditor: AgentId,
        requested: Amount,
        allocated: i32,
        paid: i32,
        tender: Amount,
    },
    Opened {
        proceeding: u32,
        authority: AgentId,
        debtor: AgentId,
    },
    OpeningRejected {
        proceeding: u32,
    },
    Guaranteed {
        guarantee: u32,
        loan: u32,
        requested: i32,
        paid: i32,
        recourse: u32,
    },
    Sold {
        proceeding: u32,
        asset: AssetId,
        buyer: AgentId,
        proceeds: i32,
    },
    SaleRejected {
        bid: u32,
    },
    Distributed {
        proceeding: u32,
        loan: u32,
        requested: i32,
        allocated: i32,
        paid: i32,
        secured: bool,
    },
    WrittenOff {
        proceeding: u32,
        loan: u32,
        principal: i32,
        interest: i32,
    },
    Closed {
        proceeding: u32,
        surplus: i32,
        deficiency: i32,
        discharged: bool,
    },
}

pub fn active<'a>(
    world: &'a World,
    book: &credit::Book,
    debtor: AgentId,
) -> Option<&'a ProceedingTerms> {
    world.recovery.proceedings.iter().find(|p| {
        p.debtor == debtor
            && book
                .recovery
                .proceedings
                .get(&p.id)
                .is_some_and(|c| c.stage == Stage::Active)
    })
}
fn rank(world: &World, loan: &Loan) -> u32 {
    world
        .claim_priorities
        .get(&finance::ContractId::Loan(loan.id))
        .copied()
        .unwrap_or(loan.priority)
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    crate::delivery_relief::validate_terms(world)?;
    for c in state.exchange.forwards.values() {
        crate::delivery_relief::validate_history(world, state, c)?;
    }
    let config = &world.recovery;
    let agent = |id| world.agents.iter().any(|a| a.id == id);
    let mut ids = BTreeSet::new();
    let mut recourse = BTreeSet::new();
    let source = |id| {
        world
            .lending
            .iter()
            .find(|a| a.id == id)
            .map(|a| (a.debtor, a.terms.creditor, a.terms.denomination))
            .or_else(|| {
                world.credit.as_ref().and_then(|c| {
                    c.offers
                        .iter()
                        .find(|o| o.id == id)
                        .map(|o| (c.application.buyer, o.loan.creditor, o.loan.denomination))
                })
            })
    };
    for g in &config.guarantees {
        let (debtor, creditor, _) = source(g.loan).ok_or(
            "guarantee requires an original loan; recursive guarantee chains are unsupported",
        )?;
        if world.credit.as_ref().is_some_and(|c| {
            c.offers.iter().any(|o| {
                o.id == g.loan
                    && matches!(
                        o.collateral.settlement,
                        credit::CollateralSettlement::ResaleProceeds { .. }
                    )
            })
        }) {
            return Err(
                "guarantees of pending-resale loans need a lien-subrogation adapter".into(),
            );
        }
        if !ids.insert(g.id)
            || !recourse.insert(g.recourse)
            || source(g.recourse).is_some()
            || !agent(g.guarantor)
            || [debtor, creditor].contains(&g.guarantor)
            || g.cap <= 0
            || g.from == 0
            || g.through < g.from
            || g.through == u32::MAX
        {
            return Err("invalid guarantee terms".into());
        }
    }
    let mut debtors = BTreeSet::new();
    let mut estates = BTreeSet::new();
    ids.clear();
    for p in &config.proceedings {
        if !ids.insert(p.id)
            || !debtors.insert(p.debtor)
            || !estates.insert(p.estate)
            || !agent(p.debtor)
            || !agent(p.authority)
            || !agent(p.estate)
            || p.debtor == p.estate
            || p.authority == p.estate
            || world
                .transaction_policy
                .as_ref()
                .is_some_and(|policy| policy.authority != p.authority)
            || p.opening_month == 0
            || p.earliest_close < p.opening_month
            || !world
                .resources
                .iter()
                .any(|r| r.id == p.denomination && r.kind == ResourceKind::Stock)
            || world
                .storage
                .weights
                .get(&p.denomination)
                .copied()
                .unwrap_or(0)
                != 0
        {
            return Err("invalid authorized proceeding".into());
        }
        let mut assets = BTreeSet::new();
        if p.assets.iter().any(|a| {
            a.minimum_price <= 0
                || !assets.insert(a.asset)
                || !world.assets.iter().any(|x| x.id == a.asset)
        }) {
            return Err("invalid liquidation inventory".into());
        }
        if world
            .agreements
            .iter()
            .chain(&world.access_offers)
            .any(|a| {
                a.debtor == p.debtor
                    && world
                        .activities
                        .coin_payments
                        .get(&a.id)
                        .is_some_and(|t| t.resource != p.denomination)
            })
        {
            return Err("estate land tender must match its cash denomination".into());
        }
        // Custody agents cannot participate in other configured economic arrangements.
        if world.participants.iter().any(|p0| p0.agent == p.estate)
            || world.assets.iter().any(|a| a.owner == p.estate)
            || world
                .lending
                .iter()
                .any(|a| a.debtor == p.estate || a.terms.creditor == p.estate)
            || config.guarantees.iter().any(|g| g.guarantor == p.estate)
            || world
                .agreements
                .iter()
                .chain(&world.access_offers)
                .any(|a| a.debtor == p.estate || a.creditor == p.estate)
            || world.market.as_ref().is_some_and(|m| {
                m.tools.iter().any(|r| {
                    r.buyer == p.estate
                        || r.provider == p.estate
                        || !state.exchange.contracts.values().any(|d| {
                            d.buyer == r.buyer
                                && d.provider == r.provider
                                && state
                                    .equipment
                                    .get(&d.asset)
                                    .is_some_and(|a| a.kind == r.kind)
                        })
                }) || !m.reserves.is_empty()
                    || m.plots.is_some()
                    || m.cash.as_ref().is_some_and(|c| c.lender == p.estate)
            })
            || state
                .exchange
                .forwards
                .values()
                .any(|c| c.debtor == p.estate || c.creditor == p.estate)
            || world.negotiation.is_some()
            || world.credit.is_some()
        {
            return Err("estate custody requires direct-loan composition without active custody-agent trading".into());
        }
        let cash = state
            .credit
            .recovery
            .proceedings
            .get(&p.id)
            .map_or(0, |c| c.cash);
        if state.balance(p.estate, p.denomination) != cash {
            return Err("estate cash does not reconcile".into());
        }
    }
    if estates.iter().any(|e| debtors.contains(e)) {
        return Err("estate cannot itself enter a proceeding".into());
    }
    ids.clear();
    for b in &config.bids {
        let p = config
            .proceedings
            .iter()
            .find(|p| p.id == b.proceeding)
            .ok_or("unknown liquidation case")?;
        if !ids.insert(b.id)
            || !agent(b.buyer)
            || estates.contains(&b.buyer)
            || b.buyer == p.debtor
            || b.price <= 0
            || b.month < p.opening_month
            || !p.assets.iter().any(|a| a.asset == b.asset)
        {
            return Err("invalid liquidation bid".into());
        }
    }
    for loan in state.credit.loans.values() {
        if loan.status == Status::Stayed && active(world, &state.credit, loan.debtor).is_none() {
            return Err("stayed loan without active proceeding".into());
        }
        if let Some(g) = config.guarantees.iter().find(|g| g.recourse == loan.id) {
            let (debtor, _, denomination) = source(g.loan).unwrap();
            if loan.debtor != debtor
                || loan.creditor != g.guarantor
                || loan.denomination != denomination
                || loan.original_principal
                    != state
                        .credit
                        .recovery
                        .paid_guarantees
                        .get(&g.id)
                        .copied()
                        .unwrap_or(0)
                || loan.collateral.is_some()
                || loan.monthly_rate_bps != 0
            {
                return Err("invalid subrogated loan".into());
            }
        }
    }
    for (&id, &paid) in &state.credit.recovery.paid_guarantees {
        let g = config
            .guarantees
            .iter()
            .find(|g| g.id == id)
            .ok_or("unknown guarantee receipt")?;
        if paid < 0 || paid > g.cap {
            return Err("invalid guarantee receipt".into());
        }
        if let Some(loan) = state.credit.loans.get(&g.recourse) {
            if loan.original_principal != paid || loan.creditor != g.guarantor {
                return Err("recourse does not reconcile".into());
            }
        } else if paid != 0 {
            return Err("guarantee payment without recourse".into());
        }
    }
    for (&id, case) in &state.credit.recovery.proceedings {
        let p = config
            .proceedings
            .iter()
            .find(|p| p.id == id)
            .ok_or("unknown proceeding receipt")?;
        if case.opened != p.opening_month
            || case.opened > state.month
            || case.cash < 0
            || (case.stage == Stage::Closed) != case.closed.is_some()
            || case.closed.is_some_and(|closed| {
                closed < p.earliest_close
                    || closed < case.opened
                    || closed > state.month
                    || crate::recovery_claims::outstanding(world, state, p.debtor)
                        .iter()
                        .any(|claim| match claim.contract {
                            finance::ContractId::Land(_) => claim.due <= closed,
                            finance::ContractId::Forward(id) => {
                                state.exchange.forwards[&id].issued <= closed
                            }
                            finance::ContractId::Loan(_) => false,
                        })
            })
            || case.secured.values().any(|v| *v < 0)
            || case.secured.keys().any(|id| {
                state.credit.loans.get(id).is_none_or(|loan| {
                    loan.debtor != p.debtor
                        || loan.denomination != p.denomination
                        || loan
                            .collateral
                            .as_ref()
                            .is_none_or(|c| !case.sold.contains(&c.asset))
                })
            })
            || (case.stage == Stage::Closed
                && (case.cash != 0 || p.assets.iter().any(|a| !case.sold.contains(&a.asset))))
            || case.secured.values().map(|v| i64::from(*v)).sum::<i64>() > i64::from(case.cash)
            || case
                .sold
                .iter()
                .any(|id| !p.assets.iter().any(|a| a.asset == *id))
        {
            return Err("invalid proceeding receipt".into());
        }
    }
    Ok(())
}

/// Open only on explicit authorization plus already-observed arrears. No retroactive
/// default inference from a payment that has not happened yet this month.
pub(crate) fn open(world: &World, state: &State, out: &mut credit::Boundary) -> Result<(), String> {
    let mut terms: Vec<_> = world
        .recovery
        .proceedings
        .iter()
        .filter(|p| p.opening_month == state.month)
        .collect();
    terms.sort_by_key(|p| p.id);
    for p in terms {
        if out.after.recovery.proceedings.contains_key(&p.id) {
            continue;
        }
        let loans: Vec<_> = out
            .after
            .loans
            .values()
            .filter(|l| l.debtor == p.debtor && l.debt().unwrap_or(0) > 0)
            .collect();
        let nonloans = crate::recovery_claims::outstanding(world, state, p.debtor);
        let valid = (loans
            .iter()
            .any(|l| l.first_unpaid.is_some_and(|m| m < state.month))
            || nonloans.iter().any(|c| c.due < state.month))
            && loans.iter().all(|l| {
                l.denomination == p.denomination
                    && l.status != Status::PendingSale
                    && l.collateral
                        .as_ref()
                        .is_none_or(|c| !c.pledged || p.assets.iter().any(|a| a.asset == c.asset))
            })
            && p.assets
                .iter()
                .all(|a| credit::owner(world, state, a.asset) == Some(p.debtor));
        if !valid {
            out.recovery
                .push(Receipt::OpeningRejected { proceeding: p.id });
            continue;
        }
        out.after.recovery.proceedings.insert(
            p.id,
            Proceeding {
                opened: state.month,
                closed: None,
                stage: Stage::Active,
                cash: 0,
                sold: BTreeSet::new(),
                secured: BTreeMap::new(),
            },
        );
        out.recovery.push(Receipt::Opened {
            proceeding: p.id,
            authority: p.authority,
            debtor: p.debtor,
        });
        out.recovery.push(Receipt::Admitted {
            proceeding: p.id,
            claims: nonloans,
        });
    }
    // Interest freezes and the full claim is eligible for estate distribution.
    // Ordinary servicing is stayed; the prior accrual marker still advances once.
    let debtors: BTreeSet<_> = out
        .after
        .loans
        .values()
        .filter(|l| active(world, &out.after, l.debtor).is_some())
        .map(|l| l.debtor)
        .collect();
    for l in out
        .after
        .loans
        .values_mut()
        .filter(|l| debtors.contains(&l.debtor) && l.debt().unwrap_or(0) > 0)
    {
        l.last_accrued = state.month;
        l.status = Status::Stayed;
    }
    Ok(())
}

/// Observed contingent call, shared by inspection and actual servicing. It does
/// not accrue interest, promise funding, or create a second borrower liability.
pub(crate) fn guarantee_claim(
    world: &World,
    book: &credit::Book,
    month: u32,
    g: &Guarantee,
) -> Result<Option<finance::Obligation>, String> {
    let Some(loan) = book.loans.get(&g.loan) else {
        return Ok(None);
    };
    if month < g.from
        || month > g.through
        || active(world, book, g.guarantor).is_some()
        || loan
            .first_unpaid
            .is_none_or(|m| month.saturating_sub(m) < g.delay_months)
    {
        return Ok(None);
    }
    let covered = if active(world, book, loan.debtor).is_some() {
        loan.debt()?
    } else {
        loan.due(month)?
    };
    let requested = covered.min(
        g.cap
            - book
                .recovery
                .paid_guarantees
                .get(&g.id)
                .copied()
                .unwrap_or(0),
    );
    if requested <= 0 {
        return Ok(None);
    }
    Ok(Some(finance::Obligation {
        transfer: finance::Transfer {
            from: g.guarantor,
            to: loan.creditor,
            amount: Amount::new(loan.denomination, requested),
        },
        settled: 0,
        condition: finance::Condition::OnOrAfterMonth(month),
        failure: finance::FailureRule::CarryArrears,
    }))
}

pub(crate) fn guarantees(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    execution: &mut finance::Execution,
) -> Result<(), String> {
    let protected = crate::commitments::protected_stock(world, state)?;
    let mut terms: Vec<_> = world
        .recovery
        .guarantees
        .iter()
        .filter(|g| g.from <= state.month && state.month <= g.through)
        .collect();
    terms.sort_by_key(|g| (g.priority, g.id));
    for g in terms {
        let Some(claim) = guarantee_claim(world, &out.after, state.month, g)? else {
            continue;
        };
        let requested = claim.outstanding();
        let mut loan = out.after.loans[&g.loan].clone();
        let payment = execution.pay_protected(
            world,
            state.month,
            &claim,
            protected
                .get(&(g.guarantor, loan.denomination))
                .copied()
                .unwrap_or(0),
        )?;
        let paid = payment.paid;
        if paid > 0 {
            out.transactions.push(credit::tx(
                format!("guarantee {} pays loan {}", g.id, loan.id),
                payment.effects,
            ));
            loan.apply_payment(paid);
            if loan.due(state.month)? == 0 {
                loan.first_unpaid = None;
            }
            *out.after.recovery.paid_guarantees.entry(g.id).or_default() += paid;
            let stayed = active(world, &out.after, loan.debtor).is_some();
            let recourse = out.after.loans.entry(g.recourse).or_insert_with(|| Loan {
                id: g.recourse,
                creditor: g.guarantor,
                debtor: loan.debtor,
                denomination: loan.denomination,
                original_principal: 0,
                principal: 0,
                interest: 0,
                interest_remainder: 0,
                monthly_rate_bps: 0,
                opened: state.month,
                last_accrued: state.month,
                term_months: RECOURSE_TERM_MONTHS,
                grace_months: 0,
                first_unpaid: None,
                status: Status::Active,
                collateral: None,
                priority: g.priority,
            });
            recourse.original_principal = recourse
                .original_principal
                .checked_add(paid)
                .ok_or("recourse overflow")?;
            recourse.principal = recourse
                .principal
                .checked_add(paid)
                .ok_or("recourse overflow")?;
            recourse.status = if stayed {
                Status::Stayed
            } else {
                Status::Active
            };
            recourse.last_accrued = state.month;
            out.after.loans.insert(loan.id, loan);
        }
        out.recovery.push(Receipt::Guaranteed {
            guarantee: g.id,
            loan: g.loan,
            requested,
            paid,
            recourse: g.recourse,
        });
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
        .bids
        .iter()
        .filter(|b| b.month == state.month)
        .collect();
    // Highest funded price first per asset, then stable identity; an unfunded bid
    // leaves the asset available to the next valid bidder.
    bids.sort_by_key(|b| (b.proceeding, b.asset, std::cmp::Reverse(b.price), b.id));
    for b in bids {
        let p = world
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == b.proceeding)
            .unwrap();
        let listed = p.assets.iter().find(|a| a.asset == b.asset).unwrap();
        let valid = credit::owner(world, state, b.asset) == Some(p.debtor)
            && out
                .after
                .recovery
                .proceedings
                .get(&p.id)
                .is_some_and(|c| c.stage == Stage::Active && !c.sold.contains(&b.asset))
            && !state.terminal.contains_key(&b.buyer)
            && active(world, &out.after, b.buyer).is_none()
            && b.price >= listed.minimum_price
            && crate::opportunities::permits(
                world,
                state,
                b.buyer,
                crate::opportunities::Action::AssetTrade,
            );
        let accepted = crate::asset_exchange::AcceptedSale {
            sale: credit::Sale {
                asset: b.asset,
                seller: p.debtor,
                price: Amount::new(p.denomination, b.price),
            },
            buyer: b.buyer,
            payees: vec![(p.estate, b.price)],
        };
        if !valid || crate::asset_exchange::settle(world, state, out, execution, &accepted).is_err()
        {
            out.recovery.push(Receipt::SaleRejected { bid: b.id });
            continue;
        }
        let case = out.after.recovery.proceedings.get_mut(&p.id).unwrap();
        case.cash = case
            .cash
            .checked_add(b.price)
            .ok_or("estate proceeds overflow")?;
        case.sold.insert(b.asset);
        for l in out
            .after
            .loans
            .values_mut()
            .filter(|l| l.debtor == p.debtor)
        {
            if l.collateral
                .as_ref()
                .is_some_and(|c| c.pledged && c.asset == b.asset)
            {
                let amount = b.price.min(l.debt()?);
                case.secured.insert(l.id, amount);
                l.collateral.as_mut().unwrap().pledged = false;
                l.status = Status::Stayed;
            }
        }
        out.recovery.push(Receipt::Sold {
            proceeding: p.id,
            asset: b.asset,
            buyer: b.buyer,
            proceeds: b.price,
        });
    }
    Ok(())
}

pub(crate) fn distribute(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    execution: &mut finance::Execution,
) -> Result<(), String> {
    let mut terms: Vec<_> = world
        .recovery
        .proceedings
        .iter()
        .filter(|p| active(world, &out.after, p.debtor).is_some())
        .collect();
    terms.sort_by_key(|p| p.id);
    for p in terms {
        let mut case = out.after.recovery.proceedings[&p.id].clone();
        // Collect only unprotected opening coins. Newly deposited funds become
        // distributable next month, like sale proceeds received at Acquire.
        let (land_requests, _) = crate::recovery_claims::cash_requests(world, state, out, p)?;
        let land_cash = land_requests.iter().try_fold(0_i32, |sum, r| {
            sum.checked_add(r.claim.outstanding())
                .ok_or("estate cash demand overflow")
        })?;
        let debt = out
            .after
            .loans
            .values()
            .filter(|l| l.debtor == p.debtor)
            .try_fold(land_cash, |sum, l| {
                sum.checked_add(l.debt()?)
                    .ok_or("estate debt overflow".to_string())
            })?;
        if debt > case.cash {
            let protected = crate::commitments::protected_stock(world, state)?;
            let claim = finance::Obligation {
                transfer: finance::Transfer {
                    from: p.debtor,
                    to: p.estate,
                    amount: Amount::new(p.denomination, debt - case.cash),
                },
                settled: 0,
                condition: finance::Condition::OnOrAfterMonth(state.month),
                failure: finance::FailureRule::CarryArrears,
            };
            let payment = execution.pay_protected(
                world,
                state.month,
                &claim,
                protected
                    .get(&(p.debtor, p.denomination))
                    .copied()
                    .unwrap_or(0),
            )?;
            if payment.paid > 0 {
                case.cash = case
                    .cash
                    .checked_add(payment.paid)
                    .ok_or("estate funding overflow")?;
                out.transactions.push(credit::tx(
                    format!("estate {} cash custody", p.id),
                    payment.effects,
                ));
            }
        }

        // The lien has first call only on the actual proceeds of its own asset.
        let mut secured: Vec<_> = case.secured.keys().copied().collect();
        secured.sort_by_key(|id| {
            (
                out.after.loans[id]
                    .collateral
                    .as_ref()
                    .map_or(0, |c| c.priority),
                *id,
            )
        });
        for id in secured {
            let loan = out
                .after
                .loans
                .get_mut(&id)
                .ok_or("estate lien without loan")?;
            let requested = case.secured[&id].min(loan.debt()?);
            let claim = estate_claim(p, loan, requested, state.month);
            let payment = execution.pay(world, state.month, true, &claim)?;
            if payment.paid > 0 {
                loan.apply_payment(payment.paid);
                case.cash -= payment.paid;
                out.transactions.push(credit::tx(
                    format!("estate {} secured distribution", p.id),
                    payment.effects,
                ));
            }
            out.recovery.push(Receipt::Distributed {
                proceeding: p.id,
                loan: id,
                requested,
                allocated: payment.paid,
                paid: payment.paid,
                secured: true,
            });
            case.secured.insert(id, (requested - payment.paid).max(0));
        }
        let mut requests: Vec<_> = out
            .after
            .loans
            .values()
            .filter(|l| l.debtor == p.debtor && l.opened < state.month && l.debt().unwrap_or(0) > 0)
            .map(|l| {
                Ok(finance::CollectionRequest {
                    contract: finance::ContractId::Loan(l.id),
                    rank: rank(world, l),
                    claim: estate_claim(p, l, l.debt()?, state.month),
                })
            })
            .collect::<Result<_, String>>()?;
        let remaining_lien: i32 = case.secured.values().sum();
        let protected = BTreeMap::from([((p.estate, p.denomination), remaining_lien)]);
        let (land_requests, lots) = crate::recovery_claims::cash_requests(world, state, out, p)?;
        requests.extend(land_requests);
        let grants = finance::proportional_lots(
            world,
            state.month,
            execution,
            &protected,
            &requests,
            &lots,
        )?;
        for request in requests {
            let finance::ContractId::Loan(id) = request.contract else {
                case.cash -= crate::recovery_claims::pay_land(
                    world,
                    state,
                    out,
                    p,
                    &request,
                    (grants[&request.contract], lots[&request.contract]),
                    execution,
                )?;
                continue;
            };
            let opening = execution
                .available
                .get(&(p.estate, p.denomination))
                .copied()
                .unwrap_or(0);
            let payment = execution.pay_protected(
                world,
                state.month,
                &request.claim,
                opening - grants[&request.contract],
            )?;
            if payment.paid > 0 {
                out.after
                    .loans
                    .get_mut(&id)
                    .unwrap()
                    .apply_payment(payment.paid);
                case.cash -= payment.paid;
                out.transactions.push(credit::tx(
                    format!("estate {} general distribution", p.id),
                    payment.effects,
                ));
            }
            out.recovery.push(Receipt::Distributed {
                proceeding: p.id,
                loan: id,
                requested: request.claim.outstanding(),
                allocated: grants[&request.contract],
                paid: payment.paid,
                secured: false,
            });
        }
        if state.month >= p.earliest_close
            && !out.after.loans.values().any(|l| {
                l.debtor == p.debtor && l.opened == state.month && l.debt().unwrap_or(0) > 0
            })
            && p.assets.iter().all(|a| case.sold.contains(&a.asset))
            && case.secured.values().all(|v| *v == 0)
        {
            let nonloans = crate::recovery_claims::outstanding(
                world,
                &crate::recovery_claims::current(state, out),
                p.debtor,
            );
            if !nonloans.is_empty() {
                out.recovery.push(Receipt::ClosureDeferred {
                    proceeding: p.id,
                    claims: nonloans,
                });
                out.after.recovery.proceedings.insert(p.id, case);
                continue;
            }
            let deficiency = out
                .after
                .loans
                .values()
                .filter(|l| l.debtor == p.debtor)
                .try_fold(0_i32, |sum, l| {
                    sum.checked_add(l.debt()?)
                        .ok_or("estate deficiency overflow".to_string())
                })?;
            // A storage-blocked creditor must not be discharged while cash remains.
            if case.cash == 0 || deficiency == 0 {
                let surplus = case.cash;
                if surplus > 0 {
                    let effects = execution.exchange(
                        world,
                        &[finance::Transfer {
                            from: p.estate,
                            to: p.debtor,
                            amount: Amount::new(p.denomination, surplus),
                        }],
                    )?;
                    out.transactions
                        .push(credit::tx(format!("estate {} surplus", p.id), effects));
                    case.cash = 0;
                }
                for l in out
                    .after
                    .loans
                    .values_mut()
                    .filter(|l| l.debtor == p.debtor && l.debt().unwrap_or(0) > 0)
                {
                    if let Some(c) = &mut l.collateral {
                        c.pledged = false;
                    }
                    l.status = Status::Enforced;
                    if p.discharge_deficiency {
                        let debt = l.debt()?;
                        out.recovery.push(Receipt::WrittenOff {
                            proceeding: p.id,
                            loan: l.id,
                            principal: l.principal,
                            interest: l.interest,
                        });
                        l.apply_payment(debt);
                        l.status = Status::Discharged;
                    }
                }
                case.stage = Stage::Closed;
                case.closed = Some(state.month);
                out.recovery.push(Receipt::Closed {
                    proceeding: p.id,
                    surplus,
                    deficiency,
                    discharged: p.discharge_deficiency,
                });
            }
        }
        out.after.recovery.proceedings.insert(p.id, case);
    }
    Ok(())
}
fn estate_claim(
    p: &ProceedingTerms,
    loan: &Loan,
    quantity: i32,
    month: u32,
) -> finance::Obligation {
    finance::Obligation {
        transfer: finance::Transfer {
            from: p.estate,
            to: loan.creditor,
            amount: Amount::new(p.denomination, quantity),
        },
        settled: 0,
        condition: finance::Condition::OnOrAfterMonth(month),
        failure: finance::FailureRule::CarryArrears,
    }
}
