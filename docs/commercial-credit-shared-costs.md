# Commercial credit: one shared operating requirement

Previously each eligible export receipt from a town received the town's entire
planned input-cost forecast. Two receipts of 100 with a shared cost of 150 each
looked unable to repay anything, although their combined net receipts were 50.

The pilot now allocates that one opening cost across the town's eligible receipts
in proportion to expected proceeds. Source-specific costs remain additional.
Underwriting still applies its coverage fraction, existing pledges, borrower
exposure, lender cash reserve and voluntary offer independently.

In the analytical two-receipt fixture, each source protects 75 of the shared
cost. Each has 25 net receipts, and the default one-half coverage policy permits
12.5 principal per source, 25 total. An existing 12.5 pledge exhausts the first
source without consuming the second. Raising the shared cost to 250 makes both
sources unfinanceable. Unequal receipts receive unequal cost shares; a single
receipt receives the full original forecast.

This corrects duplicated protection. It does not make speculative production a
committed expense, reserve new inputs, promise future output or treat borrowed
cash as income. The broader question of separating planned versus committed
operating needs remains open. Cost sharing uses only the pilot's eligible dated
receipts; it does not invent unrelated future sales to justify a loan.

## Evidence and limits

Inspection of all recorded credit/combined rounds in the completed seeds 256,
409 and 1024 found no multi-source beneficiary rounds. This fix therefore does
not explain or overturn their null credit-only outcomes. Dedicated fixtures are
necessary to exercise the defect; a new ensemble without multiple receipts would
not establish this mechanism's correctness.

The resolver's detailed capacity snapshots expose the resulting source capacity.
Old recorded rounds remain unchanged, and no historical decisions are recomputed.

All eighteen credit unit tests and strict all-target Clippy passed.

## Completed matched regression

A fresh 200-year, four-arm seed-409 comparison from the same founding checkpoint
completed using the copied `7aef164` executable. All four native runs passed.
Baseline, credit-only and issuance-only exported histories are structurally
identical to the earlier `547aaba` histories. Combined differs only by its new
capacity snapshot; removing that added field makes the entire history identical.

The one combined-arm request has eligible principal 2.0238299714692403 against
799.5607499235867 lender capacity, 5,000 borrower capacity and
39.15016635124048 net covered source receipts. Source demand including interest
is 2.2901233887678245, so no pool scales down the request. Actual disbursement
is 2.023681640625 because the existing town account representation governs the
cash transfer. The eventual loan and precision settlement match the earlier run.

Maximum absolute monetary relative residual was `2.77e-7`, other managed
residuals `4.71e-5`, and ecological C/N/P residuals `1.62e-5`.
Generated data remains under ignored `output/monetary-shared-cost-regression`.
Shader tests ran concurrently, so these durations are not performance benchmarks.

This establishes regression stability in a single-source world, not balance
improvement or validation of multiple currencies. The analytical multi-source
fixture provides the direct evidence for the cost-sharing correction.
