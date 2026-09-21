//! Policy for an oversubscribed resource pool, independent of market side or terms.
//! Eligibility and physical reservation belong to the caller, not the ranking policy.
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub seed: u64,
    pub pool: u64,
    pub round: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    /// Stable application identity, never its position in a vector.
    pub id: u64,
    pub priority: u32,
    pub requested: u32,
    pub minimum: u32,
}
/// Lower keys rank first. Domain adapters compute priority (e.g. no existing plot).
pub trait RankingPolicy {
    fn key(&self, context: Context, claim: &Claim) -> (u32, u64);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    PriorityLottery,
    Lottery,
    StablePriority,
}
impl RankingPolicy for Policy {
    fn key(&self, c: Context, claim: &Claim) -> (u32, u64) {
        let lottery = mix(c.seed ^ mix(c.pool) ^ mix(c.round) ^ mix(claim.id));
        match self {
            Self::PriorityLottery => (claim.priority, lottery),
            Self::Lottery => (0, lottery),
            Self::StablePriority => (claim.priority, claim.id),
        }
    }
}
// Fixed integer mixing rather than platform-dependent hashing or mutable RNG state.
fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Reserved(u32),
    InsufficientCapacity,
    Rejected(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub claim: Claim,
    pub offered: u32,
    pub outcome: Outcome,
}
/// Resolve one dated pool. Failed reservations release the entire proposed grant;
/// a below-minimum grant never invokes the reservation callback. Caller callbacks
/// must themselves be atomic on failure. Results are in policy order.
pub fn resolve(
    context: Context,
    policy: &dyn RankingPolicy,
    capacity: u32,
    claims: &[Claim],
    mut reserve: impl FnMut(&Claim, u32) -> Result<(), String>,
) -> Result<Vec<Receipt>, String> {
    let mut ids = BTreeSet::new();
    if claims
        .iter()
        .any(|c| !ids.insert(c.id) || c.minimum == 0 || c.minimum > c.requested)
    {
        return Err("duplicate or invalid allocation claim".into());
    }
    let mut ordered: Vec<_> = claims.iter().collect();
    ordered.sort_by_key(|c| (policy.key(context, c), c.id));
    let mut available = capacity;
    Ok(ordered
        .into_iter()
        .map(|c| {
            let offered = available.min(c.requested);
            let outcome = if offered < c.minimum {
                Outcome::InsufficientCapacity
            } else {
                match reserve(c, offered) {
                    Ok(()) => {
                        available -= offered;
                        Outcome::Reserved(offered)
                    }
                    Err(reason) => Outcome::Rejected(reason),
                }
            };
            Receipt {
                claim: c.clone(),
                offered,
                outcome,
            }
        })
        .collect())
}

/// One unit per claimant, with interchangeable acceptable slots. Earlier-ranked
/// claimants keep admission; their tentative slot may move to admit another claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alternatives {
    pub claim: Claim,
    pub slots: Vec<u64>,
}
pub fn assign(
    context: Context,
    policy: &dyn RankingPolicy,
    alternatives: &[Alternatives],
) -> Result<(Vec<Receipt>, std::collections::BTreeMap<u64, u64>), String> {
    use std::collections::BTreeMap;
    if alternatives
        .iter()
        .any(|a| a.claim.requested != 1 || a.claim.minimum != 1)
    {
        return Err("alternative slots require unit claims".into());
    }
    let choices: BTreeMap<_, Vec<_>> = alternatives
        .iter()
        .map(|a| {
            let slots: BTreeSet<_> = a.slots.iter().copied().collect();
            (a.claim.id, slots.into_iter().collect())
        })
        .collect();
    fn augment(
        id: u64,
        choices: &BTreeMap<u64, Vec<u64>>,
        occupied: &mut BTreeMap<u64, u64>,
        seen: &mut BTreeSet<u64>,
    ) -> bool {
        for &slot in &choices[&id] {
            if !seen.insert(slot) {
                continue;
            }
            let previous = occupied.get(&slot).copied();
            if previous.is_none_or(|other| augment(other, choices, occupied, seen)) {
                occupied.insert(slot, id);
                return true;
            }
        }
        false
    }
    let capacity = choices
        .values()
        .flatten()
        .copied()
        .collect::<BTreeSet<_>>()
        .len();
    let mut occupied = BTreeMap::new();
    let claims: Vec<_> = alternatives.iter().map(|a| a.claim.clone()).collect();
    let receipts = resolve(
        context,
        policy,
        u32::try_from(capacity).map_err(|_| "too many slots")?,
        &claims,
        |claim, _| {
            let mut tentative = occupied.clone();
            if !augment(claim.id, &choices, &mut tentative, &mut BTreeSet::new()) {
                return Err("no available acceptable alternative".into());
            }
            occupied = tentative;
            Ok(())
        },
    )?;
    Ok((
        receipts,
        occupied.into_iter().map(|(slot, id)| (id, slot)).collect(),
    ))
}
