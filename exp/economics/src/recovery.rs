//! Consented guarantees and authorized recovery proceedings use the credit book.
//! Asset sales produce real escrow cash; distribution occurs at the next Due boundary.
use crate::{
    credit::{self, Loan, Status},
    finance,
    model::*,
};
use std::collections::{BTreeMap, BTreeSet};

const RECOURSE_TERM_MONTHS: u32 = 1;
pub mod admission;
pub mod inventory;
pub mod market;
pub mod receivables;
mod subrogation;
mod tender;
pub use subrogation::RecourseSecurity;
pub use tender::GuaranteeTender;

/// Identifies the authoritative obligation covered by accepted contingent terms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuaranteedClaim {
    Loan(u32),
    Forward(u32),
    Wages { agreement: u32, earned_month: u32 },
    Land { agreement: u32, due: u32 },
}
impl GuaranteedClaim {
    pub fn contract(self) -> finance::ContractId {
        match self {
            Self::Loan(id) => finance::ContractId::Loan(id),
            Self::Forward(id) => finance::ContractId::Forward(id),
            Self::Wages { agreement, .. } => finance::ContractId::Wages(agreement),
            Self::Land { agreement, .. } => finance::ContractId::Land(agreement),
        }
    }
    /// Current ownership for performance/inspection; origination terms stay immutable.
    pub(crate) fn current_parties(
        self,
        world: &World,
        book: &credit::Book,
    ) -> Option<(AgentId, AgentId, ResourceId)> {
        if let Self::Loan(id) = self
            && let Some(loan) = book.loans.get(&id)
        {
            return Some((loan.debtor, loan.creditor, loan.denomination));
        }
        self.parties(world)
    }
    pub(crate) fn parties(self, world: &World) -> Option<(AgentId, AgentId, ResourceId)> {
        match self {
            Self::Loan(id) => world
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
                }),
            Self::Forward(id) => world
                .prepaid_deliveries
                .iter()
                .find(|t| t.id == id)
                .map(|t| (t.seller, t.buyer, t.goods.resource)),
            Self::Wages {
                agreement,
                earned_month,
            } => world
                .employment
                .iter()
                .find(|t| {
                    t.id == agreement
                        && (t.from..=t.through).contains(&earned_month)
                        && earned_month < u32::MAX
                })
                .map(|t| (t.employer, t.worker, t.wage_per_unit.resource)),
            Self::Land { agreement, due } => world
                .agreements
                .iter()
                .chain(&world.access_offers)
                .find(|a| {
                    a.id == agreement
                        && due > a.activated
                        && (due - a.activated).is_multiple_of(crate::commitments::MONTHS_PER_YEAR)
                        && world
                            .rights
                            .iter()
                            .any(|r| r.id == a.right && due <= r.through)
                })
                .filter(|a| !world.open_access_offers.contains(&a.id))
                .map(|a| (a.debtor, a.creditor, a.payment.resource)),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Guarantee {
    /// Accepted consent for its benefit to follow a whole-loan assignment.
    pub follows_assignment: bool,
    pub tender: GuaranteeTender,
    pub security: RecourseSecurity,
    pub id: u32,
    pub claim: GuaranteedClaim,
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
    /// Non-operating custodian; multiple estates retain separate beneficial balances.
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
    /// These catalog entries are offers until an explicit dated application succeeds.
    pub posted_guarantees: BTreeSet<u32>,
    pub guarantee_applications: Vec<admission::Application>,
    /// Shares the guarantor's remaining opening resources; ordinary claims still precede calls.
    pub guarantee_policy: finance::CollectionPolicy,
    pub proceedings: Vec<ProceedingTerms>,
    pub bids: Vec<Bid>,
    pub inventory_listings: Vec<inventory::Listing>,
    pub inventory_bids: Vec<inventory::Bid>,
    pub receivable_listings: Vec<receivables::Listing>,
    pub receivable_bids: Vec<receivables::Bid>,
    pub delivery_relief: Vec<crate::delivery_relief::Terms>,
    pub claim_relief: Vec<crate::claim_relief::Terms>,
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
    pub sold_inventory: BTreeSet<u32>,
    /// Actual proceeds reserved for the asset's existing lien, capped at its debt.
    pub secured: BTreeMap<u32, i32>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Book {
    pub assignments: BTreeMap<u32, receivables::Assignment>,
    pub accepted_guarantees: BTreeMap<u32, u32>,
    pub paid_guarantees: BTreeMap<u32, i32>,
    /// Actual advances by guarantee and month; additions become collectible next month.
    pub guarantee_advances: BTreeMap<(u32, u32), i32>,
    pub proceedings: BTreeMap<u32, Proceeding>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Receipt {
    ReceivableSold {
        proceeding: u32,
        listing: u32,
        bid: u32,
        loan: u32,
        buyer: AgentId,
        proceeds: i32,
    },
    ReceivableSaleRejected {
        bid: u32,
    },
    GuaranteeAdmission {
        guarantee: u32,
        rejection: Option<admission::Rejection>,
    },
    ClaimRelief {
        proceeding: u32,
        terms: u32,
        contract: finance::ContractId,
        creditor: AgentId,
        rejection: Option<crate::claim_relief::Rejection>,
        written_off: Option<Amount>,
    },
    WagesDistributed {
        proceeding: u32,
        agreement: u32,
        earned_month: u32,
        creditor: AgentId,
        requested: i32,
        allocated: i32,
        paid: i32,
    },
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
    AssetsPending {
        proceeding: u32,
        receivables: Vec<crate::recovery_claims::Receivable>,
        uncollected_cash: i32,
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
        claim: GuaranteedClaim,
        requested: i32,
        allocated: Option<i32>,
        paid: i32,
        tender: Amount,
        recourse: u32,
    },
    Sold {
        proceeding: u32,
        asset: AssetId,
        buyer: AgentId,
        proceeds: i32,
    },
    InventorySold {
        proceeding: u32,
        listing: u32,
        bid: u32,
        buyer: AgentId,
        proceeds: i32,
    },
    InventorySaleRejected {
        bid: u32,
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

/// Custody of one denomination does not convert an unsecured native obligation.
/// Its interest remains frozen, but actual native performance may cure it at Due.
pub(crate) fn native_performance(world: &World, book: &credit::Book, loan: &Loan) -> bool {
    loan.collateral.is_none()
        && active(world, book, loan.debtor).is_some_and(|p| loan.denomination != p.denomination)
}

fn native_loans(book: &credit::Book, p: &ProceedingTerms) -> Vec<crate::recovery_claims::Claim> {
    book.loans
        .values()
        .filter(|l| l.debtor == p.debtor && l.denomination != p.denomination)
        .filter_map(|l| {
            let remaining = l.debt().ok()?;
            (remaining > 0).then_some(crate::recovery_claims::Claim {
                contract: finance::ContractId::Loan(l.id),
                creditor: l.creditor,
                due: l.opened.saturating_add(1),
                remaining: Amount::new(l.denomination, remaining),
            })
        })
        .collect()
}
fn rank(world: &World, loan: &Loan) -> u32 {
    world
        .claim_priorities
        .get(&finance::ContractId::Loan(loan.id))
        .copied()
        .unwrap_or(loan.priority)
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    inventory::validate(world, state)?;
    receivables::validate(world, state)?;
    admission::validate(world, state)?;
    crate::delivery_relief::validate_terms(world)?;
    crate::claim_relief::validate_terms(world)?;
    for c in state.exchange.forwards.values() {
        crate::delivery_relief::validate_history(world, state, c)?;
    }
    let config = &world.recovery;
    let agent = |id| world.agents.iter().any(|a| a.id == id);
    let mut ids = BTreeSet::new();
    let mut recourse = BTreeSet::new();
    let source = |id| GuaranteedClaim::Loan(id).parties(world);
    for g in &config.guarantees {
        let (debtor, creditor, _) = g.claim.parties(world).ok_or(
            "guarantee requires original accepted terms; recursive guarantee chains are unsupported",
        )?;
        if g.follows_assignment && !matches!(g.claim, GuaranteedClaim::Loan(_)) {
            return Err("transferable guarantee requires a loan claim".into());
        }
        subrogation::terms(world, g)?;
        tender::terms(world, g)?;
        if g.security == RecourseSecurity::Unsecured
            && (world.credit.as_ref().is_some_and(|c| {
                c.offers.iter().any(|o| {
                    g.claim == GuaranteedClaim::Loan(o.id)
                        && matches!(
                            o.collateral.settlement,
                            credit::CollateralSettlement::ResaleProceeds { .. }
                                | credit::CollateralSettlement::AuthorizedLiquidation
                        )
                })
            }) || world.lending.iter().any(|a| {
                g.claim == GuaranteedClaim::Loan(a.id)
                    && a.collateral.as_ref().is_some_and(|c| {
                        c.settlement == credit::CollateralSettlement::AuthorizedLiquidation
                    })
            }))
        {
            return Err(
                "guarantees of shared-liquidation or pending-resale loans need a lien-subrogation adapter".into(),
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
    let mut custody = BTreeMap::<Account, i64>::new();
    ids.clear();
    for p in &config.proceedings {
        estates.insert(p.estate);
        if !ids.insert(p.id)
            || !debtors.insert(p.debtor)
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
                || !(world.assets.iter().any(|x| x.id == a.asset)
                    || state.equipment.contains_key(&a.asset)
                    || (state.retired_equipment.contains_key(&a.asset)
                        && state
                            .credit
                            .recovery
                            .proceedings
                            .get(&p.id)
                            .is_some_and(|c| c.sold.contains(&a.asset))))
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
        if world.lending.iter().any(|a| {
            a.debtor == p.debtor
                && a.terms.denomination != p.denomination
                && a.collateral
                    .as_ref()
                    .is_some_and(|c| p.assets.iter().any(|x| x.asset == c.asset))
        }) {
            return Err("estate listed liens must match the custody denomination".into());
        }
        // Custody agents cannot participate in other configured economic arrangements.
        if world.participants.iter().any(|p0| p0.agent == p.estate)
            || world
                .pool_inputs
                .iter()
                .any(|pool| pool.account.0 == p.estate)
            || world.households.iter().any(|h| {
                h.agent == p.estate || crate::households::membership::ever_member(h, p.estate)
            })
            || world.assets.iter().any(|a| a.owner == p.estate)
            || state.equipment.values().any(|a| a.owner == p.estate)
            || world
                .lending
                .iter()
                .any(|a| a.debtor == p.estate || a.terms.creditor == p.estate)
            || world
                .prepaid_deliveries
                .iter()
                .any(|t| t.seller == p.estate || t.buyer == p.estate)
            || world
                .employment
                .iter()
                .any(|t| t.worker == p.estate || t.employer == p.estate)
            || world.town_market.as_ref().is_some_and(|c| {
                crate::town_market::listings(c)
                    .iter()
                    .any(|listing| listing.traders.iter().any(|t| t.trader.agent == p.estate))
            })
            || world.minting.as_ref().is_some_and(|c| {
                c.issuer == p.estate
                    || c.deals
                        .iter()
                        .any(|d| d.buyer == p.estate || d.seller == p.estate)
                    || c.order_policy
                        .as_ref()
                        .is_some_and(|o| o.quotes.iter().any(|q| q.agent == p.estate))
            })
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
                }) || m.reserves.keys().any(|(seller, _)| *seller == p.estate)
                    || world.bids.iter().any(|b| b.buyer == p.estate)
                    || m.plots.is_some()
                    || m.cash.as_ref().is_some_and(|c| c.lender == p.estate)
            })
            || state
                .exchange
                .forwards
                .values()
                .any(|c| c.debtor == p.estate || c.creditor == p.estate)
            || world
                .negotiation
                .as_ref()
                .is_some_and(|s| [s.buyer.agent, s.seller.agent, s.marketplace].contains(&p.estate))
            || world.credit.as_ref().is_some_and(|c| {
                c.stock_sales.as_ref().is_some_and(|s| {
                    s.forecast.is_some()
                        || s.joint.is_some()
                        || world.bids.iter().any(|b| b.buyer == p.estate)
                }) || c.resale_buyer.is_some()
                    || c.application.buyer == p.estate
                    || c.offers.iter().any(|o| {
                        o.sale.seller == p.estate
                            || o.loan.creditor == p.estate
                            || o.collateral.settlement
                                != credit::CollateralSettlement::AuthorizedLiquidation
                            || (c.application.buyer == p.debtor
                                && p.assets.iter().any(|a| a.asset == o.sale.asset)
                                && o.loan.denomination != p.denomination)
                    })
                    || c.endowments.iter().any(|e| e.agent == p.estate)
                    || c.transfers
                        .iter()
                        .any(|t| t.transfer.from == p.estate || t.transfer.to == p.estate)
            })
        {
            return Err("estate custody requires direct loans or authorized-liquidation mortgages without active custody-agent trading".into());
        }
        if state.month >= p.opening_month
            && world.households.iter().any(|h| {
                h.agent == p.debtor
                    && crate::households::dissolution::winding_at(h, state.month).is_none()
            })
        {
            return Err("household recovery requires explicit wind-down before opening".into());
        }
        let cash = state
            .credit
            .recovery
            .proceedings
            .get(&p.id)
            .map_or(0, |c| c.cash);
        let total = custody.entry((p.estate, p.denomination)).or_default();
        *total = total
            .checked_add(i64::from(cash))
            .ok_or("custody total overflow")?;
    }
    if custody
        .iter()
        .any(|(&(agent, resource), &cash)| i64::from(state.balance(agent, resource)) != cash)
    {
        return Err("estate cash does not reconcile".into());
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
            let (debtor, _, denomination) = g.claim.parties(world).unwrap();
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
                || !subrogation::validate_loan(world, g, loan)?
                || loan.monthly_rate_bps != 0
                || loan.interest != 0
                || loan.interest_remainder != 0
                || loan.term_months != RECOURSE_TERM_MONTHS
                || loan.grace_months != 0
                || loan.priority != g.priority
                || state
                    .credit
                    .recovery
                    .guarantee_advances
                    .keys()
                    .filter_map(|(id, month)| (*id == g.id).then_some(*month))
                    .min()
                    != Some(loan.opened)
            {
                return Err("invalid subrogated loan".into());
            }
        }
    }
    let mut advanced = BTreeMap::<u32, i32>::new();
    for (&(id, month), &quantity) in &state.credit.recovery.guarantee_advances {
        let g = config
            .guarantees
            .iter()
            .find(|g| g.id == id)
            .ok_or("unknown guarantee advance")?;
        if quantity <= 0
            || month < g.from
            || month > g.through
            || month > state.month
            || admission::accepted_month(world, &state.credit, g).is_none_or(|accepted| {
                accepted > month
                    || (world.recovery.posted_guarantees.contains(&g.id) && accepted == month)
            })
        {
            return Err("invalid dated guarantee advance".into());
        }
        let total = advanced.entry(id).or_default();
        *total = total
            .checked_add(quantity)
            .ok_or("guarantee advances overflow")?;
    }
    if advanced != state.credit.recovery.paid_guarantees {
        return Err("dated advances do not reconcile to guarantee payments".into());
    }
    let mut covered_payments = BTreeMap::<GuaranteedClaim, i32>::new();
    for g in &config.guarantees {
        let paid = advanced.get(&g.id).copied().unwrap_or(0);
        if paid == 0 {
            continue;
        }
        let q = covered_payments.entry(g.claim).or_default();
        *q = q.checked_add(paid).ok_or("covered payment overflow")?;
    }
    for (claim, paid) in covered_payments {
        let actual = match claim {
            GuaranteedClaim::Wages {
                agreement,
                earned_month,
            } => state
                .employment
                .earned
                .get(&(agreement, earned_month))
                .map(|e| e.claim.settled),
            GuaranteedClaim::Land { agreement, due } => {
                state.obligations.get(&(agreement, due)).map(|o| o.paid)
            }
            GuaranteedClaim::Loan(id) => state.credit.loans.get(&id).map(|_| paid),
            GuaranteedClaim::Forward(id) => state.exchange.forwards.get(&id).map(|c| c.delivered),
        };
        if actual.is_none_or(|actual| paid > actual) {
            return Err("guarantee advances exceed actual covered settlement".into());
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
        let receivables = crate::recovery_claims::receivables(world, state, p.debtor)?;
        if case.opened != p.opening_month
            || case.opened > state.month
            || case.cash < 0
            || (case.stage == Stage::Closed) != case.closed.is_some()
            || case.closed.is_some_and(|closed| {
                closed < p.earliest_close
                    || closed < case.opened
                    || closed > state.month
                    || (state.credit.loans.values().any(|l| {
                        l.debtor == p.debtor
                            && l.opened <= closed
                            && (l.status == Status::Discharged
                                || (l.status == Status::Enforced && l.debt().unwrap_or(0) > 0))
                    }) && receivables.iter().any(|r| {
                        r.recognized < closed
                            || (r.recognized == closed
                                && matches!(r.contract, finance::ContractId::Land(_)))
                    }))
                    || native_loans(&state.credit, p)
                        .iter()
                        .any(|c| c.due <= closed)
                    || state.credit.loans.values().any(|l| {
                        l.debtor == p.debtor
                            && current_recourse(world, &state.credit, l.id, closed) > 0
                    })
                    || crate::recovery_claims::outstanding(world, state, p.debtor)
                        .iter()
                        .any(|claim| match claim.contract {
                            finance::ContractId::Land(id) => state.obligations.values().any(|o| {
                                o.agreement == id && o.due <= closed && o.outstanding() > 0
                            }),
                            finance::ContractId::Wages(id) => {
                                state
                                    .employment
                                    .earned
                                    .iter()
                                    .any(|(&(agreement, month), e)| {
                                        agreement == id
                                            && month < closed
                                            && e.claim.outstanding() > 0
                                    })
                            }
                            finance::ContractId::Forward(id) => {
                                state.exchange.forwards[&id].issued <= closed
                            }
                            finance::ContractId::Loan(_) | finance::ContractId::Guarantee(_) => {
                                false
                            }
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
                (l.denomination == p.denomination || l.collateral.is_none())
                    && l.status != Status::PendingSale
                    && l.collateral
                        .as_ref()
                        .is_none_or(|c| !c.pledged || p.assets.iter().any(|a| a.asset == c.asset))
            })
            && p.assets
                .iter()
                .all(|a| saleable_asset(world, state, p.debtor, a.asset));
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
                sold_inventory: BTreeSet::new(),
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
    // Coin servicing is stayed; unsecured native performance keeps its denomination.
    // The prior accrual marker still advances once.
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
    state: &State,
    g: &Guarantee,
) -> Result<Option<finance::Obligation>, String> {
    let month = state.month;
    if month < g.from
        || month > g.through
        || active(world, &state.credit, g.guarantor).is_some()
        || admission::accepted_month(world, &state.credit, g).is_none_or(|accepted| {
            accepted > month
                || (world.recovery.posted_guarantees.contains(&g.id) && accepted == month)
        })
    {
        return Ok(None);
    }
    let (creditor, denomination, covered, first_unpaid) = match g.claim {
        GuaranteedClaim::Loan(id) => {
            let Some(loan) = state.credit.loans.get(&id) else {
                return Ok(None);
            };
            let covered = if active(world, &state.credit, loan.debtor).is_some() {
                loan.debt()?
            } else {
                loan.due(month)?
            };
            (loan.creditor, loan.denomination, covered, loan.first_unpaid)
        }
        GuaranteedClaim::Forward(id) => {
            let Some(c) = state.exchange.forwards.get(&id) else {
                return Ok(None);
            };
            // Due precedes Acquire: preserve the first ordinary delivery window.
            // Subsequent guarantees cover its residual, not hypothetical output.
            (
                c.creditor,
                c.goods.resource,
                c.claim().outstanding(),
                c.effective_due().checked_add(1),
            )
        }
        GuaranteedClaim::Wages {
            agreement,
            earned_month,
        } => {
            let Some(e) = state.employment.earned.get(&(agreement, earned_month)) else {
                return Ok(None);
            };
            let finance::Condition::OnOrAfterMonth(due) = e.claim.condition else {
                return Err("earned wage requires due date".into());
            };
            (
                e.claim.transfer.to,
                e.claim.transfer.amount.resource,
                if due <= month {
                    e.claim.outstanding()
                } else {
                    0
                },
                Some(due),
            )
        }
        GuaranteedClaim::Land { agreement, due } => {
            let Some(o) = state.obligations.get(&(agreement, due)) else {
                return Ok(None);
            };
            let Some(a) = crate::commitments::active(world, state).find(|a| a.id == agreement)
            else {
                return Ok(None);
            };
            (
                a.creditor,
                a.payment.resource,
                if o.effective_due() <= month {
                    o.outstanding()
                } else {
                    0
                },
                Some(o.effective_due()),
            )
        }
    };
    if first_unpaid.is_none_or(|m| month < m || month - m < g.delay_months) {
        return Ok(None);
    }
    let requested = covered.min(
        g.cap
            - state
                .credit
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
            to: creditor,
            amount: Amount::new(denomination, requested),
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
    let mut pooling = crate::households::income_reservations::Reservations::new(
        world,
        state,
        execution.stored.clone(),
    );
    let mut terms: Vec<_> = world
        .recovery
        .guarantees
        .iter()
        .filter(|g| g.from <= state.month && state.month <= g.through)
        .collect();
    terms.sort_by_key(|g| (g.priority, g.id));
    loop {
        let current = crate::recovery_claims::current(state, out);
        let requests: Vec<_> = terms
            .iter()
            .filter_map(|g| match guarantee_claim(world, &current, g) {
                Ok(Some(claim)) => Some(Ok(finance::CollectionRequest {
                    contract: finance::ContractId::Guarantee(g.id),
                    rank: g.priority,
                    claim,
                })),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<_, String>>()?;
        let mut funding_requests = Vec::new();
        let mut lots = BTreeMap::new();
        for r in &requests {
            let g = terms
                .iter()
                .find(|g| r.contract == finance::ContractId::Guarantee(g.id))
                .unwrap();
            let (claim, lot) = tender::funding(world, g, &r.claim)?;
            lots.insert(r.contract, lot);
            funding_requests.push(finance::CollectionRequest { claim, ..r.clone() });
        }
        let grants = if world.recovery.guarantee_policy == finance::CollectionPolicy::Proportional {
            Some(finance::proportional_lots(
                world,
                state.month,
                execution,
                &protected,
                &funding_requests,
                &lots,
            )?)
        } else {
            None
        };
        let mut round_paid = 0_i64;
        for g in &terms {
            let Some(request) = requests
                .iter()
                .find(|r| r.contract == finance::ContractId::Guarantee(g.id))
            else {
                continue;
            };
            let requested = request.claim.outstanding();
            let lot = lots[&request.contract];
            let allocated = grants
                .as_ref()
                .map(|grants| grants[&request.contract] / lot);
            let Some(claim) =
                guarantee_claim(world, &crate::recovery_claims::current(state, out), g)?
            else {
                out.recovery.push(Receipt::Guaranteed {
                    guarantee: g.id,
                    claim: g.claim,
                    requested,
                    allocated,
                    paid: 0,
                    tender: Amount::new(
                        tender::funding(world, g, &request.claim)?
                            .0
                            .transfer
                            .amount
                            .resource,
                        0,
                    ),
                    recourse: g.recourse,
                });
                continue;
            };
            let (debtor, _, denomination) =
                g.claim.parties(world).ok_or("missing guaranteed terms")?;
            let (funding, _) = tender::funding(world, g, &claim)?;
            let payment_resource = funding.transfer.amount.resource;
            let reserve = protected
                .get(&(g.guarantor, payment_resource))
                .copied()
                .unwrap_or(0);
            let limit = (execution
                .available
                .get(&(g.guarantor, payment_resource))
                .copied()
                .unwrap_or(0)
                - reserve)
                .max(0)
                .min(grants.as_ref().map_or(i32::MAX, |g| g[&request.contract]));
            let pooled_receipt = matches!(
                g.claim,
                GuaranteedClaim::Wages { .. } | GuaranteedClaim::Forward(_)
            );
            let limit = if pooled_receipt {
                limit.min(pooling.payment_limit(world, execution, &funding)?)
            } else {
                limit.min(pooling.unpooled_payment_limit(world, execution, &funding)?)
            };
            let payment =
                execution.pay_bounded(world, state.month, true, &funding, limit / lot * lot)?;
            let paid = payment.paid / lot;
            round_paid += i64::from(paid);
            if paid > 0 {
                if pooled_receipt {
                    pooling = pooling
                        .preview(world, &payment.effects)?
                        .ok_or("guaranteed receipt exceeds pooled storage")?;
                } else {
                    pooling.reserve_unpooled(world, &payment.effects)?;
                }
                let transaction = credit::tx(
                    format!("guarantee {} pays {:?}", g.id, g.claim),
                    payment.effects.clone(),
                );
                let inherited = subrogation::collateral(world, &out.after, g)?;
                match g.claim {
                    GuaranteedClaim::Loan(id) => {
                        let loan = out.after.loans.get_mut(&id).unwrap();
                        loan.apply_payment(paid);
                        if loan.due(state.month)? == 0 {
                            loan.first_unpaid = None;
                        }
                        out.transactions.push(transaction);
                    }
                    GuaranteedClaim::Forward(id) => {
                        let c = out
                            .forward_changes
                            .entry(id)
                            .or_insert_with(|| state.exchange.forwards[&id].clone());
                        c.delivered = c
                            .delivered
                            .checked_add(paid)
                            .ok_or("guaranteed delivery overflow")?;
                        out.transactions.push(transaction);
                    }
                    GuaranteedClaim::Wages {
                        agreement,
                        earned_month,
                    } => {
                        let book = out
                            .employment
                            .get_or_insert_with(|| state.employment.clone());
                        book.earned
                            .get_mut(&(agreement, earned_month))
                            .unwrap()
                            .claim
                            .settled += paid;
                        out.transactions.push(transaction);
                    }
                    GuaranteedClaim::Land { agreement, due } => {
                        let a = crate::commitments::active(world, state)
                            .find(|a| a.id == agreement)
                            .ok_or("missing guaranteed land agreement")?;
                        if out.commitments.is_none() {
                            out.commitments = Some(crate::commitments::Settlement {
                                collections: vec![],
                                policy: world.payment_policy,
                                protected: crate::commitments::protected_stock(world, state)?,
                                obligations: crate::commitments::due_obligations(world, state)?,
                                transactions: vec![],
                            });
                        }
                        let settlement = out.commitments.as_mut().unwrap();
                        let bill = settlement.obligations.get_mut(&(agreement, due)).unwrap();
                        let txs = crate::commitments::record_payment(
                            world,
                            a,
                            bill,
                            paid,
                            g.tender == GuaranteeTender::Native,
                            payment.effects,
                        );
                        // Keep identical transfer records in the domain receipt and common batch.
                        out.transactions.extend(txs.clone());
                        settlement.transactions.extend(txs);
                    }
                }
                *out.after.recovery.paid_guarantees.entry(g.id).or_default() += paid;
                let advance = out
                    .after
                    .recovery
                    .guarantee_advances
                    .entry((g.id, state.month))
                    .or_default();
                *advance = advance.checked_add(paid).ok_or("dated advance overflow")?;
                let stayed = active(world, &out.after, debtor).is_some();
                let recourse = out.after.loans.entry(g.recourse).or_insert_with(|| Loan {
                    id: g.recourse,
                    creditor: g.guarantor,
                    debtor,
                    denomination,
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
                recourse.collateral = inherited;
                recourse.status = if stayed {
                    Status::Stayed
                } else if recourse.collateral.as_ref().is_some_and(|c| !c.pledged) {
                    Status::Enforced
                } else {
                    Status::Active
                };
                recourse.last_accrued = state.month;
                subrogation::transfer_reserved(&mut out.after, g, paid)?;
            }
            out.recovery.push(Receipt::Guaranteed {
                guarantee: g.id,
                claim: g.claim,
                requested,
                allocated,
                paid,
                tender: Amount::new(payment_resource, payment.paid),
                recourse: g.recourse,
            });
        }
        // Overlapping coverage can release a grant after another guarantor pays.
        // Re-inventory remaining claims against unspent opening funds, never receipts.
        if grants.is_none() || round_paid == 0 {
            break;
        }
    }
    Ok(())
}

/// Catalog property keeps its existing attachment-control adapter. A portable
/// durable has no attached title/work or surviving output-share contract to novate.
pub(crate) fn saleable_asset(
    world: &World,
    state: &State,
    debtor: AgentId,
    asset: AssetId,
) -> bool {
    crate::asset_exchange::owner(world, state, &state.credit, asset) == Some(debtor)
        && state.equipment.get(&asset).is_none_or(|a| {
            crate::equipment::transferable(a, state.month)
                && !state.exchange.contracts.contains_key(&asset)
        })
}

/// Proceeds are a reservation, not a payment: actual distribution is next Due.
fn lien_proceeds(
    world: &World,
    state: &State,
    out: &credit::Boundary,
    p: &ProceedingTerms,
    asset: AssetId,
    proceeds: i32,
) -> Result<BTreeMap<u32, i32>, String> {
    let mut requests = Vec::new();
    for loan in out.after.loans.values().filter(|l| l.debtor == p.debtor) {
        let Some(c) = loan
            .collateral
            .as_ref()
            .filter(|c| c.pledged && c.asset == asset)
        else {
            continue;
        };
        if loan.denomination != p.denomination {
            return Err("liquidation lien denomination differs from sale proceeds".into());
        }
        requests.push(finance::CollectionRequest {
            contract: finance::ContractId::Loan(loan.id),
            rank: c.priority,
            claim: estate_claim(p, loan, loan.debt()?, state.month),
        });
    }
    requests.sort_by_key(|r| (r.rank, r.contract));
    let mut window = finance::Execution::opening(world, state);
    window.available = [((p.estate, p.denomination), proceeds)].into();
    let shared = if world.collection_policy == finance::CollectionPolicy::Proportional {
        Some(finance::proportional_grants(
            world,
            state.month,
            &window,
            &BTreeMap::new(),
            &requests,
        )?)
    } else {
        None
    };
    let mut result = BTreeMap::new();
    for request in requests {
        let finance::ContractId::Loan(id) = request.contract else {
            unreachable!()
        };
        let limit = shared.as_ref().map_or(i32::MAX, |g| g[&request.contract]);
        let payment = window.pay_bounded(world, state.month, true, &request.claim, limit)?;
        result.insert(id, payment.paid);
    }
    Ok(result)
}

pub(crate) fn sales(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    execution: &mut finance::Execution,
) -> Result<crate::households::income_reservations::Reservations, String> {
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
        let valid = saleable_asset(world, state, p.debtor, b.asset)
            && out
                .after
                .recovery
                .proceedings
                .get(&p.id)
                .is_some_and(|c| c.stage == Stage::Active && !c.sold.contains(&b.asset))
            && market::eligible_buyer(world, state, b.buyer)
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
        // Allocate this asset's proceeds once, before they join other estate
        // cash. Unsecured claim rank cannot override collateral lien priority.
        let grants = lien_proceeds(world, state, out, p, b.asset, b.price)?;
        let case = out.after.recovery.proceedings.get_mut(&p.id).unwrap();
        case.cash = case
            .cash
            .checked_add(b.price)
            .ok_or("estate proceeds overflow")?;
        case.sold.insert(b.asset);
        for (id, amount) in grants {
            case.secured.insert(id, amount);
            let loan = out.after.loans.get_mut(&id).unwrap();
            loan.collateral.as_mut().unwrap().pledged = false;
            loan.status = Status::Stayed;
        }
        out.recovery.push(Receipt::Sold {
            proceeding: p.id,
            asset: b.asset,
            buyer: b.buyer,
            proceeds: b.price,
        });
    }
    receivables::sales(world, state, out, execution)?;
    inventory::sales(world, state, out, execution)
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
        let account = (p.estate, p.denomination);
        let pooled = execution.available.get(&account).copied().unwrap_or(0);
        let owned = state
            .credit
            .recovery
            .proceedings
            .get(&p.id)
            .map_or(0, |c| c.cash);
        // Opening beneficial ownership, not aggregate custody cash, bounds this
        // estate. New deposits cannot borrow another estate's opening liquidity.
        let allowance = pooled.min(owned);
        let mut scoped = execution.clone();
        scoped.available.insert(account, allowance);
        distribute_case(world, state, out, &mut scoped, p)?;
        let remaining = scoped.available.get(&account).copied().unwrap_or(0);
        let spent = allowance
            .checked_sub(remaining)
            .filter(|spent| *spent >= 0)
            .ok_or("invalid custody spending")?;
        scoped.available.insert(
            account,
            pooled
                .checked_sub(spent)
                .ok_or("custody spending overflow")?,
        );
        *execution = scoped;
    }
    Ok(())
}

fn distribute_case(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    execution: &mut finance::Execution,
    p: &ProceedingTerms,
) -> Result<(), String> {
    let mut case = out.after.recovery.proceedings[&p.id].clone();
    // Collect only unprotected opening coins. Newly deposited funds become
    // distributable next month, like sale proceeds received at Acquire.
    let (nonloan_requests, _) = crate::recovery_claims::cash_requests(world, state, out, p)?;
    let nonloan_cash = nonloan_requests.iter().try_fold(0_i32, |sum, r| {
        sum.checked_add(r.claim.outstanding())
            .ok_or("estate cash demand overflow")
    })?;
    let debt = out
        .after
        .loans
        .values()
        .filter(|l| l.debtor == p.debtor && l.denomination == p.denomination)
        .try_fold(nonloan_cash, |sum, l| {
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
        let current = current_recourse(world, &out.after, id, state.month);
        let loan = out
            .after
            .loans
            .get_mut(&id)
            .ok_or("estate lien without loan")?;
        let reserved = case.secured[&id].min(loan.debt()?);
        let requested = reserved.min(loan.debt()?.saturating_sub(current));
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
        case.secured.insert(id, (reserved - payment.paid).max(0));
    }
    let mut requests: Vec<_> = out
        .after
        .loans
        .values()
        .filter(|l| {
            l.debtor == p.debtor
                && l.denomination == p.denomination
                && l.opened < state.month
                && l.debt().unwrap_or(0) > 0
        })
        .map(|l| {
            Ok(finance::CollectionRequest {
                contract: finance::ContractId::Loan(l.id),
                rank: rank(world, l),
                claim: estate_claim(
                    p,
                    l,
                    l.debt()?.saturating_sub(current_recourse(
                        world,
                        &out.after,
                        l.id,
                        state.month,
                    )),
                    state.month,
                ),
            })
        })
        .collect::<Result<_, String>>()?;
    let remaining_lien: i32 = case.secured.values().sum();
    let protected = BTreeMap::from([((p.estate, p.denomination), remaining_lien)]);
    let (nonloan_requests, lots) = crate::recovery_claims::cash_requests(world, state, out, p)?;
    requests.extend(nonloan_requests);
    let grants =
        finance::proportional_lots(world, state.month, execution, &protected, &requests, &lots)?;
    for request in requests {
        if matches!(request.contract, finance::ContractId::Wages(_)) {
            case.cash -= crate::recovery_claims::pay_wages(
                world,
                state,
                out,
                p,
                &request,
                grants[&request.contract],
                execution,
            )?;
            continue;
        }
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
            l.debtor == p.debtor
                && l.debt().unwrap_or(0) > 0
                && (l.opened == state.month
                    || current_recourse(world, &out.after, l.id, state.month) > 0)
        })
        && p.assets.iter().all(|a| case.sold.contains(&a.asset))
        && inventory::cleared(world, p.id, &case)
        && case.secured.values().all(|v| *v == 0)
    {
        let mut nonloans = crate::recovery_claims::outstanding(
            world,
            &crate::recovery_claims::current(state, out),
            p.debtor,
        );
        nonloans.extend(native_loans(&out.after, p));
        nonloans.sort_by_key(|c| (c.contract, c.due));
        if !nonloans.is_empty() {
            out.recovery.push(Receipt::ClosureDeferred {
                proceeding: p.id,
                claims: nonloans,
            });
            out.after.recovery.proceedings.insert(p.id, case);
            return Ok(());
        }
        let deficiency = out
            .after
            .loans
            .values()
            .filter(|l| l.debtor == p.debtor && l.denomination == p.denomination)
            .try_fold(0_i32, |sum, l| {
                sum.checked_add(l.debt()?)
                    .ok_or("estate deficiency overflow".to_string())
            })?;
        // Ordinary collection may have just paid the debtor; those receipts
        // cannot be swept/spent again in this window. Outstanding receivables
        // likewise remain property until performed or explicitly disposed of.
        if deficiency > 0 {
            let current = crate::recovery_claims::current(state, out);
            let receivables = crate::recovery_claims::receivables(world, &current, p.debtor)?;
            let account = (p.debtor, p.denomination);
            let cash = out
                .transactions
                .iter()
                .flat_map(|t| &t.effects)
                .filter(|e| e.account == account)
                .try_fold(
                    i64::from(state.balance(p.debtor, p.denomination)),
                    |v, e| {
                        v.checked_add(i64::from(e.delta))
                            .ok_or("estate closing cash overflow")
                    },
                )?;
            let protected = crate::commitments::protected_stock(world, state)?;
            let uncollected_cash = i32::try_from(
                (cash - i64::from(protected.get(&account).copied().unwrap_or(0))).max(0),
            )
            .map_err(|_| "estate closing cash overflow")?;
            if !receivables.is_empty() || uncollected_cash > 0 {
                out.recovery.push(Receipt::AssetsPending {
                    proceeding: p.id,
                    receivables,
                    uncollected_cash,
                });
                out.after.recovery.proceedings.insert(p.id, case);
                return Ok(());
            }
        }
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
            for l in out.after.loans.values_mut().filter(|l| {
                l.debtor == p.debtor
                    && l.denomination == p.denomination
                    && l.debt().unwrap_or(0) > 0
            }) {
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
    Ok(())
}

pub(crate) fn current_recourse(world: &World, book: &credit::Book, loan: u32, month: u32) -> i32 {
    world
        .recovery
        .guarantees
        .iter()
        .find(|g| g.recourse == loan)
        .and_then(|g| book.recovery.guarantee_advances.get(&(g.id, month)))
        .copied()
        .unwrap_or(0)
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
