# Geological regions and readable rock maps

New worlds with `process_geology = true` first choose a geological setting, then a
compatible catalog rock. The setting depends on plate-boundary stress and relative
motion, elevation, water/land region, latitude and a broad spherical basin field:

- Active convergent belts favor volcanic exposure.
- Compressive uplands favor regional metamorphic exposure; intrusive margins can
  expose contact-metamorphic rocks.
- High terrain outside sedimentary basins can expose plutonic rocks, with limited
  mantle-rock exposures near extensional uplands.
- Quieter regions alternate between marine/carbonate and continental clastic
  settings. Rare quiet, subtropical basin interiors can hold evaporites.

Rock entries declare their setting, validated against their formation family.
A broad, warped world-space field chooses among compatible rocks. This replaces
independent fine-scale rock competition; it is independent of cube-face indices.
Subsurface identities, subsequent volcanic/metamorphic transformations and new
lithified sediment use the same compatible selection. Canonical rock IDs continue
to drive hardness, erosion, weathering, chemistry and mineral-host eligibility.

These are game-generation proxies for depositional and exposure environments.
The basin field is not reconstructed sea-level history or a physical sedimentary
basin solver. They do not model folding, intrusive geometry or detailed oceanic
crust differentiation. More continuous maps do not alone establish geological
accuracy. This increment changes rock distributions and therefore can change
resource availability and subsequent ecological/economic trajectories.

## Map

The map uses rust tones for igneous rocks, ochre for sedimentary rocks, and blue-gray
for metamorphic rocks. Stable rock-specific brightness variations come from one
shared catalog palette used by both the GPU and the UI legend. The expandable key
lists individual rocks; clicking a cell remains the definitive identification.
Water is blue rather than a rock-color overlay; submerged geology remains available
in cell inspection. Terrain relief also shades the geological map. Geological
colors do not invent fine-scale strata during zoom.

## Compatibility

The bundled catalog enables the new model. Catalogs and archives lacking the flag
default to the previous province model. `geological_provinces = false` still enables
the original legacy generator. Missing rock settings default to generic members
of their formation family; a missing specialized rock falls back to its family.
Existing saved rock identities are not regenerated when loaded.

## Verification

```sh
mise exec rust@1.89.0 -- cargo test --test geology -- --include-ignored --nocapture
mise exec rust@1.89.0 -- cargo test --all-targets
mise exec rust@1.89.0 -- cargo clippy --all-targets -- -D warnings
```

The GPU suite compares three seeds against the previous province model, checks
land-neighbor coherence and diversity, cube-face seam coherence, deposit-host
compatibility, regional inheritance, finite column erosion/lithification and
process-dependent deposit potential. Catalog tests check formation/setting
compatibility and archive defaults. Raw logs and screenshots remain local under
ignored output paths; only this summary is committed.

At terrain resolution 64, after one epoch, matched land-neighbor agreement was:

| Seed | Previous provinces | Process settings | Rock types on land |
|---|---:|---:|---:|
| 17 | 36.24% | 77.62% | 25 |
| 81 | 35.56% | 75.91% | 25 |
| 256 | 36.44% | 78.82% | 23 |

Cross-face neighbor agreement in the new model was 89.32%, 94.53% and 89.84%
respectively (all terrain, including water). These measure spatial continuity, not
physical realism or equal behavior across resolutions. All seven geology tests
passed on Vulkan. An initial catalog test caught an unassigned diorite setting;
assigning it to plutonic rocks fixed the failure. The final GPU results above use
that corrected catalog and compressive metamorphic updates.

The 256-resolution seed-42 desktop was inspected visually: the new palette,
shorelines, broad regions and legend render correctly. Screenshots remain in
`output/geology-views/`; none are committed. Long-run economy recalibration and
cross-resolution geological convergence remain future checks.

Final ordinary suite: 65 passed, 137 hardware fixtures ignored by that command.
The separate geology run exercised its four GPU fixtures. Clippy and formatting
checks passed. These runs do not claim full-suite GPU or cross-hardware validation.
