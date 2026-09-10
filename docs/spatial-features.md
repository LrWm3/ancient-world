# Spatial features: first increment

`Generator::spatial_features()` returns read-only native features for settlements,
roads, harbors, sea lanes and expeditions. Each cell address includes a world
identity and terrain resolution. Features reference existing entities and event
IDs; they do not own goods, people or resource stocks. Querying them performs no
GPU readback and does not change simulation decisions.

New worlds receive an identity stored in checkpoint configuration. Old archives
receive one on load; save that imported world before relying on its identity across
sessions. Independently created worlds have different identities even with the
same seed. Identity generation does not consume simulation randomness. API callers
creating a new world from an existing configuration should clear `spatial_world_id`;
the desktop's new-planet action does this automatically.

## Expeditions

New voyages store a copy of their planned cell path at launch. The adapter exposes
that path, the origin settlement, a destination-cell camp while camped or stranded,
and a recorded heritage findspot when present. These are cell-scale locations.
Planned paths are not observed tracks or evidence of each cell being visited.
The simulation still uses its existing route and travel-time rules.

In the expedition inspector, **Show planned route on atlas** selects the voyage,
opens the atlas and draws a gold route with a ring at the planned destination.
**Hide expedition route** is in the file/export controls. This increment draws
routes on the atlas only. Camp and findspot features are available through the
API/export; separate interactive marker layers remain future work.

Old voyages without a frozen path use their current route association, explicitly
marked `LegacyRouteAssociation`. They are not upgraded into witnessed tracks.
No interpolation invents a precise current position for a traveling expedition.

## Export and limits

**Export spatial features** writes `<checkpoint-name>.spatial.geojson` beside the
chosen checkpoint path. `FeatureCollection::geojson()` provides the same export
for library clients. Coordinates are longitude/latitude degrees on this fictional
sphere, with radius and clocks in `ancient_world` metadata. This uses GeoJSON
syntax; it is not WGS84 Earth geography. Entity and event IDs export as strings.

Paths split at the antimeridian for display, using linear longitude/latitude
segments between cell centers. The exporter rejects foreign world/grid references,
invalid cells and degenerate paths. `CellRegion` is a native reserved primitive;
its polygon export returns an explicit unsupported error rather than silently
omitting a region. Polygon generation, regional-grid addresses and globe overlays
are not implemented here.

Features are a current snapshot with per-feature history dates, not a temporal
geometry registry. IDs refer to existing stable records; harbor and sea-lane IDs
currently use array positions and must be revised if those arrays become reorderable.
Movement milestones, event snapshots, area adapters, filtering/indexing and the
remaining subsystem coverage are listed in the [larger plan](spatial-features-plan.md).

## Verification (2026-09-10)

- `mise exec rust@1.89.0 -- cargo test --lib --quiet`: 57 passed,
  53 hardware tests ignored. Includes cube-cell coordinate round trips, dateline
  splitting, large textual IDs, foreign-grid rejection and explicit unsupported
  region export.
- `mise exec rust@1.89.0 -- cargo test --test expeditions voyages_conserve_and_deliver_knowledge_after_exact_checkpoint_continuation -- --ignored`:
  passed on the available NVIDIA Vulkan GPU, 57.39 seconds. Seed 7, 64² terrain
  and ecology, five founding groups, 240-month preparation, hazards and automatic
  launches disabled. Tests launch/return accounting, nonmutating spatial export,
  frozen route independence and identical checkpoint continuation.
- `cargo check`, library Clippy with warnings denied, formatting and repository
  artifact checks passed. Desktop controls compile; interactive visual inspection
  of the new route overlay was not performed in this pass.

This is a small correctness fixture, not a route-performance benchmark or a
multi-seed evaluation of expedition outcomes.
