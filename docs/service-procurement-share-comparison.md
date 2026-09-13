# Procurement share comparison

Status: all eight arms completed successfully. The protocol below was fixed
before runs. Reducing the envelope barely changes activity or food access; the
policy remains opt-in and the Stage 2 currency gate remains no-go.

Use debt-free seed 1024 at 32/32 for 50 years, five civilizations, delivery-paid
exports and unchanged yield. Run all four monetary arms at surplus shares 0.10
and 0.25 with procurement enabled. Keep the cash floor, input reserve, service fee,
underwriting and issuance caps unchanged. Both series use one fixed executable.

Compare cumulative completed operator work, fees earned/refunded, outstanding
escrow, actual loans, crop harvest and terminal food access. Check the 0.25 series
against the prior screen for unintended changes. The smaller share is promising
only if it improves completed work and affordability rather than merely reducing
funding or changing terminal population. Mixed results retain the opt-in policy
and do not advance the currency gate.

This remains a single-seed parameter screen, not held-out validation or a causal
account of monthly cash shortages. Boundary cash observations and additional seeds
are still needed. Constants verification may run concurrently, so elapsed times
are not isolated performance measurements. Raw outputs belong under ignored
`output/service-procurement-share/`.

Reproduction (repeat with `0.25` and a distinct output directory):

```sh
python3 scripts/monetary_experiment.py \
  --binary output/service-procurement-share/ancient-world \
  --checkpoint 1024=output/monetary-estates-heldout-founding/1024.world \
  --years 50 --service-order-procurement --service-procurement-share 0.10 \
  --output output/service-procurement-share/low
```

## Results

All four 25% reference histories exactly equal their counterparts in the earlier
[screen](service-procurement-screening.md), including their credit records. Within
each new series, credit versus baseline and combined versus issuance histories
are exactly equal after removing only the credit records. No loans issued.

| Surplus share | Issuance | Cumulative operator worker-months | Terminal need-weighted hunger | Fees funded | Fees earned | Fees refunded |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10% | Off | 19.519026 | 0.05884939 | 1580.7754 | 682.9594 | 897.8069 |
| 25% | Off | 19.519026 | 0.05884938 | 1582.0354 | 682.9594 | 899.0669 |
| 10% | On | 17.436962 | 0.06844020 | 1146.2563 | 575.0660 | 571.1851 |
| 25% | On | 17.436962 | 0.06844271 | 1148.3792 | 576.9221 | 571.4520 |

Without issuance there are 381 orders in either series, 58 with zero completed
work. Only the first order changes funding by more than 0.0001: at month 29 its
funding falls from 4.3265 to 3.0665, while its earned fee stays about 1.4684. The
remaining reduction is predominantly a smaller refund, not more completed work.
With issuance there are 268 orders; zero-work orders increase from 42 to 43 at
the smaller share. The first two orders account for almost all the funding
reduction. Small later differences are not evidence of a broad improvement.

Operator work in the table includes ordinary invoicing as well as orders; it is
not the same measure as work attributed to escrowed orders. That latter measure
falls from 12.1658 to 12.1203 worker-months with issuance at the smaller share.
Do not infer a useful credit bridge from reserved fees or terminal population.

The largest absolute endpoint relative money residual is 1.80e-7. Residuals are
from the managed ledger's money component (index 4); these are endpoints, not
monthly maxima. Small residual escrow remains counted as money after settlement.
All native commands exited successfully. The new option also passed two CLI tests,
eleven Python reporting tests, strict all-target Clippy and the native build.

## Decision

Keep the default share unchanged and procurement opt-in. This screen does not
support the hypothesis that the 25% envelope is the main cause of the earlier
mixed balance result. It does not prove that escrow timing never matters: the
measurements are single-seed cumulative/endpoint observations. Next investigate
why a funded order lacks completed work, distinguishing workers, wages, materials,
production demand and competing commitments before changing allocation policy.
Then use boundary observations and additional seeds to test the identified cause.
Do not relax underwriting or raise service fees to manufacture credit activity.

Executable source revision: `070a87f` (before the subsequent calendar extraction).
Executable SHA-256: `0d047ae53f1850fe6d0c04ade80efa94a278b40771badb780e64bf18d270512c`.
The checkpoint is the same one identified in the earlier screening report.
