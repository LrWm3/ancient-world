//! Explicit model-specific currency convention; no inferred redemption promise.
use crate::{
    accounting::{self, Account, Flow, Line},
    commitments,
    model::*,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Newly authorized face value adds issuer equity, never operating revenue.
    /// Material and delivered monthly-service costs remain expenses.
    NonRedeemableEquity,
}
fn issue(agent: AgentId, quantity: i128, lines: &mut Vec<Line>) {
    if quantity != 0 {
        lines.extend([
            Line {
                agent,
                account: Account::Cash,
                debit: quantity,
                flow: Some(Flow::Issuance),
            },
            Line {
                agent,
                account: Account::MonetaryIssuance,
                debit: -quantity,
                flow: None,
            },
        ]);
    }
}
/// Project authorized monetary outputs out of costed inventory processing.
/// The original transaction and its material/capacity effects remain authoritative.
pub(crate) fn processes(
    world: &World,
    transactions: &[Transaction],
    coin: ResourceId,
) -> Result<(Vec<Transaction>, Vec<Line>), String> {
    let mut costs = transactions.to_vec();
    let mut lines = vec![];
    for t in &mut costs {
        let Some(change) = &t.process else { continue };
        let Some(c) = world
            .minting
            .as_ref()
            .filter(|c| c.definition == change.after.definition)
        else {
            continue;
        };
        if c.coin != coin
            || change.after.operator != c.issuer
            || change.after.beneficiary != c.issuer
        {
            return Err("unsupported monetary issuer or denomination".into());
        }
        let mut issued = 0_i128;
        for e in t.effects.iter().filter(|e| e.account.1 == coin) {
            if e.account.0 != c.issuer || e.delta <= 0 || change.after.status != Status::Completed {
                return Err("unsupported mint output".into());
            }
            issued = issued
                .checked_add(i128::from(e.delta))
                .ok_or("issuance overflow")?;
        }
        t.effects.retain(|e| e.account.1 != coin);
        issue(c.issuer, issued, &mut lines);
    }
    Ok((costs, lines))
}
/// Capacity is a delivered current-period service, not owned stock or future labor.
pub(crate) fn services(t: &Transaction, coin: ResourceId) -> Result<Vec<Line>, String> {
    let sold = t
        .effects
        .iter()
        .find(|e| e.account.1 != coin && e.delta < 0)
        .ok_or("missing service sale")?;
    let bought = t
        .effects
        .iter()
        .find(|e| e.account.1 == sold.account.1 && e.delta > 0)
        .ok_or("missing service receipt")?;
    let price = t
        .effects
        .iter()
        .find(|e| e.account == (sold.account.0, coin) && e.delta > 0)
        .ok_or("missing service proceeds")?;
    let paid = t
        .effects
        .iter()
        .find(|e| e.account == (bought.account.0, coin) && e.delta < 0)
        .ok_or("missing service payment")?;
    if t.effects.len() != 4
        || sold.account.0 == bought.account.0
        || i64::from(sold.delta) != -i64::from(bought.delta)
        || i64::from(price.delta) != -i64::from(paid.delta)
    {
        return Err("invalid paid service exchange".into());
    }
    let value = i128::from(price.delta);
    Ok(vec![
        Line {
            agent: sold.account.0,
            account: Account::Cash,
            debit: value,
            flow: Some(Flow::Operating),
        },
        Line {
            agent: sold.account.0,
            account: Account::ServiceIncome,
            debit: -value,
            flow: None,
        },
        Line {
            agent: bought.account.0,
            account: Account::Cash,
            debit: -value,
            flow: Some(Flow::Operating),
        },
        Line {
            agent: bought.account.0,
            account: Account::ServiceExpense,
            debit: value,
            flow: None,
        },
    ])
}
/// Remove only verified collection issuance from the dues transfer projection.
/// Reconstruct authorization from cumulative native receipts, not cause strings.
pub(crate) fn collection(
    world: &World,
    before: &State,
    after: &State,
    coin: ResourceId,
    transactions: &[Transaction],
) -> Result<(Vec<Transaction>, Vec<Line>), String> {
    let mut expected = BTreeMap::new();
    for (key, new) in &after.obligations {
        let Some(rule) = world.issuance.iter().find(|r| r.agreement == key.0) else {
            continue;
        };
        let old = before.obligations.get(key).map_or(0, |o| o.in_kind_paid);
        let quantity = new.in_kind_paid / rule.collected_per_token - old / rule.collected_per_token;
        if quantity < 0 {
            return Err("issuance receipts cannot be reversed".into());
        }
        if quantity == 0 {
            continue;
        }
        if rule.token != coin {
            return Err("mixed issuance denomination".into());
        }
        let agreement = commitments::active(world, after)
            .find(|a| a.id == key.0)
            .ok_or("missing issuance agreement")?;
        accounting::add(&mut expected, agreement.creditor, i128::from(quantity))?;
    }
    let mut transfers = vec![];
    let mut actual = BTreeMap::new();
    for t in transactions {
        if let [e] = t.effects.as_slice()
            && e.account.1 == coin
            && e.delta > 0
        {
            accounting::add(&mut actual, e.account.0, i128::from(e.delta))?;
        } else {
            transfers.push(t.clone());
        }
    }
    if actual != expected {
        return Err("collection issuance does not reconcile to native receipts".into());
    }
    let mut lines = vec![];
    for (issuer, quantity) in actual {
        issue(issuer, quantity, &mut lines);
    }
    Ok((transfers, lines))
}
