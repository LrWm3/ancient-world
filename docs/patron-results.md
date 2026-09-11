# Patron history acceptance results

Evaluation runs use monthly living environments at terrain/ecology edge 64, sixteen initial arriving communities, and the existing scarce-inner-continent defaults. These are diagnostic history runs, not production-resolution GPU benchmarks. Local concurrent workloads affect wall times.

## 200-year seed suite

| Seed | Population | Active towns | Traditions | Active institutions | Objects | Expeditions | Max residual |
|---|---:|---:|---:|---:|---:|---:|---:|
| 17 | 4521 | 34 | 47 | 104 | 621 | 6 | 2.23e-05 |
| 81 | 5329 | 45 | 42 | 98 | 717 | 46 | 2.81e-05 |
| 256 | 8887 | 57 | 49 | 145 | 964 | 49 | 2.33e-05 |
| 409 | 7453 | 53 | 77 | 105 | 914 | 45 | 2.42e-05 |
| 1024 | 4441 | 47 | 60 | 94 | 738 | 4 | 2.28e-05 |

Residuals are fractions, not percentages. Active towns include very small surviving settlements; the separate population and shortage measures matter.

| Seed | Accounts | Known-practice links | Recorded relationships | Shortage site-years | People records | Event records | Wall seconds |
|---|---:|---:|---:|---:|---:|---:|---:|
| 17 | 392 | 17388 | 16425 | 138 | 5119 | 134478 | 161.8 |
| 81 | 305 | 18205 | 18036 | 141 | 5519 | 148148 | 148.2 |
| 256 | 499 | 21899 | 21169 | 159 | 6808 | 177577 | 179.8 |
| 409 | 574 | 18811 | 19397 | 424 | 6210 | 180752 | 127.6 |
| 1024 | 459 | 14640 | 16012 | 155 | 5748 | 129887 | 99.9 |

| Seed | Living adult knowledge links | Active / all institutions | Manuscripts | Institution-funded voyages |
|---|---:|---:|---:|---:|
| 17 | 2757 across 1195 adults | 104 / 116 | 38 | 0 |
| 81 | 3510 across 1303 adults | 98 / 106 | 0 | 0 |
| 256 | 4532 across 2047 adults | 145 / 145 | 27 | 0 |
| 409 | 3790 across 1605 adults | 105 / 109 | 30 | 0 |
| 1024 | 2967 across 1506 adults | 94 / 98 | 29 | 0 |

| Seed | Pilgrimages | Office campaigns | Curated specimens | Lost objects | Knowledge-source links |
|---|---:|---:|---:|---:|---:|
| 17 | 17 | 168 | 2 | 48 | 16668 |
| 81 | 26 | 113 | 3 | 32 | 17485 |
| 256 | 40 | 213 | 6 | 0 | 21179 |
| 409 | 31 | 160 | 4 | 85 | 18091 |
| 1024 | 44 | 148 | 1 | 43 | 13920 |

| Seed | Living household faiths | Primary town roles | Secondary town roles |
|---|---:|---|---|
| 17 | 42 | agricultural: 33, religious: 1 | manufacturing: 25, mining: 7, religious: 1, agricultural: 1 |
| 81 | 40 | agricultural: 43, mining: 1 | manufacturing: 31, mining: 12, religious: 1 |
| 256 | 46 | agricultural: 57 | manufacturing: 38, mining: 17, religious: 2 |
| 409 | 64 | agricultural: 53 | manufacturing: 33, mining: 20 |
| 1024 | 53 | agricultural: 42, mining: 1, manufacturing: 1 | manufacturing: 24, mining: 16, religious: 1, scholarly: 1, agricultural: 1 |

Traditions and people in the first tables are historical record counts, including dead people and traditions no longer dominant. Living knowledge is shown separately; manuscripts and institutional sponsorship are contingent outcomes, not guaranteed in every seed.


All sixteen initial patrons departed; daughter towns inherited affiliations without receiving new patrons. Full per-town crop, herd and role data, annual shortages, expeditions and stage timings are in the JSON beside each archive.

## 500-year histories

| Seed | Population | Active towns | Traditions | Active institutions | Objects | Expeditions | Max residual |
|---|---:|---:|---:|---:|---:|---:|---:|
| 17 | 9641 | 63 | 224 | 191 | 1275 | 6 | 2.23e-05 |
| 81 | 11256 | 71 | 256 | 184 | 1515 | 61 | 2.81e-05 |

Residuals are fractions, not percentages. Active towns include very small surviving settlements; the separate population and shortage measures matter.

| Seed | Accounts | Known-practice links | Recorded relationships | Shortage site-years | People records | Event records | Wall seconds |
|---|---:|---:|---:|---:|---:|---:|---:|
| 17 | 2385 | 65772 | 69740 | 419 | 33625 | 558128 | 935.6 |
| 81 | 2916 | 74967 | 79821 | 1154 | 48081 | 644837 | 1162.0 |

| Seed | Living adult knowledge links | Active / all institutions | Manuscripts | Institution-funded voyages |
|---|---:|---:|---:|---:|
| 17 | 5393 across 4713 adults | 191 / 223 | 70 | 0 |
| 81 | 6166 across 7064 adults | 184 / 220 | 0 | 2 |

| Seed | Pilgrimages | Office campaigns | Curated specimens | Lost objects | Knowledge-source links |
|---|---:|---:|---:|---:|---:|
| 17 | 29 | 523 | 2 | 161 | 65052 |
| 81 | 84 | 398 | 3 | 263 | 74247 |

| Seed | Living household faiths | Primary town roles | Secondary town roles |
|---|---:|---|---|
| 17 | 98 | agricultural: 62, mining: 1 | manufacturing: 49, mining: 13, religious: 1 |
| 81 | 150 | agricultural: 70, mining: 1 | manufacturing: 55, mining: 16 |

Traditions and people in the first tables are historical record counts, including dead people and traditions no longer dominant. Living knowledge is shown separately; manuscripts and institutional sponsorship are contingent outcomes, not guaranteed in every seed.


All sixteen initial patrons departed; daughter towns inherited affiliations without receiving new patrons. Full per-town crop, herd and role data, annual shortages, expeditions and stage timings are in the JSON beside each archive.

## Matched seed-17 controls

| Mode | Population | Towns | Aid effort | Traditions | Institutions | Expeditions | Max residual |
|---|---:|---:|---:|---:|---:|---:|---:|
| Diverse farms + aid | 4521 | 34 | 464.0 | 47 | 104 | 6 | 2.23e-05 |
| Diverse farms, aid disabled | 4478 | 34 | 0.0 | 56 | 104 | 6 | 2.23e-05 |
| Legacy farms + aid | 11328 | 83 | 464.0 | 38 | 234 | 11 | 2.17e-05 |

The full 80-test hardware/catalog suite passed, followed by the additional office-campaign fixture and all four practices tests (81 distinct passing tests). Formatting and Clippy with warnings denied pass. Controlled fixtures cover conservation, finite patron service, schism ancestry, cultural identity under conquest, institutional expedition escrow/refund, ownership and custody, local pilgrimage, annual role inertia, specimen curation and checkpoint continuation.

The aid control changes assistance, not arrival supplies or initial doctrine. Later population and expedition outcomes can diverge in either direction because succession, shortages and political conflict interact. Legacy farming is a comparison model, not the abundance target.

## Observed limits

Seed 81 reached the explicit 256-tradition capacity by year 500; further schisms are suppressed at that limit and the explorer reports it. Knowledge is concentrated in active teachers and institutions; historical links do not imply universal education. Regional pilgrimages currently execute only on open routes whose round trip fits the monthly labor budget, so distant sacred sites can remain inaccessible. Unique specimens are preserved only after actual workshop deliveries; their absence in a seed is valid.

## Calibration

The earlier `patron-transmission-long` histories fell to 40 and 95 people at year 500. Controlled continuations from the same seed-17 year-200 archive isolated an unsustainably high nitrogen-fixation energy cost. Over the next century, the old-cost control reached 1,346 people, finite tool recycling alone reached 1,375, and the calibrated fixation cost plus recycling reached 5,639. The last intervention had a maximum final residual of 1.55e-5. This is a regional game calibration: fixation still consumes production energy and atmospheric nitrogen is recorded as an external input. It is not a general inner-continent fertility multiplier.

The cost is editable as `fixation_cost_kg` in the agriculture catalog. Old catalogs retain 80; new worlds use 12. Worn tools recover 90% as scrap, and remelting with fuel recovers 90% of that metal. Both stages retain irreversible losses. Raw control files are `output/patron-recycling-control.json`, `output/patron-recycling-intervention.json`, and `output/patron-nitrogen-intervention.json`.

## Reproducibility and performance

The GPU pipeline/buffer cache preserved the original 30-year samples while reducing observed time from 18.74 to 10.27 seconds. Indexed genealogy validation preserved an exact two-year continuation fingerprint and reduced observed time from 6.87 to 4.72 seconds. The event-archive transaction comparison is recorded separately in `output/patron-clone-before2.json` and `output/patron-clone-after2.json`. Concurrent compilation and GPU work make these local timings unsuitable as fixed performance promises.

An original version-two 100-year archive imported with all 33 towns’ original stock arrays intact. It received an explicit cultural baseline at month 1200 and zero fabricated patrons. The final implementation advanced it another year (`output/patron-complete-legacy-import.json`). A 200-year current archive advanced for two years with different batch sizes and a midpoint checkpoint: history, terrain and ecology matched bitwise (`output/patron-complete-replay.json`). The desktop loaded that mature history and produced `output/patron-complete-explorer.png`. Independent causal-record audits are in `output/patron-complete-audit.json`. 

## Final mature-history profile

A 24-month continuation of the final seed-17 year-200 archive took 1.25 seconds while the long seed runs were also active. CPU social processing and validation took 750 ms, GPU production plus readback 220 ms, and transaction setup 37 ms. The history fingerprint `50fee886edfc8d60` also matches the independent checkpoint/batch replay. Social record processing remains the largest measured component in this diagnostic workload.

## Memory observations

| Process | Host high-water mark |
|---|---:|
| output/patron-complete | 408860 kB |
| output/patron-complete-long17 | 462056 kB |
| output/patron-complete-long81 | 537544 kB |
| output/patron-complete-no-aid | 270616 kB |
| output/patron-complete-legacy | 292748 kB |

Host figures come from `/proc` lifetime high-water marks sampled before exit. The diagnostic GPU processes each reported approximately 301 MiB in `nvidia-smi`, including context overhead. These 64-edge worlds do not establish performance at production terrain resolutions.
