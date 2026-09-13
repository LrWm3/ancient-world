# Feasible recipe-work reservation screen

September 2026; experimental shader change **reverted**, baseline `665476f`.

## Question and intervention

Can a small protected recipe-work allowance restart workshops when town cash and
some inputs remain available? At seed 1024's year-50 boundary, site 0 holds 4,730
operating cash, 530 kg timber, 20.94 kg scrap and 12.48 kg metal, with tool and
wooden-object orders but no completed industrial work that month. This is an
endpoint observation, not proof that inputs were available throughout execution.
Its tool-deficit signal is zero, so the tool-specific maintenance allowance cannot
bind there. Its installed family capacities are also small (0.0047, 0.0016, 0.0458,
0.0003 units); cash alone does not establish viable enterprise entry.

The experiment modifies `food_worker_shares` in `shaders/economy.wgsl`. After the
existing `maintenance` calculation and before the mode-5 ablation, apply:

```wgsl
// Experiment only; not retained in the current shader.
if food_policy {
    maintenance.z = max(maintenance.z, min(feasible.z, 0.02));
}
```

The experimental source used a named `STAFFING_MAX_FEASIBLE_INDUSTRY_SHARE`
constant. Existing allocation transfers finite workers from farming while retaining
its minimum farm share. `feasible.z` excludes promised services and comes from the
existing sequential stock/input/order forecast, which includes forecast finite
extraction and food-processing recipes. This is therefore a recipe-work allowance,
not an exclusive private-industrial entitlement. It adds no goods, cash, assets,
customers or hiring eligibility. Mode 5 still removes all maintenance protection.

## Comparison

Three fifty-year candidate runs use the same 32/32 founding archives and settings
as the shared-inheritance screen: seeds 1024, 256, 409; service procurement 0.25;
contract/demand staffing, inheritance, named office service, delivery-paid exports
and abandoned-stock recovery enabled; reclamation, credit and issuance disabled.
Baseline exports are the previously completed `665476f` shared-inheritance runs.
Generated candidates, commands and logs stay ignored in
`output/industrial-work-floor/`. No performance claim is made.

| Seed | Population baseline → candidate | Ending need-weighted hunger | Cumulative operator work | Operator revenue minus wages/rent | Active operators |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 154.263 → 155.982 | 0.12297 → 0.05348 | 12.680 → 11.797 | 50.79 → 45.23 | 0 → 0 |
| 256 | 345.715 → 354.577 | 0.03746 → 0.03172 | 4.693 → 4.459 | 19.66 → 20.20 | 0 → 0 |
| 409 | 352.540 → 348.851 | 0.01888 → 0.05556 | 183.934 → 162.254 | 952.25 → 858.09 | 4 → 2 |

All three candidates complete successfully. Maximum absolute endpoint audited
relative cash residual is 2.10e-7. These are whole-history interventions: subsequent
population and price differences cannot be attributed to one immediate mediator.
No long-run checkpoint or cross-resolution study was performed for this rejected
variant. The earlier inheritance fixtures do not verify this allocation experiment.

## Decision and next connection

Revert the allowance. Food outcomes improve in two seeds but deteriorate in 409;
private operator work falls in all three. This does not demonstrate that every
alternative reservation policy fails, but this one does not justify changing the
existing default. Keep the food improvements as a possible later calibration lead.

Inspection confirms a more structural demand gap: household retail settles food
purchases, while municipal production requests material reserves. There is no
general household material-good purchase/possession/use path. Household savings
therefore cannot directly finance ordinary clothing, containers or similar goods.
Do not address that by assigning household wealth to municipal orders again.

A next bounded pilot should use existing cloth or substitutable containers, protect
food budgets, purchase actual local inventory into household-owned durable stocks,
and return worn material through the existing ecological ledger. Purchased goods
must leave town stock exactly once; use must consume finite condition/material,
and replacement demand must stop at a household service target. Compare completed
private work, household food access and cash distribution, not just retail turnover.
Avoid an artificial recurring cash debit that has no corresponding goods or use.
Unmined abandoned resources still need a separate funded extraction/access path.
