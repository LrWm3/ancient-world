# Finite expedition specimens and local applications

See [the latest memory, fleet and research extension](memory-fleets-and-research.md) for subsequent changes.

This optional extension connects expeditions to farming, health and manufacturing. Enable it after expeditions with `Generator::enable_discoveries`, the desktop's **Enable specimen research and applications** button, or `--discoveries` on the headless CLI. Existing archives load without it; enabling establishes a recorded baseline and does not award cargo to past voyages. The history evaluator's `--discoveries` implies shipping and expeditions.

## Collection and return

Ecological voyages can collect **faultroot resin**, an explicitly fictional biological material. Geological voyages can collect **phosphatic crust**. Resin availability depends on destination vegetation, geological activity and a permissive temperature range; mineral availability depends on geological activity. These are two regional specimen classes rather than individual catalog species or mapped ore veins.

The first visit establishes a finite *accessible coastal* stock at the destination cell. Harbors reaching the same cell share that stock. It represents material accessible to repeated short excursions from that camp, not all resources on the continent. This stock is recorded separately from the frozen planetary environmental baseline and town plots. Extraction is a declared C/N/P import into the civilization economy and a withdrawal from the accessible source ledger. It does not silently renew or claim to deplete the whole GPU ecosystem.

Working parties collect up to twice their competence score in kg per field month, reduced by tool availability, with a 12 kg manifest limit. Rescue vessels can carry both parties' manifests. Lost vessels write off their specimens; successful rescues transfer them. Biological cargo requires inspection tools at home. Failed inspection discards the cargo and retains the existing fictional exposure consequence. A delivered specimen event links to its expedition's departure.

Automatic charters respond to disease pressure, phosphorus-limited farming and a rotating research program. Known exhausted sources redirect collection toward the other material or charting. Knowledge alone cannot manufacture either material.

## Research and production

Each receiving settlement keeps a workshop and its learned applications. Research destroys 1.5 kg of a material, at most 0.25 kg per successful month, before establishing that material's recipe. Assays and later processing both require two worker-months, 0.1 kg tools and 0.2 kg charcoal per kg processed. Tool wear enters the existing goods/waste ledger; fuel combustion is an explicit carbon export. Research consumes its sample into declared external losses.

Workshops reserve at most two workers from the settlement's craft allocation before GPU dispatch. Ordinary GPU recipes receive the remaining craft labor. Workshops can be paused independently of markets; abandoned settlements also stop workshop production, and shortages limit throughput. The sparse workshop bookkeeping runs alongside the existing CPU expedition/history graphs; normal farming and manufacturing remain GPU kernels.

- Resin processing retains half the input mass as remedy and accounts for processing losses. Remedy is a game abstraction: it can reduce the existing local disease-burden scalar, consumes an actual dose and cannot increase population directly. Healthy towns keep only a small emergency reserve (0.0005 kg per resident); illness raises the target to two months of doses (0.01 kg per resident). Processing stops when the reserve is filled, retaining raw samples for later demand. Remedy stocks lose 1% per month to spoilage. Treatment also exports the consumed material's C/N/P.
- Mineral processing transfers its 8% phosphorus content into the receiving town's managed soil. The mineral mass is consumed in the specimen ledger; non-nutrient processing residue is outside tracked C/N/P. It cannot create nitrogen, ore or unlimited fertility.

Species domestication, propagating an imported ecosystem, specialist genealogy, trade in these new materials, and widespread technology diffusion remain outside this increment. Recipes stay with their workshop settlement, including through changes of political controller.

## Accounting and inspection

Archives contain source stocks, manifests, delivered samples, study progress, product stocks, source events and cumulative processing/treatment/spoilage/labor ledgers. Both reserved craft labor and actual processing labor are recorded; shortages can leave reserved capacity idle. The specimen subarchive is version two. Three additional conservation residuals cover resin, mineral and remedy. C/N/P inventories also enter the existing civilization ledger. The desktop shows manifest contents, workshop progress, available coastal stocks, products and residuals. History records deliveries, learned applications, production milestones, treatments and source depletion.

```sh
mise exec rust@1.89.0 -- cargo run --release --example history_evaluate -- \
  --discoveries --civilizations 5 --save-worlds --output output/discoveries-initial

# Upgrade an existing expedition world without restarting its history:
mise exec rust@1.89.0 -- cargo run --release -- --headless \
  --load output/expeditions-final.7.world --epochs 0 --discoveries \
  --history-years 20 --save output/discoveries-continued.world
```

Hardware fixtures cover source exhaustion across repeat voyages, finite fertilizer, rescue and loss manifests, independent workshop closure, demand-driven processing, research before production, useful treatment, malformed inventory rejection, and exact same-backend checkpoint continuation.

[Measured outcomes and refinement](discovery-results.md) document the seed suites and validation. The final evaluator reports and inspectable worlds use `output/discoveries-final`; `output/discoveries-initial` retains the first prototype measurements.
