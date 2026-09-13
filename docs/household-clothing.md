# Household clothing retail pilot

Opt-in through `--household-clothing[=true|false]`; omission retains archive policy.
Existing and newly initialized household economies default to disabled. This is
an economic possession/replacement pilot, not a new clothing physiology model.

## Stocks, use and timing

After monthly food settlement, resident represented households may buy the
catalog's `cloth` from actual local stock. Each account has a wardrobe recording
held kilograms, cumulative purchased kilograms, worn kilograms and spending.
Target coverage is 0.6 kg per dietary-equivalent resident (monthly recorded food
need divided by 18 kg). This uses the existing aggregate household needs, not an
invented exact member roster. It does not claim equal clothing needs at every age.

Purchasing protects one month of full food need at the local quote, irrespective
of expected common provisions. At most 10% of remaining cash is offered monthly,
and purchases stop at the coverage target. Eligible household requests are gathered
before dividing limited local stock proportionally. Floating-point stock rounding
cannot exceed the allocated quantity. The town receives representable payment;
the household loses that exact cash. Sub-unit rounding can lower the effective
price slightly; it never creates cash. There is no municipal subsidy or credit.

Resident clothing wears at 2.5% per month. Worn mass enters the existing goods-used
and material-waste counters; its catalog C/N/P returns to local detritus. Retained
wardrobes participate in world material conservation. Town inventory is no longer
subject to the old communal cloth-use calculation while household clothing is
active, avoiding duplicate household demand. The bundled catalog fixes cloth at
slot 18 for this GPU gate; CPU retail resolves its stable ID.

New purchases occur after food and current production, so their money and stock
changes affect subsequent production plans. There is no retroactive completed work.
Repeated calls within the same month do nothing. Disabling purchases preserves
existing wardrobes and their resident wear. Traveling households keep their
wardrobes without local purchases or wear during the journey. Vacant households
retain possessions without a fictitious consumer; abandoned sites cannot retail.
These retained possessions are not yet covered by estate succession or salvage.

## Integration and limits

This transfers actual savings into town operating cash and creates replacement
opportunities through existing cloth recipes, targets and finite workshop service
payments. It does not earmark a particular purchase for a private operator, force
an enterprise to enter, manufacture unavailable fiber, or grant free labor.

The model currently treats cloth as ready-to-use clothing material: no separate
sewing recipe, styles, quality lots, personal garments, cold protection or status
benefits. There is no intertown household shopping or remote wardrobe transport
shipment; possessions remain attached to the migrating household identity.
Adaptive quotes still lack explicit household cloth-budget attribution; actual
retail reduces stock, which affects subsequent quotes through existing scarcity.
Do not equate the pilot with a general consumer market or a solved circulation
problem. Calibration and verification results follow below.


## Initial balance screen

Baseline is `4c2626c` (simulation identical to `665476f`). Matching 32/32 founding
archives, seeds 1024/256/409, fifty years, service procurement 0.25,
contract/demand staffing, inheritance, named office service, delivery-paid exports
and abandoned-stock recovery enabled. Reclamation, credit and issuance disabled.
All other settings remain unchanged. Outputs stay under ignored
`output/household-clothing-screen/`; timings are not isolated benchmarks.

The first implementation protected three months of food. All three runs completed.
Seed 1024 bought 67.94 kg cloth for 452.28 currency units; population 153.905,
hunger 0.04823, operator work 16.495, margin 68.40, zero active operators.
Seed 256 had no cloth purchases and unchanged economic results. Seed 409 bought
612.04 kg for 7,536.06, but cloth production fell from 2,119.61 kg to 836.09 kg;
ending population 344.110, hunger 0.04813, operator work 139.786, margin 746.35,
and zero active operators. More private transactions did not establish improvement.

A second three-seed screen reduced only protected future food to one month.
Current-month food still settles before purchasing; the 10% surplus spending cap
remains. Results relative to the original no-clothing baseline:

| Seed | Population off → on | Ending need-weighted hunger | Operator work | Revenue minus wages/rent | Active operators | Cloth purchased / spending |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | 154.263 → 153.906 | 0.12297 → 0.04850 | 12.680 → 16.495 | 50.79 → 68.41 | 0 → 0 | 67.94 kg / 458.06 |
| 256 | 345.715 → 345.715 | 0.03746 → 0.03746 | 4.693 → 4.693 | 19.66 → 19.66 | 0 → 0 | 0 kg / 0 |
| 409 | 352.540 → 348.748 | 0.01888 → 0.03942 | 183.934 → 197.827 | 952.25 → 1,028.44 | 4 → 3 | 1,079.53 kg / 20,211.00 |

Retain the one-month reserve as an **opt-in pilot**, not a new default or a solved
balance. It supports additional real workshop business in two seeds, but does not
restore broad population viability; seed 409 still has worse ending hunger than
the old communal-use baseline. Seed 256 lacks available cloth: changing household
purchasing cannot create a supply chain. These nonlinear trajectories do not prove
that a particular later population change was caused by the first purchase.

An explicit disabled seed-409 run matches the entire old fifty-year export after
removing only new default clothing fields. The audit now reports held/purchased/
worn cloth and cumulative spending separately from cash stocks. Eight Python audit
tests pass, including that spending is not counted as additional cash.

## Century comparison and verification

A fresh paired seed-409 century uses the current shared-inheritance baseline, not
an older century trajectory. The one-month clothing reserve remains the only
behavioral difference.

| Metric at year 100 | Disabled | Clothing enabled |
| --- | ---: | ---: |
| Population | 228.439 | 239.792 |
| Ending need-weighted hunger | 0.02622 | 0.02348 |
| Cumulative operator work | 385.611 | 337.499 |
| Operator revenue minus wages and rent | 2,147.88 | 1,802.41 |
| Active operators | 3 | 3 |
| Household cloth purchases | 0 | 2,127.35 kg |
| Household clothing spending | 0 | 36,200.57 |
| Retained household cloth | 0 | 69.93 kg |

The fifty-year business advantage does not persist through the century, while
relative food/population outcomes reverse direction. Keep both horizons visible.
Actual private demand now transfers money and materials, but this is not evidence
of generally superior welfare, sustained additional private industry, or recovery
of abandoned mineral deposits.

Nine native runs completed: three with the three-month reserve, three with the
one-month reserve, one explicit fifty-year disabled control and two century arms.
Maximum absolute reported managed residuals, in order C/N/P, water, money and
goods, were `[6.67e-6, 1.54e-6, 2.80e-5, 9.30e-6, 2.10e-7, 5.68e-6]` (rounded
up). Independent endpoint cash audit residuals stayed below 2.15e-7. These bounds
are over exported endpoints, not every internal operation.

Final verification: 197 ordinary library tests pass (153 hardware/long fixtures
ignored); the clothing analytical and GPU-backed transaction fixtures pass
separately. They check food-budget protection, service-target saturation, bounded
shared stock, no buying without cash, unchanged disabled state, household mass
accounting, world cash/material residual deltas, repeated monthly invocation and
serialized next-month continuation with purchases disabled. The first GPU fixture
failed because its test accounts had not been initialized; corrected setup passes.
Eight Python audit tests, strict library/binary Clippy and native build pass.
This does not claim a full long-run save/reload or cross-hardware comparison.
