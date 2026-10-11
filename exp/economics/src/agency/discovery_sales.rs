//! Static production routes from signed stock-sale options. An outlet authorizes
//! an alternative; only a normal forecast can establish a profitable outcome.
use super::*;
use crate::marketplace::Side;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Opportunity {
    pub market: u32,
    pub definition: DefinitionId,
    pub resource: ResourceId,
    pub floor: i32,
    pub lot_quantity: i32,
    pub max_lots: i32,
    pub target: i32,
}

pub fn opportunities(w: &World, agent: AgentId) -> Result<Vec<Opportunity>, String> {
    let c = w
        .agency
        .get(&agent)
        .ok_or("missing organization controller")?;
    let Some(a) = c.operating_agreement() else {
        return Ok(vec![]);
    };
    if a.stock_markets.is_empty() {
        return Ok(vec![]);
    }
    let mint = w.minting.as_ref().ok_or("missing stock market engine")?;
    let venue = crate::marketplace::venue(w, mint.venue).ok_or("missing market venue")?;
    let mut result = vec![];
    for (&market, options) in &a.stock_markets {
        if options.side != Side::Sell {
            continue;
        }
        let m = venue
            .markets
            .iter()
            .find(|m| m.id == market)
            .ok_or("unknown sale outlet")?;
        let wanted = c
            .config
            .objectives
            .iter()
            .chain(c.config.preferences.values().flatten())
            .any(|o| {
                o.scope == objectives::Scope::Organization
                    && match o.metric {
                        objectives::Metric::Reserve { resource, target }
                        | objectives::Metric::ReserveExposure { resource, target } => {
                            resource == m.payment && target > 0
                        }
                        objectives::Metric::FundingGap { resource, .. } => resource == m.payment,
                        _ => false,
                    }
            });
        if !wanted {
            continue;
        }
        for &floor in &options.holdings {
            let target =
                i64::from(floor) + i64::from(options.max_lots) * i64::from(m.goods.quantity);
            let target = i32::try_from(target).map_err(|_| "sale production target overflow")?;
            if target <= 0 {
                return Err("nonpositive sale production target".into());
            }
            for d in &w.definitions {
                if a.processes.contains(&d.id)
                    && d.execution == Execution::Productive
                    && d.outputs
                        .iter()
                        .any(|v| v.resource == m.goods.resource && v.quantity > 0)
                {
                    result.push(Opportunity {
                        market,
                        definition: d.id,
                        resource: m.goods.resource,
                        floor,
                        lot_quantity: m.goods.quantity,
                        max_lots: options.max_lots,
                        target,
                    });
                    if result.len() > MAX_PROGRAMS {
                        return Err("sale production catalog exceeds bounded search".into());
                    }
                }
            }
        }
    }
    result.sort_by_key(|o| (o.market, o.definition, o.resource, o.target));
    Ok(result)
}
