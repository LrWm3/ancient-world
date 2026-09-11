# Resident roster verification

## Controlled checks

- Whole-resident identification leaves site population, age stocks and resources
  unchanged, fills all available whole slots, records unknown parents, is idempotent,
  and matches 12-month batched versus serialized single-month continuation.
- Relocation carries an explicit nonempty passenger manifest, reserves sufficient
  age stocks and exposes those identities as traveling. Duplicate IDs are rejected.
- A blocked journey consumes its finite provisions over 200 monthly checks. Its
  population accounting remains balanced and all passengers eventually die when its
  cohort becomes extinct. Serialized continuation matches uninterrupted execution.
- Existing relocation tests retain destination-capacity reservation, housing refusal,
  return, ancestry/faith continuity, relief testimony and financial/material checks.

## Reproduction

    cargo test --lib -- --include-ignored
    cargo test --test politics --test culture --test expeditions -- --ignored
    cargo clippy --all-targets -- -D warnings
    cargo run --release --example cultural_work_calibrate -- --resident-baseline --seeds 17,81,256 --years 10 --output output/roster-seeds.json
    cargo fmt -- --check
    python3 scripts/check_repository_artifacts.py

Rust 1.89.0, Vulkan on Quadro RTX 5000 with Max-Q Design. Outputs remain in ignored
output/. This verifies toy stock/identity rules, not empirical demographic realism.

## Ten-year expanded-roster runs

Seeds 17, 81 and 256 completed with terrain 32, ecology 16, one geological epoch,
16 initial civilizations, living history and the resident baseline enabled. No
validation or accounting failure was reported. These short runs are smoke tests,
not calibration or evidence of long-run individual demographic consistency.

| Seed | Population | Known resident identities | Positive local excess: child / adult / elder | Household departures | Largest absolute final normalized economy residual |
|---|---:|---:|---:|---:|---:|
| 17 | 2147.99 | 2086 | 0.00 / 23.80 / 3.29 | 1 | 3.73e-7 |
| 81 | 2130.96 | 2074 | 0.12 / 40.18 / 2.42 | 4 | 7.15e-7 |
| 256 | 2177.73 | 2114 | 0.00 / 49.11 / 7.10 | 0 | 6.60e-7 |

No living identities were unresolved at the final boundary. Positive age-band excess
is summed locally, never canceled by missing identities in other towns or bands.
The excess is a real remaining inconsistency: fixed birthdays and sampled identity
mortality differ from fractional cohort aging and deaths. Identifying everyone once
does not solve that authority boundary. The expanded baseline therefore remains
explicit/opt-in. The next census work should replace these competing demographic
transitions, not erase excess people to make the table look better.

Durations (approximately 13–14 seconds per seed) were measured during concurrent verification and
are not performance benchmarks. Raw logs and expanded domestic-group snapshots are
not committed.

## Final checks

159 library tests passed with all hardware tests included, zero failures and zero
ignored. The 20 culture, expedition and politics hardware integration tests also
passed, including century history and checkpoint continuation. The final relocation
fixture additionally checks vacant-estate state after a passenger head dies.

All-target Clippy with warnings denied, formatting, diff whitespace and the source-only
artifact check passed. This is 179 distinct tests; repeated verification runs are
not counted twice. Other integration targets and other GPU backends were not tested
for this increment. Final full-library execution took about 88 seconds during
concurrent work and is not a benchmark.
