# Shared-resource counterfactual results

Historical development-seed experiment, not empirical calibration. The monthly
factorial follow-up and reproducible evidence workflow are described in
[model evidence](model-evidence.md). The completed [staffing ablation](mine-staffing-evidence.md)
reverses the closure population effect in all three seeds when labor shares are fixed.

Seeds 17, 81, 256; living history, terrain 64/ecology 32, crop yield 0.33. Each pair forks the same year-one checkpoint. Treatment closes ore and clay extraction in starting towns for five years, then reopens it for five years. Values below are baseline → treatment at the end of closure. Metal production is cumulative since founding.

| Seed | Mean ore price | Metal made kg | Tool stock kg | Market deliveries | Population | Farm workers (snapshot) |
|---|---|---|---|---|---|---|
| 17 | 4.1 → 16.0 | 1620.8 → 1045.1 | 1242.9 → 875.9 | 761.0 → 509.0 | 1690.5 → 1858.6 | 765.5 → 846.6 |
| 81 | 4.3 → 15.5 | 1457.6 → 1132.9 | 1170.7 → 1029.5 | 678.0 → 551.0 | 1646.9 → 1817.5 | 688.4 → 748.9 |
| 256 | 4.3 → 16.0 | 1854.4 → 1034.7 | 1303.4 → 959.5 | 755.0 → 418.0 | 1789.4 → 1985.8 | 800.5 → 887.7 |

Five years after reopening:

| Seed | Mean ore price | Tool stock kg | Population |
|---|---|---|---|
| 17 | 4.7 → 3.6 | 1245.8 → 1228.7 | 1750.4 → 1733.7 |
| 81 | 5.0 → 4.6 | 1124.7 → 1154.2 | 1671.9 → 1637.1 |
| 256 | 5.6 → 5.8 | 1404.3 → 1321.2 | 1925.6 → 1983.7 |

Maximum sampled source residual: 0.00e+00. Maximum absolute sampled normalized economy residual: 2.43e-06. All six branches completed.

Closure changes manufacturing and trade without deleting ore or imposing a global economic penalty. Population rises relative to baseline during the closure even while tools become scarcer. Staffing snapshots show the actual labor response, but do not isolate all mediators: food allocation, construction, migration, weather exposure and subsequent social decisions also interact. This is a model counterfactual, not proof of an empirical causal coefficient. Reopening recovers supply and prices without requiring branches to converge to identical histories.

The report retains annual trajectories, both branches’ full source inventories and the pre-intervention sample in `output/resource-counterfactual.json`. Checkpoints are `output/resource-counterfactual.{17,81,256}.world`. The reusable runner is `examples/resource_counterfactual.rs`.

Verification: 22 distinct focused tests passed (13 economy, seven markets, one controlled GPU resource fixture and one competing-claims unit fixture). Clippy with warnings denied, formatting and diff whitespace checks pass. Regional archives agree with canonical depletion and same-backend checkpoint/batching tests match exactly.
