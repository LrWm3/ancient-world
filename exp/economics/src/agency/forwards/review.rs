//! Stable first-mutually-acceptable proposal policy; no global welfare objective.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub month: u32,
    pub proposal: Proposal,
    pub assessment: Result<assessment::Assessment, String>,
}

pub(crate) fn open(w: &mut World, s: &State) -> Result<(), String> {
    if !w.agency.values().any(|c| {
        c.autonomous
            && c.operating_agreement()
                .is_some_and(|a| !a.forwards.is_empty())
    }) {
        return Ok(());
    }
    for proposal in candidates(w, s)? {
        if ![proposal.terms.buyer, proposal.terms.seller]
            .iter()
            .all(|a| execution::reviewable(w, s, *a))
        {
            continue;
        }
        let result = assessment::assess(w, s, &proposal);
        let accepted = result.as_ref().is_ok_and(|a| a.mutually_beneficial());
        w.agency
            .get_mut(&proposal.terms.buyer)
            .unwrap()
            .forward_reviews
            .push(Attempt {
                month: s.month,
                proposal: proposal.clone(),
                assessment: result,
            });
        if accepted {
            execution::publish(w, s, &proposal)?;
            // One admission candidate globally per Open; subsequent reviews use
            // the now-public claim, not stale independent budget assumptions.
            break;
        }
    }
    Ok(())
}
