# Persistent finite rock columns

New planets now carry thicknesses for their existing three rock IDs. The GPU stores top, middle and basement thickness in meters, plus cumulative bedrock removed. Initial columns are an explicitly procedural geological baseline: surface units are 100–500 m thick, intermediate units 500–2,500 m, and the basement occupies the remaining initialized crust thickness. They are not reconstructed depositional histories.

## Evolution

- Weathering removes finite bedrock into the existing loose sediment inventory. Fluvial incision can consume bedrock after available sediment. Exhausting a unit exposes the next rock and shifts the remaining column upward; that exposed ID feeds existing soil, mineral and ecology calculations.
- Loose sediment above 5 m can lithify during geological epochs. Conversion is bounded by the available sediment and geological timestep and transfers thickness from sediment to rock. No extra surface height or extraction reserve is added by lithification.
- A lithified unit with a different rock ID becomes the new upper unit. To keep a three-unit limit, the two deepest units are combined under the former middle unit's ID. Their thickness is retained, but their separate lithology is lost. This is a documented approximation, not an unlimited stratigraphic archive.
- Existing tectonic crust thickening/thinning adjusts basement thickness. Existing volcanic/metamorphic resurfacing changes the upper rock identity in place; this does not model emplacement geometry or a dated eruption.

Thickness accounting uses equivalent volumes, without density changes or compaction shrinkage. Loose sediment and bedrock are the physical column terms; the existing soil-depth field remains a soil-profile proxy. Cumulative removal measures weathering and bedrock incision, not tectonic thinning. There is no full lithospheric mass ledger, bedding orientation, fault displacement, or mineral vein geometry.

Column evolution follows the existing geological water/erosion pass. Ecology-only and living-history monthly updates preserve the columns but do not run this new lithification/bedrock-exposure model. Individual earthquakes and eruptions are still future work.

## Inspection and regional use

Planet inspection shows each unit's thickness and cumulative bedrock removal. `Cell::rock_at_depth(depth_m)` returns the rock below the top of bedrock; callers account for loose sediment separately. Contacts belong to the underlying unit. Invalid depths, depths below the finite base, and unavailable legacy columns return `None`.

Regional snapshots inherit the parent column's thicknesses and three rock IDs. They are labeled as parent units: procedural regional relief does not yet cut a new local stratigraphic section. Exporting overlapping regions does not duplicate mining reserves. This gives later excavation work a persistent source column without claiming excavation-ready caves or chunks.

## Persistence and memory

World archive version 6 increases terrain cells from 160 to 176 bytes. The additional vec4 is appended after existing fields. Readback, GPU shader layouts, memory estimates and the offline audit reader use the new stride. At terrain edge 512, the two terrain buffers require an additional 48 MiB; ecology buffers are unchanged.

Versions 1–5 retain their original terrain payload sizes. Loading verifies the original checksum and expands the new components to zero. These worlds have **unavailable columns**, rather than fabricated geological histories, and continue using their previous erosion behavior. Version-one import remains explicit. Saving an imported world writes the expanded version-six representation.

Regional exports are now version 3, with appended `strata` and `rocks` fields. Older regional JSON defaults these fields to zero. Region buffers grow from 96 to 128 bytes per cell and retain the existing allocation checks.

## Verification

Controlled GPU fixtures exhaust thin upper layers and verify basement exposure, nonnegative thickness, cumulative removal and rock-plus-sediment volume conservation. Another fixture lithifies accumulated sediment and checks the same volume budget. Depth queries check exact contacts, finite bases and missing columns.

Legacy fixtures construct actual 160-byte terrain payloads for version-one import and version-three continuation. Current archives also exercise checkpoint continuation, region export/readback and rendering through the existing GPU suite. The province seed suite (17, 81, 256), long geography run, ecological accounting and managed mining checks run with the new layout.

Validation completed: 49 distinct targeted tests passed across the core, ecology, economy, geology and GPU suites, including the added depth-query and finite-column fixtures. The long geography test passed with the existing bounded lake convergence allowance. Clippy with warnings denied, Rust formatting, diff whitespace checks and Python audit-reader syntax checks passed.
