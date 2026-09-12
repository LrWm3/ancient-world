# Wildlife assembly: explicit founders and habitat connectivity

This first increment makes guild occupancy depend on prior populations and accessible
neighbors. It does **not** yet implement species, evolving ecotypes, per-island
lineages, reproductive isolation, or demographic population viability.

## Research translated into mechanisms

- [Valente et al. (2020), A simple dynamic model explains the diversity of island
  birds worldwide](https://www.nature.com/articles/s41586-020-2022-5): colonization,
  extinction and speciation vary with isolation and area. We implement the prerequisite
  of explicit occupied sources and geographic access. We do not transplant fitted
  bird rates into all animal guilds or claim that guild occupancy is species richness.
- [Emergent encoding of dispersal network topologies in spatial metapopulation
  models (2023)](https://doi.org/10.1073/pnas.2311548120): connectivity and the
  distinction between settled populations and dispersers motivate explicit source
  transfers. Here dispersal is a bounded monthly flux, not a separately resolved
  explorer population; travel time and survival remain simplified.
- [Benítez-López et al. (2021), The island rule explains consistent patterns of body
  size evolution in terrestrial vertebrates](https://www.nature.com/articles/s41559-021-01426-y):
  body-size responses provide a future comparative target. We add no universal
  island dwarfism bonus: the central landmasses are continent-sized.

These are mechanism references, not empirical calibration of this fictional world.
No external example code was copied.

See the [calibration report](wildlife-calibration.md) for retained raw seed trials,
rejected parameter changes, analytical intake verification and remaining long-run
food-web failures. Geographic isolation is implemented; ecological calibration is
still incomplete.
The [trophic follow-up](wildlife-trophic-stability.md) addresses low-density feeding
and predator overshoot, with separate controlled and long-run evidence.

## Stocks, initialization and ancestry

Existing guild pools 5–16 retain C/N/P in kg/m² of whole ecological cell. The previously
unused fourth component stores the fraction of guild biomass descended from outer-associated
founders. It is a **neutral tracer**, not a separate nutrient stock,
name, species ID, or inherited advantage. Zero includes both central and open-water
founders. In mixed coastal cells, water founders use the outer share of adjacent land;
this is a coarse association, not a claim that fish descended from land animals.
Aquatic basin ancestry cannot be reconstructed from this single tracer.

New worlds declare initial animal inventories alongside initial producer and soil
stocks. Each guild starts with 1e-6 kg C/m² of its habitat in smooth, seeded geographic
patches; N/P follow catalog stoichiometry. Patches use spherical position and a seeded
phase, so they do not break at cube faces. These are explicit pre-human starting
conditions, not a reconstruction of ancient land bridges. They are not patron arrivals.

The old `(biomass + habitat * epsilon)` feeding rule is replaced by
`biomass * feeding_rate * dt`. Empty animal stocks cannot feed or reproduce. Existing
food limitation, assimilation, maintenance, excretion and mortality still determine
subsequent abundance. Very small stocks below 1e-12 kg C/m² transfer their remaining
C/N/P to detritus or sediment; this is numerical cleanup, **not** a biologically fitted
minimum viable population.

Uniform feeding losses, mortality and growth preserve the ancestry share. Transport
mixes ancestry by carbon mass:

```text
incoming ancestry carbon = incoming carbon × source ancestry fraction
new fraction = (retained ancestry carbon + incoming ancestry carbon)
               / (retained carbon + incoming carbon)
```

Movement respiration removes carbon with the mixed ancestry share. No ancestry mass
is interpreted as extra animal biomass. `RestoreGuild` permits immigration again;
it cannot repopulate a world in which the guild has been removed everywhere.

## Habitat edges

Three vec4 fields (environment 22–24) cache left/right/down/up conductance from the
actual fine terrain edges. Aggregation updates them as terrain and water change.

- Terrestrial: both fine cells are land; elevation difference reduces conductance
  by `1 / (1 + abs(delta_elevation_m)/1000)`.
- Open water: both are the same water class (great lake or exterior ocean).
- Migratory river guild: connected downstream routing with discharge >1 m³/s at
  both ends, a flowing river mouth, or connected open water of the same class.
- Waterbirds retain cross-habitat movement, bounded by the existing monthly flux.

Conductance is the mean over the shared fine edge. The movement limit remains 2%
per neighbor per month, and absolute transfers use the smaller spherical cell area.
The diagnostic config `wildlife_open_barriers=true` replaces conductance with one;
it is an intentionally unrealistic ablation, not recommended world configuration.

Important limits: this tests shared edges, not connected components *inside* each
coarse cell. Separate habitats within one coarse cell still share a guild stock.
River feeding habitat, secondary-lake populations, flight distance, rare rafting,
waterfalls and separately timed migrant cohorts need further work. The fixed monthly
fraction also makes dispersal distance resolution-dependent. The current mechanism
is suitable scaffolding, not a resolution-independent dispersal model.

## API, explorer and compatibility

`Ecology::wildlife_report` returns area-weighted regional carbon, occupied area
(>1e-10 kg C/m²), and carbon-weighted outer ancestry per guild. Regions are ocean,
great lake, central land and outer land. Allocation within mixed cells is approximate;
these are not fine-grid animal censuses or species-diversity statistics.

The cell inspector includes a Wildlife origins panel. Ecology buffers remain 608
bytes/cell; the derived environment grows from 352 to 400 bytes/cell (18 MiB extra
at ecology resolution 256). Existing allocation checks include this increase.

Archive version 7 stores the expanded environment and `wildlife_baseline=1` for new
worlds. Versions 2–6 load without reseeding animals or changing their inventories;
missing derived fields rebuild during aggregation. Their missing ancestry baseline
is marked 0 and displayed as untracked. They adopt the new feeding/movement rules on
advancement, so this is not a promise of matching an older executable's future.

## Reproduction

```sh
mise exec rust@1.89.0 -- cargo test --lib wildlife_tests -- --ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --test ecology -- --ignored --test-threads=1
mise exec rust@1.89.0 -- cargo run --release --example wildlife_evaluate -- 120 32
mise exec rust@1.89.0 -- cargo run --release --example wildlife_evaluate -- 1200 64
```

The evaluator pairs seeds 17, 81 and 256 with intact/open barriers. It writes starting
and final regional measures, C/N/P/water budgets, complete config and GPU stage timings
under `output/wildlife`. Tests cover absence/restoration, coarse coastal separation,
actual recolonization, analytical ancestry mixing, fine-edge CPU reference checks
including all cube seams, conservation and archive continuation.

## Next useful extension

Add multiple persistent populations within guilds, with their own partitioned biomass
and traits, using this explicit-source movement path. Track landmass/watershed IDs and
within-cell habitat connectivity before claiming per-island endemism. Add specialist
diets and environmental tolerances before adaptation; keep geological energy entering
animals through real producer/prey stocks. Formal species splitting comes after those
mechanisms, not as random names applied to the neutral tracer.

## Later increment

The opt-in [thermal ecotype pilot](wildlife-thermal-ecotypes.md) adds persistent
regional temperature preferences and growth-linked adjustment. This supersedes
the absence of any persistent trait described above; it does not add species
identities or genetic populations. Current archives use version 8.
