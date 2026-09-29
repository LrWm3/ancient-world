//! Explicit withdrawal, write-off and material recovery during wind-down.
use super::*;
use crate::{equipment::RetiredAsset, opportunities};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Exhausted,
    Discard,
    Recover,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub month: u32,
    pub asset: AssetId,
    pub mode: Mode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub household: AgentId,
    pub request: Request,
    pub equipment: Option<crate::equipment::DurableAsset>,
    pub rejection: Option<disposal::Rejection>,
}

/// The last member explicitly relinquishes repair/salvage options. This records
/// an instruction only; Open rechecks it and commits the archived equipment.
pub fn request(
    world: &mut World,
    state: &State,
    household: AgentId,
    person: AgentId,
    asset: AssetId,
) -> Result<(), String> {
    request_with_mode(world, state, household, person, asset, Mode::Exhausted)
}

pub fn request_with_mode(
    world: &mut World,
    state: &State,
    household: AgentId,
    person: AgentId,
    asset: AssetId,
    mode: Mode,
) -> Result<(), String> {
    let a = world
        .households
        .iter()
        .find(|h| h.agent == household)
        .ok_or("unknown household")?;
    if state.phase != Phase::Open
        || state.pending_production.is_some()
        || dissolution::winding_at(a, state.month).is_none_or(|(_, p, _)| p != person)
        || rejection(world, state, a, asset, &mode).is_some()
    {
        return Err("retirement requires a living last member, wind-down and equipment eligible for the selected retirement mode at Open".into());
    }
    let mut next = world.clone();
    next.households
        .iter_mut()
        .find(|h| h.agent == household)
        .unwrap()
        .equipment_retirements
        .push(Request {
            month: state.month,
            asset,
            mode,
        });
    crate::settlement::validate_world(&next, state)?;
    *world = next;
    Ok(())
}

fn rejection(
    world: &World,
    state: &State,
    a: &Agreement,
    asset: AssetId,
    mode: &Mode,
) -> Option<disposal::Rejection> {
    use disposal::Rejection;
    let Some(tool) = state.equipment.get(&asset) else {
        return Some(Rejection::Unavailable);
    };
    if dissolution::winding_at(a, state.month)
        .is_none_or(|(_, p, _)| state.terminal.contains_key(&p))
        || dissolution::closed_at(a, state.month)
        || tool.owner != a.agent
        || tool.last_used_month == Some(state.month)
        || state.retired_equipment.contains_key(&asset)
    {
        return Some(Rejection::Unavailable);
    }
    if *mode == Mode::Exhausted && tool.remaining_uses != 0 {
        return Some(Rejection::NotExhausted);
    }
    if let Some(parent) = tool.attached_to {
        if *mode == Mode::Exhausted || crate::credit::owner(world, state, parent) != Some(a.agent) {
            return Some(Rejection::Attached);
        }
        if let Some(reason) = disposal::restrictions_except_equipment(world, state, parent) {
            return Some(reason);
        }
    }
    if *mode == Mode::Recover && !world.activities.salvage.contains_key(&tool.kind) {
        return Some(Rejection::Unavailable);
    }
    if let Some(reason) = disposal::restrictions(world, state, asset) {
        return Some(reason);
    }
    if !opportunities::permits(
        world,
        state,
        a.agent,
        opportunities::Action::RetireEquipment,
    ) {
        return Some(Rejection::Permission);
    }
    None
}

pub(super) fn validate(world: &World, state: &State) -> Result<(), String> {
    for a in &world.households {
        let mut dated = BTreeSet::new();
        for r in &a.equipment_retirements {
            if !dated.insert((r.month, r.asset))
                || dissolution::winding_at(a, r.month).is_none()
                || dissolution::closed_at(a, r.month)
                || (!state.equipment.contains_key(&r.asset)
                    && !state.retired_equipment.contains_key(&r.asset))
                || a.asset_sales
                    .iter()
                    .any(|s| s.month == r.month && s.assets().any(|asset| asset == r.asset))
            {
                return Err("invalid or conflicting equipment retirement instruction".into());
            }
        }
    }
    for (&asset, r) in &state.retired_equipment {
        if !world.households.iter().any(|h| {
            h.agent == r.equipment.owner
                && h.equipment_retirements.iter().any(|q| {
                    q.asset == asset
                        && q.month == r.month
                        && (q.mode != Mode::Exhausted
                            || (r.equipment.remaining_uses == 0
                                && r.equipment.attached_to.is_none()))
                })
        }) || disposal::restrictions(world, state, asset).is_some()
        {
            return Err(
                "retired equipment lacks authority or acquired an active claim/attachment".into(),
            );
        }
    }
    Ok(())
}

pub(super) fn prepare(world: &World, state: &State) -> Result<(Vec<Receipt>, Vec<Effect>), String> {
    let mut receipts: Vec<_> = world
        .households
        .iter()
        .flat_map(|a| {
            a.equipment_retirements
                .iter()
                .filter(|r| r.month == state.month)
                .map(move |r| Receipt {
                    household: a.agent,
                    request: r.clone(),
                    equipment: state.equipment.get(&r.asset).cloned(),
                    rejection: rejection(world, state, a, r.asset, &r.mode),
                })
        })
        .collect();
    receipts.sort_by_key(|r| (r.household, r.request.asset));
    let mut used = crate::storage::usage(world, &state.balances);
    let mut effects = vec![];
    let mut balances = state.balances.clone();
    for r in &mut receipts {
        if r.rejection.is_some() || r.request.mode != Mode::Recover {
            continue;
        }
        let additions: Vec<_> = world.activities.salvage[&r.equipment.as_ref().unwrap().kind]
            .iter()
            .map(|a| Effect {
                account: (r.household, a.resource),
                delta: a.quantity,
            })
            .collect();
        if !crate::storage::fits(world, &used, &additions)
            || additions.iter().any(|e| {
                balances
                    .get(&e.account)
                    .copied()
                    .unwrap_or(0)
                    .checked_add(e.delta)
                    .is_none()
            })
        {
            r.rejection = Some(disposal::Rejection::FundingOrStorage);
            continue;
        }
        crate::storage::apply(world, &mut used, &additions);
        for e in &additions {
            *balances.entry(e.account).or_default() += e.delta;
        }
        effects.extend(additions);
    }
    Ok((receipts, effects))
}

pub(super) fn publish(state: &mut State, receipts: &[Receipt]) {
    for r in receipts.iter().filter(|r| r.rejection.is_none()) {
        let equipment = state
            .equipment
            .remove(&r.request.asset)
            .expect("verified equipment retirement");
        state.retired_equipment.insert(
            r.request.asset,
            RetiredAsset {
                equipment,
                month: r.request.month,
                batch: state.next_batch,
            },
        );
    }
}
