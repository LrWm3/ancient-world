# Individual demography verification

This checks toy population accounting and identity continuity, not empirical
fertility or mortality. Individual demography remains opt-in.

## Controlled fixtures

- A child's 180-month and an adult's 720-month birthday move their actual identity
  into the correct cohort; population accounting remains balanced.
- Whole births create registered people without a second genealogy birth-credit
  allocation. Death events identify people actually removed from resident counts.
- 24 batched months equal serialized History continuation with 24 single-month
  advances on the same backend. This is a history-state checkpoint comparison;
  it does not claim a new full-world archive/backend portability test.
- Incompatible conversion leaves the original History unchanged; repeated enable
  is idempotent.
- Defensive loss expectations of .75 then .5 kill one identity and retain .25,
  including across serialized continuation.
- Arrival distinguishes a birthday completed in transit from the arrival month's
  birthday. Cohort transfers preserve total passengers.
- An analytical total-loss travel fixture removes two named passengers and one
  anonymous person exactly; reapplying it to the empty manifest removes nothing.

## Ten-year integrated runs

Terrain 32, ecology 16, one geological epoch, 16 initial civilizations, living
history; seeds 17, 81, 256. The final normalized economy residual column is the
largest absolute component of the final ledger, not a maximum over all months.

| Seed | Final resident population | Known residents | Local cohort overhang | Relocation departures | Final economy residual |
|---|---:|---:|---:|---:|---:|
| 17 | 2,151 | 2,151 | 0 | 3 | 1.26e-6 |
| 81 | 2,113 | 2,113 | 0 | 4 | 6.15e-7 |
| 256 | 2,169 | 2,169 | 0 | 0 | 3.90e-7 |

All three had no unresolved resident identities. Earlier baseline-only runs had
local adult overhangs of roughly 24–49 people after ten years. These are different
demographic trajectories, not a controlled estimate of population effects.

The first integrated attempt stopped at seed 17, month 30: fractional defensive
casualties reduced a cohort below its living roster. The next attempt stopped at
month 39: a passenger had become an adult in transit but arrived as a child.
Those failures motivated the whole-defender and travel-age fixes above. Checks
remain strict rather than silently altering identities to fit invalid stocks.

## Fifty-year extension

The same seeds and settings were extended to 600 months. Resident totals exclude
people currently on military/expedition service or relocating; those retain separate
identity and population accounting.

| Seed | Residents / known residents | Military away | Relocation departures | Final economy residual |
|---|---:|---:|---:|---:|
| 17 | 2,837 / 2,837 | 19 | 3 | 3.60e-06 |
| 81 | 2,857 / 2,857 | 0 | 14 | 5.51e-06 |
| 256 | 2,830 / 2,830 | 0 | 0 | 3.52e-06 |

All three completed with zero local overhang and no unresolved identities. This
small ensemble does not establish coverage of every late-history service, settlement
or archive-conversion path.

## Reproduction

    cargo test --lib -- --include-ignored
    cargo test --test politics --test culture --test expeditions -- --include-ignored
    cargo clippy --all-targets -- -D warnings
    cargo run --release --example cultural_work_calibrate -- --individual-demography --seeds 17,81,256 --years 10 --output output/individual-seeds.json
    cargo run --release --example cultural_work_calibrate -- --individual-demography --seeds 17,81,256 --years 50 --output output/individual-50years.json
    cargo fmt -- --check
    python3 scripts/check_repository_artifacts.py

The full library suite passed: **163 tests**, including the hardware-marked tests.
The culture (13), expedition (6) and politics (5) integration tests also passed,
for **187 tests** across these invocations, with none ignored. All-target Clippy
with warnings denied, formatting and artifact-policy checks passed.

Rust 1.89.0; Vulkan on Quadro RTX 5000 with Max-Q Design. Raw logs and structured
results stay in ignored output/. Concurrent tests make timings unsuitable for a
performance comparison.
