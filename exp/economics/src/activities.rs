//! Catalog-driven work targets and durable production/repair. Completed process
//! records are the provenance; no occupation or resource IDs enter this module.
use crate::{equipment::DurableAsset, model::*};
use std::collections::{BTreeMap, BTreeSet};

const PRODUCED_ASSET_ID_BASE: u32 = 1_000_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurableKind {
    pub name: String,
    pub lifetime: u32,
    pub attached: bool,
    pub monthly_decay: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Create(u32),
    Repair { kind: u32, restore: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Stock(Amount),
    Assets { kind: u32, count: usize },
    Maintain { kind: u32, below: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkOrder {
    pub agent: AgentId,
    pub definition: DefinitionId,
    pub priority: u32,
    pub target: Target,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoinPayment {
    pub resource: ResourceId,
    pub coins_per_unit: i32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Activities {
    pub kinds: BTreeMap<u32, DurableKind>,
    pub outcomes: BTreeMap<DefinitionId, Outcome>,
    /// Mandatory productive asset, independent of an optional labor-saving tool.
    pub required: BTreeMap<DefinitionId, u32>,
    /// Read/use access without exclusively occupying the whole parcel.
    pub shared_sites: BTreeSet<DefinitionId>,
    pub orders: Vec<WorkOrder>,
    /// Services represented as consumption tickets expire at the next Open.
    pub perishable: BTreeSet<ResourceId>,
    pub coin_payments: BTreeMap<u32, CoinPayment>,
}

pub fn attached_access(
    world: &World,
    state: &State,
    asset: &DurableAsset,
    site: Option<AssetId>,
) -> bool {
    asset.attached_to.is_none_or(|plot| {
        site == Some(plot)
            && world.rights.iter().any(|r| {
                r.asset == plot
                    && r.holder == asset.owner
                    && r.from <= state.month
                    && r.through >= state.month
            })
    })
}

pub fn bindings(
    world: &World,
    state: &State,
    p: &ProcessInstance,
    reserved: &BTreeMap<AssetId, u64>,
) -> Result<Vec<AssetId>, String> {
    let mut result = Vec::new();
    let eligible = |a: &&DurableAsset| {
        a.owner == p.operator
            && a.last_used_month != Some(state.month)
            && !reserved.contains_key(&a.id)
            && attached_access(world, state, a, p.asset)
    };
    if let Some(kind) = world.activities.required.get(&p.definition) {
        let a = state
            .equipment
            .values()
            .filter(eligible)
            .find(|a| a.kind == *kind && a.remaining_uses > 0)
            .ok_or("missing required productive asset")?;
        result.push(a.id);
    }
    if let Some(Outcome::Repair { kind, .. }) = world.activities.outcomes.get(&p.definition) {
        let max = world
            .activities
            .kinds
            .get(kind)
            .ok_or("unknown repair kind")?
            .lifetime;
        let a = state
            .equipment
            .values()
            .filter(eligible)
            .find(|a| a.kind == *kind && a.remaining_uses < max)
            .ok_or("no available damaged asset")?;
        if result.contains(&a.id) {
            return Err("repair cannot simultaneously use its target".into());
        }
        result.push(a.id);
    }
    Ok(result)
}

pub fn wants(world: &World, state: &State, order: &WorkOrder) -> bool {
    let active = state.processes.values().any(|p| {
        p.operator == order.agent && p.definition == order.definition && p.status == Status::Active
    });
    let d = world.definition(order.definition);
    let spare_site = d.asset_kind.is_some_and(|kind| {
        !world.activities.shared_sites.contains(&d.id)
            && world.rights.iter().any(|r| {
                r.holder == order.agent
                    && r.from <= state.month
                    && r.through >= state.month
                    && crate::commitments::can_start(world, state, r.id)
                    && world
                        .assets
                        .iter()
                        .any(|a| a.id == r.asset && a.kind == kind)
                    && !state.processes.values().any(|p| {
                        p.asset == Some(r.asset)
                            && p.status == Status::Active
                            && !world.activities.shared_sites.contains(&p.definition)
                    })
            })
    });
    if state.terminal.contains_key(&order.agent) || (active && !spare_site) {
        return false;
    }
    match &order.target {
        Target::Stock(a) => state.balance(order.agent, a.resource) < a.quantity,
        Target::Assets { kind, count } => {
            if let Some(wants) = crate::exchange::wants(world, state, order.agent, *kind) {
                return wants;
            }
            state
                .equipment
                .values()
                .filter(|a| a.owner == order.agent && a.kind == *kind)
                .count()
                < *count
        }
        Target::Maintain { kind, below } => state
            .equipment
            .values()
            .any(|a| a.owner == order.agent && a.kind == *kind && a.remaining_uses < *below),
    }
}

/// Applied only after process effects and its dated transition have been validated.
pub fn apply(world: &World, state: &mut State, change: &ProcessChange) -> Result<(), String> {
    let p = &change.after;
    if p.status == Status::Aborted {
        return Ok(());
    }
    let ids = bindings(world, state, p, &BTreeMap::new())?;
    let mut cursor = 0;
    if world.activities.required.contains_key(&p.definition) {
        let a = state.equipment.get_mut(&ids[cursor]).unwrap();
        cursor += 1;
        a.remaining_uses -= 1;
        a.last_used_month = Some(state.month);
    }
    if let Some(Outcome::Repair { kind, restore }) = world.activities.outcomes.get(&p.definition) {
        let a = state.equipment.get_mut(&ids[cursor]).unwrap();
        a.remaining_uses = a
            .remaining_uses
            .saturating_add(*restore)
            .min(world.activities.kinds[kind].lifetime);
        a.last_used_month = Some(state.month);
    }
    if p.status == Status::Completed
        && let Some(Outcome::Create(kind)) = world.activities.outcomes.get(&p.definition)
    {
        let spec = &world.activities.kinds[kind];
        let id = PRODUCED_ASSET_ID_BASE
            .checked_add(u32::try_from(p.id).map_err(|_| "produced asset ID overflow")?)
            .ok_or("produced asset ID overflow")?;
        if state.equipment.contains_key(&id) || world.assets.iter().any(|a| a.id == id) {
            return Err("produced asset ID collision".into());
        }
        let attached_to = if spec.attached {
            Some(p.asset.ok_or("attached output requires a plot")?)
        } else {
            None
        };
        state.equipment.insert(
            id,
            DurableAsset {
                id,
                owner: p.beneficiary,
                kind: *kind,
                remaining_uses: spec.lifetime,
                last_used_month: Some(state.month),
                attached_to,
            },
        );
    }
    Ok(())
}

pub fn age(world: &World, state: &mut State) {
    for asset in state.equipment.values_mut() {
        if let Some(spec) = world.activities.kinds.get(&asset.kind) {
            asset.remaining_uses = asset.remaining_uses.saturating_sub(spec.monthly_decay);
        }
    }
}

pub fn expiration(world: &World, state: &State) -> Vec<Effect> {
    state
        .balances
        .iter()
        .filter(|((_, r), q)| world.activities.perishable.contains(r) && **q > 0)
        .map(|(a, q)| Effect {
            account: *a,
            delta: -*q,
        })
        .collect()
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let stock = |id| {
        world
            .resources
            .iter()
            .any(|r| r.id == id && r.kind == ResourceKind::Stock)
    };
    for kind in world.activities.kinds.values() {
        if kind.lifetime == 0 || kind.monthly_decay > kind.lifetime {
            return Err("zero durable lifetime".into());
        }
    }
    for (&id, outcome) in &world.activities.outcomes {
        let d = world
            .definitions
            .iter()
            .find(|d| d.id == id)
            .ok_or("unknown durable outcome process")?;
        let kind = match outcome {
            Outcome::Create(k) => k,
            Outcome::Repair { kind, restore } => {
                if *restore == 0 || d.duration() != 1 {
                    return Err("repair must complete in one month".into());
                }
                kind
            }
        };
        let spec = world
            .activities
            .kinds
            .get(kind)
            .ok_or("unknown outcome kind")?;
        if d.execution != Execution::Productive || (spec.attached && d.asset_kind.is_none()) {
            return Err("invalid durable outcome site or phase".into());
        }
    }
    for (&d, &kind) in &world.activities.required {
        if !world.activities.kinds.contains_key(&kind)
            || !world.definitions.iter().any(|v| {
                v.id == d
                    && v.execution == Execution::Productive
                    && (!world.activities.kinds[&kind].attached || v.asset_kind.is_some())
            })
            || world
                .techniques
                .iter()
                .any(|t| t.definition == d && t.equipment_kind == Some(kind))
        {
            return Err("invalid required asset binding".into());
        }
    }
    for id in &world.activities.shared_sites {
        if !world
            .definitions
            .iter()
            .any(|d| d.id == *id && d.asset_kind.is_some())
        {
            return Err("invalid shared site use".into());
        }
    }
    let mut orders = BTreeSet::new();
    for o in &world.activities.orders {
        let d = world
            .definitions
            .iter()
            .find(|d| d.id == o.definition)
            .ok_or("unknown work order process")?;
        if !orders.insert((o.agent, o.definition))
            || !world.participants.iter().any(|p| p.agent == o.agent)
            || d.execution != Execution::Productive
        {
            return Err("invalid activity order".into());
        }
        let valid = match &o.target {
            Target::Stock(a) => {
                a.quantity > 0 && d.outputs.iter().any(|v| v.resource == a.resource)
            }
            Target::Assets { kind, count } => {
                *count > 0 && world.activities.outcomes.get(&d.id) == Some(&Outcome::Create(*kind))
            }
            Target::Maintain { kind, below } => {
                *below > 0
                    && world
                        .activities
                        .kinds
                        .get(kind)
                        .is_some_and(|k| *below <= k.lifetime)
                    && matches!(world.activities.outcomes.get(&d.id),Some(Outcome::Repair{kind:k,..}) if k==kind)
            }
        };
        if !valid {
            return Err("work target differs from process outcome".into());
        }
    }
    for (&agreement, p) in &world.activities.coin_payments {
        if p.coins_per_unit <= 0
            || !stock(p.resource)
            || *world.storage.weights.get(&p.resource).unwrap_or(&0) != 0
            || !world
                .agreements
                .iter()
                .chain(&world.access_offers)
                .any(|a| a.id == agreement && a.payment.resource != p.resource)
        {
            return Err("invalid coin payment alternative".into());
        }
    }
    if world.activities.perishable.iter().any(|r| !stock(*r)) {
        return Err("invalid expiring service ticket".into());
    }
    for a in state.equipment.values() {
        if let Some(spec) = world.activities.kinds.get(&a.kind) {
            if a.remaining_uses > spec.lifetime || spec.attached != a.attached_to.is_some() {
                return Err("invalid durable condition or attachment".into());
            }
        } else if a.attached_to.is_some() {
            return Err("attachment requires a durable specification".into());
        }
        if a.attached_to
            .is_some_and(|id| !world.assets.iter().any(|s| s.id == id))
        {
            return Err("unknown attachment plot".into());
        }
    }
    Ok(())
}
