# Yield 0.33: survival calibration and next decision

**Keep 0.33 as the working harsh-scenario calibration. Next, implement distress-driven household relocation rather than reducing yield further.** The three century runs retain scarcity and uneven outcomes while producing fewer outright failures than 0.30. The application's default yield remains unchanged; `yield_033` is now a reproducible pressure-sweep case.

## Experiment

Seeds 17, 81 and 256; 100 years; terrain/ecology edge 64; sixteen initial communities and 1,920 resident-equivalents per world. Living environments, discoveries, current settlement lifecycle and economic systems are enabled. Only crop yield differs from the ordinary scenario. The 0.30 comparisons for 17/81 come from the final lifecycle sweep; seed 256 was rerun with the current implementation. These are small-grid comparisons, not a calibrated difficulty guarantee for every world or longer histories.

```sh
python3 scripts/pressure_sweep.py --cases yield_033 --seeds 17,81,256 --years 100 --output output/yield-033 --save-worlds
python3 scripts/pressure_sweep.py --cases yield_030 --seeds 256 --years 100 --output output/yield-033-control --save-worlds
python3 scripts/settlement_audit.py output/yield-033/*.world output/yield-033-control/*.world --output output/yield-033/activity-audit.json
python3 output/yield-033/analyze.py
```

| Seed | Final residents at 0.30 → 0.33 | Abandonments at 0.30 → 0.33 | Food-crisis episodes at 0.30 → 0.33 |
|---|---:|---:|---:|
| 17 | 1,567 → 2,169 | 6 → 1 | 183 → 55 |
| 81 | 1,650 → 2,220 | 4 → 1 | 96 → 48 |
| 256 | 3,305 → 4,223 | 0 → 0 | 26 → 15 |

Food crises are episodes reaching three consecutive hungry months, not total hunger duration. The existing shortage-site-year metric samples only the year-end month and misses seasonal hunger. The sweep summary now labels that limitation and separately includes crisis counts.

Final settlement sizes at 0.33:

| Seed | Towns | Villages | Hamlets | Remnants | Ruins |
|---|---:|---:|---:|---:|---:|
| 17 | 8 | 9 | 3 | 3 | 1 |
| 81 | 6 | 7 | 6 | 0 | 1 |
| 256 | 14 | 5 | 1 | 0 | 0 |

Seeds 17 and 81 dip below their starting population by year 25, then recover: 1,768 → 1,929 → 2,062 → 2,169 and 1,783 → 1,808 → 1,959 → 2,220 at years 25/50/75/100. Seed 256 grows throughout. Seven, nine and one initial communities respectively fall below half their starting population in annual observations. Prosperous communities still sponsor expeditions: 19, 35 and 25 launches respectively. Resident totals exclude crews currently away.

All four newly generated archives passed the settlement-activity audit: no ghost notices at ruins and no residual residents in abandoned sites. All annual conservation checks passed; the largest relative residual among the 0.33 worlds was **2.663e-5**. Exact commands, annual samples, logs, saves, comparison JSON and audit results are retained in the output directories above. No simulation algorithms changed for this experiment.

## Why relocation next

Iriwick in seed 17 falls from 120 starting residents to approximately 43 at year 11, 19 at year 21 and 8 at year 31, before abandonment at year 62.5. Its records contain 28 food crises and 51 flooded inhabited months. This is prolonged local decline despite survival and growth elsewhere, not a sudden world-wide failure.

The current human migration path creates daughter settlements from populous, well-provisioned origins. It does not let distressed households move into existing communities. Raids and expeditions account for some movement but do not supply that missing response. Raising the abandonment threshold alone would conceal the remaining residents rather than give them an alternative.

The next bounded implementation should therefore:

1. Trigger relocation interest from repeated monthly hardship or sustained flood exposure; use seasonal history rather than year-end shortage alone.
2. Find reachable, inhabited inner-continent destinations with spare food and productive capacity. Require travel provisions and transport capacity; an attractive destination is not automatically an accessible refuge.
3. Move actual cohorts/households with finite belongings and costs. Preserve genealogy, affiliation and ownership records; avoid copying people, goods or money.
4. Allow receiving communities to become crowded, refuse entry or struggle themselves. Evacuation should not guarantee survival or create food.
5. Compare relocation enabled/disabled at yield 0.33 over 200 years: deaths, migration, premature abandonment, destination crowding, seasonal hunger, conservation and checkpoint continuation.

Keep smaller viable settlements and regional variation. Do not tune every seed to produce the same number of ruins. Adopt 0.33 as a permanent default only after the migration and longer-duration comparisons establish the desired difficulty.
