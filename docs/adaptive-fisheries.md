# Adaptive fisheries (experimental)

[Timber-trap follow-up](timber-fisheries.md): an optional primitive gear alternative
now addresses the early equipment bottleneck measured below. This page retains the
first policy’s specification and original evaluation.

This opt-in policy replaces the fixed 0.1% shore-fishing reservation with finite
crew, equipment and stock-dependent catch. Enable **Adaptive fishing and finite
equipment** in the explorer's economy settings, or set
`economy_catalog.agriculture.fishery.adaptive = true` through `configure_economy`.
Old catalogs default to false; turning the policy off does not delete its assets.
The independent harvest-enable switch still closes the fishery.

## Stocks, work and demand

GPU work runs before ordinary monthly production, in stable site order to prevent
multiple towns withdrawing the same receiving-cell stock concurrently. It uses the
existing coastal receiving-water mapping, rather than inventing distant fishing grounds.

Per settlement the archive now retains installed timber/tools/fiber, desired crew,
actual fishing and construction work, monthly catch, cumulative wear and investment,
and the latest stock-response factor. Equipment is held material, not a new good:
construction moves ordinary inventories into the equipment record, and wear moves
those inventories into recorded losses and their C/N/P into detritus.

Default settings (game-design hypotheses, not empirical estimates):

| Parameter | Default |
|---|---:|
| Maximum fishing plus construction share of available workers | 15% |
| Unconstrained catch per fishing worker-month | 80 kg |
| Weighted carbon density giving half catch response | 0.0001 kg C/m² |
| Target food buffer | 6 months |
| Equipment per supported fishing worker | 40 kg timber, 1 kg tools, 4 kg fiber |
| Construction per equipment unit | 2 worker-months |
| Maximum construction share of desired crew | 25% |
| Passive monthly equipment wear | 0.3% |

The regional stock response is `density / (density + half_saturation)` with weights
1, 0.25 and 0.1 for aquatic grazers, migratory river animals and aquatic predators.
It limits both the useful crew target and catch per worker. The total fishing
workforce competes with farming, extraction and crafts; previously reserved cultural
craft work remains protected. Flood cleanup reduces the workforce consistently in
fishing and ordinary production.

Only food deficits below the configured reserve target justify fishing. Raw edible
inventories count toward that target; produced fish remains subject to existing
food conversion, storage and spoilage limits. Harvest cannot exceed actual available
C/N/P or the existing bounded guild withdrawal fraction. Restoration of depleted
guilds never supplies free animals.

Equipment construction uses existing goods, preserves a basic tool reserve, and
consumes part of the same fishing labor budget. The production planner requests
missing equipment inputs from the previous crew target; workshops, recipes and
trade must actually supply them. No equipment or materials are imported when the
policy is enabled. Idle or closed equipment continues wearing, and remains in the
archive and economic ledger rather than disappearing on closure.

Settlement roles continue to derive from measured production. Increased fishing
can therefore contribute to a fishing specialization without assigning a role as
an input or granting a role-specific production bonus. In the fixed social cohort
projection, fishing remains in the existing other/care livelihood category; the
explorer's fishery inspection gives its separate work figures.

## Limits

This is a regional shore-fishery proxy. It does not yet have vessels moving across
the lake, voyage costs, exclusive fishing rights, gear types, seasonal closures,
catch-size distributions or explicitly drawn net footprints. Receiving areas still
inherit the coarse ecology grid. Wages and sale profits are not directly optimized
by this crew allocation; local food demand and biological availability drive it.

## Verification and natural-world evaluation

Validation passed: 49 ordinary tests, 23 explicitly executed GPU tests (the new
fixture plus 22 economy/living-history regressions), formatting, and Clippy with
warnings denied.

The GPU fixture isolates each mediator: sufficient starting food prevents investment;
food shortage without fiber still cannot construct equipment; declared material
imports permit construction and catches; fishing plus construction stays below 15%
of workers and ordinary production receives only the remaining workforce. Exhausted
stocks, filled stores and explicit harvest closure stop work. Closure retains and
wears equipment. Six monthly steps match a six-month batch and checkpoint continuation
exactly on the tested Vulkan backend, including ecological buffer bytes.

The three ten-year natural runs use seeds 17, 81 and 256, terrain 64, ecology 16,
yield 0.5, and the previous evaluator's 120-month ecological spinup plus 12-month
history warmup. The policy is enabled after that common warmup without importing
gear, materials or animals. Each seed has baseline, sham, removal, restoration,
closure, and removal-plus-closure branches. A fourth six-branch run uses seed 17
with yield 0.33 for ten years.

| Seed / yield | New baseline catch kg | Fishing worker-months | Construction worker-months | Site-months seeking crew | Site-months fishing |
|---|---:|---:|---:|---:|---:|
| 17 / 0.5 | 0 | 0 | 0 | 354 | 0 |
| 81 / 0.5 | 24.94 | 9.23 | 1.88 | 520 | 68 |
| 256 / 0.5 | 0 | 0 | 0 | 309 | 0 |
| 17 / 0.33 | 0 | 0 | 0 | 1125 | 0 |

These results do **not** establish fishing towns. Demand exists for some early
months, but the joint material requirement prevents most construction; later food
reserves often eliminate demand. In seed 81, removal reduces catch from 24.94 to
13.96 kg, with no claim that the restored branch's 13.76 kg indicates recovery.
The harsher run demonstrates that food pressure alone does not solve equipment
access. Do not interpret the 80 kg unconstrained rate as realized productivity.

All 16 branch controls pass: sham physical trajectories match baseline, restoration
matches removal until its scheduled intervention, and closed branches catch nothing.
Maximum absolute relative residuals across all 24 trajectories are 2.12e-6 for the
economy, 2.65e-6 for ecological C/N/P, and 1.60e-6 for water. These are aggregate
floating-point ledger checks, not proof of exact local withdrawals at coarse
planetary cell areas. Small density changes can be below f32 resolution.

Even branches with zero catch can diverge through equipment procurement orders and
released ordinary workers. Guild removal also changes the wider ecosystem. Final
population differences therefore must not be attributed solely to fish consumption.
The analyzer's ration estimate now uses the lesser of nominal food energy and the
shader's C/N/P conversion bounds (0.444444 kg equivalent/kg fish for this catalog),
and is potential food output rather than measured fish consumption.

**Decision:** retain the opt-in policy. Next investigate primitive gear alternatives,
material availability during the hungry founding years, and trade-driven fish demand.
Those would address the observed bottlenecks; simply multiplying fishing yield would
not let a town build missing equipment. Sustained fishing specialization remains an
unmet calibration target, not an implemented outcome claimed by this report.

## Reproduction and engineering scope

```sh
mise exec rust@1.89.0 -- cargo test --test adaptive_fisheries -- --include-ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --test economy --test living --test living_scenarios -- --ignored --test-threads=1
FISHERY_ADAPTIVE=1 FISHERY_OUTPUT=output/adaptive-fishery mise exec rust@1.89.0 -- cargo run --example living_fishery_evaluate -- 120
python3 scripts/analyze_living_fisheries.py output/adaptive-fishery
FISHERY_ADAPTIVE=1 FISHERY_OUTPUT=output/adaptive-fishery-harsh mise exec rust@1.89.0 -- cargo run --example living_fishery_evaluate -- 120 17 16 0.33
python3 scripts/analyze_living_fisheries.py output/adaptive-fishery-harsh
```

[Archived evidence](evidence/adaptive-fisheries/) includes compressed monthly records,
full configuration/catalog metadata, summaries, logs and checksums. The same analyzer
accepts the compressed trajectory directories directly. Quadro RTX 5000 Max-Q,
Vulkan, NVIDIA 595.84 was the tested backend; other backends remain unverified here.
Observed elapsed times include pipeline compilation, readback, archive I/O and
concurrent local workloads, so they are not stage benchmarks.

The extra state is 64 bytes per settlement (16 KiB at 256 settlements). One serial
adaptive pass precedes the existing legacy pass without adding a CPU decision/readback
boundary. Legacy sites skip the adaptive calculations. A driver compiler crash during
pipeline creation was avoided by explicitly expanding the fixed three-element
material/guild operations; this is a tested workaround, not a definitive diagnosis
of the driver's internal fault. The existing monthly production shader then applies
the measured work reservation. Old JSON histories default missing fields to zero;
old catalogs keep adaptive fishing disabled.
