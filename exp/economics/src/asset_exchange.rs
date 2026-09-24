//! Shared funded asset transfer for collateral resale and estate liquidation.
use crate::{credit, finance, model::*};

pub(crate) struct AcceptedSale {
    pub sale: credit::Sale,
    pub buyer: AgentId,
    /// Payment recipients in sale-denomination units; must sum to the price.
    pub payees: Vec<(AgentId, i32)>,
}

pub(crate) fn settle(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    execution: &mut finance::Execution,
    accepted: &AcceptedSale,
) -> Result<(), String> {
    let sale = &accepted.sale;
    let owner = out.after.owners.get(&sale.asset).copied().or_else(|| {
        world
            .assets
            .iter()
            .find(|a| a.id == sale.asset)
            .map(|a| a.owner)
    });
    if owner != Some(sale.seller)
        || sale.seller == accepted.buyer
        || sale.price.quantity <= 0
        || state.terminal.contains_key(&accepted.buyer)
        || !crate::opportunities::permits(
            world,
            state,
            accepted.buyer,
            crate::opportunities::Action::AssetTrade,
        )
        || accepted
            .payees
            .iter()
            .any(|(who, quantity)| *quantity < 0 || *who == accepted.buyer)
        || accepted
            .payees
            .iter()
            .map(|(_, quantity)| i64::from(*quantity))
            .sum::<i64>()
            != i64::from(sale.price.quantity)
    {
        return Err("invalid accepted asset sale".into());
    }
    let legs: Vec<_> = accepted
        .payees
        .iter()
        .filter(|(_, quantity)| *quantity > 0)
        .map(|(who, quantity)| finance::Transfer {
            from: accepted.buyer,
            to: *who,
            amount: Amount::new(sale.price.resource, *quantity),
        })
        .collect();
    let effects = execution.exchange(world, &legs)?;
    out.transactions.push(credit::tx(
        format!("asset {} funded sale", sale.asset),
        effects,
    ));
    credit::transfer_attachments(world, state, out, sale.asset, accepted.buyer);
    out.after.owners.insert(sale.asset, accepted.buyer);
    out.after.values.insert(sale.asset, sale.price.quantity);
    Ok(())
}
