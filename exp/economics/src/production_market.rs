//! Bounded individual work/purchase policies using the ordinary market and process engine.
use crate::{
    compute::Backend, forecast::ForecastContext, model::*, offers, simulation::Simulation,
};
use std::collections::BTreeMap;

pub const WOOD_MARKET: crate::marketplace::MarketId = 2;
const WOOD_LOT: i32 = 1;
const WOOD_PRICE: i32 = 2;
const MAX_HORIZON: u32 = 12;
const MAX_PEOPLE: usize = 4;
const MAX_PRODUCERS: usize = 3;
const OBSERVATION_MONTHS: u32 = 6;
const BUFFER_MONTHS: i128 = 2;
const EXAMPLE_HORIZON: u32 = 6;
const EXAMPLE_COINS: i32 = 24;
const EXAMPLE_FOOD: i32 = 4;
const EXAMPLE_FUEL: i32 = 2;
const EXAMPLE_CAPACITY: i32 = 2;
const EXAMPLE_STORAGE: i32 = 32;
const EXAMPLE_PRICE: i32 = 4;
const CROP_SKILL: u32 = 101;
const WOOD_SKILL: u32 = 102;
const RIGHT_THROUGH: u32 = 240;
const EXPERTISE: u32 = 100;
const PLANT_MONTHS: u32 = 1;
const TEND_MONTHS: u32 = 2;
const SEED_LOT: i32 = 1;
const CROP_YIELD: i32 = 4;
const BASIC_LABOR: i32 = 2;
const SKILLED_LABOR: i32 = 1;
const CROP_MULTIPLIER: u32 = 2;
const MONTHLY_NEED: i32 = 1;
const PRACTICE_PER_COMPLETION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Work {
    Ordinary,
    Wait,
    Produce(DefinitionId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purchases {
    None,
    All,
    Market(crate::marketplace::MarketId),
}
impl Purchases {
    pub fn allows(self, market: crate::marketplace::MarketId) -> bool {
        self == Self::All || self == Self::Market(market)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub work: Work,
    pub buy: Purchases,
}
impl Default for Choice {
    fn default() -> Self {
        Self {
            work: Work::Ordinary,
            buy: Purchases::All,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Policy {
    Plan,
    Fixed(BTreeMap<AgentId, Choice>),
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CounterpartyExpectation {
    #[default]
    Ordinary,
    /// Experimental public prior-plan signal; never read this month's decisions.
    LastPublishedPlan,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Persistence {
    #[default]
    Monthly,
    Hold {
        months: u32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionReason {
    Replanned,
    Retained,
    SafetyOverride,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedChoice {
    pub month: u32,
    pub choice: Choice,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub counterparties: CounterpartyExpectation,
    pub persistence: Persistence,
    pub demand: DemandSignal,
    pub horizon: u32,
    pub trading: bool,
    pub policy: Policy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemandSignal {
    CompletedOnly,
    IncludeUnfilledBids,
}
impl DemandSignal {
    fn limit(self, b: &Belief) -> u32 {
        match self {
            Self::CompletedOnly => b.lots_per_month,
            Self::IncludeUnfilledBids => b.lots_per_month.max(b.interested_lots),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Belief {
    pub through: u32,
    pub price: Option<i32>,
    /// Largest completed monthly volume in the observation window, in whole lots.
    pub lots_per_month: u32,
    /// Largest unfilled bid volume, an interest signal rather than a sale promise.
    pub interested_lots: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Forecast {
    pub choice: Choice,
    pub deficits: BTreeMap<ResourceId, i64>,
    pub terminal: bool,
    pub failures: usize,
    pub buffer_gap: i128,
    pub closing_coins: i32,
    pub sales: BTreeMap<crate::marketplace::MarketId, i32>,
    pub purchases: BTreeMap<crate::marketplace::MarketId, i32>,
    pub labor: i64,
    pub stock_value: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonDecision {
    pub selection_reason: SelectionReason,
    pub retain_through: u32,
    pub agent: AgentId,
    pub alternatives: Vec<Forecast>,
    pub selected: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub counterparties: BTreeMap<AgentId, ObservedChoice>,
    pub month: u32,
    pub through: u32,
    pub belief: BTreeMap<crate::marketplace::MarketId, Belief>,
    pub people: Vec<PersonDecision>,
}

pub fn validate(w: &World) -> Result<(), String> {
    let Some(c) = &w.production_market else {
        return Ok(());
    };
    let producers: Vec<_> = w
        .definitions
        .iter()
        .filter(|d| d.enabled && d.execution == Execution::Productive)
        .collect();
    if matches!(c.persistence, Persistence::Hold { months } if months == 0 || months > MAX_HORIZON)
    {
        return Err("persistence duration outside planning bounds".into());
    }
    if w.town_market.as_ref().is_none_or(|m| !m.adaptive)
        || w.participants.is_empty()
        || w.participants.len() > MAX_PEOPLE
        || producers.len() > MAX_PRODUCERS
        || w.participants.iter().any(|p| {
            w.town_market
                .as_ref()
                .is_some_and(|m| !m.traders.iter().any(|t| t.trader.agent == p.agent))
        })
        || c.horizon == 0
        || c.horizon > MAX_HORIZON
        || producers.iter().any(|d| d.duration() > c.horizon)
        || w.priority != Priority::ContinuingFirst
        || w.decision_horizon.is_some()
        || w.activities != Default::default()
        || !w.pools.is_empty()
    {
        return Err(
            "production market requires bounded independent work and an adaptive town book".into(),
        );
    }
    if let Policy::Fixed(choices) = &c.policy {
        for (agent, choice) in choices {
            if !w.participants.iter().any(|p| p.agent == *agent)
                || matches!(choice.work,Work::Produce(id) if !producers.iter().any(|d|d.id==id))
                || matches!(choice.buy, Purchases::Market(id) if !crate::town_market::listings(w.town_market.as_ref().unwrap()).iter().any(|m|m.market==id))
            {
                return Err("invalid fixed production choice".into());
            }
        }
    }
    Ok(())
}
pub(crate) fn validate_state(w: &World, s: &State) -> Result<(), String> {
    for r in &s.town_market.history {
        if let Some(d) = &r.planning {
            let mut ids = std::collections::BTreeSet::new();
            if d.counterparties.values().any(|o| o.month >= d.month)
                || d.people.iter().any(|p| p.retain_through < d.month)
                || d.month != r.month
                || d.through < d.month
                || d.belief.values().any(|b| b.through >= d.month)
                || d.people.iter().any(|p| {
                    !ids.insert(p.agent)
                        || !w.participants.iter().any(|a| a.agent == p.agent)
                        || p.selected >= p.alternatives.len()
                        || p.alternatives.len() > 4 * (MAX_PRODUCERS + 2)
                        || p.alternatives.iter().any(|a| {
                            matches!(a.choice.work,
                            Work::Produce(id) if !w.definitions.iter().any(|d|
                                d.id == id && d.execution == Execution::Productive))
                        })
                })
            {
                return Err("invalid production decision boundary".into());
            }
        }
    }
    if w.production_market
        .as_ref()
        .is_some_and(|c| matches!(c.policy, Policy::Plan))
        && !matches!(s.phase, Phase::Open | Phase::Acquire)
        && s.town_market
            .history
            .iter()
            .find(|r| r.month == s.month)
            .and_then(|r| r.planning.as_ref())
            .is_none()
    {
        return Err("missing accepted production decision".into());
    }
    Ok(())
}
pub fn choices(w: &World, d: Option<&Decision>) -> BTreeMap<AgentId, Choice> {
    match w.production_market.as_ref().map(|c| &c.policy) {
        Some(Policy::Fixed(p)) => p.clone(),
        _ => d
            .map(|d| {
                d.people
                    .iter()
                    .map(|p| (p.agent, p.alternatives[p.selected].choice))
                    .collect()
            })
            .unwrap_or_default(),
    }
}
pub fn belief(w: &World, s: &State) -> BTreeMap<crate::marketplace::MarketId, Belief> {
    crate::town_market::listings(w.town_market.as_ref().unwrap())
        .iter()
        .map(|m| {
            let lot = crate::marketplace::venue(w, m.venue)
                .unwrap()
                .markets
                .iter()
                .find(|a| a.id == m.market)
                .unwrap()
                .goods
                .quantity;
            let rows: Vec<_> = s
                .town_market
                .history
                .iter()
                .filter(|r| {
                    r.month < s.month && r.month >= s.month.saturating_sub(OBSERVATION_MONTHS)
                })
                .filter_map(|r| r.markets.get(&m.market))
                .collect();
            (
                m.market,
                Belief {
                    through: s.month.saturating_sub(1),
                    price: rows.iter().rev().find_map(|r| r.posted_price),
                    lots_per_month: rows
                        .iter()
                        .map(|r| (r.volume / lot) as u32)
                        .max()
                        .unwrap_or(0),
                    interested_lots: rows
                        .iter()
                        .map(|r| (r.unfilled_buy / lot) as u32)
                        .max()
                        .unwrap_or(0),
                },
            )
        })
        .collect()
}

pub(crate) fn work(sim: &Simulation) -> Result<Batch, String> {
    let decision = sim
        .state
        .town_market
        .history
        .iter()
        .find(|r| r.month == sim.state.month)
        .and_then(|r| r.planning.as_ref());
    let policies = choices(&sim.world, decision);
    let mut b = Batch::empty(&sim.state);
    let ordinary = sim.productive_requests(&mut b, false, None)?;
    b.receipts.clear();
    let mut ids: Vec<_> = sim.world.participants.iter().map(|p| p.agent).collect();
    ids.sort();
    let mut requests = vec![];
    for agent in ids {
        let choice = policies.get(&agent).copied().unwrap_or_default();
        for r in ordinary
            .iter()
            .filter(|r| r.agent == agent && r.existing.is_some())
        {
            requests.push(offers::Request {
                offer: offers::Id::Process(r.definition),
                agent,
                continuing: r.existing,
                need: r.need,
            });
        }
        if let Work::Produce(id) = choice.work
            && !sim.state.terminal.contains_key(&agent)
            && !sim
                .state
                .processes
                .values()
                .any(|p| p.operator == agent && p.definition == id && p.status == Status::Active)
        {
            requests.push(offers::Request::new(offers::Id::Process(id), agent));
        }
        if choice.work != Work::Wait {
            for r in ordinary
                .iter()
                .filter(|r| r.agent == agent && r.existing.is_none())
            {
                if matches!(choice.work,Work::Produce(id) if id==r.definition) {
                    continue;
                }
                requests.push(offers::Request {
                    offer: offers::Id::Process(r.definition),
                    agent,
                    continuing: None,
                    need: r.need,
                });
            }
        }
    }
    offers::resolve(sim, &requests, &mut b)?;
    Ok(b)
}
fn forecast(
    w: &World,
    s: &State,
    agent: AgentId,
    choice: Choice,
    b: &BTreeMap<crate::marketplace::MarketId, Belief>,
) -> Result<Forecast, String> {
    let c = w.production_market.as_ref().unwrap();
    let mut policies: BTreeMap<_, _> = observed_choices(w, s)
        .into_iter()
        .map(|(agent, observed)| (agent, observed.choice))
        .collect();
    policies.insert(agent, choice);
    let (mut world, mut state) = ForecastContext::new(w, s).into_parts();
    // Historical alternatives are diagnostics, not inputs to a fixed-policy branch.
    // Keep every market observation and accepted process, but avoid copying these
    // large transcripts at every hypothetical settlement boundary.
    for r in &mut state.town_market.history {
        r.planning = None;
    }
    world.production_market.as_mut().unwrap().policy = Policy::Fixed(policies);
    let mut sim = Simulation::new(world, state, Backend::Reference)?;
    let end = s
        .month
        .checked_add(c.horizon)
        .ok_or("market forecast overflow")?;
    // Counterparty choices are dated hypotheses, not observations of future decisions.
    while sim.state.month < end {
        if sim.state.phase == Phase::Acquire && sim.state.month > s.month {
            let m = sim.world.town_market.as_mut().unwrap();
            m.match_limit = Some(c.demand.limit(&b[&m.market]));
            for l in &mut m.additional {
                l.match_limit = Some(c.demand.limit(&b[&l.market]));
            }
        }
        sim.step()?;
    }
    let mut deficits = BTreeMap::new();
    for report in sim.reports.iter().filter(|r| r.agent == agent) {
        crate::forecast::needs::accumulate(
            &mut deficits,
            report.needs.iter().map(|(r, n)| (*r, n.deficit)),
        );
    }
    let labor = sim
        .ledger
        .iter()
        .flat_map(|b| &b.transactions)
        .flat_map(|t| &t.effects)
        .filter(|e| {
            e.account.0 == agent
                && e.delta < 0
                && w.resources
                    .iter()
                    .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Capacity)
        })
        .map(|e| -i64::from(e.delta))
        .sum();
    let failures = sim
        .ledger
        .iter()
        .flat_map(|b| &b.transactions)
        .filter_map(|t| t.process.as_ref())
        .filter(|p| p.after.operator == agent && p.after.status == Status::Aborted)
        .count();
    let p = w.participants.iter().find(|p| p.agent == agent).unwrap();
    let mut stocks = crate::substitution::stocks(&sim.state, agent);
    let mut gap = 0;
    for n in crate::forecast::needs::ordered(&p.needs) {
        let recipes = crate::substitution::recipes_for(w, &sim.state, agent, n.resource);
        gap += crate::substitution::allocate_recipes(
            &recipes,
            i128::from(n.quantity) * BUFFER_MONTHS,
            &mut stocks,
            &BTreeMap::new(),
        )
        .1;
    }
    let m = w.town_market.as_ref().unwrap();
    let terms = crate::marketplace::venue(w, m.venue)
        .unwrap()
        .markets
        .iter()
        .find(|a| a.id == m.market)
        .unwrap();
    let mut sales = BTreeMap::new();
    let mut purchases = BTreeMap::new();
    for r in sim
        .state
        .town_market
        .history
        .iter()
        .filter(|r| r.month >= s.month)
    {
        for a in &r.attempts {
            if matches!(a.round.outcome, crate::negotiation::Outcome::Traded { .. }) {
                if a.session.seller.agent == agent {
                    *sales.entry(a.session.market).or_default() += a.session.goods.quantity;
                }
                if a.session.buyer.agent == agent {
                    *purchases.entry(a.session.market).or_default() += a.session.goods.quantity;
                }
            }
        }
    }
    // One surplus lot per distinct listed good; price observations never become cash.
    let stock_value = crate::town_market::listings(m)
        .iter()
        .map(|listing| {
            let terms = crate::marketplace::venue(w, listing.venue)
                .unwrap()
                .markets
                .iter()
                .find(|a| a.id == listing.market)
                .unwrap();
            let surplus = stocks
                .get(&terms.goods.resource)
                .copied()
                .unwrap_or(0)
                .max(0)
                .min(i128::from(terms.goods.quantity));
            let belief = &b[&listing.market];
            if belief.lots_per_month > 0 {
                (surplus * i128::from(belief.price.unwrap_or(0)) / i128::from(terms.goods.quantity))
                    as i64
            } else {
                0
            }
        })
        .sum();
    Ok(Forecast {
        choice,
        deficits,
        terminal: sim.state.terminal.contains_key(&agent),
        failures,
        buffer_gap: gap,
        closing_coins: sim.state.balance(agent, terms.payment),
        sales,
        purchases,
        labor,
        stock_value,
    })
}
fn prior_decision(s: &State) -> Option<&Decision> {
    s.town_market
        .history
        .iter()
        .rev()
        .filter(|r| r.month < s.month)
        .find_map(|r| r.planning.as_ref())
}
fn observed_choices(w: &World, s: &State) -> BTreeMap<AgentId, ObservedChoice> {
    if w.production_market
        .as_ref()
        .is_none_or(|c| c.counterparties == CounterpartyExpectation::Ordinary)
    {
        return BTreeMap::new();
    }
    prior_decision(s)
        .map(|d| {
            d.people
                .iter()
                .map(|p| {
                    (
                        p.agent,
                        ObservedChoice {
                            month: d.month,
                            choice: p.alternatives[p.selected].choice,
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}
fn safety_score(p: &Participant, f: &Forecast) -> (bool, Vec<i64>, usize) {
    (
        f.terminal,
        crate::forecast::needs::score(&p.needs, &f.deficits),
        f.failures,
    )
}
fn persistent_selection(
    policy: Persistence,
    month: u32,
    p: &Participant,
    previous: Option<&PersonDecision>,
    alternatives: &[Forecast],
    selected: usize,
) -> Result<(usize, SelectionReason, u32), String> {
    Ok(match policy {
        Persistence::Monthly => (selected, SelectionReason::Replanned, month),
        Persistence::Hold { months } => {
            let next = month
                .checked_add(months - 1)
                .ok_or("persistence date overflow")?;
            if let Some(previous) = previous
                && previous.retain_through >= month
                && let Some(held) = alternatives
                    .iter()
                    .position(|f| f.choice == previous.alternatives[previous.selected].choice)
            {
                if safety_score(p, &alternatives[selected]) < safety_score(p, &alternatives[held]) {
                    (selected, SelectionReason::SafetyOverride, next)
                } else {
                    (held, SelectionReason::Retained, previous.retain_through)
                }
            } else {
                (selected, SelectionReason::Replanned, next)
            }
        }
    })
}

pub fn choose(w: &World, s: &State) -> Result<Option<Decision>, String> {
    let Some(c) = &w.production_market else {
        return Ok(None);
    };
    if matches!(c.policy, Policy::Fixed(_)) {
        return Ok(None);
    }
    validate(w)?;
    let belief = belief(w, s);
    let mut works = vec![Work::Ordinary, Work::Wait];
    let mut definitions: Vec<_> = w
        .definitions
        .iter()
        .filter(|d| d.enabled && d.execution == Execution::Productive)
        .map(|d| d.id)
        .collect();
    definitions.sort();
    works.extend(definitions.into_iter().map(Work::Produce));
    let mut participants: Vec<_> = w
        .participants
        .iter()
        .filter(|p| !s.terminal.contains_key(&p.agent))
        .collect();
    participants.sort_by_key(|p| p.agent);
    let mut buying = vec![Purchases::None, Purchases::All];
    let listings = crate::town_market::listings(w.town_market.as_ref().unwrap());
    if listings.len() > 1 {
        let mut ids: Vec<_> = listings.iter().map(|l| l.market).collect();
        ids.sort();
        buying.extend(ids.into_iter().map(Purchases::Market));
    }
    let mut people = vec![];
    for p in participants {
        let mut alternatives = vec![];
        for &work in &works {
            for &buy in &buying {
                alternatives.push(forecast(w, s, p.agent, Choice { work, buy }, &belief)?);
            }
        }
        let selected = (0..alternatives.len())
            .min_by_key(|i| {
                let a = &alternatives[*i];
                (
                    a.terminal,
                    crate::forecast::needs::score(&p.needs, &a.deficits),
                    a.failures,
                    a.buffer_gap,
                    std::cmp::Reverse(i64::from(a.closing_coins) + a.stock_value),
                    a.labor,
                    *i,
                )
            })
            .unwrap();
        let (selected, selection_reason, retain_through) = persistent_selection(
            c.persistence,
            s.month,
            p,
            prior_decision(s).and_then(|d| d.people.iter().find(|a| a.agent == p.agent)),
            &alternatives,
            selected,
        )?;
        people.push(PersonDecision {
            selection_reason,
            retain_through,
            agent: p.agent,
            alternatives,
            selected,
        });
    }
    Ok(Some(Decision {
        counterparties: observed_choices(w, s),
        month: s.month,
        through: s.month + c.horizon - 1,
        belief,
        people,
    }))
}
pub(crate) fn validate_work(w: &World, s: &State, b: &Batch) -> Result<(), String> {
    if w.production_market.is_some() && s.phase == Phase::Productive {
        let sim = Simulation::new(w.clone(), s.clone(), Backend::Reference)?;
        let expected = work(&sim)?;
        if b.transactions != expected.transactions || b.receipts != expected.receipts {
            return Err("altered market work instructions".into());
        }
    }
    Ok(())
}

pub fn scenario(trading: bool) -> (World, State) {
    use crate::{equipment::Technique, scenario::*};
    let (mut w, mut s) = crate::town_market::scenario();
    let (catalog, _) = with_warmth(false);
    w.resources = catalog.resources;
    w.resources.push(Resource {
        id: TOKEN,
        name: "coins".into(),
        kind: ResourceKind::Stock,
    });
    w.definitions = catalog
        .definitions
        .into_iter()
        .filter(|d| d.id != REPAIR)
        .collect();
    let crop = w.definitions.iter_mut().find(|d| d.id == GROW).unwrap();
    crop.stages = vec![
        Stage {
            name: "plant".into(),
            months: PLANT_MONTHS,
            entry_inputs: vec![Amount::new(SEED, SEED_LOT)],
            monthly_services: vec![Amount::new(LABOR, BASIC_LABOR)],
        },
        Stage {
            name: "tend and harvest".into(),
            months: TEND_MONTHS,
            entry_inputs: vec![],
            monthly_services: vec![Amount::new(LABOR, BASIC_LABOR)],
        },
    ];
    crop.outputs = vec![Amount::new(GRAIN, CROP_YIELD), Amount::new(SEED, SEED_LOT)];
    let wood = w
        .definitions
        .iter_mut()
        .find(|d| d.id == PREPARE_FUEL)
        .unwrap();
    wood.name = "collect fuel wood".into();
    wood.stages[0].entry_inputs.clear();
    wood.stages[0].monthly_services = vec![Amount::new(LABOR, BASIC_LABOR)];
    w.assets.clear();
    w.rights.clear();
    s.balances.clear();
    w.storage.weights.insert(SEED, 1);
    w.storage.weights.insert(FUEL, 1);
    for p in &mut w.participants {
        p.capacity.quantity = EXAMPLE_CAPACITY;
        p.needs = vec![
            Requirement {
                resource: NUTRITION,
                quantity: MONTHLY_NEED,
                priority: 0,
            },
            Requirement {
                resource: WARMTH,
                quantity: MONTHLY_NEED,
                priority: 1,
            },
        ];
        for (r, q) in [
            (GRAIN, EXAMPLE_FOOD),
            (FUEL, EXAMPLE_FUEL),
            (SEED, SEED_LOT),
            (TOKEN, EXAMPLE_COINS),
        ] {
            s.balances.insert((p.agent, r), q);
        }
        w.storage.capacities.insert(p.agent, EXAMPLE_STORAGE);
        w.assets.push(Asset {
            id: p.agent,
            owner: STATE_AGENT,
            kind: 1,
        });
        w.rights.push(UseRight {
            id: p.agent,
            holder: p.agent,
            asset: p.agent,
            from: 1,
            through: RIGHT_THROUGH,
            output_owner: p.agent,
        });
        let skill = if [PERSON, 89].contains(&p.agent) {
            CROP_SKILL
        } else {
            WOOD_SKILL
        };
        s.practice.insert((p.agent, skill), EXPERTISE);
    }
    w.practice_rules = vec![
        crate::equipment::PracticeRule {
            definition: GROW,
            stage: 1,
            competency: CROP_SKILL,
            points: PRACTICE_PER_COMPLETION,
        },
        crate::equipment::PracticeRule {
            definition: PREPARE_FUEL,
            stage: 0,
            competency: WOOD_SKILL,
            points: PRACTICE_PER_COMPLETION,
        },
    ];
    for stage in 0..2 {
        w.techniques.push(Technique {
            id: 100 + stage as u32,
            definition: GROW,
            stage,
            equipment_kind: None,
            competency: Some((CROP_SKILL, EXPERTISE)),
            wear: 0,
            services: vec![Amount::new(LABOR, SKILLED_LABOR)],
            output_multiplier: CROP_MULTIPLIER,
        });
    }
    w.techniques.push(Technique {
        id: 102,
        definition: PREPARE_FUEL,
        stage: 0,
        equipment_kind: None,
        competency: Some((WOOD_SKILL, EXPERTISE)),
        wear: 0,
        services: vec![Amount::new(LABOR, SKILLED_LABOR)],
        output_multiplier: 1,
    });
    for d in &w.definitions {
        w.transaction_policy.as_mut().unwrap().permissions.insert((
            crate::opportunities::PERSON_TYPE,
            crate::opportunities::Action::Process(d.id),
        ));
    }
    let m = w.town_market.as_mut().unwrap();
    m.adaptive = true;
    m.order_horizon = crate::town_market::OrderHorizon::Aligned(EXAMPLE_HORIZON);
    for t in &mut m.traders {
        t.trader.limit = EXAMPLE_PRICE;
        t.trader.opening_quote = EXAMPLE_PRICE;
    }
    w.horizon = EXAMPLE_HORIZON;
    w.priority = Priority::ContinuingFirst;
    w.production_market = Some(Config {
        counterparties: CounterpartyExpectation::Ordinary,
        persistence: Persistence::Monthly,
        demand: DemandSignal::CompletedOnly,
        horizon: EXAMPLE_HORIZON,
        trading,
        policy: Policy::Plan,
    });
    (w, s)
}

/// Same people, endowments and technologies; only the wood listing is added.
pub fn reciprocal_scenario(trading: bool) -> (World, State) {
    let (mut w, s) = scenario(trading);
    w.production_market.as_mut().unwrap().demand = DemandSignal::IncludeUnfilledBids;
    let c = w.town_market.as_mut().unwrap();
    let mut traders = c.traders.clone();
    for t in &mut traders {
        t.trader.limit = WOOD_PRICE;
        t.trader.opening_quote = WOOD_PRICE;
    }
    c.additional.push(crate::town_market::Listing {
        market: WOOD_MARKET,
        traders,
        match_limit: None,
    });
    let venue = c.venue;
    w.marketplaces
        .iter_mut()
        .find(|m| m.agent == venue)
        .unwrap()
        .markets
        .push(crate::marketplace::Market {
            id: WOOD_MARKET,
            goods: Amount::new(crate::scenario::FUEL, WOOD_LOT),
            payment: crate::scenario::TOKEN,
            price_tick: 1,
        });
    (w, s)
}

#[cfg(test)]
mod persistence_tests {
    use super::*;
    fn forecast(choice: Choice) -> Forecast {
        Forecast {
            choice,
            deficits: BTreeMap::from([(4, 0)]),
            terminal: false,
            failures: 0,
            buffer_gap: 0,
            closing_coins: 0,
            sales: BTreeMap::new(),
            purchases: BTreeMap::new(),
            labor: 0,
            stock_value: 0,
        }
    }
    #[test]
    fn holds_ignore_wealth_but_expire_and_yield_to_safety() {
        let p = Participant {
            agent: 1,
            capacity: Amount::new(3, 2),
            needs: vec![Requirement {
                resource: 4,
                quantity: 1,
                priority: 0,
            }],
        };
        let held = forecast(Choice {
            work: Work::Wait,
            buy: Purchases::All,
        });
        let mut better = forecast(Choice::default());
        better.closing_coins = 10;
        let previous = PersonDecision {
            agent: 1,
            selected: 0,
            alternatives: vec![held.clone()],
            retain_through: 3,
            selection_reason: SelectionReason::Replanned,
        };
        let mut alternatives = vec![held, better];
        let policy = Persistence::Hold { months: 3 };
        assert_eq!(
            persistent_selection(policy, 2, &p, Some(&previous), &alternatives, 1).unwrap(),
            (0, SelectionReason::Retained, 3)
        );
        assert_eq!(
            persistent_selection(policy, 4, &p, Some(&previous), &alternatives, 1).unwrap(),
            (1, SelectionReason::Replanned, 6)
        );
        alternatives[0].deficits.insert(4, 1);
        assert_eq!(
            persistent_selection(policy, 2, &p, Some(&previous), &alternatives, 1).unwrap(),
            (1, SelectionReason::SafetyOverride, 4)
        );
        alternatives[0].deficits.insert(4, 0);
        alternatives[0].failures = 1;
        assert_eq!(
            persistent_selection(policy, 2, &p, Some(&previous), &alternatives, 1)
                .unwrap()
                .1,
            SelectionReason::SafetyOverride
        );
        alternatives[0].terminal = true;
        assert_eq!(
            persistent_selection(policy, 2, &p, Some(&previous), &alternatives, 1)
                .unwrap()
                .1,
            SelectionReason::SafetyOverride
        );
    }
}
