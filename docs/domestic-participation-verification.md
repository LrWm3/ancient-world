# Known-family participation verification

This increment observes families among named people; it does not create the full
resident population. The controlled fixture identifies a child already represented
by a settlement cohort. All experiment outputs remain ignored under output/.

## Reproduction

Using Rust 1.89.0 and a hardware Vulkan adapter:

    cargo test --lib domestic::tests -- --ignored
    cargo test --lib
    cargo test --lib participation::tests::personal_absence -- --ignored
    cargo test --lib military::tests -- --ignored
    cargo test --test expeditions --test politics --test culture -- --ignored
    cargo run --release --example cultural_work_calibrate -- --years 100 --output output/domestic-enabled.json
    cargo run --release --example cultural_work_calibrate -- --years 100 --no-domestic-care --output output/domestic-disabled.json

The runner defaults to seeds 17, 81 and 256, terrain 32, ecology 16, one geological
epoch, 16 founding civilizations and living history. Enabled/disabled pairs use
the same seeds. Timings from concurrent verification runs are not benchmarks.

## Controlled claims

- Recorded partners in different ownership accounts join a domestic group with their
  recorded child; observation changes no goods, money or cohort stocks.
- Repeating observation and reservation does not duplicate memberships or work.
- A child under five requests 0.12 worker-months. Two healthy parents each reserve
  0.06, reducing personal service capacity from 0.80 to 0.74. With one deceased,
  the survivor reserves 0.12 and has 0.68 remaining.
- A joint departure decision can approve one parent but cannot also take the last
  caregiver; unsettled care prevents a mid-work departure.
- Supplying only 0.06 of a 0.12 grant records 0.06 used, releases 0.06 and leaves
  no reusable supplied labor. Repeated settlement has no additional effect.
- JSON continuation preserves the unsettled receipt. The integrated archive test
  compares twelve batched months with twelve single-month steps after save/load.
- Adulthood splits a child into an independent group; widowhood preserves the union
  identity. Duplicate membership is rejected. Legacy import begins observations
  at the current month, without inventing earlier family history.

These test the implemented toy rules and timing, not empirical caregiving behavior.
Population-wide care, anonymous dependents and nutritional outcomes remain outside
this adapter. A small ensemble can reveal regressions but cannot establish broad
calibration or high-resolution performance.

## Three-seed comparison

Quadro RTX 5000 with Max-Q Design, Vulkan; 100 years per run:

| Seed | Residents, care enabled / disabled | Active towns, enabled / disabled | Completed care worker-months | Known domestic members / groups, enabled | Maximum absolute normalized economy residual, enabled / disabled |
|---|---:|---:|---:|---:|---:|
| 17 | 2,827.38 / 2,707.22 | 16 / 16 | 32,308.08 | 2,702 / 1,201 | 8.19e-6 / 6.64e-6 |
| 81 | 2,834.03 / 3,214.19 | 17 / 18 | 33,632.65 | 2,926 / 1,246 | 1.99e-5 / 2.23e-5 |
| 256 | 2,424.25 / 2,606.55 | 16 / 16 | 30,261.40 | 2,436 / 1,256 | 5.95e-6 / 5.87e-6 |

Disabled runs completed zero care. No run reported an invalid work receipt or
accounting failure. Long-run population differences are downstream divergences;
they do not show that care directly increases or decreases fertility. The immediate
mediators are reserved personal time and reduced town service/production capacity,
verified in the controlled fixture. No parameter was tuned to force a demographic
sign or guarantee an expedition.

Known domestic membership is **not interchangeable with resident population**. It
includes people away from home and remains attached to the older sparse identity
and cohort adapters. In two runs it exceeds the resident total. A complete census
rollout must reconcile identities and movement against demographic stocks rather
than simply making this membership count the population authority.

## Regression discovered during verification

The heritage expedition test originally depended on a particular destination being
fundable after 20 years of unconstrained history. With care enabled, its eligible
town had 62.7 kg of tools against a 96.8 kg departure threshold; alternative routes
lacked harbor capacity. This was production scarcity, not missing crew identities.
The heritage fixture now disables care during its setup, as it already disables
diversified farming, to isolate minor finds and archive continuation from economic
viability. It retains explicit rejection diagnostics. The other five expedition
fixtures continue to run with care enabled. No production threshold or inventory
was bypassed in the simulation.

## Final checks

100 selected tests passed after correcting the heritage fixture: 76 ordinary library
checks, two domestic GPU fixtures, the personal-work and military fixtures, nine
culture, six expedition and five politics fixtures. The century-long genealogy test
also retained exact checkpoint continuation. The complete ignored GPU suite was not
run. Clippy across all targets, formatting, diff whitespace and the source-only
repository artifact check passed.
