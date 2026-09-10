# Geological provinces and regional prospecting

This increment takes two practical ideas from the geology review: coherent resource provinces and regional prospecting that carries planetary geology into local exploration. It does not add a mantle solver, excavatable strata, or physical ore veins.

## Planetary generation

The bundled catalog enables `geological_provinces`. Surface and inherited rock IDs are selected from continuous spherical noise fields, constrained by their existing formation classes. The same fields are used during volcanic/metamorphic resurfacing. Crust age also starts from a smooth regional field. This replaces cell-scale rock scatter with recognizable provinces while preserving the designed continents and their geological activity distribution.

Minerals declare a `deposit_setting` and `province_scale_km`. Six modeled settings influence potential:

| Setting | Influences in this game model |
|---|---|
| Magmatic | Active geology and younger crust |
| Hydrothermal | Geological activity and water availability |
| Sedimentary | Sediment cover and quieter geological settings |
| Weathered | Warm, wet conditions and older exposed crust |
| Metamorphic | Activity and thickened crust |
| Evaporite | Dry conditions with standing water or sediment cover |

These are suitability proxies, not simulated mineral reactions. For example, standing water in the evaporite proxy is not proof of a closed saline basin. Historical deposits retain a background suitability where specified; active geology is not a universal requirement.

A continuous three-dimensional field supplies each mineral's regional variation, multiplied by its setting suitability and catalog abundance. Host-rock compatibility remains mandatory. Only the strongest compatible candidate is stored in each planetary cell. Mineral names use stable hash salts, so the field does not depend on a cube face's memory index or a mineral's position in the catalog. Province scale is a noise correlation setting, not a guaranteed ore-body diameter. Narrow veins, transport-derived placers and multiple overlapping deposits remain outside this version.

Potential remains dimensionless. Nominal depth comes from the mineral catalog plus sediment cover. The three weathering-associated targets (limonite, bauxite and malachite) now use shallow nominal depths of 15, 20 and 30 m instead of the previous hundreds-to-thousands of meters. These are configurable gameplay defaults.

## Exploration and extraction

The planet inspector identifies the selected deposit setting and province scale. Regional GPU snapshots inherit the mineral identity and potential and adjust approximate target depth by the local surface elevation difference. Inspection shows these values explicitly as inherited prospects.

No regional extraction stock is created. Existing town reserve allocation already reads planetary mineral potential; mining continues to consume its finite reserves. Opening or exporting overlapping patches cannot duplicate those reserves. Changed resource geography can alter town opportunities, but this increment does not recalibrate century-scale economies.

Regional JSON exports are version 2. Previously reserved components in the existing `forcing` vector now store mineral index + 1 (zero means absent), inherited potential and local target depth. The Rust `RegionalCell::mineral_index()` query exposes the identifier. No GPU buffer stride changes are needed. Version-1 regional records with zeroed reserved components report no prospect.

Planet archives keep their existing format and include the new catalog settings. Catalogs that omit `geological_provinces` default to false; omitted mineral settings use the legacy selection mode. Thus older worlds retain their original generation behavior. Setting the flag false provides a paired control in new catalogs as well.

## Deliberately deferred

The three inherited rock IDs still have no thickness, orientation or depositional history. The inspector now says so instead of labeling them simply as strata. Persistent stratigraphic columns should eventually record deposition, erosion and burial before regional chunks attempt excavation. Fault and volcano entities, dated eruptions, aquifers and cave geometry are also separate future changes.

## Verification

Run `cargo test --test geology -- --include-ignored --nocapture --test-threads=1` on a hardware GPU. The paired seed suite (17, 81, 256) compares rock-neighbor agreement, checks cube-face seams and host restrictions, and verifies regional prospect inheritance and JSON round trips. A controlled magmatic fixture holds host rock and spatial position fixed while changing activity. Catalog tests cover invalid scales and old-catalog defaults.

### Recorded seed results

Terrain edge 64, ecology edge 32, one geological epoch and one ecological year; unweighted cell statistics:

| Seed | Neighbor rock agreement, legacy → provinces | Seam agreement, provinces | Mean potential, legacy → provinces |
|---|---:|---:|---:|
| 17 | 11.08% → 36.45% | 52.34% | 0.15882 → 0.10402 |
| 81 | 10.50% → 35.82% | 53.91% | 0.15865 → 0.10145 |
| 256 | 10.65% → 35.50% | 49.48% | 0.15868 → 0.10129 |

Agreement is measured with the grid's positive-x neighbor, including seam crossings. It is resolution-dependent and measures local coherence, not the realism of an ore province. The lower mean potential warrants future history calibration; it was not compensated by increasing resource stocks. Legacy potential can exceed one, whereas the new process-weighted path is bounded by catalog abundance.

All 47 targeted checks passed: three geology tests, five core tests, sixteen ecology tests, thirteen managed-economy tests and ten GPU integration tests. They include the long geography seed suite, hydrology/water/sediment accounting, mining-budget conservation, regional drainage, archive continuation and exports. Clippy with warnings denied and formatting checks passed.

Follow-up: [persistent rock columns](stratigraphic-columns.md) now supplies finite thickness for new planets, with explicit limits on layer history and regional detail. Its archive and regional-export versions supersede the versions described above.
