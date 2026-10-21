//! Same finite economy, with explicit working-capital lending consent.
use super::{TOWN, TOWN_TYPE, continuation::Case, financing::Stocks, work_discovery};
use crate::{
    agency::financing::Terms,
    minting::{COIN, ISSUER},
    model::*,
    opportunities::Action,
};
const MAX_ADVANCE: i32 = 8;
const LOAN_MONTHS: u32 = 6;
const FIRST_PAYMENT_DELAY_MONTHS: u32 = 1;
pub fn scenario(case: Case, stocks: Stocks, enabled: bool) -> Result<(World, State), String> {
    let (mut w, s) = work_discovery::with_input_bids(case, stocks, true)?;
    if enabled {
        let terms = Terms {
            denomination: COIN,
            max_principal: MAX_ADVANCE,
            monthly_rate_bps: 0,
            term_months: LOAN_MONTHS,
            grace_months: FIRST_PAYMENT_DELAY_MONTHS,
            programs: [1].into(),
            lend: false,
        };
        w.agency.get_mut(&ISSUER).unwrap().config.financing = Some(terms.clone());
        w.agency.get_mut(&TOWN).unwrap().config.financing = Some(Terms {
            programs: Default::default(),
            lend: true,
            ..terms
        });
        w.transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .insert((TOWN_TYPE, Action::Lend));
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}
