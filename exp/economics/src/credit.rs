//! Financed asset purchases, monthly noncompounding debt and collateral settlement.
//! Scoped financial experiment; ownership and debt share one authoritative book.
use crate::{finance, model::*};
use std::collections::{BTreeMap, BTreeSet};

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
    FixedValue { value: i32 },
    ResaleProceeds { minimum_price: i32 },
}
impl CollateralSettlement {
    fn is_valid(&self) -> bool {
        match *self {
            Self::FixedValue { value } => value > 0,
            Self::ResaleProceeds { minimum_price } => minimum_price > 0,
        }
    }
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
    pub collateral: Collateral,
}
impl Loan {
    pub fn debt(&self) -> Result<i32, String> {
        self.principal
            .checked_add(self.interest)
            .ok_or("debt overflow".into())
    }
    pub fn principal_due(&self, month: u32) -> i32 {
        if self.status == Status::Enforced {
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
            self.collateral.pledged = false;
            self.first_unpaid = None;
            self.interest_remainder = 0;
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Book {
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
    pub production_plan: Option<Box<Batch>>,
    pub stock_sale: Option<crate::stock_sale::Receipt>,
    pub decision: Option<crate::borrowing::Decision>,
    pub attachments: Vec<ProcessChange>,
    pub after: Book,
    pub events: Vec<Event>,
    pub transactions: Vec<Transaction>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BalanceSheet {
    /// Memo only: controlled for realization, excluded from lender equity.
    pub collateral_in_custody: i64,
    /// Subset of assets: borrower interest awaiting actual realization.
    pub assets_awaiting_sale: i64,
    pub coins: i64,
    pub assets: i64,
    pub principal_receivable: i64,
    pub interest_receivable: i64,
    pub principal_payable: i64,
    pub interest_payable: i64,
}
impl BalanceSheet {
    pub fn equity(&self) -> i64 {
        self.coins + self.assets + self.principal_receivable + self.interest_receivable
            - self.principal_payable
            - self.interest_payable
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
pub fn balance_sheet(
    world: &World,
    state: &State,
    agent: AgentId,
    coin: ResourceId,
) -> BalanceSheet {
    let mut b = BalanceSheet {
        coins: i64::from(state.balance(agent, coin)),
        ..Default::default()
    };
    if let Some(c) = &world.credit {
        let mut seen = BTreeSet::new();
        for o in &c.offers {
            let pending =
                state.credit.loans.values().find(|l| {
                    l.status == Status::PendingSale && l.collateral.asset == o.sale.asset
                });
            let economic_owner = pending
                .map(|l| l.debtor)
                .or_else(|| owner(world, state, o.sale.asset));
            if o.sale.price.resource == coin && pending.is_some_and(|l| l.creditor == agent) {
                b.collateral_in_custody += i64::from(
                    *state
                        .credit
                        .values
                        .get(&o.sale.asset)
                        .unwrap_or(&o.sale.price.quantity),
                );
            }
            if o.sale.price.resource == coin && pending.is_some_and(|l| l.debtor == agent) {
                b.assets_awaiting_sale += i64::from(
                    *state
                        .credit
                        .values
                        .get(&o.sale.asset)
                        .unwrap_or(&o.sale.price.quantity),
                );
            }
            if o.sale.price.resource == coin
                && seen.insert(o.sale.asset)
                && economic_owner == Some(agent)
            {
                b.assets += i64::from(
                    *state
                        .credit
                        .values
                        .get(&o.sale.asset)
                        .unwrap_or(&o.sale.price.quantity),
                );
            }
        }
    }
    for l in state
        .credit
        .loans
        .values()
        .filter(|l| l.denomination == coin)
    {
        if l.debtor == agent {
            b.principal_payable += i64::from(l.principal);
            b.interest_payable += i64::from(l.interest);
        }
        if l.creditor == agent {
            b.principal_receivable += i64::from(l.principal);
            b.interest_receivable += i64::from(l.interest);
        }
    }
    b
}
/// Visible financed offers; application feasibility and settlement remain separate.
pub fn discover<'a>(world: &'a World, state: &State, buyer: AgentId) -> Vec<&'a Offer> {
    world.credit.as_ref().map_or(vec![], |c| {
        c.offers
            .iter()
            .filter(|o| {
                crate::opportunities::permits(
                    world,
                    state,
                    buyer,
                    crate::opportunities::Action::FinancedPurchase,
                ) && buyer != o.sale.seller
                    && buyer != o.loan.creditor
                    && world.agents.iter().any(|a| a.id == buyer)
                    && !state.terminal.contains_key(&buyer)
                    && !state.terminal.contains_key(&o.sale.seller)
                    && !state.terminal.contains_key(&o.loan.creditor)
                    && !state.credit.loans.contains_key(&o.id)
                    && !state
                        .credit
                        .loans
                        .values()
                        .any(|l| l.collateral.pledged && l.collateral.asset == o.sale.asset)
                    && owner(world, state, o.sale.asset) == Some(o.sale.seller)
            })
            .collect()
    })
}
pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let Some(c) = &world.credit else {
        return if state.credit == Book::default() {
            Ok(())
        } else {
            Err("credit book without configuration".into())
        };
    };
    if world.negotiation.is_some() && c.stock_sales.as_ref().is_some_and(|p| p.joint.is_some()) {
        return Err("joint production reservations do not yet compose with negotiation".into());
    }
    if world.transaction_policy.is_some() && c.resale_buyer.is_some() {
        return Err("permission-gated collateral resale is not yet supported".into());
    }
    crate::borrowing::validate(world)?;
    crate::stock_sale::validate(world, state)?;
    // Ownership-following production is supported. Other acquisition/collection
    // drivers still require shared funding and ownership rules.
    if !world.agreements.is_empty()
        || !world.access_offers.is_empty()
        || world.market.is_some()
        || world.competition.is_some()
        || world.pool_market.is_some()
        || !world.households.is_empty()
        || !world.offers.is_empty()
        || (!world.bids.is_empty() && c.stock_sales.is_none())
        || !world.issuance.is_empty()
        || !world.pools.is_empty()
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
    if world
        .rights
        .iter()
        .any(|r| !c.attached_rights.contains(&r.id))
        || c.attached_rights.iter().any(|id| {
            !world
                .rights
                .iter()
                .any(|r| r.id == *id && c.offers.iter().any(|o| o.sale.asset == r.asset))
        })
    {
        return Err("credit production requires explicit ownership-following rights".into());
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
    let mut pledged = BTreeSet::new();
    for (&id, l) in &state.credit.loans {
        if id != l.id
            || !ids.contains(&id)
            || !agent(l.creditor)
            || !agent(l.debtor)
            || l.creditor == l.debtor
            || !coin(l.denomination)
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
            || !assets.contains(&l.collateral.asset)
            || !l.collateral.settlement.is_valid()
            || (l.collateral.pledged
                && (!pledged.insert(l.collateral.asset)
                    || owner(world, state, l.collateral.asset) != Some(l.debtor)))
            || (l.status == Status::Active) != l.collateral.pledged
            || (l.status == Status::Repaid) != (l.debt()? == 0)
        {
            return Err("invalid loan book".into());
        }
    }
    Ok(())
}
fn tx(cause: String, effects: Vec<Effect>) -> Transaction {
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
    let reason = if !crate::opportunities::permits(
        world,
        state,
        a.buyer,
        crate::opportunities::Action::FinancedPurchase,
    ) || [a.buyer, o.sale.seller, o.loan.creditor]
        .iter()
        .any(|id| state.terminal.contains_key(id))
        || a.buyer == o.sale.seller
        || a.buyer == o.loan.creditor
    {
        Some(Rejection::Ineligible)
    } else if owner(world, state, o.sale.asset) != Some(o.sale.seller)
        || out
            .after
            .loans
            .values()
            .any(|l| l.collateral.pledged && l.collateral.asset == o.sale.asset)
    {
        Some(Rejection::UnavailableAsset)
    } else if a.downpayment < o.minimum_downpayment
        || principal <= 0
        || principal > o.loan.max_principal
        || state.balance(a.buyer, coin) < a.downpayment
    {
        Some(Rejection::Downpayment)
    } else if state.balance(o.loan.creditor, coin) < principal {
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
        Loan {
            id: o.id,
            creditor: o.loan.creditor,
            debtor: a.buyer,
            denomination: coin,
            original_principal: principal,
            principal,
            interest: 0,
            interest_remainder: 0,
            monthly_rate_bps: o.loan.monthly_rate_bps,
            opened: state.month,
            last_accrued: state.month,
            term_months: o.loan.term_months,
            grace_months: o.loan.grace_months,
            first_unpaid: None,
            status: Status::Active,
            collateral: o.collateral.clone(),
        },
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
    for p in state.processes.values().filter(|p| {
        p.status == crate::model::Status::Active
            && p.asset == Some(asset)
            && p.right.is_some_and(|id| follows_owner(world, id))
    }) {
        let mut after = p.clone();
        after.operator = to;
        after.beneficiary = to;
        after.goal = None;
        out.attachments.push(ProcessChange {
            before: Some(p.clone()),
            after,
        });
    }
}
pub fn follows_owner(world: &World, right: u32) -> bool {
    world
        .credit
        .as_ref()
        .is_some_and(|c| c.attached_rights.contains(&right))
}

fn due(
    world: &World,
    state: &State,
    out: &mut Boundary,
    budgets: &mut BTreeMap<Account, i32>,
) -> Result<(), String> {
    let ids: Vec<_> = out.after.loans.keys().copied().collect();
    for id in ids {
        let mut l = out.after.loans[&id].clone();
        if matches!(l.status, Status::Repaid | Status::PendingSale) || state.month <= l.opened {
            continue;
        }
        if l.status == Status::Active {
            if l.last_accrued.checked_add(1) != Some(state.month) {
                return Err("stale monthly accrual boundary".into());
            }
            let accrued =
                i64::from(l.principal) * i64::from(l.monthly_rate_bps) + l.interest_remainder;
            let interest = i32::try_from(accrued / RATE_SCALE).map_err(|_| "interest overflow")?;
            l.interest = l
                .interest
                .checked_add(interest)
                .ok_or("interest balance overflow")?;
            l.interest_remainder = accrued % RATE_SCALE;
            l.last_accrued = state.month;
            out.events.push(Event::Accrued {
                loan: id,
                opening_principal: l.principal,
                interest,
            });
        }
        let available = budgets
            .get(&(l.debtor, l.denomination))
            .copied()
            .unwrap_or(0);
        let paid = l
            .claim(state.month)?
            .payable(state.month, true, available, i32::MAX);
        if paid > 0 {
            transfer(out, budgets, l.debtor, l.creditor, l.denomination, paid)?;
            out.events.push(Event::Paid {
                loan: id,
                interest: paid.min(l.interest),
                principal: (paid - l.interest).max(0),
            });
            l.apply_payment(paid);
        }
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
            if l.status == Status::Active && state.month - since >= l.grace_months {
                if let CollateralSettlement::ResaleProceeds { minimum_price } =
                    l.collateral.settlement
                {
                    transfer_attachments(world, state, out, l.collateral.asset, l.creditor);
                    out.after.owners.insert(l.collateral.asset, l.creditor);
                    l.collateral.pledged = false;
                    l.status = Status::PendingSale;
                    out.after.pending_sales.insert(
                        id,
                        crate::resale::PendingSale {
                            listed: state.month,
                        },
                    );
                    out.events.push(Event::RepossessedForSale {
                        loan: id,
                        asset: l.collateral.asset,
                        minimum_price,
                    });
                    out.after.loans.insert(id, l);
                    continue;
                }
                let CollateralSettlement::FixedValue { value } = l.collateral.settlement else {
                    unreachable!()
                };
                let debt = l.debt()?;
                let credit = value.min(debt);
                let surplus = (value - debt).max(0);
                if budgets
                    .get(&(l.creditor, l.denomination))
                    .copied()
                    .unwrap_or(0)
                    < surplus
                {
                    out.events
                        .push(Event::EnforcementDeferred { loan: id, surplus });
                } else {
                    transfer(out, budgets, l.creditor, l.debtor, l.denomination, surplus)?;
                    transfer_attachments(world, state, out, l.collateral.asset, l.creditor);
                    out.after.owners.insert(l.collateral.asset, l.creditor);
                    out.after.values.insert(l.collateral.asset, value);
                    l.collateral.pledged = false;
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
        out.after.loans.insert(id, l);
    }
    Ok(())
}
pub fn evaluate(world: &World, state: &State) -> Result<Option<Boundary>, String> {
    validate(world, state)?;
    let Some(c) = &world.credit else {
        return Ok(None);
    };
    if !matches!(state.phase, Phase::Open | Phase::Due | Phase::Acquire) {
        return Ok(None);
    }
    let mut out = Boundary {
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
                for e in &c.endowments {
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
            for t in c.transfers.iter().filter(|t| t.month == state.month) {
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
            crate::stock_sale::settle(world, state, &mut out, &mut budgets)?;
        }
        _ => {}
    }
    Ok(Some(out))
}
pub fn validate_batch(world: &World, state: &State, batch: &Batch) -> Result<(), String> {
    let expected = evaluate(world, state)?;
    if batch.credit != expected
        || (expected.is_some()
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
