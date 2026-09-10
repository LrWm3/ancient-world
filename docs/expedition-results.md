# Expedition v1 evaluation

All runs use the Quadro RTX 5000 with Max-Q Design, Vulkan, one geological epoch and 100 social years. The main paired suite uses seeds 0, 7, 42, 99 and 999, 64 cells per cube-face edge for terrain and ecology, and five founding civilizations. These are diagnostic histories, with a separate 256-resolution check. No success/disaster quotas are imposed.

## Refinement from measured outcomes

The first suite launched 254 vessels, returned 243, and recorded 63 crew deaths. All 33 rescue arrivals found empty camps: the stranding model left too much repair equipment, and searches launched immediately from omniscient information. No party was rescued or wholly lost.

Stranding now consumes actual timber and tools. Automatic searches wait until the expected round trip and field interval are overdue by two months; they may still reach empty camps when survivors repair their vessel or start home. Seasons use destination hemisphere. Research objectives respond to the home settlement's disease pressure and ore depletion, and returned findings or total losses have modest political effects.

The final suite launched 236 vessels: 218 returned, eight stranded parties transferred to rescue crews, one party was lost, and nine voyages were still active at the century boundary. There were 91 crew deaths and 14 empty-camp searches. Outcomes now include self-repair, rescue, delayed recovery and failure without forcing any of them.

| Seed | Charters | Returned vessels | Rescued parties | Lost parties | Crew deaths | Crew away | Confirmed knowledge |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | 53 | 49 | 1 | 0 | 21 | 24 | 112.0 |
| 7 | 37 | 32 | 2 | 1 | 22 | 16 | 84.6 |
| 42 | 55 | 51 | 2 | 0 | 16 | 16 | 115.2 |
| 99 | 40 | 38 | 1 | 0 | 11 | 8 | 90.4 |
| 999 | 51 | 48 | 2 | 0 | 21 | 8 | 143.3 |

Returned vessels include rescue missions; rescued original parties are a separate terminal outcome and do not duplicate population. Knowledge is an abstract research score, not an inventory of usable medicines or minerals.

Compared with matched shipping-only worlds, resident populations finished 49–94 people lower. Between 8 and 24 crew were away per world; remaining differences include deaths, withdrawn labor and subsequent demographic/economic effects. These deltas are not a direct death count. Commerce and treaties also changed as charters competed for supplies. The largest recorded relative conservation residual in the final sparse suite was 1.31e-5. End-to-end runs took 8.2–8.6 seconds each, including generation, annual validation and archive output; these are not isolated expedition-kernel timings.

Raw reports and inspectable worlds:

- [Initial suite](../output/expeditions-initial.md) and [JSON](../output/expeditions-initial.json).
- [Final paired suite](../output/expeditions-final.md) and [JSON](../output/expeditions-final.json).
- Open seed seven with `cargo run --release -- --load output/expeditions-final.7.world`.

## Validation

All 50 tests passed with hardware tests enabled and sequential GPU execution. This includes the three new expedition fixtures for exact checkpoint continuation, rescue/recall, and starvation conservation, plus the existing terrain, ecology, markets, shipping, politics and society regressions. Clippy passed with warnings denied.

The v1 model keeps outer camps temporary. Crews have names and descriptive roles but are not yet linked to genealogy. Destinations are one coast per harbor; inland exploration, independently specialized crew skills, specimen manufacturing, ecological invasions and permanent colonies are deferred. The normal field risk is deliberately restrained rather than calibrated to a particular fictional canon.

## Density, resolution and hazard checks

| Suite | Worlds | Charters | Returned | Rescued parties | Lost parties | Crew deaths | Max relative residual |
|---|---:|---:|---:|---:|---:|---:|---:|
| [16 founders, five seeds](../output/expeditions-dense.md) | 5 | 246 | 227 | 8 | 5 | 120 | 1.33e-05 |
| [256 resolution, seed 42](../output/expeditions-production.md) | 1 | 90 | 86 | 0 | 1 | 19 | 1.30e-05 |
| [Hazard ×3, seeds 0/7/42](../output/expeditions-stress.md) | 3 | 150 | 130 | 4 | 12 | 175 | 1.34e-05 |

All fourteen final evaluation histories completed without a validation or conservation failure. The three hazard-stress worlds use identical seeds and other settings to their sparse counterparts. More dangerous field work increases casualties and losses; it does not bypass provisions or create rescue supplies.

Desktop validation loaded the saved seed-seven world under Xvfb, exported [a smoke-test screenshot](../output/expeditions-final.7.png), opened the expedition roster and recalled an outward voyage. The panel changed it to Homeward and reported that travel home was still required; [control screenshot](../output/expedition-controls.png). The interactive check was not saved over the evaluation archive.
