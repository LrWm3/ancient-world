//! Explicit ballots for scheduled household terms; preferences are supplied by callers.
use super::*;
use std::collections::BTreeMap;

pub const DEFAULT_TURNOUT_PERCENT: u32 = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElectionTieBreak {
    MemberId,
    Vacant,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub minimum_turnout_percent: u32,
    pub tie_break: ElectionTieBreak,
}
impl Default for Rules {
    fn default() -> Self {
        Self {
            minimum_turnout_percent: DEFAULT_TURNOUT_PERCENT,
            tie_break: ElectionTieBreak::MemberId,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ballot {
    pub term_start: u32,
    pub voter: AgentId,
    /// None is an explicit abstention and counts toward turnout.
    pub candidate: Option<AgentId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedBallot {
    pub issued_month: u32,
    pub ballot: Ballot,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ElectionResult {
    pub term_start: u32,
    pub eligible: Vec<AgentId>,
    pub turnout: usize,
    pub required_turnout: usize,
    pub votes: BTreeMap<AgentId, usize>,
    pub winner: Option<AgentId>,
}

pub(super) fn validate(a: &Agreement, state: &State) -> std::result::Result<(), String> {
    let g = &a.governance;
    let mut seen = BTreeSet::new();
    let alive = |id: AgentId, month| {
        a.adults.contains(&id) && state.terminal.get(&id).is_none_or(|t| t.month >= month)
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
    let mut eligible: Vec<_> = a
        .adults
        .iter()
        .copied()
        .filter(|id| state.terminal.get(id).is_none_or(|t| t.month >= term_start))
        .collect();
    eligible.sort_unstable();
    let rules = &a.governance.charter.election;
    let required_turnout =
        (eligible.len() * rules.minimum_turnout_percent as usize).div_ceil(PERCENT as usize);
    let mut result = ElectionResult {
        term_start,
        eligible,
        turnout: 0,
        required_turnout,
        votes: BTreeMap::new(),
        winner: None,
    };
    for b in &a.governance.ballots {
        if b.ballot.term_start != term_start || !result.eligible.contains(&b.ballot.voter) {
            continue;
        }
        result.turnout += 1;
        if let Some(id) = b.ballot.candidate.filter(|id| result.eligible.contains(id)) {
            *result.votes.entry(id).or_default() += 1;
        }
    }
    if result.turnout >= required_turnout {
        let max = result.votes.values().copied().max().unwrap_or(0);
        let winners: Vec<_> = result
            .votes
            .iter()
            .filter(|(_, n)| **n == max)
            .map(|(id, _)| *id)
            .collect();
        if winners.len() == 1 || rules.tie_break == ElectionTieBreak::MemberId {
            result.winner = winners.first().copied();
        }
    }
    result
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
