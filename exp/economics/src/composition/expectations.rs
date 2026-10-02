//! Frozen counterparty submission hypotheses from previously settled town books.
//! These filter hypothetical orders; they never grant cash, goods or live consent.
use crate::{
    marketplace::{MarketId, Side},
    model::*,
    town_market::{self, OrderReason, OrderSelections},
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Policy {
    #[default]
    Ordinary,
    /// Last observed eligible choice, expiring after this many elapsed months.
    RecentSubmission { memory_months: u32 },
}
impl Policy {
    pub fn validate(self) -> Result<(), String> {
        if matches!(self, Self::RecentSubmission { memory_months: 0 }) {
            return Err("counterparty observation memory must be positive".into());
        }
        Ok(())
    }
}

pub type Key = (AgentId, MarketId, Side);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub month: u32,
    pub submitted: bool,
    /// Actual goods delivered on this side. Zero does not mean withholding.
    pub filled: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub as_of: u32,
    pub actor: AgentId,
    pub policy: Policy,
    pub evidence: BTreeMap<Key, Evidence>,
}
impl Snapshot {
    pub fn observe(state: &State, actor: AgentId, policy: Policy) -> Result<Self, String> {
        policy.validate()?;
        let mut result = Self {
            as_of: state.month,
            actor,
            policy,
            evidence: BTreeMap::new(),
        };
        let Policy::RecentSubmission { memory_months } = policy else {
            return Ok(result);
        };
        for round in &state.town_market.history {
            if round.month >= state.month || state.month - round.month > memory_months {
                continue;
            }
            for r in &round.order_receipts {
                if r.agent == actor
                    || !matches!(
                        r.reason,
                        OrderReason::Submitted | OrderReason::PlannerWithheld
                    )
                {
                    continue;
                }
                let key = (r.agent, r.market, r.side);
                if result
                    .evidence
                    .get(&key)
                    .is_some_and(|e| e.month >= round.month)
                {
                    continue;
                }
                let filled = round
                    .attempts
                    .iter()
                    .filter(|a| {
                        a.session.market == r.market
                            && match r.side {
                                Side::Buy => a.session.buyer.agent == r.agent,
                                Side::Sell => a.session.seller.agent == r.agent,
                            }
                            && matches!(a.round.outcome, crate::negotiation::Outcome::Traded { .. })
                    })
                    .map(|a| a.session.goods.quantity)
                    .sum();
                result.evidence.insert(
                    key,
                    Evidence {
                        month: round.month,
                        submitted: r.reason == OrderReason::Submitted,
                        filled,
                    },
                );
            }
        }
        Ok(result)
    }

    /// No eligible observation or expired evidence uses ordinary generation.
    /// Later ineligibility does not erase an earlier still-fresh eligible choice.
    /// Freeze the input snapshot across rollouts; do not learn from imagined trades.
    pub fn expects_submission(&self, key: Key, month: u32) -> bool {
        let Policy::RecentSubmission { memory_months } = self.policy else {
            return true;
        };
        self.evidence.get(&key).is_none_or(|e| {
            month < self.as_of
                || e.month >= self.as_of
                || month - e.month > memory_months
                || e.submitted
        })
    }

    pub(super) fn masks(&self, world: &World, month: u32) -> Result<OrderSelections, String> {
        let c = world
            .town_market
            .as_ref()
            .ok_or("expectations require a town book")?;
        let mut masks: OrderSelections = world
            .participants
            .iter()
            .map(|p| (p.agent, Default::default()))
            .collect();
        for listing in town_market::listings(c) {
            for t in &listing.traders {
                let sides = if listing.adaptive {
                    vec![Side::Buy, Side::Sell]
                } else {
                    vec![t.side]
                };
                for side in sides {
                    if t.trader.agent == self.actor
                        || self.expects_submission((t.trader.agent, listing.market, side), month)
                    {
                        masks
                            .get_mut(&t.trader.agent)
                            .ok_or("counterparty is not a participant")?
                            .insert((listing.market, side));
                    }
                }
            }
        }
        Ok(masks)
    }
}
