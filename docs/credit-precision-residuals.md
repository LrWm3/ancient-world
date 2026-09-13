# Long-run credit finding: numerical residue classified as default

The fixed `0aa337c` executable's 500-year seed-81 credit arm completed with two
loans and two defaults. This is an interim diagnosis from the ongoing four-arm
experiment, not a completed long-run report. Both loans used actual export
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
fix is needed. The currently running executable contains the classification bug and predates the newer
restructuring caller. Raw loans, receipts and experiment logs remain under ignored
`output/monetary-four-arm-five-century/`.
