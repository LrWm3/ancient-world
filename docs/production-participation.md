# Production participation boundary

Production participation remains unfinished. Workshops and the opt-in agriculture
pilot assign people; opt-in extensions assign forestry/mining and construction workers too.
Informal craft, operating services, fisheries and husbandry remain partly aggregate.
The [food-access controls](resident-payroll-balance.md) make this more consequential:
account-weighted pay and actual household needs diverge. Multiplying wages by
household size would not establish who worked or prevent overlapping assignments.

## GPU forecast foundation

`Generator::production_labor_forecast()` returns a dated, read-only projection from
the current history snapshot. It exposes farming, forestry, mining and crafting
worker-months plus existing public-service reservations, enterprise reservations,
previously observed fishing work and aggregate workforce. The GPU kernel and actual
production use the same `production_labor` / `worker_shares` functions. There is no
second CPU implementation of industry demand or labor policy.

Forecasting writes only the existing output scratch buffer. It neither produces
goods, removes nutrients, advances people, reserves work, pays wages nor updates
prices. Each row reads back 64 bytes: at the 256-site cap, 16 KiB. No planetary
terrain readback or new storage binding is required. The standalone inspection API
creates a temporary engine; the agricultural monthly integration uses the cached
engine's forecast method. This is not a measured performance improvement.

The forecast is conditional on its supplied snapshot. It does not run weather
wear or fishing. Later weather-storage changes and a new fishing allocation can
change the production planner's result. `prior_fishing` deliberately describes
existing input, not a newly predicted catch. Do not pass the scratch output to
ordinary stock settlement: a production dispatch must overwrite it first.

## Broader connection still needed

At the Reserve boundary, collect the forecast after existing service, enterprise
and vessel reservations. Match actual available residents against the remaining
sector requests. Fishing must share that same time budget rather than drawing a
second anonymous workforce. Pass individual grants as GPU production ceilings;
leave ordinary aggregate mode available. Earlier named commitments and aggregate
service reservations must not be deducted twice.

Municipal pay must then follow those actual assignments into their households.
Keep prepaid attendance, effective work, material output and any released work
separate. A month without money must not silently become a requirement that all
communal agricultural work stops; define funding and common-food entitlement
explicitly. Production should continue to enforce its physical inputs and return
completed work without retroactively consuming time released later in the month.

The forecast API alone completes none of these remaining steps. It supplies the
common aggregate input needed for individual matching and comparable receipts.

Verification: the hardware fixture checks the independent 62/8/10/20 allocation
for a 50-worker snapshot, repeated forecasts, empty population, unchanged economic
and demographic GPU buffers, and identical complete production state with or
without an intervening forecast. The standalone generator query agrees with the
cached-engine method. Commands:

```sh
cargo test production_forecast --lib -- --include-ignored
cargo test material_integration_tests --lib -- --include-ignored --test-threads=1
```

The forecast fixture and both existing material-production GPU tests passed.
The regular library suite passed 112 tests with 100 hardware tests skipped in that
invocation; all-target Clippy with warnings denied, formatting and the source-only
artifact check passed. These foundational checks did not test individual wage effects; the agriculture
pilot below adds that connection for one sector. The broader gate remains open.

## Agricultural attendance pilot

An opt-in agricultural resolver now consumes the forecast inside Reserve, after
service, enterprise and vessel reservations and before household retail. Enable it
with `History::set_agriculture_refinement(true)` or the calibration runner's
`--agriculture-refinement` (requires `--individual-demography` and
`--workshop-refinement`). Aggregate mode remains available; older archives default
to no agricultural resolver. Disabling dependent identity/workshop systems requires
turning off agriculture first at a completed boundary.

The resolver caps the request by cultivable land and distributes attendance
proportionally over eligible residents' remaining capacity. Each grant uses the
existing personal commitment ledger. It does not assign the same reserved time
again to cultural work or a private workshop. The GPU then caps cultivated hectares
and agricultural labor by granted attendance. Sowing and harvest collection use
the fraction of requested attendance supplied; uncollected managed crops become
local detritus, retaining their C/N/P. Legacy calendar crops retain their uncollected
standing inventory. Growth is still the existing managed-land abstraction, not
individual field operations or plant physiology.

Farm payroll uses current grants as household earnings weights, replacing the
account/occupation proxy **for the farm sector only**. The total municipal wage
withdrawal remains bounded by town cash and policy. Workers can contribute communal
attendance when no cash is available; neither attendance nor common entitlement
creates money or food. Wages are prepaid before retail, and later unused attendance
does not claw back food already purchased. Settlement records requested attendance,
grants and cultivated worker-months in common resolution receipts. Production
precedes this month's demographic deaths, so a subsequent death does not erase
already performed agricultural work.

With agricultural refinement alone, forestry, mining, construction, husbandry
and fishing remain aggregate. Their pay shares retain existing proxies, and they do not yet have complete personal
assignments. The GPU still bounds total sector allocations by its aggregate
workforce; the farm cap can only reduce its agricultural allowance. This is a
causal agriculture increment, not completion of all production participation.

Plans and commitments persist together, reject stale or duplicate reservation,
and settle once. The focused GPU fixture checks available versus fully committed
residents, cultivation and harvest consequences, actual household recipients,
finite payroll transfers, repeated settlement and save/load versus monthly/batched
continuation. Long-run population and income balance must be compared separately.


Pilot verification on the Quadro RTX 5000 Max-Q Vulkan backend: both forecast and
agricultural GPU fixtures pass (42.2 seconds together, including shader setup).
The regular library suite passes 112 tests, with 101 hardware tests skipped by
that invocation. All-target Clippy passes with warnings denied. These checks
establish bounded participation, payroll and continuation in the fixtures, not
population viability. The completed matched three-seed 30-year comparison follows; the century scarcity comparisons in the food-access document use
the earlier model without this pilot.


## Matched agricultural payroll / attendance comparison

Model `db6d2cb`, Vulkan Quadro RTX 5000 Max-Q; terrain 32, ecology 16, one epoch,
living history, yield scale 0.5, founding access enabled, resident-eligible payroll,
individual demography and workshops, comparison receipts enabled. Each arm ran
30 years for seeds 17, 81 and 256. Both reports finished all seeds. The only option
changed between arms was agricultural refinement. These are game-balance tests,
not fitted agricultural or demographic targets.

| Seed | Control population | Agricultural population | Physical gap, control → agricultural | Access gap, control → agricultural | Hungry accounts at year 30, control → agricultural |
|---|---:|---:|---:|---:|---:|
| 17 | 1,486 | 1,540 | 0.0486% → 0.0428% | 3.7163% → 3.2045% | 61 → 63 |
| 81 | 1,506 | 1,551 | 0.0004% → 0.0000% | 3.7454% → 3.2552% | 63 → 58 |
| 256 | 1,683 | 1,591 | 0.0000% → 0.0000% | 3.3373% → 3.1826% | 59 → 50 |

Food gaps divide cumulative monthly shortages by cumulative need. Hungry accounts
have more than 10% unmet need in the **single closing month**, out of 240 accounts
with positive needs in each run. They are not unique households ever hungry.
All runs retain 16 active sites. Maximum monthly population residual is zero;
normalized food residuals are at most 2.16e-7. The two material-production GPU
regression fixtures also pass after the shader changes.

At month 360, requested → granted agricultural worker-months are 594.324 → 585.077,
588.256 → 576.622 and 614.678 → 605.399 for seeds 17, 81 and 256. Cultivated effort
matches grants within GPU rounding. This snapshot shows remaining-capacity
shortfalls without proving they were equally small in every preceding month.

Access gaps improve across the three runs, but population rises in two and falls
in seed 256. Seed 17 even has more hungry accounts in the closing month despite
its lower cumulative access gap. The combined attendance/payroll intervention
therefore is not a general population repair. It changes both household earnings
and cultivation; do not attribute every later population difference to wages
alone. Controlled absence/payroll fixtures establish those immediate connections;
these multi-decade runs show their coupled outcomes. Other industries still use
aggregate pay proxies, and ordinary retail still leaves substantial unmet need.

Reproduce with `target/release/examples/cultural_work_calibrate --seeds 17,81,256
--years 30 --individual-demography --workshop-refinement --compare-resolution
--household-diagnostics --output output/agriculture-control.json`; add
`--agriculture-refinement` and a different output filename for the pilot arm.
Concurrent arm runtimes were 39.5–82.8 seconds per seed, including first-run GPU
setup; no speedup claim is supported.

## Held-out century results

Both arms finished seeds 409 and 1024 for 100 years under the same protocol as
above. Model `db6d2cb`; the later separate-household kin-care change is absent
from both runs. The only between-arm option is `--agriculture-refinement`.

| Seed | Control residents / active sites | Agricultural residents / active sites | Physical gap, control → agricultural | Access gap, control → agricultural |
|---|---:|---:|---:|---:|
| 409 | 89 / 13 | 767 / 16 | 0.0124% → 0.0000% | 5.6943% → 3.5663% |
| 1024 | 87 / 12 | 691 / 16 | 0.1023% → 0.0629% | 5.5542% → 3.6396% |

All maximum monthly population residuals are zero. Maximum normalized food
residuals are 5.33e-7 and 6.45e-7 for the controls, and 3.29e-7 and 3.48e-7 for
the agricultural arms. Runtime is 149.7–217.2 seconds per seed with simultaneous
arms and test compilation, so timing is not an isolated benchmark.

The improvement survives the longer interval on these two held-out seeds, but
**does not establish stable populations**. Seed 409's agricultural population
falls from 1,992 at year 10 to 1,313 at year 50 and 767 at year 100; its control
falls from 2,045 to 864 to 89. Agriculture meaningfully slows the decline instead
of repairing every cause. Cumulative access gaps remain over 3.5% despite very
small physical shortages. Keep the population/income gate open and the pilot
opt-in. The next production connection is actual extraction/construction work
and household pay, while separately reviewing dependents' food entitlement;
simply increasing crop yield is not supported by these results.

Reproduce each arm using the 30-year command above with `--seeds 409,1024
--years 100` and separate filenames. Completed raw results are ignored
`output/agriculture-heldout-control.json` and `output/agriculture-heldout-pilot.json`.
No incomplete runs were extrapolated or excluded from this comparison.


## Forestry and mining participation

The production attendance plan now optionally includes forestry and mining.
Enable `History::set_extraction_refinement(true)` after agricultural refinement,
or add `--extraction-refinement` to its calibration command. New and old histories
leave this extension off unless selected. Agricultural refinement remains its
prerequisite so the same resident cannot contribute a full anonymous farm shift
and then receive an additional named extraction shift.

One existing GPU forecast supplies all three sector requests. At each site, after
the existing service/workshop/crew reservations, farming matches first, forestry
second and mining third. Each distributes its request proportionally over the
remaining eligible personal capacity. This is an explicit food-first priority,
not a fair-share policy; earlier reservations can starve later extraction under
scarcity. There is no extra terrain readback or separate extraction forecast.

GPU grants cap forest and mining labor. Forestry reports timber removed divided
by its tool-dependent extraction rate. Mining reports the work consumed by ore
and clay together, preserving their existing alternating priority. Existing
source inventories, orders, tool difficulty, territorial claims and source
settlement still constrain physical output. Unused attendance does not become
output or retroactively fund another monthly activity. Household forestry/mining
wage shares now follow their actual granted participants; the finite municipal
payroll pool and prepaid-attendance semantics remain unchanged.

The saved plan retains the historic agricultural container for archive
compatibility, with a sector tag (old plans default to farming). Separate common
resolution receipts identify Agriculture, Forestry and Mining and record requested,
granted and productive work. At this extraction-only stage, construction, fisheries and husbandry still require
their own conversions (see the construction extension below); this extension does not claim their anonymous labor has
been reconciled with actual people. Construction in particular shares a service
pool with infrastructure operation and cannot simply reuse the mining output rule.

The focused extraction fixture compares available residents with fully committed
ones, checks remaining ore/clay under zero attendance, wages to assigned households,
productive work, duplicate reservation/settlement and checkpoint/batch continuation.
All three forecast/attendance GPU tests and both material-production GPU
regressions pass on Vulkan; the ordinary library suite passes 113 tests with
102 GPU tests skipped in that invocation. A matched balance comparison still
needs to establish whether the additional attendance and wage changes improve
whole-history behavior.


Construction boundary review: `ecological_production` starts its remaining craft
pool after `exchange.w` services, then operates waterworks, spends a bounded
asset-work share on urgent shelter, water-system recovery, housing and storage,
fits workshop assets, and executes recipes/private workshop plans. A construction
conversion must split public operation/building attendance from already named
private work before applying a cap. Capping the entire fourth sector by newly
available people would wrongly suppress work already paid and reserved upstream.
The required counterfactual is no builders with preserved private workshop grants:
construction should stop while the separately staffed workshop can still operate.


Extraction balance protocol (model `0d8959f`): paired 30-year runs for seeds 17 and
81 used terrain 32, ecology 16, one epoch, yield scale 0.5, living
history, individual demography, workshops, agricultural participation and the new
local kin-care behavior. Both arms use the same compiled model; only extraction
participation changes. Commands:

```sh
target/release/examples/cultural_work_calibrate --seeds 17,81 --years 30 --individual-demography --workshop-refinement --agriculture-refinement --compare-resolution --household-diagnostics --output output/extraction-control.json
target/release/examples/cultural_work_calibrate --seeds 17,81 --years 30 --individual-demography --workshop-refinement --agriculture-refinement --extraction-refinement --compare-resolution --household-diagnostics --output output/extraction-pilot.json
```

The completed results follow below. Raw outputs and logs remain ignored.
Long-term population stability remains an open worklist item.


## Completed extraction comparison

Both `0d8959f` arms completed the two-seed 30-year protocol above. This comparison
includes kin care in both arms and predates construction participation.

| Seed | Agricultural control population | With named extraction | Physical gap, control → extraction | Access gap, control → extraction |
|---|---:|---:|---:|---:|
| 17 | 1,540 | 1,501 | 0.04279% → 0.04224% | 3.2047% → 3.2633% |
| 81 | 1,551 | 1,565 | 0.00000% → 0.00000% | 3.2557% → 3.1458% |

All runs retain 16 active sites and have zero maximum monthly population residual;
normalized food residuals remain below 1.95e-7. Food gaps sum all months before
dividing by need. Runtimes are 39.9–84.6 seconds with simultaneous arms and initial
shader setup, not isolated benchmarks.

Seed 17 has no forestry/mining requests in the closing month; this does not prove
there was no extraction during the preceding years. At month 360 seed 81 requests
0.2966 forestry worker-month and grants 0.2361, using 0.2190; mining requests 0.5133
and grants/uses 0.1575. The latter demonstrates a remaining-capacity shortfall under
the current priority, not proof that every resource shortage has that cause.
Mixed population and access outcomes do not justify enabling extraction globally
or tuning yields to hide them. Review sustained request/grant distributions and
construction interactions before choosing a broader allocation policy.


## Construction attendance extension

`History::set_construction_refinement(true)` / `--construction-refinement` extends
the same plan after farming, forestry and mining. It requires extraction
participation. It is off by default and old plans keep their earlier sector tags.

The first construction increment used a policy ceiling: 20% of forecast craft work remaining
after services and enterprise commitments, not an exact estimate of every project.
Available residents receive bounded assignments after the other production sectors.
The GPU caps its existing asset-work pool and workshop building/fitting by those
grants and the labor left after protecting contracted workshop shifts. Stocked
materials, target capacities and the existing urgent-shelter/recovery ordering
continue to limit actual building. Operating water services is outside this cap;
recipe production continues against its existing labor and installed capacity.

Construction completion reports the actual labor decrement across building and
fitting, before recipes execute. Thus zero builders stops expansion without
pretending that an already staffed workshop also has zero workers. Wages remain
prepaid attendance: the municipal craft payroll separates the requested building
portion from its existing aggregate remainder, substitutes named grants, and
weights that portion toward their households. Unconverted craft income retains
its prior household weights. Total withdrawals remain cash-bounded; unused
building attendance does not create structures or retroactively reassign time.

This is still a partial production conversion. Informal crafts, water-system
operation, fisheries and husbandry do not yet have complete individual assignments.
The original construction request ceiling reserved more time than material/target
demand could use. The shared forecast increment below addresses that mismatch
without duplicating asset formulas on the CPU. Broad labor priority and long-term food access remain balance questions.


The balance runner now accumulates requested, granted and completed worker-months
from each completed monthly plan for farming, forestry, mining and construction.
It rejects stale/unsettled plans rather than reusing them. Reports label sector
order, columns and units; disabled sectors report zero with their option off.
These cumulative observations distinguish sustained under-allocation from an idle
closing month and must not be inferred from the older endpoint-only reports.


Construction verification: four production-forecast GPU fixtures pass, including
zero-builder/contracted-workshop separation, payroll, saved continuation and
batched versus monthly execution. The housing (2), storage (2) and waterworks (4)
integration tests also pass with ignored hardware tests explicitly enabled.
The regular library suite passes 113 tests (103 hardware tests skipped); two
material integration GPU fixtures pass separately. All-target Clippy is clean.
These checks establish bounded execution, not desirable population balance.

A matched construction comparison uses seeds 17 and 81 for 30 years, with the
same terrain/ecology/yield settings as the extraction protocol. Both arms enable
individual demography, workshops, agriculture and extraction; only
`--construction-refinement` differs. Both record cumulative production work.
Both arms completed; ignored outputs are `output/construction-control.json` and
`output/construction-pilot.json`. The tested source is committed as `c2b0c73`.


### Construction comparison results

| Seed | Control population | Named builders | Physical gap, control → builders | Access gap, control → builders | Builder requested / granted / completed worker-months |
|---|---:|---:|---:|---:|---:|
| 17 | 1,501 | 1,518 | 0.04224% → 0.04261% | 3.26325% → 3.15159% | 1,338.08 / 1,270.04 / 221.17 |
| 81 | 1,565 | 1,551 | 0% → 0% | 3.14581% → 3.29845% | 1,914.79 / 1,863.90 / 241.52 |

All four histories retain 16 active sites, with zero maximum monthly population
residual and normalized food residual below 1.77e-7. Execution takes 40.5–94.6
seconds per seed under concurrent runs, including initial shader setup; this is
not an isolated performance benchmark. Population and access effects are mixed.

Only 17.4% and 13.0% of granted builder time is completed. In contrast, 94.9% and
97.3% of requested builder time is granted. This identifies unused reservation,
rather than general builder scarcity, as the first issue to address. It does not
identify whether materials, targets or the shader's smaller asset-work share is
the principal cause; add a shared feasible-project forecast before adjusting
priority or increasing construction entitlement. Attendance remains prepaid, so
unused work also affects who receives income. These two seeds do not establish
long-term stability or justify a default-mode change.


### Feasible construction forecast

This increment replaces the 20% ceiling as the direct request with a GPU
preview of the shared construction transaction. The ceiling remains a policy
limit, but the request is only the work feasible within it, given opening
materials, housing/storage/water targets, workshop fitting and prior water-service
conditions. The preview applies wear and transfers to a local copy and discards
them; execution alone commits physical changes. It returns one additional scalar
in the existing 64-byte-per-site forecast, with no additional readback.

Opening stocks precede this month's extraction, weather and retail. Their later
changes can cause differences, so this is not an exact prediction of execution.
In particular, fresh timber may wait until next month's building request when
opening timber is absent. Live grants still cap construction, and already
contracted workshop work remains protected. The prior broad-ceiling comparison
above remains historical evidence; it does not describe the new forecast's
balance until a new matched comparison completes.


All four GPU forecast/attendance tests pass with this transaction, including an
analytical one-place housing case (2 timber + 3 bricks, 0.2 worker-month), absent
brick and absent target controls, and unchanged input inventories after preview.
Monthly/batched checkpoint continuation passes; all-target Clippy, formatting and
repository artifact checks pass. A fresh matched two-seed balance run is still
required to measure utilization and household effects.
