# Default-resolution generation profile

Measured 2026-09-12 at source `9bd4416`, following the
[mature-history profile](history-performance-profile.md). This measures natural
world generation, before founding civilizations.

**Original baseline:** the first epoch completes in a median 42.83 seconds, but a three-epoch
attempt fails during secondary-lake equilibration at the end of the second epoch.
The follow-up below addresses that reproduced failure; slow lake relaxation
remains an important performance limitation.

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

## Original one-epoch results

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

## Original multi-epoch failure

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
At the time, the run did not establish whether this was very slow physical
relaxation, numerical oscillation or a persistent local residual. The follow-up
below distinguishes those possibilities before changing the iteration budget.

## Lake-solver follow-up

The original failure was reproduced with late-pass diagnostics and a local raw
terrain capture. At pass 16,384, **23 cells** still exceeded the unchanged stopping
tolerance, with maximum depth change **0.0008507 m**. An additional 16,384 compact
passes did not immediately converge either. However, the principal active pool
rose approximately **0.85 m** over that interval. Its basin (3,351 terrain cells)
retained area-weighted water to about **6.3e-8 relative error** in a diagnostic
replay. The apparent plateau in changed-cell counts was not evidence of stationary
floating-point noise: perched water was still feeding lower water through narrow
connections. A rearranged head calculation did not remove that behavior and was
not retained.

The implementation now:

- Initializes two reusable 16-byte scratch tuples per cell from the terrain,
  relaxes only those tuples, and scatters water back once. It reuses the existing
  drainage/reduction scratch allocation, adding no large GPU buffer. Nominal
  per-pass destination writes drop from **264 MiB to 24 MiB** at terrain 512;
  neighbor and static-terrain reads remain additional traffic.
- Calculates spherical area weights once per solve rather than repeatedly for
  every directed edge on every pass. The same finite-volume flux, nonnegative
  outflow bound, dry-saddle barrier and convergence tolerance remain in use.
- Replaces the hardcoded 16,384-pass ceiling with `max_lake_iterations` in the
  TOML configuration. Zero (also the archive/default value) selects
  `max(16384, face_resolution²)`: 65,536 at 256, 262,144 at 512, and 1,048,576 at
  1024. This is a bounded engineering allowance for slow grid relaxation, not
  a convergence guarantee. Explicit values must be multiples of 16 between 16
  and 1,048,576. Completed solves stop early, as before.
- Reuses one 16-byte convergence staging buffer per solve and encodes the flag
  copy into the existing dispatch batch. This avoids allocating/freeing a staging
  buffer and submitting a separate copy command for every convergence check.
  Host readback remains 16 bytes per 16 passes; no terrain readback is added.
- Reports the final unresolved cell count and maximum above-tolerance depth
  change in progress, headless reports, desktop diagnostics and failure messages. It still fails
  explicitly at its limit and scatters the latest conservative state even on
  failure. The maximum is zero when no cell exceeds tolerance; it is not a claim
  that all remaining numerical flux is exactly zero.

These iterations solve a game-model equilibrium and do not advance extra physical
or ecological months. The longer allowance does not silently accept unfinished
water movement. This remains local relaxation, not a basin-wide direct solver;
large, poorly connected lakes can still be expensive. A whole solve remains
synchronous; incremental advancement is a separate responsiveness improvement.
An active-cell optimization
must retain a frontier for newly wetted cells and downstream overflow.

### Follow-up verification and timings

The default **512 terrain / 256 ecology**, seed **42**, three-epoch continuation
completed with exit status zero, including final terrain/ecology validation:

- Generation elapsed **432.80 s** (process wall **433.10 s**).
- Secondary lakes **354.03 s**; ecology monthly wall **62.00 s**.
- C/N/P relative residuals **2.81e-6 / 5.32e-6 / 2.62e-8**.
- Peak host RSS **784,564 KiB**; process GPU memory sampled at **1,462 MiB**,
  unchanged from the original sample. This is not a continuous GPU peak.

This is successful continuation, not a speedup comparison against the old
158-second failed run. The final part of this validation overlapped a CPU test
build; use the separate unprofiled first-epoch measurements for performance.
A live GDB attachment attempt was unavailable under the host ptrace restrictions;
no conclusion here depends on that attempt.

Three sequential, unprofiled first-epoch runs of the compact-buffer version (same configuration
and hardware, no overlapping builds or our other GPU runs) measured:

| Measure | Original median | Compact solver median |
| --- | ---: | ---: |
| Generation elapsed | 42.83 s | **40.69 s** |
| Secondary lakes | 16.90 s | **14.71 s** |
| Ecology monthly wall | 20.15 s | 20.23 s |

Individual elapsed times were **42.42 / 40.38 / 40.69 s**, with **3,680 lake
passes** and zero unresolved cells in each run. This is about **5.0% less total
time** and **12.9% less lake time**, not an order-of-magnitude acceleration.
All three retained the original first-epoch C/N/P residuals listed above.
Peak host RSS was **778,800 / 746,980 / 747,144 KiB**. The background GPU application
remained active, and three repetitions do not establish a confidence interval.

Additional default-resolution, one-epoch validation runs:

| Seed | Elapsed | Lake time | Lake passes | Final unresolved cells |
| --- | ---: | ---: | ---: | ---: |
| 17 | 40.40 s | 13.50 s | 3,376 | 0 |
| 81 | 52.14 s | 25.58 s | 6,368 | 0 |

Both passed final validation. Their largest C/N/P residuals were **1.97e-6** and
**1.92e-6**, respectively. These are short checks, not multi-epoch ensemble
validation or guarantees for the desktop's 1,000-epoch target. Basin geometry still
causes material timing variation. No 1024-grid run or cross-backend comparison was
performed in this follow-up.

Verification:

- **135 ordinary library tests passed**, 115 hardware tests remained ignored in
  that invocation; Clippy over all targets passed with warnings denied.
- **All 11 `tests/gpu.rs` hardware tests passed**: analytical/CPU-reference
  drainage, closed-pool and unequal-area seam water conservation, dry and flooded
  saddles, spill-limited overflow, water/sediment accounting, climate convergence,
  regional drainage, and small-grid seed/long-run checks. The geography fixture
  covers seeds 0, 42 and 999 at terrain 64, then advances the last world to epoch 31.
- The hardware checkpoint test produced byte-identical continued terrain and
  equal planet reservoirs after save/load. The new budget-boundary fixture forces
  a 16-pass failure, checks its residual diagnostics and retained volume, resumes
  the saved partial **lake** state, and verifies that unrelated cell fields remain
  unchanged. This is not a promise of world-archive recovery from a mid-epoch error.
- The living-history readback fixture passed, checking that lake updates still
  share one fresh terrain snapshot per month with the history/environment path.

Original baseline values above remain historical measurements, not claims about
the revised solver.

### Staging-buffer refinement

The first compact-solver Nsight run took **40.60 s**, with **808 Vulkan waits**
totaling **37.33 s**, **817 allocations** totaling **1.245 s**, and **817 frees**
totaling **0.503 s**. Queue-submit host time was only **0.038 s**. GPU work still
dominates, but the allocation trace motivated reusing the tiny lake convergence
staging buffer instead of allocating it for every batch.

The three-epoch and extra-seed checks above precede this readback-only refinement;
the full hardware suite and first-epoch timings are repeated after it. Water
arithmetic, convergence tolerance and the number of passes per batch are unchanged.
The ordinary library checks are unaffected; the final all-target Clippy check and
all eleven hardware integration tests pass again.

Final unprofiled repeats after staging reuse measured **40.40 / 40.30 / 40.26 s**,
median **40.30 s**; lake time median **14.59 s**. Relative to the original baseline,
that is approximately **5.9% less total time** and **13.7% less lake time**.
Each used 3,680 lake passes and retained the same first-epoch C/N/P residuals.
Peak host RSS was **744,920 / 745,228 / 745,356 KiB**. The improvement beyond compact
buffers alone is small compared with the variation between runs; allocation and
submission counts are the firmer evidence for the staging cleanup.

The final Nsight run took **40.44 s**. Compared with the first compact version:

| Vulkan operation | Compact | With staging reuse |
| --- | ---: | ---: |
| Memory allocations / frees | 817 / 817 | **588 / 588** |
| Queue submissions | 2,601 | **2,371** |
| Convergence/other semaphore waits | 808 | 808 |

Allocation/free API time fell from **1.245 / 0.503 s** to **0.917 / 0.362 s**.
Wait time was **37.70 s**, mostly overlapping GPU work, not a removable CPU cost.
The trace supports the reduction in allocation/submission churn; it does not
establish a large additional speedup. The diagnostic capture/replay example also
passed: a deliberate 16-pass failure retained its fixture, and replay converged in
32 further passes with global water error **-9.63e-13** (subject to the global-budget
caveat below).

The lake wall timer ends when the final scatter is submitted; subsequent CLI
validation waits for that scatter as part of the reported total generation time.
The small final scatter tail is not separately timed.

### Replaying the diagnosis

The diagnostic example captures raw current-layout cells on success **or error**,
then returns the actual generation result. A fixture is not a portable world
archive: use the matching source, catalog, seed and resolution. Keep it local.

```sh
cargo build --example lake_relaxation
# Deliberately restore the old ceiling to capture the difficult boundary.
target/debug/examples/lake_relaxation output/generation-perf/lake-fixture.bin \
  --resolution 512 --seed 42 --capture-epochs 2 --max-lake-iterations 16384
# Replay just the saved surface-water solve using the new automatic budget.
target/debug/examples/lake_relaxation output/generation-perf/lake-fixture.bin \
  --resolution 512 --seed 42
```

The replay prints area-weighted global water error. Large terminal reservoirs can
mask small local errors in that number; the GPU fixtures separately test closed
pools, saddle transfers and unequal-area cube seams.

## Original profiler observations

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

## Original optimization candidates

This workload has different bottlenecks from mature civilization history.
Secondary-lake equilibration and ecological transport/biology dominate; the
person-membership optimization does not run before civilization founding.

In the original baseline, `Generator::equilibrate_lakes` ran batches of
16 GPU `pool_step` passes and reads a small convergence flag after each batch.
Each pass reads neighbor terrain/water and writes a complete 176-byte terrain
cell. At default resolution that is nominally **264 MiB of destination cell writes
per pass**, before counting neighbor reads. The follow-up above implements a compact scratch representation. These traffic
estimates alone are not a measured wall-time speedup.

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

Use source `9bd4416` for the original baseline; the current checkout measures
the follow-up. Keep profiler captures and generated fixtures out of Git.

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
