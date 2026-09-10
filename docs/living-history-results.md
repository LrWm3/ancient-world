# Living history validation

On the Quadro RTX 5000 with Max-Q Design, seeds 0, 7 and 42 completed 100 social years with living ecology, farming, society, politics, governance, shipping, expeditions and specimen research enabled. Terrain and ecology resolution were 64, with five founding civilizations and one geological epoch. That epoch supplied an initial ten ecological years; coupled history then added 1,200 months to both clocks.

All annual ecological and history budget checks passed. These are relative conservation residuals, not biological calibration errors. Wall times include generation and reporting; runs overlapped regression work and are not isolated performance benchmarks.

| Seed | Social months | Ecological months | Max C/N/P residual | Max water residual | Seconds |
|---|---:|---:|---:|---:|---:|
| 0 | 1200 | 1320 | 1.43e-05 | 5.07e-05 | 54.6 |
| 7 | 1200 | 1320 | 1.46e-05 | 4.49e-05 | 61.7 |
| 42 | 1200 | 1320 | 1.5e-05 | 4.6e-05 | 45.6 |

[Full history outcomes](../output/living-final.md) and [monthly-clock/annual-budget data](../output/living-final.json). No wars occurred in these three runs, so they do not establish realistic warfare outcomes. Dedicated legacy warfare tests still pass.

A 256-resolution seed-42 run completed one social year with valid budgets in 18.5 seconds including generation ([report](../output/living-production.md)). The existing seed-seven specimen archive advanced from social year 100 to 105 after explicit activation, preserving its 15 settlements and reaching 1,864 residents ([history](../output/living-continued.json), `output/living-continued.world`). These checks do not benchmark 512/1024 terrain or centuries at production resolution.

The regression suite passed 56 tests including GPU fixtures. The two living-history fixtures also passed after the final calendar and archive-validation changes. They verify coarse-grid water/C/N/P accounting, shared drought forcing, changing climate and vegetation with unchanged terrain/routing, exact history/terrain/ecology checkpoint continuation, different batch sizes and rejection of incomplete saves. Clippy passed with warnings denied.

Desktop validation loaded the upgraded archive, advanced one month with Step, played several months, and paused. Both ecological and social clocks advanced, the map updated, and the saved archive was left unchanged. [Control screenshot](../output/living-controls.png).

See [design and limits](living-history.md). The current biological outcomes remain an initial coupling baseline rather than a completed calibration of living civilization history.
