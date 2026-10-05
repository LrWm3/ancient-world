//! Shared leadership vocabulary and deterministic ballot counting.
//! Each institution supplies its own dated eligibility and authority rules.
use crate::model::AgentId;
use std::collections::BTreeMap;

const PERCENT: u32 = 100;
pub const DEFAULT_TURNOUT_PERCENT: u32 = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Leadership {
    FixedFounder,
    Rotating,
    Elected,
}

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

/// Count accepted ballots against a frozen term electorate. Admission validates
/// unique voters and dates; this function does not invent voter preferences.
pub(crate) fn tally<'a>(
    term_start: u32,
    mut eligible: Vec<AgentId>,
    rules: &Rules,
    ballots: impl IntoIterator<Item = &'a AcceptedBallot>,
) -> ElectionResult {
    eligible.sort_unstable();
    eligible.dedup();
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
    for b in ballots {
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
