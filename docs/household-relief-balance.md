# Household relief: budget versus dietary target

## Question and mechanism

Earlier common-entitlement comparisons improved population without increasing
crop output, but did not stabilize the century histories. This comparison asks
whether the existing targeted cash-relief mechanism can reduce unequal food
access at the unchanged 0.5 long-run common share.

During Reserve, household payroll and dividends are credited first. Relief then
requests the cash needed to reach the configured dietary target above common
entitlement. Requests across all controlled towns share the council budget
proportionally. Grants transfer existing treasury into household wallets; retail
uses the resulting purchasing power to cap GPU consumption. Actual food remains
limited by physical stocks. Later taxation and council decisions cannot fund this
month's earlier relief retroactively.

The experiment changes existing policies, not the simulation equations or defaults.
The runner adds paired `--household-relief-share` and `--household-relief-target`
options, rejects missing partners and nonfinite/out-of-range fractions, and records
both overrides as comparison metadata.

## Protocol

Source baseline `0450bd6`, with the accompanying runner-only controls/observations.
Quadro RTX 5000 / Vulkan, development-profile example. Two seeds (17, 81), thirty
years, sixteen founding civilizations, terrain 32, ecology 16, one geological
epoch, living history, crop yield 0.5. All three arms use:

```sh
cargo build --example cultural_work_calibrate
target/debug/examples/cultural_work_calibrate \
  --seeds 17,81 --years 30 --resolution 32 --crop-yield-scale 0.5 \
  --individual-demography --workshop-refinement --agriculture-refinement \
  --extraction-refinement --construction-refinement --compare-resolution \
  --household-diagnostics \
  --household-relief-share 0.05 --household-relief-target 0.75 \
  --output output/household-relief-base.json
```

The other arms change share/target to 0.20/0.75 (`household-relief-budget.json`)
and 0.20/0.90 (`household-relief-target.json`). Raw reports/logs remain ignored.
Concurrent GPU runs are used for behavioral comparison, not performance claims.

Every month contributes to food-gap and residual observations. Decadal fiscal
snapshots distinguish cumulative household relief receipts, wages and food spending
from current wallet, council and town cash. Council `relief_paid` includes both
household transfers and the older town-level bailout mechanism; it is not a second
measure of household receipts. These are abstract currency inventories and flows.

Matched long-run differences can involve many feedbacks. They do not identify a
single immediate mediator or establish that a policy is historically realistic.
The intended comparison is game balance, with food production held fixed as an input.

## Completed results (2026-09-12)

All six thirty-year histories complete with sixteen active sites, zero maximum
monthly population residual and maximum food residual below 2.6e-7.

| Seed | Policy (share / target) | Residents at 30y | Cumulative physical gap % | Cumulative access gap % | Household relief received | Council treasury at 30y |
|---|---|---:|---:|---:|---:|---:|
| 17 | 0.05 / 0.75 | 1,510 | 0.0456 | 3.1972 | 4,847.62 | 2,298.95 |
| 17 | 0.20 / 0.75 | 1,540 | 0.0454 | 3.1630 | 5,482.96 | 2,639.86 |
| 17 | 0.20 / 0.90 | 1,545 | 0.0452 | 3.1058 | 16,885.97 | 479.64 |
| 81 | 0.05 / 0.75 | 1,547 | 0 | 3.3144 | 3,439.30 | 2,123.33 |
| 81 | 0.20 / 0.75 | 1,528 | 0 | 3.3387 | 3,544.38 | 2,911.67 |
| 81 | 0.20 / 0.90 | 1,616 | 0 | 3.0722 | 14,439.94 | 651.63 |

Increasing the spending fraction fourfold does not quadruple actual relief:
requests remain capped at the target and depend on the council that controls each
household. The resulting trajectories have mixed outcomes. Raising the target at
the larger budget increases realized support substantially and reduces access gaps
in both seeds, but is not a population repair. High-target populations decline
from 1,994/1,952 at ten years to 1,545/1,616 at thirty years.

At year thirty, the high-target worlds contain 111,983.76/108,176.92 currency units
in household wallets. Their 63/61 households with more than 10% unmet food need
have only 0.00625/0.00669 combined wallet cash after retail, against 4,176/4,622 kg
monthly food-equivalent need. These endpoint observations do not establish which
families were chronically hungry. They do show why aggregate household wealth
cannot be treated as purchasing power available to each dependent family.

No defaults change. The next population work should examine earnings per dependent,
retained estates and finite family transfers, then compare with these relief and
common-entitlement controls. Further seeds and longer horizons remain necessary;
this result does not close the population balance gate.

## Verification and reproduction

The example builds and passes Clippy with warnings denied. CLI checks reject an
unpaired override, NaN and a target above one before GPU allocation. The two
comparison-script tests pass. Completed reports pass strict matched-metadata,
horizon, finite-value and food-gap decomposition checks with:

```sh
python3 scripts/compare_food_access.py output/household-relief-base.json \
  output/household-relief-budget.json --allow-difference household_relief_share_override
python3 scripts/compare_food_access.py output/household-relief-budget.json \
  output/household-relief-target.json --allow-difference household_relief_target_override
python3 -m unittest discover -s scripts -p test_compare_food_access.py
```

These changes add runner controls and observations only. They do not alter history
scheduling, household transfer rules or checkpoint format. The six runs exercise
current monthly integration and its runtime validation; they are not a new
checkpoint-equivalence or independent cash-conservation proof.
