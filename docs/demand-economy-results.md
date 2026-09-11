# Demand-driven economy evaluation

Diagnostic terrain/ecology edge 64; monthly living history, scarce-inner-continent defaults. Stored goods exclude the separately tracked prepared-food reserve. Dry goods exclude edible raw stocks. Timber and flax growth is controlled at production; their existing stocks are retained. Excess incidental animal byproducts enter recorded compost.

| Seed | Years | Population | Dry kg/person | Wood tonnes | Flax tonnes | Shortage site-years | Deliveries | Residual | Seconds |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 17 | 200 | 4652 | 11.1 | 19.0 | 0.4 | 113 | 24014 | 2.24e-05 | 124.5 |
| 81 | 200 | 4738 | 11.4 | 19.9 | 0.1 | 65 | 39457 | 2.88e-05 | 102.9 |
| 256 | 200 | 8346 | 10.7 | 33.6 | 0.5 | 86 | 38938 | 2.45e-05 | 99.5 |

## Previous completed histories

| Seed | Population before → after | Shortage observations before → after | Deliveries before → after |
|---|---:|---:|---:|
| 17 | 4521 → 4652 | 138 → 113 | 42102 → 24014 |
| 81 | 5329 → 4738 | 141 → 65 | 46538 → 39457 |
| 256 | 8887 → 8346 | 159 → 86 | 55916 → 38938 |

These are whole-system comparisons. Historical trajectories can diverge; differences do not isolate the effect of an individual rule. Use --legacy-production for a current-build control.

## Iteration and interpretation

The first pilot stabilized dry goods but seed 81 fell to 1,225 people. Inspection isolated an old labor heuristic: ore stocks below two kg per resident repeatedly diverted 12% of workers from agriculture to mining. Under order-driven extraction, low inventory no longer means unmet demand. Disabling that mining-rush heuristic for planned economies restored the same seed to 4,738 people at year 200, with crop yields and nutrient settings unchanged.

A separate integration failure showed that specimen research was missing from the demand graph. Open workshops with samples now order real fuel and tools. Their study, treatment, fertilizer, loss and continuation fixtures pass again. A planner fixture also caught selection of unavailable scrap; recycled-metal orders now require sufficient scrap.

Dry goods per resident between years 150 and 200 remained 11.06 → 11.07 (17), 11.74 → 11.40 (81), and 10.57 → 10.70 (256). This is a stock equilibrium rather than a pile that grows proportionally to elapsed centuries. Edible raw goods and prepared food use their existing separate storage rules.

All three seeds retained substantial trade, with fewer deliveries than their previous histories. Shortage observations decreased in all three; population increased in one and decreased in two. This is not a claim that each intervention improves every town or that fewer deliveries alone prove better trade.

## Verification and performance

The complete hardware/catalog suite passed 85 tests. After the mining-labor correction, all eight focused economy tests passed again, including the overfull-yard and agricultural-labor regression. Formatting and Clippy with warnings denied pass.

A mature planned world continued for 24 months uninterrupted and in one-month batches with a midpoint save/load: history, terrain and ecology matched bitwise (`output/demand-economy-replay.json`). A previous year-200 archive opted into the new catalog and continued two years with preserved bulk overstock and maximum final accounting residual below 0.002% (`output/demand-economy-upgrade.json`). Its standing crops and inventories were not reinitialized.

A separate 24-month mature continuation took 1.29 seconds: social processing/validation 755 ms, GPU production/readback 225 ms, transaction setup 49 ms. Host high-water memory was approximately 252 MiB. Its history fingerprint matched the replay. Other simulations were active, so these are diagnostic measurements rather than an isolated speed comparison.

The new system still uses regional working-equipment allowances and population-based storage, not individually owned equipment or constructed warehouse buildings. Specialized workshop investment, paid carrier businesses and autonomous labor reallocation remain beyond this increment. Spare craft labor is visible rather than counted as useful production.

## Five-century check

Seed 81 finished 500 years with 10,775 people, 154,278 deliveries, 125 launched expeditions and maximum observed accounting residual 2.88e-5 (0.00288%). Dry stock remained 11.39 → 11.46 kg per resident over the final fifty years. Stored timber was 44.2 tonnes and flax 0.2 tonnes, compared with roughly 21,319 and 6,499 tonnes in the previous 500-year archive. The first 200 annual samples exactly matched the independent 200-year run.

This long run had 1,169 shortage site-year observations versus 1,154 previously, and population was about 4% lower. The result supports inventory stability and continued viability, not universal elimination of shortages. Population-weighted tool sufficiency was 92.9% at the endpoint.

The final audit found one small construction boundary issue: roads originating in abandoned towns could bypass the cap because those towns no longer receive monthly planning flags. The maximum observed road stock was 1,006.9 kg. The cap now consults the archived world policy, and abandoned towns perform no new road construction. A controlled fixture tests both missing monthly flags and abandonment; this final edge correction was made after the reported long run. Existing road material is retained rather than erased.

After that final correction, the year-500 world continued another two years with exact history/terrain/ecology equality between uninterrupted execution and a midpoint checkpoint with monthly batches (`output/demand-economy-long-replay.json`). The additional road regression passed, bringing the distinct passing test count to 86 across the full suite and follow-up checks. Final formatting and Clippy checks passed.
