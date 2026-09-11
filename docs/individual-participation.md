# Individual participation

The intended direction is a complete resident population with monthly activities.
This first runnable increment implements shared participation for existing named
people. It does **not** convert aggregate population into individual demography.
Settlement cohorts still supply production, consumption, births, deaths, military
manpower and expedition crew counts. Named participants are not added to those
stocks. No new population or economic inventory is created by enabling this layer.

## Identity and presence

`History.participation` stores numeric person references, known household membership,
opening-month presence, available capacity and cumulative completed cultural/research
work. Existing heads and recorded kin supply membership. A living leader without a
household can be associated with the civilization's first active settlement. Unknown
residence stays unknown. Children have no work capacity. Household relocation removes
its known members from local eligibility; death and relocation are rechecked live.

These are the existing ownership households, not newly reconstructed domestic families.
Expedition crew names remain separate identities. The roster therefore cannot be used
as a census or proof that all travelers have individual identities. The explorer labels
it as a known-person roster and displays current presence separately from its opening
observation.

## Participation contract

The monthly reservation opening captures personal capacity once. The initial ceiling
is `0.8 * (1 - 0.5 * settlement_illness)`, with the same illness clamp as the existing
workforce calculation. It is an abstract effective-work allowance for converted
services, not a simulation of waking hours or a complete personal employment schedule.

Both gates apply: available settlement service labor and available named participants.
Every grant is deducted once from the existing settlement service pool. Personal shares
partition that grant; they do not add extra workers. Ordinary production still uses
aggregate labor after service reservations.

- Cultural bundles pin their actor, successor and institutional teacher. Their named
  team divides the grant equally. This conservatively reserves possible supporting
  participants for the bundle, rather than pretending each action has separate time slots.
- Research chooses at most four known local adults, ordered by curiosity plus bounded
  accumulated research experience, then by stable person ID. The same people may also
  be cultural actors; the later reservation sees only their remaining capacity.
- Research source workshops remain workshops. Their IDs are not treated as person IDs
  or as physical visiting teachers.
- Execution checks current presence; missing participants invalidate the team's grant.
  Partial work due to materials or other existing constraints remains possible.
- Settlement records actual completed work and each person's proportional contribution.
  Only completed research builds the experience used in subsequent team selection.
- Unused commitments expire. They cannot be reassigned after execution or credited as
  productive experience. The layer does not introduce wages or change ownership.

The existing phase order is retained: research reservations precede cultural work;
execution remains in its existing subsystem stages. This is an explicit priority,
not a simultaneous optimal assignment solver. No GPU readback or population update is
introduced by participation.

## Persistence and controls

New histories enable participation. Older archives without the field retain legacy
aggregate service work. `History::set_individual_participation(bool)` switches at a
completed personal-work boundary; enabling it establishes known membership on the next
reservation. It does not invent biographies or convert population. Disabling clears
personal plan links while preserving existing economic and cultural records.

The registry, current commitments and accumulated contributions are serialized.
`History::participation_report()` and the service work report expose them; the culture
explorer includes an Individual participation panel. Historical IDs are unchanged.

## Next population-authority milestone

Complete residents require domestic households distinct from ownership accounts,
whole-person initialization with explicit rounding adjustments, individual food needs,
and adapters for every population writer. Founding, birth, death, relocation, military
recruitment and expedition casualties must all reference the same people before the GPU
cohort demographic update is disabled. Production can continue to receive aggregated
labor from the new assignments. This implementation deliberately does not maintain a
second purported full population alongside the current one.

## Verification

Verification used Rust 1.89.0 and the Quadro RTX 5000 with Max-Q Design, Vulkan
backend. Generated logs and reports remain under ignored `output/`.

The first long-run diagnostic exposed an incorrect inference in the new ledger:
several existing cultural actions retain their initial grant array after recording
completed work. Contributions now use explicit per-site completion records from
maintenance, elections, petitions, heritage work, recoveries, pilgrimages and personal
choices. Quarterly cultural receipts also include election work, which previously
occurred before their measurement window. This fixes reporting; it does not invent
additional labor. Personal presence is checked before elections spend their grant.

The calibration runner now asserts that lifetime individual contributions match the
existing cumulative cultural and research work counters. It distinguishes a passing
no-contention history from a controlled fixture in which people are actually scarce.


### Results

- Ordinary library tests: **76 passed**, 69 hardware tests ignored by that command.
- Participation tests: **four passed**, including one GPU workshop fixture (the other
  three overlap the ordinary library suite).
- GPU culture tests: **13 passed**; research exchange fixture: **one passed**.
- History environment tests: **three passed**, including full-state batch/checkpoint
  comparisons and compact/full environmental readbacks.
- Clippy across all targets with warnings denied, formatting and artifact policy passed.

That is **94 distinct tests** across the selected suites, not a claim that every
hardware test in the repository ran. The fixtures verify shared personal capacity,
partitioned team effort, no repeated settlement, preserved archived commitments,
death/relocation invalidation, and real workshop output when people are available.
Precommitting the available people blocks sample processing while leaving the samples
intact. Cultural teaching has an explicit completion-to-personal-ledger assertion.

Three matched 100-year runs used seeds 17, 81 and 256, terrain edge 32, ecology edge
16, one geological epoch, 16 initial civilizations and default crop yield 0.5.
The runner enables society, politics, governance, offices, shipping, expeditions,
discoveries and living history. One arm enables participation; the other uses
`--legacy-participation`.

| Seed | Final population | Active institutions | Known person records | Eligible named adults at last opening | Completed cultural worker-months |
| --- | ---: | ---: | ---: | ---: | ---: |
| 17 | 2,760 | 32 | 3,165 | 2,044 | 2,354.175 |
| 81 | 3,165 | 34 | 3,263 | 2,108 | 2,464.125 |
| 256 | 2,418 | 32 | 3,242 | 2,065 | 2,274.525 |

All reported non-participation observations, event-type totals and final economic
residuals matched between arms. Personal cultural totals matched cumulative subsystem
work within 0.000007 worker-months at the endpoints; the runner checks agreement at
every quarterly observation. Known records include deceased people and are not a
second population count.

**None of these ordinary histories performed research work.** Thus they verify
non-contention behavior and accounting, not balance under substantial cultural/research
competition. That interaction is established by the controlled fixture, not by claiming
that matching seed trajectories prove it. Larger populations, full resident demography,
and sustained research-heavy histories remain uncalibrated. Concurrent run timings are
not performance benchmarks.

### Reproduction

Prefix Cargo commands with `CARGO_INCREMENTAL=0 mise exec rust@1.89.0 --` in this
workspace:

```sh
cargo test --lib
cargo test --lib participation -- --include-ignored
cargo test --lib culture:: -- --ignored
cargo test --lib discoveries:: -- --ignored
cargo test --test history_environment -- --ignored
cargo clippy --all-targets -- -D warnings
cargo build --example cultural_work_calibrate
```

Then run:

```sh
target/debug/examples/cultural_work_calibrate --seeds 17,81,256 --years 100 --output output/individual-participation-enabled.json
target/debug/examples/cultural_work_calibrate --legacy-participation --seeds 17,81,256 --years 100 --output output/individual-participation-legacy.json
python3 scripts/summarize_cultural_work.py output/individual-participation-enabled.json output/individual-participation-legacy.json
```
