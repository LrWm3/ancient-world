# Flood seed audit — September 7, 2026

Follow-up: [implemented fixes and rerun results](flood-fixes-results.md).

The monthly flood response works, but this audit found two consequential gaps: persistent inundation can be treated as an endless temporary disaster, and commercial cargo can wait for years without spoiling or abandoning its journey. Settlement suitability also uses a different water-depth measure from flood damage. These should be addressed before adding expensive flood-defense infrastructure.

## Experiments

Ran 19 new worlds totaling 910 social years on the Quadro RTX 5000 Vulkan backend:

- Eight seeds (0, 7, 42, 99, 999, 123, 2026, 31415), each for 50 years with default storm probability 0.04.
- The same eight seeds and settings for 50 years with probability 0.20. Storm intensity remains 3×; only occurrence probability changes.
- Seed 0 for 100 years with storm probability zero, to investigate persistent flooding seen in the previous century suite.
- Seeds 0 and 99 at 256 cells per face for five years each, as production-resolution smoke checks.

All use one geological epoch, five founders, matching terrain/ecology grids, living history and all history features through specimen discoveries. The main comparisons use 64 cells per face. Simulation parameters and algorithms were left unchanged during the audit. Jobs overlapped, so their timings are not performance benchmarks.

## Paired 50-year results

A flooded town-month is one occupied town flooded for one simulation month. Values below accumulate across each run; they are not instantaneous inventories.

| Seed | Town-months, normal | Town-months, stormier | Stored food lost kg, normal / stormier | Crop losses kg, normal / stormier | Delayed cargo, normal / stormier |
|---|---:|---:|---:|---:|---:|
| 0 | 70 | 351 | 887,107 / 1,424,419 | 307,366 / 1,045,015 | 0 / 10 |
| 7 | 1 | 19 | 627 / 3,484 | 0 / 17,212 | 0 / 7 |
| 42 | 5 | 11 | 4,044 / 8,408 | 9,346 / 9,833 | 0 / 0 |
| 99 | 0 | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| 999 | 0 | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| 123 | 0 | 12 | 0 / 1,716 | 0 / 11,207 | 0 / 5 |
| 2026 | 0 | 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| 31415 | 0 | 0 | 0 / 0 | 0 / 0 | 0 / 0 |

Total flooded town-months increase from **76 to 393** (5.17×), stored-food losses from **891,778 to 1,438,028 kg** (1.61×), and crop losses from **316,712 to 1,083,267 kg** (3.42×). Increased storms affect existing exposed sites and introduce floods on seed 123; they do not flood every seed indiscriminately.

Road closures increase from 2 to 12 and port closures from 22 to 66. Stormier runs produce 22 delayed-cargo incidents: 17 recover before year 50, five are still waiting. Four remaining shipments have waited two months; one has waited 14 months. Default-weather runs have no delayed cargo in this first-50-year window, despite port closures: closures do not necessarily coincide with an arriving shipment.

Completed wet episodes last at most seven months under normal weather and 20 months under stormier weather. Thus ordinary recession/recovery occurs, but multi-season floods are possible. These lengths measure flood onset to recession, excluding subsequent cleanup. No town has an ongoing flood longer than a year at the 50-year endpoints.

Seed 0's final population falls from 1,098.7 to 1,059.2 in the stormier comparison (about 3.6%). Other seeds change much less. This is a coupled history experiment: additional rain can benefit water supply while damaging exposed settlements, and subsequent decisions diverge. Population need not fall proportionally to storm frequency or crop losses.

## Persistent flooding: confirmed problem to resolve

The earlier default-weather seed-0 century archive contains Lorwick, founded at month 1044 with 60 settlers. It floods at month 1057 and remains flooded through month 1200: 144 observed wet monthly steps, with 143 months elapsed since onset. It has only about 6.3 residents at the endpoint. A 120.3 kg ore shipment has waited **139 months**.

The new zero-storm century reproduces Lorwick's 143-month elapsed flood and the same 139-month cargo delay. Its total 144 flooded town-months contrast with 276 in the earlier default-weather century. This shows the persistent case does not require stochastic storm events. Ordinary runoff, stored water and the fixed surveyed bankfull reference can sustain exposure. It does not establish which individual hydrological coefficient is wrong; permanent wetlands are also possible.

Code inspection identifies a concrete inconsistency: living-history candidate filtering uses area-average water depth below 0.25 m, whereas hazard inspection divides that depth by 0.05 in river corridors. At the default century endpoint Lorwick has about 0.0222 m average water depth, corresponding to 0.444 m local exposure. Those criteria disagree about its suitability. Its flood starts after founding, so the final snapshot alone does not prove it was flooded on its founding day; historical risk screening is needed as well as consistent current-depth checks.

The shipment conserves goods, but keeping paid cargo trapped for over eleven years is an undesirable historical outcome. It was ore (good ID 1), not food as initially reported; food independently had no spoilage while waiting. Conservation success does not make that behavior plausible.

## Accounting and coverage

All annual validation checks passed in all 19 new runs. Maximum absolute relative residuals across them were approximately 1.19e−5 for managed-history budgets, 1.43e−5 for ecological C/N/P and 6.28e−5 for ecological water. These are finite-precision residuals within current tolerances, not exact equality. There were no reported budget or history-validation failures.

The two 256-resolution runs had no town floods in their first five years. They verify execution and accounting at that resolution, not mature flood behavior or resolution independence. Eight coarse seeds are useful diagnostic coverage, not a statistical calibration of all worlds. Annual samples check budgets and endpoint state; event histories provide monthly flood durations and cumulative counts.

## Recommended next changes

1. Use consistent local exposure for settlement suitability, then incorporate observed flood frequency so towns avoid or explicitly accept recurrent risk.
2. Give delayed cargo spoilage, journey termination or return behavior with explicit inventory and payment accounting.
3. Distinguish persistent wetland/inundation from temporary flood cleanup, and compare the surveyed bankfull reference against ordinary living-history discharge before tuning it.
4. Add relocation and drainage/levee decisions once the persistent-water cases have a clear interpretation.

## Reproduction and artifacts

```sh
mise exec rust@1.89.0 -- cargo build --release --example history_evaluate
target/release/examples/history_evaluate \
  --seeds 0,7,42,99,999,123,2026,31415 --resolution 64 \
  --epochs 1 --years 50 --civilizations 5 --discoveries --living-world \
  --output output/flood-audit-normal --label flood-audit-normal --save-worlds
# Repeat with --storm-probability 0.2 and output/label flood-audit-storm.
python3 scripts/analyze_flood_audit.py output/flood-audit-normal output/flood-audit-storm
```

The zero-storm control uses `--seeds 0 --years 100 --storm-probability 0`; the production checks use `--seeds 0,99 --resolution 256 --years 5` without saved worlds. Other settings match above.

Reports: [normal](../output/flood-audit-normal.md), [stormier](../output/flood-audit-storm.md), [zero storms](../output/flood-audit-no-storm.md), [256 resolution](../output/flood-audit-256.md). Corresponding JSON files retain annual samples. Saved 64-resolution worlds retain full event histories; the [analysis script](../scripts/analyze_flood_audit.py) writes per-town durations, current delayed cargo and maximum residuals to `*-analysis.json`.
