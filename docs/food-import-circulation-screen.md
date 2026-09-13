# Food import sizing and protected reserves

## Question and decision

Following the stored-ore experiments, test whether larger food imports connect
existing money to food access and useful workshop work. This is game-balance
screening, not a claim about historical markets.

The quarterly market caps each purchase at five kilograms per resident, including
food. The modeled civilian reserve ration is 18 kg per resident per month. The
candidate replaced the food ceiling with one quarterly interval, or 54 kg per
resident, while retaining five for other goods. Existing buyer need, pending
cargo, storage, seller surplus, cash, freight, routes and arrival timing still
apply. This is an upper bound, not a guaranteed purchase.

A second comparison used the candidate with either twelve or six months of seller
food reserves. Six is the existing catalog's minimum allowed value. No new money,
private-wallet access, free freight, ports or material production were added.

**Do not adopt either change as a default circulation fix.** The larger-order
candidate was rolled back. The reserve experiment only changed ignored catalogs;
the source catalog remains unchanged. The capacity limitation is real, but relaxing
it alone did not produce a broadly useful economy.

## Matched runs

Baseline: `bdedba7`, using the existing recovered-ore work screen. Three seeds,
1024, 256 and 409, each run for 600 monthly boundaries from the same founding
archives, with terrain/ecology resolution 32/32 and no geological advancement.
Credit and issuance are disabled. Delivery-paid exports, service procurement,
contract/demand workshop staffing, estate inheritance, named office service and
abandoned-stock recovery are enabled; estate reclamation is disabled. Service
procurement share is 0.25. Adaptive prices and network trade remain enabled.

Three runs changed only the food transaction ceiling. Six further runs paired
seller reserves of twelve and six months. Both arms reload the same complete
archived economy catalog through `--economy-catalog`, differing only in
`market.food_reserve_months`. This controls for the catalog-update event and
configuration-loading path. Nullable catalog fields are omitted from TOML so
serde restores their original absent value. Generated catalogs were parsed back
and compared to their source values before execution.

Command shape (substitute seed and local output paths):

```sh
ancient-world --headless --epochs 0 --history-years 50 \
  --load output/monetary-estates-heldout-founding/1024.world \
  --delivery-paid-exports --commercial-credit=false \
  --service-order-procurement=true --contract-workshop-staffing=true \
  --demand-workshop-staffing=true --household-estate-inheritance=true \
  --household-estate-reclamation=false --named-office-service=true \
  --abandoned-stock-recovery=true --service-order-credit=false \
  --council-credit=false --shared-issuance=false \
  --service-procurement-share 0.25 --history-export output/example.json
```

For paired reserve arms append `--economy-catalog output/paired-catalog.toml`.
Reproducing the rejected candidate also requires replacing only the market's
food purchase ceiling with `population * 18 * 3`; shipped code retains five.

## Results

Seed 1024 distinguishes the arms:

| Metric | Baseline | Larger food orders | Larger orders + six-month reserve |
| --- | ---: | ---: | ---: |
| Ending population | 159.871 | 157.242 | 152.943 |
| Ending need-weighted household hunger | 0.03695 | 0.05058 | 0.04372 |
| Cumulative completed operator work | 11.219 | 12.957 | 26.061 |
| Cumulative operator operating margin | 47.42 | 50.60 | 126.84 |
| Recorded food dispatches | 94 | 53 | 123 |
| Recorded food dispatched, kg | 8,844.9 | 7,841.3 | 11,629.2 |
| Recorded food sale value | 12,716.6 | 11,525.5 | 18,141.3 |

Dispatch quantities and values are sums of rounded historical event descriptions,
not an exact material or cash ledger. They show committed sales, not necessarily
delivered or eaten food. Operator margin excludes capital and financing. Ending
hunger alone does not capture the preceding fifty years of hardship.

The first differing event in the larger-order comparison is month 60, site 0
selling food to site 4: 243.4 kg for 378.1 becomes 442.9 kg for 687.8. Both arrive
at month 66. This establishes the immediate intervention: more cash and freight
committed to delayed food. It does not by itself establish the cause of every
later population difference. In particular, larger permitted batches ultimately
produced fewer dispatches and less cumulative food trade in this seed.

Seeds 256 and 409 had no recorded ordinary food sales in any arm. Their
population, hunger, operator work and margins were unchanged across all arms:
337.571 / 0.03323 / 4.500 / 18.50 for seed 256 and
352.819 / 0.05919 / 41.532 / 134.22 for seed 409. The twelve-month
catalog-loading controls reproduced the larger-order arm's reported metrics on
all three seeds. This rules out the catalog-loading step as an explanation of
these measured differences, not every possible hidden-state difference.

All nine new fifty-year runs completed successfully. Across them, absolute
relative monetary residual stayed below 1.29e-7. These are conservation checks,
not evidence that the revised economy is well balanced.

## Verification and remaining work

Before rollback, two focused tests passed: interval sizing and a GPU-founded
market fixture varying cash, freight and decision month. The latter checks finite
source/cash transfers, no usable food before arrival, and identical serialized
continuation after loading in-transit cargo. The candidate's ordinary library
suite passed 195 tests with 151 hardware tests ignored; the focused hardware test
was separately executed. Native build and strict library Clippy passed.

Local outputs and rejected source patch are under ignored
`output/food-purchase-interval-screen/` and `output/food-reserve-screen/`.
Only this summary is committed. Frozen binaries were used for the ensembles;
runtime is not reported as an isolated performance benchmark.

The next implementation should connect a real paying customer to a useful,
deliverable order. Household-backed food imports need explicit authorization,
ownership/entitlement, payment and failure/refund accounting; household savings
must not silently become unrestricted town cash or be charged twice on delivery.
Before introducing that mechanism, record why import requests fail at their actual
monthly boundary: no eligible surplus, no route, occupied freight, unavailable
cash, storage, or already-pending supply. End-state balances cannot distinguish
these causes. Ports and workshop inputs remain complementary constraints, and
unworked abandoned deposits still need an actual extraction workforce.
