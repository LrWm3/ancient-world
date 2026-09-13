# Credit event regression: seed 409, 200 years

Revision `9b21884` was built from a clean worktree and run from the same founding
checkpoint as the [late-export comparison](export-recovery-experiment.md). The
runner copied its binary before subsequent constants edits. All six arms completed
with native validation enabled. Outputs remain ignored under
`output/monetary-credit-chronicle-regression/`; this is a correctness comparison,
not a performance benchmark.

```sh
python3 scripts/monetary_experiment.py \
  --checkpoint 409=output/monetary-estates-heldout-founding/409.world \
  --years 200 --compare-export-recovery \
  --output output/monetary-credit-chronicle-regression
python3 scripts/compare_credit_chronicle.py \
  output/monetary-export-recovery-paired/409-combined.json \
  output/monetary-credit-chronicle-regression/409-combined.json
```

The checkpoint generation requirements are documented in the linked earlier
report. Use a fresh output directory and the intended source revision when
reproducing a historical comparison.

| Arm | Ending people | Completed workshop work | New credit events |
| --- | ---: | ---: | ---: |
| Baseline | 177.84691 | 403.1308277 | 0 |
| Credit | 177.84691 | 403.1308277 | 0 |
| Issuance | 115.272491 | 490.8440965 | 0 |
| Combined | 140.23437 | 492.2930996 | 2 |
| Credit + recovery | 177.84691 | 403.1308277 | 0 |
| Combined + recovery | 140.23437 | 492.2930996 | 2 |

Every arm matches its earlier complete exported history after narrowly normalizing
new loan events, the new loan-event index, and explicit event references. The
comparison retains every other field, including cash, debt, food, culture and
production. It maps only known reference paths, the governance processed-event
cursor, and event IDs embedded in civic-petition prose; arbitrary numeric values
are not normalized. Changes in unrecognized fields fail the comparison. Deliberate
changes to population, event prose and event month were rejected.

In both combined arms, loan 0 transfers 2.023681640625 units at month 698 and
reaches precision settlement at month 713. Its second event references its first.
The original 10,665 events become 10,667; all original event content matches after
reference normalization. The 0.00016061883223672524 rounding write-off remains
separate from default. There are no defaults or recoveries. Each recovery-enabled
arm also matches its current control exactly except for the enabled policy flag.

This verifies the observed chronicle addition did not change these trajectories.
It does not prove that event insertion is universally behavior-independent, nor
that credit or recovery improves balance. The previous mixed issuance results and
unpassed Stage 2 gate remain unchanged. Broader ownership, causal funding and shock
evaluation work remains open.
