# Creditor succession: ownership boundary and implementation work

Status: consensual whole-claim assignments between existing operating account
types now connect to History, dated repayments/recoveries, restructuring consent,
lender exposure, explorer reports and archives. Household recipients and automatic
estate succession remain unfinished. Original accounts must remain resolvable
with finite nonnegative cash, including after closure and final settlement.

## Why a new owner field is insufficient

Current `Loan.terms.lender` serves several different purposes:

- Original disbursement party and underwriting evidence (`state`, `underwriting`).
- Payment destination and representation-precision checks (`state`, `servicing`).
- Estate creditor and post-default recipient (`estates`, `recovery`, `export_recovery`).
- Party consenting to restructuring (`restructuring`).
- Explorer identity, event association and historical receipt validation.

Overwriting it would invalidate original cash receipts. Updating only repayment
would leave recovery and consent with the old owner. Keep the original contract
immutable and introduce dated ownership separately.

## Ownership contract

Add an append-only assignment record: stable ID, loan ID, current claim owner,
successor, decision month, effective month, legal basis, authorization and causal
event. Assign the entire remaining creditor claim; partial claims and debt trading
are deferred. Assignment moves no cash, forgives no debt and creates no income.
It also transfers the right to recover the original recorded default loss, less
recoveries already received. Original losses remain visible under their original
creditor; subsequent recovery reports identify the actual recipient.

Initially allow explicit estate succession and consensual gratuitous transfers.
Neither implies personal liability for an operator's household. A debtor cannot
become its own creditor through this operation: forgiveness needs a separate,
explicit debt-write-off rule. Reject overlapping pending assignments and requests
from a party that does not own the claim. Keep closed estates until their records
and obligations have a resolved disposition.

Household beneficiaries need a **settlement-only** payment adapter. Adding a
household recipient must not make households eligible for new credit requests or
lender offers. Principal received through inheritance is inherited capital, not
wages, workshop sales or food production; interest needs its own transfer category.
Use the existing household cash store and extend its reconciliation rather than
introducing another balance.

## Timing and provenance

Schedule an accepted assignment for the following monthly Open boundary. This
makes all receipts within a month unambiguous and prevents a Respond decision from
redirecting an earlier payment. Ownership lookup uses `(loan, receipt_month)`,
including for old recoveries. A newly assigned creditor receives future payments;
previous transfers retain their original recipients. A pending assignment is
archived but cannot receive payments early.

Before inserting an assignment, validate the whole chain, retained legal identities,
recipient settlement eligibility, and effective date. No cash need be reserved.
Activation is a dated ownership lookup, not a second mutable balance or repeated
monthly transfer. Repeat requests must reject or return the original result without
adding another record. Issuance policy and borrower obligations are unchanged.

## Integration work, in dependency order

1. Add the assignment ledger and pure dated-owner lookup. Legacy histories have
   no assignments and resolve to the original lender. Validate chain continuity,
   stable IDs, bounded dates and genuine consent or recorded estate authority.
2. Add settlement-only household receipt accounting. Distinguish inherited claims
   from inherited cash; claims are never included in monetary supply.
3. Resolve current creditor for payments, precision checks, recovery and estate
   eligibility. Preserve original lender for origination and source evidence.
4. Resolve creditor consent for restructuring. Its receipt must retain the dated
   owner that consented; pure contract evaluation must not rewrite original terms.
5. Adjust lender exposure and offers to account for inherited claims. A receiving
   account cannot evade exposure limits through assignment; household settlement
   recipients remain excluded from new lending/borrowing.
6. Expose original lender, current creditor and assignment history in reports and
   events. Validate each cash receipt against its dated recipient, not today's
   owner. Connect existing operator/institution dissolution decisions only after
   these primitives pass controlled tests.

## Required evidence

Use a loan with an actual partial payment before succession and another afterward:
old and new recipients must receive exactly their respective transfers, with the
borrower's total debt reduction matching both. Include interest, recovery after
an earlier default, zero-cash successors, and incompatible f32/f64 balances.

Test two successive owners, unauthorized and duplicate assignment, assignment to
the debtor, missing beneficiaries, and same-month repayment followed by a next-
month assignment. Check restructuring consent before/after activation. Compare
uninterrupted and serialized continuation across the effective boundary. A firm
that both owes and owns claims cannot distribute assets ahead of its own creditors;
institution relocation preserving identity requires no assignment. Officeholder
turnover similarly leaves a council's identity unchanged.

The retained-account fixture already distinguishes a valid abandoned treasury from
a missing/rebound party and negative/nonfinite cash after full repayment. That is
an archive-integrity prerequisite, not evidence that the assignment path above is
implemented. No Stage 1 milestone or balance gate is completed by this document.

Verification for the retained-account change: all 16 CPU market tests passed
(two hardware tests remained ignored in that command), and strict all-target
Clippy passed. Existing continuation checks in the abandoned-treasury fixture
still pass. This change does not alter monthly transfer timing or balances.

## Dated ownership foundation

`Ownership` records append-only, whole-claim consensual assignments. Each request
has a stable replay ID, loan, decision month, current owner, recipient, both consent
identities and an optional causal reference. The ledger supplies a sequential
assignment ID and next-month effective date. Dated lookup preserves the original
lender through the decision month and preserves previous recipients after later
assignments. No loan terms or cash records are mutated.

Validation rejects duplicate requests, nonchronological decisions, missing loans,
wrong current owners, overlapping pending assignments, self-assignment, assignment
to the borrower, absent consent and invalid activation dates. Append validates a
candidate before replacing state; errors leave the original ledger untouched.
Loaded ledgers must be validated before lookup. Month overflow rejects safely.

The explicit `History::assign_credit_claim` API accepts consent supplied by each
account's owning policy; it does not generate a political decision or estate order.
It validates operating counterparties, optional causal references and the existing
ledger, and rejects transfers from an account with live borrowing claims.
Assignments activate next month through dated lookup; no new scheduler phase or
monthly transfer is introduced. `Credit.ownership` archives the chain, defaulting
to empty for old worlds. No automatic gift policy is enabled.

Payment, servicing precision checks, retained-estate settlement, default recovery
and automatic delayed-export recovery resolve the dated owner. Restructuring
receipts preserve both original terms and the consenting creditor at that month.
Underwriting uses a transient portfolio view to charge inherited exposure to its
current owner; persisted original loan terms remain unchanged. The explorer lists
assignment effective dates alongside the original lender. Household settlement
receipts and legal estate succession remain separate unfinished work.

Verification: all three ownership unit tests and strict all-target Clippy passed.
Tests cover two successive owners, next-month boundaries, serialized continuation,
unchanged original contracts, duplicate and unauthorized requests, overlapping
assignments, self/debtor recipients, overflow, missing loans and corrupted loaded
dates/IDs. They verify ledger semantics only; cash-routing integration fixtures
remain necessary before enabling this in History.

## Integrated assignment constraints

Only existing Town, Council, Institution and Operator accounts participate in
this first API. Closed estates can still receive already assigned payments, but
cannot initiate a voluntary gift. The estate policy must establish authority and
priority before distributing claims automatically. Merely moving an institution
or replacing an officeholder requires no assignment.

Original disbursements validate against original parties. Each repayment and
recovery validates against the creditor at its own month; relabeling an old receipt
with today's owner is invalid. Defaults and recovery totals stay attached to the
original loan, so changing creditor cannot reset the recoverable loss.

The integrated market fixture now checks actual partial payments before and after
activation, unchanged cash at assignment, duplicate rejection, same-backend JSON
continuation, preserved original terms, historical recipient validation and
post-default recovery to the new owner. It also compares former/current creditor
consent against identical opening states and shows inherited exposure exhausting a
low lender limit while a higher limit permits the same new request. All 17 CPU
market tests passed; the two hardware tests in that target were not run by this
command. Full-world balance effects of an automatic succession policy remain untested
because that policy is not enabled or implemented yet.

Verification for this integration: 17 CPU market tests, 13 export-contract tests
and 22 credit unit tests passed. The recipient-balance rejection fixture also
passed after the final preflight correction. An initial command named a nonexistent
`exports` integration target; it was corrected to `cargo test --lib export_contracts`,
which actually ran all 13 tests. This is controlled integration evidence, not a
new multi-seed balance claim.

Strict all-target Clippy passed after the final change.
