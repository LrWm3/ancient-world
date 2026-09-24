# Land dues and forward admission to recovery

Implemented in the existing recovery lifecycle. A debtor with land dues or accepted
prepaid deliveries can now enter an authorized proceeding. No new debt ledger,
scheduler, implied exchange rate or synthetic delivery is introduced.
`recovery_claims::outstanding` derives namespaced claims, creditors, due dates and
remaining native quantities from land bills and forward records.

## Admission and performance

A dated authorization can open on previously recorded unpaid land bills or an
accepted overdue forward, as well as loan arrears. A future delivery alone does
not establish arrears. Admission records existing unpaid land bills and all
accepted undelivered forwards, including future maturities. Future annual rent
is not fabricated at admission: bills continue on the agreement's existing
calendar. New land acceptance is blocked during an active proceeding.

| Claim | Collection source and boundary | Record and consequence |
| --- | --- | --- |
| Loan | Opening estate coins at Due, after its lien reservation | Same loan book; stay and frozen interest; existing explicit discharge policy |
| Native land goods | Debtor's unprotected goods at Due and the existing ClearArrears retry | Same annual bill; actual native receipts count toward issuance; unpaid amounts block new use |
| Land payable in estate coins | Estate opening cash at Due, either native or an explicitly accepted alternative | Shared ranked/proportional pool with loans; whole claim units; alternative payment does not count as native collection |
| Prepaid forward | Debtor's unprotected promised commodity at its existing Acquire boundary, only from maturity | Same delivered quantity; finite stock and creditor storage; outstanding delivery still blocks another advance |

For land with a coin alternative, native goods settle first, then residual claim
units enter the estate cash pool at the agreed rate. The cash sweep includes those
eligible land claims. New deposits still wait for another Due boundary, and actual
collateral proceeds retain their prior lien. Cash grants are ceilings, not
payments: the shared executor commits the funded legs and updates the original
bills oldest first.

A forward's advance price is **not** permission to repay in coins. Its goods are
not converted to a loan, accelerated, or marked delivered by admission. Its
performance collection remains separate from the Due cash allocation. Likewise,
native land performance is not a simultaneous allocation with future forward
deliveries. This preserves the existing timing preference; admission is not a
universal cross-commodity waterfall.

## Closure and retained limits

Before closing or discharging loan deficiencies, the proceeding checks its
current non-loan claims. Every unpaid materialized land bill and every undelivered
accepted forward blocks closure, even if the latter is not due yet. The
`ClosureDeferred` receipt lists the actual blocking claims in their denominations.
The existing discharge flag still governs **loan deficiencies only**; it cannot
silently forgive rent or pretend goods were delivered.

This is conservative: an impossible delivery can leave a proceeding open
indefinitely. Negotiated termination, damages, commodity conversion and non-loan
write-offs need explicit accepted/legal terms and remain outstanding. Once claims
are fulfilled, a later Due can close. Later annual rent is still billable after
closure; a dated closure marker distinguishes that new debt from an improperly
discarded old claim when validating checkpoints.

The cash estate still uses one storage-free denomination. Accepted land coin
alternatives must use that denomination; other native goods remain performance
obligations. Multicurrency cash estates and liquidation of commodity inventories
are not added.

The legacy tool market is admitted only for servicing existing accepted deliveries:
its retained tool-request templates must correspond to existing deliveries, stock
seller reserves and plot expansion are excluded, and no replacement tool purchase
is generated in a world configured with proceedings. Active negotiation, town
markets, minting, households and the older mortgage driver remain outside this
composition. Estate custody cannot be a working, trading, lending or receiving
counterparty in those contracts. Autonomous forward origination followed by
recovery in one unchanged market configuration remains future integration work.

## Verification

The added controls in `tests/recovery.rs` cover:

- Twelve coins against two ten-coin loans and two grain of rent at two coins per
  grain: five coins to each loan and two coins settling one rent unit; no grain
  receipt or grain-linked issuance is fabricated.
- Native land collection constrained by creditor storage, later ClearArrears
  payment, issuance only after two actual grain receipts, restored new-use rights
  and later annual billing after closure.
- Future forward admission without acceleration; no delivery before Acquire at
  maturity; CPU/reference agreement and rejection of a tampered delivery batch.
- Partial forward delivery with both protected stocks and limited receiving
  space; no loan discharge while the delivery remains incomplete.
- Proceedings triggered by land-only or forward-only arrears; a future-only
  forward does not trigger an opening.
- Observer inclusion for a non-loan creditor, unchanged observed state/ledger,
  closure-blocking receipts, and rejection of a forged closed checkpoint.
- CPU/reference and checkpoint continuation for mixed land/loan cash allocation.

These controls establish scoped accounting and execution behavior, not general
insolvency law or economic calibration. Validation results are recorded below;
raw logs remain in ignored `output/economics/`.

### Completed validation

Across the final affected-suite runs, **99 tests passed in 11 suites**:
acquisition, commitments, credit, finance, forward, lending, loan views, recovery,
resale, storage/currency and telemetry. Recovery includes 20 tests (six added
admission controls). Earlier failed fixture/assertion attempts were corrected;
the final recovery and remaining-suite run passed all 44 tests, alongside the
55 passing tests in the other seven suites. Counts here do not sum repeated runs.

Formatting, all-target Clippy with warnings denied, local documentation-link
checks and repository artifact checks passed. The full crate suite was not
rerun for this change; the earlier 451-test record describes the prior loan-estate
implementation.
