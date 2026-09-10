# Waterworks calibration — September 2026

New bundled worlds now target waterworks capacity for **50% of residents**, down
from 100%. The validated `production.waterworks_target_fraction` setting supports
0–1. Older archives missing this field retain the previous 100% target. Installed
assets are never deleted to meet a lower target; they continue operating and wearing.

This is a provisional capital-planning compromise, not an optimum or a claim that
half the population should always receive sanitation. Full coverage competed with
food and other industry while exposure was generally low. Half coverage reduced
construction work by about 45% and operating work by 43% in the 50-year runs;
final populations were 1.1–2.7% higher than with full coverage. The living runs
also favored half over full coverage by 1.1–1.5% in final population. Outcomes
include downstream changes in markets, migration and politics; they are not a
controlled estimate of the medical effect alone.

## Method

Eighteen distinct runs: seeds 17, 81 and 256 at coverage targets 0, 0.5 and 1 in
two regimes. All used resolution 64, one geological epoch, 16 requested initial
civilizations, crop yield 0.33, and crowded initial housing of 0.7 places/person.
Frozen-environment histories ran 50 years; living histories ran 25 years with
storm probability 0.1. Other settings matched within each regime.

The **zero-target control retains domestic withdrawals, water shortage and
crowding-related health exposure**. It starts without waterworks and does not
build them. `--no-waterworks` remains a separate legacy control that also removes
those pathways, and was not used in this sweep. This distinction fixes the
interpretation problem in the previous enabled/disabled comparison.

Coverage, disease and waterlogging below are population-weighted **annual
snapshots**, not monthly person-time integrals. Disease is an abstract burden
index, not clinical incidence. Work is cumulative worker-months. Served residents
are also exported directly, along with installed/worn material and domestic water.
No timings are presented as benchmarks: some runs overlapped regression tests.

## Crowded starts, frozen environment, 50 years

Annual exposure means are population weighted snapshots; work is cumulative worker-months.

| Target | Seed | Final people | Mean coverage | Mean disease | Mean waterlogging | Shortage site-years | Build work | Operate work | Max residual |
|---|---|---|---|---|---|---|---|---|---|
| 0% | 17 | 1866 | 0.0% | 0.0087 | 0.0000 | 104 | 0 | 0 | 1.13e-05 |
| 0% | 81 | 1708 | 0.0% | 0.0106 | 0.0000 | 127 | 0 | 0 | 1.23e-05 |
| 0% | 256 | 2325 | 0.0% | 0.0057 | 0.0000 | 131 | 0 | 0 | 1.48e-05 |
| 50% | 17 | 1887 | 50.0% | 0.0083 | 0.0000 | 89 | 331 | 555 | 1.08e-05 |
| 50% | 81 | 1657 | 52.6% | 0.0110 | 0.0000 | 143 | 305 | 536 | 1.23e-05 |
| 50% | 256 | 2310 | 47.9% | 0.0058 | 0.0000 | 139 | 356 | 608 | 1.48e-05 |
| 100% | 17 | 1837 | 86.8% | 0.0087 | 0.0000 | 99 | 601 | 954 | 1.11e-05 |
| 100% | 81 | 1632 | 93.1% | 0.0110 | 0.0000 | 132 | 576 | 941 | 1.24e-05 |
| 100% | 256 | 2286 | 90.1% | 0.0060 | 0.0000 | 131 | 670 | 1124 | 1.38e-05 |

## Living environment, storm probability 0.1, 25 years

Annual exposure means are population weighted snapshots; work is cumulative worker-months.

| Target | Seed | Final people | Mean coverage | Mean disease | Mean waterlogging | Shortage site-years | Build work | Operate work | Max residual |
|---|---|---|---|---|---|---|---|---|---|
| 0% | 17 | 1870 | 0.0% | 0.0105 | 0.0023 | 17 | 0 | 0 | 6.25e-06 |
| 0% | 81 | 1703 | 0.0% | 0.0137 | 0.0040 | 38 | 0 | 0 | 6.59e-06 |
| 0% | 256 | 2229 | 0.0% | 0.0056 | 0.0000 | 22 | 0 | 0 | 7.69e-06 |
| 50% | 17 | 1847 | 49.6% | 0.0108 | 0.0023 | 16 | 251 | 257 | 6.25e-06 |
| 50% | 81 | 1659 | 50.3% | 0.0145 | 0.0039 | 39 | 235 | 249 | 6.59e-06 |
| 50% | 256 | 2169 | 46.7% | 0.0062 | 0.0000 | 23 | 264 | 269 | 7.68e-06 |
| 100% | 17 | 1827 | 88.6% | 0.0112 | 0.0023 | 20 | 468 | 453 | 6.25e-06 |
| 100% | 81 | 1637 | 89.4% | 0.0149 | 0.0039 | 39 | 445 | 435 | 6.58e-06 |
| 100% | 256 | 2137 | 89.4% | 0.0068 | 0.0000 | 25 | 515 | 508 | 7.68e-06 |

## Interpretation and remaining limits

- All 18 runs completed with maximum relative ledger residual below 1.5e-5.
  No sustained waterworks-disruption events occurred.
- Half coverage used fewer finite materials and less labor than universal
  coverage. Hunger remained a major contributor to disease; sanitation does
  not remove hunger or create supplies.
- Zero investment still produced higher final populations than half coverage
  in five of six comparisons. The chosen default retains partial infrastructure
  while reducing premature investment; these results do **not** establish its
  general economic superiority over leaving waterworks unbuilt.
- Mean waterlogging was only 0–0.004 in the living runs. This suite is weak
  evidence about severe persistent floods or domestic drought. The GPU fixture
  separately verifies that service reduces matched crowding exposure, and that
  exhausted water storage disables service despite intact structures.
- Next useful refinement is local investment priority driven by sustained
  water exposure and affordability, tested with controlled persistent flooding.
  There is no justification here for increasing disease or sanitation bonuses
  simply to make infrastructure pay off in every town.

## Reproduction

Build with `mise exec rust@1.89.0 -- cargo build --release --example history_evaluate`.
For each target (0, 0.5, 1), use a distinct output label without decimal suffixes:

```sh
target/release/examples/history_evaluate --seeds 17,81,256 --years 50 \
  --crop-yield-scale 0.33 --initial-housing-per-person 0.7 \
  --waterworks-target-fraction 0.5 --output output/calibration-half
```

For the living regime use `--years 25 --living-world --storm-probability 0.1`.
Summarize matching runs with:

```sh
python3 scripts/waterworks_report.py output/calibration-zero.json \
  output/calibration-half.json output/calibration-full.json \
  --output output/calibration-summary
```

The report script rejects incomplete runs and differing reported controls.
Detailed local artifacts are `output/waterworks-{crowded,living}-comparison.{json,md}`;
source runs are `output/waterworks-calibration-{crowded,living}-{zero,half,1}.json`.

Validation: 20 focused tests across waterworks, economy, housing and storage,
including GPU conservation, water exhaustion, empty recipe queues, ruin decay,
and checkpoint continuation. New checks cover bounded coverage settings,
archive defaults, and zero investment retaining domestic demand. Formatting,
Clippy and diff whitespace checks pass.
