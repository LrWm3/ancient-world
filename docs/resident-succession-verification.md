# Resident succession verification

This records the 33f8043 increment. The subsequent [estate vacancy change](estate-vacancies.md)
removes the living-head fallback discussed below.

Rust 1.89.0, Vulkan, Quadro RTX 5000 with Max-Q Design. Raw results and logs are
ignored under output/. These checks test the toy model's identity and accounting
rules; they are not demographic calibration against historical data.

## Mechanism checks

The controlled population-registry fixture checks:

- A present adult child succeeds before an unrelated account member.
- A child on expedition cannot succeed; the existing account member can.
- Two simultaneous successions cannot assign the same resident twice.
- Existing resident succession does not create people or change population stocks.
- Parent records survive appointment; kin membership agrees with the new account.
- A migrant's civilization identity is not rewritten to fill a ruling vacancy.
- An anonymous elder slot is used when adult slots are exhausted.
- Exhaustion of both slots emits one explicit succession-identity-overhang event.
- JSON continuation produces the same succession state and events.
- A successor created before the deceased head is valid. Separate analytical graph
  checks reject self-links, cycles, and out-of-range predecessor IDs.

The initial century run stopped at seed 17, quarter 98 with `invalid historical
figure`. Validation had assumed predecessor ID < successor ID. That was compatible
with freshly invented representatives but not reuse of older identities. It now
checks an acyclic succession graph independently of ID creation order. The first
expedition rescue integration run exposed the same assumption. Both are retained
here as failures encountered, rather than omitted from the verification history.

## Reproduction

    cargo test --lib -- --include-ignored
    cargo test --lib population_registry -- --include-ignored
    cargo test --test politics --test culture --test expeditions -- --ignored
    cargo clippy --all-targets -- -D warnings
    cargo run --release --example cultural_work_calibrate -- --seeds 17,81,256 --years 100 --output output/resident-succession-verified.json
    cargo fmt -- --check
    python3 scripts/check_repository_artifacts.py

The ensemble uses terrain 32, ecology 16, one geological epoch, 16 starting
civilizations, living history, domestic care and named demography enabled. No
parameters were fitted. Comparison below uses the saved runs from c3043e0 with
those same settings. Long-run differences are downstream divergence, not an
isolated causal estimate of population effects. The controlled fixture demonstrates
that succession itself does not debit or create population.

## Unrelated failures found by broad testing

Three full-library fixtures also fail on the unchanged parent commit c3043e0,
reproduced in a detached worktree on the same GPU:

- `ecology::flood_tests::routed_overflow_is_finite_and_drains_back_to_runoff`:
  drained surface water differs from the fixture's expected 0.3.
- `economy::freight_tests::sea_trade_shares_freight_with_inland_approaches_and_deduplicates_ports`:
  the fixture expects an empty cargo list.
- `shipping::harbor_tests::harbor_work_limits_construction_and_preserves_condition_history`:
  the fixture expects port capacity >999 after construction.

Their cause is not diagnosed in this increment. Do not interpret the full library
suite as passing. These remain an explicit verification backlog, separate from the
succession regression fixed here.

## Century ensemble results

All three final runs completed without reported accounting or validation errors.

| Seed | Resident population: previous / current | Known residents: previous / current | Local adult overhang: previous / current | Explicit succession overhang events | Maximum absolute normalized economy residual |
|---|---:|---:|---:|---:|---:|
| 17 | 3,032.17 / 2,840.07 | 1,268 / 576 | 16.45 / 0.00 | 0 | 4.92e-06 |
| 81 | 2,992.84 / 3,093.07 | 1,232 / 502 | 5.53 / 0.00 | 0 | 1.90e-05 |
| 256 | 2,550.63 / 2,554.70 | 945 / 386 | 0.00 / 0.00 | 0 | 4.54e-06 |

All age bands had zero positive local overhang at the final boundary, and no living
identity was unresolved. These endpoints do not prove there was never a transient
mismatch. Fewer new representatives were needed because surviving residents could
fill ownership roles. Many cohort residents remain unnamed; a smaller named roster
is not demographic mortality and is not a completed census. Population increased
in two seeds and decreased in one relative to the prior runs; no preferred population
endpoint was imposed. Concurrent timings are not a performance benchmark.

Vacant ownership accounts, complete rosters, roster-backed domestic relocation and
a common identity allocation interface remain future work. The no-slot fallback is
covered by a controlled fixture even though none of these three century runs used it.

## Final checks

The full library run completed with 150 passes and the three baseline failures
listed above. All 20 selected integration tests passed (nine culture, six expedition,
five politics), including the previously failing rescue and century-long genealogy
checkpoint/batch continuation. The five population-registry checks were rerun on the
final source and passed; they are already included in the library count. Clippy
across all targets, formatting, diff whitespace and repository artifact checks passed.
No new source, time-step or cross-hardware calibration claim is made.
