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
