//! Optional exploration through stocked, signed asks, never assumed demand.
use super::objectives::measure;
use super::*;
use crate::marketplace::Side;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Policy {
    #[default]
    Retain,
    StockedAsksOnEqualProjection,
    /// Also prefer adding signed output asks to an equally scored productive
    /// choice, preserving that choice's work and all of its quotes.
    ProductiveAsksOnEqualProjection,
}

fn losses(a: &Alternative, preference: Option<AgentId>) -> Option<&Vec<i128>> {
    if let Some(person) = preference {
        a.preferences.get(&person)
    } else {
        a.losses.as_ref()
    }
}

pub(super) fn select(
    w: &World,
    s: &State,
    agent: AgentId,
    c: &Config,
    preference: Option<AgentId>,
    retain: &Alternative,
    alternatives: &[Alternative],
) -> Result<Option<u32>, String> {
    if c.publication == Policy::Retain || losses(retain, preference).is_none() {
        return Ok(None);
    }
    let goals = preference
        .and_then(|p| c.preferences.get(&p))
        .unwrap_or(&c.objectives);
    if let Some(selected) = retain.program {
        if c.publication != Policy::ProductiveAsksOnEqualProjection {
            return Ok(None);
        }
        let mut best = None;
        let mut additions = 0;
        for a in alternatives {
            let Some(id) = a.program else { continue };
            if losses(a, preference) != losses(retain, preference) {
                continue;
            }
            let count =
                productive_additions(w, s, agent, &c.programs[&selected], &c.programs[&id], goals)?;
            if count > additions {
                additions = count;
                best = Some(id);
            }
        }
        return Ok(best);
    }
    for a in alternatives {
        let Some(id) = a.program else { continue };
        if losses(a, preference) == losses(retain, preference)
            && eligible(w, s, agent, &c.programs[&id], goals)?
        {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

/// Compare two evaluated choices, never add work or promised buyer demand.
fn productive_additions(
    w: &World,
    s: &State,
    agent: AgentId,
    selected: &Program,
    candidate: &Program,
    goals: &[Objective],
) -> Result<usize, String> {
    let Some(mint) = &w.minting else { return Ok(0) };
    if governor(w, s, agent).is_none()
        || !crate::marketplace::eligible(w, s, mint.venue, agent)
        || validate_program(w, agent, candidate).is_err()
    {
        return Ok(0);
    }
    let work = |p: &Program| {
        p.commands
            .iter()
            .filter(|c| !matches!(c, Command::PeerStockQuotes(_)))
            .cloned()
            .collect::<Vec<_>>()
    };
    if work(selected) != work(candidate) {
        return Ok(0);
    }
    let output_resources: BTreeSet<_> = selected
        .commands
        .iter()
        .flat_map(|c| match c {
            Command::WorkTargets(orders) => orders.iter().map(|o| o.definition).collect(),
            Command::StartProcess(id) => vec![*id],
            _ => vec![],
        })
        .flat_map(|id| {
            w.definition(id)
                .outputs
                .iter()
                .filter(|o| o.quantity > 0)
                .map(|o| o.resource)
        })
        .collect();
    if output_resources.is_empty() {
        return Ok(0);
    }
    let quotes = |p: &Program| {
        p.commands
            .iter()
            .filter_map(|c| match c {
                Command::PeerStockQuotes(q) => Some(q.clone()),
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>()
    };
    let previous = if selected
        .commands
        .iter()
        .any(|c| matches!(c, Command::PeerStockQuotes(_)))
    {
        quotes(selected)
    } else {
        // A work-only command retains today's portfolio; an explicit empty quote
        // command clears it. Compare the actual replacement semantics.
        existing_quotes(w, agent)
    };
    let proposed = quotes(candidate);
    if previous.iter().any(|q| !proposed.contains(q)) {
        return Ok(0);
    }
    let added: Vec<_> = proposed.iter().filter(|q| !previous.contains(q)).collect();
    let venue = crate::marketplace::venue(w, mint.venue).ok_or("missing publication venue")?;
    for q in &added {
        let m = venue
            .markets
            .iter()
            .find(|m| m.id == q.market)
            .ok_or("missing publication listing")?;
        if q.side != Side::Sell
            || !output_resources.contains(&m.goods.resource)
            || !payment_wanted(w, s, agent, goals, m.payment)?
        {
            return Ok(0);
        }
    }
    // Quotes can be posted before output exists. Acquisition still clips them to
    // actual uncommitted stock and applies floors, storage, law and payment limits.
    Ok(added.len())
}

fn existing_quotes(w: &World, agent: AgentId) -> Vec<crate::minting::orders::Quote> {
    let Some(a) = w.agency[&agent].operating_agreement() else {
        return vec![];
    };
    let Some(policy) = w.minting.as_ref().and_then(|m| m.order_policy.as_ref()) else {
        return vec![];
    };
    policy
        .peer_stocks
        .iter()
        .flat_map(|p| &p.quotes)
        .chain(
            policy
                .quotes
                .iter()
                .filter(|q| a.stock_markets.contains_key(&q.market)),
        )
        .filter(|q| q.agent == agent)
        .cloned()
        .collect()
}

fn payment_wanted(
    w: &World,
    s: &State,
    agent: AgentId,
    goals: &[Objective],
    payment: ResourceId,
) -> Result<bool, String> {
    for o in goals
        .iter()
        .filter(|o| o.scope == objectives::Scope::Organization)
    {
        let relevant = matches!(o.metric,
            objectives::Metric::Reserve { resource, .. } | objectives::Metric::ReserveExposure { resource, .. } | objectives::Metric::FundingGap { resource, .. }
            if resource == payment);
        if relevant && measure(w, s, &[], agent, std::slice::from_ref(o))?[0] > 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

/// A publication preserves the whole replaced portfolio and adds only stocked
/// asks. Eligibility is observed now; quotes apply next Open and trade at Acquire.
fn eligible(
    w: &World,
    s: &State,
    agent: AgentId,
    p: &Program,
    goals: &[Objective],
) -> Result<bool, String> {
    let [Command::PeerStockQuotes(quotes)] = p.commands.as_slice() else {
        return Ok(false);
    };
    let Some(_) = w.agency[&agent].operating_agreement() else {
        return Ok(false);
    };
    let Some(mint) = &w.minting else {
        return Ok(false);
    };
    let Some(policy) = &mint.order_policy else {
        return Ok(false);
    };
    let Some(_) = &policy.peer_stocks else {
        return Ok(false);
    };
    if governor(w, s, agent).is_none()
        || !crate::marketplace::eligible(w, s, mint.venue, agent)
        || validate_program(w, agent, p).is_err()
    {
        return Ok(false);
    }
    let existing = existing_quotes(w, agent);
    if existing.iter().any(|q| !quotes.contains(q)) {
        return Ok(false);
    }
    let additions: Vec<_> = quotes.iter().filter(|q| !existing.contains(q)).collect();
    if additions.is_empty() || additions.iter().any(|q| q.side != Side::Sell) {
        return Ok(false);
    }
    let venue = crate::marketplace::venue(w, mint.venue).ok_or("missing publication venue")?;
    let protected = crate::minting::orders::stock_protection(
        w,
        s,
        agent,
        crate::minting::orders::input_claim_months(policy),
    )?;
    for q in additions {
        let m = venue
            .markets
            .iter()
            .find(|m| m.id == q.market)
            .ok_or("missing publication listing")?;
        if !w
            .resources
            .iter()
            .any(|r| r.id == m.goods.resource && r.kind == ResourceKind::Stock)
        {
            return Ok(false);
        }
        let wanted = payment_wanted(w, s, agent, goals, m.payment)?;
        let floor =
            i128::from(q.holding).max(protected.get(&m.goods.resource).copied().unwrap_or(0));
        if !wanted
            || i128::from(s.balance(agent, m.goods.resource)) - floor < i128::from(m.goods.quantity)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests;
