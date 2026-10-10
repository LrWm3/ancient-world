//! Bilateral, independently scored proposals; ordinary adapters admit and collect.
use super::*;
use agency::objectives::{Metric, Objective, Scope};

const LOAN_GRACE_MONTHS: u32 = 1;
const FORWARD_LOT: i32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instrument {
    Loan,
    Forward,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    ProjectionFailed(String),
    PerformanceShortfall,
    NoMutualGain,
    Published,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub counterparty: AgentId,
    /// Rejected candidates can reuse IDs. Only Published correlates with public terms.
    pub candidate_id: u32,
    /// Units of the assessment resource (principal for a loan).
    pub quantity: i32,
    /// Forward prepayment in denomination units; absent for loan proposals.
    pub prepayment: Option<i32>,
    pub comparisons: BTreeMap<AgentId, (Vec<i128>, Vec<i128>)>,
    pub outcome: Outcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    pub month: u32,
    pub instrument: Instrument,
    /// Buyer for a forward, borrower for a loan.
    pub requester: AgentId,
    pub resource: ResourceId,
    pub denomination: ResourceId,
    pub horizon: u32,
    /// Loan term or forward delivery delay, distinct from the assessment horizon.
    pub duration: u32,
    /// Baseline stock candidates for forwards; funded, Lend-permitted people for loans.
    /// This is not a promise of final contract eligibility.
    /// Attempts stop after publication and can be fewer than this count.
    pub candidate_count: usize,
    pub attempts: Vec<Attempt>,
}

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
    for term in std::iter::once(rule.loan_months).chain(
        rule.alternative_loan_months
            .iter()
            .copied()
            .filter(|term| *term != rule.loan_months),
    ) {
        let mut candidate_rule = rule.clone();
        candidate_rule.loan_months = term;
        loans(w, s, c, &candidate_rule)?;
    }
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
                crate::minting::orders::required_lots(needed, held, market.goods.quantity)
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
    let lenders: Vec<_> = people(w, s)
        .into_iter()
        .filter(|lender| {
            *lender != debtor
                && s.balance(*lender, rule.denomination) >= principal
                && opportunities::permits(w, s, *lender, Action::Lend)
        })
        .collect();
    let assessment = w.discovery.as_ref().unwrap().financial.len();
    w.discovery.as_mut().unwrap().financial.push(Assessment {
        month: s.month,
        instrument: Instrument::Loan,
        requester: debtor,
        resource: rule.denomination,
        denomination: rule.denomination,
        horizon,
        duration: rule.loan_months,
        candidate_count: lenders.len(),
        attempts: vec![],
    });
    for lender in lenders {
        let id = next_id(
            w.lending
                .iter()
                .map(|l| l.id)
                .chain(s.credit.loans.keys().copied())
                .chain(w.credit.iter().flat_map(|c| c.offers.iter().map(|o| o.id))),
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
                w.discovery.as_mut().unwrap().financial[assessment]
                    .attempts
                    .push(Attempt {
                        counterparty: lender,
                        candidate_id: id,
                        quantity: principal,
                        prepayment: None,
                        comparisons: BTreeMap::new(),
                        outcome: Outcome::ProjectionFailed(e.clone()),
                    });
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
        w.discovery.as_mut().unwrap().financial[assessment]
            .attempts
            .push(Attempt {
                counterparty: lender,
                candidate_id: id,
                quantity: principal,
                prepayment: None,
                comparisons: comparisons.clone(),
                outcome: if !repaid {
                    Outcome::PerformanceShortfall
                } else if !accepted {
                    Outcome::NoMutualGain
                } else {
                    Outcome::Published
                },
            });
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
    let mut buyers = BTreeMap::<(AgentId, ResourceId), i32>::new();
    for (&agent, controller) in &w.agency {
        for objective in &controller.config.objectives {
            if let Metric::Reserve { resource, target } = objective.metric
                && objective.scope == Scope::Organization
                && resource != rule.denomination
            {
                let demand = buyers.entry((agent, resource)).or_default();
                *demand = (*demand).max(target);
            }
        }
    }
    for ((buyer, resource), target) in buyers {
        let Some(&price) = rule.unit_values.get(&resource) else {
            continue;
        };
        if w.prepaid_deliveries
            .iter()
            .any(|f| f.buyer == buyer && f.month >= s.month)
            || s.balance(buyer, resource) >= target
            || s.balance(buyer, rule.denomination) < price
            || s.exchange
                .forwards
                .values()
                .any(|f| f.creditor == buyer && f.claim().outstanding() > 0)
        {
            continue;
        }
        let horizon = c
            .horizon
            .max(rule.delivery_months + FORECAST_BUFFER_MONTHS)
            .max(rule.forward_horizon.unwrap_or(0));
        let baseline = forecast(w, s, horizon)?;
        let mut sellers: Vec<_> = w
            .agents
            .iter()
            .map(|a| a.id)
            .filter(|a| *a != buyer && baseline.state.balance(*a, resource) >= FORWARD_LOT)
            .collect();
        sellers.sort_unstable();
        let assessment = w.discovery.as_ref().unwrap().financial.len();
        w.discovery.as_mut().unwrap().financial.push(Assessment {
            month: s.month,
            instrument: Instrument::Forward,
            requester: buyer,
            resource,
            denomination: rule.denomination,
            horizon,
            duration: rule.delivery_months,
            candidate_count: sellers.len(),
            attempts: vec![],
        });
        let through = s
            .month
            .checked_add(horizon - 1)
            .ok_or("assessment date overflow")?;
        record(
            w,
            s,
            format!(
                "forward assessment buyer {buyer}: {}..={through}, {horizon} months, {} projected suppliers",
                s.month,
                sellers.len()
            ),
            BTreeMap::new(),
            false,
        );
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
                Err(error) => {
                    w.discovery.as_mut().unwrap().financial[assessment]
                        .attempts
                        .push(Attempt {
                            counterparty: seller,
                            candidate_id: id,
                            quantity: FORWARD_LOT,
                            prepayment: Some(price),
                            comparisons: BTreeMap::new(),
                            outcome: Outcome::ProjectionFailed(error),
                        });
                    continue;
                }
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
            w.discovery.as_mut().unwrap().financial[assessment]
                .attempts
                .push(Attempt {
                    counterparty: seller,
                    candidate_id: id,
                    quantity: FORWARD_LOT,
                    prepayment: Some(price),
                    comparisons: comparisons.clone(),
                    outcome: if !delivered {
                        Outcome::PerformanceShortfall
                    } else if !accepted {
                        Outcome::NoMutualGain
                    } else {
                        Outcome::Published
                    },
                });
            record(
                w,
                s,
                format!(
                    "forward proposal {id}: {seller}->{buyer}; assessment {horizon} months from {}",
                    s.month
                ),
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
