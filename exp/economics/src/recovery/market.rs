//! Discover the currently listed estate property. Prices are reserve terms;
//! only a funded, consented bid and ordinary settlement transfer title.
use crate::{model::*, opportunities, recovery};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub proceeding: u32,
    pub seller: AgentId,
    pub denomination: ResourceId,
    pub listing: recovery::Listing,
}

pub(crate) fn eligible_buyer(world: &World, state: &State, buyer: AgentId) -> bool {
    world.agents.iter().any(|a| a.id == buyer)
        && !world.recovery.proceedings.iter().any(|p| p.estate == buyer)
        && !state.terminal.contains_key(&buyer)
        && recovery::active(world, &state.credit, buyer).is_none()
        && crate::households::market::active(world, state, buyer)
        && !world.households.iter().any(|h| {
            h.agent == buyer
                && (crate::households::dissolution::winding_at(h, state.month).is_some()
                    || crate::households::dissolution::closed_at(h, state.month))
        })
        && opportunities::permits(world, state, buyer, opportunities::Action::AssetTrade)
}

pub fn discover(world: &World, state: &State, buyer: AgentId) -> Vec<Offer> {
    if !eligible_buyer(world, state, buyer) {
        return vec![];
    }
    let mut offers = vec![];
    for p in &world.recovery.proceedings {
        if buyer == p.debtor || buyer == p.estate {
            continue;
        }
        let Some(case) = state
            .credit
            .recovery
            .proceedings
            .get(&p.id)
            .filter(|c| c.stage == recovery::Stage::Active)
        else {
            continue;
        };
        for listing in &p.assets {
            if !case.sold.contains(&listing.asset)
                && recovery::saleable_asset(world, state, p.debtor, listing.asset)
            {
                offers.push(Offer {
                    proceeding: p.id,
                    seller: p.debtor,
                    denomination: p.denomination,
                    listing: listing.clone(),
                });
            }
        }
    }
    offers.sort_by_key(|o| (o.proceeding, o.listing.asset));
    offers
}
