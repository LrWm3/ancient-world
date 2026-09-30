//! Land, forward and earned-wage admission views; their source receipts remain authoritative.
use crate::{
    commitments, credit,
    finance::{self, ContractId},
    model::*,
    recovery,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    pub contract: ContractId,
    pub creditor: AgentId,
    pub due: u32,
    pub remaining: Amount,
}

/// Includes existing bills, earned wages and accepted future forward deliveries,
/// never hypothetical future annual rent or payroll. A view does not accelerate or convert performance obligations.
pub fn outstanding(world: &World, state: &State, debtor: AgentId) -> Vec<Claim> {
    let mut claims = vec![];
    for a in commitments::active(world, state).filter(|a| a.debtor == debtor) {
        for o in state
            .obligations
            .values()
            .filter(|o| o.agreement == a.id && o.paid < o.owed)
        {
            claims.push(Claim {
                contract: ContractId::Land(a.id),
                creditor: a.creditor,
                due: o.due,
                remaining: Amount::new(a.payment.resource, o.owed - o.paid),
            });
        }
    }
    for c in state
        .exchange
        .forwards
        .values()
        .filter(|c| c.debtor == debtor && c.claim().outstanding() > 0)
    {
        claims.push(Claim {
            contract: ContractId::Forward(c.id),
            creditor: c.creditor,
            due: c.effective_due(),
            remaining: Amount::new(c.goods.resource, c.claim().outstanding()),
        });
    }
    for (&(id, month), earned) in &state.employment.earned {
        if earned.claim.transfer.from == debtor && earned.claim.outstanding() > 0 {
            claims.push(Claim {
                contract: ContractId::Wages(id),
                creditor: earned.claim.transfer.to,
                due: month + 1,
                remaining: Amount::new(
                    earned.claim.transfer.amount.resource,
                    earned.claim.outstanding(),
                ),
            });
        }
    }
    claims.sort_by_key(|c| (c.contract, c.due));
    claims
}

pub(crate) fn current(state: &State, out: &credit::Boundary) -> State {
    let mut current = state.clone();
    current.credit = out.after.clone();
    if let Some(book) = &out.employment {
        current.employment = book.clone();
    }
    current
        .exchange
        .forwards
        .extend(out.forward_changes.clone());
    if let Some(s) = &out.commitments {
        current.obligations = s.obligations.clone();
    }
    current
}

/// Estate cash is eligible only where it is the native denomination or an
/// explicitly accepted tender. Forward advance prices never become cash buyouts.
pub(crate) fn cash_requests(
    world: &World,
    state: &State,
    out: &credit::Boundary,
    p: &recovery::ProceedingTerms,
) -> Result<(Vec<finance::CollectionRequest>, BTreeMap<ContractId, i32>), String> {
    let current = current(state, out);
    let claims = outstanding(world, &current, p.debtor);
    let mut requests = vec![];
    let mut lots = BTreeMap::new();
    for a in commitments::active(world, &current).filter(|a| a.debtor == p.debtor) {
        let rate = if a.payment.resource == p.denomination {
            1
        } else {
            match world.activities.coin_payments.get(&a.id) {
                Some(t) if t.resource == p.denomination => t.coins_per_unit,
                _ => continue,
            }
        };
        let contract = ContractId::Land(a.id);
        let units = claims
            .iter()
            .filter(|c| c.contract == contract && c.due <= state.month)
            .try_fold(0_i32, |n, c| {
                n.checked_add(c.remaining.quantity)
                    .ok_or("estate land claim overflow")
            })?;
        let quantity = units
            .checked_mul(rate)
            .ok_or("estate land tender overflow")?;
        lots.insert(contract, rate);
        requests.push(finance::CollectionRequest {
            contract,
            rank: world
                .claim_priorities
                .get(&contract)
                .copied()
                .unwrap_or(finance::DEFAULT_CLAIM_RANK),
            claim: finance::Obligation {
                transfer: finance::Transfer {
                    from: p.estate,
                    to: a.creditor,
                    amount: Amount::new(p.denomination, quantity),
                },
                settled: 0,
                condition: finance::Condition::OnOrAfterMonth(state.month),
                failure: finance::FailureRule::CarryArrears,
            },
        });
    }
    for t in world
        .employment
        .iter()
        .filter(|t| t.employer == p.debtor && t.wage_per_unit.resource == p.denomination)
    {
        let contract = ContractId::Wages(t.id);
        let quantity = claims
            .iter()
            .filter(|c| c.contract == contract && c.due <= state.month)
            .try_fold(0_i32, |q, c| {
                q.checked_add(c.remaining.quantity)
                    .ok_or("estate wage overflow")
            })?;
        lots.insert(contract, 1);
        requests.push(finance::CollectionRequest {
            contract,
            rank: world
                .claim_priorities
                .get(&contract)
                .copied()
                .unwrap_or(t.rank),
            claim: finance::Obligation {
                transfer: finance::Transfer {
                    from: p.estate,
                    to: t.worker,
                    amount: Amount::new(p.denomination, quantity),
                },
                settled: 0,
                condition: finance::Condition::OnOrAfterMonth(state.month),
                failure: finance::FailureRule::CarryArrears,
            },
        });
    }
    Ok((requests, lots))
}

/// Apply a funded aggregate grant oldest bill first. Each transfer and issuance
/// keeps the existing land receipt semantics; no duplicate claim balance is stored.
pub(crate) fn pay_land(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    p: &recovery::ProceedingTerms,
    request: &finance::CollectionRequest,
    grant: (i32, i32),
    execution: &mut finance::Execution,
) -> Result<i32, String> {
    let (allocated, rate) = grant;
    let ContractId::Land(id) = request.contract else {
        return Err("expected land claim".into());
    };
    let a = commitments::active(world, state)
        .find(|a| a.id == id)
        .ok_or("missing estate land agreement")?;
    let native = a.payment.resource == p.denomination;
    let mut settlement = out.commitments.clone().unwrap_or(commitments::Settlement {
        policy: world.payment_policy,
        protected: commitments::protected_stock(world, state)?,
        obligations: commitments::due_obligations(world, state)?,
        transactions: vec![],
    });
    let mut remaining = allocated / rate;
    let mut paid = 0;
    for o in settlement
        .obligations
        .values_mut()
        .filter(|o| o.agreement == id && o.due <= state.month)
    {
        let units = remaining.min(o.owed - o.paid);
        if units == 0 {
            continue;
        }
        let effects = execution.exchange(
            world,
            &[finance::Transfer {
                from: p.estate,
                to: a.creditor,
                amount: Amount::new(p.denomination, units * rate),
            }],
        )?;
        let txs = commitments::record_payment(world, a, o, units, native, effects);
        out.transactions.extend(txs.clone());
        settlement.transactions.extend(txs);
        remaining -= units;
        paid += units * rate;
    }
    out.commitments = Some(settlement);
    out.recovery.push(recovery::Receipt::LandDistributed {
        proceeding: p.id,
        agreement: id,
        creditor: a.creditor,
        requested: Amount::new(a.payment.resource, request.claim.outstanding() / rate),
        allocated: allocated / rate,
        paid: paid / rate,
        tender: Amount::new(p.denomination, paid),
    });
    Ok(paid)
}

/// A wage contract receives one grant; apply it to dated earnings oldest first.
/// The authoritative employment book is updated atomically with estate custody.
pub(crate) fn pay_wages(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    p: &recovery::ProceedingTerms,
    request: &finance::CollectionRequest,
    allocated: i32,
    execution: &mut finance::Execution,
) -> Result<i32, String> {
    let ContractId::Wages(id) = request.contract else {
        return Err("expected wage claim".into());
    };
    let book = out
        .employment
        .get_or_insert_with(|| state.employment.clone());
    let mut remaining = allocated;
    let mut total = 0;
    for (&(agreement, month), earned) in &mut book.earned {
        if agreement != id || month >= state.month || earned.claim.outstanding() == 0 {
            continue;
        }
        let mut claim = earned.claim.clone();
        claim.transfer.from = p.estate;
        let requested = claim.outstanding();
        let grant = remaining.min(requested);
        let payment = execution.pay_bounded(world, state.month, true, &claim, grant)?;
        earned.claim.settled += payment.paid;
        remaining -= payment.paid;
        total += payment.paid;
        if payment.paid > 0 {
            out.transactions.push(credit::tx(
                format!("estate {} wage distribution", p.id),
                payment.effects,
            ));
        }
        out.recovery.push(recovery::Receipt::WagesDistributed {
            proceeding: p.id,
            agreement: id,
            earned_month: month,
            creditor: claim.transfer.to,
            requested,
            allocated: grant,
            paid: payment.paid,
        });
    }
    Ok(total)
}
