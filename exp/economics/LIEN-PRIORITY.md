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
until lien subrogation is supported. Mortgages still use their existing settlement
choices; migrating them into the authorized lifecycle remains work. Inventory and
receivable collateral, disputed priority, changing ranks and autonomous valuation
remain extensions.
