# Explicit relief for overdue forward deliveries

Implemented for accepted prepaid forwards inside an authorized recovery proceeding.
An impossible delivery can now be resolved by an explicitly accepted date extension
or quantity write-off. The engine does not infer impossibility from a stock shortage,
lack of storage, or the estate's age. Without accepted terms, the residual obligation
continues to block closure.

## Accepted terms and execution

`recovery::Config.delivery_relief` contains dated `delivery_relief::Terms`: an ID,
proceeding, forward, original debtor and creditor, execution month, expected current
due date and remaining native quantity, and one action:

| Action | Effect |
| --- | --- |
| `Extend { due }` | Move the residual delivery to a future month; quantity and original advance remain unchanged. |
| `WriteOff { quantity }` | Release this positive quantity, up to the accepted expected remainder; no replacement cash claim is created. |

Configuration represents both parties' acceptance, as with direct lending. It is
not a negotiation algorithm or unilateral authority to forgive another creditor's
claim. Terms require an authorized case, an overdue expected date, and at most one
amendment per forward per month. The case must actually be active at execution.
The original parties, current date and exact residual must still match; otherwise
execution records rejection. Partial delivery between acceptance and execution
therefore requires revised consent rather than silently increasing the concession.

At **Due**, after ordinary collection and guarantee calls but before estate
distribution/closure, the adapter stages the amendment alongside the credit boundary.
Forward commodity transfers still execute at **Acquire**, using the resulting
effective due date and quantity. A valid extension at Due thus takes effect before
that month's delivery attempt. Relief may apply in the case's opening month if
all conditions hold. Future delivery alone still cannot establish insolvency.
No scheduler or allocation policy changes are involved.

The staged batch is exactly recomputed and validated before atomic publication.
Applied amendments stay on the original forward, including the actual delivered
quantity at application. Original due date, goods, advance and purchase acceptance
remain immutable. Checkpoint validation checks amendment identity, dated case,
chronology, consent, and quantity reconciliation:

```text
original promised quantity = actual delivered + explicitly written off + remaining
```

`claim()` expresses the legally remaining obligation; its generic settled counter
includes released units. The authoritative `delivered` field counts **only actual
commodity transfers**. No relief transaction creates goods, coins, storage use,
land-tax receipts or issuance. Shared agreement inspection, pledged quantities,
planning demands, receiving-storage reservations, plot eligibility and arrears
reports use the effective obligation.

A full write-off can remove the forward's closure blocker in the same Due. Partial
relief leaves the rest collectible and blocking; extending the date does not remove
the blocker. Loan deficiencies, unpaid land bills, unsold assets and other closure
conditions retain their existing rules. Later actual performance can complete a
partially released claim, with closure checked at the following Due.

`DeliveryRelief` receipts show the terms/forward/creditor, application status,
rejection reason, effective date, newly written-off quantity and remaining claim.
Settlement observers include these receipts when filtering by the forward creditor.
Rejected missing-contract receipts use zero remaining as a placeholder; the explicit
`MissingContract` reason distinguishes absence from successful fulfillment.

## Verification

Six added recovery controls exercise full release without fictional delivery or
issuance; extension followed by partial real delivery and partial release; stale
quantity/wrong-party rejection; invalid amounts/dates and atomic forged-batch or
checkpoint rejection; observational equivalence with creditor-filtered logs; and
fulfilled claims/inactive cases without phantom write-offs. Full release also
compares CubeCL CPU with reference execution and monthly versus batched continuation
from a saved state. Existing no-relief, storage-limited and future-maturity controls
remain in the recovery suite.

## Completed validation

**105 distinct tests passed across 11 affected suites in scoped runs**: acquisition
8, commitments 5, credit 9, finance 3, forwards 7, lending 18, loan views 5, recovery
26, resale 7, storage/currency 7 and telemetry 10. After adding rejection reasons
and fixing the inactive-case fixture to remove its orphaned coin-payment option,
the final recovery/telemetry run passed all 36 tests. Formatting, strict all-target
Clippy, repository artifact checks and changed Markdown link targets passed. The
full crate suite was not rerun; earlier full-suite totals are historical evidence.

## Remaining scope

Land-bill relief, commodity substitution, cash damages, automatic impossibility
assessment, autonomous renegotiation and court-imposed concessions remain absent.
These terms do not integrate recovery with household/town/minting drivers or enable
new tool purchases in the recovery servicing configuration. Non-loan relief is
currently a forward adapter, not a universal discharge rule. See
[admission boundaries](LAND-FORWARD-ADMISSION.md) and
[contract recovery](CONTRACT-RECOVERY.md).
