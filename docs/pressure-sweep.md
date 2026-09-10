# Town survival under pressure

Historical baseline: the lifecycle and post-collapse event fixes that followed this sweep are described in [settlement-lifecycle.md](settlement-lifecycle.md). Results below retain the original rules for comparison.

**Yes: the existing simulation can produce selective town collapse and complete demographic wipeouts without changing the abandonment rule.** Crop yield is the clearest difficulty control; land limits mainly change sustainable town size. Extreme tests also exposed incomplete post-collapse social cleanup and frontier-route validation failures.

This experiment changes scenario configuration, not production, mortality, migration, or abandonment rules. The baseline retains specialized workshops, cost-aware export contracts, discoveries, shipping, living environments, and scarce-island defaults. Each world starts with 16 communities (1,920 population equivalents) on diagnostic terrain/ecology edge 64. Seeds are 17, 81 and 256.

## What counts as collapse

The existing monthly history loop marks a town abandoned when population falls below **one person-equivalent**. Its event text says “one household,” but the numeric threshold is one, not a household-sized group. Abandoned places remain as ruins. Sub-one population residues stop demographic production updates; adding them across ruins does not represent an active surviving community.

Reports therefore distinguish active population, exact monthly abandonment events, and annual population declines below half/quarter of each initial town's founding population. Daughter towns count in total abandonments but are separated from initial-town failures. Shortage and recovery counts are annual observations, not complete monthly incident counts. Low population or a food shortage is not itself treated as abandonment.

## Reproducible setup

`scripts/pressure_sweep.py` schedules bounded concurrent evaluator runs, retains each exact command and log, and emits JSON/Markdown summaries. `--summarize-only` rebuilds summaries without running simulations. Failed world generation or validation is explicitly recorded as failure, never classified as survival.

The evaluator now accepts `--drought-probability` and `--drought-regime-months` alongside existing yield, plot, phosphorus, storm and drought-severity controls. It records initial town populations, annual town-level population/shortage/death/reserve observations, and abandonment events. Simulation defaults remain unchanged.

| Dimension | Baseline | Swept settings |
|---|---:|---|
| Crop yield scale | 0.5 | 0.4, 0.35, 0.3, 0.1 |
| Farm plot limit | 160 ha | 80, 20 ha |
| Island phosphorus scale | 0.25 | 0.05; 0.1 and 0.01 in combinations |
| Drought | severity 0.5, probability 0.2, 48-month regimes | severity 0.9, probability 0.6, 96-month regimes |
| Storm probability | 0.04 | 0.25 |
| Combined hard | — | yield 0.3, 80 ha, phosphorus 0.1, drought severity 0.75/probability 0.4 |
| Combined extreme | — | yield 0.1, 20 ha, phosphorus 0.01, drought severity 0.95/probability 1/120-month regimes |
| Severe survival test | — | combined extreme with baseline phosphorus, allowing habitable founding sites |

Yield 0.3 is a 40% reduction from the current 0.5 default, not a 70% reduction from current yields. These are game-model settings, not scientifically calibrated probabilities of societal collapse. Changing ecology at creation can change which sites qualify for founding; whole-history comparisons also include subsequent trade, migration and warfare feedback.

## Completed results

The sweep covered 13 configurations including baseline, with **35 completed century-long seed histories**. Eleven configurations completed all three seeds; the severe-survival case completed two before its third seed failed, and the most nutrient-poor combination failed during founding. An additional 200-year attempt failed as described below. Every completed history checked finite state and accounting; maximum observed relative accounting residual was 3.08e-5.

All abandonment events in the completed histories below concerned initial towns, so each per-seed count is out of the same 16 founding communities. Active population excludes the frozen sub-one residues in ruined towns. Surviving communities can found daughter towns.

| Setting | Abandoned towns: seeds 17 / 81 / 256 | Active population at year 100: seeds 17 / 81 / 256 |
|---|---|---|
| Baseline | 0 / 0 / 0 | 5236 / 5705 / 5597 |
| Yield 0.4 | 1 / 0 / 0 | 4042 / 4262 / 5415 |
| Yield 0.35 | 1 / 0 / 0 | 2661 / 2654 / 4746 |
| Yield 0.3 | 6 / 4 / 0 | 1553 / 1639 / 3305 |
| Yield 0.1 | 16 / 16 / 16 | 0 / 0 / 0 |
| Plot 80 ha | 0 / 0 / 0 | 3580 / 3628 / 3969 |
| Plot 20 ha | 0 / 0 / 0 | 848 / 886 / 971 |
| Phosphorus 0.05 | 0 / 0 / 0 | 5360 / 5685 / 5628 |
| Severe drought bundle | 11 / 12 / 15 | 303 / 7 / 2 |
| Storm probability 0.25 | 2 / 1 / 0 | 4875 / 5287 / 5621 |
| Combined hard | 6 / 6 / 5 | 1108 / 331 / 406 |
| Severe survival | 16 / 16 / incomplete | 0 / 0 / incomplete |

At yield 0.3, first abandonment occurred in years 50.5 and 63.5 in seeds 17 and 81. Seed 256 lost no towns. At yield 0.1, all communities were gone by years 30.4, 28.4 and 31.25 respectively. These are observed outcomes in three particular worlds, not estimated universal failure probabilities.

Population decline is broader than outright collapse. At yield 0.35, 6/6/1 initial towns fell below half their starting population even though only one was abandoned. With 20-ha plots, 11/10/11 initial towns fell below half, but none disappeared: total population was already close to its year-100 level at year 50.

This matches the production model's distinction between land and labor constraints. A land-capped town can shrink until its acreage supports its residents. Low per-hectare yields can leave food production inadequate even when land is available, because cultivation also requires a finite workforce. Trade, relief, crop diversity, weather and warfare affect which towns cross the failure threshold.

Phosphorus reduction at world creation was not an effective century-scale difficulty control here. It can alter site eligibility and the selected founding locations, and nutrient recycling/reserves buffer subsequent farming. These results do not establish that phosphorus has no effect or that longer runs would remain equally viable.

## Tuning recommendation

- **Mildly harsher:** yield 0.4, keeping other defaults. Lower growth with occasional observed abandonment.
- **More sustained hardship:** yield 0.35. More severe population declines, although abandonment frequency did not increase in this sample.
- **Clearly harsh but still viable:** yield 0.3, keeping 160-ha plots and ordinary weather. Some communities fail while others expand; the three worlds remain populated.
- **Occasional local losses:** storm probability 0.25 is a tested alternative with smaller population effects and three total abandonments. It is a deliberately large increase from 0.04, not a calibrated real-world storm frequency.
- **Catastrophe scenarios:** yield 0.1 or the severe drought bundle. These are unsuitable as ordinary difficulty defaults if continued human history is desired.

Do not combine several harsh settings assuming their effects add linearly. The combined-hard case retained only 331 active people in seed 81, substantially worse than yield 0.3 alone. No default has been changed. Repeat promising settings at the intended play resolution and with more seeds before assigning a target collapse rate.

## Limits discovered during testing

The phosphorus-0.01 extreme combination fails to find enough habitable founding sites in seed 17. That is a world/founding constraint, not a demonstrated historical collapse.

An attempted 200-year severe-survival run failed at month 1659 in seed 17 with `frontier route crosses forbidden land or has a broken seam`. The failed run and log are retained under `output/pressure-survival`; no completed 200-year result is claimed. Century runs are stored separately under `output/pressure-survival-century`.

The century severe-survival run completed seeds 17 and 81, but seed 256 hit the same route validation at month 1023. The summary retains the two completed seeds and marks the overall case failed; the failed seed is not assigned a completed-century outcome.

Collapse also exposed incomplete social cleanup. In the saved severe seed-17 world, all towns were abandoned by year 14.58, yet the following decades recorded 1,372 harvest notices, 480 inheritance events, 224 marriages and ongoing religious changes. These counts come from events after the last exact abandonment, not from a sampled estimate. The harvest notifier can read frozen last-production values at inactive towns, and religious updates do not consistently require surviving communities. This is not evidence that ruined towns were successfully producing food: demographic/production dispatch skips sub-one populations and the material ledgers remain separately checked.

Before making collapse common by default, inactive-town notifications and social actors need a proper retirement/relocation lifecycle. Traditions may legitimately survive in a diaspora, but new actions need living actors and support somewhere. Frontier routes also need to remain valid or become unavailable as extreme hydrology changes their water cells. The validation failure above has not been bypassed or hidden.

## Artifacts and verification

Results and exact commands: `output/pressure-sweep/summary.md`, `output/pressure-tuning/summary.md`, `output/pressure-survival-century/summary.md`, and `output/pressure-sweep/combined.json`. Individual reports retain annual town observations, abandonment dates, pressure parameters, conservation residuals and timings. Rejected/failed cases retain their logs. Severe completed worlds are saved for inspection.

The evaluator release build, Clippy with warnings denied, formatting check and Python compilation passed. The pressure experiments themselves exercise the new diagnostics; no simulation algorithms or default settings changed. Timings were collected under concurrent workloads and should not be used as isolated performance benchmarks.

```sh
mise exec rust@1.89.0 -- cargo build --release --example history_evaluate
python3 scripts/pressure_sweep.py --cases baseline,yield_040,yield_035,yield_030,yield_010,land_080,land_020,phosphorus_005,drought_090,storms_025,compound_hard --years 100
```
