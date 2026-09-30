//! General advances and financed purchases share monthly debt and claim settlement.
//! Ownership and debt have one authoritative book; collateral terms are optional.
use crate::{finance, model::*};
use std::collections::{BTreeMap, BTreeSet};

pub mod projection;

pub const RATE_SCALE: i64 = 10_000;
const PRICE_TICKS: i32 = 10_000;
const DOWNPAYMENT_TICKS: i32 = 2_000;
const TREASURY_TICKS: i32 = 100_000;
const MONTHLY_INCOME_TICKS: i32 = 2_100;
const MONTHLY_RATE_BPS: u32 = 100;
const TERM_MONTHS: u32 = 4;
const GRACE_MONTHS: u32 = 1;
const DISTRESSED_VALUE: i32 = 6_000;
const MAX_TERM_MONTHS: u32 = 120;
const CROP_PROVIDED_LABOR: i32 = 2;
const CROP_INITIAL_SEED: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sale {
    pub asset: AssetId,
    pub seller: AgentId,
    pub price: Amount,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoanOffer {
    pub creditor: AgentId,
    pub denomination: ResourceId,
    pub max_principal: i32,
    pub monthly_rate_bps: u32,
    pub term_months: u32,
    pub grace_months: u32,
}
/// A dated, mutually accepted advance. Discovery/underwriting supply these terms;
/// execution neither invents consent nor promises that the lender has funds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Advance {
    pub id: u32,
    pub debtor: AgentId,
    pub terms: LoanOffer,
    pub principal: i32,
    pub month: u32,
    pub collateral: Option<Collateral>,
    pub priority: u32,
}
pub fn enabled(world: &World) -> bool {
    world.credit.is_some()
        || !world.lending.is_empty()
        || !world.recovery.proceedings.is_empty()
        || !world.recovery.guarantees.is_empty()
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collateral {
    pub asset: AssetId,
    pub priority: u32,
    pub settlement: CollateralSettlement,
    pub pledged: bool,
}
/// Agreement-selected settlement: immediate fixed credit or actual later proceeds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollateralSettlement {
    FixedValue {
        value: i32,
    },
    ResaleProceeds {
        minimum_price: i32,
    },
    /// Title stays with the debtor until an authorized proceeding sells it.
    /// Compatible liens share actual proceeds; arrears alone never transfer title.
    AuthorizedLiquidation,
}
impl CollateralSettlement {
    fn is_valid(&self) -> bool {
        match *self {
            Self::FixedValue { value } => value > 0,
            Self::ResaleProceeds { minimum_price } => minimum_price > 0,
            Self::AuthorizedLiquidation => true,
        }
    }
}
/// Shared collateral requires explicit common enforcement and denomination.
/// A fixed-value or creditor-resale promise remains exclusive.
fn compatible_liens(
    a: &Collateral,
    denomination: ResourceId,
    b: &Collateral,
    other: ResourceId,
) -> bool {
    denomination == other
        && a.settlement == CollateralSettlement::AuthorizedLiquidation
        && b.settlement == CollateralSettlement::AuthorizedLiquidation
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub id: u32,
    pub sale: Sale,
    pub loan: LoanOffer,
    pub minimum_downpayment: i32,
    pub collateral: Collateral,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application {
    pub offer: u32,
    pub buyer: AgentId,
    pub month: u32,
    pub downpayment: i32,
}

/// Configured offer inspection only. Once admitted, the loan book owns terms
/// and creditor identity; an assignment does not rewrite this source offer.
pub(crate) struct OfferedLoan<'a> {
    pub debtor: AgentId,
    pub terms: &'a LoanOffer,
    pub collateral: Option<&'a Collateral>,
}

pub(crate) fn offered_loan(world: &World, id: u32) -> Option<OfferedLoan<'_>> {
    world
        .lending
        .iter()
        .find(|a| a.id == id)
        .map(|a| OfferedLoan {
            debtor: a.debtor,
            terms: &a.terms,
            collateral: a.collateral.as_ref(),
        })
        .or_else(|| {
            world.credit.as_ref().and_then(|c| {
                c.offers.iter().find(|o| o.id == id).map(|o| OfferedLoan {
                    debtor: c.application.buyer,
                    terms: &o.loan,
                    collateral: Some(&o.collateral),
                })
            })
        })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endowment {
    pub agent: AgentId,
    pub amount: Amount,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledTransfer {
    pub month: u32,
    pub transfer: finance::Transfer,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub stock_sales: Option<crate::stock_sale::Policy>,
    pub purchase_policy: crate::borrowing::Policy,
    pub resale_buyer: Option<crate::resale::Buyer>,
    /// These use rights and their active processes follow asset ownership.
    pub attached_rights: BTreeSet<u32>,
    pub offers: Vec<Offer>,
    pub application: Application,
    pub endowments: Vec<Endowment>,
    pub transfers: Vec<ScheduledTransfer>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Active,
    Repaid,
    Enforced,
    PendingSale,
    Stayed,
    Discharged,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loan {
    pub id: u32,
    pub creditor: AgentId,
    pub debtor: AgentId,
    pub denomination: ResourceId,
    pub original_principal: i32,
    pub principal: i32,
    pub interest: i32,
    pub interest_remainder: i64,
    pub monthly_rate_bps: u32,
    pub opened: u32,
    pub last_accrued: u32,
    pub term_months: u32,
    pub grace_months: u32,
    pub first_unpaid: Option<u32>,
    pub status: Status,
    pub collateral: Option<Collateral>,
    /// Lower ranks collect first; equal ranks use the configured sharing policy.
    pub priority: u32,
}
impl Loan {
    fn accepted(a: &Advance) -> Self {
        Self {
            id: a.id,
            creditor: a.terms.creditor,
            debtor: a.debtor,
            denomination: a.terms.denomination,
            original_principal: a.principal,
            principal: a.principal,
            interest: 0,
            interest_remainder: 0,
            monthly_rate_bps: a.terms.monthly_rate_bps,
            opened: a.month,
            last_accrued: a.month,
            term_months: a.terms.term_months,
            grace_months: a.terms.grace_months,
            first_unpaid: None,
            status: Status::Active,
            collateral: a.collateral.clone(),
            priority: a.priority,
        }
    }
    pub fn debt(&self) -> Result<i32, String> {
        self.principal
            .checked_add(self.interest)
            .ok_or("debt overflow".into())
    }
    pub fn principal_due(&self, month: u32) -> i32 {
        if matches!(self.status, Status::Enforced | Status::Stayed) {
            return self.principal;
        }
        let elapsed = month.saturating_sub(self.opened).min(self.term_months);
        let scheduled =
            i64::from(self.original_principal) * i64::from(elapsed) / i64::from(self.term_months);
        (scheduled - i64::from(self.original_principal - self.principal)).max(0) as i32
    }
    pub fn due(&self, month: u32) -> Result<i32, String> {
        self.principal_due(month)
            .checked_add(self.interest)
            .ok_or("payment overflow".into())
    }
    /// One debt yields symmetric receivable/payable views, not duplicated balances.
    pub fn claim(&self, month: u32) -> Result<finance::Obligation, String> {
        Ok(finance::Obligation {
            transfer: finance::Transfer {
                from: self.debtor,
                to: self.creditor,
                amount: Amount::new(self.denomination, self.due(month)?),
            },
            settled: 0,
            condition: finance::Condition::OnOrAfterMonth(self.opened + 1),
            failure: finance::FailureRule::CarryArrears,
        })
    }
    pub(crate) fn apply_payment(&mut self, amount: i32) {
        let interest = amount.min(self.interest);
        self.interest -= interest;
        self.principal -= amount - interest;
        if self.principal == 0 && self.interest == 0 {
            self.status = Status::Repaid;
            if let Some(collateral) = &mut self.collateral {
                collateral.pledged = false;
            }
            self.first_unpaid = None;
            self.interest_remainder = 0;
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Authoritative contract execution state, not a financial reporting book.
/// Financial statements are produced exclusively by accounting/financial_reporting.
pub struct Book {
    pub recovery: crate::recovery::Book,
    /// Actual spending against the scoped posted-stock purchase budget.
    pub stock_spent: i32,
    pub pending_sales: BTreeMap<u32, crate::resale::PendingSale>,
    pub loans: BTreeMap<u32, Loan>,
    /// Overrides to the catalog's opening asset ownership and carrying value.
    pub owners: BTreeMap<AssetId, AgentId>,
    pub values: BTreeMap<AssetId, i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    Downpayment,
    Funding,
    UnavailableAsset,
    Ineligible,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Advanced {
        loan: u32,
        creditor: AgentId,
        debtor: AgentId,
        amount: Amount,
    },
    RepossessedForSale {
        loan: u32,
        asset: AssetId,
        minimum_price: i32,
    },
    ResaleNoBuyer {
        loan: u32,
    },
    ResaleBid {
        loan: u32,
        quote: crate::resale::Bid,
    },
    ResaleRejected {
        loan: u32,
        bid: i32,
        minimum_price: i32,
    },
    Resold {
        loan: u32,
        buyer: AgentId,
        price: i32,
        debt_credit: i32,
        surplus: i32,
        remaining_debt: i32,
    },
    Endowed {
        agent: AgentId,
        amount: Amount,
    },
    Cashflow {
        from: AgentId,
        to: AgentId,
        amount: Amount,
    },
    Rejected {
        offer: u32,
        reason: Rejection,
    },
    Purchased {
        offer: u32,
        asset: AssetId,
        buyer: AgentId,
        price: i32,
        downpayment: i32,
        advance: i32,
    },
    Accrued {
        loan: u32,
        opening_principal: i32,
        interest: i32,
    },
    Paid {
        loan: u32,
        interest: i32,
        principal: i32,
    },
    Arrears {
        loan: u32,
        amount: i32,
        since: u32,
    },
    EnforcementDeferred {
        loan: u32,
        surplus: i32,
    },
    Enforced {
        loan: u32,
        value: i32,
        debt_credit: i32,
        surplus: i32,
        remaining_debt: i32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boundary {
    /// Actual durable title/condition after funded estate sales, in the equipment registry.
    pub equipment: BTreeMap<AssetId, crate::equipment::DurableAsset>,
    pub employment: Option<crate::employment::Book>,
    pub forward_changes: BTreeMap<AssetId, crate::forward::Contract>,
    pub recovery: Vec<crate::recovery::Receipt>,
    pub collections: Vec<finance::CollectionReceipt>,
    pub commitments: Option<crate::commitments::Settlement>,
    pub production_plan: Option<Box<Batch>>,
    pub stock_sale: Option<crate::stock_sale::Receipt>,
    pub decision: Option<crate::borrowing::Decision>,
    pub attachments: Vec<ProcessChange>,
    pub after: Book,
    pub events: Vec<Event>,
    pub transactions: Vec<Transaction>,
}
/// Apply accepted non-balance records to a committed or hypothetical state.
/// Callers separately reserve/apply transaction effects exactly once. This does
/// not execute production, advance time, or authorize the supplied boundary.
pub(crate) fn record(state: &mut State, boundary: &Boundary) {
    state.credit = boundary.after.clone();
    state.equipment.extend(boundary.equipment.clone());
    if let Some(book) = &boundary.employment {
        state.employment = book.clone();
    }
    state
        .exchange
        .forwards
        .extend(boundary.forward_changes.clone());
    if let Some(commitments) = &boundary.commitments {
        state.obligations = commitments.obligations.clone();
    }
    for change in &boundary.attachments {
        state
            .processes
            .insert(change.after.id, change.after.clone());
    }
}

pub fn owner(world: &World, state: &State, asset: AssetId) -> Option<AgentId> {
    state
        .credit
        .owners
        .get(&asset)
        .copied()
        .or_else(|| world.assets.iter().find(|a| a.id == asset).map(|a| a.owner))
}
fn purchase_party(world: &World, state: &State, agent: AgentId) -> bool {
    !state.terminal.contains_key(&agent)
        && crate::recovery::active(world, &state.credit, agent).is_none()
        && crate::households::market::active(world, state, agent)
        && !world.households.iter().any(|h| {
            h.agent == agent
                && (crate::households::dissolution::winding_at(h, state.month).is_some()
                    || crate::households::dissolution::closed_at(h, state.month))
        })
}

/// A supplied purchase application has one date. Its catalog entry is not an
/// everlasting encumbrance after that attempt or an accepted loan's repayment.
pub(crate) fn pending_purchase<'a>(world: &'a World, state: &State) -> Option<&'a Offer> {
    let c = world.credit.as_ref()?;
    (c.application.month >= state.month && !state.credit.loans.contains_key(&c.application.offer))
        .then(|| c.offers.iter().find(|o| o.id == c.application.offer))
        .flatten()
}

/// Visible financed offers; application feasibility and settlement remain separate.
pub fn discover<'a>(world: &'a World, state: &State, buyer: AgentId) -> Vec<&'a Offer> {
    world.credit.as_ref().map_or(vec![], |c| {
        c.offers
            .iter()
            .filter(|o| {
                crate::laws::evaluate_terms(
                    world,
                    state,
                    buyer,
                    crate::laws::Terms::FinancedPurchase {
                        monthly_rate_bps: o.loan.monthly_rate_bps,
                    },
                )
                .allowed
                    && buyer != o.sale.seller
                    && buyer != o.loan.creditor
                    && world.agents.iter().any(|a| a.id == buyer)
                    && [buyer, o.sale.seller, o.loan.creditor]
                        .iter()
                        .all(|a| purchase_party(world, state, *a))
                    && !state.credit.loans.contains_key(&o.id)
                    && !state.credit.loans.values().any(|l| {
                        l.collateral
                            .as_ref()
                            .is_some_and(|c| c.pledged && c.asset == o.sale.asset)
                    })
                    && owner(world, state, o.sale.asset) == Some(o.sale.seller)
            })
            .collect()
    })
}
fn validate_purchase(world: &World, state: &State) -> Result<(), String> {
    let Some(c) = &world.credit else {
        return Ok(());
    };
    if world.transaction_policy.is_some() && c.resale_buyer.is_some() {
        return Err("permission-gated collateral resale is not yet supported".into());
    }
    crate::borrowing::validate(world)?;
    crate::stock_sale::validate(world, state)?;
    // Ownership-following production is supported. Other acquisition/collection
    // drivers still require shared funding and ownership rules.
    if !world.access_offers.is_empty()
        || world.market.is_some()
        || world.competition.is_some()
        || (!world.households.is_empty()
            && c.stock_sales.as_ref().is_some_and(|s| s.joint.is_some()))
        || !world.offers.is_empty()
        || (!world.bids.is_empty() && c.stock_sales.is_none())
        || !world.issuance.is_empty()
        || (!world.pools.is_empty() && c.stock_sales.is_some())
        || state.pending_production.as_ref().is_some_and(|plan| {
            state.phase != Phase::Productive
                || plan.phase != Phase::Productive
                || plan.month != state.month
                || plan.id != state.next_batch
        })
    {
        return Err(
            "credit pilot cannot combine unrelated acquisition or collection drivers".into(),
        );
    }
    if world.rights.iter().any(|r| {
        c.offers.iter().any(|o| o.sale.asset == r.asset) && !c.attached_rights.contains(&r.id)
    }) || c.attached_rights.iter().any(|id| {
        !world
            .rights
            .iter()
            .any(|r| r.id == *id && c.offers.iter().any(|o| o.sale.asset == r.asset))
    }) {
        return Err("credit production requires explicit ownership-following rights".into());
    }
    if world.agreements.iter().any(|a| {
        world
            .rights
            .iter()
            .any(|r| r.id == a.right && c.offers.iter().any(|o| o.sale.asset == r.asset))
    }) {
        return Err("financed property cannot silently replace an existing land lease".into());
    }
    let agent = |id| world.agents.iter().any(|a| a.id == id);
    let coin = |id| {
        world
            .resources
            .iter()
            .any(|r| r.id == id && r.kind == ResourceKind::Stock)
            && *world.storage.weights.get(&id).unwrap_or(&0) == 0
    };
    let mut ids = BTreeSet::new();
    let mut assets = BTreeSet::new();
    for o in &c.offers {
        if !ids.insert(o.id)
            || !assets.insert(o.sale.asset)
            || !agent(o.sale.seller)
            || !agent(o.loan.creditor)
            || !world
                .assets
                .iter()
                .any(|a| a.id == o.sale.asset && a.owner == o.sale.seller)
            || o.sale.price.quantity <= 0
            || !coin(o.sale.price.resource)
            || o.loan.denomination != o.sale.price.resource
            || o.minimum_downpayment <= 0
            || o.minimum_downpayment >= o.sale.price.quantity
            || o.loan.max_principal <= 0
            || o.loan.monthly_rate_bps > RATE_SCALE as u32
            || o.loan.term_months == 0
            || o.loan.term_months > MAX_TERM_MONTHS
            || o.loan.grace_months > MAX_TERM_MONTHS
            || o.collateral.asset != o.sale.asset
            || !o.collateral.settlement.is_valid()
            || !o.collateral.pledged
        {
            return Err("invalid financed purchase offer".into());
        }
    }
    if !ids.contains(&c.application.offer)
        || !agent(c.application.buyer)
        || c.application.month == 0
        || c.application.month > u32::MAX - MAX_TERM_MONTHS - 1
        || c.application.downpayment <= 0
    {
        return Err("invalid purchase application".into());
    }
    for e in &c.endowments {
        if !agent(e.agent) || !coin(e.amount.resource) || e.amount.quantity <= 0 {
            return Err("invalid initial endowment".into());
        }
    }
    for t in &c.transfers {
        if t.month <= 1
            || !agent(t.transfer.from)
            || !agent(t.transfer.to)
            || !coin(t.transfer.amount.resource)
        {
            return Err("invalid scheduled cashflow".into());
        }
        t.transfer.effects()?;
    }
    Ok(())
}
pub fn validate(world: &World, state: &State) -> Result<(), String> {
    if world.collection_policy == finance::CollectionPolicy::Proportional {
        let currencies: BTreeSet<_> = world
            .activities
            .coin_payments
            .values()
            .map(|a| a.resource)
            .collect();
        if world
            .agreements
            .iter()
            .chain(&world.access_offers)
            .any(|a| {
                world.activities.coin_payments.contains_key(&a.id)
                    && currencies.contains(&a.payment.resource)
            })
        {
            return Err("alternative payment routes cannot form currency chains".into());
        }
    }
    // Cash asset transfers use the same ownership/value registry without
    // activating a lending driver. All other financial state still needs one.
    let ownership_only = Book {
        owners: state.credit.owners.clone(),
        values: state.credit.values.clone(),
        ..Book::default()
    };
    if !enabled(world) && state.credit != ownership_only {
        return Err("credit book without accepted lending configuration".into());
    }
    validate_purchase(world, state)?;
    if world
        .ownership_rights
        .iter()
        .any(|id| !world.rights.iter().any(|r| r.id == *id))
    {
        return Err("unknown ownership-following right".into());
    }
    if (!world.lending.is_empty() || !world.recovery.proceedings.is_empty())
        && !crate::acquisition::search_composition(world)
        && (world.competition.is_some()
            || world.priority == Priority::ConsequenceAware
            || (world.market.is_none()
                && (!world.offers.is_empty() || !world.access_offers.is_empty())))
    {
        return Err("general loans require a composed acquisition driver for this search/market configuration".into());
    }
    if enabled(world) && world.town_market.is_some() && world.production_market.is_some() {
        return Err("town lending does not yet compose with joint production".into());
    }
    let agent = |id| world.agents.iter().any(|a| a.id == id);
    let stock = |id| {
        world
            .resources
            .iter()
            .any(|r| r.id == id && r.kind == ResourceKind::Stock)
    };
    let mut ids: BTreeSet<_> = world
        .credit
        .iter()
        .flat_map(|c| c.offers.iter().map(|o| o.id))
        .collect();
    for a in &world.lending {
        if !ids.insert(a.id)
            || !agent(a.debtor)
            || !agent(a.terms.creditor)
            || a.debtor == a.terms.creditor
            || !stock(a.terms.denomination)
            || a.principal <= 0
            || a.principal > a.terms.max_principal
            || a.terms.monthly_rate_bps > RATE_SCALE as u32
            || a.terms.term_months == 0
            || a.terms.term_months > MAX_TERM_MONTHS
            || a.terms.grace_months > MAX_TERM_MONTHS
            || a.month == 0
            || a.month > u32::MAX - MAX_TERM_MONTHS - 1
            || a.collateral.as_ref().is_some_and(|c| {
                *world
                    .storage
                    .weights
                    .get(&a.terms.denomination)
                    .unwrap_or(&0)
                    != 0
                    || !c.pledged
                    || !c.settlement.is_valid()
                    || !world.assets.iter().any(|x| x.id == c.asset)
                    || matches!(c.settlement, CollateralSettlement::ResaleProceeds { .. })
            })
        {
            return Err("invalid general lending agreement".into());
        }
    }
    crate::recovery::validate(world, state)?;
    ids.extend(world.recovery.guarantees.iter().map(|g| g.recourse));
    let assets: BTreeSet<_> = world.assets.iter().map(|a| a.id).collect();
    for (&asset, &who) in &state.credit.owners {
        if !assets.contains(&asset) || !agent(who) {
            return Err("invalid asset owner override".into());
        }
    }
    for (&asset, &value) in &state.credit.values {
        if !assets.contains(&asset) || value <= 0 {
            return Err("invalid asset carrying value".into());
        }
    }
    crate::resale::validate(world, state)?;
    let mut pledged = BTreeMap::new();
    for (&id, l) in &state.credit.loans {
        if id != l.id
            || !ids.contains(&id)
            || !agent(l.creditor)
            || !agent(l.debtor)
            || l.creditor == l.debtor
            || !stock(l.denomination)
            || l.original_principal <= 0
            || l.principal < 0
            || l.principal > l.original_principal
            || l.interest < 0
            || !(0..RATE_SCALE).contains(&l.interest_remainder)
            || l.term_months == 0
            || l.term_months > MAX_TERM_MONTHS
            || l.monthly_rate_bps > RATE_SCALE as u32
            || l.opened == 0
            || l.opened > state.month
            || l.last_accrued < l.opened
            || l.last_accrued > state.month
            || l.collateral.as_ref().is_some_and(|c| {
                !assets.contains(&c.asset)
                    || !c.settlement.is_valid()
                    || (c.pledged
                        && (pledged.insert(c.asset, (c, l.denomination)).is_some_and(
                            |(prior, denomination)| {
                                !compatible_liens(c, l.denomination, prior, denomination)
                            },
                        ) || owner(world, state, c.asset) != Some(l.debtor)))
                    || (c.pledged && !matches!(l.status, Status::Active | Status::Stayed))
                    || (l.status == Status::Active && !c.pledged)
            })
            || (l.status == Status::PendingSale && l.collateral.is_none())
            || matches!(l.status, Status::Repaid | Status::Discharged) != (l.debt()? == 0)
        {
            return Err("invalid loan book".into());
        }
    }
    Ok(())
}

pub(crate) fn tx(cause: String, effects: Vec<Effect>) -> Transaction {
    Transaction {
        cause,
        effects,
        process: None,
        technique_use: None,
        trade: None,
        stock_trade: None,
        forward: None,
        delivery: None,
        royalty: None,
    }
}
pub(crate) fn transfer(
    out: &mut Boundary,
    budgets: &mut BTreeMap<Account, i32>,
    from: AgentId,
    to: AgentId,
    coin: ResourceId,
    quantity: i32,
) -> Result<(), String> {
    if quantity == 0 || from == to {
        return Ok(());
    }
    let available = budgets.get(&(from, coin)).copied().unwrap_or(0);
    let effects = finance::exchange_payment(from, to, Amount::new(coin, quantity), available)?;
    budgets.insert((from, coin), available - quantity);
    out.transactions
        .push(tx("credit cash transfer".into(), effects));
    Ok(())
}
fn advances(
    world: &World,
    state: &State,
    out: &mut Boundary,
    execution: &mut finance::Execution,
    pooling: &mut crate::households::income_reservations::Reservations,
) -> Result<(), String> {
    let mut requests: Vec<_> = world
        .lending
        .iter()
        .filter(|a| a.month == state.month)
        .collect();
    requests.sort_by_key(|a| a.id);
    for a in requests {
        if out.after.loans.contains_key(&a.id) {
            continue;
        }
        let creditor = a.terms.creditor;
        let reason = if crate::recovery::active(world, &out.after, a.debtor).is_some()
            || crate::recovery::active(world, &out.after, creditor).is_some()
            || [a.debtor, creditor]
                .iter()
                .any(|id| state.terminal.contains_key(id))
            || !crate::laws::evaluate_terms(
                world,
                state,
                a.debtor,
                crate::laws::Terms::Loan {
                    monthly_rate_bps: a.terms.monthly_rate_bps,
                },
            )
            .allowed
            || !crate::opportunities::permits(
                world,
                state,
                creditor,
                crate::opportunities::Action::Lend,
            ) {
            Some(Rejection::Ineligible)
        } else if a.collateral.as_ref().is_some_and(|c| {
            owner(world, state, c.asset) != Some(a.debtor)
                || out.after.loans.values().any(|l| {
                    l.collateral.as_ref().is_some_and(|p| {
                        p.pledged
                            && p.asset == c.asset
                            && !compatible_liens(c, a.terms.denomination, p, l.denomination)
                    })
                })
        }) {
            Some(Rejection::UnavailableAsset)
        } else {
            None
        };
        if let Some(reason) = reason {
            out.events.push(Event::Rejected {
                offer: a.id,
                reason,
            });
            continue;
        }
        let amount = Amount::new(a.terms.denomination, a.principal);
        let leg = finance::Transfer {
            from: creditor,
            to: a.debtor,
            amount: amount.clone(),
        };
        let mut reserved = pooling.clone();
        let effects = if reserved.reserve_unpooled(world, &leg.effects()?).is_ok() {
            execution.exchange(world, &[leg])
        } else {
            Err("advance exceeds pooled storage".into())
        };
        let Ok(effects) = effects else {
            out.events.push(Event::Rejected {
                offer: a.id,
                reason: Rejection::Funding,
            });
            continue;
        };
        *pooling = reserved;
        out.transactions
            .push(tx("contract loan advance".into(), effects));
        out.after.loans.insert(a.id, Loan::accepted(a));
        out.events.push(Event::Advanced {
            loan: a.id,
            creditor,
            debtor: a.debtor,
            amount,
        });
    }
    Ok(())
}
fn purchase(
    world: &World,
    state: &State,
    c: &Config,
    out: &mut Boundary,
    budgets: &mut BTreeMap<Account, i32>,
) -> Result<(), String> {
    let a = &c.application;
    if a.month != state.month || out.after.loans.contains_key(&a.offer) {
        return Ok(());
    }
    let o = c
        .offers
        .iter()
        .find(|o| o.id == a.offer)
        .ok_or("missing purchase offer")?;
    let principal = o
        .sale
        .price
        .quantity
        .checked_sub(a.downpayment)
        .ok_or("purchase amount overflow")?;
    let coin = o.loan.denomination;
    let reason = if !crate::laws::evaluate_terms(
        world,
        state,
        a.buyer,
        crate::laws::Terms::FinancedPurchase {
            monthly_rate_bps: o.loan.monthly_rate_bps,
        },
    )
    .allowed
        || [a.buyer, o.sale.seller, o.loan.creditor].iter().any(|id| {
            !purchase_party(world, state, *id)
                || crate::recovery::active(world, &out.after, *id).is_some()
        })
        || a.buyer == o.sale.seller
        || a.buyer == o.loan.creditor
    {
        Some(Rejection::Ineligible)
    } else if owner(world, state, o.sale.asset) != Some(o.sale.seller)
        || out.after.loans.values().any(|l| {
            l.collateral
                .as_ref()
                .is_some_and(|c| c.pledged && c.asset == o.sale.asset)
        })
    {
        Some(Rejection::UnavailableAsset)
    } else if a.downpayment < o.minimum_downpayment
        || principal <= 0
        || principal > o.loan.max_principal
        || budgets.get(&(a.buyer, coin)).copied().unwrap_or(0) < a.downpayment
    {
        Some(Rejection::Downpayment)
    } else if budgets.get(&(o.loan.creditor, coin)).copied().unwrap_or(0) < principal {
        Some(Rejection::Funding)
    } else {
        None
    };
    if let Some(reason) = reason {
        out.events.push(Event::Rejected {
            offer: o.id,
            reason,
        });
        return Ok(());
    }
    transfer(out, budgets, a.buyer, o.sale.seller, coin, a.downpayment)?;
    // When seller == lender the advance is applied to its own sale: a
    // receivable replaces the financed asset value, with no self-transfer.
    transfer(
        out,
        budgets,
        o.loan.creditor,
        o.sale.seller,
        coin,
        principal,
    )?;
    transfer_attachments(world, state, out, o.sale.asset, a.buyer);
    out.after.owners.insert(o.sale.asset, a.buyer);
    out.after.values.insert(o.sale.asset, o.sale.price.quantity);
    out.after.loans.insert(
        o.id,
        Loan::accepted(&Advance {
            id: o.id,
            debtor: a.buyer,
            terms: o.loan.clone(),
            principal,
            month: state.month,
            collateral: Some(o.collateral.clone()),
            priority: o.collateral.priority,
        }),
    );
    out.events.push(Event::Purchased {
        offer: o.id,
        asset: o.sale.asset,
        buyer: a.buyer,
        price: o.sale.price.quantity,
        downpayment: a.downpayment,
        advance: principal,
    });
    Ok(())
}
/// Transfer control and future output, never elapsed work or sunk inputs.
/// Personal need goals and personal future labor do not transfer with the asset.
pub(crate) fn transfer_attachments(
    world: &World,
    state: &State,
    out: &mut Boundary,
    asset: AssetId,
    to: AgentId,
) {
    out.attachments
        .extend(attachment_changes(world, state, asset, to));
}

pub(crate) fn attachment_changes(
    world: &World,
    state: &State,
    asset: AssetId,
    to: AgentId,
) -> Vec<ProcessChange> {
    state
        .processes
        .values()
        .filter(|p| {
            p.status == crate::model::Status::Active
                && p.asset == Some(asset)
                && p.right.is_some_and(|id| follows_owner(world, id))
        })
        .map(|p| {
            let mut after = p.clone();
            after.operator = to;
            after.beneficiary = to;
            after.goal = None;
            ProcessChange {
                before: Some(p.clone()),
                after,
            }
        })
        .collect()
}
pub fn follows_owner(world: &World, right: u32) -> bool {
    world.ownership_rights.contains(&right)
        || world
            .credit
            .as_ref()
            .is_some_and(|c| c.attached_rights.contains(&right))
}

fn accrue(loan: &mut Loan, month: u32) -> Result<i32, String> {
    if loan.last_accrued.checked_add(1) != Some(month) {
        return Err("stale monthly accrual boundary".into());
    }
    let accrued =
        i64::from(loan.principal) * i64::from(loan.monthly_rate_bps) + loan.interest_remainder;
    let interest = i32::try_from(accrued / RATE_SCALE).map_err(|_| "interest overflow")?;
    loan.interest = loan
        .interest
        .checked_add(interest)
        .ok_or("interest balance overflow")?;
    loan.interest_remainder = accrued % RATE_SCALE;
    loan.last_accrued = month;
    Ok(interest)
}

/// Preview the current collectible claim without mutating the loan book. At Open
/// or before Due collection this includes this month's accrual; after Due it does
/// not accrue twice. Future installments and stayed claims are not spendable dues.
pub(crate) fn current_claim(
    world: &World,
    state: &State,
    loan: &Loan,
) -> Result<Option<finance::CollectionRequest>, String> {
    let native = crate::recovery::native_performance(world, &state.credit, loan);
    if (crate::recovery::active(world, &state.credit, loan.debtor).is_some()
        || loan.status == Status::Stayed)
        && !native
        || matches!(
            loan.status,
            Status::Repaid | Status::Discharged | Status::PendingSale
        )
        || state.month <= loan.opened
    {
        return Ok(None);
    }
    let mut loan = loan.clone();
    if loan.status == Status::Active && loan.last_accrued < state.month {
        accrue(&mut loan, state.month)?;
    }
    let mut claim = loan.claim(state.month)?;
    // A later guarantee call may extend an older recourse loan. Its new
    // principal becomes collectible next month, just like the initial advance.
    claim.transfer.amount.quantity =
        claim
            .transfer
            .amount
            .quantity
            .saturating_sub(crate::recovery::current_recourse(
                world,
                &state.credit,
                loan.id,
                state.month,
            ));
    let contract = finance::ContractId::Loan(loan.id);
    Ok(Some(finance::CollectionRequest {
        contract,
        rank: world
            .claim_priorities
            .get(&contract)
            .copied()
            .unwrap_or(loan.priority),
        claim,
    }))
}

pub(crate) fn current_dues(
    world: &World,
    state: &State,
    agent: AgentId,
) -> Result<BTreeMap<ResourceId, i128>, String> {
    let mut result = BTreeMap::new();
    for loan in state.credit.loans.values().filter(|l| l.debtor == agent) {
        if let Some(request) = current_claim(world, state, loan)? {
            *result.entry(loan.denomination).or_default() +=
                i128::from(request.claim.outstanding());
        }
    }
    Ok(result)
}

fn due(
    world: &World,
    state: &State,
    out: &mut Boundary,
    budgets: &mut BTreeMap<Account, i32>,
) -> Result<(), String> {
    let mut execution = finance::Execution::opening(world, state);
    execution.available = budgets.clone();
    crate::recovery::open(world, state, out)?;
    let protected = crate::commitments::protected_stock(world, state)?;
    let mut collection_state = state.clone();
    collection_state.credit = out.after.clone();
    let mut requests = Vec::new();
    for loan in out.after.loans.values() {
        if let Some(request) = current_claim(world, &collection_state, loan)? {
            requests.push(request);
        }
    }
    requests.extend(crate::commitments::current_claims(
        world,
        &collection_state,
        &out.after,
    )?);
    let grants =
        finance::collection::allocate(world, &collection_state, &execution, &protected, &requests)?;
    let mut order: Vec<_> = out
        .after
        .loans
        .values()
        .map(|l| {
            (
                world
                    .claim_priorities
                    .get(&finance::ContractId::Loan(l.id))
                    .copied()
                    .unwrap_or(l.priority),
                finance::ContractId::Loan(l.id),
            )
        })
        .collect();
    order.extend(crate::commitments::active(world, state).map(|a| {
        (
            world
                .claim_priorities
                .get(&finance::ContractId::Land(a.id))
                .copied()
                .unwrap_or(finance::DEFAULT_CLAIM_RANK),
            finance::ContractId::Land(a.id),
        )
    }));
    order.sort();
    let account_for = |contract: finance::ContractId| -> Account {
        match contract {
            finance::ContractId::Loan(id) => {
                let loan = &out.after.loans[&id];
                (loan.debtor, loan.denomination)
            }
            finance::ContractId::Land(id) => {
                let agreement = crate::commitments::active(world, state)
                    .find(|a| a.id == id)
                    .unwrap();
                (agreement.debtor, agreement.payment.resource)
            }
            finance::ContractId::Forward(_)
            | finance::ContractId::Wages(_)
            | finance::ContractId::Guarantee(_) => unreachable!(),
        }
    };
    let accounts: BTreeMap<_, _> = order
        .iter()
        .map(|(_, contract)| (*contract, account_for(*contract)))
        .collect();
    let mut reserved = BTreeMap::<Account, i32>::new();
    if let Some(grants) = &grants {
        for (contract, quantity) in &grants.native {
            let value = reserved.entry(accounts[contract]).or_default();
            *value = value
                .checked_add(*quantity)
                .ok_or("collection reservation overflow")?;
        }
    }
    if let Some(grants) = &grants {
        for (contract, (resource, amount, _)) in &grants.alternative {
            let account = (accounts[contract].0, *resource);
            let value = reserved.entry(account).or_default();
            *value = value
                .checked_add(*amount)
                .ok_or("alternative reservation overflow")?;
        }
    }
    for (rank, contract) in order {
        if let Some(grants) = &grants {
            *reserved.entry(accounts[&contract]).or_default() -=
                grants.get(&contract).copied().unwrap_or(0);
        }
        if let Some((resource, amount, _)) =
            grants.as_ref().and_then(|g| g.alternative.get(&contract))
        {
            *reserved
                .entry((accounts[&contract].0, *resource))
                .or_default() -= amount;
        }
        let id = match contract {
            finance::ContractId::Loan(id) => id,
            finance::ContractId::Land(id) => {
                let settlement = if let Some(grants) = &grants {
                    crate::commitments::evaluate_allocated(
                        world,
                        &collection_state,
                        &mut execution,
                        id,
                        grants,
                        &protected,
                    )?
                } else {
                    crate::commitments::evaluate_selected(
                        world,
                        &collection_state,
                        &mut execution,
                        Some(id),
                    )?
                };
                out.collections.extend(settlement.collections.clone());
                collection_state.obligations = settlement.obligations.clone();
                out.transactions.extend(settlement.transactions.clone());
                let all = out
                    .commitments
                    .get_or_insert_with(|| crate::commitments::Settlement {
                        collections: vec![],
                        policy: settlement.policy,
                        protected: settlement.protected.clone(),
                        obligations: settlement.obligations.clone(),
                        transactions: vec![],
                    });
                all.obligations = settlement.obligations;
                all.collections.extend(settlement.collections);
                all.transactions.extend(settlement.transactions);
                *budgets = execution.available.clone();
                continue;
            }
            finance::ContractId::Forward(_)
            | finance::ContractId::Wages(_)
            | finance::ContractId::Guarantee(_) => {
                unreachable!("claims collect at their own boundary")
            }
        };
        let mut l = out.after.loans[&id].clone();
        if current_claim(world, &collection_state, &l)?.is_none() {
            continue;
        }
        if l.status == Status::Active {
            let interest = accrue(&mut l, state.month)?;
            out.events.push(Event::Accrued {
                loan: id,
                opening_principal: l.principal,
                interest,
            });
        }
        let payment = execution.pay_protected(
            world,
            state.month,
            &l.claim(state.month)?,
            protected
                .get(&(l.debtor, l.denomination))
                .copied()
                .unwrap_or(0)
                .max(grants.as_ref().map_or(0, |grants| {
                    execution
                        .available
                        .get(&(l.debtor, l.denomination))
                        .copied()
                        .unwrap_or(0)
                        - grants.get(&contract).copied().unwrap_or(0)
                })),
        )?;
        let paid = payment.paid;
        out.collections.push(finance::CollectionReceipt {
            contract,
            rank,
            debtor: l.debtor,
            creditor: l.creditor,
            requested: Amount::new(l.denomination, payment.requested),
            allocated: grants
                .as_ref()
                .map(|g| g.get(&contract).copied().unwrap_or(0)),
            paid,
        });
        if paid > 0 {
            out.transactions
                .push(tx("credit cash transfer".into(), payment.effects));
            out.events.push(Event::Paid {
                loan: id,
                interest: paid.min(l.interest),
                principal: (paid - l.interest).max(0),
            });
            l.apply_payment(paid);
        }
        *budgets = execution.available.clone();
        let unpaid = l.due(state.month)?;
        if unpaid == 0 {
            l.first_unpaid = None;
        } else {
            let since = *l.first_unpaid.get_or_insert(state.month);
            out.events.push(Event::Arrears {
                loan: id,
                amount: unpaid,
                since,
            });
            if l.status == Status::Active
                && state.month - since >= l.grace_months
                && let Some(collateral) = &mut l.collateral
            {
                if collateral.settlement == CollateralSettlement::AuthorizedLiquidation {
                    // A missed installment is not authorization to liquidate.
                    out.after.loans.insert(id, l);
                    continue;
                }
                if let CollateralSettlement::ResaleProceeds { minimum_price } =
                    collateral.settlement
                {
                    transfer_attachments(world, state, out, collateral.asset, l.creditor);
                    out.after.owners.insert(collateral.asset, l.creditor);
                    collateral.pledged = false;
                    l.status = Status::PendingSale;
                    out.after.pending_sales.insert(
                        id,
                        crate::resale::PendingSale {
                            listed: state.month,
                        },
                    );
                    out.events.push(Event::RepossessedForSale {
                        loan: id,
                        asset: collateral.asset,
                        minimum_price,
                    });
                    out.after.loans.insert(id, l);
                    continue;
                }
                let CollateralSettlement::FixedValue { value } = collateral.settlement else {
                    unreachable!()
                };
                let debt = l.principal.checked_add(l.interest).ok_or("debt overflow")?;
                let credit = value.min(debt);
                let surplus = (value - debt).max(0);
                if budgets
                    .get(&(l.creditor, l.denomination))
                    .copied()
                    .unwrap_or(0)
                    .saturating_sub(
                        reserved
                            .get(&(l.creditor, l.denomination))
                            .copied()
                            .unwrap_or(0),
                    )
                    .saturating_sub(if grants.is_some() {
                        protected
                            .get(&(l.creditor, l.denomination))
                            .copied()
                            .unwrap_or(0)
                    } else {
                        0
                    })
                    < surplus
                {
                    out.events
                        .push(Event::EnforcementDeferred { loan: id, surplus });
                } else {
                    transfer(out, budgets, l.creditor, l.debtor, l.denomination, surplus)?;
                    transfer_attachments(world, state, out, collateral.asset, l.creditor);
                    out.after.owners.insert(collateral.asset, l.creditor);
                    out.after.values.insert(collateral.asset, value);
                    collateral.pledged = false;
                    l.status = Status::Enforced;
                    l.apply_payment(credit);
                    out.events.push(Event::Enforced {
                        loan: id,
                        value,
                        debt_credit: credit,
                        surplus,
                        remaining_debt: l.debt()?,
                    });
                }
            }
        }
        execution.available = budgets.clone();
        out.after.loans.insert(id, l);
    }
    crate::recovery::guarantees(world, state, out, &mut execution)?;
    crate::delivery_relief::apply(world, state, out);
    crate::claim_relief::apply(world, state, out)?;
    crate::recovery::distribute(world, state, out, &mut execution)?;
    *budgets = execution.available;
    Ok(())
}
pub fn evaluate(world: &World, state: &State) -> Result<Option<Boundary>, String> {
    validate(world, state)?;
    if !enabled(world) {
        return Ok(None);
    }
    if !matches!(state.phase, Phase::Open | Phase::Due | Phase::Acquire) {
        return Ok(None);
    }
    let mut out = Boundary {
        equipment: BTreeMap::new(),
        employment: None,
        forward_changes: BTreeMap::new(),
        recovery: vec![],
        collections: vec![],
        commitments: None,
        production_plan: None,
        stock_sale: None,
        decision: None,
        attachments: vec![],
        after: state.credit.clone(),
        events: vec![],
        transactions: vec![],
    };
    let mut budgets = state.balances.clone();
    match state.phase {
        Phase::Open => {
            let mut opening = Batch::empty(state);
            crate::simulation::Simulation::open(world, state, &mut opening);
            out.transactions.extend(opening.transactions);
            if state.month == 1 {
                for e in world.credit.iter().flat_map(|c| &c.endowments) {
                    out.transactions.push(tx(
                        "initial coin endowment".into(),
                        vec![Effect {
                            account: (e.agent, e.amount.resource),
                            delta: e.amount.quantity,
                        }],
                    ));
                    out.events.push(Event::Endowed {
                        agent: e.agent,
                        amount: e.amount.clone(),
                    });
                }
            }
            for t in world
                .credit
                .iter()
                .flat_map(|c| &c.transfers)
                .filter(|t| t.month == state.month)
            {
                transfer(
                    &mut out,
                    &mut budgets,
                    t.transfer.from,
                    t.transfer.to,
                    t.transfer.amount.resource,
                    t.transfer.amount.quantity,
                )?;
                out.events.push(Event::Cashflow {
                    from: t.transfer.from,
                    to: t.transfer.to,
                    amount: t.transfer.amount.clone(),
                });
            }
        }
        Phase::Due => due(world, state, &mut out, &mut budgets)?,
        Phase::Acquire => {
            let mut execution = finance::Execution::opening(world, state);
            execution.available = budgets.clone();
            let mut pooling = crate::recovery::sales(world, state, &mut out, &mut execution)?;
            advances(world, state, &mut out, &mut execution, &mut pooling)?;
            budgets = execution.available;
            if let Some(c) = &world.credit {
                let funding_start = out.transactions.len();
                let should_purchase = match &c.purchase_policy {
                    crate::borrowing::Policy::Scripted => true,
                    crate::borrowing::Policy::Decline => false,
                    crate::borrowing::Policy::Compare(_)
                        if c.application.month == state.month
                            && !state.credit.loans.contains_key(&c.application.offer) =>
                    {
                        let decision = crate::borrowing::evaluate(world, state)?;
                        let accept = decision.accept;
                        out.decision = Some(decision);
                        accept
                    }
                    crate::borrowing::Policy::Compare(_) => false,
                };
                if should_purchase {
                    purchase(world, state, c, &mut out, &mut budgets)?;
                }
                crate::resale::settle(world, state, &mut out, &mut budgets)?;
                for transaction in &out.transactions[funding_start..] {
                    pooling.reserve_unpooled(world, &transaction.effects)?;
                }
                crate::stock_sale::settle(world, state, &mut out, &mut budgets, &mut pooling)?;
            }
            crate::recovery::admission::apply(world, state, &mut out)?;
        }
        _ => {}
    }
    Ok(Some(out))
}
pub fn validate_batch(world: &World, state: &State, batch: &Batch) -> Result<(), String> {
    let expected = evaluate(world, state)?;
    if batch.credit != expected
        || (expected.is_some()
            && !(state.phase == Phase::Acquire && crate::acquisition::search_composition(world))
            && batch.production_plan != expected.as_ref().and_then(|e| e.production_plan.clone()))
        || expected
            .as_ref()
            .is_some_and(|e| e.transactions != batch.transactions)
    {
        return Err("missing or altered credit settlement".into());
    }
    Ok(())
}
/// Controlled cashflows are explicit treasury transfers, not production revenue.
pub fn scenario(case: &str) -> Result<(World, State), String> {
    use crate::scenario::{PERSON, PLOT, STATE_AGENT, TOKEN};
    let (mut w, mut s) = crate::scenario::baseline();
    w.participants.clear();
    w.definitions.clear();
    w.rights.clear();
    w.resources = vec![Resource {
        id: TOKEN,
        name: "coin ticks (100 per coin)".into(),
        kind: ResourceKind::Stock,
    }];
    s.balances.clear();
    let income = |month, quantity| ScheduledTransfer {
        month,
        transfer: finance::Transfer {
            from: STATE_AGENT,
            to: PERSON,
            amount: Amount::new(TOKEN, quantity),
        },
    };
    let transfers = match case {
        "repaid" => (2..=5).map(|m| income(m, MONTHLY_INCOME_TICKS)).collect(),
        "recovered" => vec![
            income(3, 2 * MONTHLY_INCOME_TICKS),
            income(4, MONTHLY_INCOME_TICKS),
            income(5, MONTHLY_INCOME_TICKS),
        ],
        "downpayment" | "default" | "surplus" => vec![],
        _ => return Err("unknown credit scenario".into()),
    };
    w.credit = Some(Config {
        stock_sales: None,
        purchase_policy: crate::borrowing::Policy::Scripted,
        resale_buyer: None,
        attached_rights: BTreeSet::new(),
        offers: vec![Offer {
            id: 1,
            sale: Sale {
                asset: PLOT,
                seller: STATE_AGENT,
                price: Amount::new(TOKEN, PRICE_TICKS),
            },
            loan: LoanOffer {
                creditor: STATE_AGENT,
                denomination: TOKEN,
                max_principal: PRICE_TICKS - DOWNPAYMENT_TICKS,
                monthly_rate_bps: MONTHLY_RATE_BPS,
                term_months: TERM_MONTHS,
                grace_months: GRACE_MONTHS,
            },
            minimum_downpayment: DOWNPAYMENT_TICKS,
            collateral: Collateral {
                asset: PLOT,
                priority: 1,
                settlement: CollateralSettlement::FixedValue {
                    value: if case == "surplus" {
                        PRICE_TICKS
                    } else {
                        DISTRESSED_VALUE
                    },
                },
                pledged: true,
            },
        }],
        application: Application {
            offer: 1,
            buyer: PERSON,
            month: 1,
            downpayment: DOWNPAYMENT_TICKS,
        },
        endowments: vec![
            Endowment {
                agent: PERSON,
                amount: Amount::new(
                    TOKEN,
                    if case == "downpayment" {
                        DOWNPAYMENT_TICKS - 1
                    } else {
                        DOWNPAYMENT_TICKS
                    },
                ),
            },
            Endowment {
                agent: STATE_AGENT,
                amount: Amount::new(TOKEN, TREASURY_TICKS),
            },
        ],
        transfers,
    });
    Ok((w, s))
}

/// Controlled crop attachment: same debt and valuation whether maintained or not.
/// The state capacity is an explicit supplied service, not labor created by title.
pub fn crop_scenario(maintain: bool) -> Result<(World, State), String> {
    use crate::scenario::{GRAIN, GROW, LABOR, PERSON, SEED, STATE_AGENT};
    let (mut world, mut state) = scenario("default")?;
    let (catalog, _) = crate::scenario::baseline();
    world.resources.extend(
        catalog
            .resources
            .into_iter()
            .filter(|r| [GRAIN, SEED, LABOR].contains(&r.id)),
    );
    world.definitions = catalog
        .definitions
        .into_iter()
        .filter(|d| d.id == GROW)
        .collect();
    world.definitions[0]
        .outputs
        .push(Amount::new(SEED, CROP_INITIAL_SEED));
    world.rights = catalog.rights;
    world.credit.as_mut().unwrap().attached_rights = world.rights.iter().map(|r| r.id).collect();
    world.participants = vec![
        Participant {
            agent: PERSON,
            capacity: Amount::new(LABOR, CROP_PROVIDED_LABOR),
            needs: vec![],
        },
        Participant {
            agent: STATE_AGENT,
            capacity: Amount::new(LABOR, if maintain { CROP_PROVIDED_LABOR } else { 0 }),
            needs: vec![],
        },
    ];
    world.scheduled_starts.push(ScheduledStart {
        month: 1,
        agent: PERSON,
        definition: GROW,
    });
    state.balances.insert((PERSON, SEED), CROP_INITIAL_SEED);
    Ok((world, state))
}
