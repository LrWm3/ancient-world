//! Explicit bilateral prepayment terms, independent of tool underwriting.
//! Supplied terms represent both parties' consent; execution still checks law and funds.
use super::{Contract, Event, Price};
use crate::{acquisition::Resources, laws, model::*, opportunities};
use std::collections::BTreeSet;

/// Supplied terms are bilateral consent. Concurrent admission does not guarantee
/// future production; all future deliveries remain visible as distinct claims.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AdmissionPolicy {
    #[default]
    SingleOutstanding,
    Concurrent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terms {
    pub id: u32,
    pub seller: AgentId,
    pub buyer: AgentId,
    pub month: u32,
    pub due: u32,
    pub goods: Amount,
    pub prepayment: Amount,
}
impl Terms {
    pub fn contract(&self) -> Contract {
        Contract {
            id: self.id,
            debtor: self.seller,
            creditor: self.buyer,
            issued: self.month,
            due: self.due,
            goods: self.goods.clone(),
            price: Price {
                goods: self.goods.quantity,
                coins: self.prepayment.quantity,
            },
            advance: self.prepayment.clone(),
            delivered: 0,
            relief: vec![],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    Ineligible,
    ExistingDelivery,
    FundingShortfall,
    StorageShortfall,
}

pub fn enabled(w: &World) -> bool {
    !w.prepaid_deliveries.is_empty()
}

pub(crate) fn validate(w: &World, s: &State) -> Result<(), String> {
    if !enabled(w) {
        return Ok(());
    }
    if w.credit.as_ref().is_some_and(|c| c.stock_sales.is_some())
        || w.production_market.is_some()
        || w.work_choice.is_some()
        || (!crate::acquisition::search_composition(w)
            && (w.competition.is_some()
                || w.priority == Priority::ConsequenceAware
                || !w.access_offers.is_empty()))
        || !w.offers.is_empty()
        || (!w.bids.is_empty() && w.market.is_none())
    {
        return Err(
            "direct forwards require a composed acquisition driver for this search/market configuration".into(),
        );
    }
    let mut ids = BTreeSet::new();
    let stock = |id| {
        w.resources
            .iter()
            .any(|r| r.id == id && r.kind == ResourceKind::Stock)
    };
    for t in &w.prepaid_deliveries {
        if !ids.insert(t.id)
            || t.seller == t.buyer
            || t.month == 0
            || t.due <= t.month
            || ![t.seller, t.buyer]
                .iter()
                .all(|a| w.agents.iter().any(|v| v.id == *a))
            || !stock(t.goods.resource)
            || !stock(t.prepayment.resource)
            || t.goods.resource == t.prepayment.resource
            || t.goods.quantity <= 0
            || t.prepayment.quantity <= 0
            || w.storage
                .weights
                .get(&t.prepayment.resource)
                .copied()
                .unwrap_or(0)
                != 0
        {
            return Err("invalid direct prepaid delivery terms".into());
        }
    }
    for (id, c) in &s.exchange.forwards {
        let Some(t) = w.prepaid_deliveries.iter().find(|t| t.id == *id) else {
            // Tool-underwritten records are checked by their own admission adapter.
            if super::policy(w).is_some() {
                continue;
            }
            return Err("direct forward has no accepted terms".into());
        };
        let mut expected = t.contract();
        expected.delivered = c.delivered;
        expected.relief = c.relief.clone();
        crate::delivery_relief::validate_history(w, s, c)?;
        if *c != expected || c.issued > s.month || c.delivered < 0 || c.delivered > c.goods.quantity
        {
            return Err("invalid direct forward history".into());
        }
    }
    Ok(())
}

pub(crate) fn evaluate(
    w: &World,
    s: &State,
    resources: &mut Resources,
) -> Result<super::Collections, String> {
    if !enabled(w) {
        return Ok(Default::default());
    }
    // Existing deliveries precede new prepayments, all from opening funds/stocks.
    let mut available = resources.available.clone();
    let mut stored = resources.storage.clone();
    if resources.pooling.is_none() {
        resources.pooling = Some(crate::households::income_reservations::Reservations::new(
            w,
            s,
            resources.storage.clone(),
        ));
    }
    let collection = super::collect_with_pooling(
        w,
        s,
        &mut available,
        &mut stored,
        resources.pooling.as_ref(),
    )?;
    let mut result = collection.transactions;
    for t in &result {
        if matches!(t.forward, Some(Event::Delivery { contract, .. }) if w.prepaid_deliveries.iter().any(|p| p.id == contract))
        {
            resources.reserve(w, std::slice::from_ref(t))?;
        } else {
            resources.reserve_unpooled(w, std::slice::from_ref(t))?;
        }
    }
    let mut delivered = s.clone();
    for tx in &result {
        crate::exchange::record(&mut delivered, tx);
    }
    let mut admitted = delivered.exchange.forwards;
    let mut terms: Vec<_> = w
        .prepaid_deliveries
        .iter()
        .filter(|t| t.month == s.month)
        .collect();
    terms.sort_by_key(|t| t.id);
    for t in terms {
        if admitted.contains_key(&t.id) {
            continue;
        }
        let eligible =
            [t.seller, t.buyer].iter().all(|a| {
                !s.terminal.contains_key(a)
                    && crate::recovery::active(w, &s.credit, *a).is_none()
                    && crate::households::market::active(w, s, *a)
                    && !w.households.iter().any(|h| {
                        h.agent == *a
                            && (crate::households::dissolution::winding_at(h, s.month).is_some()
                                || crate::households::dissolution::closed_at(h, s.month))
                    })
                    && opportunities::permits(w, s, *a, opportunities::Action::StockTrade)
            }) && laws::evaluate_agreement(w, s, t.seller, laws::AgreementForm::PrepaidDelivery)
                .allowed;
        let failure = if !eligible {
            Some(Rejection::Ineligible)
        } else if w.prepaid_admission == AdmissionPolicy::SingleOutstanding
            && admitted
                .values()
                .any(|c| c.debtor == t.seller && c.claim().outstanding() > 0)
        {
            Some(Rejection::ExistingDelivery)
        } else if resources
            .available
            .get(&(t.buyer, t.prepayment.resource))
            .copied()
            .unwrap_or(0)
            < t.prepayment.quantity
        {
            Some(Rejection::FundingShortfall)
        } else {
            // Reserve prospective receipts against the existing shared-storage
            // rules, including buyers that are households or share a household.
            let mut receipts: Vec<_> = admitted
                .values()
                .filter(|c| c.claim().outstanding() > 0)
                .map(|c| Effect {
                    account: (c.creditor, c.goods.resource),
                    delta: c.claim().outstanding(),
                })
                .collect();
            receipts.push(Effect {
                account: (t.buyer, t.goods.resource),
                delta: t.goods.quantity,
            });
            if !crate::storage::fits(w, &resources.storage, &receipts) {
                Some(Rejection::StorageShortfall)
            } else {
                None
            }
        };
        let mut tx = super::transaction("direct prepaid delivery admission");
        if let Some(reason) = failure {
            tx.forward = Some(Event::AdmissionRejected {
                contract: t.id,
                reason,
            });
        } else {
            let c = t.contract();
            tx.effects = vec![
                Effect {
                    account: (t.buyer, t.prepayment.resource),
                    delta: -t.prepayment.quantity,
                },
                Effect {
                    account: (t.seller, t.prepayment.resource),
                    delta: t.prepayment.quantity,
                },
            ];
            if !resources.fits(w, &tx.effects)? {
                return Err("prepayment exceeds storage".into());
            }
            tx.forward = Some(Event::Accepted(Box::new(c.clone())));
            resources.reserve_unpooled(w, std::slice::from_ref(&tx))?;
            admitted.insert(c.id, c);
        }
        result.push(tx);
    }
    Ok(super::Collections {
        transactions: result,
        receipts: collection.receipts,
    })
}
