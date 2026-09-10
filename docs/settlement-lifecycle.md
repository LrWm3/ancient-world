# Settlement decline and truthful notices

Population scale now has persistent labels independent of government, founding identity and economic specialization. These are configurable-in-code game-scale categories, not universal historical definitions of towns.

| Label | Initial population range | Boundary hysteresis |
|---|---:|---|
| Town | 100+ | Decline below 90 |
| Village | 30–99 | Decline below 27; growth to 110 |
| Hamlet | 5–29 | Decline below 4.5; growth to 33 |
| Remnant community | Below 5 | Growth to 5.5 |

A new classification requires **24 consecutive months** beyond the current category's hysteresis boundary. The pending category must remain the same. A severe decline can skip categories. Founding uses the actual starting population; older archives default to a town label and acquire classifications through the same sustained-change rule. Status changes produce `settlement_reclassified` events and appear in the settlement list and evaluator samples.

Small communities are not automatically abandoned because they cannot maintain a town-sized population. A viable hamlet may persist indefinitely. Abandonment requires **12 consecutive months below one person-equivalent**; recovery to one resets that counter. GPU demographic/production updates continue during this interval instead of freezing the fractional population at the first threshold crossing.

At abandonment, the remaining sub-one statistical cohort is resolved into the cumulative mortality ledger and age cohorts become zero. This is an explicit demographic discretization rule, not an inferred evacuation or a claim that a specific named resident died. No food, money, materials, or nutrient inventories are deleted. Old archives' already-abandoned fractional cohorts are resolved in the same ledger before further production. Actual returning residents can reoccupy ruins; `settlement_reoccupied` records the change without adding people or goods.

This increment does not implement organized relocation or a new subsistence economy for remnants. Population labels do not grant yield bonuses or change economic rules. Named historical records and property claims remain available; they do not themselves establish an active resident community.

## Event corrections

- Managed farming reports positive changes in the six cumulative crop-harvest inventories, with crop names and actual kilograms. It no longer reports total calorie-equivalent food production as grain harvested on a single legacy calendar date. Multiple crops harvested together share a notice. Reading unchanged totals again produces no duplicate notice.
- Before the first resumed month, missing harvest cursors initialize from existing totals, avoiding retroactive notices for old harvests. Cursors and lifecycle counters are serialized at normal history boundaries.
- Legacy grain farming retains its grain calendar notice. Both paths skip abandoned sites, as do food-crisis and weather notices.
- Flood and transport conditions continue evolving at ruins, but no longer produce resident cleanup or inspection notices there. Road and sea-lane notices use a surviving endpoint when available.
- Abandoned household records cannot generate new heirs, succession, marriages or children. Tax collection from inhabited towns also stops at ruins.
- Teaching and conversion require living communities at both ends of contact. Ruins cannot supply the second constituency needed for a schism.
- Religious interpretation and syncretism require living affiliated interpreters. They can occur in a diaspora settlement even when the original sacred site is abandoned. Patron departures and legitimate object-loss events remain possible after local collapse.

## Validation

The controlled GPU-backed fixture exercises delayed decline, recovery, hysteresis, grace-period reset, fractional mortality accounting, inventory retention, reoccupation, actual harvested mass, duplicate-notice prevention, surviving diaspora interpretation, post-collapse social inactivity, and archive compatibility.

The full test suite passed **101 tests, zero failures**, including hardware GPU tests. After the flood-notice refinement, its expanded conservation/recovery fixture passed again; Clippy passed with warnings denied. Two 24-month exact continuation checks (a mature harsh world and an older collapsed archive) matched history, terrain and ecology across checkpoint/reload and different execution batches.

The final century reruns use seeds **17 and 81**, edge 64 terrain/ecology, sixteen initial communities, living environments and discoveries. No crop, weather or scarcity defaults changed. Exact commands, logs, archives and annual samples are in `output/settlement-lifecycle-final`; the first iteration is retained separately in `output/settlement-lifecycle`.

| Scenario | Final population, seeds 17 / 81 | Abandonments, seeds 17 / 81 |
|---|---:|---:|
| Default yield 0.5 | 5,236 / 5,705 | 0 / 0 |
| Harsh yield 0.3 | 1,567 / 1,650 | 6 / 4 |
| Collapse yield 0.1 | 0 / 0 | 16 / 16 |

The harsh worlds each recorded 27 size changes. Seed 17 ended with 5 towns, 7 villages, 2 hamlets, 1 remnant and 6 ruins; seed 81 ended with 6 towns, 1 village, 3 hamlets, 3 remnants and 4 ruins. No spontaneous reoccupation occurred in these samples; that path is covered by the controlled returning-resident fixture.

`python3 scripts/settlement_audit.py WORLD... --output REPORT.json` reconstructs abandonment/reoccupation from events and fails on local activity notices at ruined sites or nonzero resident cohorts in ruins. It intentionally treats historical archives made before this fix as audit failures when they contain the old ghost events.

All six final archives passed the activity audit: **zero local activity notices after abandonment**, and zero remaining resident cohorts in ruins. Complete collapse occurred at years 31.33 and 29.33; neither world emitted any further events through year 100. The largest reported relative conservation residual was **2.82327e-05**. These small-grid, two-seed comparisons are regression evidence, not a universal historical calibration.
