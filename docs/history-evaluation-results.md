# History evaluation results — September 6, 2026

The initial phase comprised twenty paired diagnostic runs on the Quadro RTX 5000 with Max-Q Design: seeds `0,7,42,99,999`, terrain/ecology edge 64, one geological epoch, 100 social years, two founding densities. Both controls and candidates use the same current executable and the cohort accounting fix. Controls disable new commercial rules and regional droughts. Candidates in this first table use the initial 90% drought severity, not the final 50% default. No scenario manually declares wars, changes loyalty or deletes food in these runs.

Population ranges are final resident counts **per world**. Other event counts sum the five worlds. Cross-border deliveries and recoveries use annual observation semantics described in [the methodology](history-evaluation.md). Residuals are relative fractions, not percentages.

| Experiment | Population range | Cross-border deliveries | Food crises | Wars | Treaties | Secessions | Founding migrations | Observed recoveries | Max residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| control-five | 1839–1839 | 0 | 0 | 0 | 0 | 0 | 50 | 0 | 1.33e-05 |
| improved-five | 1498–1795 | 0 | 28 | 0 | 0 | 0 | 42 | 11 | 1.37e-05 |
| control-sixteen | 5885–5885 | 9273 | 0 | 0 | 197 | 0 | 160 | 0 | 1.32e-05 |
| improved-sixteen | 5258–5723 | 7285 | 96 | 25 | 169 | 8 | 154 | 58 | 1.33e-05 |

## What changed and what did not

- The quiet earlier world was not representative of all contact densities. With sixteen founders, the original rules already generate international trade and treaties. Five-founder worlds remain separated by the lake, with no road-based international exchanges.
- Regional drought forcing breaks the nearly identical population trajectories. Food reserves, relief and production constraints now produce shortages, demographic losses and recovery. Founding migrations fall in some runs as expansion requirements become harder to meet. These are still founding migrations, not an implemented refugee system.
- Existing political rules respond to material stress: four of the five dense candidate worlds fight wars, while seed 999 remains peaceful. Conquest can expose unpaid administration and secession. There is no imposed conflict quota.
- Commercial paths expand legal reach across intermediate towns, but lower raw cross-border delivery counts are possible because buyers prefer closer supply and shortages change demand. Shipment count alone does not establish an improvement in welfare or realism.
- No diagnostic candidate world abandoned an existing settlement in these twenty runs. Fewer final settlements primarily reflect reduced founding. Severe collapse is covered separately by the drought and long political fixtures; it should not be inferred from the final settlement count.
- All observed ledger residuals remain below the 0.1% tolerance. The full suite passes 44 tests, including GPU rainfall/growth suppression, conservative multi-hop cargo, closures, hostile transit and exact same-backend checkpoint continuation. Clippy and formatting checks pass.

The initial drought parameters are a strong game-history stress setting, not empirical climate statistics. Longer and more geologically mature worlds are still needed for calibration. Ports and lake transport are the clearest remaining constraint on interaction among the usual five central civilizations. Independent household wealth, refugees, environmental feedback outside managed plots and occupation reconstruction remain unimplemented.

## Artifacts

- [Five-founder control](../output/history-control-five.md) and [candidate with paired deltas](../output/history-improved-five.md).
- [Sixteen-founder control](../output/history-control-sixteen.md) and [candidate with paired deltas](../output/history-improved-sixteen.md).
- Matching `.json` files contain the annual series and archived rule settings.
- The five-founder candidate worlds are saved as `output/history-improved-five.SEED.world` for desktop inspection.

## Production-resolution check

An additional paired run used seed 42, terrain/ecology edge 256, five founders and the same one-epoch/100-year interval. Final population changed from 1839 to 1803; the initial 90%-severity candidate recorded 6 shortage site-years and 3 observed recoveries, with 15 occupied settlements. Both worlds remained isolated internationally. The maximum candidate ledger residual was 1.33e-05, or 0.00133%.

The control took 31.58 seconds and the candidate 33.21 seconds end to end; the candidate also saved its archive. This is a scale check, not evidence of a performance gain. It also shows why diagnostic-grid demographic outcomes should not be assumed at production resolution.

See the [production control](../output/history-control-production.md), [paired candidate](../output/history-improved-production.md), and `output/history-improved-production.42.world`. Across the diagnostic and production pairs, 22 century-long runs completed.

## Continuation of the existing world

Two further runs continue `output/civilization-governance.world` from year 300 to 400 on its existing 256-resolution, epoch-ten environment. The control preserves its archived catalog; the candidate explicitly loads the new bundled catalog. The original archive remains unchanged.

| Rules | Final residents | Active sites | New food crises | Founding migrations | Wars |
|---|---:|---:|---:|---:|---:|
| Control | 6663 | 47 | 0 | 9 | 0 |
| New rules | 3396 | 48 | 269 | 10 | 0 |

The candidate's demographic decline is much stronger here than in fresh one-epoch worlds. This is evidence that the 90% drought severity is a harsh stress setting for mature settlements, not a universally calibrated default. It should be tuned against longer histories and different environmental baselines before treating the population losses as realistic. Both continuations pass all stock ledgers, with maximum managed residuals below 0.004%. In total, 24 century-long runs completed, including these two continuations.

The updated copy is `output/civilization-evaluated.world`; its control is `output/civilization-evaluation-control.world`. Their `.history.json` files preserve the full event and state records.

## Default calibration on the mature world

The same year-300 archive was also tested at 75% and 50% drought severity. The final bundled default is 50%; the 90% experiment remains available through the report CLI.

| Severity | Final residents | Active sites | Food crises |
|---|---:|---:|---:|
| 0% (archived control) | 6663 | 47 | 0 |
| 50% (final default) | 4941 | 47 | 0 |
| 75% | 3942 | 49 | 355 |
| 90% (stress) | 3396 | 48 | 269 |

This choice reduces the demographic impact relative to the stress settings; it does not establish empirical realism or eliminate the need to sample longer histories. The final-default continuation is `output/civilization-mild.world`, with its full event log in `output/civilization-mild.history.json`. All four continuations passed their inventory ledgers.

## Final-default seed suite

The final 50%-severity default was rerun across both five-seed founding-density suites, with the original paired controls retained. The following counts sum each five-world suite; population is the range of final residents per world.

| Final default | Population range | Food crises | Wars | Treaties | Cross-border deliveries | Max residual |
|---|---:|---:|---:|---:|---:|---:|
| tuned-five | 1814–1839 | 2 | 0 | 0 | 0 | 1.33e-05 |
| tuned-sixteen | 5856–5885 | 3 | 0 | 183 | 7147 | 1.32e-05 |

The fresh worlds remain mostly stable under the final default; the mature-world comparison is where food stress is substantial. The report does not equate a higher war count with realism. Across the original controls, stress experiments, production pair, mature-world calibration and final-default suite, 36 recorded century-long comparisons completed. The final code passes 44 tests, clippy with warnings denied, and formatting checks. Desktop smoke runs loaded and rendered the year-400 saved histories.

Final-default detailed reports: [five founders](../output/history-tuned-five.md) and [sixteen founders](../output/history-tuned-sixteen.md). Launch the mature final-default sample with:

```sh
cargo run -- --load output/civilization-mild.world
```

## Inter-island shipping

The next implementation adds [surveyed lake shipping](shipping.md), with material-funded harbors, shared cargo reservations, closure policies and actual trade contacts feeding diplomacy. Construction consumes existing goods; wear, cargo and money remain in the conservation ledgers. Initial port surveys prefer settlements with available construction reserves, which matters when importing old worlds with exhausted founding towns.

The final-default paired century suites keep geography, founding counts and weather fixed:

| Suite | Sea deliveries | Cross-border deliveries before → after | Treaty records before → after | Max residual |
|---|---:|---:|---:|---:|
| Five seeds, five founders, 64² faces | 1567 | 0 → 1567 | 0 → 27 | 1.33e-5 |
| Five seeds, sixteen founders, 64² faces | 2198 | 7147 → 8859 | 183 → 149 | 1.32e-5 |
| Seed 42, five founders, 256² faces | 15 | 0 → 15 | 0 → 0 | 1.33e-5 |

Both terrain and ecology use the stated edge size. Populations remain unchanged in four of the five seeds at each diagnostic founding density; seed 99 loses approximately 6.5 residents with five founders and 10.1 with sixteen. No wars occur in these final-default fresh runs. Shipping is a commercial contact mechanism, not a naval invasion route.

Trade can shift between partners rather than increase every count. In the denser suite, seed 7 records 65 fewer cross-border deliveries even though sea trade becomes possible. The total treaty count also falls; this includes renewals, not merely unique relationships. Availability of new suppliers does not guarantee sustained contact with every old partner. At production resolution, all five ports commission but only 15 sea deliveries occur and no treaties form. The diagnostic-world rates are therefore not a universal calibration.

Detailed reports: [five founders](../output/history-shipping-five.md), [sixteen founders](../output/history-shipping-sixteen.md), and [production pair](../output/history-shipping-production.md). JSON retains annual observations, actual events and residuals. The production pair took approximately 31 seconds without shipping and 33 seconds with shipping, including generation and reporting; these are end-to-end observations on this machine, not isolated kernel benchmarks.

The full suite passed 47 tests including hardware-GPU checks. Shipping-specific validation covers shared capacity, actual shipments, closures and restored service, illegal paths and assets, hostile transit after landing, conservation and exact checkpoint continuation across dispatch batch sizes. Formatting and clippy with warnings denied also pass. Desktop smoke testing now handles history archives founded before geological epoch two.

### Port placement on an existing world

The year-400 `civilization-mild.world` archive exposed a placement defect in the first shipping prototype: automatically hosting each port at the island's first settlement ignored later towns with construction supplies. Only one port commissioned and there were no sea deliveries during the following century. The corrected survey ranks available construction reserves and then requires reachable coastal access. It commissions ports at existing settlements 28 and 26 immediately by transferring their actual goods.

| Year 400 → 500 continuation | Final residents | Ready ports | Sea deliveries | New treaty records |
|---|---:|---:|---:|---:|
| First-settlement prototype | 3365 | 1 | 0 | 0 |
| Construction-reserve selection | 3361 | 2 | 344 | 8 |

This is a port-placement comparison, not a shipping-on/off attribution of the century's population decline. The other three islands remain short of tools and cannot commission ports. Even the working ports lose capacity when replacement tools become scarce. The final managed conservation residuals remain below 4.65e-5 relative; no construction inventory is supplied artificially.

The earlier state is retained in `output/civilization-shipping-first-host.history.json`; the final state and events are in `output/civilization-shipping.history.json`. The final archive and desktop capture are `output/civilization-shipping.world` and `output/civilization-shipping.png`. Interactive UI checks additionally exercised the lane checkbox and observed the resulting `sea_lane_policy` event; captures are `output/shipping-inspector.png` and `output/shipping-inspector-closed.png`. That UI intervention was not saved into the example worlds.

```sh
cargo run --release -- --load output/civilization-shipping.world
```
