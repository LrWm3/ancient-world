# Ancient World

Ancient World is a toy world generator and a game-like experiment in AI-assisted
procedural world generation. The idea is to stack a lot of interacting systems—
geology, weather, ecology, farming, trade, religion and politics—and see what falls
out of them. AI helps build and iterate the project; seeded simulation rules
produce the worlds and their histories.

**Simulation accuracy is not a project goal.** The systems use simplified rules,
fictional assumptions and parameters chosen for experimentation. The aim is to
make interesting worlds with consequences that carry between systems. Tests focus
on whether those connections work, resources are accounted for, and runs can be
reproduced. They do not establish that the results describe real ecosystems or
historical societies.

The setting puts several island continents inside a vast inland sea, surrounded
by a much larger ancient continent and an exterior ocean. The island civilizations
establish permanent settlements only on the central islands **by design**; their
journeys to the ancient continent are expeditions, not a path to colonization.
Whether the ancient continent should have civilizations of its own is still an
open design question, not a committed feature.

A native Rust globe and atlas explorer lets you watch the world evolve, inspect its layers and follow its
settlements. Dense environmental systems run on GPU compute shaders; sparse social
history runs on the CPU.

See the [documentation index](docs/README.md) for system guides and experiments,
including [crop and price comparisons](docs/crop-and-price-experiments.md) that
record failed outcomes as well as useful changes.

## Try it

Requires Rust 1.89 and a hardware GPU supported by wgpu (Vulkan, Metal or DX12).
Linux also needs a graphical session for the explorer. Software adapters are
rejected. The project includes Rust toolchain and mise configuration.

```sh
cargo run --release
# Alternatively, through mise:
mise exec rust@1.89.0 -- cargo run --release
```

The explorer starts paused. Use **Evolve** or **Step** to advance generation. Drag
the globe to orbit, drag the atlas to pan, scroll to zoom, and select a cell to
inspect it. Defaults can be expensive; start with a smaller resolution if needed.
Open **Civilizations beta** to experiment with island histories. Some extensions
require explicitly enabling their controls.

```sh
# Small headless experiment; generated files stay in output/
cargo run --release -- --headless --resolution 256 --epochs 10 \
  --save output/planet.world --export output/planet.png

# Explore a saved world
cargo run --release -- --load output/planet.world

# See available settings
cargo run --release -- --help
```

## Things to experiment with

These are rough, uneven systems, not a checklist of fully realized game features.
A name such as “climate,” “religion” or “economy” describes a collection of rules
and records. It does not imply a comprehensive model of that subject. Many of the
connections are narrow, and some optional experiments produce poor outcomes.

| Area | What you can explore |
|---|---|
| Geography and geology | Constrained island/lake geography, terrain variation, plate-inspired fields, rock and mineral distributions. [Geology](docs/geological-provinces.md) |
| Water and weather | Grid drainage, water storage, seasonal weather rules and erosion. [Regional generation](docs/regions.md), [living history](docs/living-history.md) |
| Ecology | Plant and animal biomass, nutrient bookkeeping, simplified food webs and fictional geochemical habitats. [Ecology](docs/ecology.md), [wildlife](docs/wildlife-assembly.md) |
| Farms and workshops | Managed plots, crop and livestock rules, material inventories and production orders. [Economy](docs/economy.md), [experimental alternatives](docs/crop-and-price-experiments.md) |
| Exchange and travel | Local quotes, shipments, roads, harbors and provisioned household relocation. [Shipping](docs/shipping.md), [relocation](docs/household-relocation.md) |
| People and politics | Aggregate populations, some named family records, competing interests, administration and compact conflict rules. [History guide](docs/civilizations.md), [factions](docs/faction-interests.md) |
| Foundings and beliefs | Patron-led arrivals, human traditions, institutions and attributed accounts. [Foundings](docs/patron-foundings.md), [religious change](docs/religious-dynamics.md), [modest institutional facilities](docs/material-objects-and-facilities.md) |
| Expeditions | Funded journeys, temporary camps, hazards and modest returning discoveries. [Expeditions](docs/expeditions.md) |
| Places and possessions | Abstract site assets, some persistent objects and shared resource stocks. [Site assets](docs/site-assets.md), [resources](docs/shared-resources.md) |
| Inspection | Globe/atlas layers, regional surveys, event records and a short retained settlement timeline. [Timeline](docs/history-timeline.md) |

The project is not a finished colony-management or adventure game. Regions are
surveys, not excavatable, playable towns. Historical events can be repetitive,
political and economic decisions are simplified, and tuning can produce collapse
or stagnation. The present-day map is not a reconstruction of every past year.
There is no goal of matching Dwarf Fortress or a scientific simulator in depth,
accuracy or feature coverage.

## Saves, experiments and development

The Rust library exposes generation, inspection, history advancement and archive
operations. Desktop and headless modes use the same generator. See the
[documentation index](docs/README.md) for controls, API examples and individual
system assumptions. Older archives can retain different settings; continuing them
with changed software does not promise the same future.

Checks cover selected accounting rules, small fixtures, archive handling and
repeatability. Passing them does not demonstrate that an entire world is well
balanced or behaves like a real one. Hardware-dependent tests are skipped by an
ordinary test run; run them explicitly on a supported GPU.

```sh
cargo test
cargo test --test gpu -- --ignored --test-threads=1
cargo test --test economy -- --include-ignored --test-threads=1
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The [evidence notes](docs/model-evidence.md) and
[experiment summaries](docs/evidence/README.md) record specific checks and outcomes,
including failures. Older reports describe the revision tested, not a guarantee
about every current setting. Archived designs are discarded or provisional ideas,
not commitments to implement their full scope.

Generated exports, logs and experiment results belong in ignored `output/`.
Commits contain source, editable catalogs and human-readable summaries.
