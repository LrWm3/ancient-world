# Inner-continent civilizations: implementation roadmap

> **Archived design / legacy reference — 2026-09-09.** This preserves the original proposal and its historical limitations; it is not the current implementation status or an active task list. See the [current civilization guide](../civilizations.md) and [documentation index](../README.md).

Status: the [first civilization beta](civilization-beta.md) implements inner-continent founding, GPU habitat/food/demography, expansion, food relief, succession, events, inspection and persistence. The [economic expansion](../economy.md) now adds reserved ecological plots, nutrient-limited farming, labor, finite extraction, five craft recipes and paid same-continent markets. The opt-in [social history extension](../society.md) adds age cohorts, seasonal grain, household shares and inheritance, terrain routes, councils, roads and provisioned raids. The optional [political extension](../politics.md) now records genealogy and marriages, competing council factions, regional territorial claims and provisioned conquest/repulse campaigns. The [governance extension](../governance.md) adds administrative wages, autonomy, legitimacy, secession and non-aggression agreements. The milestones below are archived ideas, not an active roadmap; complete individual demography, full crop/livestock catalogs, continuous planetary feedback, cultures, tactical/multi-front warfare and intercontinental trade remain unimplemented.

## Archived scope ideas

Build persistent societies whose settlements, people, institutions, resources and conflicts produce a causally connected history. The earlier ambition to match Dwarf Fortress is withdrawn. This archived document collects possible experiments; neither comparable simulation depth nor playable fortress/adventure modes are project goals or commitments.

The reference is the persistent world spanning centuries, trade, nobility, institutions, material-dependent crafts and revisitable sites described by [Bay 12's features page](https://bay12games.com/dwarves/features.html). Bay 12's [2012 development log](https://www.bay12games.com/dwarves/dev_2012.html) also describes succession, births, historical figures, holdings and site retirement. These are sources of inspiration, not acceptance targets; the architecture and milestones below are proposals for this project, not claims about Dwarf Fortress's internal implementation.

Civilizations appear **only on the inner continents**. This applies throughout history, not just initial seeding. The outer continent has no founded settlements, territorial claims, farming colonies, military bases or hidden starting societies. Ships may travel between eligible ports on the inner continents across the great lake. Outer-continent expeditions are outside the initial scope and must never silently become settlement mechanisms.

## Foundations to settle before the first society

1. Introduce stable `IslandId`, `RegionKey` and `SiteId`. Label connected central land independently of terrain-cell numbering. Eligibility requires the authoritative central-land mask, habitable dry ground and an island identity. Coarse cells with a small inner-land fraction are not sufficient.
2. Turn the current on-demand region snapshots into canonical, stitched regional chunks with halo cells, shared boundary river ports and fixed coordinates. Current region generation is useful for exploration, but patch-edge drainage is not yet a settlement authority. Persist terrain revisions and immutable generation provenance.
3. Distinguish geological history from historical time. Freeze large-scale tectonic epochs during ordinary civilization years. Monthly ecology, weather and water continue; exceptional terrain changes use explicit events. A society must not experience ten thousand years of uplift in one agricultural season.
4. Define an ownership contract: planet/ecology buffers own environmental inventories; sites own extracted goods, livestock and population; regional chunks own resolved terrain changes. Reservation and commit transfers prevent a mine or harvest being counted at both resolutions.
5. Version catalogs and schemas before creating persistent characters. Every reference is a stable ID, never a mutable TOML row offset. Store the actual catalogs with archives; old worlds keep their definitions.

## State model

| Record | Essential fields and relations |
| --- | --- |
| Civilization | Government, founding population, homeland, member sites, institutions, laws, diplomatic relationships; independent of species and culture |
| Culture | Language, names, values, customs, food preferences, marriage/inheritance rules, art traditions, material and biome preferences |
| Population cohort | Site, age bands, ancestry/species, culture, household class, occupations, health, food needs, births/deaths and migration |
| Historical figure | Stable identity, birth/death, family, relationships, skills, beliefs, offices, allegiances, possessions, motives and event links |
| Household | Members or cohort allocation, livelihood, dwelling, food and goods, wealth/debt, local obligations |
| Site | Canonical region footprint, founding/abandonment, rulers, residents, productive land, buildings, stockpiles, water rights, roads and port connections |
| Institution | Religion, temple, guild, merchant company, army, noble house, school, criminal organization; members, assets, rules, leadership |
| Office | Authority, duties, eligibility, succession rules, current and former holders; distinct from the person holding it |
| Resource operation | Mine, farm, fishery, forest holding or workshop; source reservation, workers, tools, input/output recipes, capacity and depletion |
| Item lot / artifact | Material, quantity, quality, creator, owner and location; important individual objects additionally retain identity and provenance |
| Route / shipment | Endpoints, allowed transport modes, distance, travel time, seasonal capacity, cargo, escort and interdiction risk |
| Conflict / army | Causes, participants, objectives, manpower, equipment, supply route, command, movement, casualties and settlement terms |
| Event | Monotonic ID, date, typed payload, actors, location, causal parents, inventory/population deltas and source revision |

Species, culture and political membership must remain separate. A conquered town does not automatically change ancestry, language or religion. Religions and merchant companies can span kingdoms. A political split preserves the identities of people, sites and institutions.

Use population cohorts for ordinary residents and persistent individuals for leaders, founders, artists, inventors, notable soldiers and consequential families. Promotion removes the person's allocation from a cohort; demotion never duplicates them. Preserve genealogical links to deceased people even when compressing routine records. Players must eventually be able to follow one person across site, army, office and culture changes.

## Scheduling and GPU boundary

Keep dense environmental and population calculations on GPU: carrying-capacity estimates, crop suitability, production constraints, cohort demographics, disease exposure, movement costs and batched route scoring. CPU scheduling handles sparse political graphs, kinship, event indexes and structured decisions. This is an explicit extension of the architecture: arbitrary social graphs should not force the environmental simulation onto CPU, and they should not require a GPU kernel for every royal marriage.

Use a monthly social tick with seasonal harvest/trade decisions and annual reviews. A deterministic event queue supports dated births, arrivals, battles and successions within a month. Read back compact site summaries, not the planet's full fields. Generate proposals in parallel, then arbitrate conflicting claims by stable ordering and explicit reservations. Counter-based randomness is keyed by world seed, system, date and entity ID; dispatch batching must not affect history.

Phase order: environment → production and inventory reservations → consumption and health → demographic changes → travel arrivals → political/military decisions → conflict resolution → event commit. Each phase reads a completed snapshot. No same-month feedback loop can repeatedly spend the same harvest.

## Milestones and acceptance gates

### 1. Geography, founding and a searchable first history

Add canonical island identities, the settlement eligibility predicate, deterministic naming, cultures, civilization records, site records and append-only typed events. Site types initially include villages, towns, hill forts, fishing ports and mining settlements. Score freshwater reliability, flood exposure, arable area, temperature, forest/ore access and transport. Place dispersed founders with finite households, tools and seed food, recording those starting inventories.

Permit more than one society per inner continent and allow empty unsuitable regions. Founding failures produce diagnostics rather than forcing towns onto ice, water or the outer continent. Territorial borders begin as travel-cost claims from inhabited sites, not ownership of an entire inner continent.

Gate: hundreds of seeds place every settlement and claim on central land. Save/load preserves names and event IDs. A map click opens a site's founding record and links its founders and civilization. Deliberately requesting an outer-continent founding fails through the same API used by normal expansion.

### 2. Subsistence, extraction and material economies

Implement crop calendars, grazing, domesticated livestock, fisheries, forestry, quarrying and finite regional mining reserves. Add labor allocation, food storage/spoilage, fuel, tools, housing and transport costs. Production recipes consume inputs with explicit units: ore plus fuel and labor yields metal, slag and losses; grain processing does not multiply calories.

Start with editable catalogs for roughly 12 crops, 8 domestic species, 30 goods and 40 recipes. This is an initial content budget, not a parity claim. Existing plant and ore IDs become references; wild ecological biomass is not automatically edible or harvestable. Regional mineral potential must be converted into a bounded extractable stock before miners can use it.

Harvest consumes plant biomass and nutrients through the environmental transfer ledger. Soil exhaustion, fallow, manure, irrigation and deforestation connect the economy back to ecology. Nutrient-rich waste is returned or exported explicitly. Do not charge maintenance twice when livestock move between ecological guild accounting and managed herds.

Gate: isolated sites survive only when production and reserves support consumption. Drought, crop failure and exhausted mines cause explainable shortages. Inventory balances include waste, spoilage and trade. No negative stocks or populations; no unlimited mining from a probability field.

### 3. Population, families and succession

Add age-structured births, mortality, disease, households, occupations, marriage, kinship, inheritance and migration. Establish rulers and offices with configurable election, appointment, hereditary and council succession. Resolve disputed succession through existing actors, legitimacy and institutions.

Historical figures have motivations and relationships that influence decisions, but cannot override physical travel time or inventory constraints. Founders' descendants can become rulers, merchants, soldiers or emigrants. Refugees retain origin and family links.

Gate: a ruler's death transfers an office and property exactly once. A killed parent cannot have later ordinary births without a prior dated conception. Migration conserves people and possessions. Named people cannot simultaneously reside in a site and be counted again in its army/cohort.

### 4. Trade, roads and island navigation

Implemented subset: [regional lake shipping](../shipping.md) now supplies actual surveyed sea paths, funded harbor assets, shared cargo capacity, closures and contact-driven diplomacy. Inland roads and intermediate-market trade are also implemented. Individual vessels/crews, navigable rivers, seasonal sailing, tolls, debt and piracy remain roadmap work.

Build land routes through regional passes and river crossings, navigable river segments, ports and great-lake shipping between inner continents. Roads and ships require materials, labor and maintenance. Travel time follows terrain, currents, season and vessel traits. Sea routes may cross water but can only establish eligible inner-continent ports.

Goods move in shipments with departure, transit and arrival state. Prices or barter values respond to scarcity and transport costs. Merchant companies, contracts, tolls, debt and piracy become extensions of the same ownership and event systems.

Gate: closing a pass or losing a port interrupts dependent supply. Ships cannot teleport cargo, duplicate it after checkpointing, or found an outer shore colony. Separate inner continents remain economically distinct until real routes connect them.

### 5. Politics, war and territorial change

Add treaties, tribute, alliances, grievances, raids, conquest, rebellions, secession and negotiated peace. Decisions use objectives, expected gain, relationships, legitimacy and supply. Armies consist of recruited people with finite equipment and food; travel and campaigning consume time and stocks.

Begin with regional battles and sieges, recording participants, casualties, captives, destruction and occupation. Local anatomical combat can later replace the battle resolver without changing army or person identities. Site destruction leaves ruins and displaced residents. A conquest changes government before it changes culture.

Gate: defeat reduces real manpower and equipment; a starving army cannot campaign forever. Territorial changes respect central-land eligibility. Civil wars split institutions and assets through explicit transfers. A war's declared cause, major battles and outcome can be followed in the history browser.

### 6. Religion, knowledge, arts and artifacts

Add belief traditions, congregations, temples, holy offices, festivals and schisms. Implement research prerequisites, discovery, teaching, written works, libraries and knowledge loss. Crafts and art draw on culturally learned forms and available materials. Important artifacts receive unique identities, creators, owners and transfer histories.

These systems should generate consequences: a destroyed library loses access to manuscripts; a displaced artisan brings skills elsewhere; a sacred object's theft changes relationships. Use structured facts to generate prose, not prose as authoritative simulation state.

Gate: artifacts have one owner/location at a time. Innovations spread through contact rather than instant global unlocks. Conflicting historical accounts can exist as beliefs, while the simulation retains an authoritative event record.

### 7. Persistent sites and playable-world foundations

Resolve towns into districts, households, farms, roads, workshops, walls, temples and cemeteries on canonical regional terrain. Building footprints obey slope, shoreline, water access and construction costs. Reserve vertical coordinates and excavated-volume records for mines and later fortress maps.

Support site activation/retirement contracts: an active local simulation temporarily owns named people, items and affected land; world history advances the rest. Retirement reconciles deltas once and preserves rooms, structures, ownership and history. Loading the same site must not regenerate away a battlefield, canal or ruined temple.

Gate: visit → modify → retire → advance history → revisit preserves identity and reconciles inventories. Two sites cannot both own the same historical figure or artifact. This was a proposed experiment in local interaction, not a commitment to fortress or adventure gameplay.

## Explorer, API and persistence

Expose `advance_history`, `inspect_site`, `inspect_entity`, `query_events`, `trace_cause`, `inspect_supply`, `found_site` and scenario APIs through the reusable generator interface. Add civilization/culture/religion overlays, borders, settlements, roads, shipping, trade shortages, wars and population maps. Provide a Legends-style browser with timelines and cross-linked people, families, offices, artifacts and places.

Use versioned archives with simulation clocks, catalogs, stable ID allocators, social snapshots, environmental transfer journals and event-log segments. Checkpoints occur after a committed month; mid-month saves are queued. Test exact continuation on one backend and tolerance-based environmental equivalence across hardware. Store material event payloads, not only random seeds. Keep indexes rebuildable and verify archive references and checksums on load.

Default initial history target: 250 social years, editable, with continuous continuation. Start calibration with 5–15 civilizations and 50–200 sites, then measure scaling toward thousands of sites and large historical populations. These are test settings, not a promise of fixed generation duration.

## Archived capability ideas (not current exit criteria)

| Capability | World-history target | Later local-gameplay dependency |
| --- | --- | --- |
| Founding, growth, decline, ruins | Proposed | Detailed building occupation |
| Cohorts, notable people, genealogy, succession | Proposed | Individual daily needs and jobs |
| Trade, crafts, finite resources, livelihoods | Proposed | Workshop operation and hauling |
| Diplomacy, raids, conquest, migration | Proposed | Tactical and anatomical combat |
| Religion, institutions, art, knowledge, artifacts | Proposed | Detailed performances and interactions |
| Historical browser and causal records | Proposed | Local memories and rumors |
| Persistent sites and reversible activation | Proposed foundation | Fortress/adventure modes |
| Excavation and subterranean settlements | Schema foundations | Voxel terrain, caves and fluid simulation |

Earlier validation ideas included 100-, 250- and 1,000-year seed suites; population and goods accounting; zero outer-continent settlements at every tick; acyclic ancestry; valid office intervals; artifact ownership uniqueness; reproducible checkpoint continuation; and bounded performance/memory growth. Include controlled famine, succession crisis, blockade, resource exhaustion, epidemic, rebellion and library-loss scenarios.

The first implementation slice should stop at a compelling, inspectable loop: **found villages → produce food and goods → grow families → exchange surplus → expand or fail → record why**. Subsequent milestones extend these same records and budgets rather than replacing a disposable civilization prototype.
