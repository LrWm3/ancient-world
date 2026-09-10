# Ecological model and tuning

The simulation uses one set of GPU kernels for both continental regions. Regional differences enter through exposed reactive geology, microbial traits, nutrient retention, water, and available habitats. The extraordinary symbioses are fictional game biology, constrained by finite supplies rather than a biomass multiplier.

## Stocks, clocks, and units

A geological epoch exposes fresh rock and updates terrain, drainage and climate. It is followed by ten ecological years by default, each containing twelve monthly steps. Ecology-only steps freeze geological elevation while continuing seasonal weather, infiltration, snow, surface storage, watershed transport, biological turnover, and lake nutrient exchange. Batches per frame never change these steps.

Ecology defaults to 256 cells per face independently of terrain resolution. Diagnostic worlds use the smaller terrain resolution. Each ecological cell stores separate terrestrial and aquatic compartments and four geographic area fractions. Coarse coastal cells therefore do not treat their full area as land or water. Fine-grid nutrient and runoff payloads follow the converged terrain drainage graph; they remain in transit across months until reaching a receiving water cell. They never jump to a neighboring coarse watershed.

C/N/P stocks are kilograms per square metre of total ecological cell area, including each compartment's area fraction. Fine watershed payloads are absolute kilograms and cubic metres. GPU aggregation uses spherical areas. Initial inventories and external exchanges are explicit; reports sum in double precision on the CPU only when requested. Region summaries apportion mixed cells by geographic area fraction and are regional estimates, not extra fine-grid ecological detail. Productivity summaries annualize the most recently completed month; they are not yearly integrals. Budget reports explicitly flag residuals exceeding 0.1%.

Each ecological record contains 32 four-component vectors:

| Indices | Contents |
| --- | --- |
| 0–4 | Canopy, understory, root mat, shallow underground, deep fault biomass C/N/P; selected producer index plus one |
| 5–16 | Twelve animal guild C/N/P inventories |
| 17–19 | Soil, detritus, dissolved groundwater C/N/P |
| 20–24 | Surface water, deep water, sediment, plankton, buried/sorbed C/N/P |
| 25 | Source-rock phosphorus; fourth component records current water inventory including river transit |
| 26 | Hydrogen kg/m², secondary reducing energy MJ/m², reactive rock kg/m², available oxidant kg/m² |
| 27 | C/N/P external exchange ledger; fourth component water exchange in metres |
| 28 | Monthly photosynthetic C, chemosynthetic C, respired C, fixed N, all kg/m² |
| 29 | Basin-current direction, vertical exchange m/month, growth-constraint code |
| 30 | Initial C/N/P and water inventories |
| 31 | Geochemical source enabled, lake mixing multiplier, oxygen availability, disabled-guild bitmask |

`Ecology::budget` includes nutrients still traveling in rivers. Water bookkeeping includes representative geological water steps and explicit exchange imposed by the artistic lake boundary. Large ocean stocks can make a global relative water residual small; local water/sediment fixtures separately test terrestrial accounting.

## Processes

Photosynthesis depends on season, temperature, moisture, light penetration, and producer traits. Hydrogen production consumes reactive source rock and requires groundwater availability and tectonic activity. Secondary reduced chemistry and oxidants have bounded reserves. Microbial carbon fixation consumes those reserves; nitrogen fixation also costs production potential. Phosphorus and trace availability constrain the result. No-light vegetation can persist only where its symbiosis meets maintenance costs.

Decomposition respires carbon and transfer nitrogen and phosphorus into retained soil stocks or dissolved groundwater. Sorption stores phosphorus; microbial mobilization releases part of it. Weathering moves phosphorus from source rock to soil. New geological exposure is a recorded external import. Maintenance respiration removes carbon. Producers retranslocate excess N/P from remaining tissue into available soil stocks; animals excrete it into soil or water, and plankton return it to surface water. This prevents nutrients becoming trapped in carbon-depleted biomass. Recycling never creates carbon, nitrogen, phosphorus, or energy.

Guilds eat catalog-defined prey compartments. Assimilation, maintenance, movement, excretion, and death transfer or respire their actual biomass. Small seed populations are established by consuming available food, not by creating biomass. Migration depends on habitat, food, season, and—in large terrestrial guilds—uphill attraction. Migratory aquatic animals can carry nutrients onto land, where losses enter detritus. Body mass converts regional biomass to population equivalents for inspection; there are no individual animals.

The great lake has surface, deep, sediment, and plankton pools. Basin geometry, prevailing wind and shallow-shelf friction set constrained circulation; wind and neighboring depth gradients also modulate peripheral upwelling. horizontal exchange is bounded to at most 2% per edge per monthly step, preserving positivity with four neighbors. Upward and downward water exchange transfers existing nutrients according to layer depth. Outer-margin upwelling is a deliberate circulation constraint. Particle settling, sediment remineralization, low-oxygen phosphorus release, and burial form the deep reservoir.

Monthly water transport uses four conservative nearest-neighbor routing substeps. Great-lake receiving cells temporarily retain local inflow; the geological lake reduction re-equilibrates its overall level. This is a regional transport model, not a fluid-dynamics solution or a three-dimensional lake model. Cavern layers similarly represent regional habitat, not navigable cave geometry.

## Catalogs

The bundled version-two catalog contains 25 rocks, 26 minerals, 8 soils, 72 producers, 24 biomes, 12 animal guilds, and 8 microbial functional groups. It includes peridotite and serpentinite reactivity, apatite and phosphorite, twelve subterranean producers, six aquatic producers, and six additional surface producers.

Rock `chemistry` is `[P mass fraction, reactive fraction, annual release fraction, trace fraction]`. Soil `chemistry` is `[retention, annual sorption, annual leaching, annual decomposition]`. Producer `ecology` specifies layer (0–4 terrestrial, 5 aquatic), N/C and P/C ratios, maintenance, symbiotic capacity, fixation investment, shade response, and turnover. Microbial rates and efficiencies and animal feeding/migration parameters are bounded fractions. All values are game calibration inputs, not measured species physiology.

## Reproducible experiments

The explorer's **Ecological experiments** panel applies interventions at a completed epoch or ecology-only monthly boundary. Region selectors use each ecological cell’s dominant geographic region; the explorer labels the ecological grid resolution. Removing a guild transfers its biomass into detritus and blocks recolonization in the chosen region. Restoring it permits recovery when food and source populations exist. Disabling geochemistry stops new supply; accumulated reserves remain usable. Mixing changes transfer rates, not the inventory of phosphorus.

Use **Audit nutrient budgets** before and after interventions to inspect regional productivity, biomass, and conservation residuals. The panel retains the pre-intervention biomass comparison. Events and their exact months are saved in checkpoints.

A headless scenario file is a JSON array:

```json
[
  {"month": 120, "region": 3, "intervention": {"GeochemicalSupply": false}},
  {"month": 132, "region": 3, "intervention": {"RemoveGuild": 3}},
  {"month": 144, "region": 1, "intervention": {"LakeMixing": 10.0}}
]
```

```sh
cargo run --release -- --headless --resolution 256 --epochs 1 --months 120 \
  --scenarios assets/scenario-example.json --budget output/budgets.json \
  --save output/experiment.world --export output/production.png --layer 16
```

Month values are absolute ecological time and must fall within the requested ecology-only interval. `region: null` applies globally. The full API supports `Generator::advance_ecology`, `Generator::scenario`, `Ecology::inspect`, `snapshot`, and `budget`.

## Archive format

Current saves use version 7 with a 400-byte environment record containing conditional geological habitats, growth diagnostics and wildlife edge conductance. Versions 2–6 remain readable without resetting inventories. See [wildlife assembly](wildlife-assembly.md) for the new ancestry baseline. See [geochemical habitats](geochemical-habitats.md) for the current schema and calibration.

Version two uses `ANCIENT2`, an eight-byte little-endian metadata length, JSON metadata, terrain cells, ecological cells, environment cells, fine routed payloads, and an eight-byte FNV checksum. Field names, units, grid sizes, catalogs, both clocks, and scenario history are included. The inactive ping-pong buffers need not be saved; every pass writes its complete output. Checkpoint loading validates the payload before upload.

Version-one terrain archives require explicit import:

```sh
cargo run --release -- --headless --import-v1 output/old.world --epochs 0 \
  --save output/imported.world
```

Import preserves terrain, upgrades catalog content, records new initial ecological inventories, and marks `imported_baseline`. It never claims exact continuation of the old ecological model and never overwrites the source unless the user explicitly selects the same output path.

## Regional calibration

The values below describe the earlier ecology model. Current habitat-aware results and reproduction commands are in [geochemical habitats](geochemical-habitats.md).

`cargo run --example calibrate -- 10` evolves seeds 0, 7, 42, 99 and 999 for 100 ecological years on a 64² diagnostic terrain/ecology grid, then samples twelve additional months. `output/calibration.json` contains integrated annual photo/chemo production, biomass by terrestrial layer, available phosphorus, reactive stock, hotspot area fraction, and conservation residuals. The run rejects budget failures and loss of the intended primarily-solar inner ecology. These are diagnostic comparisons, not claims of physiological realism or production-resolution convergence.

A geochemical hotspot is a cell with monthly chemo production above 0.00001 kg C/m² and above 10% of photosynthetic production. It is a reporting threshold, not a growth modifier. Geological activity now comes from terrain source distributions; ecology no longer applies a regional multiplier to geological energy production.

The final five-seed 101-year diagnostic suite produced 0.049–0.056 kg photosynthetic C/m²/year on inner continents and 0.158–0.161 on the outer continent. Inner vegetation retained 0.49–0.57 kg C/m², versus 1.13–1.15 on the outer continent. These are measured outcomes, not enforced ratios. Geochemical hotspots covered 0.88–3.43% of outer land under the reporting definition. Inner production remained over 99.8% solar. Maximum relative C/N/P residual was 2.09×10⁻⁵ and water residual 1.62×10⁻⁷.

The century check exposed a missing nutrient-return path: before maintenance-linked recovery/excretion, inner plant stocks declined below their initial baseline. A dedicated GPU fixture now checks that maintenance returns excess body nutrients without increasing total inventory. The catalog also assigns trees to canopy, shrubs and vines to understory, and grasses/mosses to ground cover rather than cycling layers by entry order.
