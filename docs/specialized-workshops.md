# Industry-specific workshop capacity

Industrial equipment now belongs to four persistent families: general crafts, metalworking, kilns, and textiles/leather. Smelting, remelting, tools and military equipment use metalworks; charcoal and ceramics use kilns; fiber, yarn, cloth and leather use textile/leather facilities. Writing materials use general crafts. Food processing retains its household exemption.

Recipe `work[2]` selects a family (integer 0–3 in the order above). Catalog validation rejects invalid families. Equipment supports four worker-months per unit only in its family. All industries still compete for actual craft labor and a single shared household allowance; the allowance is not replicated per industry. No productivity multiplier or free goods are introduced.

Each family's investment target follows its outstanding orders and previous actual work, with 25% headroom and 10% monthly smoothing. The shared household allowance is deducted before apportioning desired equipment. Demand includes funded exports through the existing planner. Construction materials are requested only after calculating operating demand, preventing the construction orders themselves from justifying endless capital expansion. Total installed capacity is bounded by the existing population cap.

Equipment uses the existing 20 kg wood, 30 kg bricks and 2 kg tools per unit, two worker-months to build, and 0.2% monthly wear. The original material ledger remains authoritative; family units partition those same assets, never add another inventory. Assigned equipment retains its family when demand changes, including while the rule is disabled. Legacy unassigned assets require 0.5 worker-months per unit to fit to an industry; new construction includes fitting. Both building and fitting have bounded labor allocations. Idle equipment can gradually wear out while another industry invests, rather than instantly changing function.

Bundled new worlds enable `production.specialized_workshops`. Missing settings default to false and missing family records to zero, preserving old archives without granting assets. Old recipe catalogs retain their archived family codes (previously zero/general crafts). Disabling specialization restores pooled capacity without deleting assigned assets; disabling workshops also suspends wear and investment. The explorer offers **Require industry-specific workshops** and shows family units, targets and actual work. The evaluator control `--no-specialized-workshops` changes only this policy.

The GPU enforces construction, fitting, wear and recipe capacity. The CPU schedules sparse town targets. Each town adds 64 bytes of archived/GPU state. `production_summary.workshop_types` reports aggregate units and monthly work by family; work includes household activity and therefore is not solely equipment utilization.

These are communal regional facilities, not placed buildings or privately owned firms. Family-specific construction recipes, wages, skills and realized business profit remain outside this increment.

## Validation and three paired century runs

GPU fixtures compare equal physical investments in metalworks and kilns: only matching equipment increases tool production beyond household capacity. They also check shared household limits, nonnegative state, material accounting, idle wear, funded construction and checkpoint continuation. Idle investment was refined during implementation: declining demand must not automatically repair every existing facility forever.

Seeds 17, 81 and 256 ran for 100 years at terrain/ecology edge 64 with living environments, discoveries, scarce-island defaults and cost-aware contracts. The current-build control disables only specialized workshops. Arrows show pooled → specialized capacity.

| Seed | Population | Shortage site-years | Tool sufficiency | Dry kg/person, specialized |
|---|---:|---:|---:|---:|
| 17 | 5295 → 5236 | 6 → 5 | 98.9% → 98.7% | 10.54 |
| 81 | 5718 → 5705 | 5 → 5 | 98.8% → 98.6% | 11.05 |
| 256 | 5607 → 5597 | 7 → 11 | 98.1% → 98.0% | 10.70 |

| Seed | General units | Metalworking units | Kiln units | Textile/leather units |
|---|---:|---:|---:|---:|
| 17 | 0.030 | 1.019 | 1.525 | 0.002 |
| 81 | 0 | 0.870 | 2.247 | 8.940 |
| 256 | 0.020 | 0.732 | 1.277 | 0.002 |

The facilities have functional differences and the worlds develop different industry mixes: seed 81 sustains substantial textile work relative to the other two. Investment remains modest and distributed across towns, however; this is not yet evidence of major export-centered industrial hubs. Declining trade volumes and additional shortage observations in seed 256 are tradeoffs to track, not improvements. The changes also alter subsequent political and demographic trajectories, so these whole-history comparisons cannot isolate the cause of every shortage.

Dry stocks per person rose only modestly from year-50 values of 10.41/10.94/10.34. Maximum observed relative accounting residual stayed below 2.20e-5. No additional output or abundance multiplier was used to compensate for restricted capacity.

On the Quadro RTX 5000 Max-Q, enabled histories took 60–76 seconds with concurrent workloads, close to their paired controls (60–75 seconds). These are elapsed observations, not isolated benchmarks. Social processing and validation remained the largest recorded history stage at 20–27 seconds. The additional GPU state is 64 bytes per town; larger-resolution and multi-century calibration remain outstanding.

Mature seed-17 replay passed: 24 months in one batch exactly matched monthly advancement with a checkpoint after 12 months, including history, terrain and ecology. Artifacts: `output/specialized-workshops.json`, `output/specialized-control.json`, `output/specialized-workshops-analysis.md`, saved worlds, and `output/specialized-workshops-replay.json`.

The full suite passed all 100 tests, including hardware GPU tests. Following the idle-repair correction, all 13 economic fixtures passed again with strengthened idle-wear and shared-capacity assertions. Clippy with warnings denied and formatting checks passed.
