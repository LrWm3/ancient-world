# Ancient-continent expeditions — v1 design

An explicit expedition baseline extends social history and shipping. Existing archives remain unchanged until enabled. Sparse expeditions run on the CPU alongside political graphs; dense ecology and settlement production remain on GPU.

## Complete cycle

Wealthy governments or private merchant sponsors can fund eight-person crews at working island harbors. A charter transfers money to escrow, withdraws adults from the resident workforce, and reserves actual food, timber and tools. Civilian food and tool reserves remain protected. Each polity ordinarily supports one mission at a time, with an additional rescue mission for a missing party. Automatic research proposals run annually; searches become eligible monthly after the expected return date plus two months, so the sponsor does not need omniscient knowledge of stranding. A search may find an empty camp if survivors have already departed. Manual launch and recall are available through the same generator API.

Surveyed routes cross only the great lake and terminate on dry enclosing-continent land. Sailing takes physical distance and inland harbor access into account. A coastal research camp conducts regional excursions for six months. Hazard depends on vegetation, geological activity, elevation and temperature at the actual destination, seasonal forcing, equipment, crew competence and accumulated knowledge. Seed/mission/month keyed randomness makes dispatch speed irrelevant.

Hazards cause delays, lost stores, named casualties or stranding. Stranding consumes repair materials, so repairs compete with the need to survive until rescue. Stranded crews consume provisions while attempting repairs or waiting for a separately funded rescue. Rescue crews must sail to the same camp before transferring survivors and supplies. Crew deaths enter the population ledger; lost/consumed stores enter existing goods and C/N/P ledgers. Return restores surviving adults and remaining stores. Unspent escrow returns to its original public/private sponsor; wages transfer escrow into the home economy.

Research returns charts, ecological observations or geological surveys rather than free resources. Findings become confirmed knowledge only when delivered home, improving future expedition preparation and survival. Suspect ecological collections require tools for inspection; insufficient inspection equipment can introduce a fictional exposure into the existing disease-burden model. No real-world pathogen mechanisms are represented.

## Records, controls and bounds

Archive routes, expedition rules, clocks, escrow, crew rosters, phase deadlines, objectives, provisions, casualties, findings, sponsor, cause links, knowledge and cooldowns. The history inspector shows voyages and enables launch/recall; CLI and multi-seed reports expose the same feature. Reports separate launches, returns, losses, rescues, knowledge, resident population and population away.

Temporary camps do not establish civilizations, permanent settlements or territorial claims on the outer continent. v1 has named crew records representing withdrawn adults, but no family-tree integration, individual ship navigation, inland tactical pathfinding, foreign expeditions, permanent outposts, or invasive-organism simulation. [Specimen applications](discoveries.md) now provide finite fictional remedies and phosphorus processing. Regional observations use the saved environmental baseline unless [living history](living-history.md) is enabled, which updates ecological and weather exposure while retaining terrain geometry. [Crew competence](expedition-crews.md) now connects local preparation, specialists, experience and attrition to research, hazard resistance and repairs. There is one surveyed destination per harbor, and sailing currently advances distance and provisions without separate storm encounters. These limits keep the first expedition cycle testable.

Acceptance: finite inventories, no resource or population duplication, lake-only route interiors and outer-shore endpoints, no outer settlements, wealth gates, distinct hazards across seeds, confirmed knowledge only on return, rescue transfers, exact same-backend save/resume, configurable automatic proposals, and closure/recall behavior. Test both organic seed suites and controlled shortage/hazard fixtures; do not impose success or disaster quotas.

## Default gates and controls

A harbor needs capacity 500; its town needs 80 residents and 16 adults. Each charter withdraws eight adults, 100 kg timber, 24 kg tools and 600 money. Public sponsors must hold 1,200 money; otherwise a private sponsor needs 4,600. Food covers the round trip, six field months and six reserve months, while leaving a year of civilian food. Rescue provisions allow for sixteen returning people. Civilian timber and tool reserves are also protected. These are configurable game-scale assumptions, not nautical engineering quantities.

Normal charters have a five-year cooldown per political sponsor. The UI/API exposes automatic proposals, hazard scale (0–5), reserve months (2–24) and manual launch/recall; the API also supports cooldowns (12–240 months). Research objectives respond to disease pressure or depleted ore, otherwise rotating between charts, geology, ecology and the three [heritage objectives](heritage-expeditions.md). Delivered knowledge improves future competence and lowers field risk. Successful observations slightly improve home loyalty; losing a party can increase unrest.

Enable `--expeditions` in the history evaluator to include shipping automatically. On the main headless CLI, enable its prerequisites with `--society --politics --governance --shipping --expeditions`, or load an archive that already contains them. Desktop histories with shipping and governance offer an enable button. Existing archives retain their prior baseline until explicitly enabled.

## Verification and reproducibility

```sh
mise exec rust@1.89.0 -- cargo test --all-targets -- --include-ignored --test-threads=1
mise exec rust@1.89.0 -- cargo run --release --example history_evaluate -- \
  --expeditions --civilizations 5 --save-worlds --output output/expeditions-final
mise exec rust@1.89.0 -- cargo run --release -- --load output/expeditions-final.7.world
```

The integration fixtures check survivor/store transfer on rescue, recall travel time, starvation write-offs, malformed archive rejection, forbidden outer settlement placement, and identical monthly versus batched checkpoint continuation. Multi-seed runs additionally validate every simulated year and record population, goods, money and elemental residuals. The voyage archive currently caps at 512 charters; further launches return an explicit API error, and automatic proposals stop succeeding at that limit.

See [measured results](expedition-results.md) for calibration and remaining limitations.

An optional [specimen and workshop extension](discoveries.md) now connects returned material to local applications. Enable it explicitly on an existing expedition history; the original observation-only baseline remains compatible.

Religious, patron and literary charter objectives, finite minor artifacts, and their
limits are described in [heritage expeditions](heritage-expeditions.md).

See [expedition crew competence](expedition-crews.md) for preparation, specialist losses, rescue continuity and verification.
