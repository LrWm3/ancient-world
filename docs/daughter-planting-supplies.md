# Daughter-town planting supplies

The [nutrient review](farm-nutrient-retention.md) found a daughter town with
fertile soil, water and farm labor but no seed, standing crops or harvests.
The old initialization copied only dormant seeds from the nearest older town
of the same civilization. That was not necessarily the actual parent, and a
fully planted parent could have no dormant seeds despite having usable crop
goods. Founding could succeed with no planting supplies at all.

## Founding transaction

Annual founding now checks planting supplies after its existing population,
provision, destination and settlement-cap checks. Both aggregate and individual
founding use the same plan. Individual founding rechecks when called directly.
Failure is reported as `planting_supplies` in the annual admission diagnostic;
it does not move residents, money, food or materials.

For diversified farming, the plan draws from the **actual parent**:

- Up to 20% of each crop's dormant seed reserve.
- Raw goods of that crop can fill the remaining allowance, also bounded by 20%
  of the parent's current stock; shared goods cannot be allocated twice.
- At most 2 kg per crop, with at least one edible crop receiving 0.1 kg.
- Standing plants, processed food, and another town's supplies do not qualify.

The 0.1 kg packet is a toy establishment rule, not an agronomic seeding density.
It is above the GPU's existing live-crop threshold. This avoids requiring a full
2 kg allowance when a smaller finite packet can already establish in this model.
The donor retains at least 80% of each affected reserve or raw good.

After all founding checks pass, the transaction subtracts the real supplies and
adds the same crop masses to the daughter's seed inventory. It records a
`founding_seeds` event linking parent and destination. There is no new external
inventory, production credit or nutrient source. Legacy farming retains its
existing seed accounting.

## Timing and scope

This runs inside the existing annual Respond operation, using stocks remaining
after that month's production and economic actions. Its plan is committed in
the same call; it is not an asynchronous reservation. Close initializes the
new economy and preserves the carried seeds. Open can repeat preparation without
transferring them again. The daughter first produces in a subsequent month;
there is no backdated harvest. Checkpoints retain its ordinary crop inventories.

The existing regional founding abstraction still has no explicit voyage for
these settlers. Livestock initialization remains a separate older behavior.
This does not repair already seedless towns by importing new supplies, ensure
every crop fits the new climate, or guarantee food until a first harvest.
Existing purchase-based seed establishment remains available.

## Verification

```sh
cargo test --lib founding_seeds
cargo test --lib founding_moves_whole_rosters_and_preserves_continuation -- --ignored
```

Fixtures cover an empty donor, non-food crops alone, standing plants without
seed, raw grain as a substitute, protected donor stocks, conserved crop mass,
atomic rejection, initialization without loss or duplication, and continuation.
The daughter fixture also requires a positive actual harvest. The complete
`cargo test --lib civilization:: -- --include-ignored` group passed: 21 tests,
including GPU fixtures, on the Quadro RTX 5000 with Max-Q Design. An initial
parallel run exposed shared temporary filenames in the agriculture attendance
fixtures; those paths now include the fixture's sector flags and the rerun passed.
