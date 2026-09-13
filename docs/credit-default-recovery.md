# Default recovery without rewriting the default

A default removes the live claim and liability. Later returned cash must not be
recorded as ordinary debt repayment, recreate interest-bearing debt or erase the
original loss. The separate recovery path preserves both facts.

`History::recover_defaulted_credit` accepts an explicitly authorized, dated
request naming a defaulted loan, allowance and reason (delayed proceeds, estate
surplus or voluntary settlement). It pays from the original borrower's existing
account to the original lender through the same exact-transfer adapter used for
credit. Retained closed accounts remain usable for settlement.

Recovery is capped independently by:

- The authorized allowance.
- Actual transferable borrower cash.
- Unrecovered principal and interest in the original default write-off.

The contract's interest-first split remains in use. Every committed request has
a stable caller-supplied ID, and replay is rejected before account mutation.
Zero-cash attempts retain a zero-transfer receipt; another authorized request can
retry later with a new ID. This is not automatic coercive collection.

## Separate facts and accounting

Recovery receipts are stored in `Credit.recoveries`, apart from ordinary loan
cash receipts. The loan entries, default status, accrued-through boundary,
original write-off and default-exclusion date remain unchanged. No new interest
accrues. A historical loss and a later recovery can both be reported; neither
is spendable money until actual cash moves.

Principal returned is financing, not production or sales. Recovered interest is
a separate financial transfer. Existing operator financing counters and the
world cash inventory receive the actual amounts through the account adapter.
Recovery never makes a missing or closed account eligible for new loans.

Validation checks unique request IDs, dated counterparties and currency,
nonnegative finite amounts, interest-first splitting, and cumulative recovery
within the original recorded loss. Each receipt also stores both account balances
before and after its transfer; both deltas must equal the reported amount. This
catches an overstated payment even when it remains below the original loss.
Old archives initialize an empty recovery
list and do not fabricate recoveries.

## Integration boundary

This is a reusable settlement operation, like explicit loan origination. The
opt-in [late-export policy](export-default-recovery.md) now supplies bounded
allowances after live debt servicing. Other calling systems must decide their
allowance and consent before settlement. Estate policies must share available
money with live obligations and protected operating needs,
and must not count a loss recovery as restoring ordinary borrowing eligibility.
General claim assignment and legal succession remain separate unfinished work.


The experiment runner reports recovered principal and interest separately while
retaining original default counts. No ensemble benefit is claimed for this
explicit settlement API.

## Verification

All sixteen non-GPU market integration tests passed. The new fixture checks
interest-first partial recovery, cash exhaustion, later funding from another
existing treasury, complete recovery, replay rejection, retained abandoned
accounts, serialized continuation, unchanged loan history and corrupted-receipt
rejection. Its first version exposed an overstated-transfer validation gap;
the added opening/closing account deltas close that gap. Strict all-target
Clippy and Python syntax validation of the experiment reporter passed.

The two GPU-only market fixtures were not rerun for this sparse accounting API.
Automatic export recovery remains disabled by default. Its separate integration
fixtures are documented in the linked policy record; no long-run benefit is claimed.
