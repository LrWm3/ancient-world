# Estate vacancy verification

Rust 1.89.0; Vulkan on the Quadro RTX 5000 with Max-Q Design. Generated logs and
world archives remain outside Git. This is verification of toy accounting and
identity rules, not historical or demographic calibration.

## Controlled evidence

The population-registry fixtures deliberately remove all known adult heads from
one town and assign its unchanged population stock to children. They check that:

- Exhausted adult and elder slots produce no new person or population.
- Each account retains its original head reference, ownership share and wallet.
- A vacant estate cannot relocate or supply its dead head for cultural work.
- With no candidate, the polity returns an empty living-leader query rather than
  inventing a ruler. The historical leader reference remains valid through the estate.
- Repeated social boundaries do not duplicate vacancy events or accumulate claims
  on future anonymous adults.
- A single newly available adult slot restores exactly one account; the other
  estates remain vacant. Dead predecessors stay dead.
- With another eligible head available, council caretaking is separate from estate
  inheritance: the old estate is still vacant and no new person is created.
- Disabling named demography does not undo existing vacancies by inventing heads.
- Missing fields in older archives default to no vacancy. Invalid vacancy dates
  and unmarked dead heads are rejected.
- JSON continuation preserves vacancy and recovery. A world-archive fixture compares
  24-month advancement with reload plus 24 single-month advances.

The enterprise fixture starts a real financed operator through the existing startup
path, then makes its owner's account vacant. It checks closure, release of the
lease, one-time return of exactly the company's remaining cash, unchanged physical
equipment and unchanged normalized money residual. A later entry date cannot make
the vacant owner start another firm. The account still receives a resident food
entitlement through the existing aggregate retail planner; no new food is added.

These are declared boundary fixtures. Reclassifying the fixture's age stock is not
claimed to be an endogenous demographic outcome or a population-loss experiment.

## Reproduction

    cargo test --lib
    cargo test --lib population_registry -- --include-ignored
    cargo test --lib enterprises::tests -- --include-ignored
    cargo test --test politics --test culture --test expeditions -- --ignored
    cargo clippy --all-targets -- -D warnings
    cargo run --release --example cultural_work_calibrate -- --seeds 17,81,256 --years 100 --output output/vacancy-seeds.json
    cargo fmt -- --check
    python3 scripts/check_repository_artifacts.py

The century runs use terrain 32, ecology 16, one geological epoch, 16 initial
civilizations, living history, domestic care and named demography enabled. No tuning
parameters changed. They exercise ordinary histories; the controlled fixtures above
supply the deliberately extreme conditions needed to test vacancy itself.

## Limits and verification backlog

The complete ignored library suite is not rerun for this increment. The prior broad
run found three failures also reproduced on its unchanged baseline: routed overflow,
sea-freight cargo expectations and harbor construction capacity. Those remain
unresolved; see [the prior verification record](resident-succession-verification.md).

Complete resident rosters, actual dependent counts, personal payroll, probate courts
and common identity allocation across all population adapters remain unfinished.
The tests do not establish cross-hardware equivalence or empirical realism.

## Century results

All three runs completed without a reported accounting or validation failure.
Positive identity overhang is summed separately by site; a deficit elsewhere never
cancels a surplus. No living identity was unresolved at the final boundary.

| Seed | Residents | Known residents | Child / adult / elder overhang | Vacancy / recovery events | Caretaker appointments | Maximum absolute normalized economy residual |
|---|---:|---:|---:|---:|---:|---:|
| 17 | 2,887.30 | 570 | 0.00 / 0.00 / 0.00 | 0 / 0 | 0 | 5.66e-06 |
| 81 | 3,093.07 | 502 | 0.00 / 0.00 / 0.00 | 0 / 0 | 0 | 1.90e-05 |
| 256 | 2,554.70 | 386 | 0.00 / 0.00 / 0.00 | 0 / 0 | 0 | 4.54e-06 |

Normal seed outcomes are not the proof of vacancy correctness: the extreme boundary
fixtures directly exercise no available successor, property preservation and later
recovery. The resident population remains authoritative in the GPU cohorts, and
many residents remain unnamed. Concurrent timings are not performance benchmarks.

## Checks

107 distinct selected tests passed: 78 ordinary library tests, five GPU population
registry tests, four GPU enterprise tests, and 20 culture/expedition/politics integration
tests. The ordinary tests also include two population-registry and four enterprise
checks, so those are not counted twice. The vacancy world-archive test and the
century-long genealogy continuation test both passed. Formatting, Clippy across all
targets, diff whitespace and the repository artifact check passed.
