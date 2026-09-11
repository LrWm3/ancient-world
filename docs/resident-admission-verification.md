# Resident admission and regression verification

This increment shares whole-resident admission checks between birth identification,
service recruitment and succession. It does not change population authority or
calibrate demographic parameters. GPU cohorts remain authoritative.

## Evidence

The analytical admission fixture checks fractional slots, refused batch atomicity,
two claims on the last slot, isolation by site and age, stock decline, legacy death
observation and invalid stocks/indices. A GPU-backed history fixture lets succession
identify the last anonymous adult, then asks service recruitment for another:
recruitment refuses without creating another identity or changing population.
Existing vacancy, birth/death and archive fixtures exercise the same consumers.

Three previously outstanding library failures have their assumptions corrected:

- **Flood pulse:** the exact half-release fixture now uses zero axial tilt. Its
  prescribed −5°C baseline previously warmed seasonally, evaporating some of the
  pulse before drainage. It now separately asserts zero precipitation and evaporation
  before checking exact retained/runoff water and nutrient conservation. Production
  weather and evaporation are unchanged.
- **Sea freight:** the fixture prescribes both projected arrival and remaining
  travel, and advances every monthly interval. Editing only the arrival field and
  jumping quarters did not complete the persisted voyage clock. Intermediate checks
  assert remaining travel, unchanged arrival date, continued freight reservation and
  checkpoint agreement. Existing goods/money and port-role deduplication checks remain.
- **Harbor construction:** structural construction and deterioration assertions use
  installed harbor capacity. An additional assertion requires total shipping capacity
  to remain zero without staffed vessels. Building a harbor does not create a fleet.
  Material costs, bounded labor, repeated-year behavior and restoration remain checked.

These are corrected controlled experiments, not relaxed tolerances or disabled
production mechanisms. Seasonal climate, funded vessel travel and harbor/fleet
separation retain their dedicated tests.

## Reproduction

    cargo test --lib -- --include-ignored
    cargo test --test politics --test culture --test expeditions -- --ignored
    cargo clippy --all-targets -- -D warnings
    cargo fmt -- --check
    python3 scripts/check_repository_artifacts.py

Use the repository's Rust 1.89.0 toolchain. Hardware tests require a supported GPU.
Generated logs stay in ignored output/. No raw results or binaries are committed.

## Results

On Rust 1.89.0 with Vulkan / Quadro RTX 5000 with Max-Q Design:

- Full library suite including hardware tests: **158 passed, zero failed, zero ignored**.
- Culture, expedition and politics hardware integration tests: **20 passed**,
  including century-long genealogy and checkpoint/batch continuation.
- All-target Clippy with warnings denied, formatting, diff whitespace and repository
  artifact checks passed.

This is 178 distinct passing tests, not a claim that every integration target or
hardware backend was exercised. The earlier library pass had 157 passing tests;
its count is not added again. Final library execution took about 105 seconds while
other checks ran concurrently; this is not a performance benchmark.

No yield, mortality, funding or climate parameters were tuned. No new seed ensemble
was run: the existing century integration and controlled boundary fixtures cover
this admission refactor. Full population rosters, roster-based relocation, individual
payroll and conversion of remaining legacy representative creation are still pending.
