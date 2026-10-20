//! One-hop conditional interest from explicit production goals. No work, bid,
//! financing or dependency is accepted by announcing a first-batch input gap.
use super::*;
use crate::agency::{
    self,
    objectives::{Metric, Scope},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Dependency {
    pub definition: DefinitionId,
    pub governor: AgentId,
    pub goal_target: u32,
    /// Aspirational output window, not a promised delivery or scheduled start.
    pub output_by: u32,
    pub horizon: u32,
    pub needed: i32,
    pub protected_stock: i32,
    pub signed_target: i32,
    pub ceiling: i32,
}

fn lots(r: &Request, d: &Dependency) -> i32 {
    let available = r.observed_stock - d.protected_stock;
    crate::minting::orders::required_lots(d.needed, available, r.lot_quantity)
        .min(crate::minting::orders::required_lots(
            d.signed_target,
            r.observed_stock,
            r.lot_quantity,
        ))
        .min(d.ceiling)
}

pub(super) fn append(w: &World, s: &State, snapshot: &mut Snapshot) -> Result<(), String> {
    snapshot.requests.extend(collect(w, s, snapshot, None)?);
    Ok(())
}

/// Own conditional interest for optional, separately governed bid publication.
pub(crate) fn own(w: &World, s: &State, actor: AgentId) -> Result<Vec<Request>, String> {
    collect(
        w,
        s,
        &Snapshot {
            month: s.month,
            actor,
            requests: vec![],
        },
        Some(actor),
    )
}

fn collect(
    w: &World,
    s: &State,
    snapshot: &Snapshot,
    only: Option<AgentId>,
) -> Result<Vec<Request>, String> {
    let mint = w
        .minting
        .as_ref()
        .ok_or("missing upstream market adapter")?;
    let policy = mint
        .order_policy
        .as_ref()
        .ok_or("missing upstream order policy")?;
    let venue = crate::marketplace::venue(w, mint.venue).ok_or("missing upstream venue")?;
    let mut candidates: BTreeMap<_, Request> = BTreeMap::new();
    for (&requester, c) in &w.agency {
        if only.map_or(requester == snapshot.actor, |actor| requester != actor)
            || c.config.work_requests != Policy::Upstream
            || !c.autonomous
            || !c.discover_programs
            || agency::forwards::execution::pending(w, requester)
            || !crate::minting::eligible(w, s, mint.venue, requester)
        {
            continue;
        }
        let Some(agreement) = c.operating_agreement() else {
            continue;
        };
        let Some(governor) = agency::governor(w, s, requester) else {
            continue;
        };
        let goals = agency::objectives::governing(w, s, requester).unwrap();
        let protected = crate::minting::orders::stock_protection(
            w,
            s,
            requester,
            crate::minting::orders::input_claim_months(policy),
        )?;
        // One batch per explicit recipe goal. Repeated goals are alternatives,
        // not separate buyers; input entries within the batch are additive.
        for goal in goals {
            let Metric::ProcessShortfall { definition, target } = goal.metric else {
                continue;
            };
            if goal.scope != Scope::Organization
                || target == 0
                || !agreement.processes.contains(&definition)
                || !crate::opportunities::permits(w, s, requester, Action::Process(definition))
            {
                continue;
            }
            let recipe = w.definition(definition);
            if !recipe.enabled
                || recipe.execution != Execution::Productive
                || c.config.horizon <= recipe.duration()
            {
                continue;
            }
            let output_by = s
                .month
                .checked_add(c.config.horizon)
                .ok_or("upstream window overflow")?;
            let due = output_by - recipe.duration();
            let mut needs: BTreeMap<ResourceId, i32> = BTreeMap::new();
            for input in &recipe.stages[0].entry_inputs {
                if w.resources
                    .iter()
                    .any(|r| r.id == input.resource && r.kind == ResourceKind::Stock)
                {
                    let n = needs.entry(input.resource).or_default();
                    *n = n
                        .checked_add(input.quantity)
                        .ok_or("upstream input overflow")?;
                }
            }
            for (&market, options) in &agreement.stock_markets {
                if options.side != Side::Buy {
                    continue;
                }
                let listing = venue
                    .markets
                    .iter()
                    .find(|m| m.id == market)
                    .ok_or("missing upstream listing")?;
                let Some(&needed) = needs.get(&listing.goods.resource) else {
                    continue;
                };
                let held = s.balance(requester, listing.goods.resource);
                let dependency = Dependency {
                    definition,
                    governor,
                    goal_target: target,
                    output_by,
                    horizon: c.config.horizon,
                    needed,
                    protected_stock: protected
                        .get(&listing.goods.resource)
                        .copied()
                        .unwrap_or(0)
                        .clamp(0, i128::from(held)) as i32,
                    signed_target: *options.holdings.last().ok_or("missing upstream target")?,
                    ceiling: options.max_lots,
                };
                let mut r = Request {
                    requester,
                    venue: mint.venue,
                    market,
                    resource: listing.goods.resource,
                    payment: listing.payment,
                    lot_quantity: listing.goods.quantity,
                    lots: 0,
                    limit: *options.limits.last().ok_or("missing upstream price")?,
                    observed_stock: held,
                    due: Some(due),
                    source: Source::ProspectiveInput(dependency.clone()),
                };
                r.lots = lots(&r, &dependency);
                if r.lots == 0 {
                    continue;
                }
                // A sufficient live standing request already expresses this gap.
                if snapshot.requests.iter().any(|old| {
                    old.requester == requester
                        && old.venue == r.venue
                        && old.market == market
                        && old.due.is_none()
                        && old.lots >= r.lots
                        && old.limit >= r.limit
                }) {
                    continue;
                }
                // Concrete scheduled interest takes precedence at the same key.
                // Never append a second identity or enlarge an accepted window.
                if snapshot.requests.iter().any(|old| {
                    (old.venue, old.market, old.requester, old.due)
                        == (r.venue, r.market, r.requester, r.due)
                }) {
                    continue;
                }
                let key = (r.venue, r.market, r.requester, r.due);
                match candidates.entry(key) {
                    std::collections::btree_map::Entry::Vacant(e) => {
                        e.insert(r);
                    }
                    std::collections::btree_map::Entry::Occupied(mut e) => {
                        // Max across alternative recipes, deterministic lowest
                        // definition on an equal lot count; never sum their demand.
                        let Source::ProspectiveInput(old) = &e.get().source else {
                            unreachable!()
                        };
                        if (r.lots, std::cmp::Reverse(definition))
                            > (e.get().lots, std::cmp::Reverse(old.definition))
                        {
                            e.insert(r);
                        }
                    }
                }
                if candidates.len() + snapshot.requests.len() > super::MAX_REQUESTS {
                    return Err("upstream request budget exceeded".into());
                }
            }
        }
    }
    Ok(candidates.into_values().collect())
}

pub(super) fn validate(w: &World, month: u32, r: &Request, d: &Dependency) -> Result<i32, String> {
    let c = w
        .agency
        .get(&r.requester)
        .ok_or("missing upstream controller")?;
    let a = c.operating_agreement().ok_or("unsigned upstream request")?;
    let options = a
        .stock_markets
        .get(&r.market)
        .ok_or("unsigned upstream listing")?;
    let recipe = w
        .definitions
        .iter()
        .find(|p| p.id == d.definition)
        .ok_or("unknown upstream recipe")?;
    let needed: i64 = recipe.stages[0]
        .entry_inputs
        .iter()
        .filter(|i| i.resource == r.resource)
        .map(|i| i64::from(i.quantity))
        .sum();
    let goals = c
        .config
        .preferences
        .get(&d.governor)
        .unwrap_or(&c.config.objectives);
    if !a.members.contains(&d.governor)
        || !a.processes.contains(&d.definition)
        || options.side != Side::Buy
        || !options.limits.contains(&r.limit)
        || !options.holdings.contains(&d.signed_target)
        || d.ceiling <= 0
        || d.ceiling > options.max_lots
        || recipe.execution != Execution::Productive
        || d.needed <= 0
        || i64::from(d.needed) != needed
        || !w
            .resources
            .iter()
            .any(|v| v.id == r.resource && v.kind == ResourceKind::Stock)
        || d.protected_stock < 0
        || d.protected_stock > r.observed_stock
        || d.horizon != c.config.horizon
        || d.horizon <= recipe.duration()
        || month.checked_add(d.horizon) != Some(d.output_by)
        || d.output_by.checked_sub(recipe.duration()) != r.due
        || !goals.iter().any(|g| {
            g.scope == Scope::Organization
                && g.metric
                    == (Metric::ProcessShortfall {
                        definition: d.definition,
                        target: d.goal_target,
                    })
                && d.goal_target > 0
        })
    {
        return Err("invalid prospective input dependency".into());
    }
    Ok(lots(r, d))
}
