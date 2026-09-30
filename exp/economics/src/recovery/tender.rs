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
    /// Explicit guarantor/creditor consent to a fixed payment per native claim unit.
    /// This changes performance of this guarantee only; recourse stays native.
    AgreedCoins {
        resource: ResourceId,
        coins_per_unit: i32,
    },
}

pub(super) fn terms(world: &World, g: &Guarantee) -> Result<Option<CoinPayment>, String> {
    let t = match g.tender {
        GuaranteeTender::Native => return Ok(None),
        GuaranteeTender::AcceptedLandCoins => {
            let GuaranteedClaim::Land { agreement, .. } = g.claim else {
                return Err(
                    "alternative guarantee tender requires accepted land coin terms".into(),
                );
            };
            world
                .activities
                .coin_payments
                .get(&agreement)
                .ok_or("alternative guarantee without accepted land coin terms")?
                .clone()
        }
        GuaranteeTender::AgreedCoins {
            resource,
            coins_per_unit,
        } => {
            if !matches!(
                g.claim,
                GuaranteedClaim::Loan(_)
                    | GuaranteedClaim::Wages { .. }
                    | GuaranteedClaim::Forward(_)
            ) || g.security != super::RecourseSecurity::Unsecured
            {
                return Err(
                    "agreed coin tender requires a loan, wage or forward claim and unsecured recourse"
                        .into(),
                );
            }
            CoinPayment {
                resource,
                coins_per_unit,
            }
        }
    };
    let (_, _, native) = g.claim.parties(world).ok_or("missing guaranteed terms")?;
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
    Ok(Some(t))
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
