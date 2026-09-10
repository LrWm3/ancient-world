# Integrated history diagnostics

Selected profiles are shown below (baseline yield 0.5; harsh yield 0.33). Living ecology, expeditions and repair priority enabled. See the manifest for policy and freight interventions. This is game-balance evaluation, not empirical fitting.

| Profile | Seed | People | Active sites | Abandonments | Shortage site-years | Freight saturated site-years | High-unrest site-years | Knowledge losses / gains | Water coverage | Max residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| baseline | 17 | 2257 | 24 | 1 | 868 | 0 | 1212 | 30 / 202 | 64.1% | 1.57e-05 |
| baseline | 81 | 2742 | 22 | 0 | 581 | 0 | 664 | 7 / 131 | 59.4% | 1.80e-05 |
| baseline | 256 | 2596 | 21 | 0 | 686 | 0 | 523 | 8 / 130 | 57.9% | 2.32e-05 |
| harsh | 17 | 1710 | 18 | 2 | 409 | 0 | 934 | 2 / 129 | 55.0% | 2.00e-05 |
| harsh | 81 | 1552 | 17 | 0 | 428 | 3 | 635 | 8 / 111 | 49.3% | 2.45e-05 |
| harsh | 256 | 2408 | 19 | 0 | 447 | 0 | 462 | 0 / 102 | 52.5% | 2.58e-05 |

Knowledge transitions compare consecutive annual observations of continuously active towns; losses during abandonment are excluded. Counts can miss within-year losses/recovery. Water coverage is the final unweighted active-town mean. Low capacity saturation does not prove there is demand. This suite uses one resolution and one GPU; all initial-state and catalog metadata remain in the raw JSON.
