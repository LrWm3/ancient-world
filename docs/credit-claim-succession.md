# Creditor succession: ownership boundary and implementation work

Status: a standalone dated-ownership primitive now exists in
`src/credit/ownership.rs`. It is not yet connected to History, payments or archives;
world claims are not yet assignable. Retained estate settlement exists; The current validator now requires original borrower
and lender accounts to remain resolvable, with finite nonnegative cash, including
closed accounts and fully settled contracts. Closing an account must not erase its
historical identity.

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

This primitive checks consent identities, not whether a real actor or estate
was authorized to supply them. The History adapter must verify actual counterparties,
legal authority and causal event references before calling it. Household settlement
receipts, estate allocation, current-owner payments/recovery, exposure, consent
integration and world persistence remain required before enabling assignments.
There is deliberately no world switch or serialized History field accepting an
ownership chain that existing payment code would ignore.

Verification: all three ownership unit tests and strict all-target Clippy passed.
Tests cover two successive owners, next-month boundaries, serialized continuation,
unchanged original contracts, duplicate and unauthorized requests, overlapping
assignments, self/debtor recipients, overflow, missing loans and corrupted loaded
dates/IDs. They verify ledger semantics only; cash-routing integration fixtures
remain necessary before enabling this in History.
