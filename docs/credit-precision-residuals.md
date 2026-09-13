# Long-run credit finding: numerical residue classified as default

The fixed `0aa337c` executable's 500-year seed-81 credit arm completed with two
loans and two defaults. This diagnosis comes from the completed four-arm
experiment summarized in [the long-run follow-up](shared-issuance-century.md#longer-seed-81-follow-up-500-years). Both loans used actual export
receivables and the same borrower/lender pair (town 0 borrowing from town 4).

| Loan | Open month | Principal transferred | Due at maturity | Actually paid at maturity | Written-off principal | Written-off interest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 2647 | 2.3724365234 | 2.5347611277 | 2.5346679688 | 0.0000931589 | 0.0000063740 |
| 1 | 2719 | 1.3596191406 | 1.4526457134 | 1.4526367188 | 0.0000089947 | 0.0000006154 |

The borrowers had 187.32 and 155.15 cash at maturity. The collection allowance
covered the entire due amount. Exact transfers between the account representations
paid almost all of it; subsequent tiny transfers returned zero even while cash
remained available. Three months later, ordinary grace expiry wrote off the
remaining claims and classified each loan as Defaulted. The standard 60-month
credit exclusion then applied.

This is an accounting/classification mismatch, not evidence that these borrowers
could not afford repayment. The original claim and remaining write-off are real
ledger quantities, so silently marking the whole amount paid would be wrong.
A fix should explicitly record a bounded numerical-residue settlement without
inventing transferred cash, and distinguish it from insolvency for exclusion and
reports. It must check actual transfer granularity and affordability, cap both
absolute and relative residue, and leave genuine unpaid amounts subject to default.

The credit world ended with 127.05 people versus 180.17 in the baseline. That
long-run difference does not identify the effect of the classification bug alone;
loan cash flows and subsequent decisions also differ. A matched rerun after the
fix is needed. The tested executable contains the classification bug and predates the newer
restructuring caller. Raw loans, receipts and experiment logs remain under ignored
`output/monetary-four-arm-five-century/`.

## Correction: explicit precision settlement

Monthly servicing now has a separate `PrecisionSettled` terminal status and a
`PrecisionWriteOff` ledger entry. After normal collection, a remaining claim can
qualify only if it is no more than 0.001 shared-currency unit **and** 0.0001 of
original principal (0.01%). It must fit inside the unspent collection allowance,
the borrower must still have the money, and the existing exact account-transfer
quote must return zero. These are explicit numerical-handling limits, not new
sources of spendable currency or a general debt-forgiveness policy.

The forgiven principal and interest remain recorded separately from repayments.
The collection receipt records the precision-settled amount; the ordinary cash
transfer ledger contains only real transfers. The terminal loan does not accrue
further interest or trigger default-based credit exclusion. Material shortfalls,
a wholly unpaid small loan, unavailable money, and budget-protected money do not
qualify. Old archived defaults retain their recorded interpretation; compare fresh
runs rather than retroactively rewriting financial history.

This change addresses classification and tiny claims only. It does not establish
that credit improves work or population, and it does not erase genuine lender
losses from the accounting. Matched long-run evaluation must distinguish cash
repayment, precision settlement and insolvency/default explicitly.

## Verification

The market suite passed 13 active tests (two hardware tests remained ignored),
including an exact cash-invariance and serialized-continuation fixture for
precision settlement. Matched controls with an empty account or a protected
collection budget still default. Removing the settlement receipt is rejected by
archive validation. All 15 credit unit tests passed; strict all-target Clippy and
the development executable build passed. These fixtures verify the correction's
accounting and boundaries, not its long-run balance effects.

## Fresh-code long-run comparison

A fresh seed-81 four-arm, 500-year run uses the fixed executable from `01d502b`
and the same starting checkpoint and delivery-paid-export policy. The baseline
completed at 180.166604 residents, with no loans or issuance. Its full serialized
history matches the older baseline after allowing the newly added empty
`credit.restructurings` field. The other arms are still running. The executable
also contains newer term-aware underwriting and delayed-export negotiation; this
is an integrated comparison, not an isolated precision-settlement ablation.
