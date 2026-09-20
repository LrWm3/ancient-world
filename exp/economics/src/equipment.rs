//! Finite barter and stage-specific durable equipment. No seller/agent-kind branches.
use crate::model::*;
use std::collections::BTreeSet;

const MAX_OUTPUT_MULTIPLIER: u32 = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurableAsset {
    pub id: AssetId,
    pub owner: AgentId,
    pub attached_to: Option<AssetId>,
    pub kind: u32,
    pub remaining_uses: u32,
    pub last_used_month: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub id: u32,
    pub seller: AgentId,
    pub asset: AssetId,
    pub price: Amount,
}
/// An optional stage variant: identical inputs/output/timing, alternate services.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Technique {
    /// Multiplies non-recycled output only when used for the completion stage.
    pub output_multiplier: u32,
    pub id: u32,
    pub definition: DefinitionId,
    pub stage: usize,
    pub equipment_kind: Option<u32>,
    pub competency: Option<(u32, u32)>,
    pub wear: u32,
    pub services: Vec<Amount>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TechniqueUse {
    pub technique: u32,
    pub asset: Option<AssetId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trade {
    pub offer: u32,
    pub buyer: AgentId,
}

pub fn transaction(world: &World, state: &State, trade: Trade) -> Result<Transaction, String> {
    let offer = world
        .offers
        .iter()
        .find(|o| o.id == trade.offer)
        .ok_or("unknown offer")?;
    let asset = state
        .equipment
        .get(&offer.asset)
        .ok_or("missing offered equipment")?;
    if state.phase != Phase::Acquire
        || state.filled_offers.contains(&offer.id)
        || asset.attached_to.is_some()
        || asset.owner != offer.seller
        || asset.remaining_uses == 0
        || asset.last_used_month == Some(state.month)
        || trade.buyer == offer.seller
        || !world.agents.iter().any(|a| a.id == trade.buyer)
        || state.terminal.contains_key(&trade.buyer)
        || state.terminal.contains_key(&offer.seller)
        || state.balance(trade.buyer, offer.price.resource) < offer.price.quantity
    {
        return Err("unavailable or unaffordable equipment offer".into());
    }
    Ok(Transaction {
        cause: format!("accept equipment offer {}", offer.id),
        effects: vec![
            Effect {
                account: (trade.buyer, offer.price.resource),
                delta: -offer.price.quantity,
            },
            Effect {
                account: (offer.seller, offer.price.resource),
                delta: offer.price.quantity,
            },
        ],
        process: None,
        technique_use: None,
        trade: Some(trade),
        stock_trade: None,
        forward: None,
        delivery: None,
        royalty: None,
    })
}

pub fn apply_trade(world: &World, state: &mut State, t: &Transaction) -> Result<(), String> {
    let trade = t.trade.as_ref().ok_or("missing trade")?;
    let expected = transaction(world, state, trade.clone())?;
    if t.effects != expected.effects || t.process.is_some() || t.technique_use.is_some() {
        return Err("trade legs disagree with posted offer".into());
    }
    let offer = world.offers.iter().find(|o| o.id == trade.offer).unwrap();
    state.equipment.get_mut(&offer.asset).unwrap().owner = trade.buyer;
    state.filled_offers.insert(offer.id);
    Ok(())
}

pub fn services<'a>(world: &'a World, usage: &TechniqueUse) -> &'a [Amount] {
    &world
        .techniques
        .iter()
        .find(|t| t.id == usage.technique)
        .expect("validated technique")
        .services
}

/// Validate binding, exclusive monthly use and wear before applying it to staging.
pub fn apply_use(
    world: &World,
    opening: &State,
    state: &mut State,
    t: &Transaction,
) -> Result<(), String> {
    let usage = t.technique_use.as_ref().ok_or("missing equipment use")?;
    let change = t
        .process
        .as_ref()
        .ok_or("equipment must attach to process work")?;
    let technique = world
        .techniques
        .iter()
        .find(|v| v.id == usage.technique)
        .ok_or("unknown technique")?;
    let stage = change.before.as_ref().map(|p| p.stage).unwrap_or(0);
    if state.phase != Phase::Productive
        || change.after.status == Status::Aborted
        || technique.definition != change.after.definition
        || technique.stage != stage
        || !eligible(technique, opening, change.after.operator)
    {
        return Err("ineligible technique".into());
    }
    match (technique.equipment_kind, usage.asset) {
        (None, None) => {}
        (Some(kind), Some(id)) => {
            let asset = state
                .equipment
                .get_mut(&id)
                .ok_or("unknown equipment asset")?;
            if asset.owner != change.after.operator
                || asset.kind != kind
                || asset.remaining_uses < technique.wear
                || asset.last_used_month == Some(state.month)
            {
                return Err("invalid, exhausted or already reserved equipment".into());
            }
            asset.remaining_uses -= technique.wear;
            asset.last_used_month = Some(state.month);
        }
        _ => return Err("technique equipment binding mismatch".into()),
    }
    Ok(())
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let mut rules = BTreeSet::new();
    for rule in &world.practice_rules {
        if rule.points == 0
            || !rules.insert((rule.definition, rule.stage, rule.competency))
            || !world.definitions.iter().any(|d| {
                d.id == rule.definition
                    && d.execution == Execution::Productive
                    && rule.stage < d.stages.len()
            })
        {
            return Err("invalid practice rule".into());
        }
    }
    for &(agent, kind) in state.practice.keys() {
        if !world.agents.iter().any(|a| a.id == agent)
            || !world.practice_rules.iter().any(|r| r.competency == kind)
        {
            return Err("invalid competency account".into());
        }
    }
    let mut ids = BTreeSet::new();
    for technique in &world.techniques {
        if !(1..=MAX_OUTPUT_MULTIPLIER).contains(&technique.output_multiplier) {
            return Err("invalid technique output multiplier".into());
        }
        let definition = world
            .definitions
            .iter()
            .find(|d| d.id == technique.definition)
            .ok_or("unknown technique process")?;
        if !ids.insert(technique.id)
            || (technique.equipment_kind.is_some() != (technique.wear > 0))
            || technique.competency.is_some_and(|(kind, threshold)| {
                threshold == 0 || !world.practice_rules.iter().any(|r| r.competency == kind)
            })
            || technique.stage >= definition.stages.len()
            || definition.execution != Execution::Productive
            || technique.services.iter().any(|a| {
                a.quantity <= 0
                    || !world
                        .resources
                        .iter()
                        .any(|r| r.id == a.resource && r.kind == ResourceKind::Capacity)
            })
        {
            return Err("invalid equipment technique".into());
        }
    }
    ids.clear();
    for offer in &world.offers {
        if !ids.insert(offer.id)
            || !world.agents.iter().any(|a| a.id == offer.seller)
            || !state.equipment.contains_key(&offer.asset)
            || offer.price.quantity <= 0
            || !world
                .resources
                .iter()
                .any(|r| r.id == offer.price.resource && r.kind == ResourceKind::Stock)
        {
            return Err("invalid equipment offer".into());
        }
    }
    if state.filled_offers.iter().any(|id| !ids.contains(id)) {
        return Err("unknown filled offer".into());
    }
    for (&id, asset) in &state.equipment {
        if id != asset.id
            || world.assets.iter().any(|a| a.id == id)
            || !world.agents.iter().any(|a| a.id == asset.owner)
            || asset
                .last_used_month
                .is_some_and(|m| m == 0 || m > state.month)
        {
            return Err("invalid durable asset".into());
        }
    }
    if let Some(plan) = &state.pending_production
        && (state.phase != Phase::Productive
            || plan.phase != Phase::Productive
            || plan.month != state.month
            || plan.id != state.next_batch
            || plan.production_plan.is_some()
            || plan.decision.is_some())
    {
        return Err("stale or nested production plan".into());
    }
    Ok(())
}

// Practice is derived from validated stage-completion records, never requests.
// The process transition in the ledger is the provenance for each award.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PracticeRule {
    pub definition: DefinitionId,
    pub stage: usize,
    pub competency: u32,
    pub points: u32,
}

pub fn eligible(technique: &Technique, state: &State, operator: AgentId) -> bool {
    technique.competency.is_none_or(|(kind, minimum)| {
        state.practice.get(&(operator, kind)).copied().unwrap_or(0) >= minimum
    })
}

pub fn award(world: &World, state: &mut State, change: &ProcessChange) -> Result<(), String> {
    let stage = change.before.as_ref().map(|p| p.stage).unwrap_or(0);
    if change.after.status == Status::Aborted
        || (change.after.status != Status::Completed && change.after.stage == stage)
    {
        return Ok(());
    }
    for rule in world
        .practice_rules
        .iter()
        .filter(|r| r.definition == change.after.definition && r.stage == stage)
    {
        let points = state
            .practice
            .entry((change.after.operator, rule.competency))
            .or_default();
        *points = points.checked_add(rule.points).ok_or("practice overflow")?;
    }
    Ok(())
}
