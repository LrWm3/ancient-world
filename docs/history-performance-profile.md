# Mature history performance profile

Measured 2026-09-12. This pass profiles the current living-history workload,
identifies repeated CPU work, and checks that removing it preserves outcomes.
It is a diagnostic history benchmark at terrain/ecology edges 32/16, not a
performance claim for default-resolution world generation or the interactive UI.

## Workload and tools

Baseline source: `0e353f5` (profiling harness added to `29f9e65`).
Optimized development build, Rust 1.89, `opt-level = 1`, debug symbols.
Intel i7-10750H (6 cores / 12 threads), Quadro RTX 5000 Max-Q 16 GB,
NVIDIA 595.84, Vulkan. A separate user application remained active on the GPU;
our own builds and measured repetitions run separately. Results are local
observations, not isolated-machine guarantees or cross-hardware comparisons.

The initial run creates seed 256 and advances 100 years with sixteen founders,
yield scale 0.5, individual demography, workshop/agriculture/extraction/construction
refinement, political household distribution and family support. It completed
in **425.49 seconds** of runner time (426.24 seconds process wall time),
ending at **2,511 residents**, with peak process RSS **802,132 KiB**.
All recorded simulation results match the preceding council-funding run,
excluding elapsed time. The saved century boundary supplies identical inputs
for each continuation; setup does not regenerate a different world.

Profilers:

- Linux `perf 7.0.12`: user-space cycles at 199 Hz, DWARF call stacks.
- Nsight Systems `2022.4.2.50`: Vulkan API/workload and OS runtime trace,
  CPU sampling disabled.
- Existing monthly stage wall timers and Linux process peak RSS.
- History serialization and terrain/ecology buffer fingerprints outside the
  timed continuation loop.

The first continuation of each newly located executable can spend approximately
39 seconds in setup. Shader/driver caching is a likely contributor, but this
trace does not isolate every startup component. Warm executables before comparing
steady runs. The original cold 120-month continuation took 115.74 seconds;
it must not be compared directly with a warmed optimized run as a code speedup.

## Measured bottlenecks and changes

Before optimization, a 60-month CPU profile collected 7,518 samples:

| Operation | Sampled user cycles | Interpretation |
| --- | ---: | --- |
| `person_presence` | 59.85% self | Repeated linear household and kin searches |
| `Participation::validate` | 8.86% inclusive | Rescanning all commitments for every resident |
| `open_participation` | 28.86% inclusive | Includes presence lookups |
| `population_reconciliation` | 17.06% inclusive | Includes presence lookups |

Inclusive shares overlap and must not be added together. Sampling covers the
process, including loading and final reporting, whereas monthly wall timers
cover the continuation loop. These percentages are not wall-time allocations.

Two bounded changes address that evidence:

1. Immutable bulk observations build a fresh membership index, preserving
   first-head-before-first-kin lookup precedence. Participation opening,
   population reconciliation, domestic home lookup, care planning and demographic observation
   use it. Death, expedition, military, partial relocation and abandoned-site
   rules remain shared with the scalar lookup. The index never survives a
   function call or a state mutation.
2. The participation audit accumulates commitment totals in one ordered pass,
   then checks residents against them. Per-person floating-point addition order
   is unchanged. Invalid identities, nonfinite amounts, overspending and
   inconsistent ledgers still fail validation.

A second profile after these changes still assigned 49.60% of sampled cycles to
scalar presence lookup and 36.45% inclusive to participation opening. Extending
the local index into care planning, and summing care assignments once for
participation opening, reduced the loop further. Individual departure decisions
still query current state; they do not reuse the opening index.

No scheduler phase, reservation priority, demographic rule, random stream,
archive schema or GPU kernel changes. Transaction rollback and full validation
remain enabled.

The before Vulkan trace spans checkpoint loading, 24 monthly steps and reporting.
It contains 3,404 GPU workload records totaling **0.227 seconds** of recorded
workload duration. Vulkan semaphore waits total **0.188 seconds**; memory
allocation/free APIs total **0.394 seconds**. These figures support prioritizing
CPU history work on this small-grid workload. They do not establish that
readback, ecology or GPU allocation is cheap at larger resolutions.
OS condition-variable waits overlap across threads and include idle workers;
summing them does not measure main-thread blocking.

## Repeated continuation result

Three warmed, unprofiled runs per version each advance months 1201–1260 from
the same checkpoint. Baseline and first-pass runs alternate order; final-pass
runs include an additional original-executable control (37.35 seconds) between
the first and second measurements.

| Measure | Before | First pass | Final, including care |
| --- | ---: | ---: | ---: |
| Loop time, three runs (seconds) | 37.78 / 37.16 / 37.93 | 23.60 / 23.57 / 23.43 | 21.14 / 21.34 / 21.64 |
| Median loop time | 37.78 s | 23.57 s | 21.34 s |
| Median setup stage | 0.651 s | 0.615 s | 0.612 s |
| Median production/readback stage | 2.710 s | 1.138 s | 1.121 s |
| Median social/validation stage | 34.060 s | 21.444 s | 19.281 s |
| Peak process RSS range | 323,812–332,808 KiB | 327,744–357,732 KiB | 337,612–339,892 KiB |

The final median continuation uses **43.5% less elapsed time (1.77× throughput)**.
Most of the reduction is in CPU social/validation work. The production/readback
wall stage also changed, but no GPU algorithm changed; this stage includes host
work and synchronization, and the shared adapter's clock/load is uncontrolled.
Do not interpret that difference as a kernel optimization. Independent stage
medians need not sum exactly to the median loop time.

There is no demonstrated memory reduction. Fresh index storage is linear in the
number of historical people and is discarded after each bulk observation.
Process RSS also includes archive decoding, reporting and driver allocations.

All nine repetitions and the additional baseline control finish with 2,547 residents, 8,074 historical people and
69,628 events. Their fingerprints match:

| State | FNV-1a fingerprint |
| --- | --- |
| Serialized history | `be885de846a2f8d1` |
| Terrain cells | `afb71557d87083eb` |
| Ecology cells | `523b67e0f5619033` |

These are regression fingerprints, not cryptographic proofs. The fixed fixture
tests additionally compare bulk lookup results with the scalar path directly.

## Remaining bottlenecks

The final 60-month CPU profile collected 4,283 samples and timed 21.20 seconds:

| Operation | Final sampled user cycles |
| --- | ---: |
| Scalar `person_presence` | 49.57% self |
| `open_participation`, including remaining care work | 29.40% inclusive |
| `relocation_departures_observed` | 15.77% inclusive |
| `reserve_agriculture` | 10.16% inclusive |
| `sync_domestic` | 5.28% self |
| `genealogy_month` | 3.45% self |
| `prepare_enterprises` | 3.20% self |
| `History::validate` | 4.00% inclusive |

The whole run is shorter; a large remaining percentage is not evidence that
the function got slower. Participation validation is no longer among the
inclusive entries above the 3% reporting threshold.

Next candidates are repeated household-roster construction, care projections
and neighbor help, recruitment and cultural membership queries. Several of
these run inside loops that mutate identities or commitments. Extend local
indexes only across explicitly immutable windows; do not substitute yesterday's
participation state for current presence. Domestic grouping and genealogy still
grow with historical records. Profile those callers before changing storage or
discarding any history.

The final Nsight trace has the same **3,404 workload records**, **222 semaphore
waits** and **224 allocations/frees** as the baseline. Recorded GPU workload
duration is **0.227 seconds** again; semaphore API waits total **0.169 seconds**.
The traced 24-month loop changes from **14.87 to 8.40 seconds**, with matching
history/terrain/ecology fingerprints. The evidence points to reduced CPU work,
not fewer GPU dispatches or a new transfer optimization.

Further work should separately measure default-resolution ecology/terrain,
release builds, larger or older populations, the UI, startup and an otherwise
idle machine. This pass does not extrapolate its 1.77× improvement to those
workloads. Persistent global caches, less frequent validation and removal of
rollback copies were deliberately avoided.

## Reproduction

Keep generated archives, executables, JSON, profiles and logs under ignored
`output/`. Only this human-readable summary is committed.

Build both revisions with the same toolchain/profile and preserve each executable
before rebuilding. Create the shared checkpoint with the baseline executable:

```sh
cargo build --example cultural_work_calibrate --example history_profile --example history_replay
mkdir -p output/history-perf
cargo run --example cultural_work_calibrate -- \
  --seeds 256 --years 100 --resolution 32 --crop-yield-scale 0.5 \
  --individual-demography --workshop-refinement --agriculture-refinement \
  --extraction-refinement --construction-refinement --compare-resolution \
  --household-diagnostics --family-support --save-final \
  --output output/history-perf/baseline-century.json
```

The checkpoint is `output/history-perf/baseline-century.256.world`.
`--save-final` saves after the timed run. Run a warmup, then three 60-month
continuations of each executable in alternating order:

```sh
target/debug/examples/history_profile \
  output/history-perf/baseline-century.256.world 60 output/history-perf/run.json

perf record -o output/history-perf/cpu.data -e cycles:u -F 199 \
  --call-graph dwarf,16384 -- target/debug/examples/history_profile \
  output/history-perf/baseline-century.256.world 60 output/history-perf/perf.json
perf report -i output/history-perf/cpu.data --stdio --no-inline \
  --no-children -g none --percent-limit 1 --sort comm,dso,symbol

nsys profile --trace=vulkan,osrt --sample=none --cpuctxsw=none \
  --vulkan-gpu-workload=individual --force-overwrite=true \
  -o output/history-perf/vulkan target/debug/examples/history_profile \
  output/history-perf/baseline-century.256.world 24 output/history-perf/nsys.json
nsys stats --report vulkanapisum,osrtsum output/history-perf/vulkan.nsys-rep
```

On this machine the locally unpacked Nsight collector required a separate
`QdstrmImporter` invocation from its host-tools directory. An ignored local
`libssh.so` symlink to the installed `libssh.so.4`, supplied through
`LD_LIBRARY_PATH`, resolved the importer's missing library. No system installation
was changed. Default perf inline-symbol resolution was slow; `--no-inline`
made reports practical. Neither reporting issue invalidated the captured traces.

## Verification commands

Run hardware checks explicitly; they are ignored by the ordinary test command:

```sh
cargo test --lib
cargo test --lib participation::tests -- --ignored --test-threads=1
cargo test --lib domestic::tests -- --ignored --test-threads=1
cargo test --test history_environment frozen_schedule_batch_and_checkpoint_equivalence \
  -- --ignored
cargo run --example history_replay -- output/history-perf/baseline-century.256.world \
  output/history-perf/replay.json
cargo clippy --all-targets -- -D warnings
python3 scripts/check_repository_artifacts.py
```

The new presence fixture checks household/kin precedence, military/expedition/death
precedence, partial relocation, abandonment, lost households and civilization
leader fallback against scalar queries. It rebuilds observations after mutations.
The participation fixture also deliberately corrupts a committed total, inserts
an invalid person ID and introduces NaN; validation must reject each case.

During the second optimization, fingerprint comparison caught a signed-zero
difference: Rust's empty float sum starts at `-0.0`, whereas the new care accumulator
initially used `+0.0`. The care reduction now retains that initial value whenever
a current care plan exists, and its reference check compares float bits rather
than numerical equality. The rejected run is not included in final timings.

Completed verification:

- 134 ordinary library tests passed; 115 hardware/long tests remain ignored by
  that ordinary command.
- Both participation GPU fixtures and both domestic-care GPU fixtures passed
  explicitly, including the new lookup and signed-zero checks.
- The full frozen scheduler fixture passed monthly/batched/checkpoint comparisons
  on seeds 17, 81 and 256.
- The mature century checkpoint passed 24-month uninterrupted versus
  12 monthly steps, save/reload, and 12 more monthly steps. Serialized history
  matches exactly; terrain and ecology snapshots are byte-identical.
- All-target Clippy passed with warnings denied.
- Repository artifact policy and whitespace checks passed.

This is verification of the optimization and its measured workload, not a new
ecological or historical balance calibration.
