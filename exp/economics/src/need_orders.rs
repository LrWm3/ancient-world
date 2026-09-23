//! Bounded need-directed orders for one listed bilateral market.
//! Quantities respond to actual stocks; prices remain a separate quote policy.
use crate::{acquisition::Resources, model::*, negotiation, opportunities, substitution};
use std::collections::BTreeMap;

pub(crate) const MAX_RESERVE_MONTHS: u32 = 24;
const EXAMPLE_RESERVE_MONTHS: u32 = 2;
const EXAMPLE_SELLER: AgentId = 89;
const EXAMPLE_GRAIN: i32 = 10;
const EXAMPLE_CASH: i32 = 200;
const EXAMPLE_NEED: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    /// Protect this many months of consumption without assuming future production.
    pub reserve_months: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub agent: AgentId,
    pub goods: Amount,
    /// Supplied reservation value per whole market lot, in payment units.
    pub limit: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub month: u32,
    pub buy: Option<Order>,
    pub sell: Option<Order>,
    pub buyer_deficits: BTreeMap<ResourceId, i64>,
    pub buyer_after_purchase: BTreeMap<ResourceId, i64>,
    /// Protection includes needs and known commitments; it is not a transfer.
    pub protected: BTreeMap<Account, i128>,
}

pub fn validate(world: &World) -> Result<(), String> {
    let Some(p) = &world.need_orders else {
        return Ok(());
    };
    let s = world
        .negotiation
        .as_ref()
        .ok_or("need orders require a bilateral market template")?;
    if !(1..=MAX_RESERVE_MONTHS).contains(&p.reserve_months)
        || [&s.buyer, &s.seller]
            .iter()
            .any(|t| !world.participants.iter().any(|p| p.agent == t.agent))
    {
        return Err("need orders require two participants and a bounded reserve horizon".into());
    }
    Ok(())
}

/// Existing accepted stock claims and unpaid process entry inputs. No expected
/// harvest is treated as stock; already-consumed stage inputs are not reserved twice.
fn claims(
    world: &World,
    state: &State,
    agent: AgentId,
    months: u32,
) -> Result<BTreeMap<ResourceId, i128>, String> {
    let mut result: BTreeMap<_, _> = world
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Stock)
        .map(|r| {
            (
                r.id,
                crate::commitments::projected_claims(world, state, agent, r.id, u64::from(months))
                    .values()
                    .sum(),
            )
        })
        .collect();
    for p in state
        .processes
        .values()
        .filter(|p| p.operator == agent && p.status == Status::Active)
    {
        for (index, stage) in world
            .definition(p.definition)
            .stages
            .iter()
            .enumerate()
            .skip(p.stage)
        {
            if index == p.stage && p.elapsed > 0 {
                continue;
            }
            for input in &stage.entry_inputs {
                *result.entry(input.resource).or_default() += i128::from(input.quantity);
            }
        }
    }
    // Only already accrued, currently collectible loan claims. Forecasting future
    // interest/installments remains the borrowing planner's responsibility.
    for l in state.credit.loans.values().filter(|l| {
        l.debtor == agent
            && !matches!(
                l.status,
                crate::credit::Status::Repaid | crate::credit::Status::PendingSale
            )
    }) {
        *result.entry(l.denomination).or_default() += i128::from(l.due(state.month)?);
    }
    Ok(result)
}

fn stock_map(balances: &BTreeMap<Account, i32>, agent: AgentId) -> BTreeMap<ResourceId, i128> {
    balances
        .iter()
        .filter(|((a, _), _)| *a == agent)
        .map(|((_, r), q)| (*r, i128::from(*q)))
        .collect()
}

/// Deterministic consumption uses the same recipe/substitution rules as live work.
/// No input can satisfy two needs or two months in this projection.
fn consume(
    world: &World,
    state: &State,
    agent: AgentId,
    months: u32,
    stocks: &mut BTreeMap<ResourceId, i128>,
    protect_partial: bool,
) -> BTreeMap<ResourceId, i64> {
    let p = world
        .participants
        .iter()
        .find(|p| p.agent == agent)
        .unwrap();
    let mut deficits = BTreeMap::new();
    for month in 0..months {
        for need in crate::forecast::needs::ordered(&p.needs) {
            let fulfilled = if month == 0 {
                state.balance(agent, need.resource)
            } else {
                0
            };
            let wanted = i128::from((need.quantity - fulfilled).max(0));
            let recipes = substitution::recipes_for(world, state, agent, need.resource);
            let (_, unmet) =
                substitution::allocate_recipes(&recipes, wanted, stocks, &BTreeMap::new());
            *deficits.entry(need.resource).or_default() += unmet as i64;
            // A partial stock below a recipe's minimum is still useful to retain.
            if protect_partial && unmet > 0 {
                for d in &recipes {
                    let input = &d.stages[0].entry_inputs[0];
                    let stock = stocks.entry(input.resource).or_default();
                    let lots = (unmet + i128::from(d.outputs[0].quantity) - 1)
                        / i128::from(d.outputs[0].quantity);
                    *stock -= (*stock).min(lots * i128::from(input.quantity));
                }
            }
        }
    }
    deficits
}

fn unclaimed(
    stocks: &BTreeMap<ResourceId, i128>,
    claims: &BTreeMap<ResourceId, i128>,
) -> BTreeMap<ResourceId, i128> {
    stocks
        .iter()
        .map(|(r, q)| (*r, (*q - claims.get(r).copied().unwrap_or(0)).max(0)))
        .collect()
}

pub(crate) fn generate(
    world: &World,
    state: &State,
    resources: &Resources,
) -> Result<Option<Decision>, String> {
    let Some(policy) = &world.need_orders else {
        return Ok(None);
    };
    let session = world.negotiation.as_ref().ok_or("missing order template")?;
    generate_for(world, state, resources, policy, session).map(Some)
}

pub(crate) fn generate_for(
    world: &World,
    state: &State,
    resources: &Resources,
    policy: &Policy,
    session: &negotiation::Session,
) -> Result<Decision, String> {
    generate_for_horizon(world, state, resources, policy, session, 1)
}

/// Buying ahead is an explicit planner horizon; legacy orders remain immediate.
pub(crate) fn generate_for_horizon(
    world: &World,
    state: &State,
    resources: &Resources,
    policy: &Policy,
    session: &negotiation::Session,
    months: u32,
) -> Result<Decision, String> {
    let mut observed = state.clone();
    observed.balances = resources.holdings.clone();
    let mut protected = BTreeMap::new();
    for t in [&session.buyer, &session.seller] {
        let commitments = claims(world, &observed, t.agent, policy.reserve_months)?;
        let stocks = stock_map(&resources.holdings, t.agent);
        let opening = unclaimed(&stocks, &commitments);
        let mut remaining = opening.clone();
        consume(
            world,
            &observed,
            t.agent,
            policy.reserve_months,
            &mut remaining,
            true,
        );
        for r in world
            .resources
            .iter()
            .filter(|r| r.kind == ResourceKind::Stock)
        {
            let quantity = commitments.get(&r.id).copied().unwrap_or(0)
                + opening.get(&r.id).copied().unwrap_or(0)
                - remaining.get(&r.id).copied().unwrap_or(0);
            if quantity > 0 {
                protected.insert((t.agent, r.id), quantity);
            }
        }
    }
    let agent = session.buyer.agent;
    let commitments = claims(world, &observed, agent, policy.reserve_months)?;
    let mut before = unclaimed(&stock_map(&resources.holdings, agent), &commitments);
    let mut with_goods = stock_map(&resources.holdings, agent);
    *with_goods.entry(session.goods.resource).or_default() += i128::from(session.goods.quantity);
    let mut after = unclaimed(&with_goods, &commitments);
    let buyer_deficits = consume(world, &observed, agent, months, &mut before, false);
    let buyer_after_purchase = consume(world, &observed, agent, months, &mut after, false);
    let useful = buyer_after_purchase
        .iter()
        .any(|(r, q)| *q < buyer_deficits[r])
        && buyer_after_purchase
            .iter()
            .all(|(r, q)| *q <= buyer_deficits[r]);
    let account = (session.seller.agent, session.goods.resource);
    let surplus = i128::from(resources.available.get(&account).copied().unwrap_or(0))
        - protected.get(&account).copied().unwrap_or(0);
    Ok(Decision {
        month: state.month,
        buy: useful.then(|| Order {
            agent,
            goods: session.goods.clone(),
            limit: session.buyer.limit,
        }),
        sell: (surplus >= i128::from(session.goods.quantity)).then(|| Order {
            agent: session.seller.agent,
            goods: session.goods.clone(),
            limit: session.seller.limit,
        }),
        buyer_deficits,
        buyer_after_purchase,
        protected,
    })
}

/// Two people consuming finite grain stocks; order generation runs before consumption.
pub fn scenario() -> (World, State) {
    use crate::scenario::{GRAIN, LABOR, NUTRITION, PERSON};
    let (mut world, mut state) = negotiation::scenario();
    let (catalog, _) = crate::scenario::baseline();
    world.resources.extend(
        catalog
            .resources
            .into_iter()
            .filter(|r| [LABOR, NUTRITION].contains(&r.id)),
    );
    world.definitions = catalog
        .definitions
        .into_iter()
        .filter(|d| d.execution == Execution::Consumption)
        .collect();
    world.participants = [PERSON, EXAMPLE_SELLER]
        .into_iter()
        .map(|agent| Participant {
            agent,
            capacity: Amount::new(LABOR, 0),
            needs: vec![Requirement {
                resource: NUTRITION,
                quantity: EXAMPLE_NEED,
                priority: 0,
            }],
        })
        .collect();
    for d in &world.definitions {
        world
            .transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .insert((
                opportunities::PERSON_TYPE,
                opportunities::Action::Process(d.id),
            ));
    }
    world.need_orders = Some(Policy {
        reserve_months: EXAMPLE_RESERVE_MONTHS,
    });
    state
        .balances
        .insert((EXAMPLE_SELLER, GRAIN), EXAMPLE_GRAIN);
    state
        .balances
        .insert((PERSON, crate::scenario::TOKEN), EXAMPLE_CASH);
    (world, state)
}
