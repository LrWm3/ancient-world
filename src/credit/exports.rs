//! Repayment evidence from buyer-funded, still-traveling export payments.
use super::{underwriting::Evidence, Account, RepaymentSource};
use crate::civilization::History;
use anyhow::{ensure, Result};
use std::collections::BTreeMap;

impl History {
    /// Only the actual town payee can pledge these proceeds. Callers must add
    /// their operating commitments before underwriting a working-capital loan.
    /// The risk haircut is an explicit experiment input, not observed physiology
    /// or a promise that the cargo will survive.
    pub fn export_credit_evidence(&self, expected_loss_fraction: f64) -> Result<Vec<Evidence>> {
        ensure!(
            expected_loss_fraction.is_finite() && (0. ..=1.).contains(&expected_loss_fraction),
            "invalid export credit loss assumption"
        );
        self.validate_export_identities()?;
        self.validate_export_payments()?;
        let mut sources = BTreeMap::<RepaymentSource, Evidence>::new();
        for cargo in &self.cargo {
            let Some(id) = cargo.export_payment else {
                continue;
            };
            let payment = &self.export_payments[id as usize];
            // Do not roll a delayed payment into a fresh source date. Existing
            // loans may have pledged the original source already. Its due date
            // remains immutable; overdue/known-delayed cargo cannot back new loans.
            if payment.expected_month <= self.month
                || cargo.arrives != payment.expected_month
                || self.sites[payment.seller as usize].abandoned
            {
                continue;
            }
            let receipts = payment.escrow * f64::from(cargo.kg) / f64::from(payment.original_kg)
                * (1. - expected_loss_fraction);
            if receipts <= 0. {
                continue;
            }
            let source = RepaymentSource::Export {
                contract: payment.contract,
                payment_month: payment.expected_month,
            };
            let evidence = sources.entry(source).or_insert(Evidence {
                source,
                beneficiary: Account::Town(payment.seller),
                observed_month: self.month,
                expected_receipts: 0.,
                operating_costs: 0.,
                expected_loss_fraction,
            });
            evidence.expected_receipts += receipts;
        }
        Ok(sources.into_values().collect())
    }
}
