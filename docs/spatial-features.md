# Spatial features

`Generator::spatial_features()` returns read-only native features for settlements,
roads, harbors, sea lanes, resource sources and expeditions. Each cell address includes a world
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
routes on the atlas only. The selected voyage also shows a green camp or red
stranding marker and a blue heritage findspot when those records exist. Markers
are labeled but not individually clickable.

Old voyages without a frozen path use their current route association, explicitly
marked `LegacyRouteAssociation`. They are not upgraded into witnessed tracks.
No interpolation invents a precise current position for a traveling expedition.

## Resources and regional surveys

Resource points reference the canonical source by world identity and terrain cell.
A depleted source keeps the same feature identity. Features do not copy its ledger;
inspect the source record for current quantities. Mineral names come from the
source's stored catalog identity, rather than a new prospect roll.

New regional surveys retain their own snapshot identity, parent terrain grid and
planet radius. `Region::spatial_features()` returns deduplicated parent-cell
coverage, historical site locations and resource-source snapshots. The resource
and site feature IDs match the planetary view. A survey retains its own clocks;
its snapshot does not update when the live world changes. Its coverage is the set
of sampled parent cells, which can extend beyond the fine survey edges.

**Export survey spatial features** exports the currently generated survey to
`<checkpoint-name>.survey.geojson`. Old survey files still deserialize, but without
a stored world/grid identity this export asks for regeneration rather than guessing
one from the seed. New survey identities survive serialization, while two separate
surveys have distinct identities even when their generated cells are identical.

## Historical event anchors

New events save up to three terrain-cell anchors: associated settlement, associated
other settlement, and (when recorded explicitly) a movement/discovery milestone.
Settlement associations are context, not claims that something physically happened
there. They are copied at event creation; subsequent changes to sites do not move
those anchors. Their enclosing history supplies the terrain grid, and export adds
the saved world identity.

Explicit milestones currently cover expedition landfall, rescue/empty-camp arrival,
stranding, retreat, repaired return access, return to the home town, heritage
fragment observation, and household resettlement/return. Lost parties and blocked
journeys have no invented position along a route. These remain cell-scale markers,
not surveyed coordinates, continuous travel tracks or timestamps finer than a month.

`Generator::spatial_events(start..=end)` exports an inclusive monthly range. Each
feature links to its event and date. **Export recent event locations** writes
`<checkpoint-name>.events.geojson`; its month-count control defaults to 120.
Current-world export deliberately excludes the event log. The timeline event
inspector distinguishes recorded anchors from buttons locating the current town.
Focusing a past location changes the camera, not the displayed terrain epoch.

Legacy events without snapshots may expose current site associations, explicitly
marked `LegacySiteAssociation`. They never acquire inferred milestone locations.
A new event with an explicitly empty anchor list stays unmapped. Events without
any location yield no feature; they remain in the event log. Generic site-bound
events are covered automatically, but route geometry, war extents and other
subsystem-specific occurrence locations still need dedicated adapters.

## Export and limits

**Export spatial features** writes `<checkpoint-name>.spatial.geojson` beside the
chosen checkpoint path. `FeatureCollection::geojson()` provides the same export
for library clients. Coordinates are longitude/latitude degrees on this fictional
sphere, with radius and clocks in `ancient_world` metadata. This uses GeoJSON
syntax; it is not WGS84 Earth geography. Entity and event IDs export as strings.

Paths split at the antimeridian for display, using linear longitude/latitude
segments between cell centers. The exporter rejects foreign world/grid references,
invalid cells and degenerate paths. `CellRegion` exports its unique parent-cell
centers as a `MultiPoint`, explicitly tagged `cell_region_representatives`. This
represents a cell set, not a filled polygon or exact boundary. Empty or duplicate
cell sets are rejected. Polygon generation, local regional-grid addresses and
globe overlays are not implemented here.

Current-world features are snapshots with per-feature history dates. Event anchors
are separately frozen records, not a general temporal geometry registry. IDs refer to existing stable records; harbor and sea-lane IDs
currently use array positions and must be revised if those arrays become reorderable.
Movement milestones, event snapshots, area adapters, filtering/indexing and the
remaining subsystem coverage are listed in the [larger plan](spatial-features-plan.md).

## Initial verification (2026-09-10, commit 74d451f)

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

## Resource/survey increment verification (2026-09-10)

- Library suite: 57 passed, 53 hardware tests ignored. Cell-set export now verifies
  `MultiPoint` representation and rejects empty/duplicate cells.
- Hardware test `shared_sources_connect_mining_surveys_markets_and_checkpoints`
  passed on NVIDIA Vulkan in 40.06 seconds: 32² terrain, 16² ecology, five sites,
  controlled extraction, mine closures and checkpoint/batch comparison. A 10 km,
  16² survey identifies the same source feature as the planet; its export preserves
  clocks and identity through serialization without mutating history. Missing
  legacy survey identity is rejected explicitly.
- Library Clippy (`-D warnings`), formatting and repository artifact checks passed.
  New expedition markers and survey export controls compile; they have not received
  interactive visual testing. No new expedition outcome calibration was attempted.

## Event-anchor increment verification (2026-09-10)

- Library suite: 57 passed, 53 hardware tests ignored; library Clippy passed with
  warnings denied. Formatting and source-only artifact checks passed.
- NVIDIA Vulkan expedition fixture (seed 7, 64² terrain/ecology): passed in 6.01 s
  with cached shaders. Checks frozen site associations despite a changed current
  site cell, checkpoint/batch continuation, the recorded outer landfall cell,
  inclusive monthly filtering, invalid cell rejection, legacy association labels,
  explicitly unmapped events and invalid month ranges.
- Hardware relocation fixture: passed in 1.13 s. The recorded arrival cell matches
  the receiving settlement; existing population, food, money, identity and
  serialized-continuation assertions still pass.
- The new timeline buttons and range-export controls compile; interactive visual
  testing was not performed. Timings describe fixtures, not scalability claims.
