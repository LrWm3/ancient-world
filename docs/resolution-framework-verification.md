# Projection / resolution framework verification

## Controlled evidence

- An analytical no-mortality projection checks fractional aggregate aging and whole
  individual birthdays/birth carry from the same inputs. Resolution leaves the
  projection untouched and conserves population.
- The common commit guard rejects mismatched revisions, duplicate settlement and
  backward boundaries, including after serialization.
- Comparison on/off uses identical histories and checks exact equality after removing
  only diagnostic state. Aggregate and individual modes are checked; individual
  mode also enables workshop refinement. Batched and single-month execution agree.
- A pre-framework individual archive, with no resolution field, initializes its
  commit ledger and advances successfully.
- A competing research commitment reduces workshop availability. Reversing offer
  storage order leaves the staffing result unchanged.
- A controlled installed-workshop fixture restricts staffing to one resident.
  Removing that resident's available time prevents the funded shift and payroll.
  The staffed branch pays the actual resident's household, preserves enterprise
  cash accounting and settles personal completed work. Serialization reproduces
  the reservation. A modified reserved grant is rejected by the recomputed input
  fingerprint before personal settlement. This fixture injects completed work explicitly; seed runs below
  exercise real GPU production.
- Existing identity birthday/death, defensive casualty and travel fixtures remain
  active, along with legacy aggregate market, society and ration tests.

## Integrated runs

Settings: terrain 32, ecology 16, one geological epoch, 16 starting civilizations,
living history, seeds 17/81/256, 25 years. Individual runs enable whole residents,
workshop refinement and comparison. Aggregate runs enable the shared aggregate
resolver and comparison, retaining sparse named residents and aggregate payroll.
These are independent branches, not isolated estimates of workshop behavior:
whole-resident initialization itself changes cultural opportunity and care demand.

The economy residual below is the largest absolute component of the final normalized
ledger, not a maximum across every monthly step. Raw output stays under output/.

| Individual seed | Residents | Resident overhang | Unresolved identities | Final economy residual |
|---|---:|---:|---:|---:|
| 17 | 2,481 | 0 | 0 | 2.38e-06 |
| 81 | 2,364 | 0 | 0 | 1.54e-06 |
| 256 | 2,481 | 0 | 0 | 2.80e-06 |

| Seed | Expected / actual births | Expected / actual deaths | Expected / actual child→adult transitions | Paid / completed workshop work |
|---|---:|---:|---:|---:|
| 17 | 1417.8 / 1408.0 | 838.6 / 844.0 | 1173.8 / 966.0 | 328.3 / 244.1 |
| 81 | 1399.3 / 1392.0 | 880.8 / 944.0 | 1139.4 / 932.0 | 949.7 / 778.7 |
| 256 | 1429.7 / 1420.0 | 869.0 / 859.0 | 1180.3 / 982.0 | 216.7 / 152.2 |

The whole-birth gap is retained fractional expectation, not lost residents. Birthday
aging differs substantially from the fractional cohort projection: these models
retain different information about age structure. That warrants age-distribution
analysis rather than forcing birthdays to match the forecast. Mortality differences
include sampling variation; this ensemble is too small to fit mortality parameters.

Normal workshop offers met nearly all forecast shifts in these seeds. The absence
fixture establishes the staffing connection under scarcity; the ensemble does not
show general labor scarcity. Completed work is below paid capacity because existing
production/material/demand constraints still apply. No output multiplier was added.


| Aggregate seed | Resident population | Final economy residual |
|---|---:|---:|
| 17 | 2388.75 | 1.75e-06 |
| 81 | 2422.29 | 1.70e-06 |
| 256 | 2432.40 | 1.40e-06 |

All three aggregate branches completed. Aggregate demographic resolutions match
their conditional expectations by construction; that is a consistency check, not
evidence that those expectations predict individual outcomes accurately.

## Reproduction and scope

    cargo test --lib -- --include-ignored
    cargo test --lib individual_demography -- --include-ignored
    cargo test --test society --test rations --test markets -- --include-ignored
    cargo clippy --all-targets -- -D warnings
    cargo run --release --example cultural_work_calibrate -- --individual-demography --workshop-refinement --compare-resolution --seeds 17,81,256 --years 25 --output output/resolution-individual.json
    cargo run --release --example cultural_work_calibrate -- --aggregate-resolution --compare-resolution --seeds 17,81,256 --years 25 --output output/resolution-aggregate.json
    cargo fmt -- --check
    python3 scripts/check_repository_artifacts.py

The full library run passed 168 tests. Seven targeted demographic tests (including
the additional old-archive case) and 13 market/ration/society integration tests passed.
The final staffing fixture passed after the input-revision check was added. All-target
Clippy with warnings denied, formatting, diff and repository artifact checks passed.

Rust 1.89.0, Vulkan, Quadro RTX 5000 with Max-Q Design. Concurrent compilation and GPU
tests make the run timings unsuitable for performance claims. This verifies history
serialization/batching, not a new cross-backend or full-world archive equivalence
study. Regional mode switching and richer individual reproductive behavior remain
outside this increment.
