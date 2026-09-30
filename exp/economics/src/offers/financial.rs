//! Adapters for supplied, dated consent. Discovery is not underwriting, and a
//! preview includes all scheduled claims competing in the ordinary boundary.
use super::{Id, Offer, Request, Terms};
use crate::{compute::Backend, credit, forward, model::*, simulation::Simulation};
use std::collections::BTreeSet;

pub(super) fn is_financial(id: Id) -> bool {
    matches!(
        id,
        Id::Advance(_)
            | Id::PrepaidDelivery(_)
            | Id::Guarantee(_)
            | Id::FinancedPurchase(_)
            | Id::Employment(_)
            | Id::LiquidationBid(_)
    )
}

pub(super) fn discover(w: &World, s: &State, agent: AgentId, offers: &mut Vec<Offer>) {
    // These entries already carry bilateral consent; the named borrower/seller
    // can inspect them before the date. Actual execution rechecks law and funds.
    let mut additions: Vec<_> = w
        .lending
        .iter()
        .filter(|a| a.debtor == agent && a.month >= s.month && !s.credit.loans.contains_key(&a.id))
        .map(|a| Offer {
            id: Id::Advance(a.id),
            terms: Terms::Advance(a.clone()),
        })
        .chain(
            w.prepaid_deliveries
                .iter()
                .filter(|t| {
                    t.seller == agent
                        && t.month >= s.month
                        && !s.exchange.forwards.contains_key(&t.id)
                })
                .map(|t| Offer {
                    id: Id::PrepaidDelivery(t.id),
                    terms: Terms::PrepaidDelivery(t.clone()),
                }),
        )
        .collect();
    additions.extend(
        w.employment
            .iter()
            .filter(|t| {
                w.employment_offers.contains(&t.id)
                    && t.employer == agent
                    && s.month <= t.through
                    && !s.employment.earned.contains_key(&(t.id, s.month))
            })
            .map(|t| Offer {
                id: Id::Employment(t.id),
                terms: Terms::Employment(t.clone()),
            }),
    );
    let listed = crate::recovery::market::discover(w, s, agent);
    for b in w
        .recovery
        .bids
        .iter()
        .filter(|b| b.buyer == agent && b.month >= s.month)
    {
        if let Some(offer) = listed
            .iter()
            .find(|o| o.proceeding == b.proceeding && o.listing.asset == b.asset)
        {
            additions.push(Offer {
                id: Id::LiquidationBid(b.id),
                terms: Terms::Liquidation {
                    offer: offer.clone(),
                    bid: b.clone(),
                },
            });
        }
    }
    additions.sort_by_key(|o| o.id);
    offers.extend(additions);
}

pub(super) fn prepare(sim: &Simulation, requests: &[Request]) -> Result<Batch, String> {
    if sim.state.phase != Phase::Acquire || requests.is_empty() {
        return Err("financial acceptance requires a dated Acquire request".into());
    }
    let mut seen = BTreeSet::new();
    for r in requests {
        if r.continuing.is_some() || r.need.is_some() || !seen.insert(r.offer) {
            return Err("duplicate or invalid financial application".into());
        }
        let matches = match r.offer {
            Id::LiquidationBid(id) => sim
                .world
                .recovery
                .bids
                .iter()
                .any(|b| b.id == id && b.buyer == r.agent && b.month == sim.state.month),
            Id::Advance(id) => sim.world.lending.iter().any(|a| {
                a.id == id
                    && a.debtor == r.agent
                    && a.month == sim.state.month
                    && !sim.state.credit.loans.contains_key(&id)
            }),
            Id::PrepaidDelivery(id) => sim.world.prepaid_deliveries.iter().any(|t| {
                t.id == id
                    && t.seller == r.agent
                    && t.month == sim.state.month
                    && !sim.state.exchange.forwards.contains_key(&id)
            }),
            Id::Employment(id) => {
                sim.world.employment_offers.contains(&id)
                    && sim.world.employment.iter().any(|t| {
                        t.id == id
                            && t.employer == r.agent
                            && (t.from..=t.through).contains(&sim.state.month)
                            && !sim
                                .state
                                .employment
                                .earned
                                .contains_key(&(id, sim.state.month))
                    })
            }
            Id::Guarantee(id) => {
                sim.world
                    .recovery
                    .guarantee_applications
                    .iter()
                    .any(|a| a.guarantee == id && a.month == sim.state.month)
                    && crate::recovery::admission::acceptance(&sim.world, &sim.state, id, r.agent)
                        .is_ok()
            }
            Id::FinancedPurchase(_) => {
                super::scripted_credit_request(&sim.world, &sim.state).as_ref() == Some(r)
            }
            _ => false,
        };
        if !matches {
            return Err("financial request differs from supplied dated consent".into());
        }
    }
    // Use the same household before/core/after wrapper, employment reservation,
    // purchase policy and shared acquisition window as ordinary execution.
    let mut preview = sim.clone();
    preview.backend = Backend::Reference;
    preview.step()?;
    let batch = preview
        .ledger
        .pop()
        .ok_or("missing financial acceptance boundary")?;
    for r in requests {
        let accepted = match r.offer {
            Id::LiquidationBid(id) => {
                let b = sim.world.recovery.bids.iter().find(|b| b.id == id).unwrap();
                batch.credit.as_ref().is_some_and(|c| {
                    !c.recovery.iter().any(|r| matches!(r, crate::recovery::Receipt::SaleRejected { bid } if *bid == id))
                        && c.recovery.iter().any(|r| matches!(r, crate::recovery::Receipt::Sold { proceeding, asset, buyer, proceeds } if *proceeding == b.proceeding && *asset == b.asset && *buyer == b.buyer && *proceeds == b.price))
                })
            },
            Id::Employment(id) => batch.employment.as_ref().is_some_and(|b| b.receipts.iter().any(|r| r.agreement == id && r.earned_month == sim.state.month && r.delivered > 0)),
            Id::Advance(id) => batch.credit.as_ref().is_some_and(|b| b.events.iter().any(|e| matches!(e, credit::Event::Advanced { loan, .. } if *loan == id))),
            Id::FinancedPurchase(id) => batch.credit.as_ref().is_some_and(|b| b.events.iter().any(|e| matches!(e, credit::Event::Purchased { offer, .. } if *offer == id))),
            Id::Guarantee(id) => batch.credit.as_ref().is_some_and(|b| b.recovery.iter().any(|e| matches!(e, crate::recovery::Receipt::GuaranteeAdmission { guarantee, rejection: None } if *guarantee == id))),
            Id::PrepaidDelivery(id) => batch.transactions.iter().any(|t| matches!(&t.forward, Some(forward::Event::Accepted(c)) if c.id == id)),
            _ => false,
        };
        if !accepted {
            return Err(format!(
                "financial application {:?} rejected by the shared boundary",
                r.offer
            ));
        }
    }
    Ok(batch)
}
