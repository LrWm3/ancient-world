# Commercial credit: first two-seed comparison

This is an integration and game-balance experiment, not historical calibration.
The tested executable was built from `028d98a` and copied under ignored `output/`
before other work continued. Both arms enable delivery-paid exports. Only
`--commercial-credit` changes; council credit remains disabled.

## Settings and results

Quadro RTX 5000 Max-Q; terrain/ecology edges 32/32; one geological epoch and one
year per ecological interval; five starting civilizations; default living-history
systems. Seed 17 reuses the founding checkpoint from the council smoke experiment;
seed 81 starts from a new founding checkpoint. Each arm loads the same checkpoint
for its seed and advances 200 years.

| Seed | Ending population, both arms | Credit requests | Loans | Relative money residual, both arms |
| --- | ---: | ---: | ---: | ---: |
| 17 | 172.8688493 | 0 | 0 | 2.19e-8 |
| 81 | 266.292342 | 4 | 0 | -9.68e-8 |

Serialized sites, people, events, cargo and delivery payments are identical between
the two arms for each seed. All runs completed their validation. Other tracked
relative residuals reached approximately 5.61e-5; monetary correctness does not
establish ecological calibration. Elapsed runs took 69–79 seconds, with concurrent
compilation during parts of the experiment; these are not benchmark comparisons.

## Why the requests were rejected

Seed 81's four requests came from town 0 in months 22, 23, 24 and 28. Each named
the same approximately 16.02 risk-adjusted receivable due in month 29. Quoted
input commitments ranged from 1,031.84 to 1,328.94, while cash shortfalls ranged
from 23.93 to 741.95. The lender offered 141.99–234.39 above its protected reserve.
The rejection was therefore insufficient net repayment evidence (`NoCapacity`),
not absence of lender money. Relaxing lender reserves would not fix this case.

Neither world demonstrates a population benefit from credit. The severe population
decline also occurs without credit and is not caused by new debt in these runs.
This does not prove that bridge credit is useless: it shows that this particular
receivable/working-cash policy did not identify a financeable gap in these worlds.
At the final boundary, neither seed had unresolved delivery payments and all town
cash balances exceeded the current input-cost quote.

## Reproduction and remaining checks

Build the tested revision, create or retain a founding archive, then run both arms:

```sh
cargo build
# Repeat with false and true, using distinct output filenames.
target/debug/ancient-world --headless --load output/founding.world --epochs 0 \
  --history-years 200 --delivery-paid-exports --commercial-credit=false \
  --history-export output/delivery.json
target/debug/ancient-world --headless --load output/founding.world --epochs 0 \
  --history-years 200 --delivery-paid-exports --commercial-credit=true \
  --history-export output/commercial.json
```

Record the checkpoint's seed, catalogs and generation settings; a different
founding baseline is a different experiment. Raw results and the fixed executable
remain ignored local artifacts. This report does not cover held-out seeds,
500-year runs, minting, currency exchange, combined council/commercial credit,
funded-work attribution, or controlled harvest/tax/cargo shocks. Those remain
required before the design's progression gate can be evaluated.

A subsequent boundary fix declines automatic requests beyond the shared maximum
loan duration instead of rejecting the whole monthly update. Its controlled test
uses a 121-month payment and passes; these ensemble runs used the earlier fixed
executable and did not encounter that failure.
