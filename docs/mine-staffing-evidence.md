# Mine access × labor allocation: causal follow-up

This is a model intervention experiment, not empirical calibration. It investigates
the previously observed population increase following mine closure. No production
coefficient was fitted or changed to obtain the result. The diagnostic fixed-share
option is off in ordinary worlds and old archives.

## Design

Source revision: `c136a2a` (recorded in the final evidence manifest). Seeds 17,
81, 256, terrain 64/ecology 32, crop yield scale 0.33, sixteen initial civilizations.
One geological epoch and one year of living history precede each fork. Four
branches cross normal/fixed staffing with open/closed mine access. Closure covers
all sites existing at the fork for five years; access then returns for five years.
New settlements keep independent policies. Each branch begins at the same saved
state; there is no rerolled founding, destroyed ore or injected equipment.

The fixed-share intervention uses 62/8/10/20 percent of finite available workers
for farming/forestry/mining/crafting, with the existing fishery deduction. Actual
headcount, harvest seasons, resource limits, trade, food access and demographic
responses remain active. This tests an interaction; changing the allocator also
changes its baseline economy. It does not estimate a unique natural indirect effect.

## Findings

All three seeds show the same reversal. With adaptive staffing, mine closure
increases settled population. With fixed shares, the closure reduces it. The
reversal is preceded by an increased farming share in the adaptive branches and
by higher cumulative harvests and rations during the closure. The fixed branches
do not show the farm-share increase and instead deliver fewer rations.

The observations support labor reallocation as a mechanism within this model.
They do not establish that mine closures historically improved subsistence, or
that labor explains every later difference. Population effects include food,
health, movement, construction and subsequent decisions. The generated tables
separate settled population from total living people, including travelers.

A single end-of-year production snapshot is misleading here: the cumulative
harvest benefit can coexist with lower output in the particular sampled month.
Monthly trajectories and interval totals are necessary for seasonal farming.

The existing tool rule bounds the direct productivity factor between 0.75 and 1.
Mine closure does not delete tools already in inventory, and trade and recycling
continue. Thus a substantial labor shift can outweigh the loss of new tools.
This is an internally explicable tradeoff; the appropriate response is not to
force every mine closure to reduce population.

## Follow-up status

The proposed stock-buffering experiment is complete: see [tool reserve results](tool-buffering-evidence.md). [Food-aware labor](food-security-labor.md) and the [maintenance ablation](maintenance-ablation.md) document subsequent tests; their outcomes are not implied by this earlier experiment.

[Environmental returns](environmental-returns.md) implements runoff routing and abandoned-land release with controlled conservation and continuation fixtures. Longer-run response calibration, sourced empirical targets, held-out environments, sensitivity and resolution studies remain outstanding.

## Reproduction and verification

Run `python3 scripts/evidence.py --profile full` with the pinned Rust toolchain.
The committed workflow generates a unique artifact directory, CPU/GPU logs,
source/catalog hashes, monthly JSONL journals, complete result JSON, tables and
checkpoint files. See [model evidence](model-evidence.md) for equations, units,
traceability and declared limitations. CPU CI is configured; the manual hardware
job still requires a provisioned self-hosted runner.

The full run passed 41 CPU and 96 hardware Rust tests, three evidence-runner
checks, formatting, and Clippy with warnings denied. All three no-op controls
passed. Twelve branches produced 1,440 monthly observations; all completed and
all budget guards passed. Source hashes were unchanged during the run. GPU:
Quadro RTX 5000 with Max-Q Design, Vulkan, NVIDIA driver 595.84, Rust 1.89.0.

| Seed | Closure settled population effect, adaptive | Effect, fixed shares | First-year adaptive farm-share increase |
|---|---:|---:|---:|
| 17 | +168.12 | −35.61 | +18.118 percentage points |
| 81 | +170.59 | −16.94 | +15.676 percentage points |
| 256 | +196.37 | −26.34 | +18.472 percentage points |

Including travelers, the adaptive closure effects are +159.32, +170.72 and
+190.09 people; fixed-share effects are −35.97, −17.22 and −32.19. Thus the sign
reversal is not an artifact of counting only settled residents.

Maximum sampled normalized economy residual: **2.589e-6**; ecological relative
C/N/P error: **3.680e-6**; population relative error: **1.943e-7**; normalized shared
source residual: **0** (corrected unit label; archived value unchanged). These are measured errors, not the looser 0.001
failure threshold. No-op equivalence covers full social serialization plus
sampled environmental budgets, not every GPU byte.

This final run took 35.1 seconds for CPU tests, 284.0 seconds for GPU tests and
98.9 seconds for the experiment. These are single-run wall times after earlier
compilation/shader-cache warmup, not performance benchmarks or confidence intervals.

Committed evidence:

- [Generated comparison tables](evidence/mine-staffing-tables.md).
- [Artifact retention policy](evidence/README.md): all
  sites and branches, not a selection of favorable runs. About 12.5 MB compressed.
- [Artifact retention policy](evidence/README.md): source/catalog and
  artifact hashes, exact commands, parameters, compiler, adapter and stage outcomes.

The raw decompressed JSON hash must equal `artifacts_sha256["mine/results.json"]`
in the manifest. Logs and checkpoint binaries remain in
`output/evidence-c136a2a/`; the command regenerates equivalent artifacts in a new
output directory. Timings, absolute paths and manifest timestamps naturally vary.
The report/data-only commit follows the recorded simulation revision and does
not change its rules. CPU CI is configured but a remote CI success is not claimed
by this local GPU execution.


## Executed coverage by test binary

These are fixture counts, not measured code coverage or proof of every requirement.
CPU-skipped hardware cases were subsequently executed in the hardware stage.

| Binary | CPU passed | Hardware passed | Hardware ignored |
|---|---:|---:|---:|
| `src/lib.rs` | 16 | 15 | 0 |
| `tests/alloy_processing.rs` | 0 | 2 | 0 |
| `tests/civilization.rs` | 0 | 2 | 0 |
| `tests/core.rs` | 5 | 0 | 0 |
| `tests/culture.rs` | 4 | 9 | 0 |
| `tests/discoveries.rs` | 0 | 4 | 0 |
| `tests/ecology.rs` | 3 | 13 | 0 |
| `tests/economy.rs` | 1 | 13 | 0 |
| `tests/environmental_returns.rs` | 0 | 1 | 0 |
| `tests/expeditions.rs` | 0 | 4 | 0 |
| `tests/geology.rs` | 2 | 3 | 0 |
| `tests/governance.rs` | 0 | 3 | 0 |
| `tests/gpu.rs` | 0 | 10 | 0 |
| `tests/housing.rs` | 1 | 1 | 0 |
| `tests/living.rs` | 0 | 2 | 0 |
| `tests/markets.rs` | 5 | 2 | 0 |
| `tests/mineral_processing.rs` | 0 | 1 | 0 |
| `tests/politics.rs` | 0 | 4 | 0 |
| `tests/rations.rs` | 1 | 1 | 0 |
| `tests/resources.rs` | 0 | 1 | 0 |
| `tests/shipping.rs` | 0 | 1 | 0 |
| `tests/society.rs` | 0 | 2 | 0 |
| `tests/storage.rs` | 1 | 1 | 0 |
| `tests/waterworks.rs` | 2 | 1 | 0 |
