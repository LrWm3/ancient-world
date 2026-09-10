# Maintenance reservation ablation

This experiment isolates the industrial reservation in the
[food-security allocator](food-security-labor.md). The new
`production.food_security_maintenance = false` option removes only that
reservation. Food-pressure computation, smoothing, farming targets, service
commitments, stocks, recipes and finite production remain active. Missing archive
settings default to true, preserving the previously implemented food policy.
Ordinary worlds still default to the original adaptive allocator.

GPU policy mode 5 uses the food-pressure path with a zero maintenance reservation;
mode 4 retains the reservation. No new GPU buffers or simulation coefficients are
introduced. The controlled fixture generates one geological epoch to establish
finite deposits, then starts with idle industrial staffing and a tool
deficit to verify that protection can bind. It compares identical pressure inputs
and finite worker totals, then checks ablation checkpoint/batch continuation.

## Predeclared evaluation

Seeds **7307, 12011, 19001** were selected before running these policy comparisons.
They were not used to develop the food-policy constants. Test all four policies:
original adaptive, diagnostic fixed, food pressure with maintenance, food pressure
without maintenance. Use starting tool access 1 and 0, open/closed mines, 60 months
of closure followed by 60 reopened. This is 48 branches and 5,760 observations.
All cases start after the same 12-month spinup of a 64/32 terrain/ecology world
with sixteen civilizations and yield scale 0.33.

Do not tune coefficients against these seeds. Report every case, including
failures. They are held out for this first policy evaluation, not independent
observations of real societies; subsequent tuning against them would end that
held-out status. Larger towns, different climates/resolutions and longer histories
remain outside this run.

Measure cumulative harvest, unmet calorie-equivalent rations, population,
actual tool multiplier, tools made, and ledger residuals at months 60 and 120.
Compare maintenance versus no maintenance directly, and each food policy versus
original adaptive staffing. Also report the first differing labor month and the
count of identical full monthly observations: a reservation that never binds
cannot explain an apparent improvement elsewhere in the system.

```sh
python3 scripts/evidence.py --profile focused --policy-suite \
  --seeds 7307,12011,19001 --tool-fractions 1,0 \
  --cargo 'mise exec rust@1.89.0 -- cargo' --output output/evidence-maintenance
```

The report rejects missing/duplicate policies, mismatched checkpoints and incomplete
monthly trajectories. Journals include policy names to prevent branch overwrites.
Existing two-policy commands and archives remain supported.

## Held-out results (revision de8bbf7)

All 48 branches completed, producing 5,760 monthly observations. There were no
failed branches or coefficient changes. The focused runner passed 42 CPU tests,
23 GPU tests, eight Python runner/report tests, formatting and Clippy. This does
not claim the full ignored GPU suite passed on this revision.

The reservation changed staffing in ten of twelve seed/tool/mine combinations.
With no accessible starting tools it acted in month one in all six cases. With
starting tools present it first acted in months 13, 30, 33 or 30 in four cases;
the two remaining open-mine comparisons had identical recorded observations for
all 120 months. This establishes that the reservation can bind in ordinary runs,
not merely in the deliberately idle-industry fixture.

**Maintenance protection is not yet a demonstrated improvement.** At month 60,
all ten active comparisons accumulated more unmet rations than food pressure
alone: increases ranged from 2.44 to 742.33 kg calorie equivalents. Tool production
changed by −3.39 to +12.40 kg; the final tool multiplier changed by −0.000440 to
+0.001485. Reserving workers therefore does not guarantee replacement equipment.
At month 120, maintenance's settled-population effect ranged from −10.00 to +4.04
people. These are aggregate outcomes of diverging histories, not estimates of
famine deaths caused solely by maintenance. Ration demand changes with population,
and settlement population also changes through movement.

**Food-pressure response remains promising relative to original adaptive work.**
With maintenance enabled, all six open-mine cases had higher settled population
(+17.17 to +63.64 people at ten years), higher cumulative harvest and fewer unmet
rations. Without maintenance, all six also improved (+17.17 to +64.46 people).
Closed/reopened cases were less consistent: maintenance-enabled population effects
were −2.47 to +30.86; food-only effects were +7.17 to +30.70. Three held-out seeds
at one resolution and ten years do not establish a universal policy advantage.
The default adaptive allocator remains unchanged; food response stays opt-in.

Maximum normalized residuals were economy **3.201437e-6**, shared sources **0**,
population **1.605587e-7**; ecology relative C/N/P error **3.654122e-6**, water
**2.782312e-5**. The simulation experiment took 383.9 seconds on the Quadro RTX
5000 Max-Q, Vulkan/NVIDIA 595.84, Rust 1.89.0. CPU checks took 39.5 seconds and
focused GPU checks 75.4 seconds. Shared-workstation wall times are not controlled
performance benchmarks.

## Interpretation and next experiment

Keep the reservation available as an explicit experimental control, without
strengthening it or changing the default on this evidence. Its contract only
protects feasible forestry/mining/crafting capacity; it does not prioritize tools
inside the shared recipe queue. A useful next intervention is bounded replacement
**job commitments**: reserve existing inputs and actual craft work for identified
tool orders, release those reservations when blocked, and compare against identical
food pressure. Measure order feasibility, queued labor, completed tools, wear,
actual tool effectiveness and foregone harvest before interpreting population.
New coefficients should use development seeds, followed by a new held-out set.
Longer histories, larger towns, seasons, modern alloy economies, resolution studies
and sourced targets remain outstanding.

## Archived evidence and reproduction

- [All comparisons and residual maxima](evidence/maintenance-tables.md).
- [Artifact retention policy](evidence/README.md).
- [Artifact retention policy](evidence/README.md) and
  [stage coverage/timings](evidence/maintenance-run.md).

The decompressed JSON SHA-256 matches `artifacts_sha256["mine/results.json"]`
in the manifest. The source tree was clean and unchanged throughout the run.
Checkpoints, logs and per-branch journals remain in
`output/evidence-maintenance-de8bbf7/`. Running the command above on revision
`de8bbf7` regenerates the experiment. To regenerate just the table from the archive:

```sh
gzip -dc docs/evidence/maintenance-results.json.gz > output/maintenance-replay.json
python3 scripts/summarize_coupling.py output/maintenance-replay.json
cmp output/maintenance-replay.md docs/evidence/maintenance-tables.md
```

All favorable, unfavorable and unchanged cases are kept.

Before this run, a two-month seed-17 smoke comparison preserved all eight recorded
monthly observations exactly against the earlier food-policy/fixed archive. The
initial idle-industry fixture correctly failed to activate protection because its
unadvanced world had zero ore; generating one geological epoch supplied finite
feasible deposits, without injecting workers or goods. The corrected fixture also
checks identical pressure, bounded labor and exact save/resume/batch continuation
for the food-only mode.
