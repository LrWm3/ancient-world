//! Opt-in abstract reproduction: dated persons, dependent support, and maturity.
//! No mating/genetics model, inherited rights, or automatic household membership.
use crate::{maintenance::ConditionRule, model::*};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template {
    pub capacity: Amount,
    pub needs: Vec<Requirement>,
    /// Subjects are rebound to the new person; conditions start at zero.
    pub conditions: Vec<ConditionRule>,
    pub storage: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub enabled: bool,
    /// Forecast known dependent support in planning; never hypothetical births.
    pub plan_support: bool,
    pub parents: BTreeSet<AgentId>,
    pub descendants_reproduce: bool,
    pub first_month: u32,
    pub interval_months: u32,
    pub maturity_months: u32,
    /// Lifetime registration ceiling, including terminal people; an execution bound.
    pub max_births: usize,
    pub first_id: AgentId,
    pub person: Template,
    /// One support stock in this pilot. Birth transfers this many months up front.
    pub support: Amount,
    pub birth_buffer_months: u32,
    /// Retain this stock for the donor before dependent support or birth funding.
    pub parent_reserve: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Birth {
    pub child: AgentId,
    pub parent: AgentId,
    pub month: u32,
    pub adult_at: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Book {
    pub births: BTreeMap<AgentId, Birth>,
    pub matured: BTreeSet<AgentId>,
    pub last_birth: BTreeMap<AgentId, u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Support {
    pub parent: AgentId,
    pub child: AgentId,
    pub requested: i32,
    pub delivered: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boundary {
    pub births: Vec<Birth>,
    pub matured: Vec<AgentId>,
    pub support: Vec<Support>,
    pub after: Book,
    pub transactions: Vec<Transaction>,
}
pub fn dependent(s: &State, agent: AgentId) -> bool {
    s.demography
        .births
        .get(&agent)
        .is_some_and(|b| s.month < b.adult_at)
}
pub fn dependents(s: &State, parent: AgentId) -> impl Iterator<Item = AgentId> + '_ {
    s.demography
        .births
        .values()
        .filter(move |b| {
            b.parent == parent && s.month < b.adult_at && !s.terminal.contains_key(&b.child)
        })
        .map(|b| b.child)
}
fn participant(c: &Config, child: AgentId, adult: bool) -> Participant {
    let mut capacity = c.person.capacity.clone();
    if !adult {
        capacity.quantity = 0;
    }
    Participant {
        agent: child,
        capacity,
        needs: c.person.needs.clone(),
    }
}
fn rules(c: &Config, child: AgentId) -> Vec<ConditionRule> {
    c.person
        .conditions
        .iter()
        .cloned()
        .map(|mut r| {
            r.subject = child;
            r
        })
        .collect()
}
fn name(b: &Birth) -> String {
    format!(
        "person {} (child of {}, born {})",
        b.child, b.parent, b.month
    )
}

pub fn validate(w: &World, s: &State) -> Result<(), String> {
    let Some(c) = &w.demography else {
        return if s.demography == Book::default() {
            Ok(())
        } else {
            Err("person lifecycle needs its catalog".into())
        };
    };
    if c.first_month == 0
        || c.interval_months == 0
        || c.maturity_months == 0
        || c.max_births == 0
        || c.birth_buffer_months == 0
        || c.support.quantity <= 0
        || c.parent_reserve < 0
        || c.person.storage < 0
        || c.person.capacity.quantity < 0
        || !w
            .resources
            .iter()
            .any(|r| r.id == c.support.resource && r.kind == ResourceKind::Stock)
        || !w
            .resources
            .iter()
            .any(|r| r.id == c.person.capacity.resource && r.kind == ResourceKind::Capacity)
        || c.parents.iter().any(|a| {
            !w.participants.iter().any(|p| p.agent == *a)
                || w.transaction_policy.as_ref().is_some_and(|p| {
                    p.agent_types.get(a) != Some(&crate::opportunities::PERSON_TYPE)
                })
        })
        || c.person.needs.is_empty()
        || c.person.needs.iter().any(|n| {
            n.quantity <= 0
                || !w
                    .resources
                    .iter()
                    .any(|r| r.id == n.resource && r.kind == ResourceKind::Fulfillment)
        })
        || c.person
            .needs
            .iter()
            .map(|n| n.resource)
            .collect::<BTreeSet<_>>()
            .len()
            != c.person.needs.len()
    {
        return Err("invalid reproduction configuration".into());
    }
    // Discovery's formation and market-registration adapters do not yet admit a
    // changing dependent roster. Fail explicitly rather than silently making adults.
    if w.discovery.as_ref().is_some_and(|d| d.enabled) {
        return Err("reproduction needs a dependent-aware discovery adapter".into());
    }
    let endowment = i64::from(c.support.quantity) * i64::from(c.birth_buffer_months);
    let weight = i64::from(*w.storage.weights.get(&c.support.resource).unwrap_or(&0));
    if endowment > i64::from(i32::MAX) || endowment * weight > i64::from(c.person.storage) {
        return Err("birth buffer exceeds child storage or quantity bound".into());
    }
    if s.demography.births.len() > c.max_births
        || s.demography
            .matured
            .iter()
            .any(|a| !s.demography.births.contains_key(a))
    {
        return Err("invalid person lifecycle book".into());
    }
    let mut last_birth = BTreeMap::new();
    for (index, (&key, b)) in s.demography.births.iter().enumerate() {
        let allocated = c
            .first_id
            .checked_add(u32::try_from(index).map_err(|_| "birth count overflow")?)
            .ok_or("person ID overflow")?;
        let parent_born = s.demography.births.get(&b.parent);
        if key != b.child
            || key != allocated
            || !w.participants.iter().any(|p| p.agent == b.parent)
            || (!c.parents.contains(&b.parent) && parent_born.is_none())
            || parent_born.is_some_and(|p| b.month <= p.adult_at)
            || b.month < c.first_month
            || s.terminal.get(&b.parent).is_some_and(|t| t.month < b.month)
            || last_birth
                .get(&b.parent)
                .is_some_and(|previous| b.month.saturating_sub(*previous) < c.interval_months)
            || (s.demography.matured.contains(&b.child)
                && (s.month < b.adult_at || (s.month == b.adult_at && s.phase == Phase::Open)))
            || (!s.demography.matured.contains(&b.child)
                && s.month > b.adult_at
                && !s.terminal.contains_key(&b.child))
            || w.storage.capacities.get(&b.child) != Some(&c.person.storage)
            || w.condition_rules
                .iter()
                .filter(|r| r.subject == b.child)
                .cloned()
                .collect::<Vec<_>>()
                != rules(c, b.child)
            || w.transaction_policy.as_ref().is_some_and(|p| {
                p.agent_types.get(&b.child) != Some(&crate::opportunities::PERSON_TYPE)
            })
        {
            return Err("invalid person lifecycle chronology or template".into());
        }
        last_birth.insert(b.parent, b.month);
        if b.child == b.parent
            || b.month == 0
            || b.month > s.month
            || b.adult_at
                != b.month
                    .checked_add(c.maturity_months)
                    .ok_or("maturity overflow")?
            || !w
                .agents
                .iter()
                .any(|a| a.id == b.child && a.name == name(b))
            || !w.participants.contains(&participant(
                c,
                b.child,
                s.demography.matured.contains(&b.child),
            ))
        {
            return Err("person lifecycle differs from registered catalog".into());
        }
        if dependent(s, b.child)
            && w.households
                .iter()
                .any(|h| crate::households::membership::current(h).contains(&b.child))
        {
            return Err("dependent cannot be an adult household signatory".into());
        }
    }
    if last_birth != s.demography.last_birth {
        return Err("birth cooldown differs from history".into());
    }
    // Validate the template through the ordinary component/condition checks even
    // before the first birth, using an isolated catalog with no reproduction.
    let mut probe = w.clone();
    probe.demography = None;
    let mut state = s.clone();
    state.demography = Book::default();
    let id = w
        .agents
        .iter()
        .map(|a| a.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("agent ID overflow")?;
    probe.agents.push(Agent {
        id,
        name: "template validation".into(),
    });
    probe.participants.push(participant(c, id, false));
    probe.condition_rules.extend(rules(c, id));
    crate::maintenance::validate(&probe, &state)?;
    Ok(())
}

/// Same-opening stocks; existing dependents first, oldest ID first. Births use
/// the remainder, one per eligible parent, stable parent ID priority.
pub(crate) fn evaluate(w: &World, s: &State) -> Result<Option<Boundary>, String> {
    let Some(c) = &w.demography else {
        return Ok(None);
    };
    if s.phase != Phase::Open {
        return Ok(None);
    }
    let mut b = Boundary {
        births: vec![],
        matured: vec![],
        support: vec![],
        after: s.demography.clone(),
        transactions: vec![],
    };
    let mut available = s.balances.clone();
    let mut used = crate::storage::usage(w, &s.balances);
    if crate::credit::enabled(w)
        && let Some(opening) = crate::credit::evaluate(w, s)?
    {
        for t in &opening.transactions {
            crate::storage::apply(w, &mut used, &t.effects);
        }
        for e in opening
            .transactions
            .iter()
            .flat_map(|t| &t.effects)
            .filter(|e| e.delta < 0 && e.account.1 == c.support.resource)
        {
            let held = available.entry(e.account).or_default();
            *held = held
                .checked_add(e.delta)
                .filter(|q| *q >= 0)
                .ok_or("opening credit overdraws support stock")?;
        }
    }
    for birth in s.demography.births.values() {
        if s.terminal.contains_key(&birth.child) {
            continue;
        }
        if s.month >= birth.adult_at {
            if b.after.matured.insert(birth.child) {
                b.matured.push(birth.child);
            }
            continue;
        }
        let requested = c
            .support
            .quantity
            .saturating_sub(s.balance(birth.child, c.support.resource))
            .max(0);
        let maximum = if s.terminal.contains_key(&birth.parent) {
            0
        } else {
            available
                .get(&(birth.parent, c.support.resource))
                .copied()
                .unwrap_or(0)
                .saturating_sub(c.parent_reserve)
                .max(0)
                .min(requested)
        };
        let delivered = crate::storage::transferable(
            w,
            &used,
            birth.parent,
            birth.child,
            c.support.resource,
            maximum,
        )?;
        transfer(
            &mut b,
            &mut available,
            &mut used,
            w,
            Support {
                parent: birth.parent,
                child: birth.child,
                requested,
                delivered,
            },
        )?;
    }
    if c.enabled && s.month >= c.first_month {
        let mut parents = c.parents.clone();
        if c.descendants_reproduce {
            parents.extend(
                s.demography
                    .births
                    .values()
                    .filter(|b| s.month > b.adult_at)
                    .map(|b| b.child),
            );
        }
        for parent in parents {
            if b.after.births.len() >= c.max_births {
                break;
            }
            if s.terminal.contains_key(&parent)
                || dependent(s, parent)
                || s.conditions
                    .iter()
                    .any(|((a, _), v)| *a == parent && v.deprivation > 0)
                || s.demography
                    .last_birth
                    .get(&parent)
                    .is_some_and(|m| s.month.saturating_sub(*m) < c.interval_months)
            {
                continue;
            }
            let quantity =
                i32::try_from(i64::from(c.support.quantity) * i64::from(c.birth_buffer_months))
                    .map_err(|_| "birth buffer overflow")?;
            if available
                .get(&(parent, c.support.resource))
                .copied()
                .unwrap_or(0)
                .saturating_sub(c.parent_reserve)
                < quantity
            {
                continue;
            }
            let offset = u32::try_from(b.after.births.len()).map_err(|_| "birth count overflow")?;
            let child = c.first_id.checked_add(offset).ok_or("person ID overflow")?;
            let birth = Birth {
                child,
                parent,
                month: s.month,
                adult_at: s
                    .month
                    .checked_add(c.maturity_months)
                    .ok_or("maturity overflow")?,
            };
            b.after.births.insert(child, birth.clone());
            b.after.last_birth.insert(parent, s.month);
            b.births.push(birth);
            transfer(
                &mut b,
                &mut available,
                &mut used,
                w,
                Support {
                    parent,
                    child,
                    requested: quantity,
                    delivered: quantity,
                },
            )?;
        }
    }
    Ok(Some(b))
}
fn transfer(
    b: &mut Boundary,
    balances: &mut BTreeMap<Account, i32>,
    used: &mut BTreeMap<AgentId, i128>,
    w: &World,
    r: Support,
) -> Result<(), String> {
    if r.delivered > 0 {
        let effects = crate::finance::Transfer {
            from: r.parent,
            to: r.child,
            amount: Amount::new(w.demography.as_ref().unwrap().support.resource, r.delivered),
        }
        .effects()?;
        for e in &effects {
            let held = balances.entry(e.account).or_default();
            *held = held.checked_add(e.delta).ok_or("support overflow")?;
        }
        crate::storage::apply(w, used, &effects);
        b.transactions.push(Transaction {
            cause: "dependent support".into(),
            effects,
            process: None,
            technique_use: None,
            trade: None,
            stock_trade: None,
            forward: None,
            delivery: None,
            royalty: None,
        });
    }
    b.support.push(r);
    Ok(())
}

/// Stage only catalog registrations. Book and material changes commit in Open.
/// Replay must call this at each Open using the chronological catalog, as live
/// execution does; a final catalog is not an opening historical snapshot.
pub fn prepare_open(w: &mut World, s: &State) -> Result<(), String> {
    let Some(b) = evaluate(w, s)? else {
        return Ok(());
    };
    let c = w.demography.clone().unwrap();
    let mut next = w.clone();
    for birth in &b.births {
        if next.agents.iter().any(|a| a.id == birth.child) {
            return Err("reproduction agent ID collision".into());
        }
        next.agents.push(Agent {
            id: birth.child,
            name: name(birth),
        });
        next.participants.push(participant(&c, birth.child, false));
        next.condition_rules.extend(rules(&c, birth.child));
        next.storage
            .capacities
            .insert(birth.child, c.person.storage);
        if let Some(policy) = &mut next.transaction_policy {
            policy
                .agent_types
                .insert(birth.child, crate::opportunities::PERSON_TYPE);
        }
    }
    for child in &b.matured {
        let p = next
            .participants
            .iter_mut()
            .find(|p| p.agent == *child)
            .ok_or("missing maturing person")?;
        *p = participant(&c, *child, true);
    }
    *w = next;
    Ok(())
}
pub(crate) fn verify(w: &World, s: &State, b: &Batch) -> Result<(), String> {
    let expected = evaluate(w, s)?;
    if b.demography != expected {
        return Err("missing or altered person lifecycle receipt".into());
    }
    if let Some(d) = &expected {
        let c = w.demography.as_ref().unwrap();
        for birth in &d.births {
            if !w
                .agents
                .iter()
                .any(|a| a.id == birth.child && a.name == name(birth))
                || !w.participants.contains(&participant(c, birth.child, false))
                || w.storage.capacities.get(&birth.child) != Some(&c.person.storage)
                || w.condition_rules
                    .iter()
                    .filter(|r| r.subject == birth.child)
                    .cloned()
                    .collect::<Vec<_>>()
                    != rules(c, birth.child)
                || s.balances.keys().any(|(a, _)| *a == birth.child)
            {
                return Err("birth catalog differs from lifecycle receipt".into());
            }
        }
        for child in &d.matured {
            if !w.participants.contains(&participant(c, *child, true)) {
                return Err("maturity catalog differs from lifecycle receipt".into());
            }
        }
        let mut ordinary = Batch::empty(s);
        if crate::credit::enabled(w) {
            ordinary.transactions = crate::credit::evaluate(w, s)?
                .ok_or("missing opening credit")?
                .transactions;
        } else {
            crate::simulation::Simulation::open(w, s, &mut ordinary);
        }
        ordinary.transactions.extend(d.transactions.clone());
        if b.transactions != ordinary.transactions {
            return Err("altered opening or dependent support transfers".into());
        }
    }
    Ok(())
}

pub mod provision;
pub mod scenario;

/// Atomic chronological ledger replay, including household Open staging and the
/// dynamic person catalog. Other externally changed policies remain caller inputs.
pub fn replay(
    w: &mut World,
    s: &mut State,
    batch: &Batch,
    backend: crate::compute::Backend,
    limit: usize,
) -> Result<(), String> {
    let mut world = w.clone();
    let mut state = s.clone();
    let opening = if world.households.is_empty() {
        state.clone()
    } else {
        crate::households::prepare(&world, &state)?.0
    };
    prepare_open(&mut world, &opening)?;
    crate::settlement::commit(&world, &mut state, batch, backend, limit)?;
    *w = world;
    *s = state;
    Ok(())
}
