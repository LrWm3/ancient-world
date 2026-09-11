# Living history

Enable **Living world** in the civilization panel to advance the GPU environment alongside society. **Evolve / Pause** and **Step** then operate on complete social months; the history panel also offers month/year increments. The globe and atlas display the changing canonical world buffers. Existing worlds retain snapshot behavior until explicitly enabled; the switch records its historical date and preserves both clocks.

Each coupled month runs seasonal weather, snow/melt, groundwater and runoff, lake nutrient circulation, biological production, decomposition and animal guild movement before farming and social history. Existing regional drought settings now force actual planetary rainfall; farms use those same local conditions. Cold, heat and standing water constrain crop growth. Expedition camps read current temperature and vegetation. Seasonal forcing follows the social calendar even when the previous ecological clock has a different offset.

Ecology runs on GPU at its configured resolution. History reads its updated state and commits plot withdrawals before the next environmental month. Managed land is reserved from wild terrestrial production and rainfall. Farm claims debit physical water reservoirs as well as the ecological ledger, avoiding water recreation on the following month. Nutrient and water ledgers retain explicit transfers; field 24's fourth component stores the reserved fraction of each ecological cell.

## Controls and continuation

```sh
mise exec rust@1.89.0 -- cargo run --release -- --headless \
  --load output/discoveries-final.7.world --epochs 0 \
  --living-world --history-years 5 --save output/living-continued.world

mise exec rust@1.89.0 -- cargo run --release -- \
  --load output/living-continued.world
```

`Generator::enable_living_history()` starts the baseline; subsequent `advance_history(months)` couples both simulations. Old archives without a living baseline remain compatible. Checkpoints persist the offset and reject incomplete coupled months or clock mismatches. A failed coupled step requires reloading a checkpoint; it cannot silently save a partially advanced world. Execution batching does not change results on the same backend.

## Current limits

This advances the seasonal environment, not geological epochs. Elevation, tectonics, regional terrain, river routes and basin geometry stay fixed. Living storms now cause finite river overflow and local ponding, with town damage, transport closures and recovery; see [flood history](flood-history.md). It does not rebuild shared lake surfaces or model channel migration or terrain erosion during social time. The optional [environmental return connection](environmental-returns.md) routes managed runoff downstream and returns abandoned plots to ecology. Legacy histories keep their existing behavior until explicitly enabled. Specimen collection retains its finite accessible-source inventory, with no new regrowth mechanism.

The history engine is reused while its catalog and terrain buffer remain compatible. Before production, it refreshes its ecology buffer from the canonical GPU state. Ordinary living months gather the terrain cells needed by history into a compact GPU readback, updating a private CPU geography cache. An empty or invalidated cache uses a full refresh. Default GPU navigation reads current terrain for annual surveys and pending road construction; CPU navigation reference mode still refreshes the full view for those searches. Flood checks and the history transaction share that month's observations. See [readback strategy and matched-run checks](history-environment-readback.md). Other production and validation readbacks remain. Large year increments can still block the UI. No scientific timescale equivalence between geological epochs and social years is implied.

## Verification

GPU fixtures cover changing environment with fixed terrain, coarse ecology and plot-water conservation, shared drought forcing, exact checkpoint continuation, execution batch equivalence and rejection of incomplete checkpoints. Multi-seed evaluations validate ecological and social budgets annually; JSON reports include both clocks and ecological region summaries.

See [measured results](living-history-results.md) for the century seed suite and desktop checks.
