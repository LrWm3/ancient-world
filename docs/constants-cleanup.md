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
- `src/occupation.rs`
- `src/facilities.rs`
- `src/institution_capacity.rs`

These files only received shared-constant substitutions or relocation of existing
constant declarations, and still need their full semantic pass:

- `src/society.rs`
- `src/governance.rs`
- `src/politics.rs`
- `src/economy.rs`
- `src/civilization.rs`
- `src/relocation.rs`
- `src/relief.rs`
- `src/religious_relief.rs`
- `src/expeditions.rs`
- `src/discoveries.rs`
- `src/production.rs`

The remaining source files below have not yet received this cleanup. Associated
API constants and compile-time assertions should be reviewed in context rather
than blindly moved out of their types or layout checks.

- `src/agriculture.rs`
- `src/agriculture_participation.rs`
- `src/catalog.rs`
- `src/civic_petitions/causal_tests.rs`
- `src/civic_petitions.rs`
- `src/civilization/daughter.rs`
- `src/civilization/production_forecast.rs`
- `src/config.rs`
- `src/continuity_fixture.rs`
- `src/culture/dynamics.rs`
- `src/culture/learning.rs`
- `src/culture/practices.rs`
- `src/culture/work_requests.rs`
- `src/culture.rs`
- `src/discoveries/returns.rs`
- `src/domestic/assistance.rs`
- `src/domestic/resolution.rs`
- `src/domestic.rs`
- `src/ecology.rs`
- `src/enterprises.rs`
- `src/environmental_returns.rs`
- `src/expedition_heritage.rs`
- `src/export_contracts.rs`
- `src/faction_interests.rs`
- `src/freight.rs`
- `src/gpu.rs`
- `src/grid.rs`
- `src/hazards.rs`
- `src/heritage_renown.rs`
- `src/history_atlas.rs`
- `src/history_environment.rs`
- `src/history_timeline.rs`
- `src/household_economy/council_allocation.rs`
- `src/household_economy/family_support.rs`
- `src/household_economy/nutrition.rs`
- `src/household_economy/policy.rs`
- `src/household_economy.rs`
- `src/individual_demography.rs`
- `src/institution_funding.rs`
- `src/institution_services.rs`
- `src/institution_succession.rs`
- `src/kin_support.rs`
- `src/labor.rs`
- `src/learning_resolution.rs`
- `src/lib.rs`
- `src/local_places.rs`
- `src/main.rs`
- `src/materials.rs`
- `src/metallurgy.rs`
- `src/military_supply.rs`
- `src/naming/evolution.rs`
- `src/naming.rs`
- `src/navigation.rs`
- `src/offices/service.rs`
- `src/offices.rs`
- `src/participation.rs`
- `src/population_registry.rs`
- `src/region.rs`
- `src/regional_mining.rs`
- `src/relocation/comparison.rs`
- `src/resolution.rs`
- `src/resources.rs`
- `src/road_upkeep.rs`
- `src/service_allocation.rs`
- `src/shipping.rs`
- `src/social_memory.rs`
- `src/social_state.rs`
- `src/spatial.rs`
- `src/storage.rs`
- `src/systems.rs`
- `src/territory.rs`
- `src/tool_access.rs`
- `src/trade_contact.rs`
- `src/vessels/crews.rs`
- `src/vessels/resolution.rs`
- `src/vessels.rs`
- `src/viewer.rs`
- `src/workshop_resolution.rs`

WGSL shaders also remain pending. CPU/GPU equations that share parameters need a
coherent shared definition mechanism; duplicating renamed literals on each side
does not establish a single source of truth.

## Verification

The first batch passed the ordinary library suite and the focused continuity
checks, including GPU cases for relocation, petitions, disease, peace and sieges.
Raw results stay under ignored `output/`.

Final batch verification:

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
