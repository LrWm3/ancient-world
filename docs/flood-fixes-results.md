# Flood audit fixes and verification

Follow-up: [fresh-seed audit against the broader design](ideas-seed-audit.md), including unresolved initial climate convergence and persistent-town adaptation limits.

Implemented consistent settlement exposure, remembered unsafe sites, persistent-inundation state and bounded cargo delays. Physical runoff, water storage and bankfull capacity were not retuned. Existing towns can remain on waterlogged land; the fix distinguishes that condition from temporary cleanup rather than deleting water or relocating residents automatically.

## Changes

- GPU founder surveys and monthly historical expansion screening use the same local river-corridor depth as flood damage. Once observed flooded, a candidate stays excluded from that history's expansion list. Previously the local/area-average mismatch could admit an exposed site.
- Twelve consecutive wet months classify a town as persistently inundated. Cleanup stops restarting, the inspector identifies waterlogging, and daughter-village expansion is suspended. Twelve consecutive dry months restore ordinary status. Waterlogging still affects production, and actual wet months still damage food and crops.
- Blocked food cargo loses 20% of its remaining quantity each month. After six blocked months, remaining cargo of any type is written off. Managed C/N/P exports, food spoilage and goods used/waste counters account for losses. The buyer bears the already-paid cost; no refund is created. Legacy relief shipments follow the same food-loss and termination rules.
- Old saves remain readable. Existing flood events reconstruct missing wet streaks on the next step; already overdue blocked cargo is terminated on that step. New counters, events and candidate exclusions persist through checkpoints.

The 20%, six-month and twelve-month thresholds are explicit game rules. Salvage, insurance, relocation and flood defenses remain separate work. Candidate removal is conservative remembered exposure, not a probabilistic flood-risk estimate.

## Century comparison

Same settings as the previous `flood-complete` suite: seeds 0, 7 and 42, 64 terrain/ecology cells per face, one geological epoch, five founders, 100 years, default storms and all history systems enabled.

| Seed | Flooded town-months before / after | Final population change | Delayed cargo after | Recovered cargo after | Cargo still waiting at year 100 |
|---|---:|---:|---:|---:|---:|
| 0 | 276 / 115 | +53.4 | 23 | 23 | 0 |
| 7 | 68 / 0 | +13.4 | 0 | 0 | 0 |
| 42 | 8 / 0 | −1.9 | 0 | 0 | 0 |

Seed 0's new Lorwick occupies a different site and finishes with approximately 65.5 residents, versus 6.3 at the old inundated site. None of these fresh century worlds ends with a persistently inundated town. Seed 0 still suffers approximately 1.70 million kg of cumulative stored-food loss and 539,414 kg of crop loss: existing exposed towns still flood. This is a comparison of the whole fix package, including changed settlement choices and subsequent history, not an isolated causal estimate for one rule.

## Storm stress comparison

Seeds 0, 7 and 123 each run for 50 years at storm probability 0.20, matching the corresponding seeds from `flood-audit-storm`.

| Seed | Flooded town-months before / after | Delayed contracts after | Recovered | Written off | Waiting at endpoint |
|---|---:|---:|---:|---:|---:|
| 0 | 351 / 342 | 8 | 7 | 1 | 0 |
| 7 | 19 / 0 | 7 | 0 | 5 | 2 |
| 123 | 12 / 0 | 0 | 0 | 0 | 0 |

Seed 0 produces two persistent-inundation transitions and one recovery; one town retains the classification at year 50. A food cargo spoils by approximately 147.7 kg and then 118.1 kg during successive blocked months, before delivering its remainder. Every newly written-off contract terminates at six delay months. Seed 7's two endpoint shipments have each waited one month, rather than remaining unbounded. Road disruption can occur even when the towns themselves stay dry.

## Existing-world continuation

Loaded the earlier default-weather seed-0 year-100 archive and advanced one year. On month 1201, Lorwick immediately becomes persistently inundated with a zero cleanup timer, using its historical flood onset. Its overdue 120.3 kg **ore** cargo is written off; the buyer's approximately 192.5 paid cost remains a commercial loss. The prior audit initially called this food; its good ID is 1 (ore), and that report is corrected. The lack of spoilage for food was a separate code-level gap.

The resumed world retains Lorwick's residents, plots and physical water, passes its budgets, and saves successfully as `output/flood-fixes-resumed.world`. The desktop displays its persistent state and dry-month progress: [capture](../output/flood-fixes-persistent.png).

## Verification

All **58 tests passed**, including ignored hardware-GPU fixtures. Expanded flood tests cover partial food delivery after spoilage, six-month termination for food/timber/charcoal, nutrient/food/goods/money residuals, persistent-state transitions, twelve-month dry recovery and checkpoint-equivalent continuation. Existing coupled GPU history tests cover execution batching and save/resume. All-target Clippy with warnings denied and formatting checks passed.

The six fresh seed runs total 450 social years; the old-save continuation adds one year. All annual history and ecological validations passed. Maximum absolute relative residuals in the fresh suites were approximately 1.25e−5 for managed history, 1.70e−5 for ecological C/N/P and 5.06e−5 for ecological water. Annual assertions also found no unsafe retained settlement candidates; every sampled in-transit cargo delay remained below six months.

These are coarse diagnostic worlds. This pass does not establish production-resolution flood-frequency calibration or validate the surveyed bankfull reference against living runoff. Persistent water is now handled explicitly, but hydrological calibration and town relocation remain follow-up work.

## Reproduce

```sh
mise exec rust@1.89.0 -- cargo build --release --bin ancient-world --example history_evaluate
target/release/examples/history_evaluate --seeds 0,7,42 --resolution 64 \
  --epochs 1 --years 100 --civilizations 5 --discoveries --living-world \
  --output output/flood-fixes-century --label flood-fixes-century \
  --compare output/flood-complete.json --save-worlds
target/release/examples/history_evaluate --seeds 0,7,123 --resolution 64 \
  --epochs 1 --years 50 --civilizations 5 --discoveries --living-world \
  --storm-probability 0.2 --output output/flood-fixes-storm \
  --label flood-fixes-storm --save-worlds
python3 scripts/analyze_flood_audit.py output/flood-fixes-century output/flood-fixes-storm
```

Full evaluator reports: [century](../output/flood-fixes-century.md), [storm](../output/flood-fixes-storm.md). Matching JSON and `*-analysis.json` files retain annual budgets, events, persistent-site counts and delayed-cargo state.
