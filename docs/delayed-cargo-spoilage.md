# Perishable cargo during flood delays

Flooded journeys previously spoiled only the aggregate dietary-food slot. Fish,
milk, meat, crops and preserved food could wait without perishable losses until
the six-month journey write-off. Goods now have an optional `delay_spoilage`
coefficient: the fraction of remaining cargo lost per extra month held by flood
closure, between zero and one inclusive. NaN and infinite rates are rejected.

The bundled catalog uses these game-design rates:

| Cargo | Monthly delayed loss |
|---|---:|
| Wheat, barley, millet, pulses | 2% |
| Flour | 4% |
| Tubers | 8% |
| Preserved food | 1% |
| Eggs | 10% |
| Meat | 30% |
| Fish | 35% |
| Milk | 40% |
| Aggregate dietary food | 20% |
| Tools and other unconfigured materials | 0% |

These are not empirically fitted shelf lives. They provide a bounded distinction
between preserved provisions, grain and fresh animal products under disrupted
transport. There is no refrigeration, temperature-dependent decay, packaging
model, normal-voyage spoilage or revised town-storage decay in this increment.
Rates are editable in `assets/economy.toml` and travel with archived catalogs;
there is no separate viewer configuration control.

## Transfers and consequences

On a due delivery that remains blocked, remaining mass becomes `mass * (1-rate)`.
The actual representable mass difference is recorded through the existing cargo
loss path. Each material uses its own C/N/P composition, not aggregate food ratios.
Lost matter leaves the managed settlement domain; this does not fertilize a
nearby ecological cell without a known receiving location. The seller retains the
payment already transferred at dispatch; losses do not mint a refund.

The recipient receives no stock during the delay. Reopening delivers only the
remaining quantity, so later household consumption or production has less to use.
Export-contract receipts likewise observe only delivered mass. Spoilage events
name the material and rate. Durable goods keep their mass during short delays,
but the existing six-month termination rule still writes off a blocked journey.
Preservation reduces perishable loss; it does not exempt cargo from termination.
A shipment that spoils completely terminates immediately, without a zero-mass
delivery or successful-arrival event.

A repeated market call at the same boundary does not repeat spoilage: the first
delay schedules arrival for the next month. No new GPU fields or cargo clocks
are introduced. The existing sparse CPU market stage handles these transfers.

## Compatibility and evidence

Missing or null coefficients preserve the old rates: 20% for the aggregate-food
slot and zero for other goods. Explicit zero disables spoilage, including for
aggregate food. Existing material quantities and nutrient stocks are not changed
when reading an old archive. New bundled catalogs supply the differentiated rates.

Catalog tests cover omitted fields, explicit zero/one, invalid values, and the
fresh/preserved/durable distinction. The flood fixture compares 10 kg of fish,
preserved food and tools over two delayed months, then reopens the route. Expected
remaining masses are 4.225, 9.801 and 10 kg. It checks no early receipt, one decay
per boundary, final delivery, material/C/N/P/cash accounting and serialized
continuation. Initial cargo is a declared fixture import, not simulated purchasing.
Existing tests retain generic food spoilage and six-month write-off coverage.

```sh
mise exec rust@1.89.0 -- cargo test --lib hazards::tests -- --include-ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --test markets
```

This verifies a material-specific transport consequence, not a multi-seed
calibration of shortages, food preservation profitability or trade volumes.

### Recorded validation

52 ordinary all-target tests passed (120 hardware/long-running tests remain
ignored by that command). Targeted runs passed three GPU-backed tests: the
expanded flood fixture and two market regressions. The complete market run
passed all eight tests. Formatting and Clippy with warnings denied passed.
Hardware: Quadro RTX 5000 Max-Q using Vulkan.

[Artifact retention policy](evidence/README.md) preserve
reproduction commands and distinguish the final total-loss fixture run from the
preceding market regression run.
