use super::*;
use crate::marketplace::Side;

fn stocking_target(need: i32, output: i32, input: i32, months: u32) -> Result<i32, String> {
    let batches = (i128::from(need) + i128::from(output) - 1) / i128::from(output);
    i32::try_from(batches * i128::from(input) * i128::from(months))
        .map_err(|_| "food target overflow".into())
}

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
                    .filter(|d| opportunities::permits(w, s, person, Action::Process(d.id)))
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
                        .checked_add(stocking_target(
                            need.quantity,
                            output.quantity,
                            input,
                            c.horizon,
                        )?)
                        .ok_or("food target overflow")?;
                }
            }
        }
        let sell = c.private_sales && {
            let floors = if w.participants.iter().any(|p| p.agent == person) {
                crate::need_orders::protected_stock(w, s, person, c.horizon)?
            } else {
                crate::need_orders::claims(w, s, person, c.horizon)?
            };
            let floor = floors.get(&sale.goods.resource).copied().unwrap_or(0);
            i128::from(s.balance(person, sale.goods.resource)) - floor
                >= i128::from(sale.goods.quantity)
        };
        if target > 0 || sell {
            quotes.push(crate::minting::orders::Quote {
                max_lots: None,
                agent: person,
                market: sale.id,
                side: if sell { Side::Sell } else { Side::Buy },
                limit: policy.sale_limit,
                holding: if sell { 0 } else { target },
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
    let public_sale = w
        .agency
        .get(&mint.issuer)
        .filter(|_| c.public_sales)
        .map(|controller| {
            use agency::objectives::{Metric, Scope};
            let reserve = controller
                .config
                .objectives
                .iter()
                .filter_map(|o| match o.metric {
                    Metric::Reserve { resource, target }
                        if o.scope == Scope::Organization && resource == sale.goods.resource =>
                    {
                        Some(target)
                    }
                    _ => None,
                })
                .max()
                .unwrap_or(0);
            crate::minting::orders::StockSales {
                reserve,
                claim_months: c.horizon,
            }
        });
    w.discovery.as_mut().unwrap().supply.extend(decisions);
    let policy = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    policy.quotes = quotes;
    policy.public_sale = public_sale;
    policy.private_sales = c.private_sales.then_some(c.horizon);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::stocking_target;

    #[test]
    fn large_batches_round_without_overflowing_the_intermediate_sum() {
        assert_eq!(stocking_target(i32::MAX, i32::MAX, 1, 4).unwrap(), 4);
        assert_eq!(stocking_target(i32::MAX, i32::MAX - 1, 1, 4).unwrap(), 8);
        assert_eq!(stocking_target(5, 2, 3, 4).unwrap(), 36);
        assert!(stocking_target(i32::MAX, 1, 1, 4).is_err());
    }
}
