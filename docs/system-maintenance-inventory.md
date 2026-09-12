# Existing systems and last recorded work

Inventory date: **2026-09-12**. Source inventory baseline: `fae9fb5`.

This is a maintenance inventory of implemented systems, grouped by responsibility.
It includes the startup switches in [system options](system-options.md) and broader
library subsystems. Related options share rows; archived proposals are excluded.


## How to read the dates

- **Source work** is the latest reachable commit touching any listed source path (including children of listed directories). Inline tests count as source work.
- **Documentation / evidence** is the latest reachable commit touching the linked guides or reports. It records a document update, not necessarily a new experiment or formal review. Read the report for the actual tested revision and scope.
- Dates use Git committer dates (`%cs`), with short commit IDs for inspection. A file-level change can affect only part of a grouped system; shared files deliberately produce shared dates.
- The retained history begins with source snapshot `f431224` on **2026-09-10**. A snapshot-only entry means “present by this date”; its earlier last-work/review date is unknown.
- Formal last-review dates are not consistently recorded. This inventory does not infer review, correctness, balance or completion from a recent commit. See the [integration worklist](integration-worklist.md) for remaining gaps.
- Uncommitted review changes have no commit date and are described separately below. Creating this inventory is not a technical review of each system.


## World and environment

| System and source scope | Last source work | Last documentation / evidence update | Guides and reports |
| --- | --- | --- | --- |
| **World generation, GPU execution and spherical grid**<br>[gpu.rs](../src/gpu.rs), [grid.rs](../src/grid.rs), [shaders/simulation.wgsl](../shaders/simulation.wgsl) | 2026-09-12 · `240ccf3` — Use usable heritage routes and validate lake settings before dispatch | 2026-09-12 · `cce0f34` — Optimize lake relaxation and handle slow default-resolution basins | [default-generation-performance](default-generation-performance.md), [benchmarks](benchmarks.md) |
| **Geology, regional terrain and hydrology**<br>[region.rs](../src/region.rs), [shaders/regional.wgsl](../shaders/regional.wgsl), [shaders/region_compute.wgsl](../shaders/region_compute.wgsl) | 2026-09-10 · `367ff47` — Connect resource and survey spatial identities and show expedition markers | 2026-09-10 · `a256f21` — Generate setting-driven geological regions and clarify rock maps | [regions](regions.md), [geological-regions](geological-regions.md), [geological-provinces](geological-provinces.md), [stratigraphic-columns](stratigraphic-columns.md) |
| **Ecology, food webs and geochemical habitats**<br>[ecology.rs](../src/ecology.rs), [shaders/ecology.wgsl](../shaders/ecology.wgsl) | 2026-09-12 · `3d244c8` — Add measured aquatic thermal and lake polling sensitivity controls | 2026-09-12 · `f68a1ae` — Add opt-in regional wildlife thermal preferences and migration inheritance | [ecology](ecology.md), [food-webs](food-webs.md), [geochemical-habitats](geochemical-habitats.md) |
| **Sunlight and seasonal illumination**<br>[shaders/sunlight.wgsl](../shaders/sunlight.wgsl) | 2026-09-10 · `7308c9f` — Correct ecological sunlight for hemisphere, axial tilt and day length | 2026-09-10 · `7308c9f` — Correct ecological sunlight for hemisphere, axial tilt and day length | [ecological-sunlight](ecological-sunlight.md) |
| **Wildlife assembly, trophic balance and thermal ecotypes**<br>[ecology.rs](../src/ecology.rs), [shaders/ecology.wgsl](../shaders/ecology.wgsl) | 2026-09-12 · `3d244c8` — Add measured aquatic thermal and lake polling sensitivity controls | 2026-09-12 · `3d244c8` — Add measured aquatic thermal and lake polling sensitivity controls | [wildlife-assembly](wildlife-assembly.md), [wildlife-trophic-stability](wildlife-trophic-stability.md), [wildlife-thermal-ecotypes](wildlife-thermal-ecotypes.md) |
| **Living history and environment observations**<br>[history_environment.rs](../src/history_environment.rs), [shaders/history_environment.wgsl](../shaders/history_environment.wgsl) | 2026-09-10 · `8e2d9c6` — Move history surveys and terrain route searches onto GPU | 2026-09-10 · `8e2d9c6` — Move history surveys and terrain route searches onto GPU | [living-history](living-history.md), [living-history-results](living-history-results.md), [history-environment-readback](history-environment-readback.md) |
| **Flood hazards and delayed cargo spoilage**<br>[hazards.rs](../src/hazards.rs) | 2026-09-12 · `e2f89cc` — Reserve shared road corridors for networked market cargo | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification | [flood-history](flood-history.md), [flood-seed-audit](flood-seed-audit.md), [delayed-cargo-spoilage](delayed-cargo-spoilage.md) |
| **Finite resources, regional mining and depletion evidence**<br>[resources.rs](../src/resources.rs), [regional_mining.rs](../src/regional_mining.rs) | 2026-09-12 · `fae9fb5` — Validate depletion evidence and reject malformed balance ensembles | 2026-09-12 · `dd2350b` — Preserve canonical source depletion as historical evidence | [shared-resources](shared-resources.md), [regional-mining-control](regional-mining-control.md), [source-depletion-evidence](source-depletion-evidence.md) |
| **Environmental returns and abandoned land recovery**<br>[environmental_returns.rs](../src/environmental_returns.rs), [shaders/managed_returns.wgsl](../shaders/managed_returns.wgsl) | 2026-09-12 · `f68a1ae` — Add opt-in regional wildlife thermal preferences and migration inheritance | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification | [environmental-returns](environmental-returns.md), [cross-scale-coupling](cross-scale-coupling.md) |

## History and population

| System and source scope | Last source work | Last documentation / evidence update | Guides and reports |
| --- | --- | --- | --- |
| **Monthly coordinator, founding and settlement lifecycle**<br>[civilization.rs](../src/civilization.rs), [civilization](../src/civilization) | 2026-09-12 · `60bb5dc` — Require granted farm attendance for managed herd work | 2026-09-12 · `3bd13d2` — Bound emergency town support by actual working-cash gaps | [monthly-schedule](monthly-schedule.md), [civilizations](civilizations.md), [settlement-lifecycle](settlement-lifecycle.md), [resident-daughter-founding](resident-daughter-founding.md) |
| **Society, seasons, routes and household stocks**<br>[society.rs](../src/society.rs), [shaders/society.wgsl](../shaders/society.wgsl) | 2026-09-12 · `3bd13d2` — Bound emergency town support by actual working-cash gaps | 2026-09-11 · `57de102` — Standardize great-lake landmass terminology as inner continents | [society](society.md), [social-indicators](social-indicators.md) |
| **Resident identity, reconciliation and individual demography**<br>[population_registry.rs](../src/population_registry.rs), [individual_demography.rs](../src/individual_demography.rs) | 2026-09-12 · `9bd4416` — Reduce repeated history membership and care scans with profiled verification | 2026-09-11 · `000507d` — Refresh history documentation status and navigation | [resident-rosters](resident-rosters.md), [population-reconciliation](population-reconciliation.md), [individual-demography](individual-demography.md), [individual-demography-verification](individual-demography-verification.md) |
| **Aggregate/individual resolution and comparison receipts**<br>[resolution.rs](../src/resolution.rs) | 2026-09-12 · `320ba53` — Compare projected army supply losses with realized campaign outcomes | 2026-09-11 · `384607d` — Staff merchant vessels with bounded named participation and household wages | [resolution-framework](resolution-framework.md), [resolution-framework-verification](resolution-framework-verification.md) |
| **Personal participation, commitments and labor**<br>[participation.rs](../src/participation.rs), [labor.rs](../src/labor.rs) | 2026-09-12 · `9bd4416` — Reduce repeated history membership and care scans with profiled verification | 2026-09-11 · `000507d` — Refresh history documentation status and navigation | [individual-participation](individual-participation.md), [work-execution-boundaries](work-execution-boundaries.md), [committed-crew-reservations](committed-crew-reservations.md) |
| **Domestic groups, care and kin assistance**<br>[domestic.rs](../src/domestic.rs), [domestic](../src/domestic), [kin_support.rs](../src/kin_support.rs) | 2026-09-12 · `9bd4416` — Reduce repeated history membership and care scans with profiled verification | 2026-09-12 · `0450bd6` — Allow spare family care capacity to support neighboring dependents | [domestic-participation](domestic-participation.md), [care-resolution](care-resolution.md), [neighbor-care](neighbor-care.md), [domestic-surplus-assistance](domestic-surplus-assistance.md) |
| **Household relocation and travel outcomes**<br>[relocation.rs](../src/relocation.rs), [relocation](../src/relocation) | 2026-09-12 · `c306b12` — Report conditional and realized relocation travel attrition | 2026-09-12 · `c306b12` — Report conditional and realized relocation travel attrition | [household-relocation](household-relocation.md), [relocation-travel-comparison](relocation-travel-comparison.md) |
| **Household income, nutrition and family cash support**<br>[household_economy.rs](../src/household_economy.rs), [household_economy/nutrition.rs](../src/household_economy/nutrition.rs), [household_economy/family_support.rs](../src/household_economy/family_support.rs) | 2026-09-12 · `a5d0b22` — Add scoped administration allowance and council funding evidence | 2026-09-12 · `7e2402b` — Report matched century effects of political food distribution | [household-economy](household-economy.md), [nutrition-age-balance](nutrition-age-balance.md), [household-family-support](household-family-support.md), [family-support-century](family-support-century.md) |
| **Council assistance, distribution and town support**<br>[household_economy/council_allocation.rs](../src/household_economy/council_allocation.rs), [household_economy/policy.rs](../src/household_economy/policy.rs), [society.rs](../src/society.rs) | 2026-09-12 · `3bd13d2` — Bound emergency town support by actual working-cash gaps | 2026-09-12 · `fa57363` — Record completed integrated follow-up and corrected stress ensemble | [council-administration-allowance](council-administration-allowance.md), [council-funding-balance](council-funding-balance.md), [council-stress-balance](council-stress-balance.md), [household-distribution-politics](household-distribution-politics.md), [distribution-policy-century](distribution-policy-century.md), [household-relief-balance](household-relief-balance.md) |
| **Social observations and memory**<br>[social_state.rs](../src/social_state.rs), [social_memory.rs](../src/social_memory.rs) | 2026-09-10 · `853f2de` — Connect local aid memory, staffed fleets and institutional research | 2026-09-10 · `853f2de` — Connect local aid memory, staffed fleets and institutional research | [social-indicators](social-indicators.md), [witnessed-relief](witnessed-relief.md) |

## Production and economy

| System and source scope | Last source work | Last documentation / evidence update | Guides and reports |
| --- | --- | --- | --- |
| **Farming, diversified crops and husbandry attendance**<br>[agriculture.rs](../src/agriculture.rs), [agriculture_participation.rs](../src/agriculture_participation.rs), [shaders/economy.wgsl](../shaders/economy.wgsl) | 2026-09-12 · `00f1fa9` — Bound accumulated resident work grants after long-run over-allocation | 2026-09-12 · `dd2350b` — Preserve canonical source depletion as historical evidence | [economy](economy.md), [crop-price-revisit](crop-price-revisit.md), [production-participation](production-participation.md), [husbandry-attendance](husbandry-attendance.md) |
| **Fisheries, gear, adaptive staffing and opportunity costs**<br>[agriculture.rs](../src/agriculture.rs), [economy.rs](../src/economy.rs), [shaders/economy.wgsl](../shaders/economy.wgsl) | 2026-09-12 · `1823558` — Hold cargo on flooded reserved road corridors | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification | [adaptive-fisheries](adaptive-fisheries.md), [timber-fisheries](timber-fisheries.md), [fishery-opportunity-cost](fishery-opportunity-cost.md), [natural-fishery-trajectories](natural-fishery-trajectories.md) |
| **Markets, adaptive prices and network trade**<br>[economy.rs](../src/economy.rs), [shaders/economy.wgsl](../shaders/economy.wgsl) | 2026-09-12 · `1823558` — Hold cargo on flooded reserved road corridors | 2026-09-11 · `57de102` — Standardize great-lake landmass terminology as inner continents | [economy](economy.md), [crop-and-price-experiments](crop-and-price-experiments.md), [demand-economy](demand-economy.md) |
| **Production, adaptive labor and food-security maintenance**<br>[production.rs](../src/production.rs) | 2026-09-11 · `4727d02` — Preserve institutional fee reserves during stocked-material repairs | 2026-09-11 · `c4a3449` — Record held-out construction decline and verified learning continuation | [adaptive-industries](adaptive-industries.md), [food-security-labor](food-security-labor.md), [demand-economy](demand-economy.md), [production-participation](production-participation.md) |
| **Workshops, operators, capital and occupational payroll**<br>[enterprises.rs](../src/enterprises.rs), [workshop_resolution.rs](../src/workshop_resolution.rs), [production.rs](../src/production.rs) | 2026-09-11 · `4727d02` — Preserve institutional fee reserves during stocked-material repairs | 2026-09-11 · `db6d2cb` — Connect agricultural attendance to cultivation and household earnings | [workshop-operators](workshop-operators.md), [workshop-operator-calibration](workshop-operator-calibration.md), [workshop-capital](workshop-capital.md), [specialized-workshops](specialized-workshops.md), [resident-payroll-balance](resident-payroll-balance.md) |
| **Mineral/alloy processing and tool access**<br>[metallurgy.rs](../src/metallurgy.rs), [tool_access.rs](../src/tool_access.rs) | 2026-09-10 · `3b9526a` — Add material-specific objects, task tools, and expandable institutions | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification | [mineral-processing](mineral-processing.md), [alloy-processing](alloy-processing.md), [toolmaking-capability](toolmaking-capability.md), [tool-buffering-evidence](tool-buffering-evidence.md) |
| **Housing, storage, waterworks and recovery**<br>[production.rs](../src/production.rs) | 2026-09-11 · `4727d02` — Preserve institutional fee reserves during stocked-material repairs | 2026-09-11 · `2be3f62` — Preserve ruler identity during displaced succession and record balance failures | [site-assets](site-assets.md), [estate-vacancies](estate-vacancies.md), [waterworks-recovery](waterworks-recovery.md), [essential-service-recovery](essential-service-recovery.md) |
| **Materials and institutional facilities**<br>[materials.rs](../src/materials.rs), [facilities.rs](../src/facilities.rs) | 2026-09-11 · `4727d02` — Preserve institutional fee reserves during stocked-material repairs | 2026-09-10 · `1701d16` — Tie facility investment and upkeep to component durability | [material-objects-and-facilities](material-objects-and-facilities.md) |

## Transport and conflict

| System and source scope | Last source work | Last documentation / evidence update | Guides and reports |
| --- | --- | --- | --- |
| **Navigation, spatial features and territory**<br>[navigation.rs](../src/navigation.rs), [spatial.rs](../src/spatial.rs), [territory.rs](../src/territory.rs), [shaders/navigation.wgsl](../shaders/navigation.wgsl), [shaders/navigation_survey.wgsl](../shaders/navigation_survey.wgsl), [shaders/navigation_inspect.wgsl](../shaders/navigation_inspect.wgsl) | 2026-09-11 · `57de102` — Standardize great-lake landmass terminology as inner continents | 2026-09-11 · `57de102` — Standardize great-lake landmass terminology as inner continents | [gpu-navigation](gpu-navigation.md), [spatial-features](spatial-features.md), [transport-and-cultural-contact](transport-and-cultural-contact.md) |
| **Shipping, vessels, harbor work and merchant crews**<br>[shipping.rs](../src/shipping.rs), [vessels.rs](../src/vessels.rs), [vessels](../src/vessels) | 2026-09-12 · `e2f89cc` — Reserve shared road corridors for networked market cargo | 2026-09-11 · `e41c785` — Compare crew reservations and record common-food century outcomes | [shipping](shipping.md), [harbor-work](harbor-work.md), [merchant-crew-participation](merchant-crew-participation.md), [inland-sea-freight](inland-sea-freight.md) |
| **Land freight, road capacity and upkeep**<br>[freight.rs](../src/freight.rs), [road_upkeep.rs](../src/road_upkeep.rs) | 2026-09-12 · `1823558` — Hold cargo on flooded reserved road corridors | 2026-09-12 · `1823558` — Hold cargo on flooded reserved road corridors | [land-freight-reservations](land-freight-reservations.md), [intermediate-freight](intermediate-freight.md), [road-freight-capacity](road-freight-capacity.md), [road-upkeep](road-upkeep.md) |
| **Export contracts and supplier profitability**<br>[export_contracts.rs](../src/export_contracts.rs) | 2026-09-12 · `e2f89cc` — Reserve shared road corridors for networked market cargo | 2026-09-11 · `57de102` — Standardize great-lake landmass terminology as inner continents | [export-contracts](export-contracts.md), [supplier-quotes](supplier-quotes.md) |
| **Household and religious relief logistics**<br>[relief.rs](../src/relief.rs), [religious_relief.rs](../src/religious_relief.rs) | 2026-09-12 · `e2f89cc` — Reserve shared road corridors for networked market cargo | 2026-09-12 · `62f5254` — Bound religious relief by shared endpoint freight capacity | [religious-relief](religious-relief.md), [relief-freight-capacity](relief-freight-capacity.md), [witnessed-relief](witnessed-relief.md) |
| **Military participation, supply and finite occupation**<br>[military.rs](../src/military.rs), [military_supply.rs](../src/military_supply.rs), [occupation.rs](../src/occupation.rs) | 2026-09-12 · `320ba53` — Compare projected army supply losses with realized campaign outcomes | 2026-09-12 · `320ba53` — Compare projected army supply losses with realized campaign outcomes | [individual-military-verification](individual-military-verification.md), [military-supply-comparison](military-supply-comparison.md), [finite-occupation](finite-occupation.md) |

## Government, culture and knowledge

| System and source scope | Last source work | Last documentation / evidence update | Guides and reports |
| --- | --- | --- | --- |
| **Politics, factions and recent trade interests**<br>[politics.rs](../src/politics.rs), [faction_interests.rs](../src/faction_interests.rs), [trade_contact.rs](../src/trade_contact.rs) | 2026-09-12 · `5eb23ba` — Let councils revise household distribution policies through politics | 2026-09-12 · `5eb23ba` — Let councils revise household distribution policies through politics | [politics](politics.md), [faction-interests](faction-interests.md), [recent-trade-contact](recent-trade-contact.md) |
| **Governance, diplomacy, autonomy and crisis response**<br>[governance.rs](../src/governance.rs) | 2026-09-12 · `a5d0b22` — Add scoped administration allowance and council funding evidence | 2026-09-10 · `7c2b516` — Connect institutional petitions with politics, public finance and cultural memory | [governance](governance.md), [governance-pressure](governance-pressure.md), [council-crisis-response](council-crisis-response.md) |
| **Local offices and completed public service**<br>[offices.rs](../src/offices.rs), [offices](../src/offices) | 2026-09-11 · `bd02945` — Record conditional office service resolution receipts | 2026-09-11 · `2f81e43` — Compare stable and rotating institutional work priorities | [local-offices](local-offices.md), [governance-duty-audit](governance-duty-audit.md), [shared-council-payroll](shared-council-payroll.md) |
| **Civic petitions and hearings**<br>[civic_petitions.rs](../src/civic_petitions.rs), [civic_petitions](../src/civic_petitions) | 2026-09-12 · `73742c7` — Reserve institutional hearing parties and shared service capacity | 2026-09-11 · `538bc70` — Connect faction support to household access and government accountability | [civic-petitions](civic-petitions.md), [civic-petition-causality](civic-petition-causality.md), [civic-petitions-balance](civic-petitions-balance.md) |
| **Culture, religious practices and household affiliation**<br>[culture.rs](../src/culture.rs), [culture/practices.rs](../src/culture/practices.rs), [culture/dynamics.rs](../src/culture/dynamics.rs) | 2026-09-12 · `240ccf3` — Use usable heritage routes and validate lake settings before dispatch | 2026-09-12 · `9016eb4` — Resolve adult family cultural affiliation through household membership | [religious-pluralism](religious-pluralism.md), [religious-dynamics](religious-dynamics.md), [household-cultural-affiliation](household-cultural-affiliation.md) |
| **Research/culture allocation and dated work plans**<br>[service_allocation.rs](../src/service_allocation.rs), [culture/work_requests.rs](../src/culture/work_requests.rs), [learning_resolution.rs](../src/learning_resolution.rs) | 2026-09-12 · `1360fff` — Add bounded continuing-study selection and acquisition diagnostics | 2026-09-12 · `38c0f7e` — Filter room-denied cultural demand before shared work allocation | [service-allocation](service-allocation.md), [research-cultural-work-requests](research-cultural-work-requests.md), [learning-resolution](learning-resolution.md) |
| **Institution capacity, funding, duties and room allocation**<br>[institution_capacity.rs](../src/institution_capacity.rs), [institution_funding.rs](../src/institution_funding.rs), [institution_services.rs](../src/institution_services.rs) | 2026-09-12 · `33ba2db` — Fund heritage interpretation from institutional treasuries | 2026-09-12 · `a069655` — Separate institutional operating space from membership expansion demand | [institution-capacity](institution-capacity.md), [institution-operating-budgets](institution-operating-budgets.md), [institution-service-space-review](institution-service-space-review.md), [institution-service-balance](institution-service-balance.md), [institution-working-core](institution-working-core.md), [institution-allocation-balance](institution-allocation-balance.md) |
| **Institution succession and mandates**<br>[institution_succession.rs](../src/institution_succession.rs) | 2026-09-11 · `cbf552e` — Leave infeasible election grants available for useful work | 2026-09-10 · `f431224` — Ancient World: source snapshot with summarized verification | [institutional-succession](institutional-succession.md) |
| **Learning, successor teaching and continuing study**<br>[culture/learning.rs](../src/culture/learning.rs) | 2026-09-12 · `0a6a03d` — Find eligible informal learners and preserve contact boundaries | 2026-09-12 · `32afbd9` — Record matched institutional study completion results | [knowledge-continuity](knowledge-continuity.md), [knowledge-succession](knowledge-succession.md), [informal-learning-selection](informal-learning-selection.md), [institution-student-selection](institution-student-selection.md), [institution-continuing-study](institution-continuing-study.md) |
| **Expeditions, automatic missions and named crews**<br>[expeditions.rs](../src/expeditions.rs) | 2026-09-11 · `e54dd81` — Connect witnessed heritage recoveries to local renown and pilgrimage | 2026-09-11 · `57de102` — Standardize great-lake landmass terminology as inner continents | [expeditions](expeditions.md), [expedition-results](expedition-results.md), [expedition-crews](expedition-crews.md) |
| **Discoveries, specimen research and applications**<br>[discoveries.rs](../src/discoveries.rs), [discoveries](../src/discoveries) | 2026-09-11 · `4dff636` — Compare botanical study and application inputs at monthly resolution boundaries | 2026-09-11 · `5adb2f0` — Fix discovery study precision deadlock and restore voyage integration tests | [discoveries](discoveries.md), [discovery-results](discovery-results.md), [expedition-returns](expedition-returns.md) |
| **Heritage expeditions, renown, stewardship and study funding**<br>[expedition_heritage.rs](../src/expedition_heritage.rs), [heritage_renown.rs](../src/heritage_renown.rs) | 2026-09-12 · `240ccf3` — Use usable heritage routes and validate lake settings before dispatch | 2026-09-12 · `08453d9` — Require ownership-connected present hosts for heritage visits | [heritage-expeditions](heritage-expeditions.md), [heritage-renown](heritage-renown.md), [heritage-stewardship](heritage-stewardship.md), [heritage-study-funding](heritage-study-funding.md) |
| **Historical places, objects and canonical recovery**<br>[local_places.rs](../src/local_places.rs) | 2026-09-11 · `b09e326` — Add shared individual participation for cultural and research work | 2026-09-12 · `dd2350b` — Preserve canonical source depletion as historical evidence | [regional-historical-places](regional-historical-places.md), [source-depletion-evidence](source-depletion-evidence.md) |
| **Names, languages and vocabulary evolution**<br>[naming.rs](../src/naming.rs), [naming](../src/naming) | 2026-09-12 · `e2f89cc` — Reserve shared road corridors for networked market cargo | 2026-09-12 · `f181964` — Broaden patron and shared naming vocabularies and conventions | [naming-languages](naming-languages.md), [lexicon-evolution](lexicon-evolution.md) |

## Application and development

| System and source scope | Last source work | Last documentation / evidence update | Guides and reports |
| --- | --- | --- | --- |
| **Explorer, atlas and history timeline**<br>[viewer.rs](../src/viewer.rs), [history_atlas.rs](../src/history_atlas.rs), [history_timeline.rs](../src/history_timeline.rs), [shaders/view.wgsl](../shaders/view.wgsl) | 2026-09-12 · `3bd13d2` — Bound emergency town support by actual working-cash gaps | 2026-09-10 · `2760bf1` — Add recorded settlement timeline with causal event navigation | [history-timeline](history-timeline.md) |
| **Configuration, catalogs, startup options and CLI**<br>[config.rs](../src/config.rs), [catalog.rs](../src/catalog.rs), [systems.rs](../src/systems.rs), [main.rs](../src/main.rs) | 2026-09-12 · `3d244c8` — Add measured aquatic thermal and lake polling sensitivity controls | 2026-09-10 · `7e0e655` — Enable optional systems by default for new application histories with explicit overrides | [system-options](system-options.md) |
| **Archives and checkpoint persistence**<br>[storage.rs](../src/storage.rs) | 2026-09-12 · `f68a1ae` — Add opt-in regional wildlife thermal preferences and migration inheritance | 2026-09-11 · `384607d` — Staff merchant vessels with bounded named participation and household wages | [living-history](living-history.md), [resolution-framework](resolution-framework.md) |
| **Integrated evaluation, balance and evidence tooling**<br>[scripts/run_integration_balance.py](../scripts/run_integration_balance.py), [scripts/integrated_history.py](../scripts/integrated_history.py), [scripts/compare_integrated_history.py](../scripts/compare_integrated_history.py), [scripts/check_repository_artifacts.py](../scripts/check_repository_artifacts.py), [scripts/compare_food_access.py](../scripts/compare_food_access.py), [scripts/compare_council_funding.py](../scripts/compare_council_funding.py) | 2026-09-12 · `fae9fb5` — Validate depletion evidence and reject malformed balance ensembles | 2026-09-12 · `fa57363` — Record completed integrated follow-up and corrected stress ensemble | [model-evidence](model-evidence.md), [integrated-calibration](integrated-calibration.md), [integration-balance-followup](integration-balance-followup.md), [council-stress-balance](council-stress-balance.md), [history-performance-profile](history-performance-profile.md) |


## Workspace provenance

The initial inventory observed uncommitted wildlife, lake and integration work.
That work is now recorded in `3d244c8` and the integrated follow-up report
(`fa57363`); it is no longer a pending implementation. The scoped review pass
below records subsequent work separately.

## Updating this inventory

After a system change, update its source/document dates, commit references and
path scope. Inspect the actual diff before calling an update a review. For example:

```sh
git log -1 --format="%cs %h %s" -- src/service_allocation.rs
git log -1 --format="%cs %h %s" -- docs/service-allocation.md docs/research-cultural-work-requests.md docs/learning-resolution.md
git show --stat <commit>
```

For an explicit review, record its date, reviewed revision, scope, findings and
limitations in a linked Markdown report. Refresh the pending-work section against
`git status --short`; do not turn a local observation into a committed-work date.


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
