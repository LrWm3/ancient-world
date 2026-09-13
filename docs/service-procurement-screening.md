# Automatic procurement: 50-year screening protocol

Status: all eight arms completed successfully. This single-seed screen is mixed;
procurement remains opt-in and the Stage 2 benefit gate remains no-go.

Use the existing debt-free seed-1024 founding checkpoint at terrain/ecology
resolution 32/32, five civilizations, for 50 years. Hold delivery-paid exports,
starting inventories and crop yield settings constant. Run the four standard
monetary arms with procurement enabled, then the same four with it disabled.
Service-order lending follows the credit switch only in the procurement series.
Both series explicitly set procurement and service-lending flags so the checkpoint
cannot silently choose them.

The executable is a copy of the native build for `02b74c3`, taken before the next
constants pass. The runner records its SHA-256 and the checkpoint checksum locally.
Commands:

```sh
python3 scripts/monetary_experiment.py \
  --binary output/service-procurement-screen/ancient-world \
  --checkpoint 1024=output/monetary-estates-heldout-founding/1024.world \
  --years 50 --service-order-procurement \
  --output output/service-procurement-screen/with

python3 scripts/monetary_experiment.py \
  --binary output/service-procurement-screen/ancient-world \
  --checkpoint 1024=output/monetary-estates-heldout-founding/1024.world \
  --years 50 --output output/service-procurement-screen/without
```

Assess the immediate mediators first: orders funded, fees earned versus refunded,
remaining escrow, operator completed work and actual loans. Then compare food
harvest separately from terminal household affordability, council/institutional
shortfalls, population and money residuals. Endpoint measures must not be described
as monthly maxima. More fee commitments alone are not a benefit.

This is an initial integration screen, not a held-out multi-seed benefit gate.
Failures or regressions should trigger a specific investigation, not immediate
fee increases or reserve reductions. Concurrent constants verification shares the
GPU, so elapsed times here cannot support isolated performance claims. Generated
archives/logs stay under ignored `output/service-procurement-screen/`.


## Results

Credit issued no loans in either series. Within each series, baseline versus
credit and issuance versus combined histories are exactly equal after removing
only the credit records. The four distinct outcomes are:

| Procurement | Issuance | Population at year 50 | Cumulative operator work | Terminal need-weighted hunger | Active institutions | Service orders |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| without | Off | 156.173 | 18.1846 | 0.07125 | 6 | 0 |
| with | Off | 160.775 | 19.5190 | 0.05885 | 8 | 381 |
| without | On | 157.640 | 18.8257 | 0.06133 | 8 | 0 |
| with | On | 157.701 | 17.4370 | 0.06844 | 8 | 268 |

Operator work is measured in worker-months; hunger is the reported terminal index,
not an annual average or the worst shortage. With no issuance, procurement
increases work from 18.1846 to 19.5190 and lowers terminal hunger from 0.07125 to
0.05885. With issuance, it reduces work from 18.8257 to 17.4370 and raises hunger
from 0.06133 to 0.06844. Population alone would obscure that latter regression.

Managed crop harvest differs little: 2,697,314 versus 2,697,618 kg without issuance,
and 2,708,758 versus 2,711,710 kg with issuance (ordinary invoicing versus
procurement). These harvest totals must not be confused with food affordability
or taken as evidence that additional food production explains the final outcomes.

The no-issuance procurement series funds 1,582.0354 in fees, pays 682.9594 and
refunds 899.0669. With issuance it funds 1,148.3792, pays 576.9221 and refunds
571.4520. Thus many reserved fees are never earned. No orders remain pending at
year 50, but settled orders retain aggregate refund dust of 0.00910 and 0.00521
respectively, which remains in the money inventory. Neither value is unpaid
operator revenue. Issuance arms create exactly 1,250 under the existing caps.

All native commands exit successfully. The largest absolute endpoint relative
money residual (index 4 of the managed ledger) is 1.44e-7. These endpoints are not
monthly residual maxima. The positive/negative production, replay and conservation
fixtures documented separately remain necessary implementation evidence.

## Interpretation and next checks

Automatic contracts now exist in ordinary history and release real fees for
completed work. This is a useful integration result. It is not a broad benefit
claim: one seed exhibits a sign reversal when issuance is added, more than half
of the no-issuance escrow is refunded, and service credit still finds no supported
loans. The funding-feasibility check prevents the previous misleading approvals.

Next, examine when procurement ties up cash versus when protected payment sustains
work, using shorter boundary snapshots and a smaller procurement-share comparison.
Then run additional seeds and longer intervals with the same controls. Do not
relax underwriting or increase fees merely to produce loan activity. Separate
currencies remain conditional on evidence the simpler monetary mechanisms help.

Executable SHA-256: `29fef6996042b35058e4a6ef5d281e9612567c526a3320c6c431c3d2b75c20eb`.
Checkpoint SHA-256: `a9ab4d1b36b1759bb42ab62c11adfd238d64f03cf834252c5c8e2ae53934e3c2`.
The experiment binary predates the subsequent constants extraction; the captured
checksums, not the later worktree revision, identify the tested executable.
