use crate::credit::state::Credit;
use eframe::egui;

const VISIBLE_CREDIT_RECORDS: usize = 32;

pub(super) fn panel(ui: &mut egui::Ui, credit: &Credit) {
    ui.collapsing("Credit and issuance (experimental)", |ui| {
        ui.small("Claims are not cash. Amounts below use each contract's currency; 0 is the shared currency.");
        ui.label(format!("New council loans: {} · commercial loans: {} · issuance: {} · late-export recovery: {}",
            credit.council_policy.enabled, credit.commercial_policy.enabled,
            credit.issuance.enabled, credit.export_recovery.policy.enabled));
        ui.label(format!("Authorized shared currency issued: {:.4}", credit.issuance.total_issued()));
        ui.small(format!("Showing latest {VISIBLE_CREDIT_RECORDS} contracts and issuance receipts. Full records remain in history exports."));
        let reports = credit.loan_reports();
        if reports.is_empty() { ui.label("No loans originated."); }
        for r in reports.iter().rev().take(VISIBLE_CREDIT_RECORDS) {
            ui.collapsing(format!("Loan {} · {:?} · {:?} → {:?}", r.id, r.status, r.lender, r.borrower), |ui| {
                ui.label(format!("Currency {} · opened month {} · maturity {} · annual simple rate {:.2}%",
                    r.currency.0, r.opened_month, r.maturity_month, r.annual_simple_rate * 100.));
                ui.label(format!("Repayment evidence: {:?}", r.source));
                ui.label(format!("Original principal {:.6} · principal due {:.6} · interest due {:.6}",
                    r.original_principal, r.due.principal, r.due.interest));
                for (label, amount) in [("Repaid", &r.repaid), ("Original default loss", &r.default_loss),
                    ("Precision write-off", &r.precision_writeoff), ("Post-default recovery", &r.recovered)] {
                    ui.label(format!("{label}: principal {:.6} · interest {:.6}", amount.principal, amount.interest));
                }
            });
        }
        ui.collapsing("Recent underwriting decisions", |ui| {
            for round in credit.rounds.iter().rev().take(VISIBLE_CREDIT_RECORDS) {
                ui.label(format!("Month {} · complete {}", round.month, round.complete));
                for grant in round.grants.iter().take(VISIBLE_CREDIT_RECORDS) {
                    ui.label(format!("Request {} · {:?} · requested {:.4} · eligible {:.4} · granted {:.4}",
                        grant.request, grant.decision, grant.requested, grant.eligible, grant.granted));
                    if let Some(capacity) = &grant.capacity { ui.small(format!("Capacity snapshot: {capacity:?}")); }
                }
            }
        });
        ui.collapsing("Recent issuance receipts", |ui| {
            for r in credit.issuance.receipts.iter().rev().take(VISIBLE_CREDIT_RECORDS) {
                ui.label(format!("Month {} · issuer {} · currency {} · requested {:.4} · permitted {:.4} · issued {:.4}",
                    r.month, r.issuer, r.currency.0, r.requested, r.permitted, r.issued));
                ui.small(format!("Council cash {:.4} → {:.4}", r.opening_treasury, r.closing_treasury));
            }
        });
    });
}
