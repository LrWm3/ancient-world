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

## Execution provenance

The eight-arm comparison was launched with the executable built from `ef46feb`.
Both batches use the same fixed executable: the scarcity batch copies the control
batch's executable, not a subsequently rebuilt target. Constants review continues
while these runs execute, so the second runner's recorded Git revision/working-tree
state can describe later source edits. Compare the recorded binary hashes; those
identify the actual shared executable. Elapsed times are not performance benchmarks
while shader tests and compilation run alongside the experiments.

## Completed 200-year comparison

All eight runs completed with native validation enabled. Binary and checkpoint
hashes match between batches. The control batch records revision `ef46feb`; later
constants edits do not enter either executable. Results are terminal or cumulative,
not estimates of a universal causal effect.

| Yield | Arm | Ending people | Crop harvest, kg | Completed workshop work | Terminal hunger | Loans |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 0.5 | baseline | 177.847 | 12,928,696.50 | 403.131 | 0.02832 | 0 |
| 0.5 | credit | 177.847 | 12,928,696.50 | 403.131 | 0.02832 | 0 |
| 0.5 | issuance | 115.272 | 12,568,895.33 | 490.844 | 0.03879 | 0 |
| 0.5 | combined | 140.234 | 12,634,890.99 | 492.293 | 0.02424 | 1 |
| 0.1 | baseline | 0.000 | 184,374.37 | 8.906 | N/A | 0 |
| 0.1 | credit | 0.000 | 184,374.37 | 8.906 | N/A | 0 |
| 0.1 | issuance | 0.000 | 184,341.56 | 8.776 | N/A | 0 |
| 0.1 | combined | 0.000 | 184,341.56 | 8.776 | N/A | 0 |

Each issuance-enabled arm issued exactly 1,250 units under the existing cap.
No arm defaulted, recovered a default loss or assigned an estate claim. The single
control-combined loan ended in precision settlement with a 0.00016061883223672524
write-off; that remains separate from insolvency. These worlds therefore do not
exercise estate succession or demonstrate useful credit under harvest stress.

The severe baseline and credit runs match on the reported outcomes, as do severe
issuance and combined runs. All five sites are abandoned in each severe arm; the
last recorded abandonment is month 358. Terminal food need is zero, so the runner
reports hunger as unavailable. It must not be interpreted as successful famine
relief. Retained household cash is about 49,995.64 units without issuance and
51,245.64 with issuance; ending council cash is zero. These are retained balances
after population loss, not evidence that surviving buyers could afford food.

The first-year controlled mediator check above establishes an immediate reduction
in harvest when yield is reduced. The much larger cumulative reduction over two
centuries also includes fewer workers and years of abandoned production. Capped
issuance did not prevent collapse in this scenario. This does not prove every
possible redistribution or issuance policy would fail, nor separate production
and access constraints in every intervening month.

The control reproduces the previous mixed issuance outcome: more workshop work,
but lower ending population and food production. Credit-only still originates no
loans. The conclusion remains **no-go for Stage 2 on present evidence**. Further
work should target causal financing opportunities, temporary/lost-receipt shocks
and living household food access, rather than interpreting extra currency or a
lower terminal hunger statistic as success.

The maximum absolute terminal relative money residual was 2.76e-07; this
is an endpoint measure, not a recorded maximum over every monthly boundary.
