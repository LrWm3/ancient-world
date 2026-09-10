# Fishing returns and alternative work

This opt-in extension follows the [timber-trap experiment](timber-fisheries.md),
where more catches coincided with worse food security. It reduces proposed fishing
and construction work when observed alternative food returns are higher.

Enable adaptive fishing, optionally primitive traps, and **Compare fishing with
observed food returns** in economy settings. The API flag is
`agriculture.fishery.opportunity_cost`. It defaults to false in both new catalogs
and imports missing the field. Existing rates, equipment costs and stock-response
settings are not raised by this policy.

## Decision and units

Each settlement retains an exponentially smoothed observation, in kg of dietary
food equivalent per ordinary allocated worker-month:

```
observation = max(0, monthly_food_output - current_catch * fish_food_conversion)
              / max(ordinary_allocated_workers, 1)
alternative_next = (11/12) * alternative + (1/12) * observation
```

The observation is updated after GPU production; the next month's decision reads
the completed prior value. Its memory timescale is approximately a year, not an
exact rolling twelve-month average. Recording continues with the decision rule off,
so enabling it after the common warmup does not invent a history of observations.
Old archives with no observation start at zero and learn through subsequent work.

The fish estimate uses the existing local guild-density response, catalog food
conversion constrained by C/N/P, and the expected installed gear mix. With primitive
gear enabled, unsupported crew are estimated at the trap rate; existing advanced
outfits can raise that estimate. New capacity amortizes two construction worker-months
over a provisional twelve-month operating horizon:

```
expected = rate * stock_response * food_conversion * gear_efficiency
           / (1 + construction_work_per_anticipated_operating_month)
crew_factor = clamp(1 - alternative / max(expected, 0.001), 0, 1)
proposed_crew *= crew_factor
```

The factor is a conservative heuristic, not a solved labor-market equilibrium or
an empirically fitted marginal-product equation. Both investment and operating work
follow the reduced crew target. Actual timber, tools, fiber, workers and wildlife
remain bounded by the existing stock-and-flow calculations. The factor neither
creates food nor reallocates a fictional workforce.

A new `fishery_choice` GPU vector persists the smoothed observation, latest expected
return, crew factor and policy flag. This adds 16 bytes per settlement (4 KiB at
256 settlements) and no host readback/decision stage. The inspector displays the
returns and factor; evaluator trajectories archive them.

## Idle allocation correction

Previously adaptive fishing tried to undo the legacy 0.1% fishing reservation by
dividing all worker shares by 0.999. Some legacy policies had subtracted that
reservation specifically from forestry, so the division did not restore the same
allocation. An idle adaptive fishery could therefore differ from a closed fishery.

Worker-share helpers now make the legacy reservation only for legacy fisheries.
Adaptive fisheries subtract actual work once; zero actual work returns ordinary
shares unchanged. This intentionally corrects adaptive allocation behavior, including
when the new opportunity flag is off. Historical results from earlier commits are
not used as the sole causal comparison: the evaluator adds `work_ablation` from the
same checkpoint and current implementation, disabling only the new opportunity flag.

## Controlled verification

A synthetic high observed return blocks fishing and construction despite sufficient
food demand, gear materials and wildlife. The matched disabled-rule branch catches
fish. Reducing the synthetic observation permits catches again. Idle shares match
an explicitly closed fishery exactly. The fixture checks economic/ecological budgets
and exact checkpoint/batch continuation, including ecological bytes. Existing gear,
workforce, demand, depletion and closure fixtures remain in the suite.

## Interpretation limits and remaining work

The observation is **food handling output per allocated ordinary worker**, not a
pure marginal crop yield: it can include processing imported or previously stored
food. Non-food products, wages, household entitlements and trade profits are not
valued. Subtracting current catch is an approximation when fish is retained rather
than converted; legacy warmup catches can contribute to the observation. A cold
start at zero can temporarily underestimate alternatives.

Seasonality is represented by memory, not a forecast of planting and harvest labor
or a separate idle-worker pool. A prosperous farming town may rationally do no fishing;
a low observation in the controlled fixture demonstrates an available fishing niche,
but does not prove that natural maps contain profitable fishing towns. Expected gear
mix also does not plan a fully financed future upgrade chain.

Commercial fish reservations remain pending. Edible goods are currently excluded
from repeat export contracts, and raw fish is normally converted to general food.
A commercial extension must reserve a finite catch for a funded buyer, handle
spoilage and cancellation, preserve emergency local consumption, and use the existing
route and cash ledgers. It must also compare returns including material replacement
and transport costs; adding buyers or a catch bonus would bypass these constraints.

## Matched ten-year evaluation

Final-source runs use terrain 64, ecology 16, 120 ecological spinup months and twelve
common history warmup months. Each configuration has seven branches: baseline,
sham, guild removal, halfway restoration, harvest closure, removal plus closure,
and `work_ablation`. The last branch disables only the opportunity flag after loading
the common checkpoint. The idle-allocation correction is present in both branches.
No fishing gear, materials, money or animals are imported at activation.

| Scenario | Catch with rule kg | Catch without rule kg | Unmet rations with / without | Final population with / without |
|---|---:|---:|---:|---:|
| Seed 17, yield 0.5 | 0 | 610.06 | 1.39% / 1.39% | 2,095.95 / 2,095.33 |
| Seed 81, yield 0.5 | 0 | 665.52 | 1.90% / 1.96% | 2,042.24 / 2,036.52 |
| Seed 256, yield 0.5 | 0 | 2,503.37 | 1.18% / 1.24% | 2,108.62 / 2,102.52 |
| Seed 17, yield 0.33 | 0 | 7,963.81 | 9.12% / 9.51% | 1,438.81 / 1,405.57 |
| Seed 17, yield 0.33, more generous stock response | 4.81 | 58,547.21 | 9.12% / 10.20% | 1,438.77 / 1,366.13 |

The first four configurations retain the existing half-saturation density of
0.0001 kg C/m². The last tests 0.00001, prompted by observed coastal densities near
that order of magnitude. It changes catch response, not wildlife inventory or the
finite withdrawal limit. Both its enabled and ablated branches use the same altered
setting. This is a sensitivity experiment, not an empirical calibration or a new
default. No catch-rate or food-production multiplier was increased.

**The safeguard works here by mostly rejecting fishing.** All standard baselines
have zero catch and the same reported food/population outcomes as closing harvest.
The sensitivity case permits a few catches, but remains very close to closure.
It does not demonstrate a self-sustaining fishing town, globally optimal employment,
or that the food-handling proxy correctly estimates marginal returns in every setting.
The controlled low-observation fixture, rather than these natural worlds, establishes
that the rule can permit fishing when its estimated returns justify it.

All 20 standard control checks pass across the 35 trajectories: sham trajectories
match baseline, restoration matches removal before the intervention, and all closed
branches have zero catch. Maximum absolute relative residuals are 2.61e-6 for the
economy, 2.65e-6 for ecological C/N/P, and 1.60e-6 for water. These are aggregate
floating-point ledger checks, not exact local withdrawals at planetary cell areas.

Final validation passes 49 ordinary tests and 25 explicitly executed GPU tests,
including the new analytical one-step memory recurrence, decision ablation,
idle-allocation comparison and checkpoint continuation. Formatting and Clippy with
warnings denied also pass. Tested hardware is Quadro RTX 5000 Max-Q, Vulkan,
NVIDIA 595.84; other backends are not verified by this experiment.

**Decision:** keep the rule opt-in and retain the existing stock-response default.
It prevents the demonstrated allocation loss but does not solve fishing specialization.
Next refine return estimates around seasonality, actual idle work and material costs,
and connect funded catch reservations and viable advanced-gear investment. Do not
force a fishing role or lower the comparison standard just to generate more catches.

## Reproduction

```sh
mise exec rust@1.89.0 -- cargo test --test adaptive_fisheries -- --include-ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --test economy --test living --test living_scenarios -- --ignored --test-threads=1
FISHERY_ADAPTIVE=1 FISHERY_PRIMITIVE=1 FISHERY_OPPORTUNITY=1 FISHERY_OUTPUT=output/opportunity-final-natural mise exec rust@1.89.0 -- cargo run --example living_fishery_evaluate -- 120
python3 scripts/analyze_living_fisheries.py output/opportunity-final-natural
FISHERY_ADAPTIVE=1 FISHERY_PRIMITIVE=1 FISHERY_OPPORTUNITY=1 FISHERY_OUTPUT=output/opportunity-final-harsh mise exec rust@1.89.0 -- cargo run --example living_fishery_evaluate -- 120 17 16 0.33
python3 scripts/analyze_living_fisheries.py output/opportunity-final-harsh
FISHERY_ADAPTIVE=1 FISHERY_PRIMITIVE=1 FISHERY_OPPORTUNITY=1 FISHERY_DENSITY_HALF=0.00001 FISHERY_OUTPUT=output/opportunity-final-sensitivity mise exec rust@1.89.0 -- cargo run --example living_fishery_evaluate -- 120 17 16 0.33
python3 scripts/analyze_living_fisheries.py output/opportunity-final-sensitivity
```

[Evidence](evidence/fishery-opportunity/) retains all final-source monthly trajectories,
configuration/catalog metadata, summaries, logs and checksums. The analyzer also accepts
the compressed directories directly and validates the declared branch suite. Earlier
development pilots preceded the idle-allocation fix and are not used in this table.
Run times include compilation, readbacks, serialization and concurrent workloads;
they are not isolated performance benchmarks.
