# Broad integration audit and balance follow-up

The [current worklist](integration-worklist.md) covers the broader unfinished
connections, not only expedition heritage. This increment changes two consumers
of economic activity: merchant political appeal and heritage contact. It does not
complete non-workshop labor conversion, missing resolution adapters or the other
heritage uses still listed there.

## Controlled changes

[Recent delivered trade](recent-trade-contact.md) replaces permanent cumulative
sales as the political exposure signal. The same bounded observation connects
heritage news to actual land/sea deliveries. Neither consumer changes production,
financial settlement or cargo mass. Old archives initialize an empty observation
window. Monthly Open prunes it before deliveries, and annual politics and cultural
Respond observe completed deliveries.

Mechanism tests cover traffic expiry, split consignments, inactive pairs,
serialized continuation, no-traffic isolation, one-hop transmission and custody.
The existing market fixture now additionally checks that a delivered multi-hop
cargo generates a contact receipt even if its original route has closed. This
is an observation of completed travel, not permission for new journeys.

## Longer comparison protocol

Build and run:

```sh
cargo build --example cultural_work_calibrate
python3 scripts/run_integration_balance.py --years 100
```

The suite runs seeds 17, 81, 256, 409 and 1024 for each of four conditions:

| Condition | Demographic/production refinement | Yield scale | Founding food access |
|---|---|---:|---|
| individual | Individual demography and workshop refinement | 0.5 | Communal opening with taper |
| aggregate | Aggregate demography, existing named cultural participation | 0.5 | Communal opening with taper |
| scarcity-founding | Individual demography and workshop refinement | 0.33 | Communal opening with taper |
| scarcity-static | Individual demography and workshop refinement | 0.33 | Static configured common share |

All runs use terrain edge 32, ecology edge 16, one geological epoch, living
history, society, politics, governance, offices, shipping, expeditions and
discoveries. The first pair changes demographic and workshop resolution together;
it is an integrated mode comparison, not an isolated demographic effect. The
scarcity pair changes only founding entitlement at the same yield scale.

The runner now advances and observes every month. Earlier quarterly work totals
sampled the final month in each quarter and missed the other two months. Those
old totals are not comparable to this runner's complete monthly totals.

Food is summed locally before regional aggregation: need, opening available stock,
funded entitlement, actual consumption, physical shortfall and access shortfall.
For each town, physical shortfall is max(need - available, 0), and access shortfall
is max(min(need, available) - eaten, 0). The runner checks their sum against unmet
need. It also records maximum population/food residuals, work requests/grants/use,
known identities, institutions, artifacts, heritage witnesses, event counts and
resolution reports. Ten-year samples do not replace the monthly observations.

Raw reports and logs are under ignored `output/integration-balance/`. The launcher
runs conditions sequentially and stops on a failed run. A report's `complete`
flag applies only to that condition; all four must finish before drawing ensemble
conclusions. These are toy balance comparisons at small grid resolution, not
scientific calibration, cross-resolution validation or a performance benchmark.

## Verification status

All 209 library tests passed with GPU fixtures enabled, and all eight market
integration tests passed. All-target Clippy with warnings denied, formatting,
and the repository artifact check passed. An initial build exposed two explicit
History test constructors that needed the new default observation field; those
fixtures were updated. No simulation rule was changed to make them pass.

The longer suite was launched on the Quadro RTX 5000 Vulkan backend. At this
commit, the first individual run had reached year 40 without a reported invariant
failure. This is progress, not a completed balance finding: matched conditions,
held-out seeds and the remaining years are still pending. No further behavioral
expansion is justified by this partial run alone.

The first two conditions subsequently completed all five century runs. They
exposed severe individual-mode depopulation despite clean accounting. See the
[resident payroll investigation](resident-payroll-balance.md) for the measured
population and food-access differences, the verified empty-estate income bug,
and the controls being run before accepting any balance conclusion.

The previously unstarted static-access scarcity condition is now running directly
with the release runner corresponding to `a323d84` (the current crew-comparison
addition changes observations only). Command:

```sh
target/release/examples/cultural_work_calibrate --seeds 17,81,256,409,1024 \
  --years 100 --individual-demography --workshop-refinement --compare-resolution \
  --crop-yield-scale 0.33 --no-founding-access \
  --output output/scarcity-static-residents.json
```

This explicitly retains resident-eligible payroll. Earlier failures and controls
used different payroll switches; compare reported switches before attributing a
difference to founding entitlement alone. All five static-access runs now complete; the [balance report](resident-payroll-balance.md) records their severe decline and clean population accounting. A matched founding-transition rerun with the same resident payroll is needed before attributing differences to the transition.

The matched transition-enabled rerun is now running with the same release binary
and command, omitting `--no-founding-access` and writing
`output/scarcity-founding-residents.json`. No outcome is claimed until it completes.
