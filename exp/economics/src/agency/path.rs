//! Dated completed-month observations, shared by ordinary and bilateral forecasts.
//! Scheduled unpaid amounts are evidence, not a legal default determination.
use crate::model::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum ClaimKind {
    Wage { agreement: u32, earned: u32 },
    Loan(u32),
    Delivery(u32),
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Unpaid {
    pub kind: ClaimKind,
    pub agent: AgentId,
    pub resource: ResourceId,
    pub quantity: i32,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Close {
    pub month: u32,
    pub stocks: BTreeMap<AgentId, BTreeMap<ResourceId, i32>>,
    pub unpaid: Vec<Unpaid>,
}
impl Close {
    pub fn balance(&self, agent: AgentId, resource: ResourceId) -> i32 {
        self.stocks
            .get(&agent)
            .and_then(|s| s.get(&resource))
            .copied()
            .unwrap_or(0)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Path {
    pub start: u32,
    pub closes: Vec<Close>,
}
impl Path {
    pub fn new(start: u32) -> Self {
        Self {
            start,
            closes: vec![],
        }
    }

    /// Call after Close has advanced the clock, before the next Open executes.
    pub fn record(&mut self, w: &World, s: &State) -> Result<(), String> {
        let expected = self
            .start
            .checked_add(self.closes.len() as u32)
            .and_then(|m| m.checked_add(1))
            .ok_or("path date overflow")?;
        if s.phase != Phase::Open || s.month != expected {
            return Err("path observation must follow exactly one completed month".into());
        }
        let month = s.month - 1;
        let mut stocks = BTreeMap::<AgentId, BTreeMap<ResourceId, i32>>::new();
        for (&(a, r), &quantity) in &s.balances {
            if w.resources
                .iter()
                .any(|v| v.id == r && v.kind == ResourceKind::Stock)
            {
                stocks.entry(a).or_default().insert(r, quantity);
            }
        }
        let mut unpaid = vec![];
        for (&(agreement, earned), wage) in &s.employment.earned {
            let c = &wage.claim;
            if c.condition.is_met(month, true) && c.outstanding() > 0 {
                unpaid.push(Unpaid {
                    kind: ClaimKind::Wage { agreement, earned },
                    agent: c.transfer.from,
                    resource: c.transfer.amount.resource,
                    quantity: c.outstanding(),
                });
            }
        }
        for (&id, loan) in &s.credit.loans {
            let c = loan.claim(month)?;
            if c.condition.is_met(month, true) && c.outstanding() > 0 {
                unpaid.push(Unpaid {
                    kind: ClaimKind::Loan(id),
                    agent: loan.debtor,
                    resource: loan.denomination,
                    quantity: c.outstanding(),
                });
            }
        }
        for (&id, delivery) in &s.exchange.forwards {
            let c = delivery.claim();
            if c.condition.is_met(month, true) && c.outstanding() > 0 {
                unpaid.push(Unpaid {
                    kind: ClaimKind::Delivery(id),
                    agent: delivery.debtor,
                    resource: delivery.goods.resource,
                    quantity: c.outstanding(),
                });
            }
        }
        self.closes.push(Close {
            month,
            stocks,
            unpaid,
        });
        Ok(())
    }
    pub fn minimum(&self, agent: AgentId, resource: ResourceId) -> Option<i32> {
        self.closes.iter().map(|c| c.balance(agent, resource)).min()
    }
}
