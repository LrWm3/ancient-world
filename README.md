# Ancient World

Ancient World is a toy world generator and a game-like experiment that implements basic world generation.

Its not intended to be a realistic simulator, its just a hobby project for fun.

## Try it

Requires Rust 1.89 and a hardware GPU supported by wgpu (Vulkan, Metal or DX12).
Linux also needs a graphical session for the explorer. The project includes Rust toolchain and mise configuration.

```sh
cargo run --release
# Alternatively, through mise:
mise exec rust@1.89.0 -- cargo run --release
```

The explorer starts paused. Use **Evolve** or **Step** to advance generation. Drag
the globe to orbit, drag the atlas to pan, scroll to zoom, and select a cell to
inspect it. Defaults can be expensive; start with a smaller resolution if needed.
Open **Civilizations beta** to experiment with histories on the inner continents. New histories enable optional systems by default; expand **Optional systems** before
founding to change them.

```sh
# Small headless experiment; generated files stay in output/
cargo run --release -- --headless --resolution 256 --epochs 10 \
  --save output/planet.world --export output/planet.png

# Explore a saved world
cargo run --release -- --load output/planet.world

# See available settings
cargo run --release -- --help
```

New headless histories also default to all systems on. Override individual systems
with `--disable-system` or `--enable-system` (comma-separated names; see `--help`):

```sh
cargo run --release -- --headless --civilizations 5 --history-years 10 \
  --disable-system expeditions,adaptive-prices
```

Disabling a prerequisite also disables its dependent systems. Existing saves keep
their settings unless overrides are supplied. See [system options](docs/system-options.md)
for dependencies, configuration files and startup versus live controls.

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
