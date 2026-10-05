//! Two existing citizens, supplied ballots and a fixed legal-policy menu.
use super::*;
use crate::{competition::SECOND_PERSON, membership, scenario::*};

pub const OPEN: PolicyId = 0;
pub const PAUSE_ADMISSIONS: PolicyId = 1;
pub const TERM_MONTHS: u32 = 2;
const ALLOCATION_SEED: u64 = 17;
const PLOTS: u32 = 2;

/// Both citizens start with ordinary farming opportunities. Governance can pause
/// new land/household admissions, but does not revoke granted rights or dues.
pub fn pair() -> Result<(World, State), String> {
    let (mut w, mut s) = crate::competition::scenario(PLOTS, ALLOCATION_SEED)?;
    let p = w.transaction_policy.as_mut().unwrap();
    p.permissions.insert((PERSON_TYPE, Action::FoundHousehold));
    for member in [PERSON, SECOND_PERSON] {
        s.memberships.insert(
            (member, STATE_AGENT, CITIZEN),
            membership::Agreement {
                member,
                organization: STATE_AGENT,
                role: CITIZEN,
                source_offer: p.membership_offers[0].id,
                accepted_month: s.month,
            },
        );
    }
    w.state_governance = Some(Governance {
        formation: None,
        state: STATE_AGENT,
        formed: s.month,
        constitution: Constitution {
            leadership: Leadership::Elected,
            policies: [
                (
                    OPEN,
                    LegalPolicy {
                        name: "open admissions".into(),
                        prohibited: BTreeSet::new(),
                    },
                ),
                (
                    PAUSE_ADMISSIONS,
                    LegalPolicy {
                        name: "pause new land and household agreements".into(),
                        prohibited: [
                            (Some(PERSON_TYPE), Action::LandAccess),
                            (Some(PERSON_TYPE), Action::FoundHousehold),
                        ]
                        .into(),
                    },
                ),
            ]
            .into(),
        },
        charter: Charter {
            founder: PERSON,
            term_months: TERM_MONTHS,
            election: Rules::default(),
            initial_policy: OPEN,
        },
        ballots: vec![],
        changes: vec![],
    });
    for voter in [PERSON, SECOND_PERSON] {
        cast(
            &mut w,
            &s,
            Ballot {
                term_start: s.month + TERM_MONTHS,
                voter,
                candidate: Some(SECOND_PERSON),
            },
        )?;
    }
    Ok((w, s))
}
