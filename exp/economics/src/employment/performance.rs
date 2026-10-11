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
pub(super) fn deliveries(w: &World, s: &State, worker: AgentId, horizon: u32) -> Vec<Delivery> {
    let scope: std::collections::BTreeSet<_> = crate::households::parent(w, s, worker)
        .and_then(|id| w.households.iter().find(|h| h.agent == id))
        .map(|h| {
            crate::households::members(h, s)
                .chain(std::iter::once(h.agent))
                .collect()
        })
        .unwrap_or_else(|| [worker].into());
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
