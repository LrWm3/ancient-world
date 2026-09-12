# Default-resolution generation profile

Measured 2026-09-12 at source `9bd4416`, following the
[mature-history profile](history-performance-profile.md). This measures natural
world generation, before founding civilizations.

**Result:** the first epoch completes in a median 42.83 seconds, but a three-epoch
attempt fails during secondary-lake equilibration at the end of the second epoch.
The lake solver is both a major cost and the next correctness/robustness priority.

## Configuration and method

- Default terrain edge **512**, ecology edge **256**: 1,572,864 terrain cells
  and 393,216 ecological cells across six cube faces.
- Default seed **42**, one geological epoch with **ten ecological years**
  (120 monthly ecology steps). A separate three-epoch run checks continuation.
- Default physical, climate, drainage and ecological settings; no reduced
  climate cycles, relaxed convergence tolerances or disabled ecology.
- Headless CLI, no PNG or archive export, no civilization history.
  The CLI's final terrain/ecology validation and ecological budget readbacks
  remain included in reported elapsed time.
- Rust 1.89 optimized development build (`opt-level=1`), Intel i7-10750H,
  Quadro RTX 5000 Max-Q 16 GB, NVIDIA 595.84, Vulkan.
- Three sequential unprofiled one-epoch runs, followed by separate Nsight
  Systems 2022.4.2.50 and Linux perf 7.0.12 runs. Builds and our measured
  processes do not overlap. Another user's GPU application remains active;
  results are not isolated-adapter guarantees.

“One epoch” is the headless CLI default. It is not the desktop's configurable
1,000-epoch history target. Short-run measurements should not be multiplied
by 1,000 to promise a full-history completion time.

GPU timestamps are supported. `ecological_gpu` contains the `eco/*` kernels;
`ecology_month_wall` includes their execution and host scheduling. Do not add
these overlapping totals. `secondary_lakes_wall` measures the separate
post-ecology surface-water relaxation, including convergence checks.

## One-epoch results

| Measure | Result |
| --- | ---: |
| Three elapsed times | 45.14 / 42.83 / 42.49 s |
| Median elapsed time | **42.83 s** |
| Ecology monthly wall time, median | 20.15 s |
| Ecology GPU total, median | 19.92 s |
| Secondary-lake relaxation wall time, median | 16.90 s |
| Transport GPU time, inside ecology | 9.44 s |
| Biology GPU time, inside ecology | 5.35 s |
| Terrain climate GPU time | 1.71 s |
| Terrain-to-ecology aggregation GPU time | 1.58 s |
| River collection / routing GPU time | 1.24 / 0.97 s |
| Drainage / basin / flow relaxation GPU time | 0.47 / 0.40 / 0.33 s |

Ecology and post-ecology lake relaxation account for approximately 86% of the
median elapsed time. Transport and biology are included in ecology, not extra
costs. The first run has more CPU/startup work; the two subsequent runs differ
by less than 1%. These are three observations, not a confidence interval.

Peak host RSS was **816,452 / 747,244 / 747,176 KiB**. The configured GPU allocation
estimate is **1,376 MiB**; a process-specific `nvidia-smi` observation during
generation reported **1,462 MiB**. That observation is not a continuously measured
peak. The other application used a separate 120 MiB.

Every one-epoch repetition completed terrain/ecology validation and reported the
same relative C/N/P residuals:
**6.53e-7 / 1.95e-6 / 1.52e-8**, below the 0.1% reporting tolerance.
No convergence limits were weakened. These checks establish completion and
budget validity, not full-state equivalence between seeds or hardware.

## Multi-epoch failure

`--headless --epochs 3` completed epoch one and the second epoch's 120 ecology
months, then exited with:

```text
Error: secondary lake surface flow unresolved after 16384 iterations
```

Process wall time was **158.07 seconds**, exit status **1**. The third epoch never
started, and final CLI validation/budget reporting was not reached. This is a
failed continuation, not a three-epoch timing result.

The cap is the lake solver's own 1,024 batches × 16 passes in
`Generator::equilibrate_lakes`, separate from configurable drainage iterations.
The run does not establish whether this is very slow physical relaxation,
numerical oscillation or a persistent local residual. Raising the cap or relaxing
the threshold without that diagnosis would conceal the problem.

Next investigation should capture changed-cell counts, maximum water/head changes
and the remaining active basin at successive batches, while checking water balance.
Then distinguish an iteration-efficiency problem from a convergence criterion or
floating-point problem. A compact-buffer optimization should be tested against
that case as well as the existing two-basin, saddle and conservation fixtures.

No solver change is included in this measurement pass. The failure is reproduced
by the command above in the tested build; it has been observed once in this
multi-epoch run, not established across a seed ensemble.

## Profiler observations

The Nsight run took **42.58 seconds**. It recorded:

- 764 `vkWaitSemaphores` calls totaling **39.47 seconds**.
- 773 allocations totaling **1.16 seconds**, and 773 frees totaling **0.50 seconds**.
- 2,511 queue submissions totaling **0.037 seconds of host API time**.

Wait time mostly overlaps the requested GPU work; it is not a 39-second saving
available by deleting readbacks. Batching convergence checks is already present.

There is a trace limitation: GPU workload records overlap. Their naive duration
sum is 57.16 seconds, exceeding wall time; their interval union is 34.08 seconds.
The recorded GPU interval ends at trace second 35.91 while Vulkan API activity
continues to 42.79. Do not treat this old Nsight/driver combination's workload
records as complete GPU utilization coverage. The generator's per-stage GPU
timestamps and wall timers are the main kernel-cost evidence.

The separate user-cycle perf profile collected only 321 samples, so its
percentages are coarse indications. It attributes 20.47% to memory copying,
10.26% to ecological validation and 6.56% to the ecological budget report.
Those are shares of sampled **CPU cycles**, not of elapsed generation time.
The two warmed unprofiled runs used only 1.03–1.15 seconds of user CPU and
2.33–2.45 seconds of system CPU. Removing validation would weaken the benchmark
while leaving its main GPU costs intact.

## What the code and timings identify

This workload has different bottlenecks from mature civilization history.
Secondary-lake equilibration and ecological transport/biology dominate; the
person-membership optimization does not run before civilization founding.

The secondary-lake solver in `Generator::equilibrate_lakes` runs batches of
16 GPU `pool_step` passes and reads a small convergence flag after each batch.
Each pass reads neighbor terrain/water and writes a complete 176-byte terrain
cell. At default resolution that is nominally **264 MiB of destination cell writes
per pass**, before counting neighbor reads. This is a credible candidate for
a compact water-only scratch representation, analogous to the existing compact
drainage scratch. It is a candidate, not a measured optimization in this pass.

Any such change must preserve area-weighted transfers, dry-saddle barriers,
flat-water convergence and the final terrain buffer. Simply skipping the
solver, accepting unfinished relaxation or loosening tolerances would change
the model. An active-cell approach also needs a conservative frontier: a
currently dry cell can receive overflow, so it cannot be omitted permanently.

Transport and biology should be profiled at kernel level before further
rewrites. Their current durations include a much larger ecological model than
the historical September 6 benchmark. The old timings in
[benchmarks.md](benchmarks.md) are not an equivalent before/after baseline.

## Reproduction

```sh
mkdir -p output/generation-perf
cargo build --bin ancient-world
/usr/bin/time -v target/debug/ancient-world --headless --epochs 1 \
  > output/generation-perf/run1.log 2>&1
# Repeat sequentially as run2.log and run3.log.

/usr/bin/time -v target/debug/ancient-world --headless --epochs 3 \
  > output/generation-perf/three-epochs.log 2>&1

nsys profile --trace=vulkan,osrt --sample=none --cpuctxsw=none \
  --vulkan-gpu-workload=individual --force-overwrite=true \
  -o output/generation-perf/vulkan target/debug/ancient-world \
  --headless --epochs 1
nsys stats --report vulkanapisum,osrtsum output/generation-perf/vulkan.nsys-rep

perf record -o output/generation-perf/cpu.data -e cycles:u -F 99 \
  --call-graph dwarf,16384 -- target/debug/ancient-world --headless --epochs 1
perf report -i output/generation-perf/cpu.data --stdio --no-inline \
  --no-children -g none --percent-limit 2 --sort comm,dso,symbol
```

The local Nsight importer workaround is documented in the mature-history report.
Raw logs, SQLite exports and binary traces stay ignored under `output/`.
No generated artifacts are committed.
