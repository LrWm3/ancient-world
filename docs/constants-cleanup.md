# Named constants cleanup

This is a behavior-preserving refactor, not a tuning pass. Meaningful rates,
thresholds, durations and costs belong immediately after imports in their owning
file, with units in names where useful. Independent fixture inputs and expected
answers remain literal. Catalog data, array indices, zero/one identities and
format strings are not mechanically converted to constants.

Equal numbers are shared only when they represent the same policy. For example,
land travel speed and relocation distance preference currently both use 150, but
remain independently named. Soldier rations and civilian food reserves likewise
remain distinct policies. Shared definitions live in the relevant subsystem,
not a project-wide constants module.

## Progress

The repository-wide cleanup is **in progress**. These files have received a first
semantic extraction pass (further shared-policy reconciliation may still apply):

- `src/artifact_petitions.rs`
- `src/contagion.rs`
- `src/institution_relocation.rs`
- `src/route_warnings.rs`
- `src/peace.rs`
- `src/siege.rs`
- `src/military.rs`
- `src/road_upkeep.rs`
- `src/trade_contact.rs`
- `src/kin_support.rs`
- `src/military_supply.rs`
- `src/occupation.rs`
- `src/facilities.rs`
- `src/institution_capacity.rs`
- `src/domestic.rs`
- `src/domestic/assistance.rs`
- `src/domestic/resolution.rs`
- `src/institution_funding.rs`
- `src/institution_services.rs`
- `src/service_allocation.rs`
- `src/learning_resolution.rs`
- `src/heritage_renown.rs`
- `src/offices/service.rs`
- `src/institution_succession.rs`
- `src/vessels.rs`
- `src/vessels/crews.rs`
- `src/vessels/resolution.rs`
- `src/tool_access.rs` (reviewed; remaining literals are indices, identities and numeric bounds)

- `src/config.rs`
- `src/freight.rs`
- `src/household_economy/council_allocation.rs`
- `src/household_economy/family_support.rs`
- `src/household_economy/policy.rs`
- `src/materials.rs`
- `src/metallurgy.rs`
- `src/resolution.rs`

- `src/naming.rs`
- `src/naming/evolution.rs`
- `src/workshop_resolution.rs`
- `src/local_places.rs`
- `src/social_memory.rs`
- `src/storage.rs`
- `src/spatial.rs`

- `src/agriculture.rs`
- `src/agriculture_participation.rs`
- `src/labor.rs`
- `src/catalog.rs`
- `src/resources.rs`

- `src/hazards.rs`
- `src/navigation.rs`
- `src/history_environment.rs`
- `src/faction_interests.rs`
- `src/culture/learning.rs`
- `src/relocation.rs`
- `src/relocation/comparison.rs`

- `src/offices.rs`
- `src/participation.rs`
- `src/population_registry.rs`
- `src/individual_demography.rs`
- `src/civilization/daughter.rs`
- `src/household_economy/nutrition.rs`

- `src/civic_petitions.rs`
- `src/discoveries.rs`
- `src/discoveries/returns.rs`
- `src/history_timeline.rs`
- `src/main.rs`
- `src/region.rs`
- `src/relief.rs`
- `src/religious_relief.rs`
- `src/social_state.rs`

- `src/production.rs`
- `src/export_contracts.rs`
- `src/enterprises.rs`
- `src/civilization/production_forecast.rs`

- `src/society.rs`
- `src/governance.rs`
- `src/politics.rs`
- `src/expedition_heritage.rs`
- `src/household_economy.rs`
- `src/shipping.rs`

- `src/culture.rs`
- `src/culture/dynamics.rs`
- `src/culture/practices.rs`
- `src/culture/work_requests.rs`
- `src/economy.rs`

Reviewed without further numeric extraction (geometry/layout arithmetic, static
content, already named parameters, or independent test fixtures only):

- `src/history_atlas.rs` (map projection and drawing style only)
- `src/grid.rs`
- `src/lib.rs`
- `src/systems.rs`
- `src/territory.rs`
- `src/regional_mining.rs`
- `src/environmental_returns.rs`
- `src/continuity_fixture.rs`
- `src/civic_petitions/causal_tests.rs`
- `src/expedition_heritage/patron_finds.rs`
- `src/leadership.rs`

Completed the CPU semantic pass in `src/civilization.rs`, `src/expeditions.rs`,
`src/ecology.rs` and `src/gpu.rs`. Their shader policies remain in the pending
review. `src/viewer.rs` still needs its complete semantic pass.

WGSL shaders still need their complete semantic passes. Shared agricultural and
workforce constants now use `shared_shader_parameters!` (declared in `src/lib.rs`):
the owning module declares scalar Rust parameters after imports, and the macro
emits the same literal spelling into a WGSL prefix. Use WGSL-compatible f32/u32 literals (CPU f64 parameters emit the same
literal spelling as WGSL f32) without Rust-only suffixes or separators. The pipeline must include
the owner's `SHADER_PARAMETERS` exactly once. This changes no buffer layout and
avoids runtime float formatting. Standalone shader text requires those prefixes.
Existing parameters elsewhere still need sharing reconciliation; renaming both
copies does not make them a single source of truth.

## Verification

The first batch passed the ordinary library suite and the focused continuity
checks, including GPU cases for relocation, petitions, disease, peace and sieges.
Raw results stay under ignored `output/`.

Initial batch verification:

- `cargo test --lib`: 146 passed, 125 hardware/extended cases ignored.
- Explicit include-ignored runs for institutional capacity, military rosters,
  occupation, siege, peace, institutional relocation, artifact petitions and
  contagion: 16 focused checks passed, including GPU fixtures.
- `cargo clippy --all-targets -- -D warnings`: passed.
- Repository artifact policy and whitespace checks: passed.
- Local review aid expanded numeric constants and compared executable token order
  and values across all 21 modified Rust files, allowing rustfmt closure braces,
  trailing commas and the exact integer-two to float cast: matched. This is an
  additional refactor check, not an AST equivalence proof or a balance experiment.

No parameter values, scheduling rules, archive formats or intended outcomes were
changed. This batch does not complete the repository-wide extraction.


## Succession and crew-resolution batch (2026-09-12)

Extracted institutional ballot weights, scholarly topic normalization, quarterly
ballot interval and assembly labor. Reserve/forecast/execute now use the same
subsystem-owned ballot cost. Extracted the crew resolution work tolerance.
Averages, majority arithmetic and independent fixture values stay inline. This
batch preserves values and operation order and does not complete the pending list.

This batch passed three institutional succession tests (including GPU recovery),
the crew forecast fixture, and 149 ordinary library tests. The subsequent merchant
productivity change passed seven vessel tests and 150 library tests; it is a
separate behavioral commit, not part of the constants-only extraction.

## Vessel service batch (2026-09-12)

Extracted vessel backing materials, hull cap, cargo service rates, standby staffing,
voyage completion and load floors. Named and aggregate crews now reference the same
vessel-owned staffing target, wage multiplier and price floor. Crew matching, release
and receipt tolerances retain their original f32/f64 types and values. Independent
test expectations remain literal; no arithmetic reassociation or tuning is intended.

Verification: seven vessel tests passed, including the GPU crew fixtures; the
ordinary library suite passed 151 tests (130 remain explicitly ignored). Strict
all-target Clippy also passed. This batch advances the file checklist, not the
entire repository cleanup. Generated logs remain under ignored `output/`.

## Road, contact and supply batch (2026-09-12)

Extracted road weathering rates and condition thresholds, trade-contact retention,
exposure normalization and minimum qualifying deliveries, and parent/child versus
sibling support weights. Military supply forecasts and execution now share the
military module's ration, shortfall tolerance and starvation fraction. The forecast
retains f64 rate arithmetic; execution explicitly casts the shared fraction to its
original f32 precision. Tests retain independent literal expectations.

The society module only received shared-policy substitutions in this batch; its
full extraction remains pending. No rates, thresholds, update order or archive
fields changed.

Verification: 151 ordinary library tests passed (130 explicitly ignored), including
trade-contact expiry and kin-support fixtures. Explicit hardware-enabled runs
passed both road-upkeep checks and the military-supply checkpoint/batch fixture.
Strict all-target Clippy and repository artifact/whitespace checks passed. This is
refactor verification, not a new balance calibration; logs remain ignored under
`output/`.

## Domestic and institutional work batch (2026-09-12)

Completed the semantic pass for domestic care, neighbor assistance, care and
learning resolution receipts, institutional funding and room services, and service
allocation. Cultural work requests, labor and heritage only received shared-policy
substitutions and remain pending for their complete file reviews. Heritage study
spacing/count limits now have a single owner; administration reservations and
execution share a work cost. Legacy building-wear forecasts retain f64 precision,
while execution casts the same parameters to the original f32 arithmetic.

Verification: 151 ordinary library tests passed (130 extended checks ignored).
Explicit include-ignored runs passed domestic, institutional funding, room service
and institutional capacity suites, including GPU fixtures. All-target Clippy with
warnings denied passed. No intended behavior or archive layout change.

## Configuration and material-processing batch (2026-09-12)

Named configuration defaults and allowed ranges, household faction policy tables,
family support thresholds, freight capacity parameters, resolution receipt limits,
material markup and metallurgy recipes. Mineral validation shares the metallurgy
fractions. GPU lake polling shares the configuration limits. GPU dispatch and
history refresh received partial extraction; their shader interfaces remain in
the pending review. Fixed archive slots and independent expected recipes stay
literal.

The ordinary library suite passed 151 tests before the family-support and GPU
limit follow-up; strict all-target Clippy passed after those additions. Follow-up
verification is recorded below.

Follow-up: all 151 active library tests passed again (130 extended/GPU cases
remain explicitly ignored); all-target Clippy, artifact and whitespace checks passed.

## Naming, memory and workshop resolution batch (2026-09-12)

Named lexical contact/production thresholds, adoption timing, random streams,
reference weights, naming conventions and name-size limits. Shared vocabulary
option bounds with validation. Food report producers and validators share their
maximum reserve horizon. Recovery requests and execution share effort cost with
the original f32 grant/f64 ledger arithmetic. Workshop offers, peer learning and
receipt tolerances are named; service invoices and revision checks share the
enterprise quote multiplier. Enterprise and cultural work files still need their
full semantic passes. Archive size, checksum and chunking parameters are named;
archive version/stride layouts remain explicit. Spatial validation shares config
resolution limits.

Strict all-target Clippy passed; 151 ordinary library tests passed before the
final naming extraction. Remaining verification follows. The reviewed-without-edit
list distinguishes files needing no extraction from pending files.

Final verification for this batch: 151 active library tests passed after naming
changes (130 explicitly ignored); strict all-target Clippy, artifact and diff
checks passed.

## Agriculture, workforce and catalog batch (2026-09-12)

Completed CPU semantic extraction for agricultural settings/roles, farm staffing
receipts, common labor, catalog defaults and canonical resource accounting.
Agricultural land capacity, livestock composition, slaughter fractions and
workforce penalties have single owning declarations emitted into WGSL. Domestic
capacity shares the common work parameters. Iron and nonferrous recipes share
smelting fuel/recovery, ore pricing and residue capacity; no recipe yield changed.

Verification: the hardware-enabled family-care integration test passed, exercising
the combined WGSL and shared work parameters. The complete ordinary library suite
passed 151 tests (130 extended cases ignored), and strict all-target Clippy passed.
An earlier agriculture-only test filter matched zero tests and is not evidence;
the GPU fixture and full suite are the verification used here.

## Hazards, navigation and relocation batch

Named flood, granary, navigation search, faction appeal, learning and relocation
policies. CPU/GPU corridor and depth checks share hazard declarations. Navigation
and history-gather dispatch sizes are emitted from their CPU owners. Travel
forecasts and execution share rations and starvation parameters; departure and
destination preferences retain their previous values. Navigation and history-gather
shaders have completed their semantic pass; core simulation shaders remain pending.

The hardware-enabled CPU/GPU route-inspection comparison passed after the navigation
changes. Library and strict Clippy results for the complete batch follow below.

Complete-batch verification: 151 active library tests passed (130 explicitly
ignored); strict all-target Clippy and source-artifact/whitespace checks passed.

## Demographic ownership and office policy batch

Named office selection/tenure, resident participation, population initialization,
birth/casualty decisions and household nutrition parameters. Ration and mortality
rates share society-owned declarations; CPU f64 rates preserve their precision,
while the WGSL prefix retains the original f32 literal spelling. Registry age
boundaries replace domestic/participation copies. Political family limits are
shared with individual births. Daughter founding shares original provision costs
with aggregate founding, retaining separate f32 stock and f64 ledger arithmetic.
Society, civilization and politics still need their complete semantic reviews.

All ten individual-demography tests passed with ignored GPU fixtures enabled,
including travel, defense, birthdays and checkpoint continuation. All 151 active
library tests passed (130 explicitly ignored); strict all-target Clippy, repository
artifact policy and whitespace checks passed.

## Discovery, relief and explorer batch

Named research processing and teaching limits, petitions, relief shipment and donor
policies, social-pressure memory and notification thresholds. Research forecasts
and execution share specimen costs; secular and religious relief share transfer
limits while retaining their different donor reserves. CLI defaults share config
values; regional CPU/WGSL dispatch geometry shares one declaration. Regional
terrain shader policies still need their full pass. Map projection and drawing
style literals were reviewed without unnecessary extraction.

Verification: all four discovery tests passed with GPU fixtures enabled. The
regional GPU drainage/runoff fixture passed. All 151 active library tests passed
(130 extended cases ignored), and strict all-target Clippy passed. No tuning or intended behavior change.

## Production assets and enterprise batch

Named procurement, asset investment, wage posting, lease, dividend and export
contract policies. Workshop, housing, warehouse, waterworks and fishing-equipment
material costs share production-owned scalar declarations with GPU execution and
CPU capacity checks. Enterprise and export quotes share workshop capacity and
wear. Forecast dispatch shares the history workgroup size. Existing hypothetical
procurement allowances remain distinct from actual grants and physical yields.

Verification: six export-contract fixtures passed. The hardware-enabled prepaid
workshop-capacity/checkpoint test passed. All 151 active library tests passed
(130 extended cases ignored), and strict all-target Clippy passed. No intended
parameter, scheduling, arithmetic-order or archive-layout changes.

## Shipping, public funding and political policy batch

Named harbor maintenance, road friction, household purchasing, public funding,
representative demography, political recruitment and heritage study parameters.
CPU road surveys and GPU navigation share friction and search limits. Tax
forecasts and settlement payments share the autonomy reduction; cultural requests
and heritage execution share study effort. Separate precision and distinct policy
meanings remain explicit even where their numerical values happen to match.

Verification: all 151 active library tests passed (130 extended cases ignored).
The hardware-enabled shared-treasury administration-shortfall fixture passed,
and strict all-target Clippy passed. No intended behavior change.

## Culture and market batch

Named patron service, cultural eligibility, institution founding, practical actions,
conversion and reform evidence, local price adjustment and market reserve policies.
Action requests and execution share costs and gates where they express the same
rule. Existing charity request/execution differences remain separate parameters.
Wood and livestock compositions share owning declarations while preserving f32
stock arithmetic and f64 conservation arithmetic. Topic IDs, schema slots, calendar
conversions and independent fixture expectations remain literal.

Verification: all 23 culture-filtered tests and all 23 economy-filtered tests passed
with ignored GPU cases enabled. Strict all-target Clippy passed. This is extraction
and sharing, not calibration or a change to cultural decisions or market behavior.

## Expedition, founding and GPU configuration batch

Named expedition funding, crew provision, risk and collection policies, founding
requirements, legacy relief, settlement lifecycle thresholds, ecology diagnostics
and GPU allocation/dispatch configuration. GPU navigation and survey kernels now
share the device dispatch limit; the hardware check caught and corrected a missed
survey reference. Timestamp allocations and numeric precision remain unchanged.

Verification: 151 active library tests and strict all-target Clippy passed after
the expedition/founding changes. Hardware-enabled route inspection and seasonal
aquatic production tests passed after GPU-prefix changes, exercising terrain,
ecological and history pipelines. Five expedition crew tests also passed. Core
shader semantic extraction and the viewer review remain outstanding.

## Viewer controls and export interface batch

Named camera focus/zoom/drag parameters, map dimensions and event-export defaults.
The map renderer and WGSL share workgroup dimensions; uniform size derives from
its Rust layout and row alignment uses the wgpu API constant. Regional export
resolution and displayed assay progress share their owning subsystem's limits.

The hardware-enabled checkpoint/export fixture passed: all 31 layers dispatched,
PNG dimensions and regional exports checked, and checkpoint continuation matched.
Strict all-target Clippy passed. This is a partial viewer pass; remaining displayed
model values, controls and core shader parameters still need semantic review.

## Viewer scheduling and inspection batch

Named the per-frame history time budget, step limit, visible journey-plan count
and regional smoke-test dimensions and selection. Workshop inspection shares its
existing production material requirements; granary inspection shares hazard
height/material constants, preserving the prior display's f64 arithmetic.
Strict all-target Clippy passed. No simulation values or timing changed. The review
also identified an outdated grain-container display, recorded separately for a
behavioral correction rather than preserving its formula as new constants.

## Seasonal sunlight parameters

Reviewed `shaders/sunlight.wgsl`. Named the year length, equinox phase, monthly
orbital angle and equatorial normalization at the top of the owning shader.
Preserved literal precision and arithmetic order; unit bounds and numeric
identities remain inline. The hardware-enabled solar geometry fixture passed
against its independent rotating-surface integration reference, including both
hemispheres, poles and three axial tilts. Core ecology, terrain and economy
shaders still need their remaining semantic-parameter review.

## Regional compute shader review

Reviewed `shaders/region_compute.wgsl` and moved its meaningful relief, noise,
runoff, soil, habitat, flow-convergence and pool-transfer parameters to the top of
the owning shader. Shared head/change tolerances now use the same named policy.
Preserved literal precision and expression order. Layout indices, geographic
category IDs, hash mixing and simple interpolation identities remain inline.

The hardware-enabled regional fixture passed: repeated generation, acyclic
routing, accounted runoff, pools, habitat and invalid-region handling. This
completes this shader's parameter pass; planetary simulation, ecology, economy,
society and viewer shader reviews remain separate outstanding work.

## Regional renderer parameters

Reviewed `shaders/regional.wgsl`, the visual regional refinement shader. Named
water-depth, interpolation tolerance, noise frequency, roughness, flow smoothing
and ridge-detail parameters at the top of the owning file. Numeric precision and
expression order are unchanged; hash mixing, layout indices and interpolation
identities remain inline. The hardware-enabled
`gpu_history_checkpoints_and_exports` fixture passed, including regional rendering.
This completes this file's pass, not the remaining planetary shader review.

## Society shader illness and calendar parameters

Named the common ration share and bounded redistribution passes, numerical division
floor, monthly illness sources/retention, weather bounds, shortage-duration
threshold, crop year/planting interval and seed reserve in `shaders/society.wgsl`.
Housing-related exposure now uses the existing shared material-per-person
constants. Values, numeric types and arithmetic order are preserved.

The isolated hardware ration fixture includes the shader's parameter declarations
and passed its boundedness, conservation and priority checks. The full GPU
plot-reservation/recipe conservation fixture also passed, exercising production
shader assembly and a monthly history step. Strict all-target Clippy passed.
This is a partial society shader review: the shared CPU/GPU ration
priority bound still needs consolidation; layout flags and simple identities stay
inline. Planetary and ecological shaders remain outstanding.

## Shared ration priority and normalization floors

Consolidated the CPU validation and GPU ration-priority limit in the owning
society module's shared shader parameter declarations. The isolated GPU fixture
imports that same bound. Named the two remaining population normalization floors
in the society shader, keeping them separate because exposure and seed ratios
are distinct policies. This finishes the society shader's semantic-parameter
review; bit flags, vector indices, unit bounds and boolean encoding stay inline.
The wider cleanup remains open for the other core shaders.

Verification of the shared-priority change: both ration tests (including hardware)
and the full GPU plot-reservation/recipe conservation fixture passed. Strict
all-target Clippy passed. The values and resulting policy are unchanged.

## Credit calendar conversion

Underwriting and the new restructuring resolver now use the owning credit module's
existing `MONTHS_PER_YEAR` constant. Removed duplicate declarations of this shared
conversion; rates, term arithmetic and numeric types remain unchanged. Independent
operating-reserve and policy-duration values are not merged merely because they
also happen to be twelve.

## Settlement survey and legacy monthly production

Named the generic survey temperature/rainfall envelope, yield, soil floor,
flood/elevation exclusions, groundwater bonus and elevation penalty in
`shaders/civilization.wgsl`. The monthly generic climate and managed-crop
potential recovery use those same parameters, preserving arithmetic order.
Named legacy harvest seasonality, cultivated area, storage/spoilage, food
composition and coarse birth/death parameters separately from the modern
cohort and managed-production models. Calendar and age-ration references reuse
the society subsystem's definitions. This is parameter extraction, not crop
or demographic recalibration.

The sparse `history_environment.wgsl` and `managed_returns.wgsl` shaders were also
inspected: their remaining literals describe buffer layouts, pool indices and
serial/gather mechanics rather than tunable simulation rates. No additional
policy constants were needed there. Core ecology, planetary terrain and economy
shaders still require review; this does not close the overall cleanup.

The hardware plot-reservation and recipe-conservation test passed after these
changes, as did strict all-target Clippy and the executable build. A matched
one-year history comparison from the same seed-81 checkpoint produced byte-identical
serialized history before and after extraction (same backend, default history
options). This checks that exercised path, not all alternative modes. Regional weather binning and dispatch constants in the civilization
shader remain to be reviewed; the extraction above does not claim that every
nontrivial literal in that file has been addressed.

## Shared history-weather bins and dispatch stride

Completed the settlement shader's remaining weather/dispatch parameter review.
The ecology and settlement kernels now include `shaders/history_weather.wgsl`
for their common spherical bin dimensions and minimum drought-regime duration.
The settlement dispatch stride uses the existing GPU dimension limit and history
workgroup size, matching CPU dispatch scheduling. Hash arithmetic, buffer indices
and boolean encodings remain inline. No values or operation order changed.

The hardware plot/recipe-conservation fixture passed, exercising both shader
assemblies; strict all-target Clippy and executable build passed. This does not
finish the remaining ecology, terrain, economy or map-rendering shader reviews.

## Map display parameters

Named the elevation, climate, water and ecological layer display scales in
`shaders/view.wgsl`, together with palette spacing, visibility thresholds,
atmospheric falloff, regional zoom blending, relief contrast and selection weights.
These are display parameters, not simulated physical limits. Different uses of
the same number retain separate names where their meanings differ. RGB swatches,
buffer/layer indices, cube geometry and unit interval arithmetic remain inline.

Expanding the new scalar constants back to their literal values reproduces the
previous shader expressions exactly after ignoring comments and whitespace.
The hardware checkpoint/export fixture passed: it renders all 31 atlas layers,
exports PNG and checks repeatable regional export. Strict all-target Clippy also
passed. Globe branch expressions are unchanged by literal expansion; this fixture
does not exercise interactive globe controls.

## Planetary climate and hydrology parameters

Named climate response, vapor transport/recycling, precipitation and wind
parameters, runoff conversion, snowmelt, infiltration, groundwater release,
climate convergence tolerances, and lake transport/budget thresholds in
`shaders/simulation.wgsl`. Matching processes share their existing parameters;
for example land and great-lake evaporation use one formula. The precipitation
annualization of 365 and runoff year of 31,557,600 seconds remain distinct as
before; this is not a calendar or physical-model correction.

These declarations follow `struct Params`, preserving the shared `Cell`-only
source prefix consumed by ecology, regional generation and viewer assembly.
They still precede the owning shader's functions. Expanding the constants
reproduces the previous expressions exactly after whitespace/comment removal.
Planet structure, tectonics, geology, erosion and the legacy ecological pass in
this shader remain to be reviewed. Hardware checkpoint/export validation passed,
as did strict all-target Clippy. Seasonal convergence passed for seeds 17, 81
and 256, including the deliberately under-budgeted nonconvergence control. The
secondary-lake fixture also preserved volume and equalized connected surfaces
across a cube-face seam.

## Erosion, weathering and elevation constraints

Named the thermal/fluvial erosion thresholds, rates, vegetation protection,
distance and hardness floors, bedrock weathering, soil cap, lithification and
regional elevation bounds in `shaders/simulation.wgsl`. Fluvial cutting uses the
same land floor as the artistic terrain constraint. The water-storage update
now references the existing groundwater-capacity constant. Separate river and
thermal rates remain distinct even where they share a vegetation factor.

Expanding constants reproduces the prior expressions exactly. The hardware
water/sediment-budget fixture and strict all-target Clippy passed. These checks
verify unchanged equations and conservation on the tested fixture; planet-mask,
tectonic, deposit-selection and legacy ecology parameters remain outstanding.

## Plate-field motion and geological activity

Named plate count, angular speed, spatial warp, boundary falloff, convergence
normalization, regional geological-activity thresholds, uplift, crust limits,
volcanic rejuvenation and metamorphic exposure thresholds in
`shaders/simulation.wgsl`. Plate position and relative velocity share their speed
parameters. Unit conversion remains multiplication by the original literal;
no division replacement or changed arithmetic order was introduced. Noise seed
offsets and categorical rock/setting IDs remain inline.

Constants follow `Params` and precede functions to preserve the shared `Cell`
source prefix. Literal expansion reproduces the prior shader expressions exactly.
The hardware geography fixture passed its three-seed geography checks and
30-epoch finite-state run. Strict all-target Clippy also passed.
Planet-mask, initial terrain/stratigraphy, deposit-selection and legacy ecology
parameters remain outstanding in this shader.

## Continent-mask parameters

Named inner-continent placement jitter, center angles, size/aspect variation,
outline harmonics, channel protection and enclosing/exterior shore parameters in
`shaders/simulation.wgsl`. The channel cap references the same minimum center
angle used by placement. The original 6.28 outline phase span remains 6.28; it
was not silently replaced by a more precise turn. Noise/hash seed offsets,
categorical region IDs and spherical geometry identities remain inline.

All 35 new declarations follow `Params`, preserving the shared `Cell` prefix.
Expanding them reproduces the previous shader expressions exactly. Hardware
geography checks passed for seeds 0, 42 and 999, followed by the 30-epoch
finite-state check. Strict all-target Clippy passed.
Initial terrain/stratigraphy, deposit-selection and legacy ecology parameters
remain outstanding in this shader.

## Initial planet state

Named 43 initial terrain, ridge, bathymetry, climate, water, vegetation and
stratigraphic parameters in `shaders/simulation.wgsl`. Initialization shares its
rain and great-lake-level values across the fields that describe the same stock;
it also uses the existing lapse-rate and kilometer conversion. Independent
initial terrain amplitudes remain separate from later artistic elevation bounds.
Noise offsets and categorical formation/settings IDs remain inline.

Literal expansion reproduces the previous expressions exactly. The declarations
remain outside the shared `Cell` prefix. Hardware geography/long-run and strict
all-target Clippy verification passed, including the three-seed geographic
constraints and 30-epoch finite-state fixture. Deposit and legacy ecology parameters,
plus remaining shared coast/basin thresholds, still require review.

## Geological settings, deposits and surface selection

Named 55 geological-setting, mineral-potential, soil-selection and legacy
surface-selection parameters in `shaders/simulation.wgsl`. Deposit coefficients
remain distinct by formation process even when values coincide. The basin neighbor
check now shares the existing depression threshold; coast cleanup shares the
initial lake level and lake/ocean salinity values. No new formation mechanism or
changed resource abundance is implied.

The legacy growth and disturbance locals still do not feed vegetation output;
naming those values does not make them active ecological processes. Vegetation
stocks remain governed by the separate ecology subsystem. Removing these dormant
locals can be a separate cleanup, rather than changing formulas during extraction.

All declarations remain outside the shared `Cell` prefix. Literal expansion
reproduces the prior expressions exactly. All seven geology fixtures passed, including
the hardware province/host-rule and finite-column checks. The cross-seam
secondary-lake fixture and strict all-target Clippy also passed. Remaining numeric
review includes noise construction, discrete coast voting and other shader files;
this does not declare the repository cleanup complete.

## Planetary noise and coast voting

Named noise-octave frequencies/weights, coast-neighbor voting thresholds, the
legacy mineral sampling group size, square-kilometer conversion and unresolved
routing sentinel in `shaders/simulation.wgsl`. Literal expansion reproduces the
previous expressions exactly. The GPU geography/long-run fixture and strict
all-target Clippy passed.

The final literal inspection leaves hash/salt construction, interpolation and
spherical geometry identities, array/category indices, unit-interval clamps and
representation sentinels inline. This completes the shader's first semantic
parameter review; it does not finish the larger ecology,
economy and other outstanding repository review. The separate legacy surface
selection's dormant locals are documented above rather than presented as active
biological mechanisms.

## Ecological aggregation and geological habitats

Named 18 parameters in `shaders/ecology.wgsl`: area conversion and normalization,
deposit phosphorus enrichment, groundwater/wetness response, enriched/province/vent
activity thresholds, reaction weighting and shared-edge land/channel conductance.
Equal-looking values remain separate when they describe different responses.
Indices, geometric identities and normalized clamps remain inline.

Expanding the new names back to their original literal spellings reproduces the
previous shader token sequence, including operation order. This is an aggregation
pass only; producer, consumer, lake-transport and other ecology parameters still
need review. The running held-out monetary experiments use a copied executable
from before this extraction, so these source edits cannot alter their runtime.

The aggregation coarse-grid budget/checkpoint fixture, finite hydrogen-supply
fixture and strict all-target Clippy passed.

## Initial ecological stocks and thermal encoding

Named another 23 shader constants for initial terrestrial/aquatic C/N/P stocks,
finite source rock and reduced chemistry, founder occupancy, geological exposure
and thermal-preference bounds. The temperature encoding offset is now declared
once in the ecology module's Rust/WGSL shared parameters and used by the Rust
inspection decoder and shader. Independent fixture inputs remain literal.

Literal expansion reproduces the previous shader token sequence. Initial stocks
remain declared imports; this changes neither their amounts nor their ecological
meaning. Noise phases and categorical/array layout remain inline. This does not
complete the rest of the ecology shader's parameter review.

Verification passed after the seeding/encoding extraction: coarse-grid ecological
budgets and checkpoint continuation, finite hydrogen supply, narrow-habitat
aggregation, fine-edge CPU/GPU conductance including cube seams, thermal ecotype
feeding/persistence, and strict all-target Clippy. The two extractions named 41
shader-owned parameters plus one shared Rust/WGSL encoding parameter.

## Producer selection and community competition

Named 16 additional ecology shader parameters for eligible habitat fractions,
climate and shade scoring, diet access, initial competitor shares, light response,
biomass normalization, disturbance cost and bounded composition changes.
Arithmetic identities, encoded flags and pool/category indices remain inline.

Expanding the names reproduces the previous literal token sequence exactly.
The GPU identical-competitor fixture and strict all-target Clippy passed:
competing producers still share one production budget. This is a naming-only change; broader biological cycling
and transport parameter review remains outstanding.

## Terrestrial nutrient cycling and chemical production

Named 32 more parameters in `shaders/ecology.wgsl`: temperature response,
reactive-rock conversion, oxidant supply, phosphorus release/mobilization,
decomposition and retention, producer mortality/shading, chemical access and
conversion, and nitrogen-fixation costs and limits. Existing aggregation wetness
and competitor light-response parameters are reused where they are the same
response. Pool/category indices and trivial arithmetic remain inline.

The groundwater retention/export partition is one paired parameter used by
biology and river injection. Its two entries must sum to one; it is not two
independent sources. The original 0.75/0.25 partition and all other values remain
unchanged. Expanding names to the prior literals reproduces the edited biology
expressions exactly, including evaluation order.

The monetary regression uses its copied `7aef164` executable; subsequent shader
source edits do not alter the running experiment. Aquatic cycling, consumer
feeding, transport, weather and remaining file parameters still need review.

The terrestrial pass passed GPU energy/phosphorus limitation, finite hydrogen
reserve depletion, coarse-grid budgets and checkpoint continuation, plus strict
all-target Clippy.

## Stratified lake exchange and sediment cycling

Named 18 aquatic parameters for surface/deep layer bounds, peripheral upwelling,
wind/shelf response, finite exchange caps, remineralization, oxygen response,
anoxic phosphorus release and burial. Aquatic nitrogen fixation now references
the same N/P, yield and carbon-cost parameters as terrestrial fixation.
Unit-interval clamps, category codes and geometry identities remain inline.

Literal expansion reproduces the previous aquatic expressions exactly. This does
not add hydrodynamic detail or alter circulation: the spatial bias and bounded
exchange remain the existing game model.

After the aquatic extraction, GPU lake-mixing transfer and coarse-grid
conservation/checkpoint fixtures passed, as did strict all-target Clippy.
Together these two passes name 50 additional shader parameters without tuning.

## Consumer feeding and turnover parameters

Named seven parameters in `shaders/ecology.wgsl`: prey withdrawal cap, thermal
adaptation per replacement, detrital and respiratory carbon fractions, annual
mortality, numerical extinction and division guard. The two carbon fractions
remain separate literal constants to preserve the existing rounding paths.
Catalog trait unpacking, pool indices and mathematical identities remain inline.
An exact literal-expansion comparison reproduced the original consumer block.
The GPU alternate-aquatic-food conservation fixture and strict all-target
Clippy passed. The rest of the shader review remains unfinished.

## Pairwise ecological transport parameters

Named 17 shader parameters for current direction and wind response, depth/friction,
edge-transfer caps, deep-water exchange, guild food attraction, uphill and seasonal
migration, movement respiration and numerical division guards. Repeated unrolled
guild transport blocks now use the same movement-cost parameter. Geometry identities,
calendar indexing and packed pool accesses remain inline. Encoded thermal bounds
still need a shared CPU/GPU review rather than independent duplicate constants.

Exact expansion reproduces the previous transport expressions. The hardware GPU
migration fixture passed nutrient delivery, removal/restoration and conservation
checks; strict all-target Clippy also passed. These are parameter names, not new
circulation or movement mechanics.

## Shared thermal validation and river payload bounds

The encoded thermal preference endpoints (1 and 141) now live beside the shared
81-degree offset in the ecology module's Rust/WGSL parameter table. CPU archive
validation and all twelve shader guild-transport clamps use those same endpoints.
The zero sentinel remains inline. This preserves both the encoding and its range.

Named seven river parameters: land normalization, retained/routed fractions,
seconds per year, discharge capacity multiplier, floodplain storage depth and
volume division guard. The collection and clearing passes share the same guard;
physical transfer arithmetic and complementary fraction literals are unchanged.
Literal expansion verifies the river expressions. The GPU thermal feeding and
persistence fixture, routed-overflow/runoff fixture and strict all-target Clippy
passed. An initial river filter matched zero tests; the explicitly named overflow
fixture was subsequently run and passed. No new watershed behavior is claimed.

## Ecology feedback and monthly weather parameters

Named 22 parameters for derived fertility/cover, habitat classification thresholds,
seasonal temperature and rain response, snowmelt, evaporation, infiltration,
groundwater capacity/release, flood release and plot-water normalization. Separate
habitat thresholds remain separate from similarly valued reporting thresholds;
they serve different decisions. Hash arithmetic, category IDs, freezing zero and
unit-interval bounds remain inline. Exact literal expansion matches all previous
feedback/weather expressions. The GPU dry-month secondary-lake evaporation
fixture and strict all-target Clippy passed. Remaining fallback traits and
biological numerical/diagnostic bounds still need review.

## Ecology fallback traits and biological guards

Named 16 remaining fallback-trait and numerical/diagnostic parameters in
`shaders/ecology.wgsl`: empty-catalog microbial defaults, terrestrial and aquatic
producer stoichiometry and maintenance, disturbance/settling, production and
fixation division guards, and wetness/trace diagnostic thresholds. Equal-valued
guards retain separate names where their roles differ. Category IDs and zero/one
identities remain inline. Exact literal expansion reproduced the previous shader;
no rates, arithmetic order or catalog values changed.

The hardware-backed `maintenance_returns_excess_body_nutrients_without_creating_them`
and `no_energy_and_no_phosphorus_limit_growth` fixtures each ran and passed.
Strict all-target Clippy passed. The remaining shader and repository-wide semantic
review is still open; this does not claim completion of constants cleanup.
