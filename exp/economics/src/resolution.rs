//! Immediate prerequisite clearing, separate from the policy ranking applicants.
use crate::{
    allocation::{Claim, Context, Outcome, RankingPolicy, Receipt},
    model::Account,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mechanism {
    /// Accept fewer whole lots, down to the claimant's authorized minimum.
    #[default]
    Immediate,
    /// Accept the entire requested bundle or reserve nothing.
    ConditionalBundle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub claim: Claim,
    /// Immediate prerequisites per lot. Include private capacity as scoped accounts.
    /// Expected outputs and future deliveries are not available prerequisites.
    pub inputs: BTreeMap<Account, u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortfall {
    pub account: Account,
    pub required: u64,
    pub available: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub context: Context,
    pub mechanism: Mechanism,
    pub receipts: Vec<Receipt>,
    pub shortfalls: BTreeMap<u64, Vec<Shortfall>>,
    pub remaining: BTreeMap<Account, u32>,
}

/// Preview a dated resolution. Only accepted candidates update the returned plan.
/// The callback must modify only its supplied, independently cloned plan (no
/// external side effects or interior shared mutation). It checks permissions,
/// storage, and other domain constraints before any reservation is retained.
/// Neither this function nor its receipts settle the live simulation.
pub fn resolve<P: Clone>(
    context: Context,
    policy: &dyn RankingPolicy,
    mechanism: Mechanism,
    available: &BTreeMap<Account, u32>,
    requests: &[Request],
    opening_plan: &P,
    mut accept: impl FnMut(&mut P, &Claim, u32) -> Result<(), String>,
) -> Result<(Resolution, P), String> {
    let mut ids = BTreeSet::new();
    if requests.iter().any(|r| {
        !ids.insert(r.claim.id)
            || r.claim.minimum == 0
            || r.claim.minimum > r.claim.requested
            || r.inputs.is_empty()
            || r.inputs.values().any(|q| *q == 0)
    }) {
        return Err("duplicate or invalid resolution request".into());
    }
    let mut ordered: Vec<_> = requests.iter().collect();
    ordered.sort_by_key(|r| (policy.key(context, &r.claim), r.claim.id));
    let mut result = Resolution {
        context,
        mechanism,
        receipts: vec![],
        shortfalls: BTreeMap::new(),
        remaining: available.clone(),
    };
    let mut plan = opening_plan.clone();
    for r in ordered {
        let c = &r.claim;
        let offered = r.inputs.iter().fold(c.requested, |n, (a, q)| {
            n.min(result.remaining.get(a).copied().unwrap_or(0) / q)
        });
        let minimum = match mechanism {
            Mechanism::Immediate => c.minimum,
            Mechanism::ConditionalBundle => c.requested,
        };
        let outcome = if offered < minimum {
            result.shortfalls.insert(
                c.id,
                r.inputs
                    .iter()
                    .filter_map(|(account, q)| {
                        let required = u64::from(*q) * u64::from(minimum);
                        let available = result.remaining.get(account).copied().unwrap_or(0);
                        (required > u64::from(available)).then_some(Shortfall {
                            account: *account,
                            required,
                            available,
                        })
                    })
                    .collect(),
            );
            Outcome::InsufficientCapacity
        } else {
            let mut candidate = plan.clone();
            match accept(&mut candidate, c, offered) {
                Err(reason) => Outcome::Rejected(reason),
                Ok(()) => {
                    for (account, q) in &r.inputs {
                        // offered is bounded by every input's available / quantity.
                        *result.remaining.get_mut(account).unwrap() -= offered * q;
                    }
                    plan = candidate;
                    Outcome::Reserved(offered)
                }
            }
        };
        result.receipts.push(Receipt {
            claim: c.clone(),
            offered,
            outcome,
        });
    }
    Ok((result, plan))
}
