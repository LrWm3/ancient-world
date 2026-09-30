//! Explicit inventory lots sold under an authorized proceeding. No appraisal
//! creates proceeds; unsold lots retain title and block closure.
use crate::{credit, finance, model::*, opportunities, recovery};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listing {
    pub id: u32,
    pub proceeding: u32,
    pub goods: Amount,
    pub minimum_price: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bid {
    pub id: u32,
    pub listing: u32,
    pub buyer: AgentId,
    pub month: u32,
    pub price: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub seller: AgentId,
    pub denomination: ResourceId,
    pub listing: Listing,
}

/// Discovery describes currently available lots, not a guarantee of acceptance:
/// competing bids still share the same opening funds, stock and receiving space.
pub fn discover(world: &World, state: &State, buyer: AgentId) -> Vec<Offer> {
    if !eligible_buyer(world, state, buyer) {
        return vec![];
    }
    let mut offers = vec![];
    for l in &world.recovery.inventory_listings {
        let Some(p) = world
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == l.proceeding)
        else {
            continue;
        };
        if buyer == p.debtor
            || !state
                .credit
                .recovery
                .proceedings
                .get(&p.id)
                .is_some_and(|c| {
                    c.stage == recovery::Stage::Active && !c.sold_inventory.contains(&l.id)
                })
        {
            continue;
        }
        if state.balance(p.debtor, l.goods.resource) >= l.goods.quantity {
            offers.push(Offer {
                seller: p.debtor,
                denomination: p.denomination,
                listing: l.clone(),
            });
        }
    }
    offers.sort_by_key(|o| (o.listing.proceeding, o.listing.id));
    offers
}

/// Later bilateral and town matching inherit the contribution reservation. Other shared
/// acquisition drivers still need the same adapter before admitting member bids.
fn eligible_buyer(world: &World, state: &State, buyer: AgentId) -> bool {
    let composed =
        !crate::acquisition::shared(world) || (world.market.is_none() && world.minting.is_none());
    recovery::market::eligible_buyer_for(world, state, buyer, opportunities::Action::StockTrade)
        && (crate::households::parent(world, state, buyer).is_none() || composed)
}

/// Reconstruct reservation-only pooling from classified committed proposals.
/// This carries exact fractional shares into later matching, without making
/// contributions or newly received goods spendable inside the boundary.
pub(crate) fn reservations(
    world: &World,
    state: &State,
    boundary: &credit::Boundary,
) -> Result<Option<crate::households::income_reservations::Reservations>, String> {
    if !boundary.recovery.iter().any(|r| matches!(r, recovery::Receipt::InventorySold { buyer, .. } if crate::households::parent(world, state, *buyer).is_some())) { return Ok(None); }
    let sales: Vec<_> = boundary
        .recovery
        .iter()
        .filter_map(|r| match r {
            recovery::Receipt::InventorySold { bid, .. } => Some(sale_transaction(world, *bid)),
            _ => None,
        })
        .collect::<Result<_, _>>()?;
    if sales.is_empty() {
        return Ok(None);
    }
    let mut pooling = crate::households::income_reservations::Reservations::new(
        world,
        state,
        crate::storage::usage(world, &state.balances),
    );
    for t in &boundary.transactions {
        if sales.contains(t) {
            pooling = pooling
                .preview(world, &t.effects)?
                .ok_or("inventory purchase exceeds household contribution storage")?;
        } else {
            pooling.reserve_unpooled(world, &t.effects)?;
        }
    }
    Ok(Some(pooling))
}

pub(crate) fn cleared(world: &World, proceeding: u32, case: &recovery::Proceeding) -> bool {
    world
        .recovery
        .inventory_listings
        .iter()
        .filter(|l| l.proceeding == proceeding)
        .all(|l| case.sold_inventory.contains(&l.id))
}

pub(crate) fn validate(world: &World, state: &State) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for l in &world.recovery.inventory_listings {
        let p = world
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == l.proceeding)
            .ok_or("inventory listing requires authorized proceeding")?;
        if !ids.insert(l.id)
            || l.goods.quantity <= 0
            || l.minimum_price <= 0
            || l.goods.resource == p.denomination
            || !world
                .resources
                .iter()
                .any(|r| r.id == l.goods.resource && r.kind == ResourceKind::Stock)
            || world.activities.perishable.contains(&l.goods.resource)
        {
            return Err("invalid estate inventory listing".into());
        }
    }
    ids.clear();
    for b in &world.recovery.inventory_bids {
        let l = world
            .recovery
            .inventory_listings
            .iter()
            .find(|l| l.id == b.listing)
            .ok_or("inventory bid requires listed lot")?;
        let p = world
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == l.proceeding)
            .unwrap();
        if !ids.insert(b.id)
            || b.month < p.opening_month
            || b.price <= 0
            || b.buyer == p.debtor
            || !world.agents.iter().any(|a| a.id == b.buyer)
            || world
                .recovery
                .proceedings
                .iter()
                .any(|p| p.estate == b.buyer)
        {
            return Err("invalid estate inventory bid".into());
        }
    }
    for (&id, c) in &state.credit.recovery.proceedings {
        if c.sold_inventory.iter().any(|lot| {
            !world
                .recovery
                .inventory_listings
                .iter()
                .any(|l| l.id == *lot && l.proceeding == id)
        }) || (c.stage == recovery::Stage::Closed && !cleared(world, id, c))
        {
            return Err("invalid estate inventory sale record".into());
        }
    }
    Ok(())
}

fn legs(p: &recovery::ProceedingTerms, l: &Listing, b: &Bid) -> [finance::Transfer; 2] {
    [
        finance::Transfer {
            from: p.debtor,
            to: b.buyer,
            amount: l.goods.clone(),
        },
        finance::Transfer {
            from: b.buyer,
            to: p.estate,
            amount: Amount::new(p.denomination, b.price),
        },
    ]
}

pub(crate) fn sale_transaction(world: &World, bid: u32) -> Result<Transaction, String> {
    let b = world
        .recovery
        .inventory_bids
        .iter()
        .find(|b| b.id == bid)
        .ok_or("unknown inventory bid")?;
    let l = world
        .recovery
        .inventory_listings
        .iter()
        .find(|l| l.id == b.listing)
        .ok_or("unknown inventory listing")?;
    let p = world
        .recovery
        .proceedings
        .iter()
        .find(|p| p.id == l.proceeding)
        .ok_or("unknown inventory proceeding")?;
    let mut effects = vec![];
    for leg in legs(p, l, b) {
        effects.extend(leg.effects()?);
    }
    Ok(credit::tx(
        format!("estate inventory bid {}", b.id),
        effects,
    ))
}

pub(crate) fn sales(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    execution: &mut finance::Execution,
) -> Result<crate::households::income_reservations::Reservations, String> {
    let mut pooling = crate::households::income_reservations::Reservations::new(
        world,
        state,
        crate::storage::usage(world, &state.balances),
    );
    for t in &out.transactions {
        pooling.reserve_unpooled(world, &t.effects)?;
    }
    let protected = crate::commitments::protected_stock(world, state)?;
    let mut bids: Vec<_> = world
        .recovery
        .inventory_bids
        .iter()
        .filter(|b| b.month == state.month)
        .collect();
    bids.sort_by_key(|b| (b.listing, std::cmp::Reverse(b.price), b.id));
    for b in bids {
        let l = world
            .recovery
            .inventory_listings
            .iter()
            .find(|l| l.id == b.listing)
            .unwrap();
        let p = world
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == l.proceeding)
            .unwrap();
        let stock = (p.debtor, l.goods.resource);
        let spendable = execution
            .available
            .get(&stock)
            .copied()
            .unwrap_or(0)
            .saturating_sub(protected.get(&stock).copied().unwrap_or(0));
        let eligible = out.after.recovery.proceedings.get(&p.id).is_some_and(|c| {
            c.stage == recovery::Stage::Active && !c.sold_inventory.contains(&l.id)
        }) && eligible_buyer(world, state, b.buyer)
            && recovery::active(world, &out.after, b.buyer).is_none()
            && b.price >= l.minimum_price
            && spendable >= l.goods.quantity;
        let transaction = sale_transaction(world, b.id)?;
        let reserved = pooling.preview(world, &transaction.effects)?;
        if !eligible || reserved.is_none() || execution.exchange(world, &legs(p, l, b)).is_err() {
            out.recovery
                .push(recovery::Receipt::InventorySaleRejected { bid: b.id });
            continue;
        }
        pooling = reserved.unwrap();
        let case = out.after.recovery.proceedings.get_mut(&p.id).unwrap();
        case.cash = case
            .cash
            .checked_add(b.price)
            .ok_or("estate inventory proceeds overflow")?;
        case.sold_inventory.insert(l.id);
        out.transactions.push(transaction);
        out.recovery.push(recovery::Receipt::InventorySold {
            proceeding: p.id,
            listing: l.id,
            bid: b.id,
            buyer: b.buyer,
            proceeds: b.price,
        });
    }
    Ok(pooling)
}
