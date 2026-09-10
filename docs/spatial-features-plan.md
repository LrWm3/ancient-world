# Shared spatial features and future map layers

Status: staged design, based on repository review on 2026-09-10. The first
[spatial adapters and expedition atlas route](spatial-features.md) are implemented.
The broader registry, temporal queries, area geometry and remaining subsystem
adapters below remain planned.

## Objective

Give spatially meaningful entities a consistent, queryable location, extent or
route, with enough provenance to distinguish simulation geometry from approximate
visualization. Globe, atlas, regional inspection, history navigation and export
should consume the same features. Keep resource, population and political state in
their existing authoritative systems; spatial records reference them, not duplicate
their inventories. Building this layer must not change simulation decisions.

Use a native spherical representation with a GeoJSON-shaped interchange view.
GeoJSON supplies useful Point, MultiPoint, LineString, MultiLineString, Polygon and
MultiPolygon conventions, but RFC 7946 specifies WGS84 coordinates. Our configurable
fictional sphere is not WGS84. Export a clearly identified fictional-planet
longitude/latitude visualization compatible with GeoJSON geometry syntax; do not
claim Earth geodesy or use an obsolete custom `crs` member to imply standard
compliance. Planet radius, coordinate convention and units accompany the export.
See [RFC 7946](https://datatracker.ietf.org/doc/html/rfc7946), particularly sections
3, 4 and 6. Native archives remain authoritative for spherical measurements.

## What already exists and what should conform

The following are inspected data structures, not proposed new simulations.

| Subsystem and current storage | Proposed attachment | What must change or remain explicit |
|---|---|---|
| Cube-sphere terrain: `grid.rs`, `gpu::Cell`, multiple terrain/ecology resolutions | Grid-qualified cell references; cell footprints and field views | A raw `u32` is not globally meaningful. Include world/grid identity and resolution. Reuse current face mapping. |
| Regional surveys: `region::Region`, center/width/resolution/date, `RegionalCell.route` parent IDs | Survey footprint, local grid frame, parent-cell mapping and snapshot stamp | Distinguish survey extent from a named geographic region. Patches remain dated derived snapshots, not canonical fine chunks. |
| Geography masks, land/water categories, geological plate fields | Cell-set regions; derived multipolygons and boundary paths | Existing categories are not necessarily connected landmasses. Derive connected components; do not give unstable labels permanent identities automatically. |
| Geology, strata and resource potential: `gpu::Cell`, `simulation.wgsl` | Surface classification regions, depth intervals and optional derived activity contours | Do not label every activity contour an explicit fault or ore body. There are no independently stored fault/volcano objects to attach yet. |
| Hydrology: downstream `routing`, basin labels, stored water | River segments/network, catchment cell sets, lake water footprints and outlets | A drainage catchment is not a wetted lake. Keep routing geometry separate from water coverage and refresh each at the appropriate boundary. |
| Climate/weather: cell fields and `hazards::FloodImpact` | Raster layers, sampled vectors; thresholded dated flood/drought regions | Sites with flood impacts have site-level evidence, not complete flood polygons. A physical footprint requires a threshold on the environmental field. |
| Ecology/wildlife: `EcoCell`, `Environment`, coarse layers, intervention region selectors | Field references, habitat/range masks, regional abundance layers and scenario scopes | Show ecology resolution. Abundance masks are not individual animal tracks or species ranges unless the state actually identifies species. |
| Managed agriculture/fisheries: settlement claims and management fields | Cultivation/harvest catchment associations and fractional cell coverage | Do not fabricate exact fields, pasture fences or fishing grounds from aggregate hectares. Mark fractional coverage as unlocalized within a cell. |
| `civilization::Site`, retained abandoned sites | Site point anchored to its terrain cell; known-location footprint; lifecycle interval | A cell-center marker is representative. It is not a surveyed town center or a town boundary. |
| `resources::Source`, `RegionalMine`, `metallurgy::ResidueDeposit` | Source-cell footprint, mine association, deposit point/area and depth where known | Preserve the same source ID at every zoom. A regional mine currently controls a parent source, not a generated underground excavation. |
| Site assets, facilities, private workshops, institutional meeting places | Site anchor plus role and capacity; exact footprint only when one exists | Stacked icons/grouped inspection are preferable to invented building coordinates. |
| `society::Route.cells`, road upkeep | Reusable land path and route feature; road condition on route/known segments | Reuse the existing ordered path. Distinguish physical road condition from permissions and transport cost. |
| `shipping::Port.access`, `water_cell`, `SeaLane.cells` | Port point, land access path and water route | Preserve path orientation and explicit land/water transitions. |
| `expeditions::FrontierRoute.cells`, voyage route/status/time; heritage find cell | Frontier route, journey traversal, camp/find anchors | A chosen route is not a witnessed per-month track. Use time interpolation only when marked estimated. |
| `culture::Patron.origin`, `landing`, `voyage`, service/departure | Ancient origin, landing and schematic voyage; return association | Preserve prologue/return dates and schematic status. Do not invent witnessed return tracks or new destinations. |
| `Cargo`, `Shipment`, relief appeals, export contracts | Shipment traversal, delivery endpoints and contract relationship | Land cargo often has endpoints/cost but no retained multi-edge itinerary. Capture the chosen route at dispatch going forward; do not reconstruct the historical trip with today's graph. |
| `relocation::Journey.route`, departure/arrival and household | Journey traversal; origin/destination cohorts and diaspora associations | Reuse migration records. Families still at home remain there; no extra moving population is created for the overlay. |
| `politics::controllers`, `Claim.cell/sites`, factions, governance and offices | Controlled-site sets, claimed-cell regions, contested overlaps, jurisdiction associations | Claims already cover occupied cells, nearby hinterland and road corridors. Preserve holes/unclaimed interiors. Controller of a town is not proof of ownership of an entire island. |
| Named wars and `society::Raid` | Conflict feature linking goal, parties, campaign traversals, battle/occupation anchors | A war has several geometries, not a single point or convex hull. Aggregate combat only supplies the recorded target location, not a detailed battlefield. |
| Traditions, household faith, religious dynamics and pilgrimage/relief records | Sacred-site anchors, prevalence by inhabited cell, contact links and actual recorded journeys | Faith is separate from sovereignty. Do not fill the space between believers as an exclusive religious territory or treat all contact as travel. |
| Institutions, office seats, enterprises, factions | Home anchor, member/support distribution, jurisdiction/member links | Multi-site footprints are derived only from actual records; branch offices cannot be inferred solely from influence. |
| Named people, genealogy, households | Residence/service anchors, birth/death/event places where recorded; known journeys | No daily schedules exist. Keep genealogical relationships nonspatial unless visualizing the endpoints of a relationship explicitly. |
| Artifacts, manuscripts, knowledge, accounts and naming provenance | Custody/findspot/origin roles; dated movement evidence; distribution at known holders | Ownership is not physical location. A source named after another place does not itself move there. Unknown or lost custody can have an uncertain last-known place. |
| `Event.site`, `other`, typed subjects and causal references | Multiple dated spatial roles: occurrence, origin, destination, affected area and reported place | Freeze geometry evidence at event time going forward. Do not interpret every pair of sites as an actual route. Preserve attributed accounts separately from physical events. |

Implementation source anchors: [grid](../src/grid.rs), [regional surveys](../src/region.rs),
[roads](../src/society.rs), [shipping](../src/shipping.rs), [expeditions](../src/expeditions.rs),
[politics](../src/politics.rs), [culture](../src/culture.rs), [resources](../src/resources.rs),
[regional mines](../src/regional_mining.rs), [residues](../src/metallurgy.rs),
[migration](../src/relocation.rs), [events/sites](../src/civilization.rs) and
[cargo](../src/economy.rs). Authoritative quantities and decisions stay in these systems.

## Shared primitives

Keep three layers separate: coordinates and geometry, semantic spatial features,
and transport topology. A route is not merely a LineString, and a region is not
necessarily a polygon already stored as vertices.

| Primitive | Definition |
|---|---|
| `WorldRef`, `GridRef`, `CellRef` | Persistent world identity, grid kind/version/resolution, and cell address. Regional grids also reference their frame and parent world. Seed alone is not a world ID. |
| `SurfacePosition` | Normalized planet-centered direction using f64 on CPU; optional elevation with an explicit datum. Cell anchors remain cell references until resolved. |
| `Anchor` | Explicit position, cell representative, local-frame coordinate, or entity attachment with role and temporal binding. Unknown is an explicit result, never `(0,0)`. |
| `Point` / `MultiPoint` | One or several known positions; a multi-point does not imply a filled intervening area. |
| `Path` / `MultiPath` | Ordered vertices or ordered cell edges. Each path has edge interpolation semantics and open/closed state. Distinct branches are separate paths. |
| `SurfaceArea` / `MultiArea` | Closed outer rings with holes; disconnected pieces remain separate components. Interior side/orientation is explicit for regions larger than a hemisphere. |
| `CellRegion` | Exact membership in a specified grid, using sorted runs or a bitmap; optional fractional coverage stored separately. Polygon boundaries are derived views. |
| `FieldView` | Reference to a grid field, source stamp, units, optional threshold and categorical value. Dense climate/ecology data stays a field rather than millions of vector features. |
| `FeatureGroup` | References to multiple features/roles: e.g. a war's objectives, routes and occupation areas. Do not copy child geometry or nest arbitrary JSON. |
| `RouteRef` / `Traversal` | Route identity + immutable revision + direction and departure/arrival; traversal can reference an ordered itinerary of route legs. |

A semantic `Region` references a `CellRegion` or `MultiArea`, its meaning, provenance
and validity. Use `SurveyFrame` for the existing tangent-plane patch so “region”
is not overloaded between landmass, scenario selector, catchment and survey window.

Use the viewer's current axes: latitude `asin(y)`, longitude `atan2(x,z)`.
Longitude/latitude interchange uses degrees in that order; internal distances use
planet radius in metres, areas use square metres. Sea-level elevation and positive
subsurface depth must have distinct fields. Abstract underground ecological layers
have a named stratum, not an invented depth or cave mesh.

Spherical paths do not always equal the shortest endpoint arc: cell paths preserve
intermediate vertices and local-frame paths retain their projection rules. Densify
arcs for the atlas/export at a declared error tolerance. Split antimeridian crossings
into multipart output; test poles, holes and large enclosing areas. On the globe,
clip the far hemisphere. For polygonized cell sets, derive shared edges once so
adjacent features agree across cube-face seams. Do not infer polygons from cell
centers or simplify borders independently into gaps and overlaps.

## Feature identity, roles and provenance

Proposed record shape (illustrative, not an archive schema commitment):

```text
SpatialFeature {
  id: FeatureId,
  owner: EntityRef,              // typed kind + stable ID, not an array pointer
  role: SpatialRole,             // home, extent, claim, custody, route, findspot ...
  geometry: GeometryRef,
  revision: GeometryRevision,
  validity: TimeInterval,
  source_stamp: WorldStamp,
  precision: ExactModel | CellRepresentative | PartialCoverage | Schematic | Unknown,
  derivation: Stored | Derived { algorithm_version, parameters, inputs },
  evidence: [EventRef],
  assertion: PhysicalRecord | Attributed { account, claimant },
}
```

“ExactModel” means faithful to stored model geometry, not real-world measurement.
A known cell plus an imprecise point should expose both the marker and that cell's
extent. Any display-only offset used to separate icons never changes the anchor.

One entity can have many roles: an artifact's findspot, last known custody and
claimed origin; a patron's origin, landing and return destination; an institution's
seat and supporter locations. Physical and attributed locations are separate
features, not contradictory overwrites. A feature can also have multiple subjects
without acquiring multiple owners of the same geometry or stock.

Feature IDs survive name changes. Base identity uses owner + role + stable part ID;
geometry revisions are separate. Existing sparse IDs are wrapped initially. Records
without stable IDs (some cargo, lane and transient collections) receive archived
monotonic counters before historical references rely on them. Do not derive durable
identity from a vector index that can shift after removal, or from a mutable name.
Use string IDs in JSON exports to preserve full integer identities.

## Time and history

`WorldStamp` includes geological epoch, ecology month and optional history month.
Use typed clock domains and the existing living-history clock relationship; do not
pretend one geological epoch equals a fixed number of social months. A time interval
is half-open in its stated domain. Missing start/end evidence remains unknown.

A live view may follow an entity's current anchor. A past event or dispatched journey
must bind to a geometry revision or an explicit time-of-event anchor. Roads can
reroute, towns can change controller, coastlines can move and artifacts can change
custody without moving prior events on the map. Changes append revisions at completed
simulation boundaries. Retain the revisions referenced by events and journeys.

Record future ownership/claim transitions and spatial changes as deltas, with periodic
snapshots for queries. For old saves, create an explicitly dated spatial baseline.
Do not fabricate historical borders or paths from today's state. Current field
snapshots cannot answer arbitrary past ecological maps: mark historical coverage
available/unknown, and make optional field snapshot retention a separate storage
policy. Inactive routes and abandoned sites remain queryable when history references
them. Region split/merge lineage is a separate association, not reuse of a stale
component number for a new lake or habitat patch.

## Routes, movement and causal continuity

Keep route graph edges, cost, capacity, access rules and condition in the existing
transport systems. Geometry describes those edges; it does not recompute economic
cost as straight-line distance. A multi-hop shipment records the actual ordered
edge revisions chosen at dispatch. This requires route search to optionally return
an itinerary alongside the cost currently used by callers.

Journeys bind to that itinerary and record delays, reroutes and arrival. Repeated
shipments share route geometry but have distinct traversals and finite cargo owners.
For older endpoint-only records, display endpoints and, if requested, a dashed
schematic connection. Do not produce a supposedly historical land route by running
Dijkstra on the current graph. Month-to-month movement between known endpoints is
an estimated marker unless the simulation records intermediate progress.

The first map overlays are observational. Later regional actions should resolve a
feature back to its canonical entity ID before using existing command APIs: closing
a displayed mine affects the same source and workshop supply; closing a route affects
that route's real shipments. Drawing/editing a polygon must not itself transfer land,
ore, nutrients or political control. Keep command authorization and model invariants
in the owning subsystem.

## API and rendering preparation

Expose one adapter-backed query service to desktop, headless tools and future layers:

```text
features(kind/role, view_bounds, time, grid/LOD, evidence_filter, page_limit)
geometry(feature_id, revision, error_tolerance)
features_for_entity(entity_id, time)
features_for_event(event_id)
resolve_anchor(anchor, time) -> located | approximate | unknown
export_features(query, fictional_geojson | native_spatial_archive)
```

Queries return typed units, source resolution, temporal coverage and provenance.
Bounds must handle wrapped longitude or use face-tile/spherical bounds internally.
Picking uses stable feature IDs and exact model geometry where available. Render
meshes, label layout and line thickness are caches, not saved authoritative geometry.
Separate entity name from styling. Unknown location omits geometry but retains the
searchable feature/evidence record.

Begin with a cube-face tile index over sparse features and coarse bounding caps.
Add fine geometry only for visible/query-selected layers. Cache derived masks,
polygons and line meshes by source stamp, algorithm version and LOD. A changed road
invalidates that road, not the whole world. Avoid scanning full historical events or
copying all GPU fields every frame. Use existing snapshots or bounded staged GPU
reductions for dense masks; export is an explicit operation. Benchmark before moving
polygonization or indexing onto GPU. No simulation CPU fallback is introduced.

Example later layers: ports/roads/sea lanes; active cargo and migration; war campaigns;
controlled sites versus contested claims; religious prevalence/sacred places; mines
and processing waste; artifact provenance; patron voyages; river catchments/lakes;
habitat and disturbance extents. The same filter must work in globe, atlas and region
views. Aggregate prevalence remains a graduated field, not categorical political paint.

## Implementation order and completion gates

1. **Foundation and adapters.** Add typed world/grid/entity references, primitives,
   feature/role metadata, coordinate conversions and query interfaces. Adapt site
   points, source cells, survey extents and existing road/lane/frontier paths without
   changing simulation state. Complete when every adapter resolves to the original
   source, coordinate round trips pass and source resolutions are visible.
2. **First end-to-end layers and export.** Use the common query for sites, ports,
   roads and lanes in globe and atlas; implement picking and fictional GeoJSON export.
   Test antimeridian/pole rendering and native metadata. This is a small demonstrator,
   not implementation of every possible layer.
3. **Movement evidence.** Introduce stable missing IDs, route revisions and retained
   itineraries for cargo, relief, migration, raids and expeditions. Add patron voyage
   adapters with their existing schematic precision. Complete when rerouting or closing
   today's route leaves a previously dispatched/history-bound path unchanged unless
   an explicit reroute event occurred.
4. **Social extents and historical places.** Expose exact existing claim-cell sets,
   controlled-site associations, offices, faith/support distributions, institutions,
   war feature groups and custody/findspot roles. Add event-time anchor recording and
   spatial revision history. Complete when a conquest changes control without changing
   religious affiliation or relocating historical events.
5. **Environmental and cross-scale extents.** Add connected water/land regions,
   catchments/rivers, geological classification masks, flood/habitat/abundance views,
   source/deposit extents and fractional land-use representation. Build seam-consistent
   polygonization. Complete when connected components, holes and spherical areas match
   source membership, and a regional source feature resolves to the planetary source.
6. **Persistence, coverage and performance.** Finalize versioned migration/export,
   indexes, selective history retention, change invalidation and LOD. Run integrated
   archives and seed/resolution suites; report unresolved spatial coverage rather than
   silently inventing it. Only then broaden the full layer catalog.

Each increment includes archive compatibility and targeted tests as it is introduced;
step six consolidates them rather than postponing persistence until the end.

## Validation and limits

- Typed references reject wrong world/grid, missing entity and stale route revision.
  Unknown locations, multipart emptiness and unavailable past intervals are handled
  explicitly. Geometry ownership cycles and invalid rings are rejected.
- Grid/coordinate round trips agree with CPU and GPU mapping, including face edges,
  corners, poles and longitude wrapping. Equivalent native and displayed geometry
  must pick the same features within declared tolerance.
- Cell-set area matches the spherical sum before simplification. Holes, islands,
  contested overlap and large enclosing land areas survive export and reimport into
  the native comparison fixture. Simplification has a documented geometric error
  bound and never edits the authoritative membership.
- Route adapters preserve cell order, endpoints, passability evidence and shared
  junctions. Planned, traversed and schematic paths stay distinguishable. Historical
  closure/rerouting tests must not silently substitute a new path.
- Geometries reference existing inventories: mine/source/residue, custody versus
  ownership, and migrating populations remain single-accounted. Generating layers
  and exports leaves simulation checksums unchanged; changing LOD changes no model
  quantities or random streams.
- Save/resume preserves feature IDs, revisions and temporal bindings. Old worlds gain
  an explicit current baseline and retain uncertainty for unrecorded earlier geometry.
- Exercise 32/64 diagnostic grids and 256/512/1024 terrain with independent ecology
  resolution. Measure overlay memory, query time, polygonization cost and archive
  growth across seeds 17, 81 and 256. Set performance budgets after these measurements,
  not before. Generated artifacts stay in ignored output; commit Markdown summaries.

No exact town footprints, underground meshes, individual wildlife tracks, political
borders across unclaimed wilderness or complete historical reconstruction are created
merely by introducing geometry types. Those require additional simulated state.
