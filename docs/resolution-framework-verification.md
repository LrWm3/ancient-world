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

## Same-state demographic replay

Added replayable latest-month inputs to comparison receipts. The public
`DemographicSnapshot::compare()` runs the aggregate resolver and, when reconciled
opening identities were captured, the individual resolver without committing either.
Aggregate-only capture explicitly reports unavailable individual inputs.

Verification:

- Eight demographic tests passed, including the hardware GPU fixtures, using
  `cargo test --lib individual_demography -- --include-ignored`.
- Analytical birthday fixture: one child reaching adulthood becomes one adult in
  individual resolution, versus 1/180 adult in aggregate resolution. Both preserve
  total population exactly with zero mortality and births.
- JSON replay produces identical outcomes and leaves snapshot inputs unchanged.
  Duplicate identities, incorrect birthday bands, inconsistent roster totals and
  nonfinite birth carry are rejected.
- Twelve-month comparison enabled/disabled histories remain identical after removing
  diagnostic state, in both demographic modes (including refined workshops in the
  individual branch). Captured replay births, deaths and both aging transitions
  match their actual committed receipts.
- Existing checkpoint, old-archive admission, travel, defensive casualty and
  conversion-rejection fixtures still pass.

These are conditional one-month comparisons after food/disease exposure has already
been computed. They do not model counterfactual food consumption or run an alternative
full history. No new balance rates were introduced and no new long-run seed calibration
was performed for this diagnostic-only increment.

## Workshop offers: completed practice, pay and household pressure

Offers now use actual posted wages relative to local food prices, household cash
buffers and last completed food-access records. Completed workshop practice improves
hiring priority with diminishing returns; paid idle time does not. Physical recipe
yields and firm wage-setting rules are unchanged.

Controlled checks cover monotonic wage/pressure responses, zero-pay refusal, bounded
offers and practice-based priority. The enterprise fixture compares food-pressure
records with identical cash and business capital under constrained personal capacity.
An earlier fixture changed household cash as well and confounded enterprise entry
funding with labor willingness; it was corrected rather than treated as evidence of
a labor effect. The original absence, payroll-recipient, stale reservation, checkpoint
and work-settlement checks remain part of that fixture.

Three ten-year living-history runs used terrain 32, ecology 16, one geological epoch,
individual demography and workshop refinement on the Quadro RTX 5000 Vulkan backend:

| Seed | Resident population | Local cohort overhang | Unresolved identities | Max absolute final normalized economy residual |
|---|---:|---:|---:|---:|
| 17 | 2,155 | 0 | 0 | 1.15e-6 |
| 81 | 2,118 | 0 | 0 | 6.16e-7 |
| 256 | 2,167 | 0 | 0 | 4.38e-7 |

Reproduction:
`cargo run --example cultural_work_calibrate -- --individual-demography --workshop-refinement --seeds 17,81,256 --years 10 --output output/workshop-offer-seeds.json`

These are smoke checks, not a matched long-run calibration or evidence that aggregate
labor forecasts are accurate. Existing service ceilings and finite employer demand
can mask differences in individual willingness. Family-specific technical skills,
competitive wages and allocation across all production sectors remain future work.
The 12-month comparison-on/off batch-equivalence fixture also passes with the new offers.

## Workshop family experience

Completed personal workshop work now records one of the four existing recipe
families. Matching experience contributes fully to hiring priority and other/untyped
experience at 20%, with diminishing returns. It changes who receives a scarce shift,
not the shift's wage, time or material yield.

Four focused CPU tests pass:

- Two equally motivated specialists exchange hiring priority when only the requested
  workshop family changes. Total granted time remains identical.
- Completed work credits its actual family; paid idle time credits none.
- Invalid families and repeated settlement fail without changing the learning ledger.
- Older resident records load with their total experience intact and zero invented
  family allocations. Untyped experience provides the same modest benefit in all
  families, and serialization preserves newly recorded practice.

The enterprise fixture additionally compares family learning totals against actual
settled firm work, retaining payroll, absence, stale-input and checkpoint assertions.
The participation validator now rejects nonfinite/negative total experience and
family allocations exceeding that total; the old total had lacked this check.

No new seed calibration was performed for this increment. This remains a coarse
hiring-experience proxy: no technical yield bonus, apprenticeships, skill decay or
competitive wage negotiation. The transfer coefficient is a toy tuning choice.

Full verification: `cargo test --lib -- --include-ignored` passed all 173 tests,
including GPU fixtures, in 79.82 seconds. All-target Clippy with warnings denied,
formatting and the repository artifact policy also passed.

## Learning alongside experienced coworkers

The optional individual workshop adapter now gives novices a separate, bounded
competence score when they complete work alongside more experienced coworkers at
the same firm. It uses opening family experience. Exposure is limited by both
coworkers' completed work and divided among eligible novices; competence has
diminishing returns and adds a modest hiring-priority benefit next month.
It is not extra work and never enters payroll or physical production totals.

Controlled coverage:

- An absent/idle expert, an idle novice, or equally experienced coworkers produce
  no learning gain.
- More novices divide the available expert exposure instead of multiplying it.
- Canonical person order makes learning independent of crew array order.
- A productive two-person enterprise crew develops novice competence; the same
  assignments with zero completed production do not.
- The enterprise crew resumes identically from serialization. Repeating settlement
  cannot award another gain.
- Existing resident archives initialize the new competence array to zero.

This is incidental learning, not a formal apprenticeship contract, teaching
schedule or a calibrated model of knowledge acquisition. Competence does not yet
decay or affect physical output quality. No dedicated new long-run seed ensemble
was added for this increment.

Verification: all 175 library tests passed with `cargo test --lib -- --include-ignored`
(including GPU fixtures) in 77.06 seconds. All-target Clippy with warnings denied,
formatting and repository artifact checks passed.
