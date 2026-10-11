//! Static, bounded quote portfolios. No holdings or selected peer policies enter
//! this catalog, so a later observation cannot rewrite a historical alternative.
use super::*;

pub fn portfolios(w: &World, agent: AgentId) -> Result<Vec<Vec<Quote>>, String> {
    let Some(a) = w.agency.get(&agent).and_then(|c| c.operating_agreement()) else {
        return Ok(vec![]);
    };
    if a.stock_markets.is_empty() {
        return Ok(vec![]);
    }
    let mut portfolios = vec![vec![]];
    for (&market, options) in &a.stock_markets {
        let variants = options
            .limits
            .len()
            .checked_mul(options.holdings.len())
            .and_then(|n| n.checked_add(1))
            .ok_or("stock portfolio count overflow")?;
        if portfolios
            .len()
            .checked_mul(variants)
            .is_none_or(|n| n > crate::agency::MAX_PROGRAMS)
        {
            return Err("stock portfolios exceed bounded search".into());
        }
        let mut next = vec![];
        for portfolio in portfolios {
            // An omitted market is a deliberate stop, not an unbounded quote.
            next.push(portfolio.clone());
            for &limit in &options.limits {
                for &holding in &options.holdings {
                    let mut candidate = portfolio.clone();
                    candidate.push(Quote {
                        agent,
                        market,
                        side: options.side,
                        limit,
                        holding,
                        max_lots: Some(options.max_lots),
                    });
                    next.push(candidate);
                }
            }
        }
        portfolios = next;
    }
    Ok(portfolios)
}
