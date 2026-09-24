//! Shared settlement primitives. Domain agreements own terms and authoritative receipts.
use crate::model::*;

pub const DEFAULT_CLAIM_RANK: u32 = 0;

/// Conserved account transfer, not issuance or destruction. Domain resolvers
/// restrict resource kinds and dates (including same-month capacity delegation).
/// Atomicity is provided by batch commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub from: AgentId,
    pub to: AgentId,
    pub amount: Amount,
}

impl Transfer {
    pub fn effects(&self) -> Result<Vec<Effect>, String> {
        if self.from == self.to || self.amount.quantity <= 0 {
            return Err("transfer requires distinct parties and a positive amount".into());
        }
        Ok(vec![
            Effect {
                account: (self.from, self.amount.resource),
                delta: -self.amount.quantity,
            },
            Effect {
                account: (self.to, self.amount.resource),
                delta: self.amount.quantity,
            },
        ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    OnAcceptance,
    OnOrAfterMonth(u32),
}
impl Condition {
    pub fn is_met(self, month: u32, accepted: bool) -> bool {
        match self {
            Self::OnAcceptance => accepted,
            Self::OnOrAfterMonth(due) => accepted && month >= due,
        }
    }
}

/// Scoped consequences; these do not cancel ongoing production or forgive debt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureRule {
    RejectExchange,
    CarryArrears,
    BlockNewUse,
    BlockNewAdvance,
}

/// Common view over a domain receipt. `settled` is in claim units, even when
/// payment used an alternative commodity. No duplicate mutable balance is stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Obligation {
    pub transfer: Transfer,
    pub settled: i32,
    pub condition: Condition,
    pub failure: FailureRule,
}
impl Obligation {
    pub fn outstanding(&self) -> i32 {
        self.transfer.amount.quantity.saturating_sub(self.settled)
    }
    pub fn blocks(&self, rule: FailureRule) -> bool {
        self.failure == rule && self.outstanding() > 0
    }
    /// Caller supplies opening spendable stock and current receiving storage room.
    /// Receipts from another payment cannot expand the opening spending budget.
    pub fn payable(&self, month: u32, accepted: bool, available: i32, room: i32) -> i32 {
        if self.settled < 0
            || self.settled > self.transfer.amount.quantity
            || !self.condition.is_met(month, accepted)
        {
            return 0;
        }
        let outstanding = self.outstanding().max(0);
        let possible = outstanding.min(available.max(0)).min(room.max(0));
        if self.failure == FailureRule::RejectExchange && possible < outstanding {
            0
        } else {
            possible
        }
    }
    pub fn payment(&self, quantity: i32) -> Result<Vec<Effect>, String> {
        if self.settled < 0
            || self.settled > self.transfer.amount.quantity
            || quantity > self.outstanding()
            || (self.failure == FailureRule::RejectExchange && quantity != self.outstanding())
        {
            return Err("invalid payment against obligation".into());
        }
        Transfer {
            amount: Amount::new(self.transfer.amount.resource, quantity),
            ..self.transfer.clone()
        }
        .effects()
    }
}

/// A full payment leg of an accepted exchange. All legs and ownership changes
/// must still be validated and committed together by the enclosing batch.
pub fn exchange_payment(
    from: AgentId,
    to: AgentId,
    amount: Amount,
    available: i32,
) -> Result<Vec<Effect>, String> {
    let obligation = Obligation {
        transfer: Transfer { from, to, amount },
        settled: 0,
        condition: Condition::OnAcceptance,
        failure: FailureRule::RejectExchange,
    };
    let quantity = obligation.payable(0, true, available, i32::MAX);
    obligation.payment(quantity)
}

/// One reservation window for contract legs. Incoming receipts never increase
/// spendable opening balances. Storage tracks actual net holdings separately.
#[derive(Clone, Debug)]
pub struct Execution {
    pub available: std::collections::BTreeMap<Account, i32>,
    pub(crate) stored: std::collections::BTreeMap<AgentId, i128>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Payment {
    pub requested: i32,
    pub paid: i32,
    pub remaining: i32,
    pub effects: Vec<Effect>,
}
impl Execution {
    pub fn opening(world: &World, state: &State) -> Self {
        Self {
            available: state.balances.clone(),
            stored: crate::storage::usage(world, &state.balances),
        }
    }
    pub(crate) fn from_parts(
        available: std::collections::BTreeMap<Account, i32>,
        stored: std::collections::BTreeMap<AgentId, i128>,
    ) -> Self {
        Self { available, stored }
    }
    pub fn pay_protected(
        &mut self,
        world: &World,
        month: u32,
        claim: &Obligation,
        protected: i32,
    ) -> Result<Payment, String> {
        let account = (claim.transfer.from, claim.transfer.amount.resource);
        let opening = self.available.get(&account).copied().unwrap_or(0);
        self.protect(account, protected);
        let result = self.pay(world, month, true, claim);
        self.available
            .insert(account, opening - result.as_ref().map_or(0, |p| p.paid));
        result
    }
    pub fn protect(&mut self, account: Account, quantity: i32) {
        let value = self.available.entry(account).or_default();
        *value = value.saturating_sub(quantity.max(0)).max(0);
    }
    pub fn pay(
        &mut self,
        world: &World,
        month: u32,
        accepted: bool,
        claim: &Obligation,
    ) -> Result<Payment, String> {
        if claim.settled < 0
            || claim.settled > claim.transfer.amount.quantity
            || claim.transfer.amount.quantity < 0
            || claim.transfer.from == claim.transfer.to
        {
            return Err("invalid contract claim".into());
        }
        let account = (claim.transfer.from, claim.transfer.amount.resource);
        let requested = if claim.condition.is_met(month, accepted) {
            claim.outstanding()
        } else {
            0
        };
        let paid = claim.payable(
            month,
            accepted,
            self.available.get(&account).copied().unwrap_or(0),
            crate::storage::room(
                world,
                &self.stored,
                claim.transfer.to,
                claim.transfer.amount.resource,
            ),
        );
        let effects = if paid > 0 {
            claim.payment(paid)?
        } else {
            vec![]
        };
        if paid > 0 {
            *self.available.entry(account).or_default() -= paid;
            crate::storage::apply(world, &mut self.stored, &effects);
        }
        Ok(Payment {
            requested,
            paid,
            remaining: claim.outstanding() - paid,
            effects,
        })
    }
    /// All-or-nothing exchange of any number of legs, including advance packages.
    /// Validate joint outgoing resources and final storage before changing reservations.
    pub fn exchange(&mut self, world: &World, legs: &[Transfer]) -> Result<Vec<Effect>, String> {
        let mut next = self.clone();
        let mut effects = vec![];
        for leg in legs {
            let account = (leg.from, leg.amount.resource);
            let available = next.available.entry(account).or_default();
            let leg_effects = exchange_payment(leg.from, leg.to, leg.amount.clone(), *available)?;
            *available -= leg.amount.quantity;
            effects.extend(leg_effects);
        }
        if !crate::storage::fits(world, &next.stored, &effects) {
            return Err("contract exchange exceeds storage".into());
        }
        crate::storage::apply(world, &mut next.stored, &effects);
        *self = next;
        Ok(effects)
    }
}

/// Stable identities across claim adapters; lower configured rank collects first.
/// Ordering is scoped to claims sharing a boundary, never a phase reordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContractId {
    Loan(u32),
    Land(u32),
    Forward(u32),
}

/// Allocation evidence in claim units; actual denomination-conversion legs remain
/// in the committed transactions. This is a receipt, not another balance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionReceipt {
    pub contract: ContractId,
    pub rank: u32,
    pub debtor: AgentId,
    pub creditor: AgentId,
    pub requested: Amount,
    pub paid: i32,
}
