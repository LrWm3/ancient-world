# Production participation boundary

Production participation remains unfinished. Workshops already assign people;
municipal agriculture, forestry, mining and construction still use pooled labor.
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
creates a temporary engine; the intended monthly integration can use the cached
engine's forecast method. This is not a measured performance improvement.

The forecast is conditional on its supplied snapshot. It does not run weather
wear or fishing. Later weather-storage changes and a new fishing allocation can
change the production planner's result. `prior_fishing` deliberately describes
existing input, not a newly predicted catch. Do not pass the scratch output to
ordinary stock settlement: a production dispatch must overwrite it first.

## Remaining connection

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
artifact check passed. Individual assignments and their wage effects remain
unimplemented, so these results do not close the production integration gate.
