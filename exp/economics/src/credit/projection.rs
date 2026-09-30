//! Read-only repayment demand for already accepted loans, in native units.
use crate::{credit, model::*};
use std::collections::BTreeMap;

/// Dated demand under timely future installments and the current legal state.
/// This is not a funding prediction: no new loans, future guarantees, fixture
/// cashflows, collateral realization or hypothetical lifting of stays is assumed.
/// Each projected payment reduces only a private loan copy before next interest.
pub fn dues(
    world: &World,
    state: &State,
    debtor: AgentId,
    resource: ResourceId,
    months: u32,
) -> Result<BTreeMap<u32, i128>, String> {
    let end = state
        .month
        .checked_add(months)
        .ok_or("loan projection horizon overflow")?;
    let mut result = BTreeMap::new();
    if months == 0
        || !state
            .credit
            .loans
            .values()
            .any(|l| l.debtor == debtor && l.denomination == resource)
    {
        return Ok(result);
    }
    let mut boundary = state.clone();
    for original in state
        .credit
        .loans
        .values()
        .filter(|l| l.debtor == debtor && l.denomination == resource)
    {
        let mut loan = original.clone();
        for month in state.month..end {
            if matches!(
                loan.status,
                credit::Status::Repaid | credit::Status::Discharged | credit::Status::PendingSale
            ) {
                break;
            }
            if month <= loan.opened {
                continue;
            }
            boundary.month = month;
            let Some(request) = credit::current_claim(world, &boundary, &loan)? else {
                // A current stay is not forecast to disappear by itself.
                break;
            };
            if loan.status == credit::Status::Active && loan.last_accrued < month {
                credit::accrue(&mut loan, month)?;
            }
            let paid = request.claim.outstanding();
            if paid > 0 {
                *result.entry(month).or_default() += i128::from(paid);
                loan.apply_payment(paid);
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compute::Backend,
        employment::{ArrearsPolicy, Terms},
        financial_reporting::{Audit, Opening},
        minting::{self, COIN, HOURS, ISSUER, SUPPLIER, WORKER},
        recovery::{Guarantee, GuaranteedClaim},
        simulation::Simulation,
    };

    #[test]
    fn repeated_guarantee_calls_share_dated_claims_with_planning() {
        let (mut world, mut state) = minting::scenario("normal").unwrap();
        world.minting = None;
        world.transaction_policy = None;
        world.scheduled_starts.clear();
        world.capacity_overrides.clear();
        world.priority = Priority::ContinuingFirst;
        for p in &mut world.participants {
            p.needs.clear();
            p.capacity.quantity = 2;
        }
        state.balances.clear();
        state.balances.insert((SUPPLIER, COIN), 2);
        // Unpaid work earns four coins. The first guarantee pays two; the worker
        // then buys a real service from the guarantor, funding the second call.
        for (id, employer, worker, month, hours, wage) in
            [(1, ISSUER, WORKER, 1, 2, 2), (2, WORKER, SUPPLIER, 2, 1, 2)]
        {
            world.employment.push(Terms {
                id,
                employer,
                worker,
                from: month,
                through: month,
                capacity: Amount::new(HOURS, hours),
                wage_per_unit: Amount::new(COIN, wage),
                on_arrears: ArrearsPolicy::Continue,
                rank: 0,
            });
        }
        world.recovery.guarantees.push(Guarantee {
            follows_assignment: false,
            tender: crate::recovery::GuaranteeTender::Native,
            security: crate::recovery::RecourseSecurity::Unsecured,
            id: 1,
            claim: GuaranteedClaim::Wages {
                agreement: 1,
                earned_month: 1,
            },
            guarantor: SUPPLIER,
            cap: 4,
            from: 1,
            through: 8,
            delay_months: 0,
            recourse: 101,
            priority: 0,
        });
        let mut audit = Audit::with_opening(&world, &state, COIN, Opening::default()).unwrap();
        let mut sim = Simulation::new(world, state, Backend::CubeCpu).unwrap();
        while sim.state.month < 3 || sim.state.phase != Phase::Acquire {
            audit.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.credit.loans[&101].principal, 4);
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
        assert_eq!(
            credit::current_dues(&sim.world, &sim.state, ISSUER).unwrap()[&COIN],
            2
        );
        assert_eq!(
            crate::need_orders::claims(&sim.world, &sim.state, ISSUER, 1).unwrap()[&COIN],
            2
        );
        assert_eq!(
            dues(&sim.world, &sim.state, ISSUER, COIN, 2).unwrap(),
            [(3, 2), (4, 2)].into()
        );
        let mut resumed =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
        while sim.state.month < 4 {
            audit.step(&mut sim).unwrap();
            resumed.step().unwrap();
        }
        assert_eq!(sim.state, resumed.state);
        assert_eq!(
            credit::current_dues(&sim.world, &sim.state, ISSUER).unwrap()[&COIN],
            4
        );
        for a in &sim.world.agents {
            let report = audit.book().statements(a.id, 1, 3).unwrap();
            assert_eq!(report.assets, report.liabilities + report.equity);
        }
    }
}
