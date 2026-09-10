# Ancient World

A native GPU world generator and globe/atlas explorer. A vast inland sea holds five island continents inside a surrounding supercontinent, with an exterior ocean beyond it. Natural history evolves through compute shaders; the CPU schedules work, handles the UI, catalogs and sparse social-history graphs, and reads explicit inspection/export data.


*The globe and atlas show the island continents, surrounding inland sea, and enclosing supercontinent.*

See the [documentation index](docs/README.md) for current system guides, evaluation reports, and archived designs.

## Run

Requires Rust 1.89 and a hardware Vulkan, Metal, or DX12 adapter. Linux also needs a graphical session for the explorer. The project includes `rust-toolchain.toml` and a project-local mise configuration. Software adapters are rejected; there is no CPU simulation fallback.

```sh
cargo run --release
# With mise, if its configuration has not been trusted:
mise exec rust@1.89.0 -- cargo run --release
```

The explorer starts paused. **Evolve** advances natural history; **Step** advances one bounded batch. Drag the globe to orbit, drag the atlas to pan, scroll to zoom, and click either view to inspect a cell. Both views share the selected cell and layer.

The default planet has six 512² faces, a 6,371 km radius, 23.44° tilt, and a target of 1,000 epochs. Settings include 256/512/1024 face resolutions, seed, island count, radius, tilt, epoch length, climate passes, and drainage convergence limit. Apply history controls at an epoch boundary to change physical steps. Batches per frame changes execution throughput without changing the modeled steps. **Pause** stops scheduling new batches; an already submitted GPU batch finishes first. Regenerating replaces the current world.

```sh
# Headless generation, checkpoint and full atlas export
cargo run --release -- --headless --resolution 256 --epochs 10 \
  --save output/planet.world --export output/planet.png

# Resume exactly at the saved epoch and evolve ten additional epochs
cargo run --release -- --headless --load output/planet.world --epochs 10 \
  --save output/continued.world

# Open a saved world in the desktop explorer
cargo run --release -- --load output/planet.world

# All three production resolutions; output/benchmark.json
cargo run --release -- --benchmark --epochs 1
```

Use `--config assets/example.toml` to supply full world settings and `--catalog assets/catalog.toml` to edit material/ecology data. A supplied config takes precedence over `--seed` and `--resolution`. `--layer 0..30` chooses the exported layer in the order shown in the UI. `--save` and `--export` are headless options; the desktop has its own save/export controls.

## Systems

- **Cube-sphere topology:** shared cross-face connectivity, spherical distances, solid-angle cell areas; no artificial polar cells or longitude boundary in the simulation.
- **Regional generation:** `Generator::generate_region` creates independent 16–1024² GPU grids with fine terrain, drainage ranks, tributaries, a one-year runoff/storage balance, soils and habitat-constrained plant assignments. Use **Generate explorable region** to inspect fine cells, or export PNG plus a catalog-bearing regional JSON archive. The globe retains its inexpensive visual refinement. See [regional generation](docs/regions.md).
- **Geography:** seeded inner continents with varied sizes, elongated and lobed outlines, and a broad open-water belt separating them from the enclosing shore; enclosed great lake, surrounding land and exterior ocean. Geographic class masks are immutable artistic constraints; elevation evolves within class-specific bounds.
- **Tectonics:** 16 spherical plates rotating about seeded Euler axes, domain-warped boundaries, relative-motion-driven uplift/subsidence, crust aging, volcanic resurfacing and metamorphism. This is a regional plate-field approximation, not a mantle solver or material advection model.
- **Drainage:** GPU minimax flood, lexicographic spill/rank routes, connected depression labels, and convergent river accumulation. Secondary water now moves through neighboring cells with spherical-area-weighted, inventory-limited transfers. Dry internal ridges separate pools until water reaches the saddle; overflow releases only water above the outlet spill. Surface relaxation reports unresolved convergence and preserves terrain. See [solver tolerances and limits](docs/regions.md).
- **Climate:** repeated seasonal temperature, winds, vapor advection, orographic precipitation, snow/melt and annual averages. Each epoch repeats seasonal cycles until every cell’s annual means change by at most 0.25 °C and 10 mm/year and same-season temperature/vapor change by at most 0.25 °C / 0.1 mm, or the configured cycle budget is exhausted (default 32, with early exit). Unresolved climate convergence is reported explicitly. Weather is a regional approximation.
- **Water:** infiltration, groundwater release, local surface retention and river overflow. The great lake uses deterministic GPU tree reductions for inflow, rainfall, evaporation, water level and dissolved salts. The exterior ocean is an external reservoir. The lake level is constrained to 85–240 m to preserve the enclosed setting; the accumulated artificial exchange is recorded in `planet_state()[2]` in 10¹² m³.
- **Erosion:** rock weathering, downstream fluvial erosion/deposition and conservative thermal sediment transfer. Initial mountains contain branching ridges and valleys; great-lake bathymetry grades through continental and island shelves into the deep basin. Regional strata are three rock IDs plus sediment and soil thickness, not excavation-ready voxels.
- **Ecology:** GPU carbon/nitrogen/phosphorus inventories, solar and finite geochemical production, five terrestrial ecological layers, stratified lake nutrient transport, and twelve migrating animal guilds. The catalog contains 25 rocks, 26 minerals, 8 soils, 72 producers, 24 biomes, and 8 microbial groups. See [the ecological model](docs/ecology.md) for units, controls, assumptions, and experiments, and [geochemical habitats](docs/geochemical-habitats.md) for preserved hotspot fractions, underground growth diagnostics, and seed comparisons.

An epoch advances tectonics → convergent drainage/basins → seasonal climate → runoff and convergent flow → great-lake reduction → water/erosion → ecology. Geological years per epoch apply to tectonics; seasonal climate is settled within its cycle budget and one water cycle is evaluated as a representative geological environmental step, followed by configurable ecological years (default ten) with monthly weather, water and biological evolution. Dense state remains in GPU storage buffers. Normal rendering reads those buffers directly into GPU textures; only convergence/timing diagnostics and explicitly selected cells return to the CPU.

## Persistence and API

The library exposes `Config`, `Catalog`, `ContextGpu`, `Generator`, `Progress`, `Stage`, `Cell`, and `MapRenderer`. `Generator::advance` executes a bounded batch; `run_epochs` is a blocking headless convenience. `inspect`, `snapshot`, `planet_state`, `save`, and `load` provide explicit access. `rebuild_drainage` supports diagnostic fixtures and leaves the stage at climate.

A `.world` file contains a versioned JSON header (configuration, catalogs, progress, units, grid convention, reservoir and random-state description), packed little-endian terrain and ecological cells, circulation controls, in-transit river payloads, and a checksum. Version-three saves also include patron, cultural and expanded economic records; version-two saves retain both environmental clocks and scenario history; version-one worlds require explicit `--import-v1` to initialize a new ecological baseline. Writes use a temporary file and rename. Saves occur at completed epoch boundaries or completed ecology-only months; a mid-epoch desktop save remains queued until that boundary, including while paused. Invalid versions, lengths, indices, nonfinite values, and checksums are rejected. Checkpoints resume reproducibly on the same GPU/backend and software version; cross-backend floating-point results can vary.

The UI shows estimated simulation allocation and optional GPU timestamps. GPU timings sum timestamps around every compute pass in a batch. Memory estimates include both state buffers and reduction scratch, plus an allowance for UI textures; driver allocations and host checkpoint copies are additional. Runtime adapter limits are checked before allocation. Nonconvergent drainage pauses with an error and retains its stage so a larger limit can be applied.

## Verification

```sh
# Reproducible verification + matched mine/staffing experiments (hardware GPU):
python3 scripts/evidence.py --profile full
# CPU-only CI coverage; hardware skips remain visible:
python3 scripts/evidence.py --profile cpu

cargo test
cargo test --test gpu -- --ignored --test-threads=1
cargo test --test ecology -- --include-ignored --test-threads=1
cargo test --test civilization -- --include-ignored --test-threads=1
cargo test --test economy -- --include-ignored --test-threads=1
cargo test --test society -- --include-ignored --test-threads=1
cargo test --test politics -- --include-ignored --test-threads=1
cargo test --test governance -- --include-ignored --test-threads=1
cargo test --test markets -- --include-ignored --test-threads=1
cargo test --lib -- --include-ignored --test-threads=1
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo run -- --resolution 256 --smoke-test output/explorer.png
```

Hardware tests cover an independent CPU priority-flood reference, cross-face connectivity, nested depressions, exact checkpoint continuation, all map shaders, PNG padding, archive corruption, habitat restrictions, seed topology, long evolution, water/sediment accounting, and convergence-limit errors. CPU reference implementations exist only in tests. Hardware tests are explicitly ignored in ordinary CI so missing GPU access cannot masquerade as a tested CPU fallback.

See `output/benchmark.json` after running the benchmark for measurements on your adapter. No fixed world-generation time is promised: resolution, history length, basin geometry, and GPU bandwidth dominate the cost.

The ecology grid defaults to 256² per face independently of terrain resolution. `--ecology-resolution` and `--ecology-years` override these settings. `--months` advances ecology alone after headless geological history; `--budget` exports conservation and regional productivity reports. The desktop exposes the same controls and reproducible nutrient-cycle experiments. See [measured GPU performance](docs/benchmarks.md).

For a regional export, run `cargo run --example region -- output/planet.world output/region.png` (optional cell ID and width in km follow the output path). The default is a 600 km square at 1024² cells centered on dry mountainous terrain. It writes both a PNG and a `.region.json` containing the actual fine fields and catalogs. Extents must be 1 km to half the planet radius.

For the five-seed century-scale ecological comparison, run `cargo run --example calibrate -- 10`. Results and tuning assumptions are documented in `docs/ecology.md`. Current verification includes 54 tests, including nutrient excretion, shared lake levels across cube seams, regional export determinism, and compact-drainage recovery.

The [current civilization guide](docs/civilizations.md) connects the implemented economy, families, politics, religions, expeditions and environmental feedback, and identifies remaining limits. Open **Civilizations beta** in the sidebar to found and inspect central-island societies.

[Managed farming, crafts and markets](docs/economy.md) now run in new civilization worlds. Farms reserve actual ecological C/N/P and water; finite timber/ore/clay feed workshops, and paid shipments move goods between central-island settlements. Select a settlement to inspect production and change farming or market policies. Existing first-beta histories offer an explicit economy upgrade.

[Social history](docs/society.md) adds opt-in GPU age cohorts and seasonal grain, household ownership and inheritance, terrain-constrained trade/relief routes, tax-funded councils and roads, provisioned raids and causal event references. Use **Enable social history** or headless `--society`; existing economic saves keep their behavior until enabled.

[Genealogy and territorial politics](docs/politics.md) extends social worlds with recorded parents and marriages, child inheritance, competing council factions, central-island claims and equipment/provision-limited campaigns. Enable **Dynasties and politics** or pass `--politics` after `--society`. The atlas displays administrators and disputed claims.

[Governance and diplomacy](docs/governance.md) adds paid administration, local autonomy, legitimacy, administrative secession, trade-based diplomatic trust and expiring non-aggression agreements. Enable it explicitly on political histories with **Governance and diplomacy** or `--governance`.

Paired-seed civilization history reports, regional drought forcing and multi-hop commercial markets are documented in [History evaluation](docs/history-evaluation.md). Run `cargo run --example history_evaluate -- --help` for reproducible controls and comparisons. See [measured results](docs/history-evaluation-results.md) for the stress experiments, gentler final default and mature-world continuation.

[Inter-island shipping](docs/shipping.md) now connects central-island markets through surveyed lake routes, materially funded harbors, reserved cargo capacity and trade-driven diplomacy. Enable it from the civilization controls or with `--shipping` on social history.

Wealth-funded research and rescue voyages are described in [Ancient-continent expeditions](docs/expeditions.md), with [multi-seed results](docs/expedition-results.md) and [functional crew competence](docs/expedition-crews.md). Enable them in the history panel after governance and shipping, or run `cargo run --release --example history_evaluate -- --expeditions --civilizations 5 --save-worlds --output output/expeditions`.

[Finite expedition specimens](docs/discoveries.md) extend returning voyages into local research, fictional remedies and phosphorus extraction. Enable **specimen research and applications** after expeditions, or add `--discoveries` to the history evaluator.

See [specimen calibration results](docs/discovery-results.md) for the demand-driven production refinement and seed comparisons.

### Living history

Enable **Living world** in the civilization panel to advance seasonal GPU ecology with every social month. **Evolve / Pause** and **Step** then control the coupled simulation. Existing archives remain opt-in; terrain and river geometry stay fixed while weather, water storage and ecosystems change. Seasonal storms can flood towns, damage food and crops, interrupt roads and ports, and delay cargo until recovery. See [controls, accounting and limits](docs/living-history.md) and [flood mechanics and results](docs/flood-history.md).

New-world island abundance settings reduce geological phosphorus, local managed footprints and attainable crop yields. See [calibration and seed results](docs/island-abundance.md). Existing archives retain their saved supply rules.

[Patron foundings and living traditions](docs/patron-foundings.md) now accompany new histories: named temporary guides, attributed founding accounts, household faiths, institutions, inherited objects and locally transmitted practices. Six crops, three livestock groups and finite fisheries feed the expanded GPU economy. The document includes reproducible controls, archive migration, calibration results and current regional abstractions.

[Demand-driven production](docs/demand-economy.md) adds town work orders, ingredient planning, bounded dry storage, use-related replacement demand, food-first purchasing and shared land-freight capacity. New worlds enable it; existing worlds can opt in from the history controls without replacing their inventories.

[Household relocation](docs/household-relocation.md) lets struggling island communities send provisioned families to nearby towns with spare food production and land. Journeys preserve family identity, consume finite supplies, and appear in history; the civilization panel controls new departures.

[Witnessed relief](docs/witnessed-relief.md) makes outside emergency assistance depend on arriving families carrying dated appeals. Household ties influence departure; distance, inter-town relationships and host reserves influence the response.

[Local food webs](docs/food-webs.md) add persistent competing producer mixtures and weighted guild diets, including aquatic detrital pathways, with conserved nutrient pools and archive upgrades.

[Wildlife assembly](docs/wildlife-assembly.md) adds explicit founder populations, habitat-connected dispersal, local disappearance and ancestry tracking. Regional species and evolving ecotypes remain future work.

[Geological provinces and regional prospecting](docs/geological-provinces.md) describes coherent rock/mineral fields, formation-setting controls and inherited regional resource targets.

[Persistent rock columns](docs/stratigraphic-columns.md) adds finite unit thickness, erosion-driven exposure, sediment lithification, depth queries and version-six archive compatibility.

[Social observations and pressure memory](docs/social-indicators.md) adds bounded settlement summaries and persistent hardship that informs existing migration and political decisions.

[Household income and retail food](docs/household-economy.md) adds finite wallets, paid food entitlements, and explicit transfers. [Household-owned workshop operators](docs/workshop-operators.md) lease existing equipment, fund wages from separate accounts, earn fees for completed GPU work, and close when capital or demand fails.

[Institutional operating capacity](docs/institution-capacity.md) connects upkeep, staffing, flooding, teaching and sponsorship. [Institutional succession](docs/institutional-succession.md) adds local mandates, paid ballots and service vacancies.

[Persistent site assets](docs/site-assets.md) preserve yard and shelter baselines after population loss, with material-backed warehouses and housing, shared construction labor, wear in abandoned towns, crowding pressure, housing-aware relocation, and finite waterworks with staffed sanitation and domestic water accounting.

Shared finite extraction sources, regional survey continuity and a living-history
checkpoint counterfactual are described in [shared resources](docs/shared-resources.md).

The [model evidence report](docs/model-evidence.md) maps requirements to checks,
specifies the investigated stock/flow rules, and separates game balancing from
empirical calibration. The evidence runner retains failed/incomplete runs and
records source/catalog checksums, hardware, monthly trajectories and measured errors.

[Road upkeep](docs/road-upkeep.md) connects weathering, flood damage and finite repair labor to existing road materials and travel costs.

[Harbor work](docs/harbor-work.md) makes port construction and repairs compete with roads for finite craft labor, with persistent deterioration and recovery.

Experiment outputs stay in ignored `output/`; only [human-readable summaries](docs/evidence/README.md) belong in commits.
