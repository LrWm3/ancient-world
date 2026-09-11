# GPU surveys and navigation

Default history navigation now runs on GPU. This is a game routing implementation,
not a promise to reproduce CPU Dijkstra's paths or real human travel choices.
`Generator::set_navigation_mode(NavigationMode::CpuReference)` retains the old
CPU searches for experiments. This execution setting is transient; loaded worlds
use GPU navigation unless the caller selects the reference again.

## What moved

- Inner-continent component labeling, eligible settlement-candidate compaction,
  and coastal landing compaction. Minimum-cell component IDs remain stable.
- Road pathfinding, settlement-to-harbor access, harbor-to-harbor sea lanes,
  and expedition paths from the great lake to a dry Ancient World landing.
- Monthly road flood, harbor flood/shallow-water, and sea-lane shallow-water
  predicates. Only one integer per route/port returns to the CPU.

Political route permissions, settlements chosen for connection, names, events,
harbor construction, ships and expedition decisions stay on CPU. Founding still
uses a full terrain snapshot for initialization, validation and patron origin
selection. This does not move every kind of survey onto GPU: resource catalogs,
regional survey interpretation and sparse historical choices retain their
existing implementations.

Living history gathers local settlement/candidate/source observations and uses
GPU navigation for unrestricted searches. It no longer needs annual full terrain
refreshes or complete road/sea-lane cell readbacks. Initial/load/invalidation
refreshes remain. The private static terrain cache is not a public snapshot.
Existing expedition paths now tolerate temporarily shallow lake cells during
living history, consistently with road/sea-lane validation; new paths still
require navigable water. Expedition landing cells remain observed each month.

## Search and limits

The cube-grid graph is implicit: four neighbors, including cube-face seams.
Integer atomic minimum distances and two bounded active queues propagate improved
costs. A per-wave mark deduplicates queued cells. A discovered terminal bounds
further exploration; positive costs make that pruning safe. Eight initial waves,
then batches of 64, amortize convergence checks. There is no CPU priority queue
and no full distance-field readback.

Roads require dry central land; harbor access reaches the nearest navigable
lake cell; sea routes remain in the great lake; expeditions terminate on dry
outer land. Costs retain elevation/discharge friction for land travel. Positive
meter-scale integer weights use GPU spherical distances. Search budgets are
3,000 cost-km for roads, 2,000 for harbor access, and 20,000 for sea/expedition
routes. These are weighted costs, not necessarily geometric lengths. Unlike the
old road search, a destination beyond the budget is not admitted merely because
it was the next queue item.

After convergence, GPU reconstruction follows strictly decreasing distances,
choosing the neighbor with smallest distance-plus-edge cost and stable cell-ID
ties. It returns the sum of the reconstructed edges. This avoids requiring
separately compiled floating-point calculations to round identically. Exact
CPU path equality and exact GPU optimality under independently recomputed weights
are not acceptance requirements. Contiguity, permitted terrain, finite costs,
and no cycles are requirements.

An unresolved search, queue overflow or reconstruction failure is an error,
not an unreachable destination. Searches have a wave limit of approximately
32 times the face edge. Component labeling uses minimum-root hooking and pointer
compression, with a 128-iteration convergence limit. No silent CPU fallback is
used. The design follows common frontier and connected-component techniques;
it does not embed an external graph library.

Workspace is approximately 20 bytes per terrain cell, plus a transient
32-byte-per-cell survey output buffer. The memory estimate includes both;
individual storage/dispatch limits are checked. Pipelines and navigation
workspace are reused until terrain-buffer/epoch invalidation. Queries are
currently serial and route-inspection buffers are rebuilt monthly. Batching
queries, hierarchy/landmarks, and incremental route repair are future optimizations,
not implemented features. API counters separate route, survey, and inspection
readback bytes; they exclude other simulation and archive transfers.

## Evidence

Hardware: Quadro RTX 5000 Max-Q, NVIDIA 595.84, Vulkan, Rust 1.89 optimized dev
profile. Outputs remain ignored under `output/`; only summaries are committed.

The first 512/256 trial failed path reconstruction: exact integer equality after
recomputing floating-point edges was too strict. This prompted descending-cost
reconstruction, rather than treating the failure as no route.

Tests cover matched founding candidates/sites/patrons from an untouched checkpoint,
road/harbor/sea/expedition reachability and path legality, a flooded destination,
a forced cube-seam path, explicit water-change hazard comparisons, history
outcomes, conservation, save/reload and different advancement batches.

### Three-seed outcomes (64 terrain / 16 ecology)

Each mode starts from the same pre-founding checkpoint, establishes 16 towns,
then runs 36 living months after a two-month setup. This compares CPU navigation
with GPU navigation, not just two terrain-readback modes.

| Seed | Population, both modes | Events, both modes | CPU setup / history seconds | GPU setup / history seconds |
|---|---:|---:|---:|---:|
| 17 | 962.246 | 813 | 0.459 / 1.247 | 1.282 / 1.266 |
| 81 | 986.005 | 920 | 0.408 / 1.162 | 0.937 / 1.293 |
| 256 | 945.582 | 810 | 0.398 / 1.164 | 1.058 / 1.313 |

All retained 16 towns, five ports, ten sea lanes and five expedition routes;
road counts were 30, 25 and 30. Event records matched. Ecological ledgers passed.
The separate 27-route comparison found the same paths and less than 0.02% cost
variation. Founding surveys matched when started from the same untouched state.

At terrain 512 (ecology 16), the same three-seed route fixture also passed:
13 of 27 paths differed from CPU Dijkstra, and the largest reported cost difference
was 0.536%. All passed contiguity, habitat, reachability and acyclicity checks.
Because the two implementations evaluate spherical distances differently, these
cost differences do not measure pure route optimality. GPU query totals for nine
routes were 240, 182 and 151 ms, excluding setup and CPU reference snapshots.
Founding surveys and the forced seam/blocked-destination fixtures also passed.

GPU setup is slower on these small worlds. Endpoint-bound pruning and shorter
initial batches reduced the nine-query fixture from 832 encoded waves to 328;
this is a work-count reduction, not an equal percentage runtime improvement.
These are single observations, with CPU reference executed first. Compilation
was occurring during parts of testing, so treat wall timings as indicative,
not a controlled statistical benchmark. The main demonstrated benefit is avoiding
large terrain transfers, not beating CPU Dijkstra on tiny maps.

### Full versus compact observations

At 64/16, all three seeds matched exact history, terrain and ecology for 36
months under drought/storm forcing, plus four months of checkpoint/batch
continuation. Each used one full refresh and 35 gathered refreshes instead of
36 full reads. Terrain transfer was 4.60–5.23 MiB rather than 148.50 MiB.

The 512/256 result below compares GPU navigation in both branches; only the
terrain readback strategy changes. Thus it isolates removal of full snapshots,
not GPU-versus-CPU route quality. Exact equality includes serialized history,
terrain bytes, ecology bytes, clocks, and the resumed continuation. Initial
founding and explicit verification/archive readbacks are excluded from timing
and transfer totals. Small summary and economic/ecological transfers are not
included in the terrain byte count.

| Grid, seed | Compact terrain MiB | Full terrain MiB | Compact history seconds | Full history seconds |
|---|---:|---:|---:|---:|
| 512 / 256, 17 | 267.66 | 9,504.00 | 5.605 | 16.098 |

The final default-grid run passed, including continuation: 97.2% fewer terrain
bytes and 65.2% less measured history time. It retained 637 road cells, 1,954
sea-lane cells, and five expedition routes. Only one seed was used at this grid.
The ordinary library suite passed 61 tests (65 GPU fixtures skipped); the new
water-intervention fixture and the navigation/history fixtures were run explicitly.
All-target clippy also passed after removing a redundant numeric cast in the
existing civic-balance example.

Reproduce:

```sh
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test navigation -- --ignored --nocapture --test-threads=1
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib navigation::tests -- --ignored --nocapture
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test history_environment -- --ignored --nocapture
HISTORY_GATHER_RESOLUTION=512 HISTORY_GATHER_ECOLOGY_RESOLUTION=256 HISTORY_GATHER_SEEDS=17 CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test history_environment -- --ignored --nocapture
```

To repeat the larger route check, prefix the navigation command with
`NAVIGATION_RESOLUTION=512` and filter to `gpu_routes_are_valid_and_compare_with_cpu`.

`NAVIGATION_RESOLUTION` changes the route fixture's terrain grid. The fixture's
2% CPU cost comparison is a regression guard for these test worlds, not a promise
of equal routes on arbitrary worlds or other hardware. Larger history ensembles,
other GPUs and high-settlement-count timing remain unmeasured.
