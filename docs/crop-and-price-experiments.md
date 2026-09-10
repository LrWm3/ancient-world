# Crop processes and price formation: experimental alternatives

The default agriculture and economy are game models. Their outputs are not
validated estimates of ancient crop yields, wages, prices or population growth.
Conserving nutrients and money establishes accounting correctness, not agronomic
or historical accuracy. Base prices, policy shares and crop coefficients remain
explicit design assumptions.

Subsequent work changed [shared crop-resource allocation](crop-resource-competition.md)
in both modes. The historical comparison below predates that change.

This increment adds two independently selectable alternatives. **Neither is enabled
by default.** Controlled comparisons exposed severe crop underproduction and large
staple-price drift. Keeping them experimental is an evaluation result, not evidence
that the default is scientifically correct.

## Seasonal crop processes

`assets/agriculture-seasonal.toml` adds a `season` record to each crop. The GPU uses
the existing monthly calendar and crop inventories:

1. Six months before the crop's harvest month, transfer existing seed to standing
   crop. Seed in storage cannot support growth.
2. Multiply potential growth by a seven-month canopy proxy
   `[0.2, 0.6, 1, 1, 0.8, 0.4, 0.1]`. Growth is zero outside this season.
3. Apply the existing temperature response, then cap growth by available soil N/P
   and stored water. The old additional rainfall-suitability multiplier is omitted
   in this mode; rainfall still enters the water inventory upstream.
4. Reproductive-phase water shortage damages standing crop; frost also damages it.
   Move the lost C/N/P into detritus, without resetting fertility.
5. At harvest, return the non-harvested fraction to detritus, retain finite seed and
   transfer the remaining yield into goods.

Standing crop and seed are kg of crop-equivalent material, with composition supplied
by the existing goods catalog. Available water is m³; water consumption remains
`water_m3_kg × growth`. Carbon uptake is a declared atmospheric import. N/P uptake
subtracts existing soil inventories. Residue is not a new nutrient source.

The experimental preset replaces the tuber-specific `yield_scale = 6` with 1 and
normalizes growth by `0.45 / crop_carbon_fraction`. This makes fresh-mass differences
explicit in the carbon budget, but does **not** establish measured yields. Other
crop multipliers remain. Harvest index 0.65, reproductive damage coefficient 0.30
and frost coefficient 0.5 are provisional hypotheses, identical across crops.

FAO AquaCrop connects canopy, transpiration, biomass and harvest index, and distinguishes
water-stress responses. Those are useful mechanisms to borrow, not a claim that this
monthly approximation implements AquaCrop. We do not solve daily reference ET,
thermal-time development, a layered root-zone water balance, cultivar-specific canopy
expansion, or measured dry-matter partitioning. Even residue currently has the crop
good's chemistry, not separate straw/root chemistry.
[FAO calculation scheme](https://www.fao.org/aquacrop/overview/calculation-scheme/en).
Stage-dependent water stress is also motivated by FAO's distinction between vegetative,
flowering and yield-formation sensitivity; the coefficients here are not fitted to its
crop data. [FAO yield response](https://www.fao.org/4/X0490E/x0490e0e.htm).

## Adaptive local quotes

With `[market] adaptive_prices = true`, CPU monthly quotes depend on:

- The previous quote and current inventory relative to its target.
- Existing supplier-cost estimates: recipe inputs, food-valued labor, upkeep,
  accessible materials and known recipes.
- Quantity-weighted prices of paid deliveries received that month. Delayed cargo
  is excluded because spoilage can change its paid/kg without a price negotiation.
- Treasury and resident-household cash, discounting unfunded desired stock.

For previous price `p`, stock `s`, target `t`, cash `M`, estimated cost `c` and
observed delivery price `v`, the implemented rules are:

```text
scarcity = clamp((t-s) / max(t+s, 1), -1, 1)
           [zero target: -1 for surplus, otherwise 0]
shortage = max(t-s, 0)
f = clamp(M / max(shortage*p, 0.0001), 0, 1) [1 if no shortage]
d = min(scarcity,0) + max(scarcity,0)*f - (1-f) [last term only if shortage]
cost_signal = clamp(log(c/p), -1, 1) [0 if no estimate]
trade_signal = clamp(log(v/p), -1, 1) [0 if no delivery]
delta = clamp(0.08*d + 0.08*(min(cost_signal,0)+max(cost_signal,0)*f)
              + 0.12*trade_signal, -0.15, 0.15)
p_next = clamp(p * exp(delta), 0.0001, 1000000)
```

The rate limit controls adjustment speed, replacing the permanent 0.4–4× base-price
band in this mode. Absolute numerical guards remain. Costs read a completed price
snapshot, so iterating goods cannot feed an earlier quote back into a later one.
Quotes and delivery observations do not transfer money. Existing dispatch still
reserves inventory, payment and route capacity.

This is a bounded price-adjustment hypothesis, **not a market-clearing solution**.
Cash is a stock, not monthly effective demand. The same cash participates in multiple
quote signals (actual purchases still share a finite budget). A desired warehouse
reserve is not a willingness-to-pay curve. Existing labor cost is still the
`18 × food price` opportunity-cost proxy, not observed wages; raw-material estimates
retain a base-price component. Household payroll, common-share and relief rules have
not been replaced. Observing deliveries does not independently identify a good's
underlying value.

## Controlled evaluation, 2026-09-10

Hardware: Quadro RTX 5000 Max-Q, Vulkan; Rust 1.89, optimized test profile.
Terrain 64, ecology 32, one geological epoch with one ecological year, eight founding
groups; society, politics, governance and shipping enabled. Run 600 history months
with the planet fixed. These are model comparisons, not held-out empirical calibration.

The first un-funded quote rule pushed seed-17 wheat to 10,498 currency units/kg.
Adding purchasing capacity lowered that to 665, still far above the default quote.
A slower rate alone was not accepted as a fix. The following recorded comparisons
use the revised purchasing-capacity rule:

| Seed | Seasonal crops | Adaptive quotes | Final population | Occupied towns | Cumulative crop food equivalent | First town wheat quote |
|---|---|---|---:|---:|---:|---:|
| 17 | off | off | 1,261.9 | 8 | 17,049,324 | 8.00 |
| 17 | on | off | 0 | 0 | 364,211 | 0.80 |
| 17 | off | on | 1,321.6 | 8 | 16,426,116 | 664.65 |
| 17 | on | on | 0 | 0 | 363,660 | 9,406.49 |
| 81 | off | off | 1,326.5 | 8 | 14,242,973 | 8.00 |
| 81 | on | on | 0 | 0 | 294,665 | 14,396.87 |
| 256 | off | off | 892.1 | 8 | 11,861,004 | 8.00 |
| 256 | on | on | 0 | 0 | 275,486 | 8,787.02 |

Maximum absolute normalized economy residual across these cases was 0.0000137
(acceptance threshold 0.001). Good accounting did not prevent bad outcomes.
Abandoned-town quotes are not transactions and cannot be interpreted as market prices.
The price column is one site, not a price index. Cumulative crop production is converted
using each good's configured food-energy equivalent, not inferred dietary adequacy.

The crop comparison changes season length, seed establishment, partitioning and the
tuber coefficient together. It identifies a failed package, not each component's
individual effect. Existing founding provisions, productive land and food demand were
tuned around the old package. Neither increasing a yield constant until everyone
survives nor removing price bounds establishes calibration.

## Reproduction and compatibility

Both modes are stored in the economy catalog. Missing `season` and `adaptive_prices`
fields deserialize to legacy behavior. GPU catalog packing appends six seasonal
records before recipes; simulation-state and archive inventories keep their layout.
An explicit seasonal run through the reusable API can configure:

```rust
let mut catalog = EconomyCatalog::bundled()?;
catalog.agriculture = Some(AgricultureCatalog::seasonal_experiment());
catalog.market.adaptive_prices = true; // independent experiment
// After founding; configure_economy validates and uploads the new tables.
generator.configure_economy(catalog)?;
```

Pricing alone can also be enabled in an economy TOML override using the existing
`--economy-catalog` option. The seasonal preset is an agriculture catalog, not a
standalone economy override.

```sh
mise exec rust@1.89.0 -- cargo test --lib adaptive_price_tests -- --include-ignored
mise exec rust@1.89.0 -- cargo test --test economy -- --include-ignored --test-threads=1
# Prints the full factorial crop/price comparison for each of three seeds:
mise exec rust@1.89.0 -- cargo test --test economy seasonal_crops_and_adaptive_prices_seed_comparison -- --ignored --nocapture
```

Fixtures check stored-seed dormancy, finite phosphorus, full-season accounting,
seasonal/adaptive checkpoint continuation versus monthly stepping, catalog bounds and
old-field defaults. Price fixtures check controlled input-cost transmission into actual
local quotes, paid-price evidence, unfunded shortages and rate bounds. The long-run
test asserts accounting, not viability: collapsing towns must remain visible in its
output rather than being mistaken for a passed calibration test.

Verification completed: all 22 economy tests passed, including the 12-run factorial
comparison (137.99 seconds for the suite); both adaptive-price tests passed. After
extending the seasonal fixture, its checkpoint/batch comparison passed separately.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the repository
artifact-policy check passed. These are implementation checks, not calibration success.

Before promoting either experiment: isolate crop-process changes, measure monthly
harvest/consumption and first-harvest gaps, fit plausible land/food budgets, then evaluate
held-out seeds. For prices, distinguish funded orders and consumption flows from reserve
targets, share purchasing budgets among goods, and diagnose producer prices separately
from retail prices and nominal money supply. Empirical targets and uncertainty remain
required before making claims about real agronomy or ancient economies.
