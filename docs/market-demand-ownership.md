# Ownership-aware adaptive quote demand

Adaptive quotes previously combined all local household cash with the town
operating account and allocated that total across every good's stock shortage.
Thus private savings could support an ore or tool quote even though ordinary
household retail purchases only food. This was a pricing signal, not an actual
transfer, but it could imply buyers that did not exist.

Town cash now supports the existing municipal order basket. Household cash
supports only the aggregate food quote, bounded by the last completed monthly
need less common provisions, valued at the opening food price, and by available
cash. Savings above that monthly requirement do not create extra demand.
Traveling households are excluded as before; dietary observations from a different
site are excluded after a move. A missing observation contributes no private
budget until the next completed retail allocation.

The monthly completed allocation forecasts the next month's demand; it does not
repeat the previous purchase. Nothing is debited by quote formation. Actual retail,
imports, service procurement, institution purchases and investments retain their
own existing finite transfers. The town's quote support is still a proportional
cash ceiling, not a simultaneous reservation or equilibrium auction. This change
also does not introduce household durable-goods consumption.

## Verification

A numerical fixture checks that a household with 10,000 cash, 20 food-equivalent
kg need, five common kg and price two supports only 30 currency units of food
demand. Low cash, fully common provision and zero need reduce that budget. Private
food money cannot change industrial quote support; municipal cash can. An empty
order basket allocates no municipal cash. These checks exercise the same helpers
used in monthly quote formation.

The numerical fixture and existing adaptive-quote unit test pass. The hardware
recipe-cost/actual-quote test passes (0.96 s); automatic service procurement with
checkpoint/batch continuation passes (1.72 s). Native build and strict all-target
Clippy pass. These are focused checks, not a rerun of the entire history suite.

## Matched 50-year screen

The screen reuses the three argument arrays and frozen 32/32 founding checkpoints
from `output/service-hiring-screen`, changing only the executable and export
location. Credit and issuance are off; adaptive prices, networked trade, service
procurement (0.25), demand/contract staffing, inheritance, named office service and
abandoned stock recovery are on. Outputs live under ignored
`output/quote-ownership-screen`. Compilation overlapped the first run, so the
screen supports behavior comparisons, not isolated timing claims.

All runs complete; maximum absolute relative cash residual is 9.81e-8.

| Seed | Population before → after | Completed operator work before → after | Operator operating margin before → after | Terminal need-weighted hunger before → after |
| --- | ---: | ---: | ---: | ---: |
| 1024 | 164.045 → 160.980 | 6.737 → 11.060 | 47.266 → 48.342 | 0.0487 → 0.0356 |
| 256 | 322.662 → 314.377 | 3.782 → 3.809 | 17.143 → 15.860 | 0.0481 → 0.0533 |
| 409 | 317.718 → 311.528 | 48.034 → 49.862 | 228.843 → 188.416 | 0.0746 → 0.0840 |

Terminal population-weighted tool quotes fall from 30.75 to 6.58, 24.05 to
11.51, and 19.04 to 11.09 respectively. Corresponding food quotes fall from
2.197 to 1.715, 1.615 to 1.417, and 1.719 to 1.511. These are local endpoint
offers weighted by remaining population, not realized transaction-price indices.

This correction removes unsupported private financing from industrial demand;
it is not a successful general welfare calibration. All three populations are
lower, and two end with greater hunger despite lower nominal food quotes. Paid
work remains over 99.6% completed and cumulative operator margins remain positive,
so restoring wasted payroll would hide the issue. Subsequent work should inspect
real household income/food entitlement and actual nonfood purchasing opportunities.
These nonlinear runs do not establish which later mediator caused each population
change. Circulation and broad economic recovery remain unfinished.
