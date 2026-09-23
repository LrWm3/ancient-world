//! Two-person calibration fixture; initial buffers cover one order horizon.
use crate::{
    model::*,
    production_market::{self as pm, Choice, Policy, Purchases, Work},
    scenario::*,
};
use std::collections::BTreeMap;

pub const CROP_PERSON: AgentId = PERSON;
pub const WOOD_PERSON: AgentId = 91;
const OPENING_BUFFER_MONTHS: i32 = 6;
const REMOVED_PERSON_IDS: [AgentId; 2] = [89, 92];

pub fn scenario(trading: bool) -> (World, State) {
    let (mut w, mut s) = pm::reciprocal_scenario(trading);
    let keep = |id: AgentId| !REMOVED_PERSON_IDS.contains(&id);
    w.agents.retain(|a| keep(a.id));
    w.participants.retain(|p| keep(p.agent));
    w.assets.retain(|a| keep(a.id));
    w.rights.retain(|r| keep(r.holder));
    w.transaction_policy
        .as_mut()
        .unwrap()
        .agent_types
        .retain(|id, _| keep(*id));
    w.storage.capacities.retain(|id, _| keep(*id));
    let market = w.town_market.as_mut().unwrap();
    market.traders.retain(|t| keep(t.trader.agent));
    for listing in &mut market.additional {
        listing.traders.retain(|t| keep(t.trader.agent));
    }
    s.balances.retain(|(a, _), _| keep(*a));
    s.practice.retain(|(a, _), _| keep(*a));
    s.town_market.positions.retain(|a, _| keep(*a));
    for agent in [CROP_PERSON, WOOD_PERSON] {
        for resource in [GRAIN, FUEL] {
            s.balances.insert((agent, resource), OPENING_BUFFER_MONTHS);
        }
    }
    (w, s)
}
pub fn directed(w: &mut World) {
    w.production_market.as_mut().unwrap().policy = Policy::Fixed(BTreeMap::from([
        (
            CROP_PERSON,
            Choice {
                work: Work::Produce(GROW),
                buy: Purchases::Market(pm::WOOD_MARKET),
            },
        ),
        (
            WOOD_PERSON,
            Choice {
                work: Work::Produce(PREPARE_FUEL),
                buy: Purchases::Market(crate::negotiation::GRAIN_MARKET),
            },
        ),
    ]));
}
