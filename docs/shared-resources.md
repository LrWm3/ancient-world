# Shared accessible resources and coupled counterfactuals

This increment connects finite extraction sources to GPU mining, production,
supplier quotes, expedition demand, planetary inspection and regional surveys.
It is an opt-in migration for existing and newly founded histories; ordinary
history without the migration retains the previous town-reserve behavior.

## Authority and transfers

`History.resources` owns one accessible ore/clay source per planetary cell.
Enabling the feature transfers existing unmined town reserves into those sources,
preserving their sum. Towns occupying the same cell share its inventory. Source
IDs are planetary cell IDs within the archived world/grid. Migrated sources are
marked as legacy baselines; prior extraction is not reconstructed.

At each monthly production boundary, active open claimants receive equal source
shares, each capped at five kg per resident (an upper bound on monthly extraction).
Grants round downward to fit the source. Town reserve fields are temporary GPU
allowances, not persistent additional stocks. The GPU still decides actual mining
from workforce, orders, storage and other production constraints. Readback commits
the allowance consumed to `remaining`/`extracted`, then clears every allowance.
Unused stock stays in canonical custody. A competing town cannot spend another
town's allowance, and closure or abandonment does not destroy unmined material.

Supplier replacement-cost quotes and expedition resource-demand decisions read
canonical availability rather than these cleared buffers. Thus depletion and
closure affect existing production/market behavior without a separate prosperity
modifier. Site inspection uses the same read-only availability query.

Later settlement claims cannot refill an existing source, including exhausted
ones. A first claim in a previously unregistered cell initializes a declared
accessible prospect from terrain potential: at most 1 km², 0.05 kg/m² ore times
mineral potential, and 0.2 kg/m² clay times the existing sediment proxy. These are
provisional game coefficients. The accessible area is capped by spherical parent
cell area. It is initialized once and persisted; it is not renewed on reoccupation.
Legacy initial sources preserve their actual existing quantity instead of being
rescaled to this new-prospect rule.

## Scope of geological continuity

This is **accessible economic ore and clay**, not a resolved mineral-specific ore
body. Planetary mineral metadata remains prospect information. The optional [mineral processing extension](mineral-processing.md) identifies remaining sources and connects three distinct iron ores to processing; the legacy path stays generic. Mining does not currently subtract rock-column thickness.

The planet inspector shows canonical remaining/initial quantities separately from
mineral potential. Regional archives include matching `resource_sources` and the
source `history_month`, with defaults for older archives. They are dated read-only
views: exporting overlapping regions creates no reserves and cannot spend them.
Unregistered cells have prospect information but no initialized extraction stock.
The region's existing terrain remains a derived snapshot, not an editable mine.

Source accounting satisfies `initial = remaining + extracted` separately for
ore/clay. The existing goods ledger accounts for extracted output and subsequent
crafting, wear, trade and ownership. GPU float32 transfers retain the existing
numerical tolerances; capped monthly grants reduce subtraction precision loss.
This is not detailed batch provenance through smelting and household ownership.

## Controls and persistence

In the history panel, choose **Enable shared extraction sources**. The selected
town then has a checkbox to close/reopen ore and clay extraction. Headless APIs:

```rust
generator.enable_shared_resources()?;
generator.set_mine_closed(site_id, true)?;
generator.advance_history(12)?;
generator.set_mine_closed(site_id, false)?;
```

Both APIs require complete boundaries and reject incomplete living-history clocks.
Closure records an event and modifies access only; repeated identical requests
are no-ops. It closes both ore and clay extraction, not only one commodity.
All source stocks, claimant registrations and closures serialize with history.
Legacy archives default to no shared-resource registry. An outstanding allowance
or inconsistent source ledger makes history validation fail.

## Counterfactual experiment

`resource_counterfactual` creates seeds 17, 81 and 256 at terrain 64/ecology 32,
one geological epoch, 16 starting civilizations and crop yield 0.33. It enables
shared sources and living history, advances one year, and saves one checkpoint.
Baseline and treatment both load/continue the same state. Treatment closes the
starting towns' extraction for five years, then reopens it for five years. New
settlements are not automatically included in the intervention.

```sh
mise exec rust@1.89.0 -- cargo run --release --example resource_counterfactual
```

The JSON report records annual ore/metal/tool production, tool stock, ore prices,
farm/extraction staffing, latest food output, shortages, population, trade,
source inventories and conservation residuals. It writes `complete: false`
between seeds and `true` only after all requested runs complete. Checkpoints
remain available for inspection. Annual production counters are cumulative;
subtract the fork sample to obtain post-intervention output.

This is a specific checkpoint-branch experiment, not yet a general intervention
scheduler. Exogenous forcing shares the same seed and calendar; endogenous events
may diverge. The treatment does not require a prescribed population response.

## Verification

- Controlled competing claims cannot overspend; unused allowances return, closure
  releases access to the other claimant, and a new claimant cannot refill a source.
- GPU fixture verifies closure preserves material and stops ore production,
  reopening resumes extraction, regional inspection agrees with source state,
  archived legacy state remains compatible, and batched/checkpoint continuation
  matches uninterrupted monthly execution.
- Economy and market regressions retain the legacy path and validate downstream
  ledger behavior. Multi-seed living experiments exercise economic responses.

Remaining work: broader mineral-specific processing, cross-cell access rights and mining routes, batch provenance and physical archaeological waste. [Managed runoff and abandoned-land returns](environmental-returns.md) now have an optional coupled implementation.

[Measured three-seed counterfactual results](resource-counterfactual-results.md)
show changes in prices, production, trade and population, including recovery.

[Alloy processing and physical residue](alloy-processing.md) extends identified sources to copper/tin industries, distinct recycled metals and retained on-site waste.
