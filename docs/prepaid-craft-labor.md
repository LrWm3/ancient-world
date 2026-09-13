# Prepaid craft labor isolation

A hardware fixture reproduced a reservation leak in the shared GPU craft pool.
With ten worker-months, eight prepaid metalworking months and sufficient orders,
metal and wheat, adding a flour recipe before the metal recipe reduced paid
metalworking execution from eight to five months. Flour used the other five.
The failing assertion and actual values are retained in the ignored local red-test
log; this was observed before changing the shader.

The recipe loop now subtracts remaining attendance reserved for other workshop
families before calculating a recipe's available time. Household processing can
use only unreserved craft labor. Industrial recipes can use unreserved time plus
their own family's reservation. Effective firm output capacity is divided by the
existing productivity factor to recover actual attendance units. Consumption of
firm capacity and labor remains in the existing execution code; no new workers,
extra payroll or materials are introduced.

The calculation is repeated after each recipe consumes labor/capacity, including
priority and spare-capacity waves. Actual orders, inputs, output storage, residue
capacity, equipment and overall labor still constrain production. Unused protected
attendance can remain idle when its own work is infeasible; this patch does not
silently reassign paid workers to unrelated household tasks or guarantee all
contracts complete.

This fixes competition within the recipe loop. It does not establish that all
earlier building/waterworks work or all CPU staffing forecasts honor every labor
claim; those require their own boundary checks. In particular, correct time
isolation can reduce household food processing when the upstream allocator hires
too many industrial workers. Balance comparisons must report that tradeoff.


## Verification

The pre-fix fixture failed with `prepaid=5 flour=5` when the competing household
recipe was enabled. After the fix, all three GPU recipe-allocation fixtures pass
(45.84 s including first-use pipeline setup): unrelated household work cannot
spend the eight reserved months; blocked recipes do not strand usable firm work;
experienced attendance remains bounded; recovered inputs only enable matching
production after delivery. Total recipe work stays within ten months in the
competition fixture and metal input use equals its one-to-one tool output.

The existing automatic procurement hardware fixture also passes, including real
escrow, missing-input/zero-order controls, money accounting and checkpoint/batch
continuation. This does not test every earlier construction or service priority.


The three-seed screen reuses the exact native argument arrays from the completed
contract-share after-runs, changing only the executable and export destination.
Seeds 1024, 256 and 409 each advance 50 years from their frozen 32/32 founding
checkpoint. Credit and issuance remain off; service procurement at 0.25,
contract/demand staffing, estate inheritance, named office service and abandoned
stock recovery remain on. Local outputs and exact commands are under
`output/prepaid-isolation-screen`; controls are
`output/contract-share-screen/SEED-after.json`. Native build and strict all-target
Clippy pass. Compilation overlaps part of the first run, so no isolated timing
claim is made.


## Three-seed results

All runs completed. Values below compare the previous committed executable with
the isolated-craft-time version at year 50. Work, pay and margins are cumulative;
hunger is terminal household-need-weighted hunger, not lifetime deprivation.

| Seed | Population before → after | Paid work before → after | Completed operator work before → after | Operating margin before → after | Hunger before → after |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 164.485 → 164.048 | 6.896 → 6.863 | 6.770 → 6.737 | 44.040 → 43.789 | 0.0550 → 0.0609 |
| 256 | 307.153 → 317.548 | 16.859 → 14.442 | 6.197 → 9.401 | -242.988 → -84.532 | 0.0450 → 0.0464 |
| 409 | 328.579 → 337.700 | 193.960 → 224.263 | 102.481 → 123.268 | -2578.427 → -2805.641 | 0.0806 → 0.0700 |

No private operators survive to year 50 in seeds 1024/256; seed 409 increases from
one to two. Maximum absolute relative money residual among new runs is 1.05e-7.
The direct fixture establishes the reservation leak and its correction. The seed
results establish mixed wider consequences, not universal improvement: seed 256
has substantially less paid-but-unused work and smaller losses, while seed 409
pays substantially more work and accumulates larger losses despite higher output.

Retain the correction because a recipe should not silently spend another job's
reserved attendance. Continue reviewing earlier construction/waterworks consumption
and actual input timing versus staffing forecasts. Any intentional reassignment
should have an explicit release and resulting work/payment receipt. Sustained
private viability, household purchasing-power circulation and wider recovery
coverage remain unproven.

## Construction boundary follow-up

Aggregate construction previously used an unbounded construction budget, while
named construction subtracted already prepaid workshop attendance. A controlled
GPU case with ten craft worker-months, 9.5 prepaid metalworking months, ample
metal/orders and an enabled housing project reproduced the difference: aggregate
housing used one month and only nine prepaid months executed. The failing result
was observed before changing the shader.

Construction now starts with the unreserved remainder in both modes. Named
construction additionally caps that remainder by its actual grant. The existing
shared construction budget continues through housing, waterworks construction,
storage, workshop building and fitting; it does not grant separate copies of the
remaining time to each project. No money or workers are added. Water-service
operation still precedes this window and needs separate staffing/priority review.
Protecting contracted work is not evidence that the upstream mix of industrial
and essential-service reservations is well balanced.

The extended hardware fixture now covers ordinary crafting, a competing food
recipe, aggregate housing and named housing. Both housing cases complete the
9.5 reserved months and positive construction bounded by the remaining 0.5;
combined recipe/construction work stays within ten. All three recipe-allocation
GPU tests pass (45.79 s including cold pipeline setup). Automatic service
procurement/checkpoint continuation passes (2.09 s). Native build and strict
all-target Clippy pass.

Matched 50-year runs use the previous section's arguments and checkpoints,
changing only the executable and output location. All three complete. Local
results are in `output/building-reservation-screen`; controls remain
`output/prepaid-isolation-screen`. Compilation overlaps the first run, so these
are behavior comparisons rather than isolated performance benchmarks.

| Seed | Population before → after | Paid attendance before → after | Completed operator work before → after | Operating margin before → after | Terminal weighted hunger before → after |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 164.048 → 164.042 | 6.863 → 6.863 | 6.737 → 6.737 | 43.789 → 43.789 | 0.0609 → 0.0599 |
| 256 | 317.548 → 319.626 | 14.442 → 12.029 | 9.401 → 8.262 | -84.532 → -55.605 | 0.0464 → 0.0513 |
| 409 | 337.700 → 329.123 | 224.263 → 214.886 | 123.268 → 118.153 | -2805.641 → -2688.858 | 0.0700 → 0.0648 |

Maximum absolute relative cash residual is 8.19e-8. This is a verified reservation
fix, not a general welfare improvement: losses shrink in two seeds but completed
work also falls, seed 409 loses population and seed 256 ends with greater hunger.
The long-run comparisons do not isolate those later changes to one mediator.
Workshops remain uneconomic in two runs, so the broader circulation task remains
unfinished. Next review water-service priority versus hiring forecasts and the
remaining paid-but-unproductive attendance before increasing procurement or fees.
