//! Strict adapter: same-denomination finance, valued land and costed spot trades.
//! Unsupported positions/events fail before either simulation or reporting publishes.
use crate::{
    accounting::{self, Account, Book, Entry, Flow, Line},
    credit,
    model::*,
    recovery,
    simulation::Simulation,
};
use std::collections::BTreeMap;
type Positions = BTreeMap<(AgentId, Account), i128>;
type Flows = BTreeMap<(AgentId, Account, Flow), i128>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Audit {
    book: Book,
    boundary: State,
    asset_values: BTreeMap<AssetId, i128>,
    inventory: crate::inventory_accounting::Inventory,
}
fn positions(
    world: &World,
    state: &State,
    coin: ResourceId,
    values: &BTreeMap<AssetId, i128>,
    inventory: &crate::inventory_accounting::Inventory,
) -> Result<Positions, String> {
    if !world
        .resources
        .iter()
        .any(|r| r.id == coin && r.kind == ResourceKind::Stock)
        || !state.equipment.is_empty()
        || !state.exchange.forwards.is_empty()
        || !state.obligations.is_empty()
        || !world.households.is_empty()
    {
        return Err("financial adapter requires cash/loan positions; equipment, forward, land-dues and household adapters remain unsupported".into());
    }
    inventory.validate(world, state, coin)?;
    let mut p = BTreeMap::new();
    for (&(agent, r), &q) in &state.balances {
        if q == 0 {
            continue;
        }
        if r == coin {
            accounting::add(&mut p, (agent, Account::Cash), i128::from(q))?;
        } else if world
            .resources
            .iter()
            .any(|x| x.id == r && x.kind == ResourceKind::Stock)
        {
            let h = inventory
                .0
                .get(&(agent, r))
                .ok_or("missing inventory basis")?;
            accounting::add(&mut p, (agent, Account::Inventory(r)), h.cost)?;
        }
    }
    if world.credit.as_ref().is_some_and(|c| {
        c.offers
            .iter()
            .any(|o| o.sale.price.resource != coin || o.loan.denomination != coin)
    }) {
        return Err("mixed asset purchase denominations".into());
    }
    for asset in &world.assets {
        let initial = *values
            .get(&asset.id)
            .ok_or("missing explicit tangible valuation")?;
        let value = state
            .credit
            .values
            .get(&asset.id)
            .map_or(initial, |v| i128::from(*v));
        if initial < 0 || value < 0 {
            return Err("negative tangible valuation".into());
        }
        let owner = state
            .credit
            .loans
            .values()
            .find(|l| {
                l.status == credit::Status::PendingSale
                    && l.collateral.as_ref().is_some_and(|c| c.asset == asset.id)
            })
            .map(|l| l.debtor)
            .or_else(|| credit::owner(world, state, asset.id))
            .ok_or("missing asset owner")?;
        accounting::add(&mut p, (owner, Account::Tangible(asset.id)), value)?;
    }
    for l in state.credit.loans.values() {
        if l.denomination != coin {
            return Err("mixed loan denominations require an explicit valuation adapter".into());
        }
        for (agent, a, q) in [
            (l.creditor, Account::LoanReceivable(l.id), l.principal),
            (l.creditor, Account::InterestReceivable(l.id), l.interest),
            (l.debtor, Account::LoanPayable(l.id), -l.principal),
            (l.debtor, Account::InterestPayable(l.id), -l.interest),
        ] {
            accounting::add(&mut p, (agent, a), i128::from(q))?;
        }
    }
    for terms in &world.recovery.proceedings {
        if terms.denomination != coin {
            return Err("mixed estate denominations".into());
        }
        let q = i128::from(
            state
                .credit
                .recovery
                .proceedings
                .get(&terms.id)
                .map_or(0, |c| c.cash),
        );
        accounting::add(&mut p, (terms.estate, Account::Cash), -q)?;
        accounting::add(&mut p, (terms.estate, Account::CustodyCash(terms.id)), q)?;
        accounting::add(
            &mut p,
            (terms.estate, Account::CustodyPayable(terms.id)),
            -q,
        )?;
        accounting::add(&mut p, (terms.debtor, Account::RestrictedCash(terms.id)), q)?;
    }
    p.retain(|_, v| *v != 0);
    Ok(p)
}
fn flow(
    flows: &mut Flows,
    agent: AgentId,
    account: Account,
    kind: Flow,
    amount: i128,
) -> Result<(), String> {
    accounting::add(flows, (agent, account, kind), amount)
}
fn result(lines: &mut Vec<Line>, agent: AgentId, account: Account, debit: i128) {
    if debit != 0 {
        lines.push(Line {
            agent,
            account,
            debit,
            flow: None,
        });
    }
}
fn disposal(
    lines: &mut Vec<Line>,
    opening: &Positions,
    seller: AgentId,
    asset: AssetId,
    price: i32,
) -> Result<(), String> {
    let basis = opening
        .get(&(seller, Account::Tangible(asset)))
        .copied()
        .unwrap_or(0);
    let gain = i128::from(price)
        .checked_sub(basis)
        .ok_or("disposal overflow")?;
    result(
        lines,
        seller,
        if gain >= 0 {
            Account::DisposalGain
        } else {
            Account::DisposalLoss
        },
        -gain,
    );
    Ok(())
}
impl Audit {
    pub fn new(world: &World, state: &State, denomination: ResourceId) -> Result<Self, String> {
        Self::with_assets(world, state, denomination, BTreeMap::new())
    }
    pub fn with_assets(
        world: &World,
        state: &State,
        denomination: ResourceId,
        asset_values: BTreeMap<AssetId, i128>,
    ) -> Result<Self, String> {
        Self::with_inventory(world, state, denomination, asset_values, BTreeMap::new())
    }
    /// Opening total carrying costs, not unit quotes, for each noncash stock holding.
    pub fn with_inventory(
        world: &World,
        state: &State,
        denomination: ResourceId,
        asset_values: BTreeMap<AssetId, i128>,
        inventory_costs: BTreeMap<crate::model::Account, i128>,
    ) -> Result<Self, String> {
        crate::settlement::validate_world(world, state)?;
        let inventory = crate::inventory_accounting::Inventory::open(
            world,
            state,
            denomination,
            inventory_costs,
        )?;
        if state.phase != Phase::Open {
            return Err("open a reporting book at a month opening".into());
        }
        Ok(Self {
            book: Book::open_at(
                denomination,
                state.month.checked_sub(1).ok_or("invalid opening month")?,
                positions(world, state, denomination, &asset_values, &inventory)?,
            )?,
            asset_values,
            inventory,
            boundary: state.clone(),
        })
    }
    pub fn book(&self) -> &Book {
        &self.book
    }
    /// Atomic composed execution: accounting failure publishes neither side.
    pub fn step(&mut self, sim: &mut Simulation) -> Result<(), String> {
        let mut next = sim.clone();
        next.step()?;
        self.record(
            &sim.world,
            &sim.state,
            next.ledger.last().ok_or("missing committed batch")?,
            &next.state,
        )?;
        *sim = next;
        Ok(())
    }
    pub fn record(
        &mut self,
        world: &World,
        before: &State,
        batch: &Batch,
        after: &State,
    ) -> Result<(), String> {
        if *before != self.boundary {
            return Err("accounting checkpoint/boundary mismatch".into());
        }
        let mut verified = before.clone();
        crate::settlement::commit(
            world,
            &mut verified,
            batch,
            crate::compute::Backend::Reference,
            crate::settlement::DEFAULT_EFFECT_LIMIT,
        )?;
        if verified != *after {
            return Err("accounting requires the exact committed state".into());
        }
        if batch.transactions.iter().any(|t| {
            t.process.is_some()
                || t.trade.is_some()
                || t.delivery.is_some()
                || t.forward.is_some()
                || t.royalty.is_some()
        }) || batch.minting.is_some()
            || batch.town_market.is_some()
        {
            return Err("transaction needs an explicit accounting adapter".into());
        }
        let coin = self.book.denomination();
        let negotiated = crate::negotiation::transactions(world, before, &batch.negotiation)?;
        let trades: Vec<_> = batch
            .transactions
            .iter()
            .filter(|t| t.stock_trade.is_some() || negotiated.contains(t))
            .collect();
        let (inventory, trade_lines) = self.inventory.settle(&trades, coin)?;
        for t in &batch.transactions {
            if trades.contains(&t) {
                continue;
            }
            let has_stock = t.effects.iter().any(|e| {
                e.delta != 0
                    && world
                        .resources
                        .iter()
                        .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Stock)
            });
            if has_stock
                && !batch
                    .credit
                    .as_ref()
                    .is_some_and(|c| c.transactions.contains(t))
            {
                return Err("stock transaction has no recognized financial source".into());
            }
            for e in &t.effects {
                if e.delta != 0
                    && e.account.1 != coin
                    && world
                        .resources
                        .iter()
                        .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Stock)
                {
                    return Err("unpriced stock movement".into());
                }
            }
        }
        let opening = positions(world, before, coin, &self.asset_values, &self.inventory)?;
        let closing = positions(world, after, coin, &self.asset_values, &inventory)?;
        let mut delta = closing.clone();
        for (key, value) in &opening {
            accounting::add(&mut delta, key.clone(), -*value)?;
        }
        let mut lines = vec![];
        let mut flows = Flows::new();
        for l in trade_lines {
            if let Some(kind) = l.flow {
                flow(&mut flows, l.agent, l.account, kind, l.debit)?;
            } else {
                lines.push(l);
            }
        }
        let mut interest: BTreeMap<_, _> = before
            .credit
            .loans
            .iter()
            .map(|(id, l)| (*id, l.interest))
            .collect();
        if let Some(c) = &batch.credit {
            let loan = |id: &u32| {
                c.after
                    .loans
                    .get(id)
                    .or_else(|| before.credit.loans.get(id))
                    .ok_or("missing reporting loan")
            };
            for e in &c.events {
                use credit::Event;
                match e {
                    Event::Advanced {
                        creditor,
                        debtor,
                        amount,
                        ..
                    } => {
                        flow(
                            &mut flows,
                            *creditor,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(amount.quantity),
                        )?;
                        flow(
                            &mut flows,
                            *debtor,
                            Account::Cash,
                            Flow::Financing,
                            i128::from(amount.quantity),
                        )?;
                    }
                    Event::Accrued {
                        loan: id,
                        interest: q,
                        ..
                    } => {
                        let l = loan(id)?;
                        let previous = interest.entry(*id).or_default();
                        *previous = previous
                            .checked_add(*q)
                            .ok_or("interest reporting overflow")?;
                        result(
                            &mut lines,
                            l.creditor,
                            Account::InterestIncome,
                            -i128::from(*q),
                        );
                        result(
                            &mut lines,
                            l.debtor,
                            Account::InterestExpense,
                            i128::from(*q),
                        );
                    }
                    Event::Paid {
                        loan: id,
                        principal,
                        interest: q,
                    } => {
                        let l = loan(id)?;
                        *interest.entry(*id).or_default() -= *q;
                        for (agent, sign, kind) in [
                            (l.creditor, 1, Flow::Investing),
                            (l.debtor, -1, Flow::Financing),
                        ] {
                            flow(
                                &mut flows,
                                agent,
                                Account::Cash,
                                kind,
                                i128::from(*principal) * sign,
                            )?;
                            flow(
                                &mut flows,
                                agent,
                                Account::Cash,
                                Flow::Operating,
                                i128::from(*q) * sign,
                            )?;
                        }
                    }
                    Event::Endowed { agent, amount } => {
                        result(
                            &mut lines,
                            *agent,
                            Account::Capital,
                            -i128::from(amount.quantity),
                        );
                        flow(
                            &mut flows,
                            *agent,
                            Account::Cash,
                            Flow::Financing,
                            i128::from(amount.quantity),
                        )?;
                    }
                    Event::Cashflow { from, to, amount } => {
                        result(
                            &mut lines,
                            *from,
                            Account::TransferExpense,
                            i128::from(amount.quantity),
                        );
                        result(
                            &mut lines,
                            *to,
                            Account::TransferIncome,
                            -i128::from(amount.quantity),
                        );
                        flow(
                            &mut flows,
                            *from,
                            Account::Cash,
                            Flow::Operating,
                            -i128::from(amount.quantity),
                        )?;
                        flow(
                            &mut flows,
                            *to,
                            Account::Cash,
                            Flow::Operating,
                            i128::from(amount.quantity),
                        )?;
                    }
                    Event::Purchased {
                        offer,
                        asset,
                        buyer,
                        price,
                        downpayment,
                        advance,
                    } => {
                        let o = world
                            .credit
                            .as_ref()
                            .and_then(|c| c.offers.iter().find(|o| o.id == *offer))
                            .ok_or("missing purchase offer")?;
                        disposal(&mut lines, &opening, o.sale.seller, *asset, *price)?;
                        flow(
                            &mut flows,
                            *buyer,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*downpayment),
                        )?;
                        flow(
                            &mut flows,
                            o.sale.seller,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*downpayment),
                        )?;
                        if o.loan.creditor != o.sale.seller {
                            flow(
                                &mut flows,
                                o.loan.creditor,
                                Account::Cash,
                                Flow::Investing,
                                -i128::from(*advance),
                            )?;
                            flow(
                                &mut flows,
                                o.sale.seller,
                                Account::Cash,
                                Flow::Investing,
                                i128::from(*advance),
                            )?;
                        }
                    }
                    Event::Enforced {
                        loan: id,
                        value,
                        debt_credit,
                        surplus,
                        ..
                    } => {
                        let l = loan(id)?;
                        let asset = l.collateral.as_ref().ok_or("missing collateral")?.asset;
                        disposal(&mut lines, &opening, l.debtor, asset, *value)?;
                        let q = interest.entry(*id).or_default();
                        *q -= (*q).min(*debt_credit);
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*surplus),
                        )?;
                        flow(
                            &mut flows,
                            l.debtor,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*surplus),
                        )?;
                    }
                    Event::Resold {
                        loan: id,
                        buyer,
                        price,
                        debt_credit,
                        surplus,
                        ..
                    } => {
                        let l = loan(id)?;
                        let asset = l.collateral.as_ref().ok_or("missing collateral")?.asset;
                        disposal(&mut lines, &opening, l.debtor, asset, *price)?;
                        let q = interest.entry(*id).or_default();
                        let paid_interest = (*q).min(*debt_credit);
                        *q -= paid_interest;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Operating,
                            i128::from(paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*debt_credit - paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.debtor,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*surplus),
                        )?;
                        flow(
                            &mut flows,
                            *buyer,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*price),
                        )?;
                    }
                    Event::Rejected { .. }
                    | Event::Arrears { .. }
                    | Event::RepossessedForSale { .. }
                    | Event::ResaleNoBuyer { .. }
                    | Event::ResaleBid { .. }
                    | Event::ResaleRejected { .. }
                    | Event::EnforcementDeferred { .. } => {}
                }
            }
            for r in &c.recovery {
                match r {
                    recovery::Receipt::Guaranteed {
                        guarantee,
                        loan: id,
                        paid,
                        ..
                    } => {
                        let l = loan(id)?;
                        let g = world
                            .recovery
                            .guarantees
                            .iter()
                            .find(|g| g.id == *guarantee)
                            .ok_or("missing guarantee")?;
                        let q = interest.entry(*id).or_default();
                        let paid_interest = (*q).min(*paid);
                        *q -= paid_interest;
                        flow(
                            &mut flows,
                            g.guarantor,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*paid),
                        )?;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Operating,
                            i128::from(paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*paid - paid_interest),
                        )?;
                    }
                    recovery::Receipt::Distributed {
                        proceeding,
                        loan: id,
                        paid,
                        ..
                    } => {
                        let l = loan(id)?;
                        let q = interest.entry(*id).or_default();
                        let paid_interest = (*q).min(*paid);
                        *q -= paid_interest;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Operating,
                            i128::from(paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.creditor,
                            Account::Cash,
                            Flow::Investing,
                            i128::from(*paid - paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.debtor,
                            Account::RestrictedCash(*proceeding),
                            Flow::Operating,
                            -i128::from(paid_interest),
                        )?;
                        flow(
                            &mut flows,
                            l.debtor,
                            Account::RestrictedCash(*proceeding),
                            Flow::Financing,
                            -i128::from(*paid - paid_interest),
                        )?;
                    }
                    recovery::Receipt::WrittenOff {
                        loan: id,
                        principal,
                        interest,
                        ..
                    } => {
                        let l = loan(id)?;
                        let loss = i128::from(*principal) + i128::from(*interest);
                        result(&mut lines, l.creditor, Account::CreditLoss, loss);
                        result(&mut lines, l.debtor, Account::DebtRelief, -loss);
                    }
                    recovery::Receipt::Sold {
                        proceeding,
                        asset,
                        buyer,
                        proceeds,
                    } => {
                        let p = world
                            .recovery
                            .proceedings
                            .iter()
                            .find(|p| p.id == *proceeding)
                            .ok_or("missing estate")?;
                        disposal(&mut lines, &opening, p.debtor, *asset, *proceeds)?;
                        flow(
                            &mut flows,
                            *buyer,
                            Account::Cash,
                            Flow::Investing,
                            -i128::from(*proceeds),
                        )?;
                        flow(
                            &mut flows,
                            p.debtor,
                            Account::RestrictedCash(*proceeding),
                            Flow::Investing,
                            i128::from(*proceeds),
                        )?;
                    }
                    recovery::Receipt::SaleRejected { .. }
                    | recovery::Receipt::Opened { .. }
                    | recovery::Receipt::OpeningRejected { .. }
                    | recovery::Receipt::Admitted { .. }
                    | recovery::Receipt::ClosureDeferred { .. }
                    | recovery::Receipt::Closed { .. } => {}
                    _ => return Err("recovery event needs an explicit accounting adapter".into()),
                }
            }
        }
        // Residual cash movements may only be a verified transfer between owned
        // cash and the same debtor's estate account. No unexplained income plug.
        let mut classified: BTreeMap<_, i128> = BTreeMap::new();
        for ((agent, a, _), q) in &flows {
            accounting::add(&mut classified, (*agent, a.clone()), *q)?;
        }
        let mut residual: BTreeMap<_, i128> = delta
            .iter()
            .filter(|((_, a), _)| a.cash())
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        for (k, v) in &classified {
            accounting::add(&mut residual, k.clone(), -*v)?;
        }
        for p in &world.recovery.proceedings {
            let restricted = residual
                .remove(&(p.debtor, Account::RestrictedCash(p.id)))
                .unwrap_or(0);
            if restricted != 0 {
                flow(
                    &mut flows,
                    p.debtor,
                    Account::RestrictedCash(p.id),
                    Flow::Internal,
                    restricted,
                )?;
                flow(
                    &mut flows,
                    p.debtor,
                    Account::Cash,
                    Flow::Internal,
                    -restricted,
                )?;
                accounting::add(&mut residual, (p.debtor, Account::Cash), restricted)?;
            }
        }
        if residual.values().any(|v| *v != 0) {
            return Err("unclassified cash movement; no financial statements published".into());
        }
        for ((agent, account), debit) in delta {
            if debit != 0 && !account.cash() {
                lines.push(Line {
                    agent,
                    account,
                    debit,
                    flow: None,
                });
            }
        }
        for ((agent, account, kind), debit) in flows {
            if debit != 0 {
                lines.push(Line {
                    agent,
                    account,
                    debit,
                    flow: Some(kind),
                });
            }
        }
        let mut candidate = self.book.clone();
        candidate.post(Entry {
            id: format!("batch:{}", batch.id),
            month: batch.month,
            batch: Some(batch.id),
            description: format!("Verified {:?} financial boundary", batch.phase),
            lines,
        })?;
        let mut reported: Positions = candidate
            .balances()
            .iter()
            .filter(|((_, a), _)| {
                matches!(
                    a.class(),
                    accounting::Class::Asset | accounting::Class::Liability
                )
            })
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        reported.retain(|_, v| *v != 0);
        if reported != closing {
            return Err("journal does not reconcile to authoritative positions".into());
        }
        self.inventory = inventory;
        self.book = candidate;
        self.boundary = after.clone();
        Ok(())
    }
}
