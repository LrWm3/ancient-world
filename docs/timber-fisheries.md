# Timber traps and early fishery investment

[Opportunity-cost follow-up](fishery-opportunity-cost.md) adds observed food returns
to crew decisions and corrects idle adaptive workforce allocation. The results below
remain the original timber-trap experiment, before those changes.

This follow-up to [adaptive fisheries](adaptive-fisheries.md) addresses the observed
failure to construct equipment during food shortages. It adds a second, finite gear
class rather than increasing the underlying fishing yield.

Enable **Adaptive fishing and finite equipment** and **Allow primitive timber fish
traps** in economy settings. API users set `agriculture.fishery.adaptive` and
`primitive_gear` in the economy catalog. Both remain opt-in; old archives and catalogs
retain their previous behavior.

## Mechanism and accounting

| Gear | Material per supported worker | Construction | Catch rate relative to configured rate |
|---|---|---|---|
| Existing outfit | 40 kg timber, 1 kg tools, 4 kg fiber | 2 worker-months | 100% |
| Timber traps | 20 kg timber | 2 worker-months | 35% |

These are explicit game-design assumptions, not measured historical productivity.
Timber traps abstract hand-built wooden shore traps; they do not represent boats,
metal hooks or textile nets. Both classes wear by 0.3% each month, even when harvest
is closed. Traps consume the existing finite timber inventory; there is no arrival
import or new forest resource. Timber moves into held equipment, then through the
existing loss and C/N/P detritus ledgers as it wears.

The GPU constructs the existing outfit when its inputs are available and fills
remaining desired capacity with traps when allowed. Existing trap capacity counts
toward the target, preventing duplicate outfits for the same crew. One construction
budget and one workforce ceiling apply across both types. Better outfits receive
crew first; residual crew can use traps. Catch and actual work are computed using
the resulting combined productivity and existing local stock limits.

The production planner requests trap timber when fiber or surplus tools are missing,
rather than placing simultaneous orders for two full alternatives. Existing useful
gear reduces those requests. Disabling traps retains the physical timber but removes
its operating capacity. No automatic upgrade or timber recycling is claimed.

The archived `fishery_traps` vector contains held timber, cumulative installation,
cumulative wear, and the policy flag. It adds 16 bytes per settlement, or 4 KiB at
256 settlements. Dense work remains in the existing GPU fishery pass. The inspector
shows held and worn trap timber, and evaluator records now include equipment input
stocks and procurement targets to expose future shortages.

## Verification

A controlled GPU fixture first provides wood with traps disabled and verifies that
missing fiber prevents catches. Enabling traps permits production from the same
material supply. It checks the lower productivity bound, supported crew, shared
construction/fishing/ordinary workforce, timber accounting, economic and ecological
budgets, exact checkpoint and batch continuation, and retained equipment wear after
closure. The existing advanced-gear fixture and ordinary economy/living-history
regressions are also exercised.

## Remaining commercial connection

This completes a subsistence equipment alternative, not commercial fishing towns.
Local reserves and wildlife still drive fishing demand. Current repeat export
contracts explicitly exclude edible goods (`observe_export_delivery`), and monthly
food processing converts raw fish into the general food inventory. Thus simply
adding export demand to the catch target would not preserve a shipment for sale.

The next useful extension is a bounded, funded reservation of fish for an actual
buyer, with explicit food-processing priority, spoilage and delivery. It must use
existing money, routes and cargo accounting, release reservations on cancellation,
and preserve emergency local food access. Gear upgrades and sale-price/operating-cost
comparisons can then justify investment in better outfits. These are still pending;
this change does not fabricate buyers or guarantee a fishing role.

## Ten-year seed evaluation

Terrain 64, ecology 16, seeds 17/81/256, 120 ecological spinup months and 12 common
history warmup months; no gear, material or wildlife imports at activation. The
standard yield is 0.5, with a separate yield-0.33 run for seed 17. Each configuration
runs six branches: baseline, sham, guild removal, restoration halfway through,
harvest closure, and removal plus closure.

| Seed / yield | Previous adaptive catch kg | With traps, catch kg | Fishing worker-months | Construction worker-months | Potential fish food / ration need |
|---|---:|---:|---:|---:|---:|
| 17 / 0.5 | 0 | 654.30 | 156.78 | 25.70 | 0.0081% |
| 81 / 0.5 | 24.94 | 665.52 | 125.05 | 23.27 | 0.0082% |
| 256 / 0.5 | 0 | 2,502.70 | 314.56 | 38.21 | 0.0311% |
| 17 / 0.33 | 0 | 7,950.41 | 906.68 | 46.76 | 0.1223% |

The previous column is the archived first adaptive policy, not legacy fixed-labor
fishing. All new baselines establish catches without subsidized equipment. The
change resolves the joint-material bootstrap failure, but output remains a small
share of aggregate food needs, and it does not establish sustained fishing towns.

**The pressure comparison is not an unqualified success.** At yield 0.33, baseline
unmet rations are 9.51%, versus 9.12% with harvest closed; final population is
1,404.57 versus 1,438.81. Equipment procurement and fishing compete with ordinary
production. The outcomes are consistent with that opportunity cost, but the full
nonlinear population difference is not an isolated estimate of displaced farming.
Guild removal cuts catch to 5,444.90 kg; restoration yields 5,948.79 kg, not full
recovery. The closed/removal-plus-closed controls produce no catch.

All 16 control checks pass: sham trajectories match their baselines, restoration
matches removal until intervention, and all harvest closures stop catches. Maximum
absolute relative residuals across the 24 trajectories are 2.36e-6 for the economy,
2.65e-6 for ecological C/N/P, and 1.60e-6 for water. These are aggregate f32 ledger
checks, not exact local mass arithmetic at planetary cell areas.

The harsher pilot exposed a roundoff failure: `(timber / 20) * 20` could exceed the
available timber by a tiny amount, creating a negative inventory and subsequently
a negative housing-construction job. Construction now clamps the actual transferred
mass to inventory, then derives installed capacity from that mass. The corrected
24 trajectories completed. Initial exploratory runs are not used in this table.

Validation on the final source: 49 ordinary tests and 24 explicitly run GPU tests,
including both fishery fixtures and 22 economy/living-history regressions; formatting
and Clippy with warnings denied also pass. Both fixtures verify checkpoint/batch
continuation. Quadro RTX 5000 Max-Q / Vulkan / NVIDIA 595.84 is the tested backend.

**Decision:** keep both policies opt-in. Prioritize opportunity-aware work allocation
before commercial expansion: compare observed productive returns, account for seasonal
farm demand and idle labor, and require counterfactual evidence of improved food
security rather than targeting more catch. Funded fish reservations remain the next
trade connection after that. No rate multiplier was raised to manufacture a favorable
result.

## Reproduction

```sh
mise exec rust@1.89.0 -- cargo test --test adaptive_fisheries -- --include-ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --test economy --test living --test living_scenarios -- --ignored --test-threads=1
FISHERY_ADAPTIVE=1 FISHERY_PRIMITIVE=1 FISHERY_OUTPUT=output/timber-fisheries-v2 mise exec rust@1.89.0 -- cargo run --example living_fishery_evaluate -- 120
python3 scripts/analyze_living_fisheries.py output/timber-fisheries-v2
FISHERY_ADAPTIVE=1 FISHERY_PRIMITIVE=1 FISHERY_OUTPUT=output/timber-fisheries-harsh-v2 mise exec rust@1.89.0 -- cargo run --example living_fishery_evaluate -- 120 17 16 0.33
python3 scripts/analyze_living_fisheries.py output/timber-fisheries-harsh-v2
```

[Evidence](evidence/timber-fisheries/) contains all monthly trajectories, full catalog
and configuration metadata, summaries, logs and checksums. The analyzer also accepts
those compressed evidence directories directly. Elapsed times include compilation,
readback, serialization and concurrent local workloads; they are not benchmarks.
