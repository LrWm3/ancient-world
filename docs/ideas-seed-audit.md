# World and history design audit

Seven new century histories broadly support the intended structure, but expose weak links between it and varied historical outcomes. Settlement geography, trade, genealogy, regional government, expeditions and conservative flood consequences operate. Ordinary histories remain very comfortable and repetitive; discoveries have little practical impact, and the second ecological energy system is weak at continental scale.

**Important qualification:** all seven initial geological climates stopped at the configured four-cycle limit with `climate_converged = false`. The histories and their budgets completed successfully, but these are not fully converged physical baselines. This limits ecological calibration conclusions and should be resolved before tuning productivity or weather severity from these measurements.

## Experiments

New seeds 17, 81, 256 and 8675309 each ran 100 social years with five founding civilizations. Seeds 17 and 81 also ran with sixteen founders to increase political contact. A final seed-17 sixteen-founder control raised drought severity from 0.5 to 0.9; this is an intentional pressure test, not the proposed new default.

All seven use current flood fixes, terrain/ecology edge 64, one geological epoch, living history and all social systems through specimen discoveries. No manual wars or disasters were injected. Each history starts after ten ecological years and advances another century. These are diagnostic grids and short geological histories, not mature 1,000-epoch worlds. Jobs overlapped on the Quadro RTX 5000 Vulkan backend, so timings are not benchmarks. No simulation settings were changed between seeds within a suite.

| Seed | Founders | Final residents | Active towns | Delivered cargo | Treaties signed | Wars | Expedition charters |
|---|---:|---:|---:|---:|---:|---:|---:|
| 17 | 5 | 1,786 | 15 | 1,109 | 1 | 0 | 40 |
| 81 | 5 | 1,767 | 15 | 1,053 | 0 | 0 | 51 |
| 256 | 5 | 1,741 | 15 | 688 | 1 | 0 | 68 |
| 8675309 | 5 | 1,769 | 15 | 879 | 6 | 0 | 27 |
| 17 | 16 | 5,816 | 48 | 3,425 | 40 | 0 | 27 |
| 81 | 16 | 5,794 | 47 | 2,401 | 24 | 0 | 40 |
| 17, severe drought | 16 | 5,640 | 48 | 3,355 | 35 | 3 | 26 |

Residents exclude expedition crews away. Deliveries and treaties are cumulative events, not simultaneous contracts or agreements. Expedition charters include rescue vessels.

## Central civilization structure: working, growth too uniform

Every permanent settlement in every saved world occupies the central-island geographic class. Every surveyed expedition endpoint occupies the enclosing continent. Temporary expedition camps remain distinct from permanent settlements.

All four sparse runs end with exactly three towns on each of five islands, and three towns per polity. Dense seed 17 has 3, 3, 12, 15 and 15 towns on its five islands; each of its sixteen polities still administers exactly three towns. Dense seed 81 has 6, 6, 9, 12 and 14 towns: one polity has two towns, the other fifteen have three.

Thus multi-town governments and multiple polities per island work. However, the near-uniform two daughter villages per founder and narrow population range indicate that expansion thresholds and demographic schedules dominate geographic/economic variation in these runs. Town count is not yet a rich emergent settlement hierarchy.

Genealogy and institutions are active: sparse worlds have 376–384 recorded marriages and 1,052–1,066 people with recorded parents; dense worlds have 1,228–1,238 marriages and 3,413–3,436 with recorded parents. Successions, inheritance and faction shifts occur. These counts show functioning records and transitions; they do not establish that individual personalities or family rivalries drive major history.

## Markets and politics: responsive under pressure, quiet otherwise

Inter-island commerce and denser international networks operate. The six ordinary worlds have no recorded food crises, no annual shortage site-years and no wars. Their ending aggregate food stocks correspond to roughly **15.5–18.4 years of demand** using the market's 18 kg/person/month reserve convention. This is a stock-to-demand ratio, not a claim that individual food batches are that old. It suggests that generous buffers can suppress the material pressures meant to differentiate histories.

The severe-drought control produces 14 food crises, three wars, three governance crises, three secessions and six annually observed controller changes. One explicit chain is:

1. Month 721: a town records three consecutive hungry months.
2. Month 732: its polity declares war over a contested corridor and mobilizes seven adults, citing that food crisis.
3. Month 778: the conquered town restores local administration after a year of governance crisis.

This is a useful example of material conditions causing war and administrative limits undermining conquest. Peaceful normal worlds are not inherently failures. But current automatic war triggers depend narrowly on recent food crises or raids plus contested land routes and supplies. Additional goals and grievances would create variety without imposing a war quota. Scarcity calibration also needs attention before increasing aggression.

## Flood fixes: bounded cargo, adaptation still missing

Every annual observation in the seven runs retains only locally safe expansion candidates, and every sampled cargo delay is below six months. Dense seed 17 records 89 delayed contracts: 78 recover and 11 are written off; none remains waiting at the endpoint. Dense seed 81 records 45 delays: 42 recover and three are written off. Sparse seed 256 has one endpoint shipment waiting three months.

New flooding still occurs after settlement. Dense seed 17's Galan has been continuously wet for 118 elapsed months at the endpoint, with about 100 residents. It is correctly classified as persistently inundated, with a zero temporary-cleanup timer and no indefinite cargo backlog. The pressure control retains the same wet site with about 89 residents.

This confirms the distinction introduced by the fix, but also its limit: towns do not yet build defenses or relocate in response. Persistent water was not artificially removed, and chronic inundation can remain socially unresolved for years. That is a stronger next target than further cosmetic flood events.

## Ancient World expeditions: journeys work, discoveries matter too little

The six ordinary worlds launch 253 voyages. At the century endpoints, 234 are returned, thirteen original parties have been rescued, two parties are lost and four voyages are still outward. Rescue-vessel returns and rescued original parties are distinct terminal records, not duplicate survivors. There are setbacks, repair, retreat and provision costs, but complete losses are uncommon under these settings.

The spectrum of outcomes works for a restrained expedition beta. It does not yet establish the deeply dangerous, economically transformative frontier described in the setting. Destinations remain limited coastal routes, not an inland expedition network.

Concrete effects remain small: **remedy use is zero in all seven runs**, including the drought control. Remedies are manufactured, but much expires unused. Total applied specimen phosphorus in the six ordinary worlds ranges from about 1.63 to 9.12 kg against existing managed-soil stocks of millions of kg—less than 0.001% in every world. This comparison does not rule out a useful local application, but the aggregate agricultural effect is tiny.

Useful next content should respond to actual unmet needs and produce observable local benefits, costs or hazards. Merely increasing the abstract knowledge score or forcing more deaths would not address that gap.

## Ecological distinction: partial support, not yet the full intended scale

Endpoint area-weighted terrestrial producer carbon on the enclosing continent is approximately **1.71–1.91×** the central-island average across the four fresh geographies. Central shallow/deep underground producer biomass is zero; the outer continent retains both underground layers. This supports a regional ecological distinction.

However, geochemical production is only **0.069–0.111%** of total outer terrestrial photosynthetic-plus-chemosynthetic production at these endpoints. About **1.6–2.5% of outer land area** has at least a 10% geochemical share, with a small absolute-production floor to exclude numerical near-zero ratios. A large share in a low-productivity cell does not imply a lush hotspot. Average outer underground carbon is only **0.00081–0.00108 kg/m²**, roughly 0.09–0.13% of its terrestrial producer biomass.

The second energy pathway and underground habitats therefore exist, but do not yet produce the strong layered ecological scale envisaged. These are single-month endpoint production shares and short-geological-history worlds, not annual means or proof of a missing mechanism. Converged climate and longer geological-history comparisons should precede parameter changes. Calibration should preserve finite substrates and phosphorus constraints rather than adding a regional biomass multiplier.

## Great-lake barrier: qualitative pattern present

At the same endpoints, lake surface phosphorus concentration in the peripheral angular band is **2.36–4.11×** that in the central band. Approximately **81.7–84.1%** of phosphorus in the five measured lake compartments is in deep water. Peripheral vertical exchange is also stronger. This is consistent with the intended deep nutrient reservoir and less enriched central surface waters.

For reproducibility, “central” is angular distance below 0.6 radians from the basin axis; “peripheral” is 0.75–1.05 radians. These are basin bands, not individual island-shore and outer-shore samples. Surface concentration divides phosphorus inventory by modeled surface-layer water volume. The deep fraction's denominator includes surface water, deep water, active sediment, plankton and buried material; it excludes animal and groundwater compartments.

The deep reservoir is partly initialized explicitly and circulation asymmetry is an artistic constraint. This observational contrast does not independently prove that river transport generated it, or quantify how much lake nutrient reaches island farms. No mixing intervention was run in this audit.

## Recommended order

1. Resolve the initial climate convergence limit and repeat representative baselines before ecological calibration.
2. Improve food storage and settlement growth constraints, then add persistent-flood relocation/defense decisions. Measure diversity of town sizes, reserves and recoveries instead of targeting a disaster rate.
3. Connect family/faction interests, trade dependence and territorial opportunities to political choices beyond hunger-triggered war.
4. Make expedition findings meet actual local needs, then calibrate geochemical hotspots and underground production against longer, converged environmental histories.

## Validation and artifacts

All seven runs completed 100 social years with annual history and ecological budget checks passing. Maximum absolute relative residuals were approximately 1.29e−5 for managed history, 1.68e−5 for ecological C/N/P and 5.23e−5 for ecological water. **Budget success does not imply climate convergence or realistic calibration.** No simulation code or gameplay settings were changed during this audit beyond the declared founder-density and drought controls.

Reports and matching JSON/world files: [five-founder suite](../output/ideas-audit-sparse.md), [sixteen-founder suite](../output/ideas-audit-dense.md), [drought pressure control](../output/ideas-audit-pressure.md). The [analysis script](../scripts/analyze_ideas_audit.py) reads matching-grid archive fields and writes `*-ideas.json`; the existing flood-analysis script supplies per-town duration and delayed-cargo diagnostics.

```sh
mise exec rust@1.89.0 -- cargo build --release --example history_evaluate
target/release/examples/history_evaluate --seeds 17,81,256,8675309 \
  --resolution 64 --epochs 1 --years 100 --civilizations 5 \
  --discoveries --living-world --save-worlds \
  --output output/ideas-audit-sparse --label ideas-audit-sparse
# Dense: seeds 17,81; civilizations 16; output/label ideas-audit-dense.
# Pressure: seed 17; civilizations 16; --drought-severity 0.9;
# output/label ideas-audit-pressure. Other arguments match.
python3 scripts/analyze_ideas_audit.py output/ideas-audit-sparse \
  output/ideas-audit-dense output/ideas-audit-pressure
```

The subsequent [history realism refinement](history-refinement.md) fixes the climate warm-up limit, bounds grain storage, varies settlement incentives, and adds material-funded flood adaptation, with new seed comparisons.
