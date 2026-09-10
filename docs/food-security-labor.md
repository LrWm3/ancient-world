# Experimental food-security staffing

`production.food_security_labor` defaults to false. It requires enabled adaptive
planning and excludes diagnostic fixed staffing. The policy changes finite work
allocation, not crop coefficients, tool supply, initial inventories or market rules.

## Monthly contract

Before production, the planner records:

- Monthly food need: current child/adult/elder populations × 10/18/14 kg calorie
  equivalents. Without demographic cohorts, use 18 kg per resident.
- Food-reserve months: the town's available aggregate food divided by that need.
  Cargo and unharvested crops do not count as available food.
- Target pressure: the larger of previous ration shortfall and
  `clamp(1 − reserve months / 3, 0, 1)`.
- Tool deficit: `clamp(1 − effective tools / (0.5 × population), 0, 1)`;
  existing bronze/copper equivalents retain their ordinary efficiencies.

The GPU uses `pressure = 0.75 × previous pressure + 0.25 × target` for both
cultivated-area allocation and actual production staffing, then commits that
memory once per production month. Replanning alone does not advance the memory.
The provisional farm floor is `0.62 + 0.20 × pressure` of available workers.
Unchanged gradual reassignment still limits ordinary occupation swings.

Feasible forestry, raw-ore extraction and metal/tool recipe work define a limited
maintenance reservation. Its total is capped at `0.08 × tool deficit` of the
workforce. It uses the same current inputs, orders, storage room, knowledge and
finite extraction forecast as ordinary planning. Existing service commitments
remain protected. If needed, reservations reduce the provisional farm floor,
but never below 0.62. All shares remain bounded by the existing workforce and
fishery deduction. No new workers or materials are created.

The reservation protects industrial **capacity**, not a particular finished tool.
Craft recipes still share their pool and ordinary scheduling; a reservation alone
does not guarantee replacement output. This is a limitation to measure before
considering stronger job priorities. The constants are experimental game rules,
not empirically fitted estimates. The default allocator remains unchanged.

`Economy.food_labor` stores remembered pressure, instantaneous target pressure,
reserve months and tool deficit. These are archived diagnostics, not inventories.
Older saves initialize the fields to zero and leave the policy disabled.

## Reproduction

```sh
python3 scripts/evidence.py --profile full --tool-fractions 1,0.25,0 \
  --food-security --reference docs/evidence/tool-buffering-results.json.gz \
  --cargo 'mise exec rust@1.89.0 -- cargo' --output output/evidence-food-labor
```

This retains seeds 17/81/256, identical checkpoint creation, five closed years
and five reopened years. The experimental policy replaces only the adaptive
branches. Fixed branches must exactly reproduce all prior monthly measurements,
excluding the new diagnostic field, or report generation fails. Generated tables
compare the new policy with the previous adaptive policy at each matched stock
and mine-access setting, alongside the within-policy mine effects.

Tests cover controlled pressure response, unchanged worker totals, finite goods
budgets, existing industrial capacity, exact checkpoint/batch continuation,
configuration rejection and old archive defaults. The experiment is causal model
evidence, not empirical calibration or held-out validation.

The baseline WGSL allocator is kept in a separate function with its original
arithmetic. Folding new policy calculations into its body caused small floating
point differences during the disabled-policy spinup; the fixed-control guard
caught this. The affected experiment was stopped. A short control run after
isolating the new path reproduced all four prior monthly samples exactly.

The next ensemble exposed a pre-existing numerical bug in ration-limited raid
mustering at seed 256, history month 48: dividing food to bound recruits and then
multiplying back produced a cost slightly above stock, leaving −0.0000305 kg.
The corrected transfer caps both the debit and carried provisions by available
food. It does not clamp away a debt after transferring a larger quantity. The
CPU regression fixture uses an exact f32 bit pattern that provably triggered the
old overdraw; focused evidence now also executes the existing provisioned-raid
conservation/continuation fixture. Settlement validation reports the offending
site, month and fields instead of only a generic error.

## Completed comparison at revision 60189b2

The final **focused** run passed 42 CPU Rust tests, 22 hardware tests and six
Python checks, formatting and Clippy. All 36 branches completed: 4,320 monthly
observations and three no-op checks. All **2,160 fixed-control observations**
exactly reproduce the previous archive after removing only the new diagnostic
field. Source hashes stayed unchanged during execution.

At five years, all nine open-mine comparisons have greater cumulative harvest,
lower population-weighted food shortfall and higher settled population than the
previous adaptive policy:

| Seed | Starting tool access | Population difference | Harvest difference kg | Food shortfall difference, percentage points |
|---|---:|---:|---:|---:|
| 17 | 1 | +36.48 | +26,180 | −0.587 |
| 17 | 0.25 | +44.57 | +42,583 | −1.023 |
| 17 | 0 | +55.52 | +46,887 | −1.269 |
| 81 | 1 | +33.80 | +27,480 | −0.760 |
| 81 | 0.25 | +55.99 | +47,698 | −1.372 |
| 81 | 0 | +54.82 | +48,019 | −1.569 |
| 256 | 1 | +17.08 | +20,884 | −0.525 |
| 256 | 0.25 | +44.37 | +46,228 | −1.044 |
| 256 | 0 | +54.95 | +46,540 | −1.371 |

Closed-mine effects are smaller and mixed: population differences range from
−0.18 to +14.02 at five years, and some cumulative harvests decline slightly.
The existing allocator already moves substantial idle industrial labor into
farming when mines close. Additional food-pressure response therefore has less
scope to help. These comparisons support that interpretation without identifying
a unique causal pathway for every subsequent population difference.

At five years, the largest tool-multiplier reduction is 0.00343 (0.343 percentage
points). At ten years, the largest reduction is 0.00616 (0.616 percentage points,
seed 17, quarter-access, continuously open mines). Thus the improved food outcome
does not imply zero equipment cost. Population advantages in open-mine branches
remain +18.60 to +57.25 at ten years; formerly closed branches range from −0.31
to +46.56 after reopening. The report preserves monthly site measurements to
avoid hiding seasonal or distributional differences in these interval summaries.

The experiment changes food pressure and maintenance protection together. It does
**not** separately establish the benefit of the maintenance reservation. The subsequent [maintenance ablation](maintenance-ablation.md) isolates it on
three predeclared held-out seeds and finds mixed equipment effects and an early
food cost. Larger towns, crop-season and longer-history checks remain pending. The policy remains opt-in pending those tests and sourced targets. These
are modeled outcomes, not historical calibration; all three seeds were used in
development.

Maximum normalized residuals: economy **3.306e-6**, shared sources **0**, population
**1.942e-7**. Ecology relative C/N/P error was **3.680e-6** and water error
**2.539e-5**. All guards passed. GPU: Quadro RTX 5000 with Max-Q Design, Vulkan,
NVIDIA 595.84; Rust 1.89.0. The final experiment took 274.7 seconds; CPU tests
22.4 seconds and focused GPU stages 35.9 seconds. These are single-run wall times
on a shared workstation, not controlled performance benchmarks.

Use the reproduction command above with `--profile focused` to match this final
run's coverage. `full` additionally executes the rest of the ignored hardware
suite. The earlier full-suite run passed 98 GPU tests at revision `3bea509`, but
its experiment was stopped for control drift; that does not constitute a full-suite
pass on the corrected final revision.

## Evidence artifacts

- [Generated comparisons](evidence/food-labor-tables.md).
- [Artifact retention policy](evidence/README.md).
- [Artifact retention policy](evidence/README.md) and
  [coverage/stage report](evidence/food-labor-run.md).
- Failed attempt manifests: [Artifact retention policy](evidence/README.md)
  and [Artifact retention policy](evidence/README.md).
- [Artifact retention policy](evidence/README.md).

The decompressed result hash equals `artifacts_sha256["mine/results.json"]` in
the final manifest. Complete logs, checkpoint binaries and monthly JSONL journals
remain in `output/evidence-food-60189b2/`; failed-run logs and partial observations
remain in their separately named output directories. Final archived results contain
all branches, including mixed outcomes. No parameters were tuned after inspecting
seed results. Corrections fixed disabled-policy arithmetic and a bounded transfer;
they did not change crop or demographic coefficients.
