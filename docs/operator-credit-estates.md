# Closed workshop accounts and credit

A closed workshop retains its identity and releases its equipment lease. If it
has live borrowing obligations or unrecovered default losses, closure retains its cash instead of paying that
cash to its owner. A debt-free closure keeps the existing immediate return of
cash. A closed creditor can receive payments on its existing loans; it cannot
issue a new loan or resume production.

`credit::estates` resolves closed operator and [institutional](institution-credit-estates.md) accounts using the existing cash,
financing and capital-return ledgers. At each settlement window it:

1. Observes opening estate cash and accrues existing borrowing obligations to
   the current month.
2. Divides that opening cash proportionally across live borrowing claims, up to
   each amount owed. This includes unmatured principal because the existing
   loan contract allows early repayment without penalty.
3. Commits actual payments using the same exact-transfer adapter as ordinary
   credit. Unavailable creditors do not cause their share to go to the owner.
4. Returns remaining cash only when the estate has neither live borrowing claims nor unrecovered default losses.
   Future repayments to a closed creditor pass through its retained account and
   can subsequently reach the same existing household owner.

Plans use opening cash, so a payment received during a pass cannot be respent by
another estate in that same pass just because its ID sorts later. Payments and
owner distributions remain separate transfers; claims are never spendable cash.
Loan IDs, original parties, repayment receipts, operator financing counters and
household capital-return counters preserve provenance. Eligible receivables now follow [dated estate succession](credit-claim-succession.md)
to the existing owner household from the next month. A household does not inherit
personal liability for an operator's unpaid debt. Loss-bearing estates retain
their cash and claims pending repayment/recovery; default does not bypass priority.

## Monthly boundary

The existing five-phase schedule remains in place. Estate settlement runs before
and after ordinary Open credit collection, after Reserve operator preparation
(which can close a lease), and after Execute/settle operator accounting (which can
close an unviable firm). The first Open pass gives an already closed debtor's
retained cash a chance to repay before ordinary grace-period default. The second
handles cash received through that month's ordinary collection. Later closure
windows can only spend cash then available; they do not restore production time.

An insolvent estate does not automatically erase its remaining debt at closure.
It retains the original maturity, interest and grace/default rules. Incoming
receivables can therefore still service its obligations. A defaulted claim keeps
its loss record. A separate [explicit recovery API](credit-default-recovery.md)
can return authorized cash after default; this estate pass does not automatically
allocate recovery payments or implement general bankruptcy priority.
Institutional closure uses the same pass, with a separate exclusion for
institutions temporarily inactive during relocation.

## Verification

The hardware-backed operator fixture constructs a finite workshop and exercises
ample-cash closure, insufficient cash shared equally between equal claims, refusal
of new borrowing by a closed operator, later repayment to a closed creditor,
conserved cash, ledger validation and serialized continuation. A full GPU month
also checks Reserve closure, repayment before owner distribution, no subsequent
operator work, and identical checkpoint continuation. The service-order
sources are supplied test contracts, not automatic underwriting evidence.

The focused hardware fixture passed, including the full scheduled month and
checkpoint equality. Its later-payment assertion uses the actual receipt amount:
the mixed `f32` town / `f64` operator transfer can be slightly below a requested
10 units, and only those transferred units may reach the owner. The test does
not round the claim into fabricated cash.

```sh
cargo test --lib operator_estates_pay_claims_before_owners_and_receive_later_repayments -- --ignored --nocapture
```

Regression verification also passed: all 15 CPU market tests (two unrelated GPU
market tests excluded), the existing six-month funded/unfunded GPU workshop and
checkpoint comparison, and strict all-target Clippy. These are focused execution
checks; long-run credit balance and broader legal succession remain separate work.
