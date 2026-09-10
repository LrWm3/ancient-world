# Regional terrain and basin hydrology

## Generating and inspecting regions

Select a planet cell, expand **Save, resume & export**, then choose **Generate explorable region**. Evolution pauses while a 512² regional snapshot is generated. Click the regional map to inspect elevation, soil, water, climate, drainage destination, flow and plant identity. The original globe/atlas zoom remains a fast visual preview; the separate regional window contains independently computed fields.

The headless API is `Generator::generate_region(center, width_km, resolution)`. Centers are spherical directions and normalized after validation. Width is the tangent-plane extent in kilometres; resolution is a power of two from 16 through 1024. Extents range from 1 km to half the planet radius. Allocation checks include the existing planet, two regional buffers and the adapter's individual storage-buffer limit.

`cargo run --example region -- output/planet.world output/region.png [CELL_ID] [WIDTH_KM]` generates 1024² cells, defaulting to a 600 km patch. At that extent nominal spacing is approximately 586 metres. The desktop's 512² patch has approximately 1.17 km spacing. Smaller extents allow finer cells, but inherited geological/ecological information remains limited by the planet grids.

A PNG export also writes `.region.json`. This versioned, self-contained derived archive includes seed, geological epoch, ecological month, center, width, resolution, catalog definitions, convergence counts and every regional cell. It does not modify or add inventories to the parent world archive.

## GPU computation

1. Sample the cube-sphere planet across face seams. Interpolate macro elevation, tectonic stress, temperature and rain. World-space seeded noise generates physical subcell bed elevations with kilometre-scale ridges and finer roughness.
2. Solve an eight-neighbor local minimax drainage surface without flattening terrain. Steepest descent selects routes on slopes; stable ranks resolve flats. Edge cells and inherited open water are outlets. Each route decreases spill height or its integer routing rank, so flats cannot cycle.
3. Record inherited lake/ocean water as an explicit starting inventory, with shorelines determined by fine bed elevation against interpolated water levels. Accumulate one representative year's local runoff. Each depression retains water up to its spill capacity; excess flows to its downstream cell. Converge the accumulation rather than silently cutting off long rivers.
4. Relax retained pool water through neighboring cells that share a spill elevation. Water cannot jump a dry ridge. Preserve volume using regional spherical areas.
5. Derive soil depth from inherited soil, slope and deposition potential. Inherit substrate, identify alluvial soils, and select surface plants from catalog temperature, precipitation, fertility, substrate, layer and regional restrictions. These are habitat assignments, not independent carbon stocks or a duplicate ecological evolution.

All terrain, drainage, flow, pool and habitat algorithms run on GPU. CPU work schedules dispatches, checks convergence and performs inspection/serialization/image encoding. Each iterative stage has an explicit limit and fails with its stage name if unresolved. No CPU simulation fallback is used.

`forcing[0]` records the initial water volume in each cell. The regional accounting identity is initial water plus local runoff equals final water plus outlet outflow. Inherited water bodies are fixed-level outlets for this representative experiment.

Cell fields use metres for elevations/depths, square metres for area, °C and mm/year for climate, and cubic metres per representative year for runoff/outflow. Inspection converts annual outflow to m³/s. `route[0] == u32::MAX` means a patch outlet. Parent IDs reference the source planet; local IDs index the regional grid.

## Boundaries and limitations

Regions are bounded catchment snapshots. They account for their own rainfall runoff and report boundary outflow; they do not yet inject upstream river discharge from outside the patch. Terrain noise is anchored in world coordinates, but independent patch drainage is not yet a stitched global fine river network. Broader regional climate and geological categories come from the planet. No excavation geometry, individual trees, settlement structures or local animal agents are created.

The regional annual water calculation is a representative runoff/storage experiment, not a multi-year regional weather simulation. Small puddles remain inspectable; the natural map emphasizes water deeper than 0.5 m and rivers exceeding the display threshold. PNG colors do not change the physical water inventory. Generated snapshots retain their source date if planet evolution resumes; regenerate to inspect current conditions.

Canonical chunks, cross-patch river boundary contracts and persistent terrain edits are prerequisites in the civilization roadmap. A generated patch must not yet be treated as an independently evolving source of food, ore or water for a settlement.

## Planet basin solver

The former basin-wide level solve could redistribute water across an internal ridge even when neither pool had reached it. The replacement reads frozen water states and computes bounded volume transfers over actual neighboring cube-sphere cells. Transfers use both cells' spherical areas; four outgoing edges cannot spend more than 96% of a source inventory in a pass. The reverse edge applies the same transfer to its receiving cell.

Water within a spill basin responds to the bed and water surface at each edge. Separate pools remain separate below the saddle. On a routed outlet, transfer is also capped by water above the source's spill elevation. Water received downstream remains in physical storage and continues through the existing water systems. Terrain is untouched; no reset or new water source is introduced.

Relaxation is numerical equilibrium iteration, not physical elapsed time or a shallow-water velocity solver. It stops when per-pass depth changes are below `max(0.0005 m, 5e-7 * max(abs(bed), abs(depth)))`; an edge-head deadband of comparable scale avoids floating-point chatter. This is a local update tolerance, not a promise of one globally uniform surface to 0.5 mm. The solver permits at most 4096 passes, reports iteration counts and wall time, and raises an error if unresolved. The greater geographic fidelity costs more passes than the old shared-volume approximation. The removed basin-wide reduction buffer saves 32 bytes per planet cell (192 MiB at 1024² per face).

Tests cover separated and overtopping pools, outlet spill retention, volume conservation, cube-seam exchange, drainage against a CPU priority-flood reference, regional determinism, runoff accounting and plant habitat restrictions. The long-history suite exercises coupled evolution rather than only isolated pool fixtures. A 256² one-epoch production check completed with C/N/P relative budget residuals below 1.8e-6 on the Quadro RTX 5000. Previous optimization benchmarks predate this solver and are retained as historical measurements, not current timing promises.
