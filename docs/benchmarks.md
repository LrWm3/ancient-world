# Measured GPU performance

Measured 2026-09-06 on the 16 GB Quadro RTX 5000 with Max-Q Design, NVIDIA driver 595.84, Vulkan, Rust 1.89, optimized development build. Each run creates seed 42 and advances one geological epoch plus ten ecological years (120 months), with 256² ecology. Both columns use the refined terrain, geology, lake solver, nutrient recovery/excretion and catalog calibration. These historical timing runs preceded the stored-lake evaporation correction, true regional generation and the subsequent neighbor-based basin solver. They do not measure the current hydrology build. An isolated reference build restored the original transport, full-cell drainage, neighbor-only basin labeling and redundant aggregation; the comparison isolates these optimizations.

| Terrain edge | Before wall seconds | After wall seconds | Before ecology GPU seconds | After ecology GPU seconds | Estimated MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| 256 | 14.25 | 10.62 | 10.72 | 7.20 | 620 |
| 512 | 18.19 | 11.59 | 13.48 | 9.30 | 1088 |
| 1024 | 55.59 | 30.30 | 26.59 | 19.11 | 2960 |

These are single-run observations, not guaranteed generation times or statistical confidence intervals. Wall time includes pipeline creation, initialization, synchronization and scheduling; driver shader caches and clock state can affect it. GPU timestamps provide the more direct kernel comparison. Hardware and backend portability still need testing on additional machines. Estimated memory includes simulation buffers and a display allowance, not host archive copies or all driver allocations.

## Optimization iterations

1. **Rejected:** distributing each transport compartment to separate invocations increased 256² transport plus settlement time to 10.77 s from 7.98 s. It was reverted.
2. **Retained:** fixed compartment accesses reduced transport to 5.72 s; skipping completely empty exchanges reduced it further to about 4.43 s. The unrolled blocks are intentional: dynamic private-array indexing was slower on this adapter.
3. **Retained:** drainage relaxation now ping-pongs 16-byte tuples in existing scratch instead of full 160-byte cells. At 1024² it fell from 14.03 s to 3.80 s. An odd-batch interruption/recovery test covers scratch-side selection.
4. **Retained:** following basin representatives shortened labeling from 8.88 s to 1.68 s at 1024², without changing the final connected-component labels.
5. **Retained:** initialized ecology skips the redundant aggregation before monthly weather; the required post-weather aggregation remains. At 1024² aggregation fell from 7.41 s to 3.71 s.
6. **Rejected:** a direct-storage river-collection rewrite regressed; the original kernel remains.

The largest remaining measured stages at 1024² are ecological transport (~4.60 s), drainage (~3.80 s), terrain aggregation (~3.71 s), and monthly river routing (~3.37 s). Further major gains likely require a broader storage-layout or drainage-algorithm change, with new numerical validation.

Run `cargo run -- --benchmark --epochs 1` to reproduce `output/benchmark.json`. `eco/*` entries are per-kernel GPU totals; `ecological_gpu` is their aggregate, not an additional cost. `ecology_month_wall` and `secondary_lakes_wall` are wall-clock diagnostics. Do not sum these overlapping categories.

## Numerical validation

A controlled 32² one-epoch comparison produced byte-identical terrain, environment and river buffers before and after optimization. Ecological inventories differed by at most 5.59e-7 in absolute per-area units; same-build checkpoint continuation remains exact on this backend. The then-current 19 tests covered CPU-reference drainage, conservation, scenarios, cube seams, lake levels and exports.

Long-history checks completed 100 ecological years at 256² and 1,000 years at 64² with finite state and budget residuals within tolerance. Maximum relative C/N/P residual was 2.29e-5 for the former and 1.48e-4 for the latter. These calibration runs preceded the final stored-lake evaporation fix, which has a dedicated dry-month regression test.
