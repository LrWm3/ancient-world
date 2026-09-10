# Copper, tin, tools and processing residues

After enabling shared resources and mineral-specific iron processing, select **Enable copper, tin, bronze and physical residue**, or call `Generator::enable_alloy_processing()`. Existing iron-only histories retain their previous behavior until this explicit monthly-boundary upgrade. Existing inventories and past processing losses are preserved; no historical waste is reconstructed.

Chalcopyrite, malachite and cassiterite now have separate finite raw-good inventories, producing copper or tin through fuel- and labor-consuming GPU recipes. Bronze requires 90% copper and 10% tin plus fuel. Copper, tin, bronze, copper tools, bronze tools and their two scrap types remain distinct in storage, market targets, shipments and archives. Recipe validation prevents creating copper/tin or converting bronze scrap into legacy iron-compatible metal. Remelting returns 90% of scrap metal using fuel, with the remainder retained as residue.

Copper and bronze tools contribute to farming's existing tool effect. The initial model gives copper tools 60% effectiveness and twice the wear rate; bronze tools use the ordinary rate. Tool demand accounts for existing effective stocks across the three tool families rather than requesting a full reserve of each. Copper-source towns can make copper tools without tin, and select bronze when tin or bronze stock is available; tin-source towns seek copper for alloying. These are initial game coefficients. Construction, institutional projects and military equipment still use their existing generic tool/equipment stocks; universal substitution and copper/bronze weapons are future work.

New goods use reserved slots 35–44. Upgrades reject occupied slots or existing recipe/cargo references. Recipe definitions and mineral identities are archived, and active material IDs cannot be relabeled through catalog replacement. The strongest mapped mineral remains the sole identified local source. This does not add minor deposits or access to distant mines.

## Persistent physical residue

Declared inorganic smelting losses become on-site mineral residue, separately from fuel consumption and other external losses. Recipe `work[3]` now declares inorganic residue kg per batch and is validated against available inorganic input loss. The physical deposit is removed from the old discarded-mass counter, so it is not counted twice. Identified smelting cannot omit its residue through catalog editing.

Each town starts this feature with disposal capacity of 0.02 kg per square metre of its already claimed land. This is a fixed baseline allowance; population growth, repeated enabling and reoccupation cannot refill or resize it. Every GPU processing batch checks remaining disposal space, and adaptive labor forecasting sees the same constraint. A full deposit stops the affected recipe without consuming its inputs. Stock and cumulative deposition have an explicit equality check; no cleanup/export process exists yet.

Deposits have one authoritative location: the settlement's economic state. They survive abandonment, carry a first-deposition historical event, appear in settlement inspection and are included as dated read-only regional survey snapshots. Exporting a survey creates no spendable material. The model is aggregate mineral residue, not chemically resolved slag, tailings ponds or excavatable layers. It does not simulate leaching, toxicity, dump geometry, cleanup or reprocessing.

## Verification and calibration

GPU fixtures cover three seeds with matched 1 kg versus 100 kg disposal allowances, finite ore/metal/residue balance, production blockage, retained ruins, regional survey agreement and exact checkpoint/batch continuation. Controlled inventories establish the requirement for both alloy ingredients, copper tooling without tin, and separate scrap recovery. A market fixture verifies copper and tin cargo identity, conservation and payment at dispatch only. Existing iron, economy and environmental-return regressions also pass.

`mise exec rust@1.89.0 -- cargo run --example alloy_evaluation` runs seeds 17, 81 and 256 for five years at terrain 64/ecology 32, 16 starting civilizations and crop yield 0.33, with living history and environmental returns. It writes `output/alloy-evaluation.json`, marking completion only after all seeds finish. Quantities below are cumulative production, including any recycled feedstock:

| Seed | Copper metal kg | Copper tools kg | Copper scrap kg | Residue held kg | Bronze kg |
|---|---:|---:|---:|---:|---:|
| 17 | 140.7 | 139.8 | 27.7 | 511.8 | 0 |
| 81 | 127.7 | 124.3 | 25.7 | 533.7 | 0 |
| 256 | 145.5 | 141.2 | 27.3 | 723.4 | 0 |

The first pass produced copper but no useful copper-only products. Adding copper tools established a working industry without supplying free tin. None of these founding source sets contained cassiterite; bronze therefore remained absent. Natural disposal capacity did not bind within five years; the controlled reduced-capacity comparisons verify that it can bind. Economic residuals remained below 0.000001 and source residuals were zero; ecological budgets passed.

These are short establishment runs, not long-history calibration. The next geological/trade extension should improve access to real secondary or distant deposits and evaluate tin transport, rather than adding tin to towns independently of the terrain.
