# Military participation verification

## Scope

New campaigns share people with local cultural/research work and expeditions.
Recruitment identifies existing cohort residents where necessary, withdraws an equal
whole-person count from the resident population, and records active service.
Casualties and return change those identities and the existing population ledger
together. Careers persist after an army is removed.

This is another adapter toward individual population authority. It does not provide
a complete census, individual defenders, soldier wages, domestic household needs,
or individual farm/industry employment. Legacy armies remain aggregate. The 15%
maximum experience advantage and fractional expected-loss carry are toy-game rules.

## Boundary checks

The GPU-backed named-campaign fixture checks:

- One duty per live soldier; a later recruitment cannot take the same people.
- Military absence removes local personal-work capacity.
- Simultaneous expedition/military duties and roster/count corruption are rejected.
- Legacy archives missing military fields load without invented campaign biographies.
- Expected losses of 0.75 followed by 0.5 cause exactly one death and retain 0.25.
  The death changes the person and traveling population, not local death credit.
- Serialized continuation chooses the same casualty and retains identical state.
- Returning survivors keep identity and household; a newly 60-year-old returns to
  the elder cohort. Population remains accounted for.
- Recorded service increases preparedness, capped at 15%.
- Total loss closes the army: no ghost carriers return its stores. Provisions
  become declared spoilage and equipment enters the existing lost-material reserve.
  Population, food and material accounting remain balanced. This reserve is an
  accounting sink, not a newly generated navigable battlefield salvage site.

Existing provisioned-raid and occupation fixtures check the integrated monthly
pipeline, finite provisioning, political control, withdrawal, budget residuals and
checkpoint/batch equivalence. The research fixture checks competing named service
requests and absence; the expedition suite exercises the shared recruitment path.

## Failure found and corrected

The first integrated raid run failed closing validation: a household head died in
battle after the previous succession pass. Household succession now follows army
outcomes within society response. A dead current owner is resolved even for an
abandoned home site; candidate heirs are rechecked for death and active service.
Inheritance consumes no additional population or mortality credit.

The occupation test also assumed withdrawal must be the final event of a month.
It now identifies the occupation-ending event explicitly, allowing the consequent
inheritance event to follow. Withdrawal behavior and accounting assertions remain.

## Reproduction

Rust 1.89.0, development/test profile, Vulkan on Quadro RTX 5000 with Max-Q Design.
Generated outputs belong under ignored output/.

    CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib
    CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib named_campaign_presence -- --ignored --test-threads=1
    CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib provisioned_raids_conserve -- --ignored --test-threads=1
    CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib finite_occupation_withdraws -- --ignored --test-threads=1
    CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib personal_absence_and_shared_work -- --ignored --test-threads=1
    CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test expeditions -- --ignored --test-threads=1
    CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test politics -- --ignored --test-threads=1
    CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test culture -- --ignored --test-threads=1
    CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo run --example cultural_work_calibrate -- --seeds 17,81,256 --years 100 --output output/individual-military-final.json
    CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo clippy --all-targets -- -D warnings
    mise exec rust@1.89.0 -- cargo fmt --check
    python3 scripts/check_repository_artifacts.py

The ensemble uses one geological epoch, terrain 32 / ecology 16 cells per face,
16 initial civilizations and living history. The runner checks personal work totals
against cultural/research completion every quarter. Monthly validation checks army
membership and existing conservation ledgers. Larger production resolutions and
cross-GPU reproducibility are not established by this run.

## Final results

All 100 selected tests passed: 76 ordinary library tests, four targeted GPU library
fixtures, six expedition tests, five politics tests and nine culture tests.
This is not the entire repository GPU suite. Clippy with warnings denied, formatting
and the repository artifact check also passed.

All three 100-year worlds completed monthly validation and quarterly personal-work
reconciliation without errors. Endpoint resident counts remain fractional because
resident demography is still cohort-based; active soldiers are counted separately.

| Seed | Residents | Active towns | Wars | Distinct soldiers | Military deaths | Soldiers still away | Service months | Max absolute normalized economic residual |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 17 | 2707.22 | 16 | 7 | 79 | 11 | 0 | 1354 | 6.642021e-06 |
| 81 | 3214.19 | 18 | 11 | 156 | 12 | 16 | 2546 | 2.231842e-05 |
| 256 | 2606.55 | 16 | 3 | 35 | 3 | 0 | 470 | 5.874388e-06 |

Soldiers still away in seed 81 have valid active campaigns and household duties;
these are not missing returns. The worlds also completed 3, 22 and 0 ancient-continent
voyages respectively, exercising military service alongside expedition recruitment.
No total army loss occurred in these three worlds; that edge case is established
by the controlled fixture rather than claimed as a natural ensemble observation.

These runs support integration and bounded behavior at diagnostic resolution. They
do not establish realistic casualty rates, a complete individual population model,
or portability across GPU backends. No balance parameters were fitted to require
a particular number of wars or a particular winner.
