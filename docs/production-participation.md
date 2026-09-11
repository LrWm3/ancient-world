# Production participation boundary

Production participation remains unfinished. Workshops and the opt-in agriculture
pilot assign people; forestry, mining and construction still use pooled labor.
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

Forestry, mining, construction, husbandry and fishing are still aggregate. Their
pay shares retain existing proxies, and they do not yet have complete personal
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
population viability. Matched three-seed 30-year comparisons are evaluated below
when complete; the century scarcity comparisons in the food-access document use
the earlier model without this pilot.
