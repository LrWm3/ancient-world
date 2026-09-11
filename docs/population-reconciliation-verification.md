# Population identity verification

## Reproduction

Rust 1.89.0; Quadro RTX 5000 with Max-Q Design, Vulkan. Raw outputs are ignored under
output/. These are toy-rule verification and regression runs, not demographic fitting.

    cargo test --lib
    cargo test --lib population_registry -- --include-ignored
    cargo test --lib domestic::tests -- --ignored
    cargo test --lib military::tests -- --ignored
    cargo test --lib participation::tests::personal_absence -- --ignored
    cargo test --lib relocation::tests -- --ignored
    cargo test --test expeditions --test politics --test culture -- --ignored
    cargo run --release --example cultural_work_calibrate -- --years 100 --output output/population-reconciliation-final.json
    cargo run --release --example cultural_work_calibrate -- --years 100 --legacy-named-demography --output output/population-reconciliation-final-control.json

The paired controls use the same program with the adapter disabled. Defaults are
seeds 17, 81 and 256, terrain 32, ecology 16, one geological epoch, 16 civilizations,
living history, care enabled. Concurrent run timings are not performance benchmarks.

## Mechanism checks

- Boundary ages 0, 15 and 60 classify consistently, including integer extremes.
- Historical birth credit alone cannot create a new identity in an already represented
  child cohort. A newly available slot permits one identity without another stock debit.
- Two half-death allocations produce one named death and retain only fractional carry.
  Old unused death credit at other sites does not trigger deaths.
- Assignment changes no population/food stock and consumes exactly one existing death
  credit. Repeating the same month's assignment does nothing.
- Serialization followed by assignment matches uninterrupted assignment exactly.
- Expedition identities are excluded from local demographic mortality. Every existing
  identity appears in exactly one reconciliation category.
- A deceased head cannot launch an ownership-account relocation before succession.
  The existing conserved relocation fixture continues to pass.
- Legacy histories lacking the new state retain the old adapter.

The integrated domestic, genealogy, culture and expedition fixtures exercise exact
archive continuation and monthly/batched equivalence. Boundary fixtures use declared
synthetic stocks; they are independent of a particular seed becoming hungry or fertile.

## Initial diagnosis

Before the adapter, seed 81 at year 100 had 788 named resident children, 1,741 adults
and 389 elders, against cohort stocks of 763.77, 1,415.40 and 654.86. Eight people were
on expeditions; none were unresolved. Travelers therefore did not explain the adult
excess. Summing positive excess separately at each site exposed 260.76 children,
510.18 adults and 27.15 elders; global totals concealed some local mismatches.

The corrections address two causes: identifying children beyond available slots,
and protecting named people under 70 from local cohort mortality. Fixed birthdays,
fractional cohort aging, succession representatives and ownership-account movement
still prevent exact identity/cohort agreement.

## Final paired results

Positive overhang is summed by settlement and age band; a deficit elsewhere does not cancel it. The relocation guard is active in both controls.

| Seed | Residents: new / legacy | Local excess children: new / legacy | Adults | Elders | Known residents: new / legacy | Maximum absolute normalized economy residual: new / legacy |
|---|---:|---:|---:|---:|---:|---:|
| 17 | 3,032.17 / 2,827.38 | 0.00 / 68.89 | 16.45 / 344.28 | 0.00 / 17.73 | 1,268 / 2,676 | 5.19e-06 / 8.19e-06 |
| 81 | 2,992.84 / 2,834.03 | 0.00 / 260.76 | 5.53 / 510.18 | 0.00 / 27.15 | 1,232 / 2,918 | 1.65e-05 / 1.99e-05 |
| 256 | 2,550.63 / 2,424.25 | 0.00 / 25.95 | 0.00 / 500.93 | 0.00 / 39.88 | 945 / 2,436 | 5.97e-06 / 5.95e-06 |

All six runs completed without a reported accounting or validation failure. No parameter
was fitted to these endpoints. Smaller overhang is the relevant identity-alignment
result; higher resident population in these three runs is downstream historical
divergence, not evidence that the adapter reduces physical mortality. The controlled
fixture proves it assigns identities without debiting population again.

There are now fewer named residents, leaving many explicit unrepresented slots. This
is preferable to treating excessive named records as population, but is not a finished
census. Residual adult overhang remains in two seeds. Birthdays, representative
succession and roster-free population movement still need a shared population authority.

## Final checks

104 distinct selected tests passed: 77 ordinary library tests, two new GPU boundary
fixtures, two domestic, one military, one personal-work, one relocation, nine culture,
six expedition and five politics tests. Age-boundary coverage also appears in the
ordinary library total. The century-long genealogy checkpoint test passed. Clippy
across all targets, formatting and the source-only artifact check passed. The full
ignored GPU suite and other backends/resolutions were not exercised.
