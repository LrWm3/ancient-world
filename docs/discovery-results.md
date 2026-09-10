# Expedition specimen evaluation

The initial and refined suites use seeds 0, 7, 42, 99 and 999, five founding civilizations, 64 cells per cube-face edge for terrain and ecology, one geological epoch and 100 social years. The GPU is a Quadro RTX 5000 with Max-Q Design. Additional runs cover stronger regional drought (severity 0.9; seeds 0/7/42), a 256-resolution seed-42 world, and a 20-year continuation of an existing observation-only expedition archive.

## Refinement from observed demand

The initial workshops processed all returned resin, even when nobody needed treatment. None of the five receiving port communities had treatment demand. This was a production-policy defect, not evidence that expeditions should forcibly introduce disease.

Workshops now preserve raw samples and process only enough for a small emergency reserve, with a higher target during illness. Research and crafting reserve only the labor they can provisionally use. A separate workshop switch pauses work without closing the settlement market. Local study and processing ledgers validate against delivered inventories and the global ledger.

| Five-seed aggregate | Initial | Refined |
|---|---:|---:|
| Remedy manufactured (kg) | 199.96 | 12.87 |
| Remedy spoiled (kg) | 191.02 | 11.51 |
| Remedy consumed (kg) | 0.00 | 0.00 |
| Farm phosphorus applied (kg) | 14.62 | 14.62 |
| Actual assay/processing labor (worker-months) | 1240.68 | 492.34 |

Refinement reduced unnecessary remedy manufacturing by 93.6%, preserving the same phosphorus delivery and retaining most resin for future demand. The sparse suite showed no population change at the report's one-decimal precision. This is useful infrastructure with finite costs, not a demonstrated population-growth bonus.

## Final outcomes

| Suite / seed | Resin collected kg | Mineral collected kg | Remedy made kg | Remedy used kg | Farm P kg | Exhausted coastal sources |
|---|---:|---:|---:|---:|---:|---:|
| final / 0 | 88.60 | 30.65 | 2.64 | 0.00 | 2.33 | 2 |
| final / 7 | 75.11 | 68.39 | 3.25 | 0.00 | 5.11 | 0 |
| final / 42 | 112.76 | 8.33 | 3.23 | 0.00 | 0.31 | 2 |
| final / 99 | 81.21 | 8.40 | 1.92 | 0.00 | 0.54 | 2 |
| final / 999 | 64.74 | 91.71 | 1.83 | 0.00 | 6.33 | 1 |
| stress / 0 | 96.70 | 36.27 | 2.45 | 0.00 | 2.78 | 2 |
| stress / 7 | 75.11 | 68.39 | 3.25 | 0.00 | 5.11 | 0 |
| stress / 42 | 112.76 | 8.33 | 3.23 | 0.00 | 0.31 | 2 |
| production / 42 | 145.06 | 16.38 | 4.72 | 0.00 | 0.88 | 5 |

All nine final evaluation histories passed annual validation. The largest recorded relative civilization residual was 1.35e-05; the largest additional specimen/product residual was 3.82e-15. Phosphorus output is transferred into soil, not an automatic yield multiplier. Other limitations, particularly nitrogen and water, still apply.

Neither ordinary nor drought-stress receiving workshops naturally consumed remedies. These ports retained low disease burden; medicines are not yet transported to other settlements. The controlled illness fixture does demonstrate that illness raises production and consumes finite doses. Distribution through markets is therefore a more useful next connection than increasing background disease simply to force demand.

The existing seed-seven expedition archive upgraded explicitly at year 100, continued to year 120 and saved with valid budgets. Earlier voyages received no retroactive specimens.

## Evidence and reproduction

- [Initial report](../output/discoveries-initial.md) and [JSON](../output/discoveries-initial.json). These are prototype measurements; their temporary development archives precede the final specimen subarchive format.
- [Refined paired report](../output/discoveries-final.md) and [JSON](../output/discoveries-final.json).
- [Drought-stress report](../output/discoveries-stress.md) and [256-resolution report](../output/discoveries-production.md).
- [Upgraded history JSON](../output/discoveries-continued.json).

The full 53-test regression suite passed after initial integration. Following the refinement, the expedition, economy and specimen suites passed again, including a new demand-response fixture: 54 distinct tests in total. The final four specimen fixtures cover rescue/loss transfer, source depletion and fertilizer, workshop pause and checkpoint continuation, and healthy-to-ill demand response. Clippy passed with warnings denied.

```sh
mise exec rust@1.89.0 -- cargo run --release -- --load output/discoveries-final.7.world
```

See [the design and limits](discoveries.md). The environmental source remains an explicit accessible-coast baseline outside managed plots; it does not model whole-continent depletion or specimen domestication.

Desktop validation loaded the final seed-seven archive, exported a [smoke screenshot](../output/discoveries-final.7.png), opened the specimen panel and paused Coran's workshop. The checkbox and historical policy event updated correctly; [workshop control screenshot](../output/discovery-controls.png). The interactive test did not overwrite the evaluation archive.
