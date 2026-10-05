//! Reserve joint prerequisites before granting optional land commitments.
use super::*;
use crate::allocation::{self, Claim, Context};

const LAND_ALLOCATION_POLICY: allocation::Policy = allocation::Policy::StablePriority;

fn pool(w: &World, s: &State, transactions: &[Transaction]) -> BTreeMap<Account, i128> {
    let mut available: BTreeMap<_, _> = s
        .balances
        .iter()
        .map(|(k, q)| (*k, i128::from(*q)))
        .collect();
    for p in &w.participants {
        if let Some(household) = crate::households::parent(w, s, p.agent) {
            let share = i128::from(crate::households::labor_reserve(
                w,
                s,
                p.agent,
                p.capacity.resource,
            ));
            *available.entry((p.agent, p.capacity.resource)).or_default() -= share;
            *available
                .entry((household, p.capacity.resource))
                .or_default() += share;
        }
    }
    // Admission cannot promise inputs already sold at this Acquire boundary.
    // Incoming purchases remain conservative: only finalized opening stocks count.
    for effect in transactions
        .iter()
        .flat_map(|t| &t.effects)
        .filter(|e| e.delta < 0)
    {
        let held = available.entry(effect.account).or_default();
        *held = (*held + i128::from(effect.delta)).max(0);
    }
    // Existing production has first claim on future monthly service capacity.
    for process in s.processes.values().filter(|p| p.status == Status::Active) {
        let mut peak = BTreeMap::<ResourceId, i128>::new();
        for stage in &w.definition(process.definition).stages[process.stage..] {
            let mut monthly = BTreeMap::<ResourceId, i128>::new();
            for a in &stage.monthly_services {
                *monthly.entry(a.resource).or_default() += i128::from(a.quantity);
            }
            for (r, q) in monthly {
                let v = peak.entry(r).or_default();
                *v = (*v).max(q);
            }
        }
        for (r, q) in peak {
            let held = available.entry((process.operator, r)).or_default();
            let own = (*held).min(q).max(0);
            *held -= own;
            if let Some(parent) = crate::households::parent(w, s, process.operator) {
                let shared = available.entry((parent, r)).or_default();
                *shared = (*shared - (q - own)).max(0);
            }
        }
    }
    available
}

fn reserve(
    w: &World,
    s: &State,
    person: AgentId,
    d: &ProcessDefinition,
    available: &BTreeMap<Account, i128>,
) -> Option<BTreeMap<Account, i128>> {
    let mut stocks = BTreeMap::<ResourceId, i128>::new();
    let mut services = BTreeMap::<ResourceId, i128>::new();
    for stage in &d.stages {
        for a in &stage.entry_inputs {
            *stocks.entry(a.resource).or_default() += i128::from(a.quantity);
        }
        let mut monthly = BTreeMap::<ResourceId, i128>::new();
        for a in &stage.monthly_services {
            *monthly.entry(a.resource).or_default() += i128::from(a.quantity);
        }
        for (resource, amount) in monthly {
            let maximum = services.entry(resource).or_default();
            *maximum = (*maximum).max(amount);
        }
    }
    let mut next = available.clone();
    for (resource, amount) in stocks {
        let held = next.entry((person, resource)).or_default();
        if *held < amount {
            return None;
        }
        *held -= amount;
    }
    for (resource, amount) in services {
        let held = next.entry((person, resource)).or_default();
        let own = (*held).min(amount).max(0);
        *held -= own;
        if own < amount {
            let household = crate::households::parent(w, s, person)?;
            let agreement = w.households.iter().find(|h| h.agent == household)?;
            if agreement
                .governance
                .constitution
                .activities
                .as_ref()
                .is_some_and(|ids| !ids.contains(&d.id))
            {
                return None;
            }
            let shared = next.entry((household, resource)).or_default();
            if *shared < amount - own {
                return None;
            }
            *shared -= amount - own;
        }
    }
    Some(next)
}

pub(super) fn land(
    w: &World,
    s: &mut State,
    accepted: &mut Vec<(u32, AgentId)>,
    transactions: &[Transaction],
) -> Result<Vec<allocation::Receipt>, String> {
    let mut available = pool(w, s, transactions);
    let mut candidates = BTreeMap::<AgentId, Vec<(u32, DefinitionId)>>::new();
    for person in people(w, s) {
        if crate::commitments::active(w, s).any(|x| {
            x.debtor == person
                && w.rights
                    .iter()
                    .any(|r| r.id == x.right && r.through >= s.month)
        }) {
            continue;
        }
        for id in opportunities::relevant_access(w, s, person) {
            let Ok(a) = crate::commitments::acceptance_for(w, s, id, person) else {
                continue;
            };
            let right = w.rights.iter().find(|r| r.id == a.right).unwrap();
            let asset = w.assets.iter().find(|x| x.id == right.asset).unwrap();
            for d in opportunities::processes(w, s, person) {
                if d.enabled
                    && d.execution == Execution::Productive
                    && d.asset_kind == Some(asset.kind)
                    && d.duration() <= right.through.saturating_sub(s.month)
                    && d.outputs.iter().any(|out| {
                        out.resource == a.payment.resource && out.quantity > a.payment.quantity
                    })
                    && reserve(w, s, person, d, &available).is_some()
                {
                    candidates.entry(person).or_default().push((id, d.id));
                }
            }
        }
    }
    let claims: Vec<_> = candidates
        .keys()
        .map(|id| Claim {
            id: u64::from(*id),
            priority: 0,
            requested: 1,
            minimum: 1,
        })
        .collect();
    allocation::resolve(
        Context {
            seed: 0,
            pool: u64::from(w.transaction_policy.as_ref().unwrap().authority),
            round: u64::from(s.month),
        },
        &LAND_ALLOCATION_POLICY,
        u32::try_from(w.access_offers.len()).map_err(|_| "too many land offers")?,
        &claims,
        |claim, _| {
            let person = claim.id as u32;
            for &(id, definition) in &candidates[&person] {
                let Ok(agreement) = crate::commitments::acceptance_for(w, s, id, person) else {
                    continue;
                };
                let Some(next) = reserve(w, s, person, w.definition(definition), &available) else {
                    continue;
                };
                available = next;
                s.accepted_agreements.insert(id, agreement);
                accepted.push((id, person));
                return Ok(());
            }
            Err("plot or jointly required stock/capacity already reserved".into())
        },
    )
}
