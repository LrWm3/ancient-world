//! Recipe requirements explain a possible response; they are not a work grant.
use super::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum RequirementKind {
    EntryStock,
    MonthlyService,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Requirement {
    pub resource: ResourceId,
    pub kind: RequirementKind,
    pub from: u32,
    pub through: u32,
    /// First-batch entry amount, or per-month service for its indicated stage.
    pub quantity: i64,
    /// Current uncommitted stock allocated once within this explanation only.
    /// Services remain unreserved: old monthly capacity is not future labor.
    pub covered_now: i64,
    pub missing_now: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Route {
    pub request: usize,
    pub definition: DefinitionId,
    pub floor: i32,
    pub target: i32,
    pub batches: i64,
    /// First batch only; later batches require further scheduling.
    pub first_output_available: u32,
    pub first_batch_meets_date: bool,
    pub protected_output_now: i128,
    pub price_crosses: bool,
    pub process_permitted_now: bool,
    pub office_occupied_now: bool,
    pub market_eligible_now: bool,
    pub site_kind_required: Option<u32>,
    pub equipment_kind_required: Option<u32>,
    pub output_room_now: i32,
    /// Deliberately false: a request never verifies future counterparty funding.
    pub buyer_funding_verified: bool,
    pub requirements: Vec<Requirement>,
}

pub fn explain(w: &World, s: &State, snapshot: &Snapshot) -> Result<Vec<Route>, String> {
    if snapshot.month != s.month || s.phase != Phase::Open {
        return Err("stale work request explanation".into());
    }
    let actor = snapshot.actor;
    let Some(a) = w.agency.get(&actor).and_then(|c| c.operating_agreement()) else {
        return Ok(vec![]);
    };
    let horizon = w
        .minting
        .as_ref()
        .and_then(|c| c.order_policy.as_ref())
        .map_or(1, crate::minting::orders::input_claim_months);
    let protected = crate::minting::orders::stock_protection(w, s, actor, horizon)?;
    let used = crate::storage::usage(w, &s.balances);
    let mut result = vec![];
    for (index, request) in snapshot.requests.iter().enumerate() {
        let Some(options) = a
            .stock_markets
            .get(&request.market)
            .filter(|o| o.side == Side::Sell)
        else {
            continue;
        };
        for definition in &a.processes {
            let d = w.definition(*definition);
            if !d.enabled || d.execution != Execution::Productive {
                continue;
            }
            let output: i64 = d
                .outputs
                .iter()
                .filter(|o| o.resource == request.resource)
                .map(|o| i64::from(o.quantity))
                .sum();
            if output <= 0 {
                continue;
            }
            for &floor in &options.holdings {
                let target = i64::from(floor)
                    + i64::from(request.lots.min(options.max_lots))
                        * i64::from(request.lot_quantity);
                let target = i32::try_from(target).map_err(|_| "request work target overflow")?;
                let available = |r| {
                    (i128::from(s.balance(actor, r)) - protected.get(&r).copied().unwrap_or(0))
                        .max(0) as i64
                };
                let deficit =
                    (i64::from(target) - i64::from(s.balance(actor, request.resource))).max(0);
                let batches = (deficit + output - 1) / output;
                let start = s.month.checked_add(1).ok_or("work date overflow")?;
                let delivery = start
                    .checked_add(if batches > 0 { d.duration() } else { 0 })
                    .ok_or("work date overflow")?;
                let mut requirements = vec![];
                let mut remaining = BTreeMap::new();
                let mut from = start;
                for stage in d.stages.iter().filter(|_| batches > 0) {
                    let next = from
                        .checked_add(stage.months)
                        .ok_or("work stage date overflow")?;
                    for input in &stage.entry_inputs {
                        let quantity = i64::from(input.quantity);
                        let stock = w
                            .resources
                            .iter()
                            .any(|r| r.id == input.resource && r.kind == ResourceKind::Stock);
                        let pool = remaining
                            .entry(input.resource)
                            .or_insert_with(|| if stock { available(input.resource) } else { 0 });
                        let covered = quantity.min(*pool);
                        *pool -= covered;
                        requirements.push(Requirement {
                            resource: input.resource,
                            kind: RequirementKind::EntryStock,
                            from,
                            through: from,
                            quantity,
                            covered_now: covered,
                            missing_now: quantity - covered,
                        });
                    }
                    for service in &stage.monthly_services {
                        let quantity = i64::from(service.quantity);
                        requirements.push(Requirement {
                            resource: service.resource,
                            kind: RequirementKind::MonthlyService,
                            from,
                            through: next - 1,
                            quantity,
                            covered_now: 0,
                            missing_now: quantity,
                        });
                    }
                    from = next;
                }
                result.push(Route {
                    request: index,
                    definition: *definition,
                    floor,
                    target,
                    batches,
                    first_output_available: delivery,
                    protected_output_now: protected.get(&request.resource).copied().unwrap_or(0),
                    first_batch_meets_date: request.due.is_none_or(|due| delivery <= due),
                    price_crosses: options.limits.iter().any(|p| *p <= request.limit),
                    process_permitted_now: crate::opportunities::permits(
                        w,
                        s,
                        actor,
                        Action::Process(*definition),
                    ),
                    office_occupied_now: crate::agency::governor(w, s, actor).is_some(),
                    market_eligible_now: crate::minting::eligible(w, s, request.venue, actor),
                    site_kind_required: d.asset_kind,
                    equipment_kind_required: w.activities.required.get(definition).copied(),
                    output_room_now: crate::storage::room(w, &used, actor, request.resource),
                    buyer_funding_verified: false,
                    requirements,
                });
                if result.len() > super::super::MAX_PROGRAMS {
                    return Err("request route budget exceeded".into());
                }
            }
        }
    }
    Ok(result)
}
