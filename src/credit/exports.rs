//! Repayment evidence from buyer-funded, still-traveling export payments.
use super::{underwriting::Evidence, Account, RepaymentSource};
use crate::civilization::History;
use anyhow::{ensure, Context, Result};
use std::collections::BTreeMap;

/// A forecast date changes; the pledged receivable identity does not.
#[derive(Clone, Debug)]
pub struct DelayedReceipt {
    pub evidence: Evidence,
    pub expected_payment_month: u32,
}

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
    /// Revised evidence only for an existing export loan. Observing delayed
    /// proceeds here does not make them eligible collateral for a fresh loan.
    pub fn export_restructuring_evidence(
        &self,
        loan_id: u64,
        loss: f64,
    ) -> Result<Option<DelayedReceipt>> {
        ensure!(
            loss.is_finite() && (0. ..=1.).contains(&loss),
            "invalid restructuring loss assumption"
        );
        self.validate_export_identities()?;
        self.validate_export_payments()?;
        let loan = self
            .credit
            .loans
            .get(loan_id as usize)
            .filter(|l| l.id == loan_id)
            .context("missing export loan")?;
        let RepaymentSource::Export {
            contract,
            payment_month,
        } = loan.terms.source
        else {
            return Ok(None);
        };
        let Account::Town(seller) = loan.terms.borrower else {
            return Ok(None);
        };
        if self.sites.get(seller as usize).is_none_or(|s| s.abandoned) {
            return Ok(None);
        }
        let mut receipts = 0.;
        let mut arrival = self.month;
        for cargo in &self.cargo {
            let Some(id) = cargo.export_payment else {
                continue;
            };
            let payment = &self.export_payments[id as usize];
            if payment.contract != contract
                || payment.expected_month != payment_month
                || payment.seller != seller
                || cargo.arrives <= self.month
                || self.siege_blocks_cargo(cargo)
                || self.freight_path_flooded(&cargo.freight_edges)
                || self.flood_blocks_delivery(cargo.from, cargo.to, cargo.sea_lane)
            {
                continue;
            }
            // Staffing-based voyage estimates assume future crew funding. Do not
            // treat an unfinished vessel voyage as assured repayment evidence.
            if cargo
                .voyage_clock
                .as_ref()
                .is_some_and(|c| c.remaining > 0.)
            {
                continue;
            }
            let amount =
                payment.escrow * f64::from(cargo.kg) / f64::from(payment.original_kg) * (1. - loss);
            if amount > 0. {
                receipts += amount;
                arrival = arrival.max(cargo.arrives);
            }
        }
        ensure!(receipts.is_finite(), "restructuring receipts overflow");
        if receipts <= 0. {
            return Ok(None);
        }
        Ok(Some(DelayedReceipt {
            evidence: Evidence {
                source: loan.terms.source,
                beneficiary: loan.terms.borrower,
                observed_month: self.month,
                expected_receipts: receipts,
                operating_costs: self.commercial_input_costs()[seller as usize],
                expected_loss_fraction: loss,
            },
            expected_payment_month: arrival,
        }))
    }
}
