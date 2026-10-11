//! Observed process completions from committed batches, not work reservations.
use crate::model::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Completion {
    pub month: u32,
    pub process: u64,
    /// Operator at completion, which need not be the output beneficiary.
    pub operator: AgentId,
    pub definition: DefinitionId,
}

/// Inclusive month window over a trusted committed ledger. Includes consumption
/// as well as production; consumers can select definitions. Does not inspect
/// nested production plans or infer completion from closing state. Repeated
/// observations of a process are represented once, at its earliest completion.
/// Reversed intervals return no rows.
/// This is a diagnostic reader, not a substitute for settlement validation.
pub fn observe(ledger: &[Batch], operator: AgentId, from: u32, through: u32) -> Vec<Completion> {
    let mut completed = BTreeMap::<u64, Completion>::new();
    for batch in ledger {
        for change in batch.all_transactions().filter_map(|t| t.process.as_ref()) {
            let p = &change.after;
            if p.operator != operator
                || p.status != Status::Completed
                || change
                    .before
                    .as_ref()
                    .is_some_and(|p| p.status == Status::Completed)
            {
                continue;
            }
            let entry = completed.entry(p.id).or_insert(Completion {
                month: batch.month,
                process: p.id,
                operator: p.operator,
                definition: p.definition,
            });
            entry.month = entry.month.min(batch.month);
        }
    }
    let mut rows: Vec<_> = completed
        .into_values()
        .filter(|c| (from..=through).contains(&c.month))
        .collect();
    rows.sort_by_key(|c| (c.month, c.process));
    rows
}
