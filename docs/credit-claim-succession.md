# Creditor succession: ownership boundary and implementation work

Status: consensual whole-claim assignments between existing operating account
types now connect to History, dated repayments/recoveries, restructuring consent,
lender exposure, explorer reports and archives. Household settlement-only recipients and bounded automatic operator/institution
estate succession are supported. Original accounts must remain resolvable
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

Household beneficiaries use a **settlement-only** payment adapter. They cannot
originate loans, submit lending offers or debit their wallet through credit
operations. Actual receipts increase the existing household cash store and separate
`credit_principal_received` / `credit_interest_received` counters; household
reconciliation includes both. These are cash receipts on acquired claims, not wages,
workshop sales, dividends or food production. Assignment alone increments neither
wallet nor receipt counters. Old archives initialize the counters to zero.

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
receipts and bounded estate succession are now implemented.

Verification: all three ownership unit tests and strict all-target Clippy passed.
Tests cover two successive owners, next-month boundaries, serialized continuation,
unchanged original contracts, duplicate and unauthorized requests, overlapping
assignments, self/debtor recipients, overflow, missing loans and corrupted loaded
dates/IDs. They verify ledger semantics only; cash-routing integration fixtures
remain necessary before enabling this in History.

## Integrated assignment constraints

Existing Town, Council, Institution and Operator accounts can initiate consensual
assignments. A Household can receive a claim and its subsequent payments, but
cannot initiate a credit gift or new loan. Closed estates can still receive
already assigned payments, but cannot initiate a voluntary gift. The estate policy must establish authority and
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
and are not established by these controlled tests.

Verification for this integration: 17 CPU market tests, 13 export-contract tests
and 22 credit unit tests passed. The recipient-balance rejection fixture also
passed after the final preflight correction. An initial command named a nonexistent
`exports` integration target; it was corrected to `cargo test --lib export_contracts`,
which actually ran all 13 tests. This is controlled integration evidence, not a
new multi-seed balance claim.

Strict all-target Clippy passed after the final change.

## Household payment boundary

An accepting household must have an existing stable household record and wallet;
no account is silently created. A household recorded lost in relocation cannot
accept a new claim or consent to a restructuring. Payments on an existing claim
can still reach its retained wallet; the existing household closure/retail path
returns lost-household cash through its `estate_returned` ledger. Vacancy or a
new household head does not change the household's account identity. Automatic
succession of the claim itself remains unimplemented.

Counter overflow, invalid receipt stocks and incompatible account representations
are checked before either wallet changes. Household food buying continues to use
its existing cash budget; no extra food, employment or political weight is awarded
by the credit adapter. Automatic estate distributions still require the separate
authority and priority policy described above.

Household verification: all 18 CPU market tests and strict all-target Clippy passed.
The new fixture checks zero cash at assignment, separate principal/interest receipts,
unchanged wages/dividends/relief, global money and household-wallet reconciliation,
serialized continuation, lost-recipient rejection, old counter defaults, household
creditor consent, and rejection of new household loans and credit debits. An initial
fixture used a vector operation on the lost-household set; this was corrected before
the passing run. Automatic estate succession and lost-household claim redistribution
are not covered by this fixture.

## Automatic closed-estate succession

The existing estate pass now schedules receivables as well as distributing cash.
A closed operator without borrowing obligations assigns eligible claims to its
existing owner household; an inactive institution assigns them to its home town.
The record distinguishes statutory estate authority from voluntary consent and
preserves the original account, beneficiary and next-month activation date.
It moves no cash and does not alter the borrower’s debt or original loan evidence.

Zero-cash estates are included. Repeated passes within the month do not duplicate
a pending assignment. Incoming payments during the decision month still reach the
estate; payments from the next month reach the beneficiary. Institutional relocation
is excluded, as are missing or lost household beneficiaries and transfers that
would make a debtor its own creditor. Those claims remain in their original account.

Both live borrowing debt and unrecovered default losses block distributions of
cash and claims. Default is not permission to give away assets ahead of creditors.
This same priority check applies to voluntary gifts. [Automatic cash recovery](estate-default-recovery.md)
now reduces retained losses through real transfers, sharing opening cash with live debt.
Lost-household claim succession and contested beneficiary selection remain outside
this policy. The authority record is a compact policy decision, not a reconstructed
legal proceeding or independent historical proof of every eligibility condition.

Verification: both explicitly enabled hardware estate fixtures passed (one test
each). The operator fixture covers zero-cash succession, decision-month ownership,
next-month household receipts, repeated-pass idempotence, serialized continuation,
cash retained after default and assignment permitted after an actual full recovery.
The institutional fixture checks next-month town receipt and relocation exclusion.
All 18 CPU market tests and 22 credit unit tests passed. The first broad estate
filter ran zero tests because those fixtures are ignored by default; the subsequent
explicit hardware invocations above supply the estate execution evidence.

Strict all-target Clippy also passed after the final fixture update.
