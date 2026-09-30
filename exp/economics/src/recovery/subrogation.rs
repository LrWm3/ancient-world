//! Explicit inheritance of authorized-liquidation security, including proceeds
//! already reserved in custody. Recourse retains its ordinary next-month timing.
use super::{Guarantee, GuaranteedClaim};
use crate::{
    credit::{self, Collateral, Loan},
    model::*,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RecourseSecurity {
    #[default]
    Unsecured,
    /// Inherit the original lien rank; equal ranks use the estate allocation policy.
    InheritLiquidationLien,
}

pub(super) fn terms<'a>(world: &'a World, g: &Guarantee) -> Result<Option<&'a Collateral>, String> {
    if g.security == RecourseSecurity::Unsecured {
        return Ok(None);
    }
    let mut claim = g.claim;
    let mut visited = std::collections::BTreeSet::new();
    loop {
        let GuaranteedClaim::Loan(id) = claim else {
            return Err("lien subrogation requires a secured loan".into());
        };
        if !visited.insert(id) {
            return Err("cyclic lien subrogation".into());
        }
        if let Some(offered) = credit::offered_loan(world, id) {
            let collateral = offered
                .collateral
                .filter(|c| c.settlement == credit::CollateralSettlement::AuthorizedLiquidation)
                .ok_or("lien subrogation requires authorized-liquidation collateral")?;
            return Ok(Some(collateral));
        }
        let source = world
            .recovery
            .guarantees
            .iter()
            .find(|source| source.recourse == id)
            .filter(|source| source.security == RecourseSecurity::InheritLiquidationLien)
            .ok_or("lien subrogation requires an uninterrupted inherited lien")?;
        claim = source.claim;
    }
}

pub(super) fn validate_loan(world: &World, g: &Guarantee, loan: &Loan) -> Result<bool, String> {
    Ok(match (terms(world, g)?, loan.collateral.as_ref()) {
        (None, None) => true,
        (Some(expected), Some(actual)) => {
            expected.asset == actual.asset
                && expected.priority == actual.priority
                && expected.settlement == actual.settlement
        }
        _ => false,
    })
}

/// Capture before the original creditor's final payment releases its pledge.
pub(super) fn collateral(
    world: &World,
    book: &credit::Book,
    g: &Guarantee,
) -> Result<Option<Collateral>, String> {
    if terms(world, g)?.is_none() {
        return Ok(None);
    }
    let GuaranteedClaim::Loan(id) = g.claim else {
        unreachable!()
    };
    Ok(Some(
        book.loans
            .get(&id)
            .and_then(|l| l.collateral.clone())
            .ok_or("subrogation without original collateral")?,
    ))
}

/// Paid debt transfers its reservation, never duplicates it. Any uncovered part
/// of the new recourse remains an ordinary deficiency after collateral is sold.
pub(super) fn transfer_reserved(
    book: &mut credit::Book,
    g: &Guarantee,
    paid: i32,
) -> Result<(), String> {
    if g.security == RecourseSecurity::Unsecured {
        return Ok(());
    }
    let GuaranteedClaim::Loan(id) = g.claim else {
        unreachable!()
    };
    for case in book.recovery.proceedings.values_mut() {
        let amount = case.secured.get(&id).copied().unwrap_or(0).min(paid);
        if amount > 0 {
            *case.secured.get_mut(&id).unwrap() -= amount;
            let reserve = case.secured.entry(g.recourse).or_default();
            *reserve = reserve
                .checked_add(amount)
                .ok_or("subrogated proceeds overflow")?;
        }
    }
    Ok(())
}
