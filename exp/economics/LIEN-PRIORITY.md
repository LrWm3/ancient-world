# Shared collateral and realized-proceeds priority

Direct loan terms can select `CollateralSettlement::AuthorizedLiquidation`.
This explicitly permits multiple compatible liens on one catalog asset. The
loans must have the same debtor/owner and denomination, and each records its own
collateral priority. Fixed-value repossession and creditor-managed resale remain
exclusive promises; a later incompatible pledge is refused without advancing funds.

A missed installment records arrears. It does not transfer title, discharge debt,
or open a proceeding. An independently authorized proceeding must list the asset,
and an eligible funded buyer must meet its reserve price. The existing custody,
asset-transfer and contract-execution paths handle the sale.

At Acquire, the sale price is allocated only among liens on that asset. Lower
collateral ranks reserve proceeds first. Equal ranks use the existing collection
policy: Stable follows loan identity; Proportional shares against outstanding
debt, including deterministic integer remainders. Reservations cannot exceed the
asset's actual proceeds or the covered debts. They do not yet count as payments.
At the following Due, the estate pays those reservations before its ordinary
creditor waterfall. Surplus sale proceeds become ordinary estate cash. Residual
secured debts remain ordinary deficiencies after the lien is released.

Loan collection priority and collateral priority are separate. A high-priority
ordinary claim cannot take another asset's reserved proceeds. The rule applies
to single liens too, preserving their existing funded-sale behavior.

## Verification

`tests/recovery.rs` covers two ten-coin loans on one asset sold for eight:

| Lien ranks | Policy | Reserved/paid to first, second |
| --- | --- | --- |
| 0, 1 | Either | 8, 0 |
| 0, 0 | Stable | 8, 0 |
| 0, 0 | Proportional | 4, 4 |

Reversed configuration order gives identical results. An unfunded buyer leaves
title, both liens and both debts intact. Without an authorized proceeding, even
zero-grace loans remain in arrears with the debtor retaining title. A separate
two-asset test sells collateral for four and twelve coins against ten-coin debts:
it reserves four and ten, then distributes the remaining two to the first loan's
deficiency. Payments are six and ten; no creditor consumes the other lien's
reservation. CPU/reference, checkpoint continuation and separate financial
statements agree. Controlled pre-book losses establish default; there are no
unrecorded transfers during the measured recovery interval.

## Remaining scope

This does not negotiate lien ranking, infer consent, automatically open/list an
estate, or permit cross-currency liens in one custody account. Direct configured
terms remain the consent source. Guarantees of these loans are explicitly refused
until lien subrogation is supported. Mortgages can opt into this lifecycle as described below; their specialized
stock-sale planner still needs an adapter. Inventory and
receivable collateral, disputed priority, changing ranks and autonomous valuation
remain extensions.

### Household/member integration

A winding household can owe two secured ten-coin debts, one to the state and one
to its own member. An eight-coin sale pays four to each under equal-rank
proportional sharing. The member's seven unpledged coins are untouched, and the
four-coin repayment is not pooled back into the household. Without authorized
write-off, six-coin deficiencies survive and block dissolution. With explicit
loan discharge and completed estate closure, the household can release its final
affiliation. The member receivable and household liability remain separate in
both cases. CPU/reference, checkpoint and financial statements reconcile.

### Opted-in mortgage recovery

Financed purchases can now select `AuthorizedLiquidation` too. Their downpayment,
loan admission and ownership-following rights remain unchanged. An independently
authorized estate can sell the purchased property through the same funded-sale,
custody and distribution path as direct-loan collateral. Custody cannot receive
endowments, supply scheduled funding or become a mortgage counterparty. The
specialized mortgage stock-sale planner and creditor-resale buyer are still
excluded from this composition; legacy enforcement is not silently converted.

`tests/mortgage_recovery.rs` buys land for eight coins with two down and six
borrowed. A later four-coin sale leaves a two-coin deficiency; a nine-coin sale
pays six and returns three to the debtor. Unfunded bids leave property and debt
intact. Attached crops retain stage, elapsed work and consumed seed at sale; the
buyer controls their remaining obligations. Adequate buyer labor completes the
crop, while no labor causes failure. Crop output does not alter the supplied sale
price. Actual production costs use explicit joint-output shares. CPU/reference,
checkpoint continuation and all parties' financial statements agree, without
controlled losses or injected income in this fixture.

## Explicit guarantee subrogation

`Guarantee.security = InheritLiquidationLien` is an accepted term for an
authorized-liquidation loan. Actual payment creates zero-interest recourse with
the original asset, denomination and collateral rank. It preserves the existing
loan book and ordinary guarantee cap; no new coins, title or appraisal value are
created. Equal-rank original and inherited liens use the existing Stable or
Proportional policy; original-creditor-first protection is not implied.

Before sale, recourse inherits the active pledge, including when full payment
releases the original creditor's pledge. After sale, the paid portion inherits
any proceeds already reserved for the original claim. The reservation is moved,
not duplicated. Uncovered amounts remain ordinary deficiencies. A new recourse
addition cannot collect even reserved proceeds until a later Due; withheld
proceeds stay in custody and block premature closure.

The default remains explicitly unsecured recourse for existing supported claims.
Fixed-value and creditor-resale arrangements cannot request inherited liquidation
security. Autonomous consent, cross-currency subrogation, general security
assignment and negotiated creditor-first clauses remain outside this adapter.

`tests/recovery.rs` compares six- and ten-coin guarantees of a ten-coin loan,
payment before and after an eight-coin collateral sale, both allocation policies,
CPU/reference results, checkpoint continuation and separate financial statements.
Checkpoint validation rejects removal of agreed inherited collateral.

A household-guarantor continuation keeps its member's coins separate, recovers
actual proceeds, and compares retained versus discharged recourse deficiency.
Dissolution waits for receivable clearance and the following Open distribution;
member affiliation and financial reporting are preserved until then.
