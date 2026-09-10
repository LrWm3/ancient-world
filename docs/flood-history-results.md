# Flood history validation

Follow-up: the [broader seed audit](flood-seed-audit.md) tests eight paired seeds and identifies persistent-inundation and indefinitely delayed-cargo issues beyond these initial checks.

Measured on the Quadro RTX 5000 with Max-Q Design, Vulkan. Final reports: [century suite](../output/flood-complete.md), [JSON](../output/flood-complete.json), and [storm stress suite](../output/flood-stress.md). Runs use seeds 0, 7 and 42, 64 cells per face for terrain and ecology, one geological epoch, five founding civilizations, and all history systems through specimen discoveries enabled.

## Century outcomes

| Seed | Flood episodes | Completed recoveries | Flooded town-months | Food lost, kg | Crops lost, kg | Road closures | Port closures | Cargo delayed / recovered |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 54 | 51 | 276 | 1,720,645 | 569,301 | 34 | 38 | 19 / 18 |
| 7 | 41 | 39 | 68 | 208,250 | 89,210 | 60 | 0 | 3 / 3 |
| 42 | 5 | 4 | 8 | 5,993 | 15,056 | 4 | 0 | 6 / 6 |

Losses accumulate across a century and are not simultaneous stock deficits. Another flood can interrupt cleanup; ongoing recovery and cargo waiting at the endpoint are permitted. Seed 0 retains an inundated town at year 100. Relative to the previous living-history century report, final populations change by −63.9, +14.2 and +1.9 respectively. The paired comparison includes this whole increment, so it does not isolate one mechanism's causal effect.

All annual ecological and history budget checks passed. Maximum century ecological relative residuals were 1.70e−5 for C/N/P and 5.06e−5 for water. These are floating-point tolerances, not exact conservation claims. Managed food, population, goods, money and nutrient checks also passed.

## Refinement and stress comparison

The initial whole-cell flooding model produced no town floods in these three centuries. Water spread over planetary cells concealed river exposure. The final bounded 5% corridor model resolves local depth while retaining the same finite water inventory; no disaster quota or seed-specific outcome was added.

Increasing storm probability from 0.04 to 0.20 over matching first-30-year runs raises seed 0's flooded town-months from 45 to 184, stored-food losses from 422,237 to 715,046 kg, and crop losses from 159,500 to 464,389 kg. Seeds 7 and 42 have no town floods in either first-30-year interval. Town location and river exposure matter; extra rainfall does not force disasters everywhere. Stress-run annual budgets passed.

## Fixtures and desktop

All 58 tests passed on the final implementation, including ignored GPU fixtures. The final all-target run was interrupted after 42 passing tests; the remaining five integration targets were rerun successfully (16 tests). Formatting checks and all-target Clippy with warnings denied passed.

The GPU overflow fixture checks finite routed water, proportional C/N/P deposition, storage caps and drainage back into runoff. The sparse-history fixture verifies food/nutrient transfers, recovery, manual closure preservation, archive continuation and cargo delivery exactly once after reopening. Existing coupled-history fixtures cover coarse ecology, water conservation, deterministic batching and checkpoint continuation.

An older `living-continued.world` archive resumed from year 105 to 106 with valid budgets and was saved as `output/flood-continued.world`. The desktop loaded final seed 0, stepped one complete month and displayed Lorwick's active flood, cleanup period, losses and food crisis. [Desktop capture](../output/flood-controls.png).

A final seed-42 run at 256 cells per face completed generation plus one living-history year in approximately 13.6 seconds and passed budgets. This is a smoke check, not an isolated performance benchmark or a 512/1024-resolution calibration. Full-resolution long histories and infrastructure adaptation remain follow-up work.
