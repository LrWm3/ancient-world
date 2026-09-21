# System maintenance inventory

Inventory date: **2026-09-21**. Committed source baseline: **`9d8d870`**.

This is a maintenance map of the toy world generator, its implemented extensions, and its independent economics experiment. It inventories code and controls, not simulation accuracy, feature parity, or completion of earlier proposals. Permanent human settlements belong on the inner continents; Ancient World civilizations are not an implemented goal assumed by this inventory.

## Reading and maintaining this inventory

- Each entry describes implemented behavior, owning source, and selected guides/tests. Small helper modules and test fixtures belong to their parent feature; the coverage appendix lists every tracked runtime module, shader, catalog, evaluation tool and fixture explicitly.
- **Source work** is the latest reachable committer date (`%cs`), commit and subject touching the listed source scope. **Documentation/evidence** is the same calculation over the linked guides and test files. Neither is a fresh review date or proof that a test ran.
- Retained history begins at snapshot `f431224` (2026-09-10). A snapshot-only timestamp means “present by,” not original implementation date. Shared files can give several features the same timestamp without proving each changed.
- Tests may require a GPU or explicit ignored-test selection. This documentation pass did not run simulation, calibration or performance suites. The historical reviews at the end retain their original results and limitations.
- Registry defaults below describe new application setup at this revision. Archive state, explicit overrides, low-level constructors and prerequisite suppression can differ. Features outside the registry may use separate APIs/configuration; do not infer activation from their presence here.
- The pass began at `17e7fab` with unrelated economics changes in the worktree. Those changes were committed separately as `173f05a` and `9d8d870` during the pass; the inventory baseline was refreshed to include their published agreements/offers/allocation/competition modules. This update changes only this document; any later worktree changes are outside its pinned baseline.
- Proposal-only documents (including broader currency/general-credit designs and archived roadmaps) are not evidence of an implementation. See [integration worklist](integration-worklist.md) for outstanding work; absence from this inventory is not a claim that an idea was rejected.

Navigation: [Planet, ecology and regional continuity](#planet-ecology-and-regional-continuity) · [Founding, population, work and household life](#founding-population-work-and-household-life) · [Production, construction and logistics](#production-construction-and-logistics) · [Credit, obligations and monetary evidence](#credit-obligations-and-monetary-evidence) · [Politics, institutions, conflict and culture](#politics-institutions-conflict-and-culture) · [Expeditions, historical evidence and naming](#expeditions-historical-evidence-and-naming) · [Application, catalogs and evidence](#application-catalogs-and-evidence) · [Standalone economics experiment — not integrated into the world](#standalone-economics-experiment--not-integrated-into-the-world) · [Registry](#registry-coverage-and-defaults) · [File coverage](#file-coverage-appendix) · [Historical reviews](#historical-review-records)

## Planet, ecology and regional continuity

| Feature and source scope | Implemented behavior and boundary | Source work | Documentation/evidence and checks |
| --- | --- | --- | --- |
| <a id="planet"></a>**Planet generation and compute orchestration**<br>[gpu.rs](../src/gpu.rs), [grid.rs](../src/grid.rs), [shaders/simulation.wgsl](../shaders/simulation.wgsl) | Cube-sphere topology, constrained inner continents/great lake/Ancient World geography, plate-field tectonics, elevation, seasonal climate, weather, erosion, drainage and lake relaxation; GPU limits, convergence and epoch boundaries. These are coupled toy models, not free continental drift. | 2026-09-13 · `8bb4b68` — Name geological spatial offsets without changing generated fields | 2026-09-12 · `71ccd52` — Correct regional slopes and dated social evidence; isolate alloy controls<br>[default-generation-performance.md](../docs/default-generation-performance.md), [lake-polling-review.md](../docs/lake-polling-review.md), [benchmarks.md](../docs/benchmarks.md), [tests/core.rs](../tests/core.rs), [tests/gpu.rs](../tests/gpu.rs), [tests/geology.rs](../tests/geology.rs) |
| <a id="aquatic"></a>**Ecological stocks, layered producers and aquatic compartments**<br>[ecology.rs](../src/ecology.rs), [shaders/ecology.wgsl](../shaders/ecology.wgsl) | C/N/P and water budgets, finite geological substrates, solar/chemical production, microbes, five terrestrial layers, aquatic production, lake exchange and biological recycling; independent ecology resolution and scenario interventions. | 2026-09-13 · `713db74` — Place ecology parameters at the owning shader header | 2026-09-12 · `f68a1ae` — Add opt-in regional wildlife thermal preferences and migration inheritance<br>[ecology.md](../docs/ecology.md), [geochemical-habitats.md](../docs/geochemical-habitats.md), [living-scenarios.md](../docs/living-scenarios.md), [tests/ecology.rs](../tests/ecology.rs), [tests/living_scenarios.rs](../tests/living_scenarios.rs) |
| <a id="wildlife"></a>**Food webs, dispersal and ecotypes**<br>[ecology.rs](../src/ecology.rs), [shaders/ecology.wgsl](../shaders/ecology.wgsl) | Competing producers, terrestrial/aquatic guilds, size-aware diets, migration, island assembly and thermal preferences. Regional biomass and traits do not imply individual creatures or unrestricted speciation. | 2026-09-13 · `713db74` — Place ecology parameters at the owning shader header | 2026-09-12 · `3d244c8` — Add measured aquatic thermal and lake polling sensitivity controls<br>[food-webs.md](../docs/food-webs.md), [wildlife-assembly.md](../docs/wildlife-assembly.md), [wildlife-trophic-stability.md](../docs/wildlife-trophic-stability.md), [wildlife-thermal-ecotypes.md](../docs/wildlife-thermal-ecotypes.md), [tests/ecology.rs](../tests/ecology.rs) |
| <a id="sunlight"></a>**Seasonal sunlight**<br>[shaders/sunlight.wgsl](../shaders/sunlight.wgsl) | Latitude, axial tilt, hemisphere and day-length response shared with ecological light calculations. | 2026-09-13 · `0b088f9` — Name seasonal sunlight shader parameters | 2026-09-10 · `7308c9f` — Correct ecological sunlight for hemisphere, axial tilt and day length<br>[ecological-sunlight.md](../docs/ecological-sunlight.md), [tests/sunlight.rs](../tests/sunlight.rs) |
| <a id="regional"></a>**Regional terrain, geological provinces and strata**<br>[region.rs](../src/region.rs), [shaders/regional.wgsl](../shaders/regional.wgsl), [shaders/region_compute.wgsl](../shaders/region_compute.wgsl) | Higher-resolution terrain, slopes, drainage, rock columns, deposits, faults and volcanic features; inherited planet context and regional inspection. Columns and surveys are not excavatable voxel terrain. | 2026-09-13 · `e3144f5` — Name visual regional refinement parameters | 2026-09-12 · `71ccd52` — Correct regional slopes and dated social evidence; isolate alloy controls<br>[regions.md](../docs/regions.md), [geological-regions.md](../docs/geological-regions.md), [geological-provinces.md](../docs/geological-provinces.md), [stratigraphic-columns.md](../docs/stratigraphic-columns.md), [tests/geology.rs](../tests/geology.rs) |
| <a id="living"></a>**Living history, surveys and monthly environment**<br>[history_environment.rs](../src/history_environment.rs), [shaders/history_environment.wgsl](../shaders/history_environment.wgsl), [shaders/history_weather.wgsl](../shaders/history_weather.wgsl) | History-sized GPU observations, monthly weather and sparse survey/readback paths connect ecology and settlements without requiring a full terrain snapshot every month. | 2026-09-13 · `cdc11e3` — Share history weather and settlement dispatch parameters | 2026-09-13 · `27bddbd` — Add paid recovery of abandoned bulk stocks through finite land cargo<br>[living-history.md](../docs/living-history.md), [history-environment-readback.md](../docs/history-environment-readback.md), [tests/history_environment.rs](../tests/history_environment.rs), [tests/living.rs](../tests/living.rs) |
| <a id="hazards"></a>**Floods and delayed cargo spoilage**<br>[hazards.rs](../src/hazards.rs) | Exposure damages towns and obstructs travel; delayed deliveries retain their inventories and incur recorded spoilage. Flood diagnostics distinguish terrain and settlement effects. | 2026-09-13 · `27bddbd` — Add paid recovery of abandoned bulk stocks through finite land cargo | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification<br>[flood-history.md](../docs/flood-history.md), [flood-fixes-results.md](../docs/flood-fixes-results.md), [delayed-cargo-spoilage.md](../docs/delayed-cargo-spoilage.md), [tests/living.rs](../tests/living.rs) |
| <a id="resources"></a>**Canonical resource stocks and extraction**<br>[resources.rs](../src/resources.rs), [regional_mining.rs](../src/regional_mining.rs) | Shared finite sources, mining controls, source allocation/depletion and regional survey continuity; extraction claims settle against canonical reserves. | 2026-09-12 · `a82fca7` — Share agricultural and workforce constants between Rust and WGSL | 2026-09-12 · `dd2350b` — Preserve canonical source depletion as historical evidence<br>[shared-resources.md](../docs/shared-resources.md), [regional-mining-control.md](../docs/regional-mining-control.md), [source-depletion-evidence.md](../docs/source-depletion-evidence.md), [tests/resources.rs](../tests/resources.rs) |
| <a id="returns"></a>**Environmental returns and land recovery**<br>[environmental_returns.rs](../src/environmental_returns.rs), [shaders/managed_returns.wgsl](../shaders/managed_returns.wgsl) | Managed removals and returned water/nutrients/sediment feed living ecology, downstream receiving waters and abandoned-land recovery; frozen history has a different boundary contract. | 2026-09-12 · `f68a1ae` — Add opt-in regional wildlife thermal preferences and migration inheritance | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification<br>[environmental-returns.md](../docs/environmental-returns.md), [cross-scale-coupling.md](../docs/cross-scale-coupling.md), [tests/environmental_returns.rs](../tests/environmental_returns.rs) |
| <a id="nutrients"></a>**Farm replenishment, manure retention and phosphorus runoff**<br>[production.rs](../src/production.rs), [economy.rs](../src/economy.rs), [shaders/economy.wgsl](../shaders/economy.wgsl), [environmental_returns.rs](../src/environmental_returns.rs) | Finite source release, crop N/P/water requests and fulfillment, retained manure, runoff mobility and ecological returns. Retention experiments improve some trajectories but do not establish a sustainable equilibrium. | 2026-09-14 · `1c0bfb7` — Add farm phosphorus retention controls and century comparisons | 2026-09-17 · `5bfaf42` — Document matched century nutrient tests without settlement expansion<br>[farm-nutrient-retention.md](../docs/farm-nutrient-retention.md), [fixed-footprint-nutrients.md](../docs/fixed-footprint-nutrients.md), [growth-nutrient-screen.md](../docs/growth-nutrient-screen.md), [tests/farm_nutrients.rs](../tests/farm_nutrients.rs), [tests/nutrient_retention.rs](../tests/nutrient_retention.rs) |
| <a id="geology"></a>**Planetary tectonics, geology and mineral potential**<br>[gpu.rs](../src/gpu.rs), [shaders/simulation.wgsl](../shaders/simulation.wgsl) | Plate membership/motion fields, boundary activity, crust age, rock exposure and finite-depth stratigraphic/deposit context seed regional geology and resource potential; the designed land/water template constrains terrain evolution. | 2026-09-13 · `8bb4b68` — Name geological spatial offsets without changing generated fields | 2026-09-10 · `a256f21` — Generate setting-driven geological regions and clarify rock maps<br>[geological-provinces.md](../docs/geological-provinces.md), [stratigraphic-columns.md](../docs/stratigraphic-columns.md), [tests/geology.rs](../tests/geology.rs) |
| <a id="hydrology"></a>**Basins, rivers, lakes and sediment routing**<br>[gpu.rs](../src/gpu.rs), [shaders/simulation.wgsl](../shaders/simulation.wgsl) | Drainage rebuilding, depression/spill propagation, routing rank, flow accumulation and lake relaxation couple runoff, groundwater storage, erosion and deposition; convergence limits are surfaced rather than claiming every bounded solve finished. | 2026-09-13 · `8bb4b68` — Name geological spatial offsets without changing generated fields | 2026-09-12 · `71ccd52` — Correct regional slopes and dated social evidence; isolate alloy controls<br>[regions.md](../docs/regions.md), [default-generation-performance.md](../docs/default-generation-performance.md), [lake-polling-review.md](../docs/lake-polling-review.md), [tests/gpu.rs](../tests/gpu.rs), [tests/core.rs](../tests/core.rs) |
| <a id="climate"></a>**Seasonal climate and transient weather**<br>[gpu.rs](../src/gpu.rs), [shaders/simulation.wgsl](../shaders/simulation.wgsl), [shaders/history_weather.wgsl](../shaders/history_weather.wgsl) | Temperature, wind, moisture, precipitation, snow/melt and elevation effects provide environmental inputs to ecology and history; geological epochs and living-history months have distinct update paths. | 2026-09-13 · `8bb4b68` — Name geological spatial offsets without changing generated fields | 2026-09-13 · `27bddbd` — Add paid recovery of abandoned bulk stocks through finite land cargo<br>[living-history.md](../docs/living-history.md), [ecological-sunlight.md](../docs/ecological-sunlight.md), [tests/living.rs](../tests/living.rs), [tests/history_environment.rs](../tests/history_environment.rs) |

## Founding, population, work and household life

| Feature and source scope | Implemented behavior and boundary | Source work | Documentation/evidence and checks |
| --- | --- | --- | --- |
| <a id="history"></a>**Monthly schedule and history authority**<br>[civilization.rs](../src/civilization.rs), [shaders/civilization.wgsl](../shaders/civilization.wgsl) | Open, Reserve, Execute/settle, Respond and Close coordinate observations, dated grants, committed effects, causal events and census/accounting checks; includes candidate selection and settlement lifecycle. | 2026-09-17 · `f380350` — Require finite parent planting supplies for daughter founding | 2026-09-17 · `f380350` — Require finite parent planting supplies for daughter founding<br>[monthly-schedule.md](../docs/monthly-schedule.md), [civilizations.md](../docs/civilizations.md), [settlement-lifecycle.md](../docs/settlement-lifecycle.md), [tests/civilization.rs](../tests/civilization.rs) |
| <a id="founding"></a>**Arrival provisions and daughter settlement supplies**<br>[civilization/founding_provisions.rs](../src/civilization/founding_provisions.rs), [civilization/founding_seeds.rs](../src/civilization/founding_seeds.rs), [civilization/daughter.rs](../src/civilization/daughter.rs) | Declared arrival food/storage support; daughter founding transfers actual parent households and finite planting supplies instead of granting new arrival inventories. | 2026-09-17 · `f380350` — Require finite parent planting supplies for daughter founding | 2026-09-17 · `f380350` — Require finite parent planting supplies for daughter founding<br>[four-year-founding-provisions.md](../docs/four-year-founding-provisions.md), [daughter-planting-supplies.md](../docs/daughter-planting-supplies.md), [resident-daughter-founding.md](../docs/resident-daughter-founding.md), [tests/founding_food.rs](../tests/founding_food.rs) |
| <a id="expansion"></a>**Growth controls and settlement admission**<br>[civilization/growth.rs](../src/civilization/growth.rs) | Optional growth interventions, land/candidate/census ceilings and annual admission reason counts; experiments separate possible land cells from funded habitable settlements. | 2026-09-17 · `f380350` — Require finite parent planting supplies for daughter founding | 2026-09-14 · `9370d38` — Diagnose growth reversals with monthly food and crop boundary probes<br>[settlement-growth-experiment.md](../docs/settlement-growth-experiment.md), [population-cap-screen.md](../docs/population-cap-screen.md), [inner-land-cell-capacity.md](../docs/inner-land-cell-capacity.md), [tests/civilization.rs](../tests/civilization.rs) |
| <a id="society"></a>**Aggregate demography, households and seasons**<br>[society.rs](../src/society.rs), [shaders/society.wgsl](../shaders/society.wgsl) | Cohort aging, births/deaths, food/work exposure, household stocks, routes and town support; aggregate authority remains available alongside individual refinement. | 2026-09-14 · `264b8f1` — Add staged growth experiments with restartable population gates | 2026-09-12 · `c34218f` — Add constructed defenses and supply-limited sieges with shared military freight<br>[society.md](../docs/society.md), [illness-and-work.md](../docs/illness-and-work.md), [tests/society.rs](../tests/society.rs), [tests/health_labor.rs](../tests/health_labor.rs) |
| <a id="residents"></a>**Resident identity and individual demography**<br>[population_registry.rs](../src/population_registry.rs), [individual_demography.rs](../src/individual_demography.rs) | Stable people, resident rosters, birthdays, births/deaths, membership and presence reconciliation. Archive conversion and aggregate mode are explicit authority choices. | 2026-09-14 · `264b8f1` — Add staged growth experiments with restartable population gates | 2026-09-11 · `000507d` — Refresh history documentation status and navigation<br>[resident-rosters.md](../docs/resident-rosters.md), [individual-demography.md](../docs/individual-demography.md), [population-reconciliation.md](../docs/population-reconciliation.md), [tests/civilization.rs](../tests/civilization.rs) |
| <a id="resolution"></a>**Projection, refinement and reconciliation**<br>[resolution.rs](../src/resolution.rs), [continuity_fixture.rs](../src/continuity_fixture.rs) | Aggregate expectations, individual outcomes, comparable metrics and once-only commits; shared continuity fixtures support boundary checks, not a second population ledger. | 2026-09-12 · `9865f31` — Name configuration, household policy and material processing constants | 2026-09-11 · `384607d` — Staff merchant vessels with bounded named participation and household wages<br>[resolution-framework.md](../docs/resolution-framework.md), [resolution-framework-verification.md](../docs/resolution-framework-verification.md), [tests/civilization.rs](../tests/civilization.rs) |
| <a id="participation"></a>**Activity commitments and labor receipts**<br>[participation.rs](../src/participation.rs), [labor.rs](../src/labor.rs) | Bounded named availability, eligibility, requests, grants, actual effort, release and absence; occupation is separate from monthly assignment. | 2026-09-14 · `7289927` — Add funded ruin resettlement and dormant ownership expiry | 2026-09-12 · `d374a11` — Separate skilled production capacity from actual worker time<br>[individual-participation.md](../docs/individual-participation.md), [work-execution-boundaries.md](../docs/work-execution-boundaries.md), [production-experience.md](../docs/production-experience.md), [tests/health_labor.rs](../tests/health_labor.rs) |
| <a id="farmwork"></a>**Farm, extraction and construction participation**<br>[agriculture_participation.rs](../src/agriculture_participation.rs), [civilization/production_forecast.rs](../src/civilization/production_forecast.rs) | GPU production forecasts feed named worker reservations and effective labor allowances; experience, household earnings and settlement of actual work remain separate from physical output. | 2026-09-17 · `f380350` — Require finite parent planting supplies for daughter founding | 2026-09-12 · `d374a11` — Separate skilled production capacity from actual worker time<br>[production-participation.md](../docs/production-participation.md), [production-experience.md](../docs/production-experience.md), [husbandry-attendance.md](../docs/husbandry-attendance.md), [tests/economy.rs](../tests/economy.rs) |
| <a id="workshops"></a>**Workshop individual refinement**<br>[workshop_resolution.rs](../src/workshop_resolution.rs) | Workshop requests, named commitments, effective completion and household wages feed the shared resolution receipts; payments cannot imply free output. | 2026-09-12 · `b9a41b2` — Name vocabulary, social memory and workshop resolution policies | 2026-09-11 · `db6d2cb` — Connect agricultural attendance to cultivation and household earnings<br>[resolution-framework.md](../docs/resolution-framework.md), [resident-payroll-balance.md](../docs/resident-payroll-balance.md), [tests/economy.rs](../tests/economy.rs) |
| <a id="domestic"></a>**Domestic groups, dependents and care**<br>[domestic.rs](../src/domestic.rs), [domestic/assistance.rs](../src/domestic/assistance.rs), [domestic/resolution.rs](../src/domestic/resolution.rs), [kin_support.rs](../src/kin_support.rs) | Family membership, dependent care, nearby assistance and bounded work consequences; pooled-care counterfactuals observe rather than duplicate actual assignments. | 2026-09-12 · `822b0bb` — Share demographic rates and name resident and office policies | 2026-09-12 · `0450bd6` — Allow spare family care capacity to support neighboring dependents<br>[domestic-participation.md](../docs/domestic-participation.md), [neighbor-care.md](../docs/neighbor-care.md), [care-resolution.md](../docs/care-resolution.md), [domestic-surplus-assistance.md](../docs/domestic-surplus-assistance.md), [tests/health_labor.rs](../tests/health_labor.rs) |
| <a id="relocation"></a>**Household journeys and travel comparison**<br>[relocation.rs](../src/relocation.rs), [relocation/comparison.rs](../src/relocation/comparison.rs) | Funded departures, provisions, named/aggregate travelers, arrival, attrition and retained family identity; dated comparison receipts explain forecast versus realized travel losses. | 2026-09-14 · `7289927` — Add funded ruin resettlement and dormant ownership expiry | 2026-09-12 · `c306b12` — Report conditional and realized relocation travel attrition<br>[household-relocation.md](../docs/household-relocation.md), [relocation-travel-comparison.md](../docs/relocation-travel-comparison.md), [individual-travel-verification.md](../docs/individual-travel-verification.md), [tests/society.rs](../tests/society.rs) |
| <a id="resettlement"></a>**Ruin resettlement and expiring claims**<br>[relocation/resettlement.rs](../src/relocation/resettlement.rs) | Funded household footholds reuse abandoned sites; provisions, tenure notices, claim challenges/expiry and failed travel retain explicit accounting. | 2026-09-14 · `118bc9c` — Enable ruin resettlement by default with explicit opt-out | 2026-09-14 · `118bc9c` — Enable ruin resettlement by default with explicit opt-out<br>[ruin-resettlement.md](../docs/ruin-resettlement.md), [tests/civilization.rs](../tests/civilization.rs) |
| <a id="retail"></a>**Household ownership, income and purchasing**<br>[household_economy.rs](../src/household_economy.rs), [household_economy/nutrition.rs](../src/household_economy/nutrition.rs) | Retail budgets, commons, payroll/dividends, actual earners/dependents and food allocation; needs-based access and gradual aggregate nutrition are distinct optional policies. Allocation views do not consume food twice. | 2026-09-14 · `17d8639` — Add bounded municipal food purchasing relief and evaluate growth | 2026-09-13 · `ca103a1` — Test local surplus-wallet food support without removing retail revenue<br>[household-economy.md](../docs/household-economy.md), [household-adult-payroll.md](../docs/household-adult-payroll.md), [food-access-and-nutritional-stress.md](../docs/food-access-and-nutritional-stress.md), [tests/rations.rs](../tests/rations.rs), [tests/economy.rs](../tests/economy.rs) |
| <a id="familycash"></a>**Family assistance and local solidarity**<br>[household_economy/family_support.rs](../src/household_economy/family_support.rs), [household_economy/solidarity.rs](../src/household_economy/solidarity.rs) | Opening-wallet gifts and optional non-kin food assistance transfer finite money before ordinary retail, with recipient caps and receipts. | 2026-09-13 · `ca103a1` — Test local surplus-wallet food support without removing retail revenue | 2026-09-13 · `e563baf` — Observe demographic exposure windows and identify early physical food shortages<br>[household-family-support.md](../docs/household-family-support.md), [food-solidarity.md](../docs/food-solidarity.md), [tests/rations.rs](../tests/rations.rs) |
| <a id="welfare"></a>**Council and municipal relief policies**<br>[household_economy/council_allocation.rs](../src/household_economy/council_allocation.rs), [household_economy/municipal_relief.rs](../src/household_economy/municipal_relief.rs), [household_economy/policy.rs](../src/household_economy/policy.rs) | Political distribution platforms, annual working-cash support, administration allowance and needs-first reserves compete for real public cash; municipal retail assistance is a separate payer. | 2026-09-14 · `9ddbcd8` — Add needs-first municipal surplus allocation and matched continuation tests | 2026-09-14 · `9ddbcd8` — Add needs-first municipal surplus allocation and matched continuation tests<br>[council-administration-allowance.md](../docs/council-administration-allowance.md), [council-welfare-reserves.md](../docs/council-welfare-reserves.md), [municipal-food-relief.md](../docs/municipal-food-relief.md), [municipal-needs-first.md](../docs/municipal-needs-first.md), [household-distribution-politics.md](../docs/household-distribution-politics.md), [tests/rations.rs](../tests/rations.rs) |
| <a id="estates"></a>**Household inheritance and unclaimed estates**<br>[household_economy/inheritance.rs](../src/household_economy/inheritance.rs), [household_economy/reclamation.rs](../src/household_economy/reclamation.rs) | Economic ownership succession, retained cash and bounded office-assisted reclamation; neither property nor reclaiming an estate automatically confers political office. | 2026-09-13 · `665476f` — Divide local estates among eligible descendants and audit cash recovery | 2026-09-13 · `665476f` — Divide local estates among eligible descendants and audit cash recovery<br>[household-estate-inheritance.md](../docs/household-estate-inheritance.md), [unclaimed-estate-reclamation.md](../docs/unclaimed-estate-reclamation.md), [tests/society.rs](../tests/society.rs) |
| <a id="wealthtax"></a>**Progressive household cash tax**<br>[household_economy/wealth_tax.rs](../src/household_economy/wealth_tax.rs) | Annual marginal cash taxation bounded by delivered administration, with transfers and receipts. This is not a valuation/tax of all physical wealth. | 2026-09-13 · `d48267d` — Add progressive household cash tax and funded practical research pilots | 2026-09-13 · `d48267d` — Add progressive household cash tax and funded practical research pilots<br>[circulation-and-workshop-recovery.md](../docs/circulation-and-workshop-recovery.md), [tests/society.rs](../tests/society.rs) |
| <a id="clothing"></a>**Household clothing and replacement demand**<br>[household_economy/clothing.rs](../src/household_economy/clothing.rs) | Finite cloth purchases become owned wardrobes, wear and replacement demand; links household spending to workshops. | 2026-09-13 · `b0a6e3e` — Connect bounded household clothing demand to adaptive quotes | 2026-09-13 · `b0a6e3e` — Connect bounded household clothing demand to adaptive quotes<br>[household-clothing.md](../docs/household-clothing.md), [tests/economy.rs](../tests/economy.rs) |
| <a id="health"></a>**Traveler-linked contagious illness**<br>[contagion.rs](../src/contagion.rs) | One aggregate SEIR infection, household travel partitions and arrival-only cargo contact; feeds existing illness/work/demography without a second mortality debit. No individual infection histories or army/expedition disease pools. | 2026-09-13 · `27bddbd` — Add paid recovery of abandoned bulk stocks through finite land cargo | 2026-09-12 · `9d333a3` — Add conserved SEIR partitions and traveler-borne exposure<br>[contagious-illness.md](../docs/contagious-illness.md), [tests/health_labor.rs](../tests/health_labor.rs) |
| <a id="social"></a>**Social projections, pressures and local memory**<br>[social_state.rs](../src/social_state.rs), [social_memory.rs](../src/social_memory.rs) | Age/livelihood, food/cash/ownership and affiliation projections, remembered pressures, destination food reports, relief experience and reciprocity; dated evidence has local reach and decays. | 2026-09-13 · `5b0b2c8` — Name research, relief, social pressure and explorer parameters | 2026-09-10 · `853f2de` — Connect local aid memory, staffed fleets and institutional research<br>[social-indicators.md](../docs/social-indicators.md), [memory-fleets-and-research.md](../docs/memory-fleets-and-research.md), [tests/society.rs](../tests/society.rs) |
| <a id="warnings"></a>**Traveling danger warnings**<br>[route_warnings.rs](../src/route_warnings.rs) | Observed unsafe-route evidence carried by survivors biases later preferences; no omniscient report distribution or replacement of physical route checks. | 2026-09-12 · `b6e35c1` — Name continuity and institutional tuning constants in owning modules | 2026-09-12 · `b91d33a` — Carry dated route warnings through surviving household arrivals<br>[traveling-route-warnings.md](../docs/traveling-route-warnings.md), [tests/society.rs](../tests/society.rs) |

## Production, construction and logistics

| Feature and source scope | Implemented behavior and boundary | Source work | Documentation/evidence and checks |
| --- | --- | --- | --- |
| <a id="agriculture"></a>**Crop, livestock and fishery production**<br>[agriculture.rs](../src/agriculture.rs), [shaders/economy.wgsl](../shaders/economy.wgsl) | Editable crops, seasonal canopy/biomass/harvest state, nutrients/water, planting stocks, livestock feed/manure and managed aquatic withdrawals. Seasonal calibration is a game model, not agronomic validation. | 2026-09-14 · `1c0bfb7` — Add farm phosphorus retention controls and century comparisons | 2026-09-14 · `9cbea32` — Verify sustained settlement growth and document food access and fiscal limits<br>[crop-price-revisit.md](../docs/crop-price-revisit.md), [crop-resource-competition.md](../docs/crop-resource-competition.md), [founding-farm-balance.md](../docs/founding-farm-balance.md), [tests/economy.rs](../tests/economy.rs), [tests/farm_nutrients.rs](../tests/farm_nutrients.rs) |
| <a id="fisheries"></a>**Fishing effort, gear and local opportunity cost**<br>[agriculture.rs](../src/agriculture.rs), [economy.rs](../src/economy.rs), [shaders/economy.wgsl](../shaders/economy.wgsl) | Adaptive effort, accessible biomass, primitive gear and labor opportunity-cost controls connect fisheries to food and timber/tool use. | 2026-09-14 · `1c0bfb7` — Add farm phosphorus retention controls and century comparisons | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification<br>[adaptive-fisheries.md](../docs/adaptive-fisheries.md), [fishery-opportunity-cost.md](../docs/fishery-opportunity-cost.md), [natural-fishery-trajectories.md](../docs/natural-fishery-trajectories.md), [timber-fisheries.md](../docs/timber-fisheries.md), [tests/adaptive_fisheries.rs](../tests/adaptive_fisheries.rs) |
| <a id="market"></a>**Goods, demand, prices and network trade**<br>[economy.rs](../src/economy.rs), [shaders/economy.wgsl](../shaders/economy.wgsl) | Catalog goods/recipes, inventory/money/CNP accounting, demand-aware prices, supplier quotes and constrained shipments; prices remain abstract game signals. | 2026-09-14 · `1c0bfb7` — Add farm phosphorus retention controls and century comparisons | 2026-09-14 · `264b8f1` — Add staged growth experiments with restartable population gates<br>[economy.md](../docs/economy.md), [demand-economy.md](../docs/demand-economy.md), [supplier-quotes.md](../docs/supplier-quotes.md), [market-demand-ownership.md](../docs/market-demand-ownership.md), [tests/economy.rs](../tests/economy.rs), [tests/markets.rs](../tests/markets.rs) |
| <a id="production"></a>**Production planning and essential maintenance**<br>[production.rs](../src/production.rs) | Bounded land/labor/material allocation, adaptive labor, food-security priority, replacement tools, specialization and maintenance; storage/housing/waterworks persist and require resources. | 2026-09-14 · `1c0bfb7` — Add farm phosphorus retention controls and century comparisons | 2026-09-11 · `57de102` — Standardize great-lake landmass terminology as inner continents<br>[adaptive-industries.md](../docs/adaptive-industries.md), [food-security-labor.md](../docs/food-security-labor.md), [site-assets.md](../docs/site-assets.md), [waterworks-recovery.md](../docs/waterworks-recovery.md), [tests/storage.rs](../tests/storage.rs), [tests/housing.rs](../tests/housing.rs), [tests/waterworks.rs](../tests/waterworks.rs) |
| <a id="operators"></a>**Enterprise ownership and workshop capital**<br>[enterprises.rs](../src/enterprises.rs), [enterprises/feasibility.rs](../src/enterprises/feasibility.rs) | Operators lease/service workshops, pay staff, maintain capital and plan against inputs and demand; returns depend on completed production, not merely paid work. | 2026-09-13 · `43ff17e` — Account for public service labor before funding workshop shifts | 2026-09-13 · `1994758` — Evaluate spare workshop capacity and separate cash stocks from spending<br>[workshop-operators.md](../docs/workshop-operators.md), [workshop-capital.md](../docs/workshop-capital.md), [workshop-input-feasibility.md](../docs/workshop-input-feasibility.md), [demand-aware-workshop-staffing.md](../docs/demand-aware-workshop-staffing.md), [tests/economy.rs](../tests/economy.rs) |
| <a id="orders"></a>**Funded workshop service orders**<br>[enterprises/orders.rs](../src/enterprises/orders.rs) | Town-owned escrow pays completed future services; procurement, contract-aware staffing, demand staffing and shortfall observations are separate controls. | 2026-09-13 · `e401393` — Bound workshop staffing by usable industrial orders and stock | 2026-09-13 · `4d01c9d` — Record mixed outcomes from contract-aware workshop staffing<br>[workshop-service-orders.md](../docs/workshop-service-orders.md), [workshop-service-procurement.md](../docs/workshop-service-procurement.md), [contract-aware-workshop-staffing.md](../docs/contract-aware-workshop-staffing.md), [service-order-shortfall-observations.md](../docs/service-order-shortfall-observations.md), [tests/economy.rs](../tests/economy.rs) |
| <a id="materials"></a>**Material substitution and scalable facilities**<br>[materials.rs](../src/materials.rs), [facilities.rs](../src/facilities.rs) | Wood/ceramic/metal suitability, embodied materials, component condition, repair orders and rooms support objects and institutions with variable construction investment. | 2026-09-13 · `d1c91cb` — Share production asset costs and name enterprise policies | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[material-objects-and-facilities.md](../docs/material-objects-and-facilities.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="metals"></a>**Mineral processing and alloys**<br>[metallurgy.rs](../src/metallurgy.rs), [tool_access.rs](../src/tool_access.rs) | Host-specific mineral processing, finite refining/alloy recipes, tool material quality/access and substitute retention affect extraction and repair without creating source mass. | 2026-09-12 · `a82fca7` — Share agricultural and workforce constants between Rust and WGSL | 2026-09-13 · `3ba7fc4` — Retain useful alloy tool stocks in production and trade targets<br>[mineral-processing.md](../docs/mineral-processing.md), [alloy-processing.md](../docs/alloy-processing.md), [tool-substitution-retention.md](../docs/tool-substitution-retention.md), [tests/mineral_processing.rs](../tests/mineral_processing.rs), [tests/alloy_processing.rs](../tests/alloy_processing.rs) |
| <a id="exports"></a>**Export contracts and payment custody**<br>[export_contracts.rs](../src/export_contracts.rs), [export_contracts/identities.rs](../src/export_contracts/identities.rs), [export_contracts/payments.rs](../src/export_contracts/payments.rs) | Persistent contract identity, delivery/default records and optional buyer-funded delivery escrow separate departure, arrival and earned sale income; connects to credit evidence. | 2026-09-14 · `264b8f1` — Add staged growth experiments with restartable population gates | 2026-09-14 · `264b8f1` — Add staged growth experiments with restartable population gates<br>[export-contracts.md](../docs/export-contracts.md), [export-default-recovery.md](../docs/export-default-recovery.md), [tests/markets.rs](../tests/markets.rs) |
| <a id="shipping"></a>**Ports, harbors and sea trade**<br>[shipping.rs](../src/shipping.rs) | Harbor work, staged access, finite fleets/port capacity and shipping links connect inner continents; committed funding and arrival timing matter. | 2026-09-14 · `b2dfb80` — Make food connection planning horizon explicit and evaluate annual lookahead | 2026-09-13 · `ece3f1e` — Trace founding farm output and identify harvest-aware staffing gap<br>[shipping.md](../docs/shipping.md), [inland-sea-freight.md](../docs/inland-sea-freight.md), [harbor-work.md](../docs/harbor-work.md), [early-food-and-staged-harbors.md](../docs/early-food-and-staged-harbors.md), [tests/shipping.rs](../tests/shipping.rs) |
| <a id="vessels"></a>**Vessels and named crew service**<br>[vessels.rs](../src/vessels.rs), [vessels/crews.rs](../src/vessels/crews.rs), [vessels/resolution.rs](../src/vessels/resolution.rs) | Ships, condition, crew funding, named port-service assignments, experience and conditional labor receipts; preserves preceding-month prepaid voyage capacity. | 2026-09-13 · `faa1c5e` — Trace early food shortages and test materially bounded staged harbor openings | 2026-09-12 · `d374a11` — Separate skilled production capacity from actual worker time<br>[merchant-crew-participation.md](../docs/merchant-crew-participation.md), [merchant-crew-productivity.md](../docs/merchant-crew-productivity.md), [committed-crew-reservations.md](../docs/committed-crew-reservations.md), [tests/shipping.rs](../tests/shipping.rs) |
| <a id="freight"></a>**Land corridors and road upkeep**<br>[freight.rs](../src/freight.rs), [road_upkeep.rs](../src/road_upkeep.rs) | Shared corridor reservations, intermediate freight, road capacity and funded upkeep constrain cargo and relief. Cargo retains reserved routes when delayed. | 2026-09-12 · `9865f31` — Name configuration, household policy and material processing constants | 2026-09-14 · `264b8f1` — Add staged growth experiments with restartable population gates<br>[land-freight-reservations.md](../docs/land-freight-reservations.md), [road-freight-capacity.md](../docs/road-freight-capacity.md), [road-upkeep.md](../docs/road-upkeep.md), [intermediate-freight.md](../docs/intermediate-freight.md), [tests/markets.rs](../tests/markets.rs) |
| <a id="navigation"></a>**GPU navigation and resource surveys**<br>[navigation.rs](../src/navigation.rs), [shaders/navigation.wgsl](../shaders/navigation.wgsl), [shaders/navigation_inspect.wgsl](../shaders/navigation_inspect.wgsl), [shaders/navigation_survey.wgsl](../shaders/navigation_survey.wgsl) | Bounded GPU path/survey operations and sparse inspection support travel and resource decisions; approximate navigation has explicit reachability/limit handling. | 2026-09-13 · `1ec1566` — Name expedition and founding policies and share GPU dispatch limits | 2026-09-11 · `57de102` — Standardize great-lake landmass terminology as inner continents<br>[gpu-navigation.md](../docs/gpu-navigation.md), [tests/navigation.rs](../tests/navigation.rs) |
| <a id="relief"></a>**Secular and religious relief journeys**<br>[relief.rs](../src/relief.rs), [religious_relief.rs](../src/religious_relief.rs) | Requests, witnessed need, funding, actual food/freight and delivered aid connect displaced families, institutions and inter-town relations. | 2026-09-13 · `27bddbd` — Add paid recovery of abandoned bulk stocks through finite land cargo | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[witnessed-relief.md](../docs/witnessed-relief.md), [religious-relief.md](../docs/religious-relief.md), [relief-freight-capacity.md](../docs/relief-freight-capacity.md), [tests/society.rs](../tests/society.rs), [tests/culture.rs](../tests/culture.rs) |
| <a id="recovery"></a>**Abandoned bulk-stock recovery**<br>[stock_recovery.rs](../src/stock_recovery.rs) | Paid recovery from retained canonical inventories uses buyer freight and fleets; depleted sources and lost cargo cannot be recovered twice. | 2026-09-13 · `faa1c5e` — Trace early food shortages and test materially bounded staged harbor openings | 2026-09-13 · `d4f8b75` — Recover abandoned coastal stocks with buyer-funded sea voyages<br>[abandoned-stock-recovery.md](../docs/abandoned-stock-recovery.md), [sea-stock-recovery.md](../docs/sea-stock-recovery.md), [tests/resources.rs](../tests/resources.rs) |

## Credit, obligations and monetary evidence

| Feature and source scope | Implemented behavior and boundary | Source work | Documentation/evidence and checks |
| --- | --- | --- | --- |
| <a id="credit"></a>**Loan contracts and account transfers**<br>[credit.rs](../src/credit.rs), [credit/state.rs](../src/credit/state.rs), [credit/accounts.rs](../src/credit/accounts.rs) | Currency-tagged principal, interest, maturity, status and ledger entries transfer existing cash between account adapters; loans do not mint money. Main runtime uses shared currency, not foreign exchange. | 2026-09-13 · `340edfe` — Check service loan receipts against attainable funded work | 2026-09-13 · `4d01c9d` — Record mixed outcomes from contract-aware workshop staffing<br>[credit-implementation.md](../docs/credit-implementation.md), [credit-precision-residuals.md](../docs/credit-precision-residuals.md) |
| <a id="underwriting"></a>**Lending requests and underwriting**<br>[credit/underwriting.rs](../src/credit/underwriting.rs), [credit/taxes.rs](../src/credit/taxes.rs), [credit/exports.rs](../src/credit/exports.rs) | Snapshot proposals use tax receipts and buyer-funded delivery proceeds as repayment evidence; offers remain separate from accepted transfers. | 2026-09-13 · `340edfe` — Check service loan receipts against attainable funded work | 2026-09-13 · `373b285` — Report credit requests, constraints and actual funding separately<br>[credit-request-evaluation.md](../docs/credit-request-evaluation.md), [credit-capacity-diagnostics.md](../docs/credit-capacity-diagnostics.md), [credit-tax-base-shock.md](../docs/credit-tax-base-shock.md) |
| <a id="councilcredit"></a>**Council tax bridges and institutional lenders**<br>[credit/councils.rs](../src/credit/councils.rs) | Optional council borrowing, administration/relief evidence, institutional lending and operating reserves; affordable requests may still receive no loan. | 2026-09-13 · `1d34c38` — test: trace automatic council credit through administrative payroll | 2026-09-13 · `1d34c38` — test: trace automatic council credit through administrative payroll<br>[council-credit-service-bridge.md](../docs/council-credit-service-bridge.md), [council-credit-reviews.md](../docs/council-credit-reviews.md), [institution-credit-reserves.md](../docs/institution-credit-reserves.md) |
| <a id="commercialcredit"></a>**Commercial and service-order credit**<br>[credit/commercial.rs](../src/credit/commercial.rs) | Working-cash requests for operators/exports and funded service receivables; allocation respects shared costs and actual lender funds. | 2026-09-13 · `340edfe` — Check service loan receipts against attainable funded work | 2026-09-13 · `02b74c3` — Procure future workshop services from bounded town surplus<br>[commercial-request-allocation.md](../docs/commercial-request-allocation.md), [commercial-credit-shared-costs.md](../docs/commercial-credit-shared-costs.md), [workshop-service-credit.md](../docs/workshop-service-credit.md) |
| <a id="debtservice"></a>**Repayment, arrears and restructuring**<br>[credit/servicing.rs](../src/credit/servicing.rs), [credit/restructuring.rs](../src/credit/restructuring.rs) | Monthly snapshot collection, scheduled obligations and one permitted consented extension preserve original terms and dated outcomes. | 2026-09-13 · `340edfe` — Check service loan receipts against attainable funded work | 2026-09-13 · `4d01c9d` — Record mixed outcomes from contract-aware workshop staffing<br>[credit-restructuring.md](../docs/credit-restructuring.md), [credit-implementation.md](../docs/credit-implementation.md) |
| <a id="debtrecovery"></a>**Default and late-proceeds recovery**<br>[credit/recovery.rs](../src/credit/recovery.rs), [credit/export_recovery.rs](../src/credit/export_recovery.rs) | Explicit post-default recovery and optional late export-cash settlements; a recovery does not reopen a written-off contract. | 2026-09-13 · `defde67` — Recover closed-estate defaults and verify the actual money ledger | 2026-09-13 · `defde67` — Recover closed-estate defaults and verify the actual money ledger<br>[credit-default-recovery.md](../docs/credit-default-recovery.md), [export-default-recovery.md](../docs/export-default-recovery.md), [export-recovery-experiment.md](../docs/export-recovery-experiment.md) |
| <a id="creditestates"></a>**Closed accounts and creditor succession**<br>[credit/estates.rs](../src/credit/estates.rs), [credit/ownership.rs](../src/credit/ownership.rs) | Retained institutional/operator estates settle debt and residual cash; dated creditor ownership transfers claims without duplicating cash or rewriting original loan terms. | 2026-09-13 · `4d1fe86` — Allow local descendants to inherit vacant household economic estates | 2026-09-13 · `defde67` — Recover closed-estate defaults and verify the actual money ledger<br>[institution-credit-estates.md](../docs/institution-credit-estates.md), [operator-credit-estates.md](../docs/operator-credit-estates.md), [credit-claim-succession.md](../docs/credit-claim-succession.md), [estate-default-recovery.md](../docs/estate-default-recovery.md) |
| <a id="issuance"></a>**Bounded shared-currency issuance**<br>[credit/issuance.rs](../src/credit/issuance.rs) | Dated, capped council issuance is a declared external money source; no civilization-specific currency exchange is implemented here. | 2026-09-13 · `0aa337c` — Add dated capped shared-currency issuance experiment | 2026-09-13 · `01d502b` — Distinguish untransferable credit residue from insolvency<br>[shared-issuance-smoke.md](../docs/shared-issuance-smoke.md), [shared-issuance-century.md](../docs/shared-issuance-century.md) |
| <a id="creditreport"></a>**Monetary reports, chronicle and explorer**<br>[credit/report.rs](../src/credit/report.rs), [credit/chronicle.rs](../src/credit/chronicle.rs), [viewer_credit.rs](../src/viewer_credit.rs) | Read-only account concentrations, debt/claim views, actual monetary milestones and UI inspection distinguish money stocks from repeated flows. | 2026-09-13 · `a560627` — Support settlement-only household creditor receipts | 2026-09-13 · `defde67` — Recover closed-estate defaults and verify the actual money ledger<br>[credit-chronicle.md](../docs/credit-chronicle.md), [credit-explorer.md](../docs/credit-explorer.md), [monetary-residual-audit.md](../docs/monetary-residual-audit.md) |

## Politics, institutions, conflict and culture

| Feature and source scope | Implemented behavior and boundary | Source work | Documentation/evidence and checks |
| --- | --- | --- | --- |
| <a id="politics"></a>**Genealogy, factions and political interests**<br>[politics.rs](../src/politics.rs), [faction_interests.rs](../src/faction_interests.rs) | Kinship, faction membership, plural interests, mobilization, material pressures, heritage and competition influence political support and conflict. | 2026-09-13 · `622548c` — Name shipping, household and political policy parameters | 2026-09-12 · `af93467` — Separate political succession from estates and add faction heritage appeal<br>[politics.md](../docs/politics.md), [faction-interests.md](../docs/faction-interests.md), [tests/politics.rs](../tests/politics.rs) |
| <a id="leadership"></a>**Constitutional succession and representation**<br>[leadership.rs](../src/leadership.rs) | Hereditary/faction/council rules, internal replacement, personal accountability and renown are separate from property inheritance; configurable property/household/resident weights expose their calculation. | 2026-09-12 · `d475f2d` — Add funded institutional recovery and political representation policies | 2026-09-12 · `d475f2d` — Add funded institutional recovery and political representation policies<br>[leadership-continuity.md](../docs/leadership-continuity.md), [leadership-recovery-and-representation.md](../docs/leadership-recovery-and-representation.md), [tests/politics.rs](../tests/politics.rs) |
| <a id="governance"></a>**Government, duties and diplomacy**<br>[governance.rs](../src/governance.rs) | Councils, taxes, public funding, administrative work, trust, autonomy and non-aggression link legitimacy to delivered services and finite budgets. | 2026-09-13 · `622548c` — Name shipping, household and political policy parameters | 2026-09-13 · `27bddbd` — Add paid recovery of abandoned bulk stocks through finite land cargo<br>[governance.md](../docs/governance.md), [governance-duty-audit.md](../docs/governance-duty-audit.md), [governance-pressure.md](../docs/governance-pressure.md), [tests/governance.rs](../tests/governance.rs) |
| <a id="offices"></a>**Local offices and named attendance**<br>[offices.rs](../src/offices.rs), [offices/service.rs](../src/offices/service.rs) | Jurisdictions, holders, succession and optional personal duty reservations feed local service capacity; funding remains owned by governance. | 2026-09-12 · `822b0bb` — Share demographic rates and name resident and office policies | 2026-09-13 · `27bddbd` — Add paid recovery of abandoned bulk stocks through finite land cargo<br>[local-offices.md](../docs/local-offices.md), [shared-council-payroll.md](../docs/shared-council-payroll.md), [tests/governance.rs](../tests/governance.rs) |
| <a id="petitions"></a>**Civic petitions and causal hearings**<br>[civic_petitions.rs](../src/civic_petitions.rs), [civic_petitions/causal_tests.rs](../src/civic_petitions/causal_tests.rs) | Named parties, feasible hearing work and funded responses connect civic claims to actual production/service outcomes; causal fixtures are declared setups. | 2026-09-13 · `5b0b2c8` — Name research, relief, social pressure and explorer parameters | 2026-09-13 · `27bddbd` — Add paid recovery of abandoned bulk stocks through finite land cargo<br>[civic-petitions.md](../docs/civic-petitions.md), [civic-petition-causality.md](../docs/civic-petition-causality.md), [tests/governance.rs](../tests/governance.rs) |
| <a id="military"></a>**Armies, campaigns and finite occupation**<br>[military.rs](../src/military.rs), [military_supply.rs](../src/military_supply.rs), [occupation.rs](../src/occupation.rs) | Recruitment, shared people, travel, food/equipment, losses, return and occupied-site supply; projected and realized military losses are compared without a duplicate casualty ledger. | 2026-09-12 · `6fa6e77` — Extract road contact and military supply constants | 2026-09-12 · `c34218f` — Add constructed defenses and supply-limited sieges with shared military freight<br>[individual-military-verification.md](../docs/individual-military-verification.md), [finite-occupation.md](../docs/finite-occupation.md), [military-supply-comparison.md](../docs/military-supply-comparison.md), [tests/politics.rs](../tests/politics.rs) |
| <a id="peace"></a>**Negotiated peace and payment obligations**<br>[peace.rs](../src/peace.rs) | Bilateral offers, acceptance, withdrawal and dated finite council payments record dues, transfers, shortfalls and breach consequences. | 2026-09-12 · `b6e35c1` — Name continuity and institutional tuning constants in owning modules | 2026-09-12 · `0dd8a98` — Add bilateral peace with finite installments and enforceable breach<br>[negotiated-peace.md](../docs/negotiated-peace.md), [tests/politics.rs](../tests/politics.rs) |
| <a id="siege"></a>**Defensive assets, land sieges and resupply**<br>[siege.rs](../src/siege.rs) | Persistent defenses, work/material construction, besieging armies, constrained cargo access and reserved food supply; bounded land-war extension rather than tactical combat. | 2026-09-13 · `27bddbd` — Add paid recovery of abandoned bulk stocks through finite land cargo | 2026-09-12 · `4ca6126` — Apply siege access restrictions to captured freight itineraries<br>[supply-limited-sieges.md](../docs/supply-limited-sieges.md), [tests/politics.rs](../tests/politics.rs) |
| <a id="patrons"></a>**Patron arrivals, witnesses and service**<br>[culture.rs](../src/culture.rs), [assets/patrons.toml](../assets/patrons.toml) | Named guides, arrival accounts, limited aid, scheduled return to the Ancient World, inherited traditions and keepsakes; intended service and actually delivered assistance remain distinct. | 2026-09-14 · `238c31a` — Give new foundings four years of food and storage with hunger regression | 2026-09-14 · `1f249ca` — Revise patron history details and task completion<br>[patron-foundings.md](../docs/patron-foundings.md), [patron-results.md](../docs/patron-results.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="religion"></a>**Religious affiliation, interpretation and practices**<br>[culture/dynamics.rs](../src/culture/dynamics.rs), [culture/practices.rs](../src/culture/practices.rs) | Contact, leadership, household affiliation, dissent, schisms, syncretism, offerings, charity, pilgrimage and attributed accounts respond to social conditions and material limits. | 2026-09-13 · `8db559f` — Name cultural action and market policies with shared accounting constants | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[religious-dynamics.md](../docs/religious-dynamics.md), [religious-pluralism.md](../docs/religious-pluralism.md), [household-cultural-affiliation.md](../docs/household-cultural-affiliation.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="agents"></a>**Cultural people, goals and unique objects**<br>[culture.rs](../src/culture.rs), [culture/practices.rs](../src/culture/practices.rs) | Named traits/skills, relationships, decisions, manuscripts, creation, dedication, sale, theft, inheritance, loss/recovery and destruction; physical custody differs from ownership claims. | 2026-09-14 · `238c31a` — Give new foundings four years of food and storage with hunger regression | 2026-09-14 · `1f249ca` — Revise patron history details and task completion<br>[patron-foundings.md](../docs/patron-foundings.md), [knowledge-continuity.md](../docs/knowledge-continuity.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="artifactclaims"></a>**Artifact ownership petitions**<br>[artifact_petitions.rs](../src/artifact_petitions.rs) | Bounded local hearings can return custody/title, fund compensation or leave a claim unresolved; use existing completed office capacity. | 2026-09-12 · `b6e35c1` — Name continuity and institutional tuning constants in owning modules | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[artifact-petitions.md](../docs/artifact-petitions.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="institutions"></a>**Institution capacity, facilities and service funding**<br>[institution_capacity.rs](../src/institution_capacity.rs), [institution_funding.rs](../src/institution_funding.rs), [institution_services.rs](../src/institution_services.rs) | Readiness, rooms, embodied facilities, upkeep, repair, administration and operating funding constrain religious, merchant, craft and scholarly services. | 2026-09-14 · `7289927` — Add funded ruin resettlement and dormant ownership expiry | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[institution-capacity.md](../docs/institution-capacity.md), [institution-operating-budgets.md](../docs/institution-operating-budgets.md), [institution-working-core.md](../docs/institution-working-core.md), [institution-service-space-review.md](../docs/institution-service-space-review.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="instleadership"></a>**Institution succession and leadership recovery**<br>[institution_succession.rs](../src/institution_succession.rs) | Mandates, eligible local membership, elections and funded recruitment after vacancy use real work/funds; recovery does not teleport absent members. | 2026-09-12 · `eef7fcb` — Extract succession policy and crew resolution constants | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[institutional-succession.md](../docs/institutional-succession.md), [leadership-recovery-and-representation.md](../docs/leadership-recovery-and-representation.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="instmove"></a>**Institution relocation**<br>[institution_relocation.rs](../src/institution_relocation.rs) | Funded direct-road moves retain identity/treasury and carry accessible portable property; buildings stay behind, service pauses, arrival rechecks access. Branch services and automatic relocation remain unimplemented. | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[institution-relocation.md](../docs/institution-relocation.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="service"></a>**Research/culture allocation and execution plans**<br>[service_allocation.rs](../src/service_allocation.rs), [culture/work_requests.rs](../src/culture/work_requests.rs) | Research-first or weighted shared allowances, minimum useful grants, room/member eligibility and dated plans; scoped policies do not allocate all town labor fairly. | 2026-09-13 · `d48267d` — Add progressive household cash tax and funded practical research pilots | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[service-allocation.md](../docs/service-allocation.md), [research-cultural-work-requests.md](../docs/research-cultural-work-requests.md), [institution-allocation-balance.md](../docs/institution-allocation-balance.md), [tests/culture.rs](../tests/culture.rs), [tests/discoveries.rs](../tests/discoveries.rs) |
| <a id="learning"></a>**Knowledge transmission and learning resolution**<br>[culture/learning.rs](../src/culture/learning.rs), [learning_resolution.rs](../src/learning_resolution.rs) | Teaching, contact, manuscripts, successor training and continuing study retain acquisition provenance; projections and actual learning are compared with eligibility/work constraints. | 2026-09-12 · `fc6d9a4` — Name hazard, navigation and relocation policy constants | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[knowledge-succession.md](../docs/knowledge-succession.md), [informal-learning-selection.md](../docs/informal-learning-selection.md), [institution-continuing-study.md](../docs/institution-continuing-study.md), [learning-resolution.md](../docs/learning-resolution.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="research"></a>**Practical production knowledge research**<br>[culture/practical_research.rs](../src/culture/practical_research.rs) | Optional local study develops usable production topics through bounded work and knowledge prerequisites; separate from expedition specimen research. | 2026-09-13 · `d48267d` — Add progressive household cash tax and funded practical research pilots | 2026-09-13 · `d48267d` — Add progressive household cash tax and funded practical research pilots<br>[circulation-and-workshop-recovery.md](../docs/circulation-and-workshop-recovery.md), [tests/culture.rs](../tests/culture.rs) |

## Expeditions, historical evidence and naming

| Feature and source scope | Implemented behavior and boundary | Source work | Documentation/evidence and checks |
| --- | --- | --- | --- |
| <a id="expeditions"></a>**Expedition objectives, crews and frontier travel**<br>[expeditions.rs](../src/expeditions.rs) | Funding, objectives, named specialties, frontier routes, danger, rescues, recall and return connect expeditions to labor, supplies and home institutions; no permanent human colonization of the Ancient World. | 2026-09-13 · `1ec1566` — Name expedition and founding policies and share GPU dispatch limits | 2026-09-12 · `48fd5ff` — Give patron searches archetype-specific archaeological finds<br>[expeditions.md](../docs/expeditions.md), [expedition-crews.md](../docs/expedition-crews.md), [expedition-experience-and-renown.md](../docs/expedition-experience-and-renown.md), [tests/expeditions.rs](../tests/expeditions.rs) |
| <a id="discoveries"></a>**Finite collections, research and applications**<br>[discoveries.rs](../src/discoveries.rs), [discoveries/returns.rs](../src/discoveries/returns.rs) | Specimens and typed botanical returns use finite source/cargo accounting; research and processing unlock bounded applications including stored remedies, with explicit use policies. | 2026-09-13 · `5a56c15` — Name viewer controls and share map dispatch and export limits | 2026-09-12 · `f5823e9` — Keep stored remedies available when specimen research is paused<br>[discoveries.md](../docs/discoveries.md), [expedition-returns.md](../docs/expedition-returns.md), [tests/discoveries.rs](../tests/discoveries.rs) |
| <a id="heritage"></a>**Archaeology, patron finds and heritage**<br>[expedition_heritage.rs](../src/expedition_heritage.rs), [expedition_heritage/patron_finds.rs](../src/expedition_heritage/patron_finds.rs) | Religious/patron search, inscriptions, old literature and varied archetype-associated finds become retained objects/evidence; interpretations do not prove patron presence. | 2026-09-13 · `8db559f` — Name cultural action and market policies with shared accounting constants | 2026-09-12 · `48fd5ff` — Give patron searches archetype-specific archaeological finds<br>[heritage-expeditions.md](../docs/heritage-expeditions.md), [tests/expeditions.rs](../tests/expeditions.rs) |
| <a id="renown"></a>**Expedition renown and heritage stewardship**<br>[heritage_renown.rs](../src/heritage_renown.rs) | Successful expeditions/rescues and weighted heritage outcomes affect people, institutions, traditions and politics; finite access, hosting, study funding and decay limit continuing benefits. | 2026-09-12 · `1d9fa25` — Broaden expedition renown and turn worker experience into bounded productivity | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[heritage-renown.md](../docs/heritage-renown.md), [heritage-stewardship.md](../docs/heritage-stewardship.md), [heritage-study-funding.md](../docs/heritage-study-funding.md), [expedition-experience-and-renown.md](../docs/expedition-experience-and-renown.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="places"></a>**Persistent historical places and material evidence**<br>[local_places.rs](../src/local_places.rs) | Ruins, structures, objects, retained ownership and source evidence expose past actions in regional context rather than creating duplicate loot inventories. | 2026-09-12 · `b9a41b2` — Name vocabulary, social memory and workshop resolution policies | 2026-09-12 · `dd2350b` — Preserve canonical source depletion as historical evidence<br>[regional-historical-places.md](../docs/regional-historical-places.md), [source-depletion-evidence.md](../docs/source-depletion-evidence.md), [tests/resources.rs](../tests/resources.rs) |
| <a id="naming"></a>**Semantic names and civilization languages**<br>[naming.rs](../src/naming.rs), [naming/evolution.rs](../src/naming/evolution.rs) | Root vocabularies, sound changes, weighted constructions and local/religious/historical references name people, patrons, places, objects and wars; lexical alternatives and delivered-trade borrowing affect future names, not retroactive renaming. | 2026-09-13 · `27bddbd` — Add paid recovery of abandoned bulk stocks through finite land cargo | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[naming-languages.md](../docs/naming-languages.md), [lexicon-evolution.md](../docs/lexicon-evolution.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="contact"></a>**Recent trade and cultural contact**<br>[trade_contact.rs](../src/trade_contact.rs) | Completed trade provides dated contact evidence for social/cultural decisions instead of assuming route existence means current exchange. | 2026-09-13 · `f8c83b0` — Measure food import constraints without changing market outcomes | 2026-09-13 · `547aaba` — Settle institutional estates while preserving traveling accounts<br>[recent-trade-contact.md](../docs/recent-trade-contact.md), [transport-and-cultural-contact.md](../docs/transport-and-cultural-contact.md), [tests/culture.rs](../tests/culture.rs) |
| <a id="spatial"></a>**Spatial features and territorial history**<br>[spatial.rs](../src/spatial.rs), [territory.rs](../src/territory.rs) | Shared points, paths, cell regions and geometry export attach entities/events/expeditions to maps; dated territorial snapshots support history overlays. Precision metadata distinguishes schematic from terrain-resolved geometry. | 2026-09-12 · `b9a41b2` — Name vocabulary, social memory and workshop resolution policies | 2026-09-10 · `aec193f` — Add dated territory and household journey atlas overlays<br>[spatial-features.md](../docs/spatial-features.md), [tests/history_timeline.rs](../tests/history_timeline.rs) |

## Application, catalogs and evidence

| Feature and source scope | Implemented behavior and boundary | Source work | Documentation/evidence and checks |
| --- | --- | --- | --- |
| <a id="viewer"></a>**Globe, atlas and history explorer**<br>[viewer.rs](../src/viewer.rs), [history_atlas.rs](../src/history_atlas.rs), [history_timeline.rs](../src/history_timeline.rs), [shaders/view.wgsl](../shaders/view.wgsl) | Map layers, selection/inspection, history timelines, causal navigation and linked entity pages expose current and retained history. | 2026-09-14 · `118bc9c` — Enable ruin resettlement by default with explicit opt-out | 2026-09-10 · `2760bf1` — Add recorded settlement timeline with causal event navigation<br>[history-timeline.md](../docs/history-timeline.md), [tests/history_timeline.rs](../tests/history_timeline.rs) |
| <a id="configuration"></a>**Configuration, registry and library entry points**<br>[config.rs](../src/config.rs), [systems.rs](../src/systems.rs), [systems/policies.rs](../src/systems/policies.rs), [main.rs](../src/main.rs), [lib.rs](../src/lib.rs), [assets/example.toml](../assets/example.toml), [assets/scenario-example.json](../assets/scenario-example.json) | CLI/application setup, dependency-aware startup switches and bounded live policy changes; the registry appendix records actual defaults. Library baseline constructors and old archives may differ. | 2026-09-14 · `118bc9c` — Enable ruin resettlement by default with explicit opt-out | 2026-09-14 · `118bc9c` — Enable ruin resettlement by default with explicit opt-out<br>[system-options.md](../docs/system-options.md), [constants-cleanup.md](../docs/constants-cleanup.md), [tests/core.rs](../tests/core.rs) |
| <a id="catalogs"></a>**Validated editable catalogs**<br>[catalog.rs](../src/catalog.rs), [assets/catalog.toml](../assets/catalog.toml), [assets/economy.toml](../assets/economy.toml), [assets/agriculture.toml](../assets/agriculture.toml), [assets/agriculture-seasonal.toml](../assets/agriculture-seasonal.toml), [assets/materials.toml](../assets/materials.toml) | Rocks/minerals/soils/biomes/producers/microbes/guilds, goods/recipes, crops/animals and materials supply stable IDs, traits and bounded GPU tables; seasonal agriculture has a separate catalog. | 2026-09-13 · `3bd7c8f` — Add bounded investment in useful food shipping connections | 2026-09-12 · `f68a1ae` — Add opt-in regional wildlife thermal preferences and migration inheritance<br>[ecology.md](../docs/ecology.md), [economy.md](../docs/economy.md), [crop-price-revisit.md](../docs/crop-price-revisit.md), [material-objects-and-facilities.md](../docs/material-objects-and-facilities.md), [tests/core.rs](../tests/core.rs) |
| <a id="archives"></a>**World archives and consistent checkpoints**<br>[storage.rs](../src/storage.rs) | Versioned world state, catalogs, GPU buffers and sparse history persist at consistent boundaries with legacy migration and validation; generated archives remain untracked. | 2026-09-14 · `780eabf` — Register recent policy experiments with shared CLI and startup controls | 2026-09-11 · `384607d` — Staff merchant vessels with bounded named participation and household wages<br>[living-history.md](../docs/living-history.md), [resolution-framework.md](../docs/resolution-framework.md), [tests/core.rs](../tests/core.rs), [tests/living.rs](../tests/living.rs) |
| <a id="diagnostics"></a>**Demographic windows and food-flow diagnostics**<br>[demographic_audit.rs](../src/demographic_audit.rs), [demographic_audit/food.rs](../src/demographic_audit/food.rs) | Optional demographic exposure and monthly food/nutrient observations distinguish production, access and timing; net unclassified food flow is not a conservation proof. | 2026-09-14 · `9370d38` — Diagnose growth reversals with monthly food and crop boundary probes | 2026-09-14 · `1c0bfb7` — Add farm phosphorus retention controls and century comparisons<br>[demographic-window-audit.md](../docs/demographic-window-audit.md), [growth-food-diagnostics.md](../docs/growth-food-diagnostics.md), [monthly-food-balance.md](../docs/monthly-food-balance.md), [tests/food_diagnostics.rs](../tests/food_diagnostics.rs) |
| <a id="growthtools"></a>**Growth stage gates and food/nutrient investigation**<br>[examples/growth_ladder.rs](../examples/growth_ladder.rs), [examples/inner_land_count.rs](../examples/inner_land_count.rs), [scripts/run_growth_investigation.py](../scripts/run_growth_investigation.py), [scripts/report_growth_bottlenecks.py](../scripts/report_growth_bottlenecks.py), [scripts/report_early_food.py](../scripts/report_early_food.py), [scripts/report_founding_farms.py](../scripts/report_founding_farms.py), [scripts/report_monthly_food.py](../scripts/report_monthly_food.py), [scripts/report_demographic_windows.py](../scripts/report_demographic_windows.py), [scripts/summarize_growth_food.py](../scripts/summarize_growth_food.py), [scripts/summarize_growth_funding.py](../scripts/summarize_growth_funding.py) | Checkpointed growth ladders, decline gates, land capacity, food supply/access and local bottleneck reports; optional long-run experiments, not ordinary test-suite assertions. | 2026-09-14 · `1c0bfb7` — Add farm phosphorus retention controls and century comparisons | 2026-09-17 · `5bfaf42` — Document matched century nutrient tests without settlement expansion<br>[settlement-growth-experiment.md](../docs/settlement-growth-experiment.md), [growth-local-bottlenecks.md](../docs/growth-local-bottlenecks.md), [growth-food-diagnostics.md](../docs/growth-food-diagnostics.md), [fixed-footprint-nutrients.md](../docs/fixed-footprint-nutrients.md) |
| <a id="financialtools"></a>**Monetary and distribution experiments**<br>[scripts/monetary_experiment.py](../scripts/monetary_experiment.py), [scripts/audit_circulation.py](../scripts/audit_circulation.py), [scripts/compare_credit_chronicle.py](../scripts/compare_credit_chronicle.py), [scripts/compare_food_access.py](../scripts/compare_food_access.py), [scripts/compare_council_funding.py](../scripts/compare_council_funding.py), [scripts/summarize_food_access.py](../scripts/summarize_food_access.py), [scripts/summarize_nutrition.py](../scripts/summarize_nutrition.py), [scripts/economy_report.py](../scripts/economy_report.py), [examples/nutrition_evaluate.rs](../examples/nutrition_evaluate.rs) | Matched credit/issuance, cash circulation, council funding and household access comparisons retain controls, residues and limits. | 2026-09-13 · `ca103a1` — Test local surplus-wallet food support without removing retail revenue | 2026-09-13 · `defde67` — Recover closed-estate defaults and verify the actual money ledger<br>[monetary-residual-audit.md](../docs/monetary-residual-audit.md), [credit-chronicle-regression.md](../docs/credit-chronicle-regression.md), [council-funding-balance.md](../docs/council-funding-balance.md), [household-nutrition-calibration.md](../docs/household-nutrition-calibration.md) |
| <a id="integrationtools"></a>**Integrated history and policy evidence**<br>[scripts/integrated_history.py](../scripts/integrated_history.py), [scripts/compare_integrated_history.py](../scripts/compare_integrated_history.py), [scripts/analyze_integrated_history.py](../scripts/analyze_integrated_history.py), [scripts/run_integration_balance.py](../scripts/run_integration_balance.py), [scripts/evidence.py](../scripts/evidence.py), [scripts/build_history_evaluator.py](../scripts/build_history_evaluator.py), [scripts/policy_suite.py](../scripts/policy_suite.py), [scripts/pressure_sweep.py](../scripts/pressure_sweep.py), [scripts/settlement_audit.py](../scripts/settlement_audit.py), [scripts/analyze_civic_balance.py](../scripts/analyze_civic_balance.py), [scripts/analyze_flood_audit.py](../scripts/analyze_flood_audit.py), [scripts/analyze_ideas_audit.py](../scripts/analyze_ideas_audit.py), [examples/history_evaluate.rs](../examples/history_evaluate.rs), [examples/history_replay.rs](../examples/history_replay.rs), [examples/civic_balance.rs](../examples/civic_balance.rs) | Seed/policy/stress suites, replay and evidence validation connect mechanisms to measured outcomes; recorded fixtures and balance screens are not scientific calibration. | 2026-09-11 · `8fc146d` — Connect politics and heritage to recent trade and launch broader balance audit | 2026-09-11 · `7bde9f2` — Expose shared GPU labor forecasts and record scarcity ensemble<br>[model-evidence.md](../docs/model-evidence.md), [integrated-calibration.md](../docs/integrated-calibration.md), [integration-balance-followup.md](../docs/integration-balance-followup.md), [pressure-sweep.md](../docs/pressure-sweep.md) |
| <a id="productiontools"></a>**Production and service evaluation**<br>[scripts/evaluate_enterprises.py](../scripts/evaluate_enterprises.py), [scripts/publish_enterprise_evidence.py](../scripts/publish_enterprise_evidence.py), [scripts/summarize_cultural_work.py](../scripts/summarize_cultural_work.py), [scripts/summarize_harbor_work.py](../scripts/summarize_harbor_work.py), [scripts/summarize_institution_succession.py](../scripts/summarize_institution_succession.py), [scripts/summarize_road_upkeep.py](../scripts/summarize_road_upkeep.py), [scripts/summarize_toolmaking.py](../scripts/summarize_toolmaking.py), [scripts/waterworks_report.py](../scripts/waterworks_report.py), [examples/cultural_work_calibrate.rs](../examples/cultural_work_calibrate.rs), [examples/toolmaking_evidence.rs](../examples/toolmaking_evidence.rs), [examples/alloy_evaluation.rs](../examples/alloy_evaluation.rs) | Workshop, tool/alloy, harbor/road, waterworks and institutional work audits distinguish requests, funding, completion and outcomes. | 2026-09-12 · `d98b6b5` — Distinguish active council arrears from retained historical counters | 2026-09-11 · `9fbffb8` — Test joint institutional funding and space limits<br>[workshop-operator-calibration.md](../docs/workshop-operator-calibration.md), [toolmaking-capability.md](../docs/toolmaking-capability.md), [institution-allocation-balance.md](../docs/institution-allocation-balance.md) |
| <a id="ecologicaltools"></a>**Ecology and continuity controls**<br>[examples/calibrate.rs](../examples/calibrate.rs), [examples/foodweb_evaluate.rs](../examples/foodweb_evaluate.rs), [examples/wildlife_evaluate.rs](../examples/wildlife_evaluate.rs), [examples/living_fishery_evaluate.rs](../examples/living_fishery_evaluate.rs), [examples/coupling_evidence.rs](../examples/coupling_evidence.rs), [examples/resource_counterfactual.rs](../examples/resource_counterfactual.rs), [examples/region.rs](../examples/region.rs), [scripts/wildlife_report.py](../scripts/wildlife_report.py), [scripts/analyze_living_fisheries.py](../scripts/analyze_living_fisheries.py), [scripts/summarize_coupling.py](../scripts/summarize_coupling.py), [scripts/check_continuity.py](../scripts/check_continuity.py) | Ecology/wildlife/fishery sweeps, cross-scale counterfactuals and regional examples examine finite resource coupling and sensitivity. | 2026-09-12 · `c34218f` — Add constructed defenses and supply-limited sieges with shared military freight | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification<br>[wildlife-calibration.md](../docs/wildlife-calibration.md), [resource-counterfactual-results.md](../docs/resource-counterfactual-results.md), [cross-scale-coupling.md](../docs/cross-scale-coupling.md) |
| <a id="patrontools"></a>**Patron and expedition evaluation**<br>[scripts/patron_audit.py](../scripts/patron_audit.py), [scripts/patron_report.py](../scripts/patron_report.py), [scripts/summarize_expedition_crews.py](../scripts/summarize_expedition_crews.py) | Arrival/aid and crew outcome reports examine patron service and expedition participation. | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification | 2026-09-12 · `78f8c7e` — Keep expedition skill transfer specific to civilian specialties<br>[patron-results.md](../docs/patron-results.md), [expedition-crews.md](../docs/expedition-crews.md) |
| <a id="performance"></a>**Performance and repository checks**<br>[examples/history_profile.rs](../examples/history_profile.rs), [examples/lake_relaxation.rs](../examples/lake_relaxation.rs), [scripts/check_repository_artifacts.py](../scripts/check_repository_artifacts.py) | History profiling, lake convergence/polling benchmarks and source-only artifact enforcement; generated measurements belong under ignored output. | 2026-09-12 · `3d244c8` — Add measured aquatic thermal and lake polling sensitivity controls | 2026-09-12 · `3d244c8` — Add measured aquatic thermal and lake polling sensitivity controls<br>[history-performance-profile.md](../docs/history-performance-profile.md), [default-generation-performance.md](../docs/default-generation-performance.md), [lake-polling-review.md](../docs/lake-polling-review.md) |
| <a id="ci"></a>**Evidence CI workflow**<br>[.github/workflows/evidence.yml](../.github/workflows/evidence.yml) | CPU evidence runs on push/pull request; full GPU evidence requires manual dispatch to a trusted self-hosted Vulkan runner. Workflow artifacts are uploaded separately from source commits; configuration is not proof of a successful CI run. | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification | 2026-09-11 · `57de102` — Standardize great-lake landmass terminology as inner continents<br>[model-evidence.md](../docs/model-evidence.md), [evidence/README.md](../docs/evidence/README.md) |

## Standalone economics experiment — not integrated into the world

| Feature and source scope | Implemented behavior and boundary | Source work | Documentation/evidence and checks |
| --- | --- | --- | --- |
| <a id="xcore"></a>**Process model, settlement and CPU compute**<br>[exp/economics/src/model.rs](../exp/economics/src/model.rs), [exp/economics/src/settlement.rs](../exp/economics/src/settlement.rs), [exp/economics/src/compute.rs](../exp/economics/src/compute.rs), [exp/economics/src/simulation.rs](../exp/economics/src/simulation.rs), [exp/economics/src/lib.rs](../exp/economics/src/lib.rs), [exp/economics/src/main.rs](../exp/economics/src/main.rs) | Independent crate with stable IDs, dated staged processes, validation/staging and segmented balance settlement; CubeCL CPU and reference execution are experimental, not a replacement for the world economy. | 2026-09-21 · `173f05a` — economics: unify agreements and clear competing offers atomically | 2026-09-21 · `9d8d870` — docs: record agreement design and simultaneous clearing results<br>[exp/economics/README.md](../exp/economics/README.md), [exp/economics/EXPERIMENTS.md](../exp/economics/EXPERIMENTS.md), [exp/economics/tests/economics.rs](../exp/economics/tests/economics.rs) |
| <a id="xsearch"></a>**Planning, search and opportunity discovery**<br>[exp/economics/src/planning.rs](../exp/economics/src/planning.rs), [exp/economics/src/search.rs](../exp/economics/src/search.rs), [exp/economics/src/opportunities.rs](../exp/economics/src/opportunities.rs) | Bounded candidate strategies share forecast evaluation and ordinary settlement; discovery permissions do not replace stock, time or access feasibility. | 2026-09-21 · `173f05a` — economics: unify agreements and clear competing offers atomically | 2026-09-21 · `aa3f6b9` — economics: add citizenship-gated opportunities and swappable search<br>[exp/economics/SEARCH.md](../exp/economics/SEARCH.md), [exp/economics/OPPORTUNITIES.md](../exp/economics/OPPORTUNITIES.md), [exp/economics/DATED-CANDIDATES.md](../exp/economics/DATED-CANDIDATES.md), [exp/economics/tests/planning.rs](../exp/economics/tests/planning.rs), [exp/economics/tests/search.rs](../exp/economics/tests/search.rs), [exp/economics/tests/opportunities.rs](../exp/economics/tests/opportunities.rs) |
| <a id="xrights"></a>**Access, membership and production plots**<br>[exp/economics/src/commitments.rs](../exp/economics/src/commitments.rs), [exp/economics/src/membership.rs](../exp/economics/src/membership.rs), [exp/economics/src/plots.rs](../exp/economics/src/plots.rs) | Accepted access obligations, scoped citizenship permissions and additional taxed plot requests govern dated use rights. Only committed mechanisms are inventoried here. | 2026-09-21 · `173f05a` — economics: unify agreements and clear competing offers atomically | 2026-09-21 · `aa3f6b9` — economics: add citizenship-gated opportunities and swappable search<br>[exp/economics/CITIZENSHIP.md](../exp/economics/CITIZENSHIP.md), [exp/economics/ADDITIONAL-PLOTS.md](../exp/economics/ADDITIONAL-PLOTS.md), [exp/economics/tests/access.rs](../exp/economics/tests/access.rs), [exp/economics/tests/membership.rs](../exp/economics/tests/membership.rs), [exp/economics/tests/plots.rs](../exp/economics/tests/plots.rs) |
| <a id="xneeds"></a>**Needs, finite pools, storage and substitution**<br>[exp/economics/src/maintenance.rs](../exp/economics/src/maintenance.rs), [exp/economics/src/pools.rs](../exp/economics/src/pools.rs), [exp/economics/src/storage.rs](../exp/economics/src/storage.rs), [exp/economics/src/substitution.rs](../exp/economics/src/substitution.rs) | Nutrition/warmth and generic upkeep consequences affect capacity/lifecycle; finite regenerating pools, storage weights and whole-lot substitutes share inventory budgets. | 2026-09-20 · `a285350` — economics: cda experiment | 2026-09-21 · `173f05a` — economics: unify agreements and clear competing offers atomically<br>[exp/economics/FORAGING.md](../exp/economics/FORAGING.md), [exp/economics/STORAGE-CURRENCY.md](../exp/economics/STORAGE-CURRENCY.md), [exp/economics/tests/conditions.rs](../exp/economics/tests/conditions.rs), [exp/economics/tests/foraging.rs](../exp/economics/tests/foraging.rs), [exp/economics/tests/storage_currency.rs](../exp/economics/tests/storage_currency.rs) |
| <a id="xproduction"></a>**Durable equipment and specialized activities**<br>[exp/economics/src/equipment.rs](../exp/economics/src/equipment.rs), [exp/economics/src/activities.rs](../exp/economics/src/activities.rs), [exp/economics/src/crafts.rs](../exp/economics/src/crafts.rs) | Barter/techniques, equipment services, wear, repair and catalog-driven work targets support mining, refining, fishing, livestock and housing fixtures; no dedicated occupation controller is implied. | 2026-09-21 · `173f05a` — economics: unify agreements and clear competing offers atomically | 2026-09-21 · `aa3f6b9` — economics: add citizenship-gated opportunities and swappable search<br>[exp/economics/ACTIVITIES.md](../exp/economics/ACTIVITIES.md), [exp/economics/PRODUCTION-AUDIT.md](../exp/economics/PRODUCTION-AUDIT.md), [exp/economics/tests/equipment.rs](../exp/economics/tests/equipment.rs), [exp/economics/tests/activities.rs](../exp/economics/tests/activities.rs) |
| <a id="xtrade"></a>**Stock exchange, tokens and commodity forwards**<br>[exp/economics/src/currency.rs](../exp/economics/src/currency.rs), [exp/economics/src/exchange.rs](../exp/economics/src/exchange.rs), [exp/economics/src/forward.rs](../exp/economics/src/forward.rs), [exp/economics/src/finance.rs](../exp/economics/src/finance.rs) | Finite posted bids, collection-linked issuance, upfront tool purchases, prepaid production and shared financial claims/payment primitives; not general-purpose banking or a main-world currency migration. | 2026-09-21 · `17e7fab` — economics: share financial claims and payment settlement primitives | 2026-09-21 · `17e7fab` — economics: share financial claims and payment settlement primitives<br>[exp/economics/TRADING.md](../exp/economics/TRADING.md), [exp/economics/FORWARD-TRADING.md](../exp/economics/FORWARD-TRADING.md), [exp/economics/STATE-PRICING.md](../exp/economics/STATE-PRICING.md), [exp/economics/tests/finance.rs](../exp/economics/tests/finance.rs), [exp/economics/tests/exchange.rs](../exp/economics/tests/exchange.rs), [exp/economics/tests/forward.rs](../exp/economics/tests/forward.rs) |
| <a id="xhouseholds"></a>**Agreement-formed households**<br>[exp/economics/src/households.rs](../exp/economics/src/households.rs) | Collective accounts, pooled stores/income, shared shelter, member debt support and spare labor decisions use explicit transfer boundaries; population demographics are not implemented. | 2026-09-20 · `a285350` — economics: cda experiment | 2026-09-20 · `a285350` — economics: cda experiment<br>[exp/economics/HOUSEHOLDS.md](../exp/economics/HOUSEHOLDS.md), [exp/economics/tests/households.rs](../exp/economics/tests/households.rs) |
| <a id="xscenarios"></a>**Controlled standalone scenarios**<br>[exp/economics/src/scenario.rs](../exp/economics/src/scenario.rs), [exp/economics/src/trading_scenario.rs](../exp/economics/src/trading_scenario.rs) | One-, four- and many-person fixtures, provider-count comparisons and abstract-unit production/finance scenarios exercise the process model independently of terrain and history. | 2026-09-21 · `173f05a` — economics: unify agreements and clear competing offers atomically | 2026-09-20 · `a285350` — economics: cda experiment<br>[exp/economics/FOUR-PEOPLE.md](../exp/economics/FOUR-PEOPLE.md), [exp/economics/EXPERIMENTS.md](../exp/economics/EXPERIMENTS.md), [exp/economics/tests/multi_person.rs](../exp/economics/tests/multi_person.rs) |
| <a id="xagreements"></a>**Common accepted agreements and consequences**<br>[exp/economics/src/agreements.rs](../exp/economics/src/agreements.rs), [exp/economics/src/offers.rs](../exp/economics/src/offers.rs) | Derived membership/land/production terms, scoped breach consequences and common discover/prepare/accept bundles reuse authoritative receipts; acceptance reserves a dated production plan rather than issuing its outputs. Not a universal contract interpreter. | 2026-09-21 · `173f05a` — economics: unify agreements and clear competing offers atomically | 2026-09-21 · `9d8d870` — docs: record agreement design and simultaneous clearing results<br>[exp/economics/AGREEMENTS.md](../exp/economics/AGREEMENTS.md), [exp/economics/MARKET-AGREEMENTS.md](../exp/economics/MARKET-AGREEMENTS.md), [exp/economics/tests/agreements.rs](../exp/economics/tests/agreements.rs), [exp/economics/tests/process_offers.rs](../exp/economics/tests/process_offers.rs) |
| <a id="xallocation"></a>**Contested offers and simultaneous allocation**<br>[exp/economics/src/allocation.rs](../exp/economics/src/allocation.rs), [exp/economics/src/competition.rs](../exp/economics/src/competition.rs) | Stable-ID claims, priority/lottery policies, minimum grants and atomic reservations clear open plot applications; alternate-slot matching and joint-budget validation retain existing work and explicit fallback ordering. No two-sided auction or price clearing. | 2026-09-21 · `173f05a` — economics: unify agreements and clear competing offers atomically | 2026-09-21 · `9d8d870` — docs: record agreement design and simultaneous clearing results<br>[exp/economics/CONTESTED-OFFERS.md](../exp/economics/CONTESTED-OFFERS.md), [exp/economics/tests/allocation.rs](../exp/economics/tests/allocation.rs), [exp/economics/tests/competition.rs](../exp/economics/tests/competition.rs) |

## Registry coverage and defaults

Exact `System` variants from [systems.rs](../src/systems.rs). CLI spellings use kebab-case. **On** means the default request is enabled; disabled prerequisites suppress descendants. **Live policy** identifies `REGISTERED_POLICIES`, whose explicit changes use the completed-boundary API in [policies.rs](../src/systems/policies.rs); it does not mean arbitrary startup systems can be toggled safely mid-month.

All live policies default off except `RuinResettlement`; the other registry systems default on. `GradualNutrition` requires aggregate demography. `DemographicAudit` requires GPU aggregate demography without resolution overrides; food-flow observations have their own diagnostic path. Registry configuration is not a report of a saved world’s actual activation.

| Registry option | Default | Live policy | Inventory entry |
| --- | --- | --- | --- |
| `CouncilCredit` | Off | Yes | [Council tax bridges and institutional lenders](#councilcredit) |
| `InstitutionCreditLenders` | Off | Yes | [Council tax bridges and institutional lenders](#councilcredit) |
| `InstitutionCreditOperatingReserve` | Off | Yes | [Council tax bridges and institutional lenders](#councilcredit) |
| `CommercialCredit` | Off | Yes | [Commercial and service-order credit](#commercialcredit) |
| `ServiceOrderCredit` | Off | Yes | [Commercial and service-order credit](#commercialcredit) |
| `ServiceOrderProcurement` | Off | Yes | [Funded workshop service orders](#orders) |
| `ContractWorkshopStaffing` | Off | Yes | [Funded workshop service orders](#orders) |
| `DemandWorkshopStaffing` | Off | Yes | [Funded workshop service orders](#orders) |
| `HouseholdEstateInheritance` | Off | Yes | [Household inheritance and unclaimed estates](#estates) |
| `HouseholdClothing` | Off | Yes | [Household clothing and replacement demand](#clothing) |
| `HouseholdWealthTax` | Off | Yes | [Progressive household cash tax](#wealthtax) |
| `CouncilWelfareReserves` | Off | Yes | [Council and municipal relief policies](#welfare) |
| `NeedsBasedFood` | Off | Yes | [Household ownership, income and purchasing](#retail) |
| `GradualNutrition` | Off | Yes | [Household ownership, income and purchasing](#retail) |
| `FoodSolidarity` | Off | Yes | [Family assistance and local solidarity](#familycash) |
| `MunicipalFoodRelief` | Off | Yes | [Council and municipal relief policies](#welfare) |
| `MunicipalWelfareReserves` | Off | Yes | [Council and municipal relief policies](#welfare) |
| `InstitutionWorkingCore` | Off | Yes | [Institution capacity, facilities and service funding](#institutions) |
| `InstitutionOperatingFunding` | Off | Yes | [Institution capacity, facilities and service funding](#institutions) |
| `DemographicAudit` | Off | Yes | [Demographic windows and food-flow diagnostics](#diagnostics) |
| `StagedHarbors` | Off | Yes | [Ports, harbors and sea trade](#shipping) |
| `PracticalResearch` | Off | Yes | [Practical production knowledge research](#research) |
| `HouseholdEstateReclamation` | Off | Yes | [Household inheritance and unclaimed estates](#estates) |
| `AbandonedStockRecovery` | Off | Yes | [Abandoned bulk-stock recovery](#recovery) |
| `RuinResettlement` | On | Yes | [Ruin resettlement and expiring claims](#resettlement) |
| `NamedOfficeService` | Off | Yes | [Local offices and named attendance](#offices) |
| `ExportDefaultRecovery` | Off | Yes | [Default and late-proceeds recovery](#debtrecovery) |
| `SharedIssuance` | Off | Yes | [Bounded shared-currency issuance](#issuance) |
| `DeliveryPaidExports` | Off | Yes | [Export contracts and payment custody](#exports) |
| `OccupationalPayroll` | On | No | [Household ownership, income and purchasing](#retail) |
| `NegotiatedAutonomy` | On | No | [Government, duties and diplomacy](#governance) |
| `CivicPetitions` | On | No | [Civic petitions and causal hearings](#petitions) |
| `AutomaticExpeditions` | On | No | [Expedition objectives, crews and frontier travel](#expeditions) |
| `Society` | On | No | [Aggregate demography, households and seasons](#society) |
| `Politics` | On | No | [Genealogy, factions and political interests](#politics) |
| `Governance` | On | No | [Government, duties and diplomacy](#governance) |
| `Offices` | On | No | [Local offices and named attendance](#offices) |
| `Shipping` | On | No | [Ports, harbors and sea trade](#shipping) |
| `Expeditions` | On | No | [Expedition objectives, crews and frontier travel](#expeditions) |
| `Discoveries` | On | No | [Finite collections, research and applications](#discoveries) |
| `LivingWorld` | On | No | [Living history, surveys and monthly environment](#living) |
| `SharedResources` | On | No | [Canonical resource stocks and extraction](#resources) |
| `MineralProcessing` | On | No | [Mineral processing and alloys](#metals) |
| `AlloyProcessing` | On | No | [Mineral processing and alloys](#metals) |
| `EnvironmentalReturns` | On | No | [Environmental returns and land recovery](#returns) |
| `SocialIndicators` | On | No | [Social projections, pressures and local memory](#social) |
| `Enterprises` | On | No | [Enterprise ownership and workshop capital](#operators) |
| `DiversifiedFarming` | On | No | [Crop, livestock and fishery production](#agriculture) |
| `AdaptivePrices` | On | No | [Goods, demand, prices and network trade](#market) |
| `NetworkTrade` | On | No | [Goods, demand, prices and network trade](#market) |
| `Fisheries` | On | No | [Fishing effort, gear and local opportunity cost](#fisheries) |
| `AdaptiveFishing` | On | No | [Fishing effort, gear and local opportunity cost](#fisheries) |
| `PrimitiveFishingGear` | On | No | [Fishing effort, gear and local opportunity cost](#fisheries) |
| `FishingOpportunityCost` | On | No | [Fishing effort, gear and local opportunity cost](#fisheries) |
| `Production` | On | No | [Production planning and essential maintenance](#production) |
| `AdaptiveLabor` | On | No | [Production planning and essential maintenance](#production) |
| `FoodSecurityLabor` | On | No | [Production planning and essential maintenance](#production) |
| `FoodSecurityMaintenance` | On | No | [Production planning and essential maintenance](#production) |
| `ReplacementToolJobs` | On | No | [Production planning and essential maintenance](#production) |
| `ToolmakingExpertise` | On | No | [Production planning and essential maintenance](#production) |
| `Workshops` | On | No | [Production planning and essential maintenance](#production) |
| `PersistentStorage` | On | No | [Production planning and essential maintenance](#production) |
| `PersistentHousing` | On | No | [Production planning and essential maintenance](#production) |
| `Waterworks` | On | No | [Production planning and essential maintenance](#production) |
| `WaterworksRepairPriority` | On | No | [Production planning and essential maintenance](#production) |
| `SpecializedWorkshops` | On | No | [Production planning and essential maintenance](#production) |
| `ExportContracts` | On | No | [Export contracts and payment custody](#exports) |
| `SupplierProfitability` | On | No | [Export contracts and payment custody](#exports) |

## Other controls and resolution coverage

The registry is not the complete public API. These existing controls remain attached to their owning subsystem rather than being additional `System` variants. Their prerequisites and archive behavior are defined by the linked implementation; a callable API is not necessarily active in every world.

| Control family | Existing extension/interface | Owner |
| --- | --- | --- |
| Population authority | `set_named_demography`, `enable_individual_demography`, `set_demographic_resolution(mode, compare)`; roster conversion and comparison are separate from aggregate population. | [Residents](#residents), [resolution](#resolution) |
| Individual work | `set_individual_participation`, `set_workshop_refinement`, `set_agriculture_refinement`, `set_extraction_refinement`, `set_construction_refinement`, `set_domestic_households`. Effective work can differ from committed time. | [Participation](#participation), [farm/extraction/construction](#farmwork), [domestic](#domestic) |
| Scoped allocation | Research-first/weighted service sharing; stable/rotating institution member priority; essential-first duty policy; council administration protection and annual support targets. These apply to different pools. | [Service plans](#service), [public support](#welfare) |
| Environment experiments | `Config` ecology resolution/clocks, solar/mixing scales, island phosphorus, plot hectares/crop yield, wildlife barrier ablation and thermal ecotypes. `wildlife_open_barriers` and `wildlife_ecotypes` default false. Ecology scenarios can disable geochemical supply, suppress/restore guilds or change mixing. | [Ecology](#aquatic), [wildlife](#wildlife), [configuration](#configuration) |
| Solver/readback controls | Drainage/lake iteration ceilings, lake convergence polling, navigation mode and history readback mode support convergence and reference comparisons; they are not new physical systems. | [Planet](#planet), [navigation](#navigation), [living history](#living) |
| Founding and food | `FoundingOptions`, `set_founding_food_months`, founding-access policy, ration priority, per-site policies and growth interventions; crop catalog and nutrient-retention settings are separate from registry toggles. | [Patrons](#patrons), [founding](#founding), [expansion](#expansion), [nutrients](#nutrients) |
| Physical access | Road/sea-lane closure, mine closure, regional mining limits and tool-stock access; these intervene in existing inventories and feasibility. | [Resources](#resources), [navigation](#navigation), [shipping](#shipping), [metals](#metals) |
| Social policy and travel | Household relocation, witnessed/religious relief, contagious illness/introduction, succession rule, political weighting and occupation duration have dedicated APIs. | [Relocation](#relocation), [relief](#relief), [health](#health), [leadership](#leadership), [military](#military) |
| Sparse committed actions | Expedition launch/recall/rules, specimen workshop access, botanical use, institution move, artifact petition, peace offer/acceptance, defense construction and military supply. These requests still require resources and eligible actors. | [Expeditions](#expeditions), [discoveries](#discoveries), [institutions](#instmove), [petitions](#artifactclaims), [peace](#peace), [sieges](#siege) |

All **13** `resolution::System` receipt categories are implemented in the common comparison vocabulary. A receipt category alone does not mean the entire subsystem has individual authority.

| Receipt category | Owning implementation |
| --- | --- |
| `Demography` | [Resident identity and individual demography](#residents) |
| `Workshop` | [Workshop individual refinement](#workshops) |
| `Research` | [Knowledge transmission and learning resolution](#learning) |
| `Culture` | [Knowledge transmission and learning resolution](#learning) |
| `DomesticCare` | [Domestic groups, dependents and care](#domestic) |
| `MerchantCrew` | [Vessels and named crew service](#vessels) |
| `Agriculture` | [Farm, extraction and construction participation](#farmwork) |
| `Forestry` | [Farm, extraction and construction participation](#farmwork) |
| `Mining` | [Farm, extraction and construction participation](#farmwork) |
| `Construction` | [Farm, extraction and construction participation](#farmwork) |
| `OfficeService` | [Local offices and named attendance](#offices) |
| `RelocationTravel` | [Household journeys and travel comparison](#relocation) |
| `MilitarySupply` | [Armies, campaigns and finite occupation](#military) |

## File coverage appendix

Explicit coverage at the pinned baseline; a directory mention alone is not counted. Multiple links mean shared ownership, not duplicate runtime execution. Inline unit tests remain under their source module. Separate tests and audit programs are grouped below. Build manifests/toolchains/lockfiles are infrastructure, not extra simulated systems.

### Main Rust modules

| File | Owning feature/evidence family |
| --- | --- |
| [agriculture.rs](../src/agriculture.rs) | [Crop, livestock and fishery production](#agriculture), [Fishing effort, gear and local opportunity cost](#fisheries) |
| [agriculture_participation.rs](../src/agriculture_participation.rs) | [Farm, extraction and construction participation](#farmwork) |
| [artifact_petitions.rs](../src/artifact_petitions.rs) | [Artifact ownership petitions](#artifactclaims) |
| [catalog.rs](../src/catalog.rs) | [Validated editable catalogs](#catalogs) |
| [civic_petitions.rs](../src/civic_petitions.rs) | [Civic petitions and causal hearings](#petitions) |
| [civic_petitions/causal_tests.rs](../src/civic_petitions/causal_tests.rs) | [Civic petitions and causal hearings](#petitions) |
| [civilization.rs](../src/civilization.rs) | [Monthly schedule and history authority](#history) |
| [civilization/daughter.rs](../src/civilization/daughter.rs) | [Arrival provisions and daughter settlement supplies](#founding) |
| [civilization/founding_provisions.rs](../src/civilization/founding_provisions.rs) | [Arrival provisions and daughter settlement supplies](#founding) |
| [civilization/founding_seeds.rs](../src/civilization/founding_seeds.rs) | [Arrival provisions and daughter settlement supplies](#founding) |
| [civilization/growth.rs](../src/civilization/growth.rs) | [Growth controls and settlement admission](#expansion) |
| [civilization/production_forecast.rs](../src/civilization/production_forecast.rs) | [Farm, extraction and construction participation](#farmwork) |
| [config.rs](../src/config.rs) | [Configuration, registry and library entry points](#configuration) |
| [contagion.rs](../src/contagion.rs) | [Traveler-linked contagious illness](#health) |
| [continuity_fixture.rs](../src/continuity_fixture.rs) | [Projection, refinement and reconciliation](#resolution) |
| [credit.rs](../src/credit.rs) | [Loan contracts and account transfers](#credit) |
| [credit/accounts.rs](../src/credit/accounts.rs) | [Loan contracts and account transfers](#credit) |
| [credit/chronicle.rs](../src/credit/chronicle.rs) | [Monetary reports, chronicle and explorer](#creditreport) |
| [credit/commercial.rs](../src/credit/commercial.rs) | [Commercial and service-order credit](#commercialcredit) |
| [credit/councils.rs](../src/credit/councils.rs) | [Council tax bridges and institutional lenders](#councilcredit) |
| [credit/estates.rs](../src/credit/estates.rs) | [Closed accounts and creditor succession](#creditestates) |
| [credit/export_recovery.rs](../src/credit/export_recovery.rs) | [Default and late-proceeds recovery](#debtrecovery) |
| [credit/exports.rs](../src/credit/exports.rs) | [Lending requests and underwriting](#underwriting) |
| [credit/issuance.rs](../src/credit/issuance.rs) | [Bounded shared-currency issuance](#issuance) |
| [credit/ownership.rs](../src/credit/ownership.rs) | [Closed accounts and creditor succession](#creditestates) |
| [credit/recovery.rs](../src/credit/recovery.rs) | [Default and late-proceeds recovery](#debtrecovery) |
| [credit/report.rs](../src/credit/report.rs) | [Monetary reports, chronicle and explorer](#creditreport) |
| [credit/restructuring.rs](../src/credit/restructuring.rs) | [Repayment, arrears and restructuring](#debtservice) |
| [credit/servicing.rs](../src/credit/servicing.rs) | [Repayment, arrears and restructuring](#debtservice) |
| [credit/state.rs](../src/credit/state.rs) | [Loan contracts and account transfers](#credit) |
| [credit/taxes.rs](../src/credit/taxes.rs) | [Lending requests and underwriting](#underwriting) |
| [credit/underwriting.rs](../src/credit/underwriting.rs) | [Lending requests and underwriting](#underwriting) |
| [culture.rs](../src/culture.rs) | [Patron arrivals, witnesses and service](#patrons), [Cultural people, goals and unique objects](#agents) |
| [culture/dynamics.rs](../src/culture/dynamics.rs) | [Religious affiliation, interpretation and practices](#religion) |
| [culture/learning.rs](../src/culture/learning.rs) | [Knowledge transmission and learning resolution](#learning) |
| [culture/practical_research.rs](../src/culture/practical_research.rs) | [Practical production knowledge research](#research) |
| [culture/practices.rs](../src/culture/practices.rs) | [Religious affiliation, interpretation and practices](#religion), [Cultural people, goals and unique objects](#agents) |
| [culture/work_requests.rs](../src/culture/work_requests.rs) | [Research/culture allocation and execution plans](#service) |
| [demographic_audit.rs](../src/demographic_audit.rs) | [Demographic windows and food-flow diagnostics](#diagnostics) |
| [demographic_audit/food.rs](../src/demographic_audit/food.rs) | [Demographic windows and food-flow diagnostics](#diagnostics) |
| [discoveries.rs](../src/discoveries.rs) | [Finite collections, research and applications](#discoveries) |
| [discoveries/returns.rs](../src/discoveries/returns.rs) | [Finite collections, research and applications](#discoveries) |
| [domestic.rs](../src/domestic.rs) | [Domestic groups, dependents and care](#domestic) |
| [domestic/assistance.rs](../src/domestic/assistance.rs) | [Domestic groups, dependents and care](#domestic) |
| [domestic/resolution.rs](../src/domestic/resolution.rs) | [Domestic groups, dependents and care](#domestic) |
| [ecology.rs](../src/ecology.rs) | [Ecological stocks, layered producers and aquatic compartments](#aquatic), [Food webs, dispersal and ecotypes](#wildlife) |
| [economy.rs](../src/economy.rs) | [Farm replenishment, manure retention and phosphorus runoff](#nutrients), [Fishing effort, gear and local opportunity cost](#fisheries), [Goods, demand, prices and network trade](#market) |
| [enterprises.rs](../src/enterprises.rs) | [Enterprise ownership and workshop capital](#operators) |
| [enterprises/feasibility.rs](../src/enterprises/feasibility.rs) | [Enterprise ownership and workshop capital](#operators) |
| [enterprises/orders.rs](../src/enterprises/orders.rs) | [Funded workshop service orders](#orders) |
| [environmental_returns.rs](../src/environmental_returns.rs) | [Environmental returns and land recovery](#returns), [Farm replenishment, manure retention and phosphorus runoff](#nutrients) |
| [expedition_heritage.rs](../src/expedition_heritage.rs) | [Archaeology, patron finds and heritage](#heritage) |
| [expedition_heritage/patron_finds.rs](../src/expedition_heritage/patron_finds.rs) | [Archaeology, patron finds and heritage](#heritage) |
| [expeditions.rs](../src/expeditions.rs) | [Expedition objectives, crews and frontier travel](#expeditions) |
| [export_contracts.rs](../src/export_contracts.rs) | [Export contracts and payment custody](#exports) |
| [export_contracts/identities.rs](../src/export_contracts/identities.rs) | [Export contracts and payment custody](#exports) |
| [export_contracts/payments.rs](../src/export_contracts/payments.rs) | [Export contracts and payment custody](#exports) |
| [facilities.rs](../src/facilities.rs) | [Material substitution and scalable facilities](#materials) |
| [faction_interests.rs](../src/faction_interests.rs) | [Genealogy, factions and political interests](#politics) |
| [freight.rs](../src/freight.rs) | [Land corridors and road upkeep](#freight) |
| [governance.rs](../src/governance.rs) | [Government, duties and diplomacy](#governance) |
| [gpu.rs](../src/gpu.rs) | [Planet generation and compute orchestration](#planet), [Planetary tectonics, geology and mineral potential](#geology), [Basins, rivers, lakes and sediment routing](#hydrology), [Seasonal climate and transient weather](#climate) |
| [grid.rs](../src/grid.rs) | [Planet generation and compute orchestration](#planet) |
| [hazards.rs](../src/hazards.rs) | [Floods and delayed cargo spoilage](#hazards) |
| [heritage_renown.rs](../src/heritage_renown.rs) | [Expedition renown and heritage stewardship](#renown) |
| [history_atlas.rs](../src/history_atlas.rs) | [Globe, atlas and history explorer](#viewer) |
| [history_environment.rs](../src/history_environment.rs) | [Living history, surveys and monthly environment](#living) |
| [history_timeline.rs](../src/history_timeline.rs) | [Globe, atlas and history explorer](#viewer) |
| [household_economy.rs](../src/household_economy.rs) | [Household ownership, income and purchasing](#retail) |
| [household_economy/clothing.rs](../src/household_economy/clothing.rs) | [Household clothing and replacement demand](#clothing) |
| [household_economy/council_allocation.rs](../src/household_economy/council_allocation.rs) | [Council and municipal relief policies](#welfare) |
| [household_economy/family_support.rs](../src/household_economy/family_support.rs) | [Family assistance and local solidarity](#familycash) |
| [household_economy/inheritance.rs](../src/household_economy/inheritance.rs) | [Household inheritance and unclaimed estates](#estates) |
| [household_economy/municipal_relief.rs](../src/household_economy/municipal_relief.rs) | [Council and municipal relief policies](#welfare) |
| [household_economy/nutrition.rs](../src/household_economy/nutrition.rs) | [Household ownership, income and purchasing](#retail) |
| [household_economy/policy.rs](../src/household_economy/policy.rs) | [Council and municipal relief policies](#welfare) |
| [household_economy/reclamation.rs](../src/household_economy/reclamation.rs) | [Household inheritance and unclaimed estates](#estates) |
| [household_economy/solidarity.rs](../src/household_economy/solidarity.rs) | [Family assistance and local solidarity](#familycash) |
| [household_economy/wealth_tax.rs](../src/household_economy/wealth_tax.rs) | [Progressive household cash tax](#wealthtax) |
| [individual_demography.rs](../src/individual_demography.rs) | [Resident identity and individual demography](#residents) |
| [institution_capacity.rs](../src/institution_capacity.rs) | [Institution capacity, facilities and service funding](#institutions) |
| [institution_funding.rs](../src/institution_funding.rs) | [Institution capacity, facilities and service funding](#institutions) |
| [institution_relocation.rs](../src/institution_relocation.rs) | [Institution relocation](#instmove) |
| [institution_services.rs](../src/institution_services.rs) | [Institution capacity, facilities and service funding](#institutions) |
| [institution_succession.rs](../src/institution_succession.rs) | [Institution succession and leadership recovery](#instleadership) |
| [kin_support.rs](../src/kin_support.rs) | [Domestic groups, dependents and care](#domestic) |
| [labor.rs](../src/labor.rs) | [Activity commitments and labor receipts](#participation) |
| [leadership.rs](../src/leadership.rs) | [Constitutional succession and representation](#leadership) |
| [learning_resolution.rs](../src/learning_resolution.rs) | [Knowledge transmission and learning resolution](#learning) |
| [lib.rs](../src/lib.rs) | [Configuration, registry and library entry points](#configuration) |
| [local_places.rs](../src/local_places.rs) | [Persistent historical places and material evidence](#places) |
| [main.rs](../src/main.rs) | [Configuration, registry and library entry points](#configuration) |
| [materials.rs](../src/materials.rs) | [Material substitution and scalable facilities](#materials) |
| [metallurgy.rs](../src/metallurgy.rs) | [Mineral processing and alloys](#metals) |
| [military.rs](../src/military.rs) | [Armies, campaigns and finite occupation](#military) |
| [military_supply.rs](../src/military_supply.rs) | [Armies, campaigns and finite occupation](#military) |
| [naming.rs](../src/naming.rs) | [Semantic names and civilization languages](#naming) |
| [naming/evolution.rs](../src/naming/evolution.rs) | [Semantic names and civilization languages](#naming) |
| [navigation.rs](../src/navigation.rs) | [GPU navigation and resource surveys](#navigation) |
| [occupation.rs](../src/occupation.rs) | [Armies, campaigns and finite occupation](#military) |
| [offices.rs](../src/offices.rs) | [Local offices and named attendance](#offices) |
| [offices/service.rs](../src/offices/service.rs) | [Local offices and named attendance](#offices) |
| [participation.rs](../src/participation.rs) | [Activity commitments and labor receipts](#participation) |
| [peace.rs](../src/peace.rs) | [Negotiated peace and payment obligations](#peace) |
| [politics.rs](../src/politics.rs) | [Genealogy, factions and political interests](#politics) |
| [population_registry.rs](../src/population_registry.rs) | [Resident identity and individual demography](#residents) |
| [production.rs](../src/production.rs) | [Farm replenishment, manure retention and phosphorus runoff](#nutrients), [Production planning and essential maintenance](#production) |
| [region.rs](../src/region.rs) | [Regional terrain, geological provinces and strata](#regional) |
| [regional_mining.rs](../src/regional_mining.rs) | [Canonical resource stocks and extraction](#resources) |
| [relief.rs](../src/relief.rs) | [Secular and religious relief journeys](#relief) |
| [religious_relief.rs](../src/religious_relief.rs) | [Secular and religious relief journeys](#relief) |
| [relocation.rs](../src/relocation.rs) | [Household journeys and travel comparison](#relocation) |
| [relocation/comparison.rs](../src/relocation/comparison.rs) | [Household journeys and travel comparison](#relocation) |
| [relocation/resettlement.rs](../src/relocation/resettlement.rs) | [Ruin resettlement and expiring claims](#resettlement) |
| [resolution.rs](../src/resolution.rs) | [Projection, refinement and reconciliation](#resolution) |
| [resources.rs](../src/resources.rs) | [Canonical resource stocks and extraction](#resources) |
| [road_upkeep.rs](../src/road_upkeep.rs) | [Land corridors and road upkeep](#freight) |
| [route_warnings.rs](../src/route_warnings.rs) | [Traveling danger warnings](#warnings) |
| [service_allocation.rs](../src/service_allocation.rs) | [Research/culture allocation and execution plans](#service) |
| [shipping.rs](../src/shipping.rs) | [Ports, harbors and sea trade](#shipping) |
| [siege.rs](../src/siege.rs) | [Defensive assets, land sieges and resupply](#siege) |
| [social_memory.rs](../src/social_memory.rs) | [Social projections, pressures and local memory](#social) |
| [social_state.rs](../src/social_state.rs) | [Social projections, pressures and local memory](#social) |
| [society.rs](../src/society.rs) | [Aggregate demography, households and seasons](#society) |
| [spatial.rs](../src/spatial.rs) | [Spatial features and territorial history](#spatial) |
| [stock_recovery.rs](../src/stock_recovery.rs) | [Abandoned bulk-stock recovery](#recovery) |
| [storage.rs](../src/storage.rs) | [World archives and consistent checkpoints](#archives) |
| [systems.rs](../src/systems.rs) | [Configuration, registry and library entry points](#configuration) |
| [systems/policies.rs](../src/systems/policies.rs) | [Configuration, registry and library entry points](#configuration) |
| [territory.rs](../src/territory.rs) | [Spatial features and territorial history](#spatial) |
| [tool_access.rs](../src/tool_access.rs) | [Mineral processing and alloys](#metals) |
| [trade_contact.rs](../src/trade_contact.rs) | [Recent trade and cultural contact](#contact) |
| [vessels.rs](../src/vessels.rs) | [Vessels and named crew service](#vessels) |
| [vessels/crews.rs](../src/vessels/crews.rs) | [Vessels and named crew service](#vessels) |
| [vessels/resolution.rs](../src/vessels/resolution.rs) | [Vessels and named crew service](#vessels) |
| [viewer.rs](../src/viewer.rs) | [Globe, atlas and history explorer](#viewer) |
| [viewer_credit.rs](../src/viewer_credit.rs) | [Monetary reports, chronicle and explorer](#creditreport) |
| [workshop_resolution.rs](../src/workshop_resolution.rs) | [Workshop individual refinement](#workshops) |

### Shaders and editable assets

| File | Owning feature/evidence family |
| --- | --- |
| [assets/agriculture-seasonal.toml](../assets/agriculture-seasonal.toml) | [Validated editable catalogs](#catalogs) |
| [assets/agriculture.toml](../assets/agriculture.toml) | [Validated editable catalogs](#catalogs) |
| [assets/catalog.toml](../assets/catalog.toml) | [Validated editable catalogs](#catalogs) |
| [assets/economy.toml](../assets/economy.toml) | [Validated editable catalogs](#catalogs) |
| [assets/example.toml](../assets/example.toml) | [Configuration, registry and library entry points](#configuration) |
| [assets/materials.toml](../assets/materials.toml) | [Validated editable catalogs](#catalogs) |
| [assets/patrons.toml](../assets/patrons.toml) | [Patron arrivals, witnesses and service](#patrons) |
| [assets/scenario-example.json](../assets/scenario-example.json) | [Configuration, registry and library entry points](#configuration) |
| [shaders/civilization.wgsl](../shaders/civilization.wgsl) | [Monthly schedule and history authority](#history) |
| [shaders/ecology.wgsl](../shaders/ecology.wgsl) | [Ecological stocks, layered producers and aquatic compartments](#aquatic), [Food webs, dispersal and ecotypes](#wildlife) |
| [shaders/economy.wgsl](../shaders/economy.wgsl) | [Farm replenishment, manure retention and phosphorus runoff](#nutrients), [Crop, livestock and fishery production](#agriculture), [Fishing effort, gear and local opportunity cost](#fisheries), [Goods, demand, prices and network trade](#market) |
| [shaders/history_environment.wgsl](../shaders/history_environment.wgsl) | [Living history, surveys and monthly environment](#living) |
| [shaders/history_weather.wgsl](../shaders/history_weather.wgsl) | [Living history, surveys and monthly environment](#living), [Seasonal climate and transient weather](#climate) |
| [shaders/managed_returns.wgsl](../shaders/managed_returns.wgsl) | [Environmental returns and land recovery](#returns) |
| [shaders/navigation.wgsl](../shaders/navigation.wgsl) | [GPU navigation and resource surveys](#navigation) |
| [shaders/navigation_inspect.wgsl](../shaders/navigation_inspect.wgsl) | [GPU navigation and resource surveys](#navigation) |
| [shaders/navigation_survey.wgsl](../shaders/navigation_survey.wgsl) | [GPU navigation and resource surveys](#navigation) |
| [shaders/region_compute.wgsl](../shaders/region_compute.wgsl) | [Regional terrain, geological provinces and strata](#regional) |
| [shaders/regional.wgsl](../shaders/regional.wgsl) | [Regional terrain, geological provinces and strata](#regional) |
| [shaders/simulation.wgsl](../shaders/simulation.wgsl) | [Planet generation and compute orchestration](#planet), [Planetary tectonics, geology and mineral potential](#geology), [Basins, rivers, lakes and sediment routing](#hydrology), [Seasonal climate and transient weather](#climate) |
| [shaders/society.wgsl](../shaders/society.wgsl) | [Aggregate demography, households and seasons](#society) |
| [shaders/sunlight.wgsl](../shaders/sunlight.wgsl) | [Seasonal sunlight](#sunlight) |
| [shaders/view.wgsl](../shaders/view.wgsl) | [Globe, atlas and history explorer](#viewer) |

### Standalone experiment modules

| File | Owning feature/evidence family |
| --- | --- |
| [exp/economics/src/activities.rs](../exp/economics/src/activities.rs) | [Durable equipment and specialized activities](#xproduction) |
| [exp/economics/src/agreements.rs](../exp/economics/src/agreements.rs) | [Common accepted agreements and consequences](#xagreements) |
| [exp/economics/src/allocation.rs](../exp/economics/src/allocation.rs) | [Contested offers and simultaneous allocation](#xallocation) |
| [exp/economics/src/commitments.rs](../exp/economics/src/commitments.rs) | [Access, membership and production plots](#xrights) |
| [exp/economics/src/competition.rs](../exp/economics/src/competition.rs) | [Contested offers and simultaneous allocation](#xallocation) |
| [exp/economics/src/compute.rs](../exp/economics/src/compute.rs) | [Process model, settlement and CPU compute](#xcore) |
| [exp/economics/src/crafts.rs](../exp/economics/src/crafts.rs) | [Durable equipment and specialized activities](#xproduction) |
| [exp/economics/src/currency.rs](../exp/economics/src/currency.rs) | [Stock exchange, tokens and commodity forwards](#xtrade) |
| [exp/economics/src/equipment.rs](../exp/economics/src/equipment.rs) | [Durable equipment and specialized activities](#xproduction) |
| [exp/economics/src/exchange.rs](../exp/economics/src/exchange.rs) | [Stock exchange, tokens and commodity forwards](#xtrade) |
| [exp/economics/src/finance.rs](../exp/economics/src/finance.rs) | [Stock exchange, tokens and commodity forwards](#xtrade) |
| [exp/economics/src/forward.rs](../exp/economics/src/forward.rs) | [Stock exchange, tokens and commodity forwards](#xtrade) |
| [exp/economics/src/households.rs](../exp/economics/src/households.rs) | [Agreement-formed households](#xhouseholds) |
| [exp/economics/src/lib.rs](../exp/economics/src/lib.rs) | [Process model, settlement and CPU compute](#xcore) |
| [exp/economics/src/main.rs](../exp/economics/src/main.rs) | [Process model, settlement and CPU compute](#xcore) |
| [exp/economics/src/maintenance.rs](../exp/economics/src/maintenance.rs) | [Needs, finite pools, storage and substitution](#xneeds) |
| [exp/economics/src/membership.rs](../exp/economics/src/membership.rs) | [Access, membership and production plots](#xrights) |
| [exp/economics/src/model.rs](../exp/economics/src/model.rs) | [Process model, settlement and CPU compute](#xcore) |
| [exp/economics/src/offers.rs](../exp/economics/src/offers.rs) | [Common accepted agreements and consequences](#xagreements) |
| [exp/economics/src/opportunities.rs](../exp/economics/src/opportunities.rs) | [Planning, search and opportunity discovery](#xsearch) |
| [exp/economics/src/planning.rs](../exp/economics/src/planning.rs) | [Planning, search and opportunity discovery](#xsearch) |
| [exp/economics/src/plots.rs](../exp/economics/src/plots.rs) | [Access, membership and production plots](#xrights) |
| [exp/economics/src/pools.rs](../exp/economics/src/pools.rs) | [Needs, finite pools, storage and substitution](#xneeds) |
| [exp/economics/src/scenario.rs](../exp/economics/src/scenario.rs) | [Controlled standalone scenarios](#xscenarios) |
| [exp/economics/src/search.rs](../exp/economics/src/search.rs) | [Planning, search and opportunity discovery](#xsearch) |
| [exp/economics/src/settlement.rs](../exp/economics/src/settlement.rs) | [Process model, settlement and CPU compute](#xcore) |
| [exp/economics/src/simulation.rs](../exp/economics/src/simulation.rs) | [Process model, settlement and CPU compute](#xcore) |
| [exp/economics/src/storage.rs](../exp/economics/src/storage.rs) | [Needs, finite pools, storage and substitution](#xneeds) |
| [exp/economics/src/substitution.rs](../exp/economics/src/substitution.rs) | [Needs, finite pools, storage and substitution](#xneeds) |
| [exp/economics/src/trading_scenario.rs](../exp/economics/src/trading_scenario.rs) | [Controlled standalone scenarios](#xscenarios) |

### Evaluation and maintenance programs

| File | Owning feature/evidence family |
| --- | --- |
| [examples/alloy_evaluation.rs](../examples/alloy_evaluation.rs) | [Production and service evaluation](#productiontools) |
| [examples/calibrate.rs](../examples/calibrate.rs) | [Ecology and continuity controls](#ecologicaltools) |
| [examples/civic_balance.rs](../examples/civic_balance.rs) | [Integrated history and policy evidence](#integrationtools) |
| [examples/coupling_evidence.rs](../examples/coupling_evidence.rs) | [Ecology and continuity controls](#ecologicaltools) |
| [examples/cultural_work_calibrate.rs](../examples/cultural_work_calibrate.rs) | [Production and service evaluation](#productiontools) |
| [examples/foodweb_evaluate.rs](../examples/foodweb_evaluate.rs) | [Ecology and continuity controls](#ecologicaltools) |
| [examples/growth_ladder.rs](../examples/growth_ladder.rs) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [examples/history_evaluate.rs](../examples/history_evaluate.rs) | [Integrated history and policy evidence](#integrationtools) |
| [examples/history_profile.rs](../examples/history_profile.rs) | [Performance and repository checks](#performance) |
| [examples/history_replay.rs](../examples/history_replay.rs) | [Integrated history and policy evidence](#integrationtools) |
| [examples/inner_land_count.rs](../examples/inner_land_count.rs) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [examples/lake_relaxation.rs](../examples/lake_relaxation.rs) | [Performance and repository checks](#performance) |
| [examples/living_fishery_evaluate.rs](../examples/living_fishery_evaluate.rs) | [Ecology and continuity controls](#ecologicaltools) |
| [examples/nutrition_evaluate.rs](../examples/nutrition_evaluate.rs) | [Monetary and distribution experiments](#financialtools) |
| [examples/region.rs](../examples/region.rs) | [Ecology and continuity controls](#ecologicaltools) |
| [examples/resource_counterfactual.rs](../examples/resource_counterfactual.rs) | [Ecology and continuity controls](#ecologicaltools) |
| [examples/toolmaking_evidence.rs](../examples/toolmaking_evidence.rs) | [Production and service evaluation](#productiontools) |
| [examples/wildlife_evaluate.rs](../examples/wildlife_evaluate.rs) | [Ecology and continuity controls](#ecologicaltools) |
| [scripts/analyze_civic_balance.py](../scripts/analyze_civic_balance.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/analyze_flood_audit.py](../scripts/analyze_flood_audit.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/analyze_ideas_audit.py](../scripts/analyze_ideas_audit.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/analyze_integrated_history.py](../scripts/analyze_integrated_history.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/analyze_living_fisheries.py](../scripts/analyze_living_fisheries.py) | [Ecology and continuity controls](#ecologicaltools) |
| [scripts/audit_circulation.py](../scripts/audit_circulation.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/build_history_evaluator.py](../scripts/build_history_evaluator.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/check_continuity.py](../scripts/check_continuity.py) | [Ecology and continuity controls](#ecologicaltools) |
| [scripts/check_repository_artifacts.py](../scripts/check_repository_artifacts.py) | [Performance and repository checks](#performance) |
| [scripts/compare_council_funding.py](../scripts/compare_council_funding.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/compare_credit_chronicle.py](../scripts/compare_credit_chronicle.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/compare_food_access.py](../scripts/compare_food_access.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/compare_integrated_history.py](../scripts/compare_integrated_history.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/economy_report.py](../scripts/economy_report.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/evaluate_enterprises.py](../scripts/evaluate_enterprises.py) | [Production and service evaluation](#productiontools) |
| [scripts/evidence.py](../scripts/evidence.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/integrated_history.py](../scripts/integrated_history.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/monetary_experiment.py](../scripts/monetary_experiment.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/patron_audit.py](../scripts/patron_audit.py) | [Patron and expedition evaluation](#patrontools) |
| [scripts/patron_report.py](../scripts/patron_report.py) | [Patron and expedition evaluation](#patrontools) |
| [scripts/policy_suite.py](../scripts/policy_suite.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/pressure_sweep.py](../scripts/pressure_sweep.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/publish_enterprise_evidence.py](../scripts/publish_enterprise_evidence.py) | [Production and service evaluation](#productiontools) |
| [scripts/report_demographic_windows.py](../scripts/report_demographic_windows.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/report_early_food.py](../scripts/report_early_food.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/report_founding_farms.py](../scripts/report_founding_farms.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/report_growth_bottlenecks.py](../scripts/report_growth_bottlenecks.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/report_monthly_food.py](../scripts/report_monthly_food.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/run_growth_investigation.py](../scripts/run_growth_investigation.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/run_integration_balance.py](../scripts/run_integration_balance.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/settlement_audit.py](../scripts/settlement_audit.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/summarize_coupling.py](../scripts/summarize_coupling.py) | [Ecology and continuity controls](#ecologicaltools) |
| [scripts/summarize_cultural_work.py](../scripts/summarize_cultural_work.py) | [Production and service evaluation](#productiontools) |
| [scripts/summarize_expedition_crews.py](../scripts/summarize_expedition_crews.py) | [Patron and expedition evaluation](#patrontools) |
| [scripts/summarize_food_access.py](../scripts/summarize_food_access.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/summarize_growth_food.py](../scripts/summarize_growth_food.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/summarize_growth_funding.py](../scripts/summarize_growth_funding.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/summarize_harbor_work.py](../scripts/summarize_harbor_work.py) | [Production and service evaluation](#productiontools) |
| [scripts/summarize_institution_succession.py](../scripts/summarize_institution_succession.py) | [Production and service evaluation](#productiontools) |
| [scripts/summarize_nutrition.py](../scripts/summarize_nutrition.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/summarize_road_upkeep.py](../scripts/summarize_road_upkeep.py) | [Production and service evaluation](#productiontools) |
| [scripts/summarize_toolmaking.py](../scripts/summarize_toolmaking.py) | [Production and service evaluation](#productiontools) |
| [scripts/waterworks_report.py](../scripts/waterworks_report.py) | [Production and service evaluation](#productiontools) |
| [scripts/wildlife_report.py](../scripts/wildlife_report.py) | [Ecology and continuity controls](#ecologicaltools) |
| [.github/workflows/evidence.yml](../.github/workflows/evidence.yml) | [Evidence CI workflow](#ci) |

### Tests and standalone audit programs

| File | Owning feature/evidence family |
| --- | --- |
| [exp/economics/examples/activities_audit.rs](../exp/economics/examples/activities_audit.rs) | [Durable equipment and specialized activities](#xproduction) |
| [exp/economics/examples/foraging_audit.rs](../exp/economics/examples/foraging_audit.rs) | [Needs, finite pools, storage and substitution](#xneeds) |
| [exp/economics/examples/forward_audit.rs](../exp/economics/examples/forward_audit.rs) | [Stock exchange, tokens and commodity forwards](#xtrade) |
| [exp/economics/examples/household_audit.rs](../exp/economics/examples/household_audit.rs) | [Agreement-formed households](#xhouseholds) |
| [exp/economics/examples/multi_person_audit.rs](../exp/economics/examples/multi_person_audit.rs) | [Controlled standalone scenarios](#xscenarios) |
| [exp/economics/examples/plots_audit.rs](../exp/economics/examples/plots_audit.rs) | [Access, membership and production plots](#xrights) |
| [exp/economics/examples/production_audit.rs](../exp/economics/examples/production_audit.rs) | [Durable equipment and specialized activities](#xproduction) |
| [exp/economics/examples/storage_currency_audit.rs](../exp/economics/examples/storage_currency_audit.rs) | [Needs, finite pools, storage and substitution](#xneeds) |
| [exp/economics/examples/trading_audit.rs](../exp/economics/examples/trading_audit.rs) | [Stock exchange, tokens and commodity forwards](#xtrade) |
| [exp/economics/tests/access.rs](../exp/economics/tests/access.rs) | [Access, membership and production plots](#xrights) |
| [exp/economics/tests/activities.rs](../exp/economics/tests/activities.rs) | [Durable equipment and specialized activities](#xproduction) |
| [exp/economics/tests/agreements.rs](../exp/economics/tests/agreements.rs) | [Common accepted agreements and consequences](#xagreements) |
| [exp/economics/tests/allocation.rs](../exp/economics/tests/allocation.rs) | [Contested offers and simultaneous allocation](#xallocation) |
| [exp/economics/tests/commitments.rs](../exp/economics/tests/commitments.rs) | [Access, membership and production plots](#xrights) |
| [exp/economics/tests/competition.rs](../exp/economics/tests/competition.rs) | [Contested offers and simultaneous allocation](#xallocation) |
| [exp/economics/tests/conditions.rs](../exp/economics/tests/conditions.rs) | [Needs, finite pools, storage and substitution](#xneeds) |
| [exp/economics/tests/economics.rs](../exp/economics/tests/economics.rs) | [Process model, settlement and CPU compute](#xcore) |
| [exp/economics/tests/equipment.rs](../exp/economics/tests/equipment.rs) | [Durable equipment and specialized activities](#xproduction) |
| [exp/economics/tests/exchange.rs](../exp/economics/tests/exchange.rs) | [Stock exchange, tokens and commodity forwards](#xtrade) |
| [exp/economics/tests/experience.rs](../exp/economics/tests/experience.rs) | [Durable equipment and specialized activities](#xproduction) |
| [exp/economics/tests/finance.rs](../exp/economics/tests/finance.rs) | [Stock exchange, tokens and commodity forwards](#xtrade) |
| [exp/economics/tests/foraging.rs](../exp/economics/tests/foraging.rs) | [Needs, finite pools, storage and substitution](#xneeds) |
| [exp/economics/tests/forward.rs](../exp/economics/tests/forward.rs) | [Stock exchange, tokens and commodity forwards](#xtrade) |
| [exp/economics/tests/households.rs](../exp/economics/tests/households.rs) | [Agreement-formed households](#xhouseholds) |
| [exp/economics/tests/membership.rs](../exp/economics/tests/membership.rs) | [Access, membership and production plots](#xrights) |
| [exp/economics/tests/multi_person.rs](../exp/economics/tests/multi_person.rs) | [Controlled standalone scenarios](#xscenarios) |
| [exp/economics/tests/opportunities.rs](../exp/economics/tests/opportunities.rs) | [Planning, search and opportunity discovery](#xsearch) |
| [exp/economics/tests/payment_policy.rs](../exp/economics/tests/payment_policy.rs) | [Access, membership and production plots](#xrights) |
| [exp/economics/tests/planning.rs](../exp/economics/tests/planning.rs) | [Planning, search and opportunity discovery](#xsearch) |
| [exp/economics/tests/plots.rs](../exp/economics/tests/plots.rs) | [Access, membership and production plots](#xrights) |
| [exp/economics/tests/pricing.rs](../exp/economics/tests/pricing.rs) | [Stock exchange, tokens and commodity forwards](#xtrade) |
| [exp/economics/tests/process_offers.rs](../exp/economics/tests/process_offers.rs) | [Common accepted agreements and consequences](#xagreements) |
| [exp/economics/tests/repeated_and_warmth.rs](../exp/economics/tests/repeated_and_warmth.rs) | [Needs, finite pools, storage and substitution](#xneeds) |
| [exp/economics/tests/search.rs](../exp/economics/tests/search.rs) | [Planning, search and opportunity discovery](#xsearch) |
| [exp/economics/tests/storage_currency.rs](../exp/economics/tests/storage_currency.rs) | [Needs, finite pools, storage and substitution](#xneeds) |
| [scripts/test_audit_circulation.py](../scripts/test_audit_circulation.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/test_compare_council_funding.py](../scripts/test_compare_council_funding.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/test_compare_food_access.py](../scripts/test_compare_food_access.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/test_evaluate_enterprises.py](../scripts/test_evaluate_enterprises.py) | [Production and service evaluation](#productiontools) |
| [scripts/test_evidence.py](../scripts/test_evidence.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/test_growth_food_summary.py](../scripts/test_growth_food_summary.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/test_integrated_history.py](../scripts/test_integrated_history.py) | [Integrated history and policy evidence](#integrationtools) |
| [scripts/test_monetary_experiment.py](../scripts/test_monetary_experiment.py) | [Monetary and distribution experiments](#financialtools) |
| [scripts/test_report_demographic_windows.py](../scripts/test_report_demographic_windows.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/test_report_early_food.py](../scripts/test_report_early_food.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/test_report_founding_farms.py](../scripts/test_report_founding_farms.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/test_report_growth_bottlenecks.py](../scripts/test_report_growth_bottlenecks.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/test_report_monthly_food.py](../scripts/test_report_monthly_food.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/test_run_growth_investigation.py](../scripts/test_run_growth_investigation.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [scripts/test_summarize_growth_funding.py](../scripts/test_summarize_growth_funding.py) | [Growth stage gates and food/nutrient investigation](#growthtools) |
| [tests/adaptive_fisheries.rs](../tests/adaptive_fisheries.rs) | [Fishing effort, gear and local opportunity cost](#fisheries) |
| [tests/alloy_processing.rs](../tests/alloy_processing.rs) | [Mineral processing and alloys](#metals) |
| [tests/civilization.rs](../tests/civilization.rs) | [Monthly schedule and history authority](#history), [Growth controls and settlement admission](#expansion), [Resident identity and individual demography](#residents), [Projection, refinement and reconciliation](#resolution), [Ruin resettlement and expiring claims](#resettlement) |
| [tests/core.rs](../tests/core.rs) | [Planet generation and compute orchestration](#planet), [Basins, rivers, lakes and sediment routing](#hydrology), [Configuration, registry and library entry points](#configuration), [Validated editable catalogs](#catalogs), [World archives and consistent checkpoints](#archives) |
| [tests/culture.rs](../tests/culture.rs) | [Material substitution and scalable facilities](#materials), [Secular and religious relief journeys](#relief), [Patron arrivals, witnesses and service](#patrons), [Religious affiliation, interpretation and practices](#religion), [Cultural people, goals and unique objects](#agents), [Artifact ownership petitions](#artifactclaims), [Institution capacity, facilities and service funding](#institutions), [Institution succession and leadership recovery](#instleadership), [Institution relocation](#instmove), [Research/culture allocation and execution plans](#service), [Knowledge transmission and learning resolution](#learning), [Practical production knowledge research](#research), [Expedition renown and heritage stewardship](#renown), [Semantic names and civilization languages](#naming), [Recent trade and cultural contact](#contact) |
| [tests/discoveries.rs](../tests/discoveries.rs) | [Research/culture allocation and execution plans](#service), [Finite collections, research and applications](#discoveries) |
| [tests/ecology.rs](../tests/ecology.rs) | [Ecological stocks, layered producers and aquatic compartments](#aquatic), [Food webs, dispersal and ecotypes](#wildlife) |
| [tests/economy.rs](../tests/economy.rs) | [Farm, extraction and construction participation](#farmwork), [Workshop individual refinement](#workshops), [Household ownership, income and purchasing](#retail), [Household clothing and replacement demand](#clothing), [Crop, livestock and fishery production](#agriculture), [Goods, demand, prices and network trade](#market), [Enterprise ownership and workshop capital](#operators), [Funded workshop service orders](#orders) |
| [tests/environmental_returns.rs](../tests/environmental_returns.rs) | [Environmental returns and land recovery](#returns) |
| [tests/expeditions.rs](../tests/expeditions.rs) | [Expedition objectives, crews and frontier travel](#expeditions), [Archaeology, patron finds and heritage](#heritage) |
| [tests/farm_nutrients.rs](../tests/farm_nutrients.rs) | [Farm replenishment, manure retention and phosphorus runoff](#nutrients), [Crop, livestock and fishery production](#agriculture) |
| [tests/food_diagnostics.rs](../tests/food_diagnostics.rs) | [Demographic windows and food-flow diagnostics](#diagnostics) |
| [tests/founding_food.rs](../tests/founding_food.rs) | [Arrival provisions and daughter settlement supplies](#founding) |
| [tests/geology.rs](../tests/geology.rs) | [Planet generation and compute orchestration](#planet), [Regional terrain, geological provinces and strata](#regional), [Planetary tectonics, geology and mineral potential](#geology) |
| [tests/governance.rs](../tests/governance.rs) | [Government, duties and diplomacy](#governance), [Local offices and named attendance](#offices), [Civic petitions and causal hearings](#petitions) |
| [tests/gpu.rs](../tests/gpu.rs) | [Planet generation and compute orchestration](#planet), [Basins, rivers, lakes and sediment routing](#hydrology) |
| [tests/health_labor.rs](../tests/health_labor.rs) | [Aggregate demography, households and seasons](#society), [Activity commitments and labor receipts](#participation), [Domestic groups, dependents and care](#domestic), [Traveler-linked contagious illness](#health) |
| [tests/history_environment.rs](../tests/history_environment.rs) | [Living history, surveys and monthly environment](#living), [Seasonal climate and transient weather](#climate) |
| [tests/history_timeline.rs](../tests/history_timeline.rs) | [Spatial features and territorial history](#spatial), [Globe, atlas and history explorer](#viewer) |
| [tests/housing.rs](../tests/housing.rs) | [Production planning and essential maintenance](#production) |
| [tests/living.rs](../tests/living.rs) | [Living history, surveys and monthly environment](#living), [Floods and delayed cargo spoilage](#hazards), [Seasonal climate and transient weather](#climate), [World archives and consistent checkpoints](#archives) |
| [tests/living_scenarios.rs](../tests/living_scenarios.rs) | [Ecological stocks, layered producers and aquatic compartments](#aquatic) |
| [tests/markets.rs](../tests/markets.rs) | [Goods, demand, prices and network trade](#market), [Export contracts and payment custody](#exports), [Land corridors and road upkeep](#freight) |
| [tests/mineral_processing.rs](../tests/mineral_processing.rs) | [Mineral processing and alloys](#metals) |
| [tests/navigation.rs](../tests/navigation.rs) | [GPU navigation and resource surveys](#navigation) |
| [tests/nutrient_retention.rs](../tests/nutrient_retention.rs) | [Farm replenishment, manure retention and phosphorus runoff](#nutrients) |
| [tests/politics.rs](../tests/politics.rs) | [Genealogy, factions and political interests](#politics), [Constitutional succession and representation](#leadership), [Armies, campaigns and finite occupation](#military), [Negotiated peace and payment obligations](#peace), [Defensive assets, land sieges and resupply](#siege) |
| [tests/rations.rs](../tests/rations.rs) | [Household ownership, income and purchasing](#retail), [Family assistance and local solidarity](#familycash), [Council and municipal relief policies](#welfare) |
| [tests/resources.rs](../tests/resources.rs) | [Canonical resource stocks and extraction](#resources), [Abandoned bulk-stock recovery](#recovery), [Persistent historical places and material evidence](#places) |
| [tests/shipping.rs](../tests/shipping.rs) | [Ports, harbors and sea trade](#shipping), [Vessels and named crew service](#vessels) |
| [tests/society.rs](../tests/society.rs) | [Aggregate demography, households and seasons](#society), [Household journeys and travel comparison](#relocation), [Household inheritance and unclaimed estates](#estates), [Progressive household cash tax](#wealthtax), [Social projections, pressures and local memory](#social), [Traveling danger warnings](#warnings), [Secular and religious relief journeys](#relief) |
| [tests/storage.rs](../tests/storage.rs) | [Production planning and essential maintenance](#production) |
| [tests/sunlight.rs](../tests/sunlight.rs) | [Seasonal sunlight](#sunlight) |
| [tests/waterworks.rs](../tests/waterworks.rs) | [Production planning and essential maintenance](#production) |

### Coverage and refresh procedure

This pass maps **138 main Rust files**, **15 shaders**, **8 editable/example assets**, **30 standalone Rust files**, **68 registry variants**, **62 evaluation/maintenance programs**, **one evidence CI workflow**, and **84 separate test/audit files**. All have an explicit mapping above; these are file-coverage counts, not a count of independently validated features.

When updating, pin a new committed revision, compare its `git ls-tree -r --name-only` file list and `System` enum against these tables, inspect changed implementation, and refresh both source and evidence scopes. Check links against the pinned revision. Use the actual scope when computing dates, for example:

```sh
git log 9d8d870 -1 --format="%cs %h %s" -- src/credit/servicing.rs src/credit/restructuring.rs
git log 9d8d870 -1 --format="%cs %h %s" -- docs/credit-restructuring.md docs/credit-implementation.md
python3 scripts/check_repository_artifacts.py
git diff --check
```

Before publishing, inspect the staged diff and stage only the intended documentation. Keep raw audits, generated manifests and experiment outputs under ignored `output/`. Record actual review/test dates separately; do not reinterpret old results using today’s defaults.

## Historical review records

The following records are preserved from the previous inventory. Their revision references, test counts, defaults and statements about what was pending apply to those historical passes, not necessarily the baseline above. No historical test result below was rerun for this inventory update.

## Five-system review pass (2026-09-12)

Starting revision: `fa57363`. These are explicit scoped reviews, distinct from
file-touch dates above. Each iteration records a decision and its evidence.

### 1. Heritage visits and pilgrimage routing

Reviewed `heritage_renown::destination` and `Culture::pilgrimage`. Both selected
the first direct route, so an expensive route could conceal a usable parallel
route; changing only destination selection would still fail during execution.
A shared `visit_distance` now quotes the cheapest finite, passable direct route in
both places. Actual travel work, food, offerings, custody and faith gates remain.
The pilgrimage and heritage GPU fixtures cover an expensive first route, a shorter
alternative, flooded alternatives, actual consumption, access and continuation.
No multi-hop travel or free destination service is implied.


Both focused GPU tests passed in this pass: pilgrimage execution (1 test) and
heritage study/access (1 test).

### 2. Lake runtime validation

Reviewed `Generator::equilibrate_lakes`. Public configuration can be changed
after construction; an invalid odd iteration ceiling bypassed constructor
validation and contradicted the solver's even scratch-buffer scheduling.
The solver now validates configuration before cache invalidation, initialization
or GPU dispatch. A focused GPU test passed: invalid iteration/poll settings leave
terrain byte-identical and progress unchanged, while valid exhausted budgets
still preserve water and can resume to convergence. Numerical rules and polling
defaults are unchanged.

### 3. Wildlife thermal feeding — retain current behavior

Reviewed shader thermal matching, prey-limited feeding, maintenance, trait
inheritance and the CPU/GPU fixture. Thermal matching scales feeding demand;
it does not add biomass or replace prey. The fixture independently calculates
feeding and maintenance, compares widths 15/30 and the disabled control, checks
budgets, and exercises saved continuation and removal/restoration. The focused GPU
fixture passed again during this review.

No further tuning is justified by this review. The previous nine matched runs
in [thermal ecotypes](wildlife-thermal-ecotypes.md) show sensitivity, not evidence
that the wider setting is better. Retain width 15 and the opt-in ecotype pilot.
Temperature remains a regional environmental proxy; this is not a depth-resolved
aquatic physiology model.

### 4. Resident agriculture allocation — retain corrected arithmetic

Reviewed eligibility, per-person reservation, the double-precision remaining
allowance, downward rounding into GPU work allowances, and stored-grant audits.
Actual commitments reduce the remaining budget; the GPU allowance cannot exceed
their sum. The existing 399-worker regression reproduces the old overgrant and
checks the fix; the 121-worker fixture distinguishes reduction roundoff from
real excess and rejects NaN. Both focused CPU regressions passed again.

No additional arithmetic or tolerance change is warranted. This review does not
establish fair allocation between sectors or stable population balance. The
previous corrected council comparisons demonstrate why long trajectories must
be rerun after even a small reservation change.

### 5. Council reporting: active versus retained arrears

Reviewed the council diagnostic producer and comparison renderer. Inactive
settlements retain administration counters, so the largest stored unpaid streak
is not necessarily current service distress. Reports now include active site IDs
and display both retained and active maxima. Old reports explicitly show
`unknown` for active arrears; they cannot be reconstructed from an active-town
count. Duplicate, invalid or out-of-range IDs are rejected.

Four Python fixtures passed (inactive exclusion, empty active set, legacy
metadata, invalid metadata). This is an observation change, not a policy change;
previous 200-year reports remain valid for their recorded funding totals and
stored counters, without retroactively claiming active-site coverage.

The one-year seed-17 smoke run at terrain 32/ecology 16 completed: all 16
active IDs matched the reported active-town count; stored and active arrears
were both 12 months. Rendering the prior corrected 200-year comparison succeeded
with active arrears explicitly unknown. The initial terrain-16 attempt failed
before history because it could not place 16 habitable founding sites; no claim
is made that that configuration supports this runner's founding count.

Implementation of iterations 1–2: `240ccf3`; council reporting: `d98b6b5`. This pass adds no new balance
parameters and does not repeat the earlier long ensembles.

Final verification: all 139 ordinary library tests passed (117 hardware tests
remain opt-in); the four focused GPU tests described above were run explicitly.
Both grant regressions are included in the ordinary suite. All-target Clippy
passed with warnings denied, as did the four report tests, source-artifact
policy and whitespace checks. Generated run data and logs remain ignored.

## Ten-system review pass (2026-09-12)

Fresh pass beginning at `e90e4b2`; the previous five iterations are not counted
again. Each entry below identifies a bounded review, not whole-system completion.
Verification results follow after execution.

1. **Council evidence validation.** Reviewed `compare_council_funding.arrears`.
   Active-site IDs were validated, but negative, fractional or nonfinite stored
   counters could still be rendered as evidence. Reject these, including counters
   belonging to inactive sites; months must be nonnegative integers. The CLI also
   handles missing indexed observations as input errors. No council policy changes.

2. **Matched food-access ensembles.** Reviewed `compare_food_access.compare`.
   Comparing two empty ensembles could succeed, and duplicate declared seeds
   disappeared through set conversion. Reject both and give an explicit error
   for runs without observations. These checks strengthen the evidence boundary;
   no food or demographic model changes.

3. **Finite extraction evidence.** Reviewed resource allowance settlement,
   depletion thresholds and archive validation. Add a fixture that consumes both
   pools while retaining a subgram ore remainder: depletion is evidence, not a
   stock deletion. Ore and clay generate distinct events; validation now rejects
   reuse of one event for both pools. This does not establish complete material
   provenance: event text still carries the material label, so typed identification
   of swapped historical references remains a possible future improvement.

4. **Reserved freight corridors — retain.** Reviewed `freight_path_flooded`,
   corridor normalization/capacity and the actual arrival fixture. Cargo retains
   its reserved edges rather than selecting an unreserved detour; flood delay
   retains goods and capacity and cannot repeat within one month. Existing
   serialized continuation checks cover resumed delivery. No new routing rule is
   warranted here. Missing historical edges still have no invented flood
   observation; closed-route embargo semantics are outside this flood review.

5. **Managed husbandry participation — retain.** Reviewed `farm_attendance`,
   feed delivery, collection and slaughter in `economy.wgsl`, plus the production
   attendance fixture. Granted farm attendance bounds managed activities while
   biological mortality continues without workers. Aggregate staffing is the
   explicit control. No extra worker pool or wage is introduced. This shares farm
   attendance across crops and animals; it is not a distinct husbandry occupation
   allocation, and that larger granularity change is not justified by this review.

6. **Learning projections and informal contact — retain.** Reviewed
   `culture/learning.rs`: lesson forecasts clone opening study state, paid
   execution advances actual progress, and informal exposure is capped across
   topics/channels once per month. Candidate selection can use opening knowledge,
   preventing newly acquired topics relaying within the same contact pass.
   Preserve these contracts. The pacing constants remain game choices; bounded
   transmission does not imply a realistic theory of education.

7. **Institutional operating work — retain.** Reviewed
   `reserve_institution_work` and `reserve_up_to`. Essential-first grants basic
   upkeep and administration before expanding repairs on the same commitment.
   Expansion rejects stale, settled, used, cancelled, absent or ineligible
   participants; minimum useful grants apply before creating commitments.
   Existing scarcity fixtures compare both policies with identical capacities and
   checkpoint settlement. No schedule or default-policy change warranted; this
   scoped policy does not solve sharing across all public services.

8. **Heritage visitation integration — retain.** Rechecked destination selection
   against actual pilgrimage execution after `240ccf3`. Both use the same
   cheapest passable direct route. Destination recognition still needs accessible
   custody, an ownership-connected present host and faith relevance; actual
   travel consumes work, food and offerings. Keep the implementation. A direct
   caravan quote is not a multi-hop itinerary or an independently simulated
   travelling population.

9. **Lake solver failure and restart — retain.** Reviewed preflight validation,
   even scratch-buffer batches, convergence flag readback, partial-state scatter
   and error handling. The conservative unfinished state is available for
   explicit restore/resume; exhausted budgets never report convergence.
   Keep current tolerance and poll defaults. A failed generator remains in its
   error state until explicit restoration; simply editing the limit is not a
   general recovery protocol. This review does not resolve synchronous UI waits.

10. **Wildlife thermal configuration and food limits — retain.** Reviewed
    configuration validation/defaults, uploaded thermal width, feeding demand,
    finite prey subtraction, assimilation and C/N/P-limited growth. Thermal
    preferences change opportunity, not nutrient stocks, and wider tolerance
    cannot replace absent prey. Retain the opt-in pilot and current width pending
    stronger balance evidence. Regional environmental temperature is still a
    proxy, not depth-specific lake temperature or a full species model.

### Verification of the ten iterations

Implementation changes: `fae9fb5`. No parameter tuning or production equations
changed in this pass.

| Iteration | Executed evidence | Result |
| --- | --- | --- |
| 1 | Five council-report unit tests, including corrupt inactive counters; rerendered the corrected 200-year comparison | Passed |
| 2 | Five food-access comparison tests, including empty/duplicate/no-observation cases; rerendered the same matched ensemble | Passed |
| 3 | Four resource fixtures: claimant order, finite settlement, subgram/separate-event evidence, regional control | Passed |
| 4 | GPU `junction_capacity_is_reserved_until_delivery_even_after_rerouting`, including flood hold, same-month delay guard and continuation; ordinary sea-approach topology fixture | Passed |
| 5 | GPU `agricultural_attendance_controls_cultivation_income_and_continuation`, including absent-worker and aggregate controls, herd products/feed/mortality and continuation | Passed |
| 6 | Four `culture::learning::tests` covering opening forecasts, partial work, encounter selection and bounded exposure | Passed |
| 7 | `essential_work_competes_with_repairs_without_extra_people_or_time`: seven capacities, both policies, no excess grants and saved settlement | Passed |
| 8 | Both GPU heritage-access and pilgrimage execution fixtures | Passed |
| 9 | GPU lake budget failure/preservation/resume fixture and ordinary configuration budget/poll tests | Passed |
| 10 | GPU thermal feeding/persistence fixture and ordinary thermal configuration/default test | Passed |

The full ordinary library suite passed **140 tests**, with 117 hardware tests
remaining opt-in; **six focused GPU tests** were explicitly executed above.
This is scoped verification, not a new multi-seed balance ensemble. Raw logs and
rendered comparisons are ignored under `output/review10-*`.
All-target Clippy with warnings denied, repository artifact policy and whitespace
checks also passed. Seven scoped reviews retained current behavior with the
limitations above; three produced implementation or validation improvements.

## Oldest-first five-system pass (2026-09-12)

Requested selection changed from recently touched systems to the longest
untouched ones. Selection uses the **latest source commit timestamp** across
each existing inventory row's source paths, ascending; names break timestamp
ties. Each selected row is excluded for the remainder of this pass even if the
review warrants no code change. Documentation-only edits do not change source
age. Git dates establish recorded file activity, not historical maintenance
before the repository snapshot.

At opening revision `b883380`, the resulting order is:

| Iteration | System | Latest source commit before review | Unix commit timestamp |
| --- | --- | --- | ---: |
| 1 | Sunlight and seasonal illumination | `7308c9f` | 1789011581 |
| 2 | Mineral/alloy processing and tool access | `3b9526a` | 1789067291 |
| 3 | Geology, regional terrain and hydrology | `367ff47` | 1789083519 |
| 4 | Social observations and memory | `853f2de` | 1789087542 |
| 5 | Living history and environment observations | `8e2d9c6` | 1789094488 |

The interrupted preceding pass left one resource-settlement improvement.
Its complete-result preflight and four invalid-batch atomicity cases passed
alongside the existing resource fixtures (five tests total), and it was committed
separately as `fa13d4a`. It is **not** counted among these five oldest systems.

### 1. Sunlight — retain

Read the complete shader and its ecology consumers: latitude is passed as
sin(latitude), tilt in radians, and month follows the existing ecological clock.
Polar night/day guards avoid invalid divisions and acos inputs. The GPU test
compared **7,236** samples against independent numerical integration of a rotating
surface at tilts 0°, 23.44° and 90°: maximum absolute error **0.000000548**
(tolerance 0.00002). Hemisphere reversal, zero-tilt invariance and approximately
constant spherical mean passed. No change justified. Equal-month representative
days and circular orbit remain intentional approximations.

### 2. Mineral/alloy processing and tool access — isolate controls

Reviewed ore identities, finite recovery/residue recipes, copper/tin/bronze scrap
chains, activation guards and restricted-tool custody. Alloy activation prepares
a cloned catalog before changing live state; occupied slots and unsupported
mineral identities are checked. Tool restriction is a one-time custody transfer,
retaining sub-ULP remainders rather than an ongoing production cap. No additional
mechanism change identified; cargo, residue, bronze and tool-custody fixtures
provide the scoped checks for this review.

The two alloy GPU fixtures initially failed: their supposed local-only negative
controls could import missing tin or mineral feedstock through open markets.
Closing markets explicitly in the fixture and asserting zero sales/purchases
restored both tests, including the three-seed residue/ruin/checkpoint cases.
The separate copper/tin cargo identity fixture preserves trade coverage.
Restricted-tool custody, unavailable production effects, invalid-input handling
and saved continuation also passed. Production/custody rules remain unchanged;
this iteration improves experimental isolation rather than disabling trade in
the game.

### 3. Regional terrain — geometric slope correction

Read generation, drainage/rank selection, flow, pool relaxation, habitat and
regional export attachment code. Habitat's slope calculation used one cell
width even for diagonal neighbors. This inflated diagonal gradients, thinning
derived soil and reducing plant cover. The new regression computes neighbor
distances from their 2-D coordinates and checks the resulting soil/cover while
retaining independent priority-flood, acyclic-route and runoff checks.

Also replaced the stale documentation claim of a fixed 4,096-pass planetary
lake budget with the current automatic/configurable budget and a link to polling
evidence. Regional fields remain derived snapshots; this change does not add
canonical resources or geological depth.

The regression failed against the original shader (cell 0 soil 0.95000774
versus geometric expectation 0.9526111), then passed after multiplying diagonal
runs by sqrt(2). This retains the existing area-derived square-cell width;
it is not a geodesically exact slope solver. Heights, drainage and physical
water are unchanged; derived soil and vegetation cover can intentionally differ.

### 4. Social observations and memory — respect evidence arrival

Reviewed the derived social projections, remembered pressure, monthly debounce,
local testimony and aid reciprocity. Social indicators retain aggregate population
authority; no demographic transition or scheduler change was needed.

Dated memory queries could previously use testimony or aid before its receipt.
They now return neutral influence until arrival. Equal-date reports select the
later causal event rather than the last caller, and replaying the same report
cannot postpone its arrival. Invalid dates, nonfinite/out-of-range food evidence,
negative/nonfinite aid and self-directed evidence are rejected before insertion.
These guards affect local evidence, not material inventories.

All four ordinary memory tests passed, including arrival boundaries, replay,
reversed report insertion, serialization and invalid inputs. The GPU social
projection/conservation/debounce fixture also passed. Memory remains a compact
latest-record summary: this does not reconstruct overwritten earlier testimony
or introduce a credibility or social-network model.

### 5. Living history and environment observations — retain

Reviewed CPU consumer selection, cache invalidation, the 176-byte bitwise gather
ABI, GPU limits and dated observation access. Sites, prospective settlements,
neighbors and resource/discovery locations refresh explicitly. GPU navigation
reads live terrain; CPU reference navigation additionally refreshes route cells
and performs full refreshes for annual searches/new settlements. Buffer/epoch
changes, restored state and explicit full mode invalidate or refresh the cache.
No additional change justified by this review.

The seam/restored-buffer GPU gather fixture passed. The private dense CPU cache
is only partially refreshed and must not be presented as a complete world
snapshot. Its transfer statistics cover terrain observations, not ecology,
archive serialization or all GPU readbacks. Matching current consumer fixtures
cannot prove a future consumer safely reads an unlisted cell; additions must
extend observed-cell selection and differential tests together.

### Completion evidence

All five selected rows were reviewed in source-age order. Three yielded changes
(regional slope, memory boundaries, and isolated alloy controls); two retained
current behavior with the limitations above.

| Check | Result |
| --- | --- |
| Sunlight independent quadrature | 1 GPU fixture passed; 7,236 samples |
| Alloy processing | 2 GPU fixtures passed after closing fixture markets |
| Restricted-tool custody and continuation | 1 GPU fixture passed |
| Regional geometry, drainage and runoff | 1 GPU fixture passed; new assertion first failed on old shader |
| Social projections and event debounce | 1 GPU fixture passed |
| Gather ABI, seams and restored terrain | 1 GPU fixture passed |
| Environment readback comparisons | 3 GPU fixtures passed, including seeds 17/81/256 at terrain 64 and ecology 16, frozen batch/checkpoint equivalence and due-food-cargo timing |
| Ordinary library suite | 143 passed; 117 hardware fixtures remain opt-in |

Ten targeted GPU fixtures ran explicitly; the ordinary test count does not imply
execution of the entire ignored suite. The library run includes copper/tin cargo
identity, social pressure recovery and stale-observation rejection. This is a
maintenance verification pass, not a long-run balance or cross-hardware study.
Logs remain ignored under `output/oldest-*`. All-target Clippy initially caught
an iterator-style issue in the carried resource preflight; the final verification
also checks that correction against the five resource fixtures.

Final resource rerun: **5 passed**. All-target Clippy with warnings denied,
repository artifact policy and whitespace checks passed.

## New oldest-first ten-system batch (opening `19d5491`)

This is a new batch, not an extension of the completed five reviews above.
Source timestamps determine order; a reviewed row is excluded for the remainder
of this batch even when retained unchanged. Thus unchanged systems can recur
across batches. The first ten at the opening revision are:

| # | System | Opening source revision |
| --- | --- | --- |
| 1 | Sunlight and seasonal illumination | `7308c9f` |
| 2 | Mineral/alloy processing and tool access | `3b9526a` |
| 3 | Living history and environment observations | `8e2d9c6` |
| 4 | Historical places, objects and canonical recovery | `b09e326` |
| 5 | Expeditions, automatic missions and named crews | `e54dd81` |
| 6 | Navigation, spatial features and territory | `57de102` |
| 7 | Discoveries, specimen research and applications | `4dff636` |
| 8 | Local offices and completed public service | `bd02945` |
| 9 | Institution succession and mandates | `cbf552e` |
| 10 | Housing, storage, waterworks and recovery | `4727d02` |

### Reviews and decisions

1. **Sunlight — retain.** Re-read the short shader and checked unchanged consumer
   units/clock contracts against the preceding review. Hemisphere phase, polar
   guards and circular-orbit daily integration remain appropriate for the toy.
   Re-ran the independent GPU quadrature fixture; no new change justified.
2. **Alloys/tools — retain.** Re-read activation, catalog preparation, stable ore
   identity, residue capacity and boundary-only restricted custody. The repaired
   local-only alloy fixtures passed again. No new physical rule justified;
   restricted custody remains an intervention, not an ongoing import cap.
3. **Environment observations — retain.** Rechecked the same source revision's
   month guard, consumer set and cache invalidation against the previous audit.
   Re-ran full/gathered and continuation comparisons. The private partial cache
   still must not be exposed as a full terrain snapshot.
4. **Local places — retain.** Traced recovery request validation into cultural
   plan validation and execution. Survey seed/clocks/object contents, local
   residence, unique pending object/site and current survivability are checked.
   Unfunded requests wait; unavailable residents/objects cancel. Successful work
   changes custody of the canonical object without clearing ownership claims or
   minting materials. This remains parent-cell co-location, not local travel.
5. **Expeditions — correct unrelated skill transfer.** Recruitment preserved
   specialties but return assigned every survivor's competence to craftsmanship,
   which fed later craft and institution decisions. Limit civilian transfer to
   engineering/craft and navigation/survey counterparts after fieldwork; retain
   all other role experience in existing personal voyage records. Initial civilian
   preparation uses the same mapping. Stores, crew identities and veteran-role
   continuity remain on the existing paths. Old skills are not rewritten.
6. **Navigation/spatial/territory — retain.** Reviewed frontier convergence and
   reconstruction guards, island labeling, route-query shape checks, cell/world
   references and dated territorial revisions. No route is returned as success
   when convergence fails. Same-month territory recording replaces only that
   boundary; overlaps and older control are retained. Spatial features are
   references/coverage, not new claims or physical stocks. Rounded path costs
   may differ from CPU searches; verification must check traversability as well
   as cost, not require path identity.
7. **Discoveries — preserve care during research closure.** Policy prose promises
   stored remedies remain usable while research is paused, but execution skipped
   treatment along with processing. Separate existing-stock treatment from
   research execution. Keep expiration and consumption in the C/N/P ledger;
   abandoned sites still receive no treatment. Teacher contact is a method-copying
   opportunity, not a newly funded teacher service or loss of known methods on
   workshop closure.
8. **Offices — retain.** Reviewed holder eligibility, controller changes, terms,
   quarterly selection and dated named-service reservations. Delivered capacity
   requires the same current holder/controller and completed work. Invalid
   settlement batches preflight before spending personal commitments; released
   time cannot be reused in already-completed GPU production. The unnamed-staff
   floor and suitability scores remain explicit toy rules.
9. **Institution mandates — retain.** Reviewed present-member eligibility,
   religious affiliation, vacancy, quarterly ballots, minimum useful convening
   work and causal events. Membership alone does not confer local authority;
   stale or insufficient grants do not elect a leader. Ties use stable IDs and
   a divided ballot requires another paid deliberation. This is not a detailed
   electoral process; the second deliberation can select a plurality winner.
10. **Town assets/production — retain.** Reviewed finite baseline allowances,
    incoming cargo versus physical inventories, funded contract forecasts,
    upstream recipe recursion guards, material substitution and migration-aware
    housing requests. Existing structures retain condition and historical capacity;
    event thresholds suppress minor monthly changes. Procurement is a forecast,
    not a promise of this month's production. No new asset rule justified by
    this review; targeted persistence and water-service checks follow below.

### Verification

Fresh checks use the available Vulkan GPU, with generated files and raw logs under
ignored `output/oldest10-*`. They are controlled correctness/integration fixtures,
not a new long-run balance ensemble or a cross-hardware portability claim.

| Review | Fresh targeted result |
| --- | --- |
| Sunlight | 1 GPU quadrature fixture passed |
| Alloys/tools | 2 GPU alloy fixtures passed; custody review uses prior fixture evidence |
| Living environment | 3 GPU gathered/full and continuation fixtures passed |
| Canonical local recovery | 1 GPU paid recovery/continuation fixture passed |
| Expeditions | All 6 GPU cases passed across the funded suite and final rescue/recall rerun; 1 new ordinary skill-transfer fixture passed |
| Navigation and territory | 1 three-seed GPU route comparison and 1 territorial-history fixture passed |
| Discoveries | 1 GPU research/treatment fixture passed, including paused-workshop and abandoned-site controls |
| Offices | 3 GPU selection/service/continuation fixtures passed |
| Institution mandates | 1 GPU vacancy, funding and continuation fixture passed |
| Town assets | 1 housing, 1 storage and 2 waterworks GPU fixtures passed |

All 24 targeted GPU fixtures passed across the documented runs. The ordinary
library suite passed **144 tests**, with **117 hardware/explicit fixtures ignored**
by that command; the targeted runs above execute the relevant ignored cases.
All-target Clippy with warnings denied passed. Repository artifact policy and
whitespace checks passed before the final documentation commit. The two behavior changes are
`f5823e9` (stored remedies during research closure) and `78f8c7e`
(role-specific expedition skill transfer).

The initial expedition GPU run failed all six fixtures before crew assignment:
its twenty-year "prosperous" setup lacked tools and/or sponsor cash under current
balance. This is not evidence that the skill change broke travel. The fixture now
explicitly imports 200 kg ordinary tools and 5,000 currency per town, recording
initial stocks, cash and C/N/P, and validates the world before launch. Actual
harbor, crew and civilian-reserve checks remain active. These tests establish
voyage accounting under funded conditions, not natural expedition affordability.
The previously repaired alloy fixtures similarly keep their local markets closed.

The funded expedition suite then passed five cases; the sixth completed its rescue
but lacked food and sponsor capital for a third voyage after another five years.
That recall subcase now transfers existing neighboring food and town cash to its
origin/council before launch. It passes without weakening launch requirements;
the subsequent history validation still checks conservation. This fixture setup
does not establish that towns naturally replenish repeated expedition costs.