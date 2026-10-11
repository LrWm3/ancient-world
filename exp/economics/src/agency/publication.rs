//! Optional exploration through stocked, signed asks, never assumed demand.
use super::*;
use crate::marketplace::Side;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Policy {
    #[default]
    Retain,
    StockedAsksOnEqualProjection,
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
    if c.publication != Policy::StockedAsksOnEqualProjection
        || retain.program.is_some()
        || losses(retain, preference).is_none()
    {
        return Ok(None);
    }
    let goals = preference
        .and_then(|p| c.preferences.get(&p))
        .unwrap_or(&c.objectives);
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
    let Some(a) = w.agency[&agent].operating_agreement() else {
        return Ok(false);
    };
    let Some(mint) = &w.minting else {
        return Ok(false);
    };
    let Some(policy) = &mint.order_policy else {
        return Ok(false);
    };
    let Some(peers) = &policy.peer_stocks else {
        return Ok(false);
    };
    if governor(w, s, agent).is_none()
        || !crate::marketplace::eligible(w, s, mint.venue, agent)
        || validate_program(w, agent, p).is_err()
    {
        return Ok(false);
    }
    let existing: Vec<_> = peers
        .quotes
        .iter()
        .chain(
            policy
                .quotes
                .iter()
                .filter(|q| a.stock_markets.contains_key(&q.market)),
        )
        .filter(|q| q.agent == agent)
        .collect();
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
        let mut wanted = false;
        for o in goals
            .iter()
            .filter(|o| o.scope == objectives::Scope::Organization)
        {
            let payment_goal = matches!(o.metric,
                objectives::Metric::Reserve { resource, .. } | objectives::Metric::FundingGap { resource, .. }
                    if resource == m.payment);
            if payment_goal && measure(w, s, &[], agent, std::slice::from_ref(o))?[0] > 0 {
                wanted = true;
            }
        }
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
