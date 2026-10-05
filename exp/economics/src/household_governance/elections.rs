//! Explicit ballots for scheduled household terms; preferences are supplied by callers.
use super::*;

pub use crate::governance::{
    AcceptedBallot, Ballot, DEFAULT_TURNOUT_PERCENT, ElectionResult, ElectionTieBreak, Rules,
};

pub(super) fn validate(a: &Agreement, state: &State) -> std::result::Result<(), String> {
    let g = &a.governance;
    let mut seen = BTreeSet::new();
    let alive = |id: AgentId, month| {
        crate::households::membership::roster_at(a, month).contains(&id)
            && state.terminal.get(&id).is_none_or(|t| t.month >= month)
    };
    if !(1..=PERCENT as u32).contains(&g.charter.election.minimum_turnout_percent)
        || g.ballots.iter().any(|b| {
            g.constitution.leadership != Leadership::Elected
                || g.charter.term_months == 0
                || b.issued_month < a.formed
                || b.issued_month > state.month
                || b.ballot.term_start <= b.issued_month
                || !(b.ballot.term_start - a.formed).is_multiple_of(g.charter.term_months)
                || !seen.insert((b.ballot.term_start, b.ballot.voter))
                || crate::households::dissolution::winding_at(a, b.issued_month).is_some()
                || !alive(b.ballot.voter, b.issued_month)
                || b.ballot
                    .candidate
                    .is_some_and(|id| !alive(id, b.issued_month))
        })
    {
        return Err("invalid household election rules or ballot".into());
    }
    Ok(())
}

/// Resolve against the term's opening electorate so later deaths cannot rewrite history.
pub(super) fn resolve(a: &Agreement, state: &State, term_start: u32) -> ElectionResult {
    let mut eligible: Vec<_> = crate::households::membership::roster_at(a, term_start)
        .into_iter()
        .filter(|id| state.terminal.get(id).is_none_or(|t| t.month >= term_start))
        .collect();
    eligible.sort_unstable();
    crate::governance::tally(
        term_start,
        eligible,
        &a.governance.charter.election,
        &a.governance.ballots,
    )
}

/// Accept one immutable ballot per voter per future regular term, atomically.
pub fn cast(
    world: &mut World,
    state: &State,
    household: AgentId,
    ballot: Ballot,
) -> std::result::Result<(), String> {
    if state.terminal.contains_key(&ballot.voter)
        || ballot
            .candidate
            .is_some_and(|id| state.terminal.contains_key(&id))
    {
        return Err("ballots require living voters and candidates".into());
    }
    let mut candidate = world.clone();
    let a = candidate
        .households
        .iter_mut()
        .find(|a| a.agent == household)
        .ok_or("unknown household")?;
    a.governance.ballots.push(AcceptedBallot {
        issued_month: state.month,
        ballot,
    });
    crate::households::validate(&candidate, state)?;
    *world = candidate;
    Ok(())
}
