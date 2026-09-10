# Bounded social observations and pressure memory

This increment adapts the proposed aggregate-society model to the existing simulation. It preserves authoritative GPU age cohorts, finite household journeys, household property claims, named people and traditions. It does not create a second population, wealth or food inventory.

## State and update order

An optional `SocialState` on `Society` contains one fixed-layout `SocialCell` per settlement. Dense fields use fixed arrays and a pure bounded pressure update; the CPU currently gathers existing sparse household/cultural records. This is CPU work with a potential GPU migration path, not new GPU-resident society simulation.

| Field | Meaning |
|---|---|
| 3 × 5 age/livelihood projection | Children, working-age adults and elders across farming, forestry, mining, craft and other/care |
| Eight ownership bins | Resident household counts by beneficial ownership relative to the local mean household share |
| Four pressures | Remembered hunger, disease burden, waterlogging/cleanup disruption and ownership inequality |
| Four named faith shares + other | Existing household affiliations, normalized by resident household counts (not property shares) |
| Nine faction-interest shares (three legacy slots plus six additional slots) | Existing household faction affiliations, normalized locally |
| Exposure and status | Latest observations, sustained-strain/recovery counters, episode flag and observation month |

Labor allocations are projected into the existing adult cohort, capped at the available adults. Children and elders remain in other/care. This is an observational age × livelihood table, not recruitment or employment transitions. Its rows sum to existing age cohorts; it never adds workers to production. Other/care must not be interpreted as unemployment.

Ownership is not net worth or a population class histogram. The bins count households, using less than 1/8, 1/4, 1/2, 1, 2, 4 and 8 times the mean share as boundaries. A household Gini provides an inequality observation. No new money or possessions are assigned. Food remains pooled by the existing settlement economy; class-specific entitlement and differential famine mortality are deliberately deferred until actual distribution rules exist.

Culture and faction vectors summarize the existing household model rather than introducing another conversion system. The top four traditions retain stable IDs; smaller traditions remain in an explicit other share. Absent affiliation data yields zero shares, not an invented dominant culture. Household shares are an existing population/affiliation proxy, not a census of individual beliefs. Travelers are excluded from resident observations; their age cohorts and possessions remain in the existing journey records.

## Pressure and consequences

Each pressure moves 18% toward a higher monthly exposure and 8% toward a lower one. Shortages therefore leave a lingering effect after food supply recovers. Pressure is updated once per completed history month; extra boundary reads only refresh projections.

The update runs after that month's economic, migration, cultural and annual political work. Its pressure memory informs subsequent decisions:

- Household departure willingness receives a bounded additional urgency of at most 0.2. Existing shortage/flood triggers, provisions, route permissions, destination capacity, household choice and journey accounting remain authoritative.
- The farming-interest political urgency blends current shortage equally with remembered hunger. Without indicators, the previous instantaneous rule is preserved. The existing vote weights, funding and political requirements still determine outcomes.

Morale and migration-pressure values are explicit weighted proxies derived from pressure, not additional currencies or independent agents. Disease and inequality are observed here but do not yet introduce new mortality, taxation or class behavior.

A `social_strain` event requires three consecutive observations above 0.55 hunger/disruption pressure. A `social_recovery` requires six below 0.25. The episode flag prevents repeated monthly notices. Empty sites do not report a successful recovery. Events include measured pressure/shortage values and, where available, link to preceding relevant local events. Full time series are not retained in the state.

## APIs, UI and archives

Newly enabled societies start an observation baseline automatically. Older saved societies deserialize with `indicators = None` and retain previous behavior until explicitly enabled with `Generator::enable_social_indicators()` or the **Observe social pressures** button. Enabling records a baseline at the current month; it does not reconstruct prior pressures.

`History::social_indicators(site)` exposes read-only observations. Settlement inspection shows pressure, morale, labor projections, ownership bins and minority affiliations. The existing archive header serializes the state and clocks; no GPU buffer or world archive version changes are necessary for this increment.

The state is bounded per settlement. Sparse gathering currently uses per-site household lists and affiliation maps; a GPU implementation would need packed input tables and a reduction stage. Existing settlement limits remain unchanged. No GPU speedup is claimed.

## Evaluation

`history_evaluate` now includes final social state in its JSON reports. `--no-social-indicators` disables both observation and decision modifiers for a paired control. For example:

```sh
cargo run --release --example history_evaluate -- --seeds 17,81,256 --years 50 --crop-yield-scale 0.33 --output output/social-indicators --label enabled
cargo run --release --example history_evaluate -- --seeds 17,81,256 --years 50 --crop-yield-scale 0.33 --output output/social-indicators-control --label disabled --no-social-indicators
```

Controlled tests check bounded pressure buildup/recovery, projection totals, unchanged population stocks, repeated-read idempotence, debounced strain/recovery events and absent-field archive defaults. Existing history tests exercise relocation/relief constraints, culture, politics, conservation and checkpoint/batch-speed continuation.

## Cohort food allocation

Monthly consumption now allocates actual food across the three existing age cohorts. Household ownership shares are not treated as a population census. Class-specific purchases and income transfers remain deferred.

Pre-transition child/adult/elder populations require the existing 10/18/14 kg calorie-equivalent food per month. The GPU withdraws food once; ration allocation partitions that consumed quantity without creating another stock or withdrawal.

Default priorities give equal proportional sufficiency. `Generator::set_ration_priority(site, [child, adult, elder])` accepts additional priorities from 0 to 3 at a completed boundary and records a policy event. Half of consumed food is distributed in proportion to need; the rest is weighted by unmet need and priority, capped at need with surplus redistributed. Fully supplied settlements meet all needs regardless of policy. Daughter settlements start with equal priorities.

Cohort shortages drive existing age-specific famine mortality; adult sufficiency drives the existing birth-rate reduction. Disease still uses aggregate shortage. Policy redistributes deprivation; it cannot eliminate the underlying shortage. These remain game-model demographic rates.

Fixed arrays `Demography.ration_need` and `ration_eaten` retain the last production month's three cohort values and total. `ration_priority` is also fixed. These observations precede later migration, raids and aging, so cannot be reconstructed from final resident populations. Settlement inspection displays allocations and policy. Old archives default absent fields to zero and resume with equal allocation. GPU strides now derive from the Rust structure instead of a hardcoded 48 bytes.

The evaluator accepts `--protect-vulnerable-rations` to apply `[3,0,3]` to initial settlements and reports sampled allocations. This is an explicit policy scenario, not an automatic decision or a universally beneficial optimization.

GPU fixtures cover absent cohorts, unequal needs, no food, partial supply and oversupply. Allocations must reconcile with consumed food, stay below need and retain the common floor. Integration checks reconcile allocations with the food ledger and test nondefault policy checkpoint continuation. All 50 targeted library, culture, market, political, ration and society tests passed, including the century-history fixture. Clippy with warnings denied, formatting and whitespace checks passed.

## Follow-on implementation

[Household income and retail food](household-economy.md) now implements finite wallets and a common-plus-purchased food allocation. The earlier roadmap below describes the broader class model, which remains partial.

## Next useful additions

The next substantial economic step would be explicit household/class food entitlements and income transfers, with a reconciled distribution ledger. Only then should poverty bins affect mortality or relief allocation. Persistent livelihood skills could later travel with existing migrant cohorts. Housing, disease transmission, labor-market recruitment and institutional capacity would each need their own measured inputs and budget rules; they are not inferred from the current proxy fields.

### Recorded first-pass results

Three paired seeds ran for 50 years at crop yield scale 0.33, with 16 initial settlements and otherwise matching settings:

| Seed | Final population, enabled / disabled | Strain / recovery episodes (enabled) | Household departures, enabled / disabled |
|---|---:|---:|---:|
| 17 | 2329.6 / 2330.9 | 1 / 1 | 2 / 2 |
| 81 | 2014.1 / 2012.8 | 1 / 1 | 0 / 0 |
| 256 | 2971.9 / 2973.6 | 0 / 0 | 0 / 0 |

All 16 settlements remained active in each run. Final maximum site hunger pressure was 20.2%, 24.5% and 12.1% respectively. This is modest feedback, not a new collapse mechanism or evidence that pressure increases migration in every world. The event filter recorded sustained episodes and recovery without requiring one in every seed.

The history evaluator's maximum relative residual across its existing material, water, money, goods, food and population checks was below 1.54e-5 in both modes. All 48 targeted tests passed: 22 library tests, 13 culture tests, seven market tests, four political tests and two society tests. Those include conserved relocation/relief fixtures and century-history checkpoint continuation comparing a 12-month batch with twelve monthly calls. Clippy with warnings denied, formatting and diff whitespace checks passed. Wall times shared GPU capacity with tests and are not controlled performance benchmarks.

### Ration-policy evaluation

Paired 50-year runs at crop yield 0.33, with 16 initial settlements:

| Seed | Equal-ration population | Child/elder priority population | Difference |
|---|---:|---:|---:|
| 17 | 2330.1 | 2271.2 | -2.5% |
| 81 | 2012.5 | 1938.4 | -3.7% |
| 256 | 2972.4 | 2901.1 | -2.4% |

All 16 settlements remained active in every run. Maximum relative conservation residual stayed below 1.54e-5. Protecting vulnerable cohorts did not increase total population in these runs: lower adult sufficiency affects births and adult survival, with subsequent economic and historical feedback. These aggregate results do not isolate each causal contribution or establish an optimal policy. Reports are in `output/ration-equal.json` and `output/ration-priority.json`. Execution overlapped other GPU tests; timings are not a controlled benchmark.

## Distribution-aware hardship

The first selected addition from the [archived history design menu](archive/history-simulation-considerations.md) is real household distributions and sustained minority deprivation. Existing cohorts, migration, faiths and factions already cover much of that document's first stage; adding another population model would duplicate them.

- `cash_buffer[8]` counts resident accounts by liquid purchasing power: below 1/8, 1/4, 1/2, 1, 2, 4, 8, or at least 8 months of food at the current local price. Monthly need uses the existing equal-household-size approximation and current age cohorts. These are cash buffers, not net worth, social classes, or counts of people.
- `food_security[4]` counts households with observed unmet food need of at most 2%, 10%, 30%, or more than 30%. Missing observations are excluded rather than classified as secure. The inspector reports the observed count.
- `household_stress` stores mean observed shortage, severely deprived share, remembered severe share, and sample count. Memory rises by 18% toward worse conditions per observed month and falls by 8% during recovery.
- `distribution_status` debounces history: remembered severe deprivation of at least 25% for three observations opens an episode; six observations below 10% close it. The new events are `household_deprivation` and `household_deprivation_recovery`, with measured counts and causal links to preceding local hardship or access events.

The migration-pressure proxy uses the greater of aggregate hunger memory and severe-household deprivation memory. Existing household choice, crisis gates, routes, provisions and destination capacity still arbitrate actual movement. This can reveal a hungry minority that a tolerable settlement average hides; it does not create migrants, mortality or money directly.

Food accounts retain the settlement where consumption was observed. A household moving after monthly consumption cannot relabel an origin meal as a destination observation. Cash observations can reflect current residency immediately; food observations require a current-month sample from that same settlement. Missing months freeze deprivation memory rather than fabricating recovery. Repeated reads do not advance pressure or counters.

All dense observations remain fixed-size CPU arrays. They serialize with zero defaults for older archives, and do not introduce a new GPU state buffer. Existing JSON evaluation reports include them through `social_state`.

Controlled checks include an approximately one-third severely deprived minority with mean household shortage below 20%, unchanged authoritative stocks, histogram totals, idempotent boundary reads, delayed recovery, migration-origin filtering and missing-field archive compatibility. The next promising choices from the design menu are measurable institutional capacity and persistent site assets; these remain separate work, not implied by the distribution fields.

### Distribution evaluation

Three 50-year runs at crop yield 0.33:

| Seed | Final population | Deprivation episodes | Recovery episodes | Final observed households |
|---|---:|---:|---:|---:|
| 17 | 1995.2 | 16 | 11 | 255 |
| 81 | 1744.2 | 18 | 14 | 248 |
| 256 | 2394.4 | 8 | 6 | 254 |

All runs completed with relative conservation residuals below 1.54e-5. Repeated episodes represent recovery followed by renewed hardship, not monthly notices. Final observation counts exclude missing samples and post-consumption arrivals. Reports are in `output/household-distributions.json`; the detailed distributions are under each run’s `social_state.sites`. These are household observations, not an individual census or a calibrated class model.

Validation: 31 targeted tests passed (25 library, four politics, two society), including century histories, conserved household journeys and batch-versus-monthly checkpoint continuation. Clippy with warnings denied, formatting and whitespace checks passed.
