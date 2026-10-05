//! Bilateral, independently scored proposals; ordinary adapters admit and collect.
use super::*;
use agency::objectives::{Metric, Objective, Scope};

const LOAN_GRACE_MONTHS: u32 = 1;
const FORWARD_LOT: i32 = 1;

fn objectives(w: &World, agent: AgentId, coin: ResourceId) -> Vec<Objective> {
    let mut result = w
        .agency
        .get(&agent)
        .map(|c| c.config.objectives.clone())
        .unwrap_or_else(|| needs(w, agent));
    // Never improve a financial projection merely by rolling debt beyond it.
    result.push(Objective {
        scope: Scope::Organization,
        metric: Metric::Debt(coin),
    });
    result
}

pub(super) fn discover(w: &mut World, s: &State, c: &Config) -> Result<(), String> {
    let Some(rule) = &c.finance else {
        return Ok(());
    };
    loans(w, s, c, rule)?;
    forwards(w, s, c, rule)
}

fn loans(w: &mut World, s: &State, c: &Config, rule: &FinanceRule) -> Result<(), String> {
    let Some(mint) = w.minting.clone() else {
        return Ok(());
    };
    let Some(policy) = &mint.order_policy else {
        return Ok(());
    };
    let debtor = mint.issuer;
    if !w.agency.contains_key(&debtor)
        || !opportunities::permits(w, s, debtor, Action::Borrow)
        || s.credit
            .loans
            .values()
            .any(|l| l.debtor == debtor && l.debt().unwrap_or(i32::MAX) > 0)
        || w.lending
            .iter()
            .any(|a| a.debtor == debtor && a.month >= s.month)
    {
        return Ok(());
    }
    let venue = crate::marketplace::venue(w, mint.venue).ok_or("missing finance venue")?;
    let definition = w.definition(mint.definition);
    let mut budget = 0i32;
    for (&market, &price) in &policy.input_limits {
        let market = venue
            .markets
            .iter()
            .find(|m| m.id == market)
            .ok_or("unknown financing input")?;
        let needed: i32 = definition.stages[0]
            .entry_inputs
            .iter()
            .chain(&definition.stages[0].monthly_services)
            .filter(|a| a.resource == market.goods.resource)
            .map(|a| a.quantity)
            .sum();
        let held = if w
            .resources
            .iter()
            .any(|r| r.id == market.goods.resource && r.kind == ResourceKind::Capacity)
        {
            0
        } else {
            s.balance(debtor, market.goods.resource)
        };
        budget = budget
            .checked_add(
                ((needed - held).max(0) / market.goods.quantity)
                    .checked_mul(price)
                    .ok_or("funding overflow")?,
            )
            .ok_or("funding overflow")?;
    }
    let principal = budget
        .saturating_sub(s.balance(debtor, rule.denomination))
        .max(0);
    if principal == 0 {
        return Ok(());
    }
    let horizon = c.horizon.max(rule.loan_months + FORECAST_BUFFER_MONTHS);
    let baseline = forecast(w, s, horizon)?;
    let borrower_objectives = objectives(w, debtor, rule.denomination);
    for lender in people(w, s) {
        if lender == debtor
            || s.balance(lender, rule.denomination) < principal
            || !opportunities::permits(w, s, lender, Action::Lend)
        {
            continue;
        }
        let id = next_id(
            w.lending
                .iter()
                .map(|l| l.id)
                .chain(s.credit.loans.keys().copied()),
        )?;
        let advance = crate::credit::Advance {
            id,
            debtor,
            principal,
            month: s.month,
            collateral: None,
            priority: 0,
            terms: crate::credit::LoanOffer {
                creditor: lender,
                denomination: rule.denomination,
                max_principal: principal,
                monthly_rate_bps: rule.monthly_rate_bps,
                term_months: rule.loan_months,
                grace_months: LOAN_GRACE_MONTHS,
            },
        };
        let mut candidate = w.clone();
        candidate.lending.push(advance.clone());
        // The borrower evaluates a productive use of the requested capital.
        // This provisional target is confined to its forecast, not a public promise.
        let target = s.month.checked_add(1).ok_or("funded start overflow")?;
        if !candidate
            .scheduled_starts
            .iter()
            .any(|p| p.agent == debtor && p.definition == mint.definition && p.month == target)
        {
            candidate.scheduled_starts.push(ScheduledStart {
                month: target,
                agent: debtor,
                definition: mint.definition,
            });
            let p = candidate
                .minting
                .as_mut()
                .unwrap()
                .order_policy
                .as_mut()
                .unwrap();
            if p.month == 0 {
                p.month = target;
            } else if target > p.month {
                p.additional_months.insert(target);
            }
        }
        let projected = match forecast(&candidate, s, horizon) {
            Ok(v) => v,
            Err(e) => {
                record(
                    w,
                    s,
                    format!("loan {lender}->{debtor}: {e}"),
                    BTreeMap::new(),
                    false,
                );
                continue;
            }
        };
        let mut lender_objectives = objectives(w, lender, rule.denomination);
        lender_objectives.push(Objective {
            scope: Scope::Organization,
            metric: Metric::Reserve {
                resource: rule.denomination,
                target: s.balance(lender, rule.denomination),
            },
        });
        let comparisons = [
            (
                debtor,
                (
                    loss(&baseline, debtor, &borrower_objectives)?,
                    loss(&projected, debtor, &borrower_objectives)?,
                ),
            ),
            (
                lender,
                (
                    loss(&baseline, lender, &lender_objectives)?,
                    loss(&projected, lender, &lender_objectives)?,
                ),
            ),
        ]
        .into();
        let repaid = projected
            .state
            .credit
            .loans
            .get(&id)
            .is_some_and(|l| l.status == crate::credit::Status::Repaid);
        let accepted = repaid && mutually_beneficial(&comparisons);
        record(
            w,
            s,
            format!("loan proposal {id}: {lender}->{debtor} principal {principal}"),
            comparisons,
            accepted,
        );
        if accepted {
            w.lending.push(advance);
            break;
        }
    }
    Ok(())
}
fn mutually_beneficial(comparisons: &BTreeMap<AgentId, (Vec<i128>, Vec<i128>)>) -> bool {
    comparisons.values().all(|(a, b)| b <= a) && comparisons.values().any(|(a, b)| b < a)
}

fn forwards(w: &mut World, s: &State, c: &Config, rule: &FinanceRule) -> Result<(), String> {
    // An organization's stock reserve objective is demand; a forecast surplus is
    // an offer candidate. One finite lot is attempted per buyer at each boundary.
    let buyers: Vec<_> = w
        .agency
        .iter()
        .flat_map(|(&a, controller)| {
            controller
                .config
                .objectives
                .iter()
                .filter_map(move |o| match o.metric {
                    Metric::Reserve { resource, target }
                        if o.scope == Scope::Organization && resource != rule.denomination =>
                    {
                        Some((a, resource, target))
                    }
                    _ => None,
                })
        })
        .collect();
    for (buyer, resource, target) in buyers {
        let Some(&price) = rule.unit_values.get(&resource) else {
            continue;
        };
        if s.balance(buyer, resource) >= target
            || s.balance(buyer, rule.denomination) < price
            || s.exchange
                .forwards
                .values()
                .any(|f| f.creditor == buyer && f.claim().outstanding() > 0)
        {
            continue;
        }
        let horizon = c.horizon.max(rule.delivery_months + FORECAST_BUFFER_MONTHS);
        let baseline = forecast(w, s, horizon)?;
        let mut sellers: Vec<_> = w
            .agents
            .iter()
            .map(|a| a.id)
            .filter(|a| *a != buyer && baseline.state.balance(*a, resource) > FORWARD_LOT)
            .collect();
        sellers.sort_unstable();
        for seller in sellers {
            let id = next_id(
                w.prepaid_deliveries
                    .iter()
                    .map(|f| f.id)
                    .chain(s.exchange.forwards.keys().copied()),
            )?;
            let terms = crate::forward::direct::Terms {
                id,
                buyer,
                seller,
                month: s.month,
                due: s
                    .month
                    .checked_add(rule.delivery_months)
                    .ok_or("delivery date overflow")?,
                goods: Amount::new(resource, FORWARD_LOT),
                prepayment: Amount::new(rule.denomination, price),
            };
            let mut candidate = w.clone();
            candidate.prepaid_deliveries.push(terms.clone());
            let projected = match forecast(&candidate, s, horizon) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let buyer_objectives = objectives(w, buyer, rule.denomination);
            let mut seller_objectives = objectives(w, seller, rule.denomination);
            seller_objectives.push(Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: rule.denomination,
                    target: baseline
                        .state
                        .balance(seller, rule.denomination)
                        .saturating_add(price),
                },
            });
            let comparisons = [
                (
                    buyer,
                    (
                        loss(&baseline, buyer, &buyer_objectives)?,
                        loss(&projected, buyer, &buyer_objectives)?,
                    ),
                ),
                (
                    seller,
                    (
                        loss(&baseline, seller, &seller_objectives)?,
                        loss(&projected, seller, &seller_objectives)?,
                    ),
                ),
            ]
            .into();
            let delivered = projected
                .state
                .exchange
                .forwards
                .get(&id)
                .is_some_and(|f| f.delivered == terms.goods.quantity);
            let accepted = delivered && mutually_beneficial(&comparisons);
            record(
                w,
                s,
                format!("forward proposal {id}: {seller}->{buyer}"),
                comparisons,
                accepted,
            );
            if accepted {
                w.prepaid_deliveries.push(terms);
                break;
            }
        }
    }
    Ok(())
}
