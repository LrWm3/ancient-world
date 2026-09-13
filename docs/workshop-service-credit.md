# Working-capital credit against workshop service orders

Funded workshop orders can now enter the commercial underwriting round when both
`--commercial-credit` and `--service-order-credit` are enabled. The second switch
is optional and defaults off for new and old histories. Omission preserves an
archived setting; false stops new proposals, not existing loan servicing.

This does not automatically post service orders. A caller still uses
`History::fund_workshop_order` to reserve town cash for a dated fee. Automatic
procurement, active-order balance comparisons and explorer controls remain open.

## Forecast and lender

`History::service_order_credit_evidence(loss)` reads live pending orders. It caps
expected work by installed leased capacity, applies the contracted service rate
and a declared loss assumption, and never forecasts more than remaining escrow.
Closed firms, abandoned sites, missing capacity, settled orders and missed payment
months cannot support a new request.

Operating costs include the expected work at the same current food-indexed wage
reference used by enterprise preparation. Refined wages include the next posted
multiplier where present. Costs also retain monthly rent through the due month.
The forecast is an approximation: available inputs, competing work, future wages
and disruptions can prevent completion. Escrow guarantees only that payment cash
was reserved, not that the operator can earn it.

The operator requests its uncovered operating-cost gap. Its local paying town
may voluntarily offer remaining cash above its existing commercial reserve, using
the same surplus-share rule as export lending. The reserved service fee is not in
that spendable account. Service and export requests share one underwriting round
and one offer per town, so overlapping opportunities cannot each spend the same
lender surplus. Existing borrower/source/exposure caps, risk-adjusted return and
prior pledged receipts still apply. A town simultaneously trying to borrow and
lend remains subject to the existing exclusion.

Underwriting deducts operating costs from expected receipts before allowing debt
service. It therefore does not assume the entire funded fee is available to repay
a loan. This conservative rule can reject otherwise tempting orders; increasing
nominal fees merely to obtain lending is not a calibration strategy.

## Timing and accounting

The commercial decision remains in Reserve, before enterprise wages are funded.
An order due this month can earn its fee later in Execute. Its loan matures at the
next monthly Open, after that receipt. This is an explicit exception to the
underwriter's strictly-future source rule; taxes and export receipts retain their
existing timing. Expired service sources remain ineligible.

Disbursement uses the operator financing ledger, not revenue. Escrow is unchanged
by lending. Only actual completed work releases service fees. Existing monthly
loan servicing, arrears, defaults and estate handling remain responsible for the
contract. A failed order refunds the town and can leave its operator in debt;
there is no automatic rescue or debt cancellation.

## Verification scope

The controlled fixture uses a deliberately valuable fixed-fee order and an
explicit opening-capital reallocation to create a solvent funding gap. It checks
that enabled lending reaches the operator, disabled lending does not, missing
capacity removes evidence, and expensive wages prevent approval. Fees remain
unearned at disbursement; money/enterprise ledgers and restored replay are checked.
The same fixture then restores the pre-Reserve world and runs the full monthly
GPU coordinator with and without service credit. Funded/completed operator work
increases with credit, the fee is earned only in that branch, and save/load
continuation matches. This demonstrates a production mediator for the deliberately
valuable contract; it does not calibrate normal fees.

A separate analytical timing fixture distinguishes current-month service fees
from current-month tax/export sources and rejects expired service fees. Matched active-order seed comparisons remain required before claiming a general
economic benefit. Automatic procurement remains unfinished.

Verification passed: the hardware-backed underwriting/full-production fixture,
the analytical timing fixture, the CLI override fixture and strict all-target
Clippy. The production fixture checks the opening-to-closing money residual within
1e-6 and leaves debt outstanding rather than treating service payment as repayment.

The final regression pass also passed all 19 active market tests (two hardware
fixtures remain ignored in that command) and the explicit GPU service-order
settlement/refund regression. These checks cover the new escrow path alongside
existing credit, issuance and account-succession behavior.
