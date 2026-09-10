# Integrated history diagnostics

Selected profiles are shown below (baseline yield 0.5; harsh yield 0.33). Living ecology, expeditions and repair priority enabled. See the manifest for policy and freight interventions. This is game-balance evaluation, not empirical fitting.

| Profile | Seed | People | Active sites | Abandonments | Shortage site-years | Freight saturated site-years | High-unrest site-years | Knowledge losses / gains | Water coverage | Max residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| baseline | 17 | 2162 | 24 | 1 | 816 | 0 | 793 | 22 / 200 | 62.3% | 1.54e-05 |
| baseline | 81 | 2703 | 22 | 0 | 544 | 0 | 681 | 5 / 148 | 60.3% | 1.78e-05 |
| baseline | 256 | 2594 | 21 | 0 | 679 | 0 | 455 | 2 / 124 | 57.6% | 2.34e-05 |
| harsh | 17 | 1693 | 19 | 1 | 379 | 0 | 430 | 13 / 140 | 44.5% | 2.00e-05 |
| harsh | 81 | 1553 | 17 | 0 | 414 | 7 | 486 | 11 / 114 | 52.4% | 2.46e-05 |
| harsh | 256 | 2419 | 18 | 0 | 430 | 0 | 266 | 0 / 96 | 53.2% | 2.67e-05 |

Knowledge transitions compare consecutive annual observations of continuously active towns; losses during abandonment are excluded. Counts can miss within-year losses/recovery. Water coverage is the final unweighted active-town mean. Low capacity saturation does not prove there is demand. This suite uses one resolution and one GPU; all initial-state and catalog metadata remain in the raw JSON.
