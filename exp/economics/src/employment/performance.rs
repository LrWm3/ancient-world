//! Freeze accepted delivery duties before forecasting voluntary labor supply.
use crate::model::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivery {
    pub contract: u32,
    pub debtor: AgentId,
    pub resource: ResourceId,
    pub due: u32,
    pub performed: i32,
    pub remaining: i32,
}
fn scope(w: &World, s: &State, worker: AgentId) -> std::collections::BTreeSet<AgentId> {
    crate::households::parent(w, s, worker)
        .and_then(|id| w.households.iter().find(|h| h.agent == id))
        .map(|h| {
            crate::households::members(h, s)
                .chain(std::iter::once(h.agent))
                .collect()
        })
        .unwrap_or_else(|| [worker].into())
}
pub(super) fn deliveries(w: &World, s: &State, worker: AgentId, horizon: u32) -> Vec<Delivery> {
    let scope = scope(w, s, worker);
    s.exchange
        .forwards
        .values()
        .filter(|c| {
            scope.contains(&c.debtor)
                && c.claim().outstanding() > 0
                && c.effective_due() <= s.month.saturating_add(horizon - 1)
        })
        .map(|c| Delivery {
            contract: c.id,
            debtor: c.debtor,
            resource: c.goods.resource,
            due: c.effective_due(),
            performed: c.performed(),
            remaining: c.claim().outstanding(),
        })
        .collect()
}
pub(super) fn observe(
    targets: &[Delivery],
    s: &State,
    completed_month: u32,
    losses: &mut [Option<i128>],
) {
    for (t, loss) in targets.iter().zip(losses) {
        if loss.is_none() && t.due <= completed_month {
            let performed = s
                .exchange
                .forwards
                .get(&t.contract)
                .map_or(t.performed, |c| c.performed());
            *loss = Some(i128::from((t.remaining - (performed - t.performed)).max(0)));
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loan {
    pub id: u32,
    pub debtor: AgentId,
    pub denomination: ResourceId,
}
pub(super) fn loans(w: &World, s: &State, worker: AgentId) -> Vec<Loan> {
    let scope = scope(w, s, worker);
    s.credit
        .loans
        .values()
        .filter(|l| scope.contains(&l.debtor))
        .map(|l| Loan {
            id: l.id,
            debtor: l.debtor,
            denomination: l.denomination,
        })
        .collect()
}
pub(super) fn arrears(targets: &[Loan], ledger: &[Batch]) -> Vec<i128> {
    targets
        .iter()
        .map(|t| {
            ledger
                .iter()
                .filter_map(|b| b.credit.as_ref())
                .flat_map(|b| &b.events)
                .filter_map(|e| match e {
                    crate::credit::Event::Arrears { loan, amount, .. } if *loan == t.id => {
                        Some(i128::from(*amount))
                    }
                    _ => None,
                })
                .max()
                .unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        delivery_relief::{Action, Applied, Terms},
        forward::direct::Terms as Forward,
        scenario::*,
    };
    #[test]
    fn accepted_relief_changes_opening_targets_but_later_relief_is_not_performance() {
        let (w, mut s) = baseline();
        s.month = 4;
        let mut c = Forward {
            id: 7,
            seller: PERSON,
            buyer: STATE_AGENT,
            month: 1,
            due: 2,
            goods: Amount::new(GRAIN, 4),
            prepayment: Amount::new(TOKEN, 1),
        }
        .contract();
        let relief = |action| Applied {
            terms: Terms {
                id: 1,
                proceeding: 1,
                contract: 7,
                debtor: PERSON,
                creditor: STATE_AGENT,
                month: 3,
                expected_due: 2,
                expected_remaining: 4,
                action,
            },
            delivered: 0,
            substituted: 0,
        };
        s.exchange.forwards.insert(7, c.clone());
        let frozen = deliveries(&w, &s, PERSON, 1);
        for action in [Action::WriteOff { quantity: 4 }, Action::Extend { due: 8 }] {
            c.relief = vec![relief(action.clone())];
            s.exchange.forwards.insert(7, c.clone());
            assert!(deliveries(&w, &s, PERSON, 1).is_empty());
            let mut losses = vec![None];
            observe(&frozen, &s, 4, &mut losses);
            assert_eq!(losses, vec![Some(4)]);
            s.exchange.forwards.get_mut(&7).unwrap().delivered = 4;
            observe(&frozen, &s, 8, &mut losses);
            assert_eq!(losses, vec![Some(4)]);
        }
        c.relief = vec![relief(Action::WriteOff { quantity: 2 })];
        s.exchange.forwards.insert(7, c);
        assert_eq!(deliveries(&w, &s, PERSON, 1)[0].remaining, 2);
    }
}
