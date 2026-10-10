use super::*;
use crate::marketplace::Side;

/// Generic counterparties come from venue eligibility and useful stocks/capacity.
/// Price limits remain explicit valuation assumptions, not negotiated ZIP prices.
pub(super) fn quotes(w: &mut World, s: &State, c: &Config) -> Result<(), String> {
    let Some(mint) = &w.minting else {
        return Ok(());
    };
    let Some(policy) = &mint.order_policy else {
        return Ok(());
    };
    let venue = crate::marketplace::venue(w, mint.venue).ok_or("missing discovery venue")?;
    let sale = venue
        .markets
        .iter()
        .find(|m| m.id == policy.sale_market)
        .ok_or("missing sale listing")?;
    let mut quotes = vec![];
    for person in people(w, s) {
        if !crate::marketplace::eligible(w, s, mint.venue, person) {
            continue;
        }
        let mut target = 0i32;
        if let Some(actor) = w.participants.iter().find(|p| p.agent == person) {
            for need in &actor.needs {
                if let Some(recipe) = w
                    .definitions
                    .iter()
                    .filter(|d| d.enabled && d.execution == Execution::Consumption)
                    .filter(|d| {
                        d.outputs.iter().any(|a| a.resource == need.resource)
                            && d.stages.iter().any(|s| {
                                s.entry_inputs
                                    .iter()
                                    .any(|a| a.resource == sale.goods.resource)
                            })
                    })
                    .min_by_key(|d| d.id)
                {
                    let output = recipe
                        .outputs
                        .iter()
                        .find(|a| a.resource == need.resource)
                        .unwrap();
                    let input: i32 = recipe
                        .stages
                        .iter()
                        .flat_map(|s| &s.entry_inputs)
                        .filter(|a| a.resource == sale.goods.resource)
                        .map(|a| a.quantity)
                        .sum();
                    target = target
                        .checked_add(
                            ((need.quantity + output.quantity - 1) / output.quantity)
                                .checked_mul(input)
                                .and_then(|q| q.checked_mul(c.horizon as i32))
                                .ok_or("food target overflow")?,
                        )
                        .ok_or("food target overflow")?;
                }
            }
        }
        if target > 0 {
            quotes.push(crate::minting::orders::Quote {
                max_lots: None,
                agent: person,
                market: sale.id,
                side: Side::Buy,
                limit: policy.sale_limit,
                holding: target,
            });
        }
        for (&id, &limit) in &policy.input_limits {
            let market = venue
                .markets
                .iter()
                .find(|m| m.id == id)
                .ok_or("missing input listing")?;
            let capacity = w
                .resources
                .iter()
                .any(|r| r.id == market.goods.resource && r.kind == ResourceKind::Capacity);
            if !capacity && s.balance(person, market.goods.resource) < market.goods.quantity {
                continue;
            }
            quotes.push(crate::minting::orders::Quote {
                max_lots: None,
                agent: person,
                market: id,
                side: Side::Sell,
                limit,
                holding: 0,
            });
        }
    }
    let decisions = super::supply::choose(w, s, c, &mut quotes)?;
    w.discovery.as_mut().unwrap().supply.extend(decisions);
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes = quotes;
    Ok(())
}
