# Compact environment observations for living history

Living history defaults to GPU gathering of the cells its monthly CPU consumers
need. The gather copies complete 176-byte cells bit for bit into one compact
buffer; it does not approximate, average, quantize, or recompute environmental
values. A private CPU terrain array retains the rest of the geography between
full refreshes. Public `snapshot()` and map rendering continue to use canonical
GPU state, never this partially refreshed history view.

## Coverage and refresh boundaries

| Consumer | Cells refreshed before monthly history |
|---|---|
| Flood damage, resource registration, farm/fishery access, claims | Every settlement and its four cube-grid neighbors |
| Expansion eligibility and prospective settlement access | Every current candidate and its four neighbors |
| Road closures | All existing road cells |
| Harbor and shipping closures | Port access paths, water cells, and sea lanes |
| Expedition conditions and source validation | Frontier routes, including destinations, and discovery sources |
| Resource source inspection | Registered source cells |

The sample list is rebuilt, sorted, and deduplicated every month. Adding towns,
routes or candidates therefore changes the next sample list. The observation
view is stamped with the target history month and rejects a different month.
The cache is only installed after a successful complete living month; a failure
drops the pending view and retains existing incomplete-boundary behavior.

Full refreshes remain on:

- The first month after construction/load or explicit strategy changes.
- Every annual update, before settlement expansion, shipping surveys and
  expedition route discovery can search arbitrary terrain.
- Pending road construction when society has not routed every existing site.
- A changed terrain epoch/buffer or a dense sample list covering every cell.

Terrain restoration and geological advancement invalidate the cache. History's
existing fixed-geography restriction remains in force. The cache and gather
buffers are transient, not archive contents. A loaded checkpoint builds a fresh
view without changing its simulated state.

Unbounded route searches depend on current water, not merely static elevation.
The annual full refresh deliberately preserves that behavior. Reducing those
transfers further needs a separate navigation observation layer or GPU routing;
using stale water to avoid the transfer would change the model.

## Verification and measurements

The differential fixture starts both strategies from the same checkpoint, with
16 founding settlements, society, politics, governance, shipping, expeditions,
discoveries, shared resources and environmental returns. It imposes a 50% drought
probability, 0.9 drought severity, 50% storm probability and 4x storm rainfall.
These are stress-test controls, not new world defaults.

For 36 months it compares the complete serialized history, exact terrain bytes,
exact ecological cell bytes, and ecological clock after every month. It then
saves/reloads the gathered branch and compares a four-month batch against four
single-month calls and the full-readback reference. Reported differences are
zero in these comparisons. Timing metadata and the transient history engine are
not included in equality assertions.

Hardware: Quadro RTX 5000 Max-Q, NVIDIA 595.84, Vulkan; Rust 1.89, optimized dev
profile. Each 36-month comparison uses four full cache refreshes and 32 gathered
refreshes instead of 36 full snapshots. Timings below exclude setup, explicit
comparison snapshots, saves, and the final four-month continuation.

| Terrain / ecology face edge | Seed | Gathered terrain MiB | Full terrain MiB | Gathered history seconds | Full history seconds |
|---|---:|---:|---:|---:|---:|
| 64 / 16 | 17 | 18.52 | 148.50 | 1.525 | 1.461 |
| 64 / 16 | 81 | 18.77 | 148.50 | 1.462 | 1.463 |
| 64 / 16 | 256 | 18.92 | 148.50 | 1.417 | 1.429 |
| 256 / 16 | 17 | 271.88 | 2376.00 | 2.103 | 4.663 |
| 512 / 16 | 17 | 1072.48 | 9504.00 | 2.779 | 11.400 |
| 512 / 256 (default grids) | 17 | 1074.18 | 9504.00 | 6.107 | 15.542 |

The 64-resolution fixtures include 84–129 road cells, 166–304 sea-lane cells,
and five expedition routes; the 512-resolution fixture includes 513 road cells
and 2,092 sea-lane cells. This exercises more than isolated settlement sampling.

Transfers fell by roughly 87–89%, including full survey refreshes. Small-world
runtime was essentially unchanged; one gathered run was slower. These are single
paired observations, with gathered history executed before reference history,
not a statistical benchmark or a cross-hardware performance claim. Ecology 16
isolates transfer overhead and is smaller than the default ecological grid.
The default-grid run also matched exactly: 88.7% fewer terrain bytes and 60.7%
less measured history time in this fixture. It had 637 road cells and 1,956
sea-lane cells. This is not a benchmark of centuries of accumulated history.

Additional fixtures verify bitwise gather layout at cube seams, changed values
after terrain restoration, cache invalidation, stale-month rejection, shared
drought forcing, conservation, checkpoint continuation and batch equivalence.
The ordinary library suite passed 61 tests (64 hardware fixtures skipped); the
gather, full-reference readback-count, two living-history fixtures, and matched
comparison fixture were run explicitly on GPU.

## Reproduction and reference mode

```sh
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test history_environment -- --ignored --nocapture
HISTORY_GATHER_RESOLUTION=256 HISTORY_GATHER_SEEDS=17 CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test history_environment -- --ignored --nocapture
HISTORY_GATHER_RESOLUTION=512 HISTORY_GATHER_ECOLOGY_RESOLUTION=256 HISTORY_GATHER_SEEDS=17 CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test history_environment -- --ignored --nocapture
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib history_environment::tests -- --include-ignored
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test living -- --ignored
```

`Generator::set_history_readback_mode(HistoryReadbackMode::Full)` selects the
reference path. `Gathered` restores the default; switching clears the cache and
counters. `history_readback_stats()` reports full/gathered refresh counts, cells,
terrain bytes, and wall time spent refreshing observations. It excludes economic,
ecological, UI and archive readbacks. The strategy is an execution setting and
is intentionally not part of world history or serialized configuration.

## Remaining costs and maintenance

This does not remove the monthly GPU synchronization dependency, economic or
budget readbacks, or annual full terrain refreshes. It retains one full terrain
array in CPU memory (264 MiB at terrain 512) and GPU observation/index buffers
sized for the sampled cells. Buffer and dispatch limits are checked before use.

The coverage contract is in `observed_cells` in `src/history_environment.rs`.
New history consumers that read dynamic terrain outside that set must extend
it or request a full refresh. The cached array must not be treated as a fully
current planet. Future field packing and route reductions can reduce payloads
further, but should retain the exact-reference comparisons.
