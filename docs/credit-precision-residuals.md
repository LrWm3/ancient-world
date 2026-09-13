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
`credit.restructurings` field. All four arms subsequently completed; results are summarized below. The executable
also contains newer term-aware underwriting and delayed-export negotiation; this
is an integrated comparison, not an isolated precision-settlement ablation.

## Follow-up: a smaller loan exceeded the forgiveness cap

The `01d502b` credit arm ended with 65.578561 residents, three loans, two
precision settlements totaling 0.000138775 and one recorded default. The default
was not insolvency: a 0.047973633 loan left 0.000108578 principal after maturity
payment. The borrower held 187.83 cash then and 33.41 at grace expiry. Its
collection allowances covered the remaining claim, but exact transfers returned
zero. The residue exceeded the relative forgiveness cap, so it accrued three
further interest increments and defaulted. Removing the first loan's exclusion
had allowed this additional small loan to originate. This is evidence of a
remaining numerical classification gap, not a controlled estimate of credit's
population effect.

Servicing now distinguishes affordable blocked collection from forgiveness.
When the unspent collection allowance and real cash cover the claim but the
exact transfer adapter returns zero, a claim outside the forgiveness caps remains
Arrears with a `precision_blocked` receipt. It is not written off or classified
as a default. Ordinary contractual interest and subsequent monthly collection
continue; future representable payments can reduce it. If actual funds or the
collection allowance later become insufficient, normal default rules apply.
The bounded `PrecisionSettled` path is unchanged. Older archived defaults retain
their historical status.

A zero-interest claim can remain blocked indefinitely while balances remain
incompatible. Reporting must retain that outstanding debt rather than count it
as paid. This is a limitation of exact transfers between mixed account precisions;
it is not an unlimited forgiveness or refinancing mechanism. Experiment summaries
now report blocked collection receipts and total outstanding debt as well as
actual defaults and precision settlements. The count is monthly receipts, not
unique loans. The `01d502b` fixed executable does not contain this follow-up fix.

The issuance-only arm completed with 182.069570 residents and 1,250 issued,
matching the older arm's population and issuance totals. No loans originated.
These terminal values do not by themselves establish improved work or food access.

Follow-up verification: all 14 active market tests passed (two hardware tests
were not selected). The new controlled small-loan fixture remains in Arrears with
an explicit blocked-collection receipt through grace expiry, without forgiving
any of that claim. An emptied-borrower control defaults; changing actual lender
balances allows a later real payment. Serialized continuation matches and monetary
residuals remain within the fixture's 1e-12 tolerance. Contradictory blocked/default
receipts are rejected. The completed long-run evaluation appears below.

Strict all-target Clippy and the executable build also passed for the blocked-
collection follow-up.

## Completed integrated comparison (`01d502b`)

| Arm | Ending residents | Loans | Recorded defaults | Precision settlements | Issued |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | 180.166604 | 0 | 0 | 0 | 0 |
| Credit | 65.578561 | 3 | 1 | 2 | 0 |
| Issuance | 182.069570 | 0 | 0 | 0 | 1,250 |
| Combined | 182.069570 | 0 | 0 | 0 | 1,250 |

All four processes completed successfully and performed their native history
validation. Issuance and combined histories differ only in the credit subtree;
credit did not add a realized loan or observable non-credit outcome there.
The credit-only default remains the precision-cap case described above. This
comparison does not establish that credit improves useful work or that issuance
passes the wider balance gate. Elapsed times were 254.27, 222.96, 211.54 and
235.69 seconds respectively, with overlapping compilation/test work; these are
run records, not controlled performance measurements.

A fresh matched run uses fixed executable `d50bceb`, including affordable blocked
collection, with the same seed/checkpoint, 500 years and four policy arms. Raw
results are under ignored `output/monetary-blocked-five-century/`. All four arms completed; summaries expose retained debt and blocked monthly receipts.

## Completed blocked-collection comparison (`d50bceb`)

The baseline completed at 180.166604 residents with byte-identical serialized
history to the preceding baseline. The corrected credit arm completed at
200.028506 residents with eight loans, no defaults and zero outstanding debt.
Actual principal disbursed totaled 33.84619140625. Real repayments totaled
36.149658203125, including 2.303973148589 interest; separately recorded precision
write-offs totaled 0.000506351714. All eight loans eventually reached
PrecisionSettled. No restructuring occurred.

The small second loan remained precision-blocked for six receipts, months
2656–2661, and settled in month 2662. Its claim remained visible until actual
collection and the bounded final residue settlement. This is the controlled
classification mechanism behaving in the long run; it is not evidence of a
general credit benefit across seeds. Later lending and historical choices diverged,
so the population difference is not a measured work-productivity effect.
Native validation passed in all four arms.

| Arm | Ending residents | Loans | Defaults | Outstanding debt | Issued |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | 180.166604 | 0 | 0 | 0 | 0 |
| Credit | 200.028506 | 8 | 0 | 0 | 0 |
| Issuance | 182.069570 | 0 | 0 | 0 | 1,250 |
| Combined | 182.069570 | 0 | 0 | 0 | 1,250 |

Issuance and combined serialized histories differ only in the credit subtree.
Issuance's serialized history is byte-identical to the preceding issuance arm.
Elapsed times were 263.63, 259.11, 205.62 and 208.07 seconds, with overlapping
compilation/testing; these are not controlled performance benchmarks.

### Work and access in the corrected credit arm

| Measure | Baseline | Credit |
| --- | ---: | ---: |
| Cumulative operator work | 1,159.4584 | 1,150.6538 |
| Reported food production | 29,673,735 | 30,390,280 |
| Final need-weighted hunger | 0.02929 | 0.03620 |
| Cumulative council town support | 80,664.33 | 85,327.77 |
| Ending council cash | 1,144.58 | 3,719.75 |
| Ending household cash | 41,953.56 | 37,807.02 |
| Ending town cash | 5,703.84 | 7,273.32 |

Definitions match the earlier issuance reports: operator work sums firms'
completed work, food is the existing food-equivalent production ledger, hunger
weights household hunger by final monthly need, and town support sums council
`relief_paid`. These cash categories are separate reported owners, not a complete
money-supply audit. More surviving people and greater cumulative food production
coexist with slightly less operator work and worse terminal food access. Credit's
work/affordability gate is therefore still unproven. Controlled immediate
mediators, losses and held-out seeds remain required before Stage 2.

### Issuance and combined activity/access

Both arms have the same measures: 592.7647 cumulative operator work,
31,814,518 reported food production, 0.01937 terminal need-weighted hunger,
87,885.47 cumulative council town support, and ending cash of 123.86 in councils,
45,091.57 in households and 6,028.79 in towns. Relative to baseline, lower terminal
hunger accompanies substantially less operator work. This remains a mixed
outcome, not evidence of a general productive benefit from issuance.

The experiment runner now writes these activity/access measures into every
successful arm's summary, including terminal food need. A zero-need population
reports weighted hunger as null, not zero: an empty town/world cannot establish
successful food access. The report function was exercised against all four
completed archives and a zero-need control. Definitions deliberately distinguish
terminal access from cumulative production and do not count selected cash
categories as the complete monetary inventory.

Decision: the numerical-default correction is supported, but the Stage 2 gate
remains unproven. The next monetary evaluation needs immediate funded-work
mediators, account-closure handling and held-out worlds; increasing currency
complexity now would make the unresolved work/access effects harder to diagnose.
