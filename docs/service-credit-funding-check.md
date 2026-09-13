# Service receipts conditional on proposed financing

The ordinary-fee experiment found that a full-order repayment forecast could
approve a loan too small to pay the operator's rent, let alone the work underlying
its receipts. The source now carries an optional opening work-funding projection:
cash, fixed costs, cost per worker-month and maximum contracted work.

## Resolution rule

The existing joint allocation still limits lender cash, borrower exposure and
pledged sources. Before disbursement, a second check gathers all proposed principal
and debt service for each projected service source. With opening cash `C`, proposed
principal `L`, rent/fixed costs `F`, wage `w` and order work `W`:

```text
attainable_work = min(W, max(0, C + L - F) / w)
attainable_receipts = full_conservative_receipts × attainable_work / W
operating_costs = F + attainable_work × w
available_receipts = max(0,
    max(0, attainable_receipts - operating_costs) × coverage - existing_pledges)
```

If proposed debt service exceeds available receipts, or attainable work is zero,
all positive proposed grants against that source receive `UnfundedWork`. No loan is committed,
no money moves and no new pledge survives. Otherwise the original bounded grants
remain approved. Thus a profitable partial order can still support a partial loan;
the policy does not require every request to receive its full amount.

Rejected grants leave lender cash unspent. The pass does not redistribute that
cash in the same round; unused capacity remains a documented limitation of the
existing proportional allocator. There is no iterative convergence claim or
automatic refinancing. The forecast conservatively retains the existing net-
receipt coverage policy; this change does not relax costs to make loans viable.

## Boundaries and diagnostics

The service-order adapter captures current operating cash and wage/rent quotes.
Only one work-funded source per borrower is allowed in a round, avoiding repeated
use of the same opening cash. Competing loans against that source are checked
jointly. Tax and export evidence carry no work projection and keep their existing
rules. Older archives default the new projection and check fields to absent.

Each grant retains both its first-pass capacity receipt and a dated funding check:
proposed principal, total source funding, attainable work, estimated fees/costs,
remaining receipt capacity and total proposed obligations. A rejected proposal
has zero committed grant and pledged receipts; the proposed amount remains visible
for diagnosis. Archive validation checks capacity against that proposed amount,
then checks the final decision and grant against the funding result.

This is still a cash-feasibility forecast. Worker availability, competing tasks,
materials, wage changes and production disruptions can reduce actual completion.
Opening cash is not an escrow reservation. Later payroll and production remain
authoritative, and successful forecasting does not guarantee repayment.

## Verification requirements

Analytical cases cover an unproductive grant that the prior forecast approved,
a profitable partially financed order, rent consuming a tiny grant, existing cash
supporting work, invalid work costs, old evidence loading, competing lenders,
order independence and a corrupted final grant. The normal-fee GPU comparison
must be rerun alongside the valuable-contract positive fixture, market regressions
and strict Clippy before treating the implementation as verified.

Verification completed: all eight underwriting unit tests passed, the eighteen-
branch ordinary-fee GPU comparison passed, the positive-credit GPU/continuation
fixture passed, all nineteen active market tests passed (two remain ignored), and
strict all-target Clippy passed. Corrected ordinary-fee cases issue no loans or
fees; the deliberately profitable production case continues to work. See the
[comparison follow-up](normal-service-credit-comparison.md#follow-up-grant-dependent-feasibility).
