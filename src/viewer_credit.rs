use crate::credit::{state::Credit, Account, RepaymentSource, Status};
use eframe::egui;

const VISIBLE_CREDIT_RECORDS: usize = 32;

fn account_name(account: Account) -> String {
    let (kind, id) = match account {
        Account::Town(id) => ("Town", id),
        Account::Council(id) => ("Council", id),
        Account::Institution(id) => ("Institution", id),
        Account::Operator(id) => ("Operator", id),
    };
    format!("{kind} #{id}")
}
fn status_name(status: Status) -> &'static str {
    match status {
        Status::Performing => "Performing",
        Status::Arrears => "In arrears",
        Status::Repaid => "Repaid",
        Status::PrecisionSettled => "Settled with rounding residue",
        Status::Defaulted => "Defaulted",
    }
}
fn source_name(source: RepaymentSource) -> String {
    match source {
        RepaymentSource::AnnualTax {
            council,
            collection_month,
        } => format!("Council #{council} tax collection in month {collection_month}"),
        RepaymentSource::Export {
            contract,
            payment_month,
        } => format!("Export contract #{contract}, payment expected in month {payment_month}"),
        RepaymentSource::ServiceOrder {
            order,
            payment_month,
        } => format!("Service order #{order}, payment expected in month {payment_month}"),
    }
}

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
            ui.collapsing(format!("Loan {} · {} · original lender {} · borrower {}", r.id, status_name(r.status), account_name(r.lender), account_name(r.borrower)), |ui| {
                ui.label(format!("Currency {} · opened month {} · maturity {} · annual simple rate {:.2}%",
                    r.currency.0, r.opened_month, r.maturity_month, r.annual_simple_rate * 100.));
                for assignment in &r.assignments {
                    ui.small(format!("Claim assigned from {} to {}, effective month {}",
                        account_name(assignment.request.from), account_name(assignment.request.to), assignment.effective_month));
                }
                ui.label(format!("Repayment evidence: {}", source_name(r.source)));
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
                    if let Some(c) = &grant.capacity {
                        ui.small("Capacity at the decision boundary (available / competing demand):");
                        ui.small(format!("Lender principal: {:.4} / {:.4}", c.lender_principal, c.lender_demand));
                        ui.small(format!("Borrower principal limit: {:.4} / {:.4}", c.borrower_principal, c.borrower_demand));
                        ui.small(format!("Repayment source: {:.4} / {:.4}", c.source_receipts, c.source_demand));
                    }
                }
            }
        });
        ui.collapsing("Recent issuance receipts", |ui| {
            for r in credit.issuance.receipts.iter().rev().take(VISIBLE_CREDIT_RECORDS) {
                ui.label(format!("Month {} · issuer {} · currency {} · requested {:.4} · permitted {:.4} · issued {:.4}",
                    r.month, r.issuer, r.currency.0, r.requested, r.permitted, r.issued));
                ui.small(format!("Council cash {:.4} to {:.4}", r.opening_treasury, r.closing_treasury));
            }
        });
    });
}
