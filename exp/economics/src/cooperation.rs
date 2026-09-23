//! Bounded cooperative discovery. Both mechanisms publish the same dated contract.
//! Delivery-versus-payment uses opening budgets; failure cancels future deliveries.
use crate::{
    acquisition::Resources,
    compute::Backend,
    finance::Transfer,
    forecast::ForecastContext,
    marketplace,
    model::*,
    production_market::{Choice, Policy, Purchases, Work},
    simulation::Simulation,
    town_market::{self, MarketResult, Round},
};
use std::collections::BTreeMap;

const TERM_MONTHS: u32 = 6;
const BUFFER_MONTHS: i128 = 2;
const MAX_DELIVERIES: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Discovery {
    Mutual,
    Posted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivery {
    pub month: u32,
    pub market: marketplace::MarketId,
    pub goods: Transfer,
    pub payment: Transfer,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    pub start: u32,
    pub through: u32,
    pub choices: BTreeMap<AgentId, Choice>,
    pub deliveries: Vec<Delivery>,
}
/// Public offer terms contain no counterparty work policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub proposer: AgentId,
    pub expires: u32,
    pub deliveries: Vec<Delivery>,
    /// Reply assessments are diagnostic receipts, not shared planning inputs.
    pub proposer_assessment: Assessment,
    pub replies: Vec<Assessment>,
    pub accepted: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    pub terminal: bool,
    pub deficits: Vec<i64>,
    pub failures: usize,
    pub buffer_gap: i128,
    pub productive_labor: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    pub agent: AgentId,
    pub choice: Choice,
    pub baseline: Score,
    pub proposed: Score,
    pub closing_coins: i32,
    pub acceptable: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boundary {
    /// Start month identifies the single scoped agreement, including its failure receipt.
    pub agreement: Option<u32>,
    pub active: Option<Contract>,
    pub offers: Vec<Offer>,
    pub assessments: Vec<Assessment>,
    pub event: String,
    pub projections: usize,
    pub joint_projections: usize,
    pub completed: Vec<Delivery>,
    pub failure: Option<String>,
}

fn policy(w: &World) -> Option<&Policy> {
    w.production_market.as_ref().map(|c| &c.policy)
}
fn latest(s: &State) -> Option<&Boundary> {
    s.town_market
        .history
        .last()
        .and_then(|r| r.cooperation.as_deref())
}
pub fn choices(w: &World, s: &State) -> Option<BTreeMap<AgentId, Choice>> {
    match policy(w) {
        Some(Policy::Agreement(c)) => Some(if (c.start..=c.through).contains(&s.month) {
            c.choices.clone()
        } else {
            BTreeMap::new()
        }),
        Some(Policy::Cooperate(_)) => Some(
            latest(s)
                .and_then(|b| b.active.as_ref())
                .filter(|c| s.month <= c.through)
                .map(|c| c.choices.clone())
                .unwrap_or_default(),
        ),
        _ => None,
    }
}
fn people(w: &World) -> Vec<AgentId> {
    let mut ids: Vec<_> = w.participants.iter().map(|p| p.agent).collect();
    ids.sort();
    ids
}
fn candidates(w: &World) -> Vec<Choice> {
    let mut works = vec![Work::Ordinary, Work::Wait];
    let mut ids: Vec<_> = w
        .definitions
        .iter()
        .filter(|d| d.enabled && d.execution == Execution::Productive)
        .map(|d| d.id)
        .collect();
    ids.sort();
    works.extend(ids.into_iter().map(Work::Produce));
    works
        .into_iter()
        .map(|work| Choice {
            work,
            buy: Purchases::None,
        })
        .collect()
}
pub fn validate(w: &World) -> Result<(), String> {
    if !matches!(policy(w), Some(Policy::Cooperate(_) | Policy::Agreement(_))) {
        return Ok(());
    }
    if w.participants.len() != 2
        || w.production_market.as_ref().unwrap().horizon != TERM_MONTHS
        || !w.production_market.as_ref().unwrap().trading
    {
        return Err("cooperation pilot requires two people and a six-month horizon".into());
    }
    if let Some(Policy::Agreement(c)) = policy(w) {
        validate_contract(w, c)?;
    }
    Ok(())
}
fn validate_contract(w: &World, c: &Contract) -> Result<(), String> {
    if c.start == 0
        || c.start.checked_add(TERM_MONTHS - 1) != Some(c.through)
        || c.choices.keys().copied().collect::<Vec<_>>() != people(w)
        || c.choices
            .values()
            .any(|choice| !candidates(w).contains(choice))
        || c.deliveries.len() > MAX_DELIVERIES
    {
        return Err("invalid cooperative contract".into());
    }
    let venue = marketplace::venue(w, w.town_market.as_ref().ok_or("missing market")?.venue)
        .ok_or("missing venue")?;
    for d in &c.deliveries {
        let m = venue
            .markets
            .iter()
            .find(|m| m.id == d.market)
            .ok_or("unknown contract market")?;
        if !(c.start..=c.through).contains(&d.month)
            || d.goods.amount != m.goods
            || d.payment.amount.resource != m.payment
            || d.payment.amount.quantity <= 0
            || d.goods.from != d.payment.to
            || d.goods.to != d.payment.from
            || !c.choices.contains_key(&d.goods.from)
            || !c.choices.contains_key(&d.goods.to)
        {
            return Err("invalid dated delivery terms".into());
        }
        d.goods.effects()?;
        d.payment.effects()?;
    }
    Ok(())
}
pub(crate) fn validate_state(w: &World, s: &State) -> Result<(), String> {
    if matches!(policy(w), Some(Policy::Cooperate(_) | Policy::Agreement(_)))
        && !matches!(s.phase, Phase::Open | Phase::Acquire)
        && s.town_market
            .history
            .last()
            .is_none_or(|r| r.month != s.month || r.cooperation.is_none())
    {
        return Err("missing current cooperative boundary".into());
    }
    if latest(s)
        .and_then(|b| b.active.as_ref())
        .is_some_and(|c| s.month <= c.through)
        && !matches!(policy(w), Some(Policy::Cooperate(_) | Policy::Agreement(_)))
    {
        return Err("cannot discard an active cooperative agreement by changing planner".into());
    }
    for r in &s.town_market.history {
        if let Some(b) = &r.cooperation {
            if let Some(c) = &b.active {
                validate_contract(w, c)?;
                if !(c.start..=c.through).contains(&r.month) || b.agreement != Some(c.start) {
                    return Err("stale cooperative contract".into());
                }
            }
            if b.completed.iter().any(|d| d.month != r.month) {
                return Err("misdated contract receipt".into());
            }
        }
    }
    Ok(())
}

/// The bounded public menu is equal monthly per-person quantities, rounded into
/// existing whole market lots. Both discovery mechanisms get exactly this menu.
fn terms(
    w: &World,
    s: &State,
    seller: AgentId,
    market: marketplace::MarketId,
) -> Result<Vec<Delivery>, String> {
    let ids = people(w);
    let other = *ids
        .iter()
        .find(|a| **a != seller)
        .ok_or("missing counterparty")?;
    let town = w.town_market.as_ref().unwrap();
    let venue = marketplace::venue(w, town.venue).unwrap();
    let mut out = vec![];
    for listing in town_market::listings(town) {
        let m = venue
            .markets
            .iter()
            .find(|m| m.id == listing.market)
            .unwrap();
        // One resource unit per month: grain lots of two every second month,
        // fuel lots of one monthly. This is a fixture menu, not inferred demand.
        let interval = u32::try_from(m.goods.quantity).map_err(|_| "invalid lot")?;
        if interval == 0 || !TERM_MONTHS.is_multiple_of(interval) {
            return Err("lot does not divide contract term".into());
        }
        let from = if m.id == market { seller } else { other };
        let to = if from == seller { other } else { seller };
        let quote = listing
            .traders
            .iter()
            .find(|t| t.trader.agent == from)
            .ok_or("unregistered seller")?
            .trader
            .limit;
        for offset in (interval - 1..TERM_MONTHS).step_by(interval as usize) {
            out.push(Delivery {
                month: s
                    .month
                    .checked_add(offset)
                    .ok_or("contract date overflow")?,
                market: m.id,
                goods: Transfer {
                    from,
                    to,
                    amount: m.goods.clone(),
                },
                payment: Transfer {
                    from: to,
                    to: from,
                    amount: Amount::new(m.payment, quote),
                },
            });
        }
    }
    out.sort_by_key(|d| (d.month, d.market));
    Ok(out)
}
/// Same quantities and prices, either regular delivery or the seller's goods
/// deferred to the last month. Payment remains delivery-versus-payment.
fn menus(
    w: &World,
    s: &State,
    seller: AgentId,
    market: marketplace::MarketId,
) -> Result<Vec<Vec<Delivery>>, String> {
    let regular = terms(w, s, seller, market)?;
    let mut deferred = regular.clone();
    let end = s
        .month
        .checked_add(TERM_MONTHS - 1)
        .ok_or("contract date overflow")?;
    for d in &mut deferred {
        if d.goods.from == seller {
            d.month = end;
        }
    }
    deferred.sort_by_key(|d| (d.month, d.market));
    Ok(vec![deferred, regular])
}

fn contract(
    s: &State,
    choices: BTreeMap<AgentId, Choice>,
    deliveries: Vec<Delivery>,
) -> Result<Contract, String> {
    Ok(Contract {
        start: s.month,
        through: s
            .month
            .checked_add(TERM_MONTHS - 1)
            .ok_or("contract horizon overflow")?,
        choices,
        deliveries,
    })
}
fn eligible(w: &World, s: &State, c: &Contract) -> bool {
    let town = w.town_market.as_ref().unwrap();
    c.choices.keys().all(|a| {
        !s.terminal.contains_key(a)
            && s.town_market
                .admission
                .as_ref()
                .is_some_and(|v| v.month == s.month && v.eligible.contains(a))
            && marketplace::eligible(w, s, town.venue, *a)
            && crate::opportunities::permits(w, s, *a, crate::opportunities::Action::StockTrade)
    })
}
fn settle(w: &World, s: &State, c: &Contract) -> Result<Vec<Transaction>, String> {
    if !eligible(w, s, c) {
        return Err("participant unavailable or outside marketplace".into());
    }
    let mut resources = Resources::opening(w, s);
    let mut transactions = vec![];
    for d in c.deliveries.iter().filter(|d| d.month == s.month) {
        for transfer in [&d.goods, &d.payment] {
            let available = resources
                .available
                .get(&(transfer.from, transfer.amount.resource))
                .copied()
                .unwrap_or(0);
            if available < transfer.amount.quantity {
                return Err(format!(
                    "market {} due {}: agent {} resource {} requires {}, opening budget {}",
                    d.market,
                    d.month,
                    transfer.from,
                    transfer.amount.resource,
                    transfer.amount.quantity,
                    available
                ));
            }
        }
        let mut effects = d.goods.effects()?;
        effects.extend(d.payment.effects()?);
        let t = Transaction {
            cause: format!(
                "cooperative agreement {} market {} delivery",
                c.start, d.market
            ),
            effects,
            process: None,
            technique_use: None,
            trade: None,
            stock_trade: None,
            forward: None,
            delivery: None,
            royalty: None,
        };
        resources
            .reserve(w, std::slice::from_ref(&t))
            .map_err(|e| format!("market {} due {}: {e}", d.market, d.month))?;
        transactions.push(t);
    }
    Ok(transactions)
}

#[derive(Clone)]
struct Projection {
    scores: BTreeMap<AgentId, Score>,
    coins: BTreeMap<AgentId, i32>,
    failed: bool,
}
/// Conditional individual forecasts endow ONLY the other party with its promised
/// outgoing amounts, disable its needs/work, and never inspect its chosen plan.
/// This is a promise assumption, not a live grant or a guarantee.
fn project(
    w: &World,
    s: &State,
    c: &Contract,
    conditional: Option<AgentId>,
) -> Result<Projection, String> {
    let (mut world, mut state) = ForecastContext::new(w, s).into_parts();
    for r in &mut state.town_market.history {
        r.planning = None;
        r.cooperation = None;
    }
    let mut hypothesis = c.clone();
    if let Some(agent) = conditional {
        let other = *people(w).iter().find(|a| **a != agent).unwrap();
        hypothesis.choices.insert(
            other,
            Choice {
                work: Work::Wait,
                buy: Purchases::None,
            },
        );
        let p = world
            .participants
            .iter_mut()
            .find(|p| p.agent == other)
            .unwrap();
        p.needs.clear();
        p.capacity.quantity = 0;
        state.processes.retain(|_, p| p.operator != other);
        state.balances.retain(|(owner, _), _| *owner != other);
        // Only the counterparty's hypothetical budget changes. No promised
        // receipts finance the evaluating person's earlier outgoing payments.
        for d in &c.deliveries {
            for t in [&d.goods, &d.payment] {
                if t.from == other {
                    let q = state
                        .balances
                        .entry((other, t.amount.resource))
                        .or_default();
                    *q = q
                        .checked_add(t.amount.quantity)
                        .ok_or("promise budget overflow")?;
                }
            }
        }
        world.storage.capacities.remove(&other);
    }
    world.production_market.as_mut().unwrap().policy = Policy::Agreement(Box::new(hypothesis));
    let mut sim = Simulation::new(world, state, Backend::Reference)?;
    while sim.state.month <= c.through {
        sim.step()?;
    }
    let coin = marketplace::venue(w, w.town_market.as_ref().unwrap().venue)
        .unwrap()
        .markets[0]
        .payment;
    let mut scores = BTreeMap::new();
    let mut coins = BTreeMap::new();
    for p in &w.participants {
        let mut deficits = BTreeMap::new();
        for report in sim.reports.iter().filter(|r| r.agent == p.agent) {
            crate::forecast::needs::accumulate(
                &mut deficits,
                report.needs.iter().map(|(r, n)| (*r, n.deficit)),
            );
        }
        let mut stocks = crate::substitution::stocks(&sim.state, p.agent);
        let mut buffer_gap = 0;
        for n in crate::forecast::needs::ordered(&p.needs) {
            let recipes = crate::substitution::recipes_for(w, &sim.state, p.agent, n.resource);
            buffer_gap += crate::substitution::allocate_recipes(
                &recipes,
                i128::from(n.quantity) * BUFFER_MONTHS,
                &mut stocks,
                &BTreeMap::new(),
            )
            .1;
        }
        let failures = sim
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter_map(|t| t.process.as_ref())
            .filter(|p2| p2.after.operator == p.agent && p2.after.status == Status::Aborted)
            .count();
        let labor = sim
            .ledger
            .iter()
            .filter(|b| b.phase == Phase::Productive)
            .flat_map(|b| &b.transactions)
            .flat_map(|t| &t.effects)
            .filter(|e| e.account == (p.agent, p.capacity.resource) && e.delta < 0)
            .map(|e| -i64::from(e.delta))
            .sum();
        scores.insert(
            p.agent,
            Score {
                terminal: sim.state.terminal.contains_key(&p.agent),
                deficits: crate::forecast::needs::score(&p.needs, &deficits),
                failures,
                buffer_gap,
                productive_labor: labor,
            },
        );
        coins.insert(p.agent, sim.state.balance(p.agent, coin));
    }
    let failed = sim
        .state
        .town_market
        .history
        .iter()
        .filter(|r| r.month >= s.month)
        .any(|r| r.cooperation.as_ref().is_some_and(|b| b.failure.is_some()));
    Ok(Projection {
        scores,
        coins,
        failed,
    })
}
fn assess(
    w: &World,
    s: &State,
    c: &Contract,
    p: &Projection,
    baseline: &BTreeMap<AgentId, Score>,
) -> Vec<Assessment> {
    let coin = marketplace::venue(w, w.town_market.as_ref().unwrap().venue)
        .unwrap()
        .markets[0]
        .payment;
    c.choices
        .iter()
        .map(|(&agent, &choice)| Assessment {
            agent,
            choice,
            baseline: baseline[&agent].clone(),
            proposed: p.scores[&agent].clone(),
            closing_coins: p.coins[&agent],
            acceptable: !p.failed
                && p.scores[&agent] <= baseline[&agent]
                && p.coins[&agent] >= s.balance(agent, coin),
        })
        .collect()
}
fn baseline(w: &World, s: &State, count: &mut usize) -> Result<BTreeMap<AgentId, Score>, String> {
    let ids = people(w);
    let mut best = BTreeMap::new();
    for agent in &ids {
        for choice in candidates(w) {
            let choices = ids
                .iter()
                .map(|a| {
                    (
                        *a,
                        if a == agent {
                            choice
                        } else {
                            Choice::default()
                        },
                    )
                })
                .map(|(a, mut c)| {
                    c.buy = Purchases::None;
                    (a, c)
                })
                .collect();
            let c = contract(s, choices, vec![])?;
            *count += 1;
            let p = project(w, s, &c, Some(*agent))?;
            if best.get(agent).is_none_or(|v| &p.scores[agent] < v) {
                best.insert(*agent, p.scores[agent].clone());
            }
        }
    }
    Ok(best)
}
fn discover(
    w: &World,
    s: &State,
    mode: Discovery,
    b: &mut Boundary,
) -> Result<Option<Contract>, String> {
    let ids = people(w);
    let base = baseline(w, s, &mut b.projections)?;
    let markets: Vec<_> = town_market::listings(w.town_market.as_ref().unwrap())
        .iter()
        .map(|m| m.market)
        .collect();
    let mut accepted: Vec<(Vec<Score>, Contract, Vec<Assessment>)> = vec![];

    if mode == Discovery::Mutual {
        let mut seen = vec![];
        for seller in &ids {
            for market in &markets {
                for deliveries in menus(w, s, *seller, *market)? {
                    if seen.contains(&deliveries) {
                        continue;
                    }
                    seen.push(deliveries.clone());
                    for a in candidates(w) {
                        for z in candidates(w) {
                            let c = contract(
                                s,
                                BTreeMap::from([(ids[0], a), (ids[1], z)]),
                                deliveries.clone(),
                            )?;
                            b.projections += 1;
                            b.joint_projections += 1;
                            let p = project(w, s, &c, None)?;
                            let checks = assess(w, s, &c, &p, &base);
                            if checks.iter().all(|v| v.acceptable)
                                && checks.iter().any(|v| v.proposed < v.baseline)
                            {
                                accepted.push((
                                    ids.iter().map(|a| p.scores[a].clone()).collect(),
                                    c,
                                    checks,
                                ));
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Stable proposer priority. Each proposer ranks distinct public terms by
        // its own best plan; rejection advances to its next terms in this boundary.
        for proposer in &ids {
            let other = *ids.iter().find(|a| *a != proposer).unwrap();
            let mut offers = vec![];
            for market in &markets {
                for deliveries in menus(w, s, *proposer, *market)? {
                    let mut plans = vec![];
                    for choice in candidates(w) {
                        let c = contract(
                            s,
                            BTreeMap::from([
                                (*proposer, choice),
                                (
                                    other,
                                    Choice {
                                        work: Work::Wait,
                                        buy: Purchases::None,
                                    },
                                ),
                            ]),
                            deliveries.clone(),
                        )?;
                        b.projections += 1;
                        let p = project(w, s, &c, Some(*proposer))?;
                        let check = assess(w, s, &c, &p, &base)
                            .into_iter()
                            .find(|a| a.agent == *proposer)
                            .unwrap();
                        if check.acceptable && check.proposed < check.baseline {
                            plans.push((check, c));
                        }
                    }
                    plans.sort_by(|a, b| a.0.proposed.cmp(&b.0.proposed));
                    if let Some(plan) = plans.into_iter().next() {
                        offers.push(plan);
                    }
                }
            }
            offers.sort_by(|a, b| a.0.proposed.cmp(&b.0.proposed));
            for (proposal, offer) in offers {
                let mut replies = vec![];
                for choice in candidates(w) {
                    let mut c = offer.clone();
                    c.choices.insert(other, choice);
                    b.projections += 1;
                    let p = project(w, s, &c, Some(other))?;
                    let check = assess(w, s, &c, &p, &base)
                        .into_iter()
                        .find(|a| a.agent == other)
                        .unwrap();
                    replies.push((check, c));
                }
                replies.sort_by(|a, b| a.0.proposed.cmp(&b.0.proposed));
                let selected = replies.iter().find(|(a, _)| a.acceptable);
                b.offers.push(Offer {
                    proposer: *proposer,
                    expires: s.month,
                    deliveries: offer.deliveries.clone(),
                    proposer_assessment: proposal.clone(),
                    replies: replies.iter().map(|(a, _)| a.clone()).collect(),
                    accepted: selected.is_some(),
                });
                if let Some((reply, c)) = selected {
                    // Two independent consents. No joint rollout, score arbitration,
                    // or access to the other person's selected work in either forecast.
                    b.assessments = vec![proposal, reply.clone()];
                    b.assessments.sort_by_key(|a| a.agent);
                    return Ok(Some(c.clone()));
                }
            }
        }
    }
    // Stable, explicit arbitration among mutually acceptable candidates. This is
    // not utilitarian optimization: first sorted participant's score breaks ties.
    accepted.sort_by(|a, b| a.0.cmp(&b.0));
    if let Some((_, c, checks)) = accepted.into_iter().next() {
        b.assessments = checks;
        Ok(Some(c))
    } else {
        Ok(None)
    }
}

pub fn evaluate(w: &World, s: &State) -> Result<Option<Round>, String> {
    let Some(policy @ (Policy::Cooperate(_) | Policy::Agreement(_))) = policy(w) else {
        return Ok(None);
    };
    validate(w)?;
    if s.phase != Phase::Acquire {
        return Err("cooperation requires Acquire".into());
    }
    let mut b = Boundary {
        agreement: None,
        active: None,
        offers: vec![],
        assessments: vec![],
        event: "Continuing".into(),
        projections: 0,
        joint_projections: 0,
        completed: vec![],
        failure: None,
    };
    let current = match policy {
        Policy::Agreement(c) => (c.start..=c.through)
            .contains(&s.month)
            .then(|| c.as_ref().clone()),
        Policy::Cooperate(mode) => {
            if let Some(c) = latest(s)
                .and_then(|b| b.active.as_ref())
                .filter(|c| s.month <= c.through)
            {
                Some(c.clone())
            } else {
                let result = discover(w, s, *mode, &mut b)?;
                b.event = if result.is_some() {
                    "Accepted"
                } else {
                    "Declined"
                }
                .into();
                result
            }
        }
        _ => unreachable!(),
    };
    let mut round = Round {
        cooperation: None,
        planning: None,
        month: s.month,
        orders: vec![],
        order_receipts: vec![],
        attempts: vec![],
        transactions: vec![],
        markets: town_market::listings(w.town_market.as_ref().unwrap())
            .iter()
            .map(|m| (m.market, MarketResult::default()))
            .collect(),
    };
    if let Some(c) = current {
        b.agreement = Some(c.start);
        match settle(w, s, &c) {
            Ok(transactions) => {
                round.transactions = transactions;
                b.completed = c
                    .deliveries
                    .iter()
                    .filter(|d| d.month == s.month)
                    .cloned()
                    .collect();
                for d in &b.completed {
                    let m = round.markets.get_mut(&d.market).unwrap();
                    m.volume += d.goods.amount.quantity;
                    m.posted_price = Some(d.payment.amount.quantity);
                }
                if s.month == c.through {
                    b.event = "Completed".into();
                }
                b.active = Some(c);
            }
            Err(reason) => {
                b.event = "Failed".into();
                b.failure = Some(reason);
            }
        }
    }
    round.cooperation = Some(Box::new(b));
    Ok(Some(round))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conditional_assessment_does_not_read_counterparty_work_choice() {
        let (mut w, s) = crate::calibration::scenario(true);
        w.production_market.as_mut().unwrap().policy = Policy::Cooperate(Discovery::Posted);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.step().unwrap();
        let ids = people(&sim.world);
        let mut c = contract(
            &sim.state,
            BTreeMap::from([
                (
                    ids[0],
                    Choice {
                        work: Work::Produce(crate::scenario::GROW),
                        buy: Purchases::None,
                    },
                ),
                (
                    ids[1],
                    Choice {
                        work: Work::Wait,
                        buy: Purchases::None,
                    },
                ),
            ]),
            terms(
                &sim.world,
                &sim.state,
                ids[0],
                crate::negotiation::GRAIN_MARKET,
            )
            .unwrap(),
        )
        .unwrap();
        let before = sim.state.clone();
        let wait = project(&sim.world, &sim.state, &c, Some(ids[0])).unwrap();
        c.choices.get_mut(&ids[1]).unwrap().work = Work::Produce(crate::scenario::PREPARE_FUEL);
        let mut hidden_world = sim.world.clone();
        let mut hidden_state = sim.state.clone();
        hidden_world
            .participants
            .iter_mut()
            .find(|p| p.agent == ids[1])
            .unwrap()
            .capacity
            .quantity = 100;
        hidden_state
            .balances
            .insert((ids[1], crate::scenario::GRAIN), 1000);
        hidden_state
            .balances
            .insert((ids[1], crate::scenario::TOKEN), 0);
        let work = project(&hidden_world, &hidden_state, &c, Some(ids[0])).unwrap();
        assert_eq!(wait.scores[&ids[0]], work.scores[&ids[0]]);
        assert_eq!(wait.coins[&ids[0]], work.coins[&ids[0]]);
        assert_eq!(sim.state, before);
    }
}
