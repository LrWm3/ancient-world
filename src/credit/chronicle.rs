//! Sparse committed monetary milestones. No historical backfill on archive load.
use super::{Account, EntryKind, Status};
use crate::civilization::History;

impl History {
    fn credit_event_site(&self, account: Account) -> Option<u32> {
        match account {
            Account::Town(id) => self.sites.get(id as usize).map(|s| s.id),
            Account::Council(id) => self
                .sites
                .iter()
                .find(|s| s.civilization == id && !s.abandoned)
                .map(|s| s.id),
            Account::Institution(id) => self
                .culture
                .as_ref()?
                .institutions
                .get(id as usize)
                .map(|i| i.site),
            Account::Operator(id) => self
                .enterprises
                .as_ref()?
                .firms
                .get(id as usize)
                .map(|f| f.site),
        }
    }
    pub(crate) fn record_credit_event(&mut self, loan: u64, kind: &str, detail: String) {
        let contract = &self.credit.loans[loan as usize];
        let site = self.credit_event_site(contract.terms.borrower);
        let other = self.credit_event_site(contract.terms.lender);
        let previous = self.credit.last_events.get(&loan).copied();
        self.event(kind, site, other, detail);
        let event = self.events.last_mut().unwrap();
        // Loan IDs are u64, while the existing subject index is u32. The persisted
        // last-event map preserves the full ID even beyond that subject range.
        if let Ok(id) = u32::try_from(loan) {
            event.subjects.push(("loan".into(), id));
        }
        event.causes = previous.into_iter().collect();
        self.credit.last_events.insert(loan, event.id);
    }
    pub(crate) fn record_credit_status(&mut self, id: u64, previous: Status) {
        let loan = &self.credit.loans[id as usize];
        if loan.status == previous {
            return;
        }
        let kind = match loan.status {
            Status::Arrears => "loan_arrears",
            Status::Repaid => "loan_repaid",
            Status::PrecisionSettled => "loan_precision_settled",
            Status::Defaulted => "loan_defaulted",
            Status::Performing => return,
        };
        if self
            .credit
            .last_events
            .get(&id)
            .and_then(|e| self.events.get(*e as usize))
            .is_some_and(|e| e.month == self.month && e.kind == kind)
        {
            return;
        }
        let written: f64 = loan
            .entries
            .iter()
            .filter(|e| matches!(e.kind, EntryKind::WriteOff | EntryKind::PrecisionWriteOff))
            .map(|e| e.principal + e.interest)
            .sum();
        let detail = format!("Loan {id} became {:?}; currency {}, principal due {:.6}, interest due {:.6}, recorded write-offs {:.6}. No cash is created by this status change.",loan.status,loan.terms.currency.0,loan.outstanding_principal,loan.interest_due,written);
        self.record_credit_event(id, kind, detail);
    }
}
