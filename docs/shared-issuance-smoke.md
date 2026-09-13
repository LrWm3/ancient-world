# Shared issuance: first matched four-arm comparison

This is an accounting and toy-balance experiment, not economic calibration.
The fixed executable was built from `0aa337c`. Two founding checkpoints (seeds
17 and 81) each ran four arms for 30 years at terrain/ecology edges 32/32,
one geological epoch, one ecological year per interval and five civilizations.
Runs used the Quadro RTX 5000 Max-Q and default living history.

Delivery-paid exports are enabled in **every** arm. Thus the baseline is the
common payment experiment with both credit pilots and issuance disabled, not an
unchanged legacy dispatch-payment world. Credit enables both council and commercial
pilots. Issuance uses the initial ten-year authorization: 25 per council annually,
250 lifetime per council. All five issuers together create at most 1,250 units.

## Outcomes

| Seed | Arm | Ending population | Loans / defaults | Money issued |
| --- | --- | ---: | ---: | ---: |
| 17 | Baseline | 550.786 | 0 / 0 | 0 |
| 17 | Credit | 550.786 | 0 / 0 | 0 |
| 17 | Issuance | 548.203 | 0 / 0 | 1,250 |
| 17 | Combined | 548.203 | 0 / 0 | 1,250 |
| 81 | Baseline | 544.890 | 0 / 0 | 0 |
| 81 | Credit | 544.890 | 0 / 0 | 0 |
| 81 | Issuance | 562.903 | 0 / 0 | 1,250 |
| 81 | Combined | 562.903 | 0 / 0 | 1,250 |

Within each seed, baseline versus credit and issuance versus combined differ only
in their serialized credit records. No other History field differs. These runs
therefore show no realized credit effect, not a successful credit intervention.

| Seed / arm | Cumulative operator work | Cumulative reported food production | Ending need-weighted hunger | Population-weighted wheat quote | Cumulative council town support |
| --- | ---: | ---: | ---: | ---: | ---: |
| 17 baseline | 50.56 | 3,341,862 | 0.0246 | 3.72 | 3,316.06 |
| 17 issuance | 55.68 | 3,324,138 | 0.0277 | 3.21 | 4,025.70 |
| 81 baseline | 74.64 | 3,326,569 | 0.0211 | 3.97 | 5,066.56 |
| 81 issuance | 53.55 | 3,370,980 | 0.0212 | 3.92 | 5,691.65 |

Operator work sums firms' completed work, not all work. Food production is the
existing food-equivalent ledger, not harvested wheat alone. Hunger weights the
last month's household hunger by household need; it is not cumulative famine.
Wheat prices are posted quotes, not transaction prices or a cost-of-living index.
Town support is `Council.relief_paid`: annual emergency council-to-town transfers,
not a measure of all household relief.

Ending council treasuries increase from 3,857.11 to 4,965.06 in seed 17 and from
2,968.48 to 3,464.90 in seed 81. This is an account-stock comparison, not tracking
which particular issued coins remained there. Other flows also change.

Issuance increased town support in both seeds, but population, food access and
work did not improve consistently. Seed 17 lost 2.58 people relative to baseline;
seed 81 gained 18.01. Lower posted wheat quotes alone did not ensure lower hunger.
These outcomes do not pass the design's benefit gate or justify enabling minting
by default. Longer runs, held-out seeds and controlled loss cases remain needed.

## Verification and reproduction

All eight native runs completed validation. Relative monetary residual magnitude
was at most 2.66e-7; other tracked managed residuals reached 9.03e-6. These are
numerical accounting observations, not proof of sensible calibration. Seed 17's
issuance run also matched exactly, across the complete exported History, when
split into 15 years, native save/load, then 15 years. The resumed command omitted
policy flags, exercising archived settings.

The reusable runner was smoke-tested through all four one-year arms. Build the
binary first, then use existing founding checkpoints, for example:

```sh
cargo build
python3 scripts/monetary_experiment.py \
  --checkpoint 17=output/credit-council-smoke/founding.world \
  --checkpoint 81=output/credit-commercial-century/seed81.world \
  --years 30 --output output/monetary-repeat
```

The output directory must be new. The runner copies the executable, records its
checksum and checkpoint checksums, runs serial arms and stops on native failure.
It records the repository revision and tracked changes; those do not by themselves
prove an existing executable was built from that revision. Raw histories, logs and
metadata remain ignored under `output/`. The checkpoint paths above are local
experiment inputs, not files distributed in the source repository. These short
runs are not a performance benchmark, a resolution study or a held-out evaluation.
