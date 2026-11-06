//! Source claim inventory and admission check for the liquid-only bootstrap.
//! These records are evidence, never spendable stock or executable loan terms.
use super::stocks::StockBindings;
use crate::{host::Boundary, model::AgentId};
use ::ancient_world::{
    civilization::History,
    credit::{Account, EntryKind, RepaymentSource, Status},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AccountBinding {
    pub source: Account,
    pub agent: AgentId,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HouseholdShare {
    pub household: u32,
    pub agent: AgentId,
    pub head: u32,
    pub vacant_since: Option<u32>,
    /// Original beneficial share of private town stock, not another inventory.
    /// Neither normalized nor applied to cash/stock by this adapter.
    pub share: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Assignment {
    pub sequence: u64,
    pub request: u64,
    pub decided_month: u32,
    pub effective_month: u32,
    pub from: Account,
    pub to: Account,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Amounts {
    pub principal: f64,
    pub interest: f64,
}

impl Amounts {
    fn validate(&self) -> Result<(), String> {
        if [self.principal, self.interest]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.)
        {
            Ok(())
        } else {
            Err("invalid or overflowing source claim total".into())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LoanClaim {
    pub loan: u64,
    pub currency: u32,
    pub original_lender: Account,
    pub creditor: Account,
    /// Includes only assignments already committed at the completed boundary.
    pub next_creditor: Account,
    pub borrower: Account,
    pub creditor_agent: Option<AgentId>,
    pub next_creditor_agent: Option<AgentId>,
    pub borrower_agent: Option<AgentId>,
    pub repayment_source: RepaymentSource,
    pub status: Status,
    pub opened_month: u32,
    pub accrued_through_month: u32,
    pub maturity_month: u32,
    pub grace_months: u32,
    pub annual_simple_rate: f64,
    pub original_principal: f64,
    pub outstanding: Amounts,
    pub repaid: Amounts,
    pub written_off: Amounts,
    pub precision_writeoff: Amounts,
    pub recovered: Amounts,
    /// Sequential subtraction in source receipt order; never restored to debt.
    pub unrecovered: Amounts,
    pub assignments: Vec<Assignment>,
}

impl LoanClaim {
    pub fn blocks_liquid_import(&self) -> bool {
        self.outstanding.principal > 0.
            || self.outstanding.interest > 0.
            // Even full recovery retains a default's policy consequences.
            || self.status == Status::Defaulted
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ClaimInventory {
    pub boundary: Boundary,
    pub accounts: Vec<AccountBinding>,
    pub household_shares: Vec<HouseholdShare>,
    /// Source sum for inspection, not an assertion of validated property law.
    pub total_household_share: f64,
    /// One record per contract involving the selected accounts now or at the
    /// next opening. An external counterparty has no experimental account.
    pub loans: Vec<LoanClaim>,
}

impl ClaimInventory {
    pub fn require_liquid_only(&self) -> Result<(), String> {
        let blocked: Vec<_> = self
            .loans
            .iter()
            .filter(|l| l.blocks_liquid_import())
            .map(|l| l.loan)
            .collect();
        if blocked.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "liquid import cannot omit source credit claims {blocked:?}; inspect_claims reports their terms and owners"
            ))
        }
    }
}

impl StockBindings {
    /// Inspect the completed source without accruing interest or moving money.
    /// This validates native credit receipts/ownership, including external
    /// counterparties, but is not the main world's full archive validator.
    pub fn inspect_claims(&self, completed: &History) -> Result<ClaimInventory, String> {
        let next_month = completed
            .month
            .checked_add(1)
            .ok_or("claim boundary overflow")?;
        if !completed
            .sites
            .get(self.site as usize)
            .is_some_and(|s| s.id == self.site && s.founded <= completed.month)
        {
            return Err("missing claim source site".into());
        }
        let society = completed
            .society
            .as_ref()
            .ok_or("missing claim source households")?;
        let wallets = society
            .household_economy
            .as_ref()
            .ok_or("missing claim source wallets")?;
        if wallets.accounts.len() != society.households.len()
            || society
                .households
                .iter()
                .enumerate()
                .any(|(id, h)| h.id as usize != id)
        {
            return Err("invalid claim source household identities".into());
        }
        wallets.validate(completed).map_err(|e| e.to_string())?;
        let expected: BTreeSet<_> = society
            .households
            .iter()
            .filter(|h| h.site == self.site)
            .map(|h| h.id)
            .collect();
        let mut accounts = BTreeMap::from([(Account::Town(self.site), self.town_agent)]);
        let mut agents = BTreeSet::from([self.town_agent]);
        let mut seen = BTreeSet::new();
        let mut household_shares = Vec::new();
        for binding in &self.wallets {
            if !expected.contains(&binding.household)
                || !seen.insert(binding.household)
                || !agents.insert(binding.agent)
            {
                return Err("incomplete, duplicate or merged claim account binding".into());
            }
            let h = &society.households[binding.household as usize];
            if h.founded > completed.month
                || !h.share.is_finite()
                || h.share < 0.
                || h.vacant_since
                    .is_some_and(|m| m < h.founded || m > completed.month)
                || completed
                    .people
                    .get(h.head as usize)
                    .is_none_or(|p| p.id != h.head)
            {
                return Err("invalid household ownership evidence".into());
            }
            accounts.insert(Account::Household(h.id), binding.agent);
            household_shares.push(HouseholdShare {
                household: h.id,
                agent: binding.agent,
                head: h.head,
                vacant_since: h.vacant_since,
                share: h.share,
            });
        }
        if seen != expected {
            return Err("claim inventory must cover every local household".into());
        }
        household_shares.sort_by_key(|h| h.household);
        let total_household_share: f64 = household_shares.iter().map(|h| h.share).sum();
        if !total_household_share.is_finite() {
            return Err("household share sum overflow".into());
        }
        completed.validate_credit().map_err(|e| e.to_string())?;
        let mut loans = Vec::new();
        for report in completed.credit.loan_reports() {
            let source = &completed.credit.loans[report.id as usize];
            let creditor = completed
                .credit
                .ownership
                .owner_at(source, completed.month)
                .map_err(|e| e.to_string())?;
            let next_creditor = completed
                .credit
                .ownership
                .owner_at(source, next_month)
                .map_err(|e| e.to_string())?;
            if ![creditor, next_creditor, report.borrower]
                .iter()
                .any(|a| accounts.contains_key(a))
            {
                continue;
            }
            let losses: Vec<_> = source
                .entries
                .iter()
                .filter(|e| matches!(e.kind, EntryKind::WriteOff))
                .collect();
            if (report.status == Status::Defaulted
                && (losses.len() != 1 || (losses[0].principal == 0. && losses[0].interest == 0.)))
                || (report.status != Status::Defaulted && !losses.is_empty())
            {
                return Err("source default status disagrees with write-off evidence".into());
            }
            let mut unrecovered = Amounts::default();
            if report.status == Status::Defaulted {
                unrecovered = Amounts {
                    principal: losses[0].principal,
                    interest: losses[0].interest,
                };
                for recovery in completed
                    .credit
                    .recoveries
                    .iter()
                    .filter(|r| r.request.loan == source.id)
                {
                    unrecovered.principal -= recovery.transfer.principal;
                    unrecovered.interest -= recovery.transfer.interest;
                }
            }
            let claim = LoanClaim {
                loan: report.id,
                currency: report.currency.0,
                original_lender: report.lender,
                creditor,
                next_creditor,
                borrower: report.borrower,
                creditor_agent: accounts.get(&creditor).copied(),
                next_creditor_agent: accounts.get(&next_creditor).copied(),
                borrower_agent: accounts.get(&report.borrower).copied(),
                repayment_source: report.source,
                status: report.status,
                opened_month: report.opened_month,
                accrued_through_month: source.accrued_through_month,
                maturity_month: report.maturity_month,
                grace_months: source.terms.grace_months,
                annual_simple_rate: report.annual_simple_rate,
                original_principal: report.original_principal,
                outstanding: Amounts {
                    principal: report.due.principal,
                    interest: report.due.interest,
                },
                repaid: Amounts {
                    principal: report.repaid.principal,
                    interest: report.repaid.interest,
                },
                written_off: Amounts {
                    principal: report.default_loss.principal,
                    interest: report.default_loss.interest,
                },
                precision_writeoff: Amounts {
                    principal: report.precision_writeoff.principal,
                    interest: report.precision_writeoff.interest,
                },
                recovered: Amounts {
                    principal: report.recovered.principal,
                    interest: report.recovered.interest,
                },
                unrecovered,
                assignments: report
                    .assignments
                    .into_iter()
                    .map(|a| Assignment {
                        sequence: a.sequence,
                        request: a.request.id,
                        decided_month: a.request.month,
                        effective_month: a.effective_month,
                        from: a.request.from,
                        to: a.request.to,
                    })
                    .collect(),
            };
            for amounts in [
                &claim.outstanding,
                &claim.repaid,
                &claim.written_off,
                &claim.precision_writeoff,
                &claim.recovered,
                &claim.unrecovered,
            ] {
                amounts.validate()?;
            }
            loans.push(claim);
        }
        Ok(ClaimInventory {
            boundary: Boundary {
                site: self.site,
                month: completed.month,
            },
            accounts: accounts
                .into_iter()
                .map(|(source, agent)| AccountBinding { source, agent })
                .collect(),
            household_shares,
            total_household_share,
            loans,
        })
    }
}
