# Lake convergence polling review

The compact lake solver still spends most of its time executing local relaxation.
This follow-up tests a smaller change before attempting a new solver or an
incremental desktop state machine.

`Config.lake_poll_passes` permits 16, 32, 64 or 128 GPU passes per convergence
readback. Default and legacy archives retain 16. All intervals and supported
iteration limits are even, preserving the ping-pong scratch side. The final batch
is shortened to the remaining iteration budget. Neither the water-flux equations,
stopping tolerance nor explicit unresolved failure changes. Larger batches may
perform extra below-tolerance passes before observing convergence, so different
intervals do not promise identical completed worlds.

The replay example accepts `--lake-poll-passes` and optional `--output-cells` for
local field comparison. Raw fixtures and results belong under ignored `output/`.

## Fixed-work comparison

On the Quadro RTX 5000 Max-Q / Vulkan, replay the previously captured seed-42
terrain-512 unresolved lake state for **4,096 passes**, with ecology resolution
256 configured but no ecological advancement. Three repetitions of each interval
alternate order (16/64, 64/16, 16/64). Our council evaluation processes are paused
for these measurements and resumed afterward; no Cargo compilation overlaps.
The other user's background GPU application remains active.

| Poll interval | Readbacks | Lake wall times | Median |
| --- | ---: | --- | ---: |
| 16 | 256 | 16.251 / 16.286 / 16.260 s | 16.260 s |
| 64 | 64 | 16.176 / 16.158 / 16.182 s | 16.176 s |

This is **75% fewer convergence readbacks** but only **0.52% lower median solver
time**. It is not evidence of a material application speedup. Initialization and
final full fixture snapshots are excluded from the solver timer in both arms.
The two final 264 MiB cell outputs have identical SHA-256
`2d5b3988e52bb4f57cb5017e159ed0b413d18c9de98aa591b3b72936759cf30b`.

All six deliberately limited runs fail explicitly: 54 cells remain above the
unchanged tolerance, maximum reported change 0.0009651184 m. Relative global water
volume error is 5.18e-12. This global measure includes large terminal reservoirs;
it is not a substitute for the small closed-pool conservation fixture. These are
fixed-work timings of unfinished relaxation, **not completed generation times**.

A separate completed generation check uses default terrain 512 / ecology 256,
seed 42, one geological epoch and ten ecological years. Interval 16 converges at
3,680 passes; interval 64 at 3,712, both with zero above-tolerance cells. Only 207
of 1,572,864 terrain cells differ, exclusively in water depth; maximum difference
is **0.005852 m**. All other terrain fields match bit-for-bit and both water fields
remain finite and nonnegative. Extra below-tolerance passes accumulate small depth
changes, as expected. These generation checks overlap history tests/runs and are
validation cases, not uncontended timing measurements. They do not validate a
multi-epoch default change.

The seam fixture compares intervals 16/32/64/128, retains the dry barrier and water
budget, and keeps final local depths within 0.002 m. It also checks that a 16-pass
limit is never enlarged by a larger polling interval. Configuration tests reject
unsupported intervals and preserve the old default.

Reproduce using the capture instructions in
[default-generation-performance.md](default-generation-performance.md), then:

```sh
cargo run --example lake_relaxation -- output/generation-perf/failed-lakes-original.bin \
  --resolution 512 --max-lake-iterations 4096 --lake-poll-passes 16 \
  --output-cells output/generation-perf/poll-16.bin
```

Repeat with 64. An unresolved error is expected for this particular captured state;
a fresh converged capture is not equivalent benchmark input. The checksum above
identifies the retained local fixture replay, not a promise that recapturing with a
later solver version reproduces the original input byte-for-byte.

## Decision and remaining responsiveness work

Retain interval 16 by default. The optional control is useful for experiments,
but changing polling alone does not justify an advertised performance improvement.
No physical time is advanced by these equilibrium passes.

`equilibrate_lakes` remains synchronous. A real responsiveness change needs a
transient solve state containing source-buffer identity, scratch parity, iteration
count, budget and pending flag readback. `advance` could then dispatch one bounded
batch, yield, and resume after asynchronous mapping. Terrain mutation/restore must
invalidate that transient state; normal archives should wait for final scatter and
a consistent boundary. The viewer must not present unfinished water as a completed
epoch. This is a larger change than increasing dispatch batches and is not claimed
implemented here. A faster basin/frontier solver separately needs tests for newly
wetted cells, saddles, overflow, seams and conservation before replacing relaxation.
