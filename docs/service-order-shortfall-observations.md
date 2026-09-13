# Service-order labor shortfall observations

Protocol: repeat the debt-free seed-1024, 32/32, five-civilization 50-year
procurement comparison at its existing 25% surplus share. Keep fees, wage policy,
underwriting, delivery-paid exports and issuance caps unchanged. Run the usual
baseline/credit/issuance/combined arms with procurement enabled in every arm.

The executable includes the new due-month labor observations. Compare each
history against the earlier 25% series after removing only the new execution
observation fields. A recording-only change should leave all other history equal.
Then count observed orders with no labor request, no funded labor, or funded
labor without completion. Report partial completion separately. These categories
can overlap; do not sum them as mutually exclusive causes.

Requested labor is already cash-capped. A zero request cannot distinguish absent
work demand from unaffordable wages; funded labor without output does not identify
which material, demand or capacity constraint bound. Inspect those boundaries
before changing policies. Old missing observations remain unknown, not zero.

Artifacts belong under ignored `output/service-order-observations/`. This run can
overlap constants verification, so elapsed times are not isolated benchmarks.
All four arms completed successfully. No new balance benefit is claimed.


## Results

Each history exactly equals the prior 25% reference after removing only the new
order execution fields. Credit issued no loans; the observations did not change
production, prices, transfers, population or existing event records.

| Arm | Observed orders | No request | No funded labor | Funded, zero completion | Partial contract completion | Labor below contract |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 381 | 0 | 0 | 58 | 281 | 208 |
| credit | 381 | 0 | 0 | 58 | 281 | 208 |
| issuance | 268 | 0 | 0 | 42 | 210 | 146 |
| combined | 268 | 0 | 0 | 42 | 210 | 146 |

Partial completion and labor-below-contract comparisons use a 1e-6 worker-month
reporting tolerance. Zero-request/funding/completion counts use recorded exact
zero. Categories overlap and are not additive causal attributions. All orders
in these runs have observations; no old archive gaps are being filled in.

Without issuance, operator labor observed at order boundaries totals 24.002408
requested, 24.002408 funded and 14.313232 completed worker-months, versus
29.573347 contracted worker-months. Operator completion can exceed an individual
contract: ordinary invoicing pays only the uncovered work. These sums must not
be treated as duplicate production or as the whole history's operator activity.

The largest absolute endpoint relative money residual is 1.44e-07. All native
commands exited successfully. As before, these are endpoints, not monthly maxima.

## Interpretation

The zero-output cases are not cases of entirely missing wage funding: every one
had positive funded labor. Many other orders also cover more work than the firm
actually funds. The grant/request totals show little aggregate loss between the
cash-capped request and funding stages here. This does not rule out cash limiting
the request earlier, nor identify why funded workers fail to complete output.

Inspection shows that procurement quotes lagged recipe demand, while operators
choose desired shifts from installed capacity and previously demonstrated work.
They are separate forecasts; an escrowed contract does not itself raise the
operator's desired shift or reserve next month's materials. This is a concrete
integration boundary to test next. Compare an explicit contract-aware shift
request against the current request with cash, worker capacity and inputs fixed;
separately inspect production/input constraints in the funded-zero cases. Do not
assume that larger loans or fees would fix either gap.

Keep procurement opt-in and Stage 2 currencies gated. The single-seed observation
screen narrows the investigation; it is not sufficient evidence to tune all worlds.

Executable SHA-256: `9035f24710a9d99aedacff3dec5c6f3d96d4d2a8f2b7a4b26eda244b3dcb7bdc`.
Source implementation: `40e1525`; subsequent spatial-constant edits were excluded
from the frozen executable.


Reproduction from a build of the indicated source:

```sh
python3 scripts/monetary_experiment.py \
  --binary target/debug/ancient-world \
  --checkpoint 1024=output/monetary-estates-heldout-founding/1024.world \
  --years 50 --service-order-procurement --service-procurement-share 0.25 \
  --output output/service-order-observations
```

Use a new output directory when repeating; the runner refuses to overwrite runs.
