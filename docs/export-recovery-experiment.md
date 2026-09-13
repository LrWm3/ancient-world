# Late-export recovery: paired balance protocol

The first balance comparison uses seed 409's existing founding checkpoint at
32 terrain and ecology cells per face, five civilizations and 600 starting people.
It advances 200 living-history years. Four standard arms (baseline, credit,
issuance, combined) explicitly disable export recovery. Two additional arms
repeat credit and combined with recovery enabled at the default 25% new-proceeds
allowance and 100-unit operating cash floor. All arms use delivery-paid exports.

Reproduce with:

```sh
cargo build --bin ancient-world
python3 scripts/monetary_experiment.py \
  --checkpoint 409=output/monetary-estates-heldout-founding/409.world \
  --years 200 --compare-export-recovery \
  --output output/monetary-export-recovery-paired
```

The checkpoint must already exist; its generation settings are recorded in
[the held-out report](monetary-estates-heldout.md). The runner copies the binary
before execution and records binary/checkpoint hashes locally. Do not overwrite
an existing output directory. This launch used revision `e2c0c08` with a clean
worktree. Concurrent shader tests make elapsed times unsuitable as benchmarks.

## Interpretation fixed before results

Compare each recovery arm with its matching credit or combined control first.
Report actual defaults, recovered principal and interest, and borrower/lender
cash alongside completed work, food access and population. No matching defaults
means no test of recovery's balance benefit; do not increase losses or lower
reserves simply to obtain positive activity. The controlled delayed-cargo fixture
separately verifies a case in which the mechanism executes.

If there are transfers, inspect their immediate cash and reserve effects before
attributing later demographic divergence to them. Original write-offs remain
visible even after full recovery. None of these comparisons by itself passes the
broader Stage 1 gate or justifies separate currencies.

Status: all six arms completed successfully. Raw output remains under ignored
`output/`.

## Interim regression check

Baseline, credit-only and issuance have completed successfully. A recursive
comparison with the prior shared-cost regression found only the newly added
`credit.recoveries` and `credit.export_recovery` fields; all pre-existing history
fields match. Baseline and credit-only still originate no loans. Issuance yields
115.27 people versus baseline 177.85, while reported completed workshop work rises
from 403.13 to 490.84. These mixed results reproduce the previous run. The final comparisons below supersede the pending status of the remaining arms.


## Completed results

| Arm | People | Completed workshop work | Need-weighted hunger | Loans | Defaults | Recovery |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Baseline | 177.85 | 403.13 | 0.02832 | 0 | 0 | 0 |
| Credit | 177.85 | 403.13 | 0.02832 | 0 | 0 | 0 |
| Issuance | 115.27 | 490.84 | 0.03879 | 0 | 0 | 0 |
| Combined | 140.23 | 492.29 | 0.02424 | 1 | 0 | 0 |
| Credit + recovery | 177.85 | 403.13 | 0.02832 | 0 | 0 | 0 |
| Combined + recovery | 140.23 | 492.29 | 0.02424 | 1 | 0 | 0 |

Both recovery histories are structurally identical to their corresponding controls
except for the policy's enabled flag. The single combined loan ends settled,
with a precision write-off of 0.00016061883223672524 currency units; this is not a
default and does not trigger recovery. Every issuance arm creates 1,250 units.

Across all six runs, maximum absolute managed relative residual is below 4.71e-5,
money residual below 2.77e-7 and ecological residual below 1.62e-5. All native
validation exits succeeded. Timings include concurrent verification and are not
performance comparisons.

**Decision:** retain recovery as an opt-in mechanism. This ensemble proves no
recovery benefit because there are no eligible defaults. Do not loosen underwriting
or manufacture a default merely to improve that headline. The controlled delayed
cargo fixture establishes the transfer mechanism; broader shock evaluation and
unfinished Stage 1 ownership/reporting work remain. Mixed issuance outcomes still
do not pass the Stage 2 gate.
