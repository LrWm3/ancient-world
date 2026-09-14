# Local food solidarity experiment

This is a scoped game policy for testing food affordability without removing town
food-sale revenue. It does not solve physical food shortages or establish a
historically calibrated welfare model.

Enable with `--food-solidarity=true`, disable with `--food-solidarity=false`.
New histories default to disabled; omission preserves the archived setting.
The independent `--gradual-nutrition` experiment can be combined with it.

## Monthly contract

At Reserve, household retail preparation applies existing family support and
council relief first. It then collects local household food-budget gaps and
surplus-wallet allowances together, before recomputing paid food entitlements.
Execute/settle retains ordinary finite-food consumption and retail payments.

A contributing household retains three months of its full dietary budget at the
current local quote. At most 5% of cash above that reserve contributes each month.
Only households in the local retail plan with positive dietary need contribute.
Recipient need excludes already-common food and food affordable from their wallet.
Contributions and assistance are proportional to eligible allowances and gaps.

The transfer is capped by all three of:

- Total remaining household food-budget gaps.
- Total donor allowances after protected savings.
- Opening local food stock beyond already-funded entitlements, valued at the
  current food quote and capped by total dietary need.

No future harvest or incoming shipment is assumed available. Transfers stay within
the settlement. No food or money is created. This is an explicit contribution
policy, not a model of voluntary charitable preferences. It does not exempt debtors
or predict other future spending obligations beyond the protected food reserve.

Transferred money becomes the recipient's cash, not an escrow. Subsequent ordinary
retail pays the town for actual food. If execution cannot fulfill the entitlement,
unspent money remains with the recipient. The policy therefore supports access;
it does not guarantee consumption or continuous town funding.

## Accounting and persistence

Account counters distinguish `solidarity_sent` and `solidarity_received` from
wages, relief, purchases and production. Both enter wallet conservation; their
world totals must balance. The circulation audit reports them as cumulative flows,
never additional money stocks. The archived policy retains its last processed
month and per-site requests, donor allowance, stock-backed ceiling and transfer.
A repeated call in that month cannot collect again. Old archives default to zero
counters and a disabled policy.

## Verification and balance screen

Allocation fixtures cover finite cash, protected reserves, absent residents, no
available food, fully funded diets, stock ceilings, household ordering, repeated
monthly calls and serialized policy continuation. These are allocation fixtures,
not a claim that a complete GPU world resumes identically across all backends.

### Matched century results

Seeds 1024, 256 and 409 start from the same five-town, 600-person founding archives
as the [food access/health screen](food-access-and-nutritional-stress.md).
Terrain/ecology are 32/32; environment frozen; aggregate demography. Both arms use
gradual nutrition, needs-based food disabled, wealth tax, council welfare reserves,
clothing and practical research. Credit and issuance are disabled. Only solidarity
changes. The control is the previous health-only export at `bbc6b22`.

| Seed | Policy | Final population | Active sites | Endpoint need-weighted hunger | Town cash | Completed operator work |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | Off | 137.31 | 2 | 0.0259 | 4,813 | 88.82 |
| 1024 | On | 122.28 | 2 | 0.0110 | 4,975 | 53.42 |
| 256 | Off | 297.47 | 4 | 0.0560 | 9,844 | 8.91 |
| 256 | On | 287.61 | 4 | 0.0049 | 12,276 | 13.65 |
| 409 | Off | 257.52 | 3 | 0.0232 | 9,328 | 251.43 |
| 409 | On | 257.76 | 3 | 0.0281 | 8,682 | 368.34 |

Lifetime solidarity transfers are 11,764 / 33,357 / 28,561 currency units,
respectively. Sender/recipient counter totals differ by less than 4e-11.
Those cumulative flows can reuse money; they are not extra currency stocks.
Household food payments are 1.765 / 3.368 / 3.207 million, versus control
1.831 / 3.173 / 3.003 million. Town receipts therefore remain substantial,
unlike the earlier fully common food arm that eliminated food-sale revenue.

This does not establish population recovery. Births are 521.2 / 981.2 / 902.4
versus 541.1 / 993.4 / 905.6; deaths are 998.9 / 1293.6 / 1244.7 versus
1003.8 / 1295.9 / 1248.0. Lower cumulative deaths are not proof of a lower
mortality rate, because population exposure also changes. Endpoint hunger is not
lifetime deprivation. Long nonlinear histories cannot identify why births fell
from endpoint cash and food alone.

All three runs pass the existing monthly validation. Independently audited endpoint
money residuals are below 3.6e-7 relative (under 0.018 in a 50,000-unit money stock).
No conservation tolerance was changed. Ordinary verification: 204 library tests
pass, 156 extended/hardware tests remain ignored; ten circulation-audit tests pass;
strict all-target Clippy passes. Native runs exercise GPU production and household
settlement, but are not a full extended GPU suite, cross-hardware comparison, or
maximum-population benchmark.

Keep this policy opt-in. It successfully separates funded access from free
entitlement, but final population remains below the original 600 in every seed.
The next demographic investigation should measure age-specific exposure, births,
nutrition and displacement over shorter matched windows. Do not increase transfer
rates solely to improve the final population count.

### Reproduction

Using an equivalent founding archive, run the following twice, changing only
`--food-solidarity` and the export path. Repeat seeds 1024, 256 and 409. The stored
local founding archives and raw results are deliberately ignored artifacts, so
regenerating a different founding archive is a new comparison baseline.

```sh
target/debug/ancient-world --headless --epochs 0 --history-years 100 \
  --load output/monetary-estates-heldout-founding/1024.world \
  --history-export output/food-solidarity-screen/1024.json \
  --food-solidarity=true --needs-based-food=false --gradual-nutrition=true \
  --council-welfare-reserves=true --household-wealth-tax=true \
  --practical-research=true --household-clothing=true --delivery-paid-exports \
  --commercial-credit=false --service-order-procurement=true \
  --contract-workshop-staffing=true --demand-workshop-staffing=true \
  --household-estate-inheritance=true --household-estate-reclamation=false \
  --named-office-service=true --abandoned-stock-recovery=true \
  --service-order-credit=false --council-credit=false --shared-issuance=false \
  --service-procurement-share 0.25
CARGO_INCREMENTAL=0 cargo test --lib
PYTHONPATH=scripts python3 -m unittest scripts/test_audit_circulation.py
cargo clippy --all-targets -- -D warnings
```

Disabled-policy regression: a fresh seed-1024 century run matches the entire
previous health-only history export after removing only the new default policy
and zero transfer counters. This is one same-backend regression, not an assertion
of universal bitwise portability.

The subsequent [demographic window audit](demographic-window-audit.md) finds that
physical food shortages already drive heavy losses in years two and three, before
purchasing gaps become substantial. It records the mediators at the GPU boundary
instead of inferring early causes from late surviving households.
