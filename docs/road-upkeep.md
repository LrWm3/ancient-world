# Road upkeep and material continuity

Newly surveyed roads now retain maintenance state alongside their canonical
`road_bricks` stock. No second inventory or permanent improvement bonus is added.

Every monthly social boundary removes 0.2% of remaining road material, or 2.2%
while flooded. Removed material enters the origin settlement's existing used-good,
dry-waste and C/N/P detritus ledgers. Closed roads and roads serving abandoned towns
still weather. This is an abstract road-surface deterioration model: the waste is
accounted at the funding endpoint, not spatially deposited along individual cells.

Annual public works spend existing bricks and council money, and now consume
unused craft labor reported by monthly GPU production. The rate is 100 kg per
worker-month, with at most 100 kg built per route per annual works season and a
1,000 kg improvement ceiling. Each route draws from the endpoint's remaining
shared labor budget. Building transfers goods into the existing road stock rather
than recording them as consumed twice. This December-work proxy does not yet
schedule construction crews throughout the year.

The existing travel-cost equation already reads that stock:

`effective distance = route distance / (1 + min(road_bricks / 1000, 1))`

Thus deterioration affects subsequent land itineraries and road approaches to sea
ports. Committed journeys retain their original itineraries. Roads remain usable
dirt paths after their improvements disappear; flooding can independently close
them. There is no new per-edge freight-capacity arbitration in this increment.

A road that has reached 800 kg is considered established. Dropping below 500 kg
records deterioration; rebuilding to 800 kg records restoration. Events reference
the same road and link to its previous event. The inspector shows weathered mass,
cumulative work and condition. These thresholds affect notices, not travel cost.

Persistence stores the observed month and last works month, preventing duplicate
wear or construction at a repeated boundary. Older archives without upkeep retain
their original road behavior; missing historical losses are not fabricated.

## Verification and balance checks

The rates are game-balance assumptions, not fitted historical engineering data.
With no repairs and no floods, 1,000 kg becomes approximately 486.4 kg after 30
years. Continuous flooding is deliberately much harsher. A funded annual 100 kg
works season can cover ordinary wear on an established road, provided the town
has the spare labor and bricks.

Focused tests cover the analytical decay curve, flood acceleration, material
accounting, travel-cost response, abandoned/closed roads, repeat-call idempotence,
serialized continuation, labor-limited building, the improvement ceiling and
causally linked restoration. The society integration suite checks conservation and
full checkpoint continuation through production, trade and population history.

Reproduce the focused tests with:

```sh
mise exec rust@1.89.0 -- cargo test --lib road_ -- --include-ignored
mise exec rust@1.89.0 -- cargo test --test society -- --ignored
```

The history evaluator now retains route state at completion and annual totals for
road material, losses, work and impaired roads, so ensembles can distinguish
maintained roads from roads that never had enough resources to improve.

## Retained 50-year runs

Seeds 17 and 81 use terrain/ecology resolution 64, 16 initial civilizations,
yield scale 0.33, living environment, discoveries, offices and linked enterprise
income. The preceding expedition-crew build uses matching settings. Raw hashed
trajectories, source/build manifests and verification logs are in
[Artifact retention policy](evidence/README.md).

| Seed | Roads / ever established | Remaining kg | Weathered kg | Work, worker-months | Final population, previous → current | Max relative residual |
|---|---:|---:|---:|---:|---:|---:|
| 17 | 24 / 3 | 5,698 | 5,081 | 107.79 | 1,760 → 1,803 | 1.25e-5 |
| 81 | 22 / 9 | 10,808 | 9,663 | 204.71 | 1,588 → 1,528 | 1.32e-5 |

Neither run produced a deterioration or restoration event. That does not establish
that abandonment is harmless: the controlled neglect fixture exercises the warning
and recovery path. Many roads never became established, so their slow travel is
underinvestment rather than collapse. Population differences include nonlinear
feedback and cannot be attributed solely to travel speed; this comparison changes
both road wear and repair labor. The immediate road-stock and travel mediators are
verified separately. Broader seeds and targeted disasters remain useful follow-ups.

Verification: 63 ordinary tests passed (135 GPU fixtures remained ignored in that
command); the focused road command passed three tests, including two GPU fixtures;
two society and two market GPU integration tests passed. Clippy and formatting
checks passed. GPU execution used Vulkan on the Quadro RTX 5000 Max-Q; concurrent
host activity prevents treating elapsed times as isolated benchmarks.

Reproduce the retained ensemble and summary:

```sh
python3 scripts/build_history_evaluator.py --output output/road-build-new
python3 scripts/evaluate_enterprises.py --binary output/road-build-new/evaluator.bin \
  --build-manifest output/road-build-new/manifest.json --output output/road-runs-new \
  --seeds 17,81 --years 50 --modes operators-linked
python3 scripts/summarize_road_upkeep.py --current output/road-runs-new \
  --baseline docs/evidence/expedition-crews/final --output output/road-runs-new/roads.json
```
