//! Explicit, dated cash sales during solvent wind-down. No auction or forced sale.
use super::*;
use crate::{asset_exchange, credit, finance, opportunities};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sale {
    pub month: u32,
    pub asset: AssetId,
    pub buyer: AgentId,
    pub price: Amount,
    /// Explicitly included equipment attached to the catalog asset. The total
    /// price includes these amounts; the remainder is the catalog asset's cost.
    pub attachments: Vec<Attachment>,
    /// Explicit acceptance of title-following use rights and unfinished work.
    pub control: Option<Control>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Control {
    pub rights: Vec<u32>,
    pub processes: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachment {
    pub asset: AssetId,
    /// Allocated consideration in the sale's denomination (not seller basis).
    pub consideration: i32,
}

impl Sale {
    pub(crate) fn assets(&self) -> impl Iterator<Item = AssetId> + '_ {
        std::iter::once(self.asset).chain(self.attachments.iter().map(|a| a.asset))
    }

    /// Supplied allocations must exactly exhaust one positive package price.
    /// Equipment may have zero cost. The catalog registry still requires a
    /// positive root value; no appraisal is inferred from condition.
    pub(crate) fn values(&self) -> Result<Vec<(AssetId, i32)>, String> {
        let mut remaining = self.price.quantity;
        let mut ids = BTreeSet::from([self.asset]);
        if remaining <= 0 {
            return Err("asset sale requires a positive total price".into());
        }
        for a in &self.attachments {
            if !ids.insert(a.asset) || a.consideration < 0 || a.consideration > remaining {
                return Err("invalid asset package price allocation".into());
            }
            remaining -= a.consideration;
        }
        if remaining == 0 {
            return Err("asset package requires positive root consideration".into());
        }
        Ok(std::iter::once((self.asset, remaining))
            .chain(self.attachments.iter().map(|a| (a.asset, a.consideration)))
            .collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    Unavailable,
    NotExhausted,
    Encumbered,
    Attached,
    Permission,
    FundingOrStorage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub household: AgentId,
    pub sale: Sale,
    pub rejection: Option<Rejection>,
    /// Opening condition, present for durable equipment. Replay checks it; the
    /// transfer preserves wear and attachment state rather than creating a tool.
    pub equipment: Option<crate::equipment::DurableAsset>,
    /// Complete opening attachment set, including on rejected package sales.
    pub attachments: Vec<crate::equipment::DurableAsset>,
    pub processes: Vec<ProcessChange>,
}

/// Supply mutually accepted sale terms at Open. The last member authorizes the
/// household side; the named buyer's consent is an explicit scenario input.
/// Funds and permissions are checked again at execution. Failed sales expire.
pub fn accept(
    world: &mut World,
    state: &State,
    household: AgentId,
    person: AgentId,
    mut sale: Sale,
) -> Result<(), String> {
    sale.values()?;
    sale.attachments.sort_by_key(|a| a.asset);
    if let Some(c) = &mut sale.control {
        c.rights.sort();
        c.processes.sort();
    }
    let a = world
        .households
        .iter()
        .find(|a| a.agent == household)
        .ok_or("unknown household")?;
    if state.phase != Phase::Open
        || sale.month != state.month
        || state.pending_production.is_some()
        || dissolution::winding_at(a, state.month).is_none_or(|(_, p, _)| p != person)
        || rejection(world, state, a, &sale).is_some()
    {
        return Err("asset disposal requires a living last member, wind-down and transferable property at Open".into());
    }
    let mut next = world.clone();
    next.households
        .iter_mut()
        .find(|a| a.agent == household)
        .unwrap()
        .asset_sales
        .push(sale);
    crate::settlement::validate_world(&next, state)?;
    *world = next;
    Ok(())
}

pub(super) fn validate(world: &World, state: &State) -> Result<(), String> {
    for a in &world.households {
        let mut dated = BTreeSet::new();
        for s in &a.asset_sales {
            s.values()?;
            if let Some(c) = &s.control
                && (!world.assets.iter().any(|a| a.id == s.asset)
                    || c.rights.iter().copied().collect::<BTreeSet<_>>().len() != c.rights.len()
                    || c.processes.iter().copied().collect::<BTreeSet<_>>().len()
                        != c.processes.len()
                    || c.rights
                        .iter()
                        .any(|id| !world.rights.iter().any(|r| r.id == *id))
                    || c.processes
                        .iter()
                        .any(|id| !state.processes.contains_key(id)))
            {
                return Err("invalid asset control transfer terms".into());
            }
            if s.assets().any(|asset| !dated.insert((s.month, asset)))
                || dissolution::winding_at(a, s.month).is_none()
                || dissolution::closed_at(a, s.month)
                || s.buyer == a.agent
                || s.price.quantity <= 0
                || !world.agents.iter().any(|b| b.id == s.buyer)
                || (!world.assets.iter().any(|b| b.id == s.asset)
                    && !state.equipment.contains_key(&s.asset)
                    && !state.retired_equipment.contains_key(&s.asset))
                || !world
                    .resources
                    .iter()
                    .any(|r| r.id == s.price.resource && r.kind == ResourceKind::Stock)
                || world.activities.perishable.contains(&s.price.resource)
                || (!s.attachments.is_empty()
                    && (!world.assets.iter().any(|b| b.id == s.asset)
                        || s.attachments.iter().any(|b| {
                            !state.equipment.contains_key(&b.asset)
                                && !state.retired_equipment.contains_key(&b.asset)
                        })))
            {
                return Err("invalid dated household asset sale".into());
            }
        }
    }
    Ok(())
}

fn rejection(world: &World, state: &State, a: &Agreement, sale: &Sale) -> Option<Rejection> {
    let person = dissolution::winding_at(a, state.month).map(|(_, p, _)| p);
    if person.is_none_or(|p| state.terminal.contains_key(&p))
        || dissolution::closed_at(a, state.month)
        || state.terminal.contains_key(&sale.buyer)
        || world
            .households
            .iter()
            .any(|h| h.agent == sale.buyer && dissolution::winding_at(h, state.month).is_some())
        || asset_exchange::owner(world, state, &state.credit, sale.asset) != Some(a.agent)
    {
        return Some(Rejection::Unavailable);
    }
    if let Some(tool) = state.equipment.get(&sale.asset) {
        if tool.attached_to.is_some() {
            return Some(Rejection::Attached);
        }
        if !crate::equipment::transferable(tool, state.month) {
            return Some(Rejection::Unavailable);
        }
    }
    if let Some(reason) = package_rejection(world, state, a.agent, sale) {
        return Some(reason);
    }
    if [a.agent, sale.buyer]
        .into_iter()
        .any(|who| !opportunities::permits(world, state, who, opportunities::Action::AssetTrade))
    {
        return Some(Rejection::Permission);
    }
    None
}

pub(super) fn prepare(world: &World, state: &State) -> Result<(Vec<Receipt>, Vec<Effect>), String> {
    let mut sales: Vec<_> = world
        .households
        .iter()
        .flat_map(|a| {
            a.asset_sales
                .iter()
                .filter(|s| s.month == state.month)
                .map(move |s| (a, s))
        })
        .collect();
    // Explicit stable priority for accepted sales sharing the buyer's opening
    // budget. Receipts never replenish this boundary's spending allowance.
    sales.sort_by_key(|(a, s)| (a.agent, s.asset));
    let mut execution = finance::Execution::opening(world, state);
    let mut receipts = vec![];
    let mut effects = vec![];
    for (a, sale) in sales {
        let mut reason = rejection(world, state, a, sale);
        if reason.is_none() {
            match asset_exchange::fund(
                world,
                state,
                &state.credit,
                &mut execution,
                &asset_exchange::AcceptedSale {
                    sale: credit::Sale {
                        asset: sale.asset,
                        seller: a.agent,
                        price: sale.price.clone(),
                    },
                    buyer: sale.buyer,
                    payees: vec![(a.agent, sale.price.quantity)],
                },
            ) {
                Ok(payment) => effects.extend(payment),
                Err(_) => reason = Some(Rejection::FundingOrStorage),
            }
        }
        receipts.push(Receipt {
            household: a.agent,
            sale: sale.clone(),
            rejection: reason.clone(),
            equipment: state.equipment.get(&sale.asset).cloned(),
            attachments: attached(state, sale.asset).cloned().collect(),
            processes: if reason.is_none() && sale.control.is_some() {
                crate::credit::attachment_changes(world, state, sale.asset, sale.buyer)
            } else {
                vec![]
            },
        });
    }
    Ok((receipts, effects))
}

pub(super) fn publish(state: &mut State, receipts: &[Receipt]) {
    for r in receipts.iter().filter(|r| r.rejection.is_none()) {
        if r.equipment.is_some() {
            state
                .equipment
                .get_mut(&r.sale.asset)
                .expect("verified durable disposal")
                .owner = r.sale.buyer;
        } else {
            state.credit.owners.insert(r.sale.asset, r.sale.buyer);
            state.credit.values.insert(
                r.sale.asset,
                r.sale.values().expect("verified sale values")[0].1,
            );
        }
        for tool in &r.attachments {
            state
                .equipment
                .get_mut(&tool.id)
                .expect("verified attachment")
                .owner = r.sale.buyer;
        }
        for p in &r.processes {
            state.processes.insert(p.after.id, p.after.clone());
        }
    }
}

/// Shared restrictions for disposal and permanent retirement. Neither path may
/// discard another party's claim, a standing offer or an attached obligation.
pub(super) fn restrictions(world: &World, state: &State, asset: AssetId) -> Option<Rejection> {
    restrictions_except_equipment(world, state, asset)
        .or_else(|| attached(state, asset).next().map(|_| Rejection::Attached))
}

fn attached(
    state: &State,
    asset: AssetId,
) -> impl Iterator<Item = &crate::equipment::DurableAsset> {
    state
        .equipment
        .values()
        .filter(move |a| a.attached_to == Some(asset))
}

fn package_rejection(
    world: &World,
    state: &State,
    seller: AgentId,
    sale: &Sale,
) -> Option<Rejection> {
    if sale.attachments.is_empty() {
        if attached(state, sale.asset).next().is_some() {
            return Some(Rejection::Attached);
        }
        return control_rejection(world, state, seller, sale);
    }
    if !world.assets.iter().any(|a| a.id == sale.asset)
        || sale
            .attachments
            .iter()
            .map(|a| a.asset)
            .collect::<BTreeSet<_>>()
            != attached(state, sale.asset).map(|a| a.id).collect()
    {
        return Some(Rejection::Attached);
    }
    if let Some(reason) = control_rejection(world, state, seller, sale) {
        return Some(reason);
    }
    for tool in attached(state, sale.asset) {
        if tool.owner != seller
            || tool.remaining_uses == 0
            || tool.last_used_month == Some(state.month)
        {
            return Some(Rejection::Unavailable);
        }
        if let Some(reason) = restrictions(world, state, tool.id) {
            return Some(reason);
        }
    }
    None
}

fn control_rejection(
    world: &World,
    state: &State,
    seller: AgentId,
    sale: &Sale,
) -> Option<Rejection> {
    let Some(c) = &sale.control else {
        return restrictions_except_equipment(world, state, sale.asset);
    };
    if let Some(reason) = encumbrances(world, state, sale.asset) {
        return Some(reason);
    }
    let rights: Vec<_> = world
        .rights
        .iter()
        .filter(|r| r.asset == sale.asset && r.through >= state.month)
        .collect();
    let processes: Vec<_> = state
        .processes
        .values()
        .filter(|p| p.asset == Some(sale.asset) && p.status == Status::Active)
        .collect();
    if rights.iter().map(|r| r.id).collect::<BTreeSet<_>>() != c.rights.iter().copied().collect()
        || processes.iter().map(|p| p.id).collect::<BTreeSet<_>>()
            != c.processes.iter().copied().collect()
        || rights.iter().any(|r| {
            !crate::credit::follows_owner(world, r.id)
                || crate::commitments::holder(world, state, r) != Some(seller)
                || crate::commitments::output_owner(world, state, r) != Some(seller)
                || crate::commitments::active(world, state).any(|a| a.right == r.id)
        })
        || processes.iter().any(|p| {
            p.operator != seller
                || p.beneficiary != seller
                || p.right.is_none_or(|id| !c.rights.contains(&id))
        })
    {
        return Some(Rejection::Attached);
    }
    if !opportunities::permits(world, state, sale.buyer, opportunities::Action::LandAccess)
        || processes.iter().any(|p| {
            !opportunities::permits(
                world,
                state,
                sale.buyer,
                opportunities::Action::Process(p.definition),
            )
        })
    {
        return Some(Rejection::Permission);
    }
    None
}

pub(super) fn restrictions_except_equipment(
    world: &World,
    state: &State,
    asset: AssetId,
) -> Option<Rejection> {
    if let Some(reason) = encumbrances(world, state, asset) {
        return Some(reason);
    }
    if world
        .rights
        .iter()
        .any(|r| r.asset == asset && r.through >= state.month)
        || state
            .processes
            .values()
            .any(|p| p.asset == Some(asset) && p.status == Status::Active)
    {
        return Some(Rejection::Attached);
    }
    None
}

fn encumbrances(world: &World, state: &State, asset: AssetId) -> Option<Rejection> {
    if state.exchange.contracts.contains_key(&asset)
        || world
            .offers
            .iter()
            .any(|o| o.asset == asset && !state.filled_offers.contains(&o.id))
    {
        return Some(Rejection::Encumbered);
    }
    if state.credit.loans.values().any(|l| {
        l.collateral
            .as_ref()
            .is_some_and(|c| c.asset == asset && c.pledged)
    }) || world
        .lending
        .iter()
        .any(|l| l.month >= state.month && l.collateral.as_ref().is_some_and(|c| c.asset == asset))
        || crate::credit::pending_purchase(world, state).is_some_and(|o| o.sale.asset == asset)
        || world.recovery.proceedings.iter().any(|p| {
            p.assets.iter().any(|l| l.asset == asset)
                && state
                    .credit
                    .recovery
                    .proceedings
                    .get(&p.id)
                    .is_none_or(|case| !case.sold.contains(&asset))
        })
    {
        return Some(Rejection::Encumbered);
    }
    None
}
