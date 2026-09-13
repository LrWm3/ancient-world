//! Dated creditor identity. This ledger owns claims, never cash or original terms.
//! Original terms remain immutable; History resolves dated payment recipients.
use super::{Account, Loan, Status};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const ASSIGNMENT_NOTICE_MONTHS: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub loan: u64,
    pub month: u32,
    pub from: Account,
    pub to: Account,
    /// Authorization evidence supplied by the owning decision system.
    pub owner_consent: Option<Account>,
    pub recipient_consent: Option<Account>,
    pub cause: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum Basis {
    #[default]
    Consent,
    OperatorEstate {
        household: u32,
        closed_month: u32,
    },
    InstitutionEstate {
        site: u32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assignment {
    #[serde(default)]
    pub basis: Basis,
    pub sequence: u64,
    pub request: Request,
    pub effective_month: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Ownership {
    assignments: Vec<Assignment>,
}

impl Ownership {
    pub fn assignments(&self) -> &[Assignment] {
        &self.assignments
    }

    /// Original identity remains available on Loan. Earlier receipts retain their
    /// dated owner even after further assignments. Call validate on loaded ledgers.
    pub fn owner_at(&self, loan: &Loan, month: u32) -> Result<Account> {
        ensure!(month >= loan.opened_month, "ownership before origination");
        let mut owner = loan.terms.lender;
        for assignment in &self.assignments {
            let request = &assignment.request;
            if request.loan == loan.id && assignment.effective_month <= month {
                ensure!(request.from == owner, "broken creditor ownership chain");
                owner = request.to;
            }
        }
        Ok(owner)
    }

    /// Validate all decisions, including assignments pending for the next month.
    /// Counterparty existence and causal authority require the History adapter.
    pub fn validate(&self, loans: &[Loan], month: u32) -> Result<()> {
        let mut requests = BTreeSet::new();
        let mut owners = BTreeMap::<u64, (Account, u32)>::new();
        let mut previous_month = 0;
        for (sequence, assignment) in self.assignments.iter().enumerate() {
            let request = &assignment.request;
            let loan = loans
                .get(request.loan as usize)
                .filter(|loan| loan.id == request.loan)
                .context("missing assignment loan")?;
            ensure!(
                assignment.sequence == sequence as u64
                    && requests.insert(request.id)
                    && request.month >= previous_month
                    && request.month >= loan.opened_month
                    && request.month <= month
                    && request.month.checked_add(ASSIGNMENT_NOTICE_MONTHS)
                        == Some(assignment.effective_month),
                "invalid assignment identity or date"
            );
            let (owner, effective) = owners
                .get(&request.loan)
                .copied()
                .unwrap_or((loan.terms.lender, loan.opened_month));
            ensure!(
                request.month >= effective
                    && request.from == owner
                    && request.from != request.to
                    && request.to != loan.terms.borrower,
                "conflicting or unauthorized claim assignment"
            );
            let authorized = match assignment.basis {
                Basis::Consent => {
                    request.owner_consent == Some(owner)
                        && request.recipient_consent == Some(request.to)
                }
                Basis::OperatorEstate {
                    household,
                    closed_month,
                } => {
                    matches!(request.from, Account::Operator(_))
                        && request.to == Account::Household(household)
                        && closed_month <= request.month
                        && request.owner_consent.is_none()
                        && request.recipient_consent.is_none()
                }
                Basis::InstitutionEstate { site } => {
                    matches!(request.from, Account::Institution(_))
                        && request.to == Account::Town(site)
                        && request.owner_consent.is_none()
                        && request.recipient_consent.is_none()
                }
            };
            ensure!(authorized, "invalid assignment authority");
            owners.insert(request.loan, (request.to, assignment.effective_month));
            previous_month = request.month;
        }
        Ok(())
    }

    /// Commit a consensual, whole-claim gift for next month's boundary. This pure
    /// primitive does not authorize an estate distribution or mutate any account.
    /// The caller must resolve actual consent, legal identities and event evidence.
    pub fn assign(&mut self, loans: &[Loan], month: u32, request: Request) -> Result<u64> {
        self.assign_with_basis(loans, month, request, Basis::Consent)
    }

    pub(super) fn assign_with_basis(
        &mut self,
        loans: &[Loan],
        month: u32,
        request: Request,
        basis: Basis,
    ) -> Result<u64> {
        self.validate(loans, month)?;
        ensure!(request.month == month, "assignment requires current month");
        let loan = loans
            .get(request.loan as usize)
            .filter(|loan| loan.id == request.loan)
            .context("missing assignment loan")?;
        ensure!(
            matches!(
                loan.status,
                Status::Performing | Status::Arrears | Status::Defaulted
            ),
            "settled loan has no assignable claim"
        );
        let effective_month = month
            .checked_add(ASSIGNMENT_NOTICE_MONTHS)
            .context("assignment month overflow")?;
        let sequence = self.assignments.len() as u64;
        let mut candidate = self.clone();
        candidate.assignments.push(Assignment {
            basis,
            sequence,
            request,
            effective_month,
        });
        candidate.validate(loans, month)?;
        *self = candidate;
        Ok(sequence)
    }
}

impl crate::civilization::History {
    pub fn credit_owner_at(&self, loan: u64, month: u32) -> Result<Account> {
        let contract = self
            .credit
            .loans
            .get(loan as usize)
            .filter(|l| l.id == loan)
            .context("missing ownership loan")?;
        self.credit.ownership.owner_at(contract, month)
    }

    pub(crate) fn validate_credit_ownership(&self) -> Result<()> {
        self.credit
            .ownership
            .validate(&self.credit.loans, self.month)?;
        for assignment in self.credit.ownership.assignments() {
            let request = &assignment.request;
            for party in [request.from, request.to] {
                let cash = self.settlement_balance(party)?.value();
                ensure!(
                    cash.is_finite() && cash >= 0.,
                    "invalid claim-owner account"
                );
            }
            if let Some(cause) = request.cause {
                ensure!(
                    self.events
                        .get(cause as usize)
                        .is_some_and(|e| e.id == cause && e.month <= request.month),
                    "invalid assignment cause"
                );
            }
        }
        Ok(())
    }

    /// Explicit consent from both owning policies is required. Only operating
    /// accounts can make voluntary gifts; estate distributions are a separate policy.
    /// Accounts with live borrowing claims or unrecovered defaults cannot give away assets.
    pub fn assign_credit_claim(&mut self, request: Request) -> Result<u64> {
        self.validate_credit()?;
        for party in [request.from, request.to] {
            let cash = if matches!(party, Account::Household(_)) && party == request.to {
                if let Account::Household(id) = party {
                    ensure!(
                        !self
                            .society
                            .as_ref()
                            .context("missing household society")?
                            .relocation
                            .lost_households
                            .contains(&id),
                        "lost household cannot accept a new claim"
                    );
                }
                self.settlement_balance(party)?.value()
            } else {
                self.credit_account_cash(party)?
            };
            ensure!(
                cash.is_finite() && cash >= 0.,
                "invalid assignment account cash"
            );
        }
        ensure!(
            !self.credit.account_has_debt(request.from),
            "indebted account cannot give away creditor assets"
        );
        if let Some(cause) = request.cause {
            ensure!(
                self.events
                    .get(cause as usize)
                    .is_some_and(|e| e.id == cause && e.month <= request.month),
                "invalid assignment cause"
            );
        }
        let (loan, from, to) = (request.loan, request.from, request.to);
        let sequence = self
            .credit
            .ownership
            .assign(&self.credit.loans, self.month, request)?;
        let effective = self.credit.ownership.assignments()[sequence as usize].effective_month;
        self.record_credit_event(loan, "loan_assigned", format!(
            "Creditor claim on loan {loan} assigned by consent from {from:?} to {to:?}, effective month {effective}; no cash transferred and original terms retained."));
        Ok(sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credit::{RepaymentSource, Terms, SHARED_CURRENCY};

    fn loan() -> Loan {
        Loan::record_disbursement(
            0,
            Terms {
                lender: Account::Town(0),
                borrower: Account::Town(1),
                currency: SHARED_CURRENCY,
                source: RepaymentSource::Export {
                    contract: 0,
                    payment_month: 12,
                },
                annual_simple_rate: 0.12,
                maturity_month: 12,
                grace_months: 2,
            },
            0,
            100.,
        )
        .unwrap()
    }
    fn request(id: u64, month: u32, from: Account, to: Account) -> Request {
        Request {
            id,
            loan: 0,
            month,
            from,
            to,
            owner_consent: Some(from),
            recipient_consent: Some(to),
            cause: None,
        }
    }

    #[test]
    fn dated_successors_preserve_contract_and_serialized_boundaries() {
        let loans = vec![loan()];
        let original = serde_json::to_value(&loans).unwrap();
        let mut owners = Ownership::default();
        assert_eq!(owners.owner_at(&loans[0], 0).unwrap(), Account::Town(0));
        owners
            .assign(
                &loans,
                4,
                request(10, 4, Account::Town(0), Account::Council(2)),
            )
            .unwrap();
        assert_eq!(owners.owner_at(&loans[0], 4).unwrap(), Account::Town(0));
        assert_eq!(owners.owner_at(&loans[0], 5).unwrap(), Account::Council(2));
        let mut resumed: Ownership =
            serde_json::from_value(serde_json::to_value(&owners).unwrap()).unwrap();
        for world in [&mut owners, &mut resumed] {
            world
                .assign(
                    &loans,
                    5,
                    request(11, 5, Account::Council(2), Account::Institution(3)),
                )
                .unwrap();
            world.validate(&loans, 5).unwrap();
            assert_eq!(world.owner_at(&loans[0], 4).unwrap(), Account::Town(0));
            assert_eq!(world.owner_at(&loans[0], 5).unwrap(), Account::Council(2));
            assert_eq!(
                world.owner_at(&loans[0], 6).unwrap(),
                Account::Institution(3)
            );
        }
        assert_eq!(
            serde_json::to_value(owners).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        assert_eq!(original, serde_json::to_value(loans).unwrap());
    }

    #[test]
    fn rejected_decisions_leave_ownership_unchanged() {
        let loans = vec![loan()];
        let mut owners = Ownership::default();
        owners
            .assign(
                &loans,
                4,
                request(10, 4, Account::Town(0), Account::Council(2)),
            )
            .unwrap();
        let before = serde_json::to_value(&owners).unwrap();
        let mut no_consent = request(11, 5, Account::Council(2), Account::Town(3));
        no_consent.owner_consent = None;
        let mut no_acceptance = no_consent.clone();
        no_acceptance.owner_consent = Some(Account::Council(2));
        no_acceptance.recipient_consent = None;
        for bad in [
            request(10, 5, Account::Council(2), Account::Town(3)),
            request(11, 4, Account::Town(0), Account::Town(3)),
            request(11, 5, Account::Town(0), Account::Town(3)),
            request(11, 5, Account::Council(2), Account::Town(1)),
            request(11, 5, Account::Council(2), Account::Council(2)),
            request(11, 6, Account::Council(2), Account::Town(3)),
            no_consent,
            no_acceptance,
        ] {
            assert!(owners.assign(&loans, 5, bad).is_err());
            assert_eq!(before, serde_json::to_value(&owners).unwrap());
        }
        assert!(owners
            .assign(
                &loans,
                u32::MAX,
                request(12, u32::MAX, Account::Council(2), Account::Town(3))
            )
            .is_err());
        assert_eq!(before, serde_json::to_value(&owners).unwrap());
    }

    #[test]
    fn loaded_corrupt_chains_are_rejected() {
        let loans = vec![loan()];
        let mut owners = Ownership::default();
        owners
            .assign(
                &loans,
                4,
                request(10, 4, Account::Town(0), Account::Town(2)),
            )
            .unwrap();
        let pristine = serde_json::to_value(&owners).unwrap();
        for (field, value) in [
            ("effective_month", 4),
            ("effective_month", 6),
            ("sequence", 1),
        ] {
            let mut bad = pristine.clone();
            bad["assignments"][0][field] = value.into();
            let loaded: Ownership = serde_json::from_value(bad).unwrap();
            assert!(loaded.validate(&loans, 4).is_err());
        }
        assert!(owners.validate(&loans, 3).is_err());
        assert!(owners.validate(&[], 4).is_err());
    }
}
