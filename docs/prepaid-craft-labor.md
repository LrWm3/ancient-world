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
