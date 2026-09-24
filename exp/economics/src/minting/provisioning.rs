//! Bounded purchasing-power policy. Expectations are not future food reservations.
use super::*;
use crate::marketplace::Side;

const MAX_HORIZON_MONTHS: u32 = 12;
const MAX_MONTHLY_LOTS: i32 = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub consumption: DefinitionId,
    pub horizon_months: u32,
    pub leisure: DefinitionId,
    /// Membership in this map selects actors; None means income comes from labor sales.
    pub earning: BTreeMap<AgentId, Option<DefinitionId>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    Covered,
    SeekIncome,
    NoFoodAccess,
    AwaitFood,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub agent: AgentId,
    pub food_required: i32,
    pub food_held: i32,
    pub expected_food_access: i32,
    pub cash_gap: i32,
    pub choice: Choice,
}

pub fn validate(w: &World, c: &Config, p: &orders::Policy, v: &Policy) -> Result<(), String> {
    let consumer = w
        .definitions
        .iter()
        .find(|d| d.id == v.consumption)
        .ok_or("missing provision consumer")?;
    let leisure = w
        .definitions
        .iter()
        .find(|d| d.id == v.leisure)
        .ok_or("missing leisure process")?;
    let venue = marketplace::venue(w, c.venue).ok_or("missing venue")?;
    let sale = venue
        .markets
        .iter()
        .find(|m| m.id == p.sale_market)
        .ok_or("missing food listing")?;
    if v.horizon_months == 0
        || v.horizon_months > MAX_HORIZON_MONTHS
        || v.earning.is_empty()
        || consumer.execution != Execution::Consumption
        || consumer.stages.len() != 1
        || consumer.outputs.len() != 1
        || consumer.stages[0].entry_inputs.len() != 1
        || consumer.stages[0].entry_inputs[0] != sale.goods
        || consumer.outputs[0].quantity <= 0
        || sale.goods.quantity <= 0
        || leisure.execution != Execution::Productive
        || leisure.duration() != 1
        || leisure.stages.len() != 1
        || !leisure.stages[0].entry_inputs.is_empty()
        || leisure.stages[0].monthly_services.is_empty()
        || leisure
            .outputs
            .iter()
            .any(|a| a.resource == c.coin || a.resource == sale.goods.resource)
    {
        return Err("invalid provision or leisure recipe".into());
    }
    for (&agent, earning) in &v.earning {
        let actor = w
            .participants
            .iter()
            .find(|a| a.agent == agent)
            .ok_or("missing provision participant")?;
        let need = actor
            .needs
            .iter()
            .find(|n| n.resource == consumer.outputs[0].resource)
            .ok_or("missing recurring need")?;
        if agent == c.issuer
            || need.quantity <= 0
            || need.quantity % consumer.outputs[0].quantity != 0
            || need.quantity / consumer.outputs[0].quantity > MAX_MONTHLY_LOTS
            || !p
                .quotes
                .iter()
                .any(|q| q.agent == agent && q.market == p.sale_market && q.side == Side::Buy)
            || earning.is_some_and(|id| {
                id == v.leisure
                    || !w
                        .definitions
                        .iter()
                        .any(|d| d.id == id && d.execution == Execution::Productive)
            })
        {
            return Err("invalid provision participant policy".into());
        }
        let lots =
            i64::from(need.quantity / consumer.outputs[0].quantity) * i64::from(v.horizon_months);
        if lots * i64::from(p.sale_limit) > i64::from(i32::MAX)
            || lots * i64::from(sale.goods.quantity) > i64::from(i32::MAX)
        {
            return Err("provision horizon overflow".into());
        }
    }
    Ok(())
}
fn monthly(w: &World, v: &Policy, agent: AgentId) -> i32 {
    let d = w.definition(v.consumption);
    let need = w
        .participants
        .iter()
        .find(|a| a.agent == agent)
        .unwrap()
        .needs
        .iter()
        .find(|n| n.resource == d.outputs[0].resource)
        .unwrap();
    need.quantity / d.outputs[0].quantity * d.stages[0].entry_inputs[0].quantity
}

pub fn decision(
    w: &World,
    s: &State,
    c: &Config,
    p: &orders::Policy,
    v: &Policy,
    agent: AgentId,
) -> Decision {
    let d = w.definition(v.consumption);
    let food = d.stages[0].entry_inputs[0].resource;
    let lot = d.stages[0].entry_inputs[0].quantity;
    let required = monthly(w, v, agent) * v.horizon_months as i32;
    let held = s.balance(agent, food);
    // Conservative equal-share expectation among configured, admitted actors.
    // Actual matching still uses price then agent ID and never reserves future grain.
    let admitted = marketplace::eligible(w, s, c.venue, agent)
        && marketplace::eligible(w, s, c.venue, c.issuer)
        && opportunities::permits(w, s, agent, Action::Process(v.consumption))
        && d.enabled
        && p.quotes.iter().any(|q| {
            q.agent == agent
                && q.market == p.sale_market
                && q.side == Side::Buy
                && q.limit >= p.sale_limit
        });
    let buyers = v
        .earning
        .keys()
        .filter(|a| marketplace::eligible(w, s, c.venue, **a))
        .count()
        .max(1) as i32;
    let access = if admitted {
        s.balance(c.issuer, food) / lot / buyers * lot
    } else {
        0
    };
    let shortage = (required - held).max(0);
    let lots = (i64::from(shortage) + i64::from(lot) - 1) / i64::from(lot);
    let cost = (lots * i64::from(p.sale_limit)) as i32;
    let gap = (cost - s.balance(agent, c.coin)).max(0);
    let usable = d.enabled && opportunities::permits(w, s, agent, Action::Process(v.consumption));
    let choice = if !usable {
        Choice::NoFoodAccess
    } else if held < monthly(w, v, agent) && access >= shortage && gap == 0 {
        Choice::AwaitFood
    } else if shortage == 0 || (access >= shortage && gap == 0) {
        Choice::Covered
    } else if access < shortage {
        Choice::NoFoodAccess
    } else {
        Choice::SeekIncome
    };
    Decision {
        agent,
        food_required: required,
        food_held: held,
        expected_food_access: access,
        cash_gap: gap,
        choice,
    }
}

pub fn generate(
    w: &World,
    s: &State,
    c: &Config,
    p: &orders::Policy,
    v: &Policy,
) -> Result<orders::Plan, String> {
    // Retain recipe-derived bids and opening funding checks, but make food sales
    // independent of mint authority, its funding gap, and remaining mint dates.
    let mut plan = orders::generate_fixed(w, s, c, p)?;
    let bids: Vec<_> = plan
        .orders
        .iter()
        .filter(|o| o.agent == c.issuer && o.side == Side::Buy)
        .cloned()
        .collect();
    plan.orders.clear();
    plan.deals.clear();
    let venue = marketplace::venue(w, c.venue).ok_or("missing venue")?;
    let sale = venue
        .markets
        .iter()
        .find(|m| m.id == p.sale_market)
        .ok_or("missing food listing")?;
    if marketplace::eligible(w, s, c.venue, c.issuer) {
        plan.orders.push(orders::Order {
            agent: c.issuer,
            market: p.sale_market,
            side: Side::Sell,
            limit: p.sale_limit,
            lots: (s.balance(c.issuer, sale.goods.resource) / sale.goods.quantity)
                .min(MAX_MONTHLY_LOTS),
        });
    }
    for q in p
        .quotes
        .iter()
        .filter(|q| q.side == Side::Buy && v.earning.contains_key(&q.agent))
    {
        if !marketplace::eligible(w, s, c.venue, q.agent) {
            continue;
        }
        let qty = (monthly(w, v, q.agent) - s.balance(q.agent, sale.goods.resource)).max(0);
        let lots = ((i64::from(qty) + i64::from(sale.goods.quantity) - 1)
            / i64::from(sale.goods.quantity)) as i32;
        if lots > 0 {
            plan.orders.push(orders::Order {
                agent: q.agent,
                market: q.market,
                side: Side::Buy,
                limit: q.limit,
                lots,
            });
        }
    }
    orders::clear(w, s, c, &mut plan)?;
    let mut after = s.clone();
    for deal in &plan.deals {
        for e in transaction(w, s, c, deal)?.effects {
            *after.balances.entry(e.account).or_default() += e.delta;
        }
    }
    plan.provision = v
        .earning
        .keys()
        .map(|a| decision(w, &after, c, p, v, *a))
        .collect();
    for q in p.quotes.iter().filter(|q| q.side == Side::Sell) {
        let Some(choice) = plan.provision.iter().find(|d| d.agent == q.agent) else {
            continue;
        };
        if choice.choice != Choice::SeekIncome {
            continue;
        }
        let market = venue
            .markets
            .iter()
            .find(|m| m.id == q.market)
            .ok_or("missing input market")?;
        let lots = ((s.balance(q.agent, market.goods.resource) - q.holding).max(0)
            / market.goods.quantity)
            .min(1);
        let price = i64::from(q.limit.max(choice.cash_gap));
        let tick = i64::from(market.price_tick);
        let limit = i32::try_from((price + tick - 1) / tick * tick)
            .map_err(|_| "provision ask overflow")?;
        if lots > 0 {
            plan.orders.push(orders::Order {
                agent: q.agent,
                market: q.market,
                side: Side::Sell,
                limit,
                lots,
            });
        }
    }
    plan.orders.extend(bids);
    orders::clear(w, s, c, &mut plan)?;
    Ok(plan)
}

/// Scope the experiment's work selection without embedding food rules in the
/// generic activity catalog or resolver. The chosen process still spends hours.
pub fn activity_allowed(w: &World, s: &State, agent: AgentId, definition: DefinitionId) -> bool {
    let Some(c) = &w.minting else {
        return true;
    };
    let Some(p) = &c.order_policy else {
        return true;
    };
    let Some(v) = &p.provisioning else {
        return true;
    };
    let Some(earning) = v.earning.get(&agent) else {
        return true;
    };
    let choice = decision(w, s, c, p, v, agent).choice;
    if definition == v.leisure {
        return choice == Choice::Covered;
    }
    if Some(definition) == *earning {
        return choice == Choice::SeekIncome;
    }
    true
}
