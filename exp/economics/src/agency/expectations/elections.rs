//! Repeat observed individual ballots in private state-election hypotheses.
//! No office is extended: ordinary eligibility, turnout and tally still decide.
use crate::{
    governance::{Ballot, Leadership},
    model::*,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Evidence {
    pub state: AgentId,
    pub issued_month: u32,
    pub term_start: u32,
    pub voter: AgentId,
    pub candidate: Option<AgentId>,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Attempt {
    pub state: AgentId,
    pub month: u32,
    pub term_start: u32,
    pub voter: AgentId,
    pub candidate: Option<AgentId>,
    pub accepted: bool,
    pub failure: Option<String>,
}

pub(super) fn observe(w: &World, s: &State, actor: AgentId, memory: u32) -> Vec<Evidence> {
    let Some(g) = &w.state_governance else {
        return vec![];
    };
    if g.state == actor
        || !w.agency.contains_key(&g.state)
        || g.constitution.leadership != Leadership::Elected
        || g.charter.term_months == 0
        || s.month < g.formed
    {
        return vec![];
    }
    let term = g.formed + ((s.month - g.formed) / g.charter.term_months) * g.charter.term_months;
    let mut result: Vec<_> = g
        .ballots
        .iter()
        .filter(|b| {
            b.ballot.term_start == term
                && b.issued_month < s.month
                && s.month - b.issued_month <= memory
        })
        .map(|b| Evidence {
            state: g.state,
            issued_month: b.issued_month,
            term_start: term,
            voter: b.ballot.voter,
            candidate: b.ballot.candidate,
        })
        .collect();
    result.sort_by_key(|e| e.voter);
    result
}

pub(super) fn apply(
    snapshot: &super::Snapshot,
    w: &mut World,
    s: &State,
    memory: u32,
) -> Result<Vec<Attempt>, String> {
    let Some(g) = &w.state_governance else {
        return Ok(vec![]);
    };
    let Some(term) = s.month.checked_add(1) else {
        return Err("forecast election month overflow".into());
    };
    if g.state == snapshot.actor
        || !w.agency.contains_key(&g.state)
        || g.constitution.leadership != Leadership::Elected
        || g.charter.term_months == 0
        || term <= g.formed
        || !(term - g.formed).is_multiple_of(g.charter.term_months)
    {
        return Ok(vec![]);
    }
    let state = g.state;
    let mut result = vec![];
    for e in &snapshot.ballots {
        if e.state != state
            || e.issued_month >= snapshot.as_of
            || e.term_start > snapshot.as_of
            || s.month - e.issued_month > memory
            || w.state_governance
                .as_ref()
                .unwrap()
                .ballots
                .iter()
                .any(|b| b.ballot.term_start == term && b.ballot.voter == e.voter)
        {
            continue;
        }
        let ballot = Ballot {
            term_start: term,
            voter: e.voter,
            candidate: e.candidate,
        };
        let outcome = crate::state_governance::cast(w, s, ballot);
        result.push(Attempt {
            state,
            month: s.month,
            term_start: term,
            voter: e.voter,
            candidate: e.candidate,
            accepted: outcome.is_ok(),
            failure: outcome.err(),
        });
    }
    Ok(result)
}
