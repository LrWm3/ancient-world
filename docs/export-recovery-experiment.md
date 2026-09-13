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

Status: launched; results pending. Raw output remains under ignored `output/`.

## Interim regression check

Baseline, credit-only and issuance have completed successfully. A recursive
comparison with the prior shared-cost regression found only the newly added
`credit.recoveries` and `credit.export_recovery` fields; all pre-existing history
fields match. Baseline and credit-only still originate no loans. Issuance yields
115.27 people versus baseline 177.85, while reported completed workshop work rises
from 403.13 to 490.84. These mixed results reproduce the previous run. The combined
and recovery-enabled arms are still pending; no recovery balance conclusion yet.
