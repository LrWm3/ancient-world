//! Shared settlement primitives. Domain agreements own terms and authoritative receipts.
use crate::model::*;

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
