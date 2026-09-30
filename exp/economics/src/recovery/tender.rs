//! A guarantee selects one agreed performance route. Caps and recourse retain
//! native claim units; payment lots prevent fractional discharge of those units.
use super::{Guarantee, GuaranteedClaim};
use crate::{activities::CoinPayment, finance, model::*};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GuaranteeTender {
    #[default]
    Native,
    /// Use the original land agreement's accepted coin rate, without native fallback.
    AcceptedLandCoins,
}

pub(super) fn terms(world: &World, g: &Guarantee) -> Result<Option<CoinPayment>, String> {
    if g.tender == GuaranteeTender::Native {
        return Ok(None);
    }
    let GuaranteedClaim::Land { agreement, .. } = g.claim else {
        return Err("alternative guarantee tender requires accepted land coin terms".into());
    };
    let t = world
        .activities
        .coin_payments
        .get(&agreement)
        .ok_or("alternative guarantee without accepted land coin terms")?;
    let (_, _, native) = g
        .claim
        .parties(world)
        .ok_or("missing guaranteed land terms")?;
    if t.resource == native
        || t.coins_per_unit <= 0
        || !world
            .resources
            .iter()
            .any(|r| r.id == t.resource && r.kind == ResourceKind::Stock)
        || world.storage.weights.get(&t.resource).copied().unwrap_or(0) != 0
    {
        return Err("guarantee coin tender must be distinct, positive and storage-free".into());
    }
    Ok(Some(t.clone()))
}

pub(super) fn funding(
    world: &World,
    g: &Guarantee,
    claim: &finance::Obligation,
) -> Result<(finance::Obligation, i32), String> {
    let Some(t) = terms(world, g)? else {
        return Ok((claim.clone(), 1));
    };
    let mut result = claim.clone();
    result.transfer.amount = Amount::new(
        t.resource,
        claim
            .outstanding()
            .checked_mul(t.coins_per_unit)
            .ok_or("guarantee tender overflow")?,
    );
    result.settled = 0;
    Ok((result, t.coins_per_unit))
}
