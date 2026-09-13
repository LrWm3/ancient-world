# Monetary policies under reduced crop production

## Protocol fixed before results

Use seed 409's existing founding checkpoint at 32 terrain/ecology cells per face,
five civilizations and 600 initial people. Run 200 living-history years with
delivery-paid exports in all arms. Compare baseline, credit only, bounded issuance
only and combined at each of two explicit crop-yield settings: 0.5 (control) and
0.1 (severe sustained reduction). These are game-scale settings, not measured
agronomic yields. Hold every other starting setting and account balance fixed.

The override changes subsequent managed crop production, not opening food stocks,
cash or ecosystem inventories. The monthly production upload reads the changed
configuration. It does not change crop labor or nutrient availability directly;
actual yields still follow those limits. This is a sustained production constraint,
not a one-season failed harvest, and it does not cover every required shock fixture.

The runner records the override, revision and executable/checkpoint hashes locally,
then executes a fixed copy of the binary. Run from a clean committed build:

```sh
cargo build --bin ancient-world
python3 scripts/monetary_experiment.py \
  --checkpoint 409=output/monetary-estates-heldout-founding/409.world \
  --years 200 --crop-yield-scale 0.5 \
  --output output/monetary-crop-control
python3 scripts/monetary_experiment.py \
  --checkpoint 409=output/monetary-estates-heldout-founding/409.world \
  --years 200 --crop-yield-scale 0.1 \
  --output output/monetary-crop-scarcity
```

Report cumulative food-equivalent production, completed workshop work, terminal
need-weighted hunger, population, cash by account class, debt, default losses and
issuance. Actual crop harvest is a separate mediator: food-equivalent production
also includes other sources. Compare monetary arms within each yield setting
before comparing across settings. A later population difference alone does not
identify the causal channel.

A severe food shortfall despite increased cash is evidence that issuance cannot
replace physical production. Conversely, lower hunger is not sufficient evidence
of more food access if it results from population loss and lower terminal need.
No result here automatically passes the credit/issuance gate or justifies Stage 2.
The existing weak lending demand and mixed issuance results remain relevant.

## Initial boundary verification

Native one-year continuations of the same loaded checkpoint produced 50,196.85 kg
of managed crop harvest at 0.5 and 10,047.49 kg at 0.1. Food-equivalent production
was 43,230.60 versus 8,631.08. Both runs passed native validation. This confirms
that the loaded-world override reaches production; it does not yet establish the
long-run effect of any monetary policy. Both Python and native CLI rejected NaN,
and strict all-target Clippy passed. No default settings changed.
