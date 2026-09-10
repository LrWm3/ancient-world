# Local producer competition and flexible diets

The first food-web increment adds two persistent producer identities per terrestrial layer and aquatic producer pool. Up to four weighted food sources per animal guild replace its single fixed source. Dense selection, competition and feeding remain GPU work.

## Producers

The five terrestrial layers and aquatic producers retain their existing **shared C/N/P pools**. Each now has a composition record containing two catalog IDs and a biomass fraction. This is a regional mixture model: it does not add independent per-species nutrient compartments or individual organisms.

Suitable incumbents persist. If a habitat becomes unsuitable, the affected fraction goes into detritus (aquatic sediment for aquatic producers) before replacements establish. A bounded competition equation adjusts relative abundance using growth, light response, nutrient requirements, maintenance and turnover. The combined population uses biomass-weighted traits in the existing production calculation. Both competitors share its energy and nutrient budget. Matching competitors therefore cannot double production simply by having different IDs.

The six existing aquatic producers now have differing temperature tolerances, growth rates, phosphorus requirements, maintenance and turnover. Faster bloom-formers carry larger nutrient and turnover costs; slower producers can remain competitive under scarcity. Cell inspection shows both producer names and their estimated biomass shares.

This is local assembly and competition, not dispersal, ecotype evolution or speciation. Replacement candidates still come from the suitable catalog. Tiny surviving fractions represent regional persistence, not confirmed individual survivors. Aquatic transport still moves the bulk producer pool and uses the receiving community's composition; it does not transport species-specific propagules.

## Consumers

Guild TOML entries accept a `diet` with one to four distinct edible compartment IDs and positive weights. Missing/empty diets use the legacy `prey` field. Validation rejects duplicate entries, self-feeding, invalid weights and non-food pools in explicit diets.

A guild has one demand budget. The shader distributes it across weighted, locally accessible sources and caps withdrawal from each source. It transfers actual C/N/P into consumer tissue; assimilation losses, respiration, excretion and mortality follow the existing ledgers. Migration attraction uses the same diet instead of only the first food source.

Examples include understory/root-mat/detritus small herbivores; canopy/understory browsers; predators with several herbivore sources; underground-feeding detritivores; aquatic grazers using plankton and organic sediment; and aquatic predators using grazers and migratory animals. Terrestrial and aquatic sources require their respective local habitat fractions. This prevents remote water from feeding a dry-land cell.

The twelve existing guilds remain intact, including aquatic grazers, aquatic predators, migratory river animals and colonial waterbirds. New giant/colossal aquatic guilds and explicit small-river or isolated-lake habitat fractions are deferred: they need basin-aware habitat and transport work, not merely larger body-mass labels. This increment improves diets in the currently represented aquatic habitats; it does not claim detailed food webs in every river reach.

## Archives and controls

`producer_competition = true` in the bundled catalog enables the mixture model. Set it to false for single-producer comparisons. Empty guild diets provide fixed-prey controls. Existing saves carry their own archived catalogs; older catalogs default to single-producer behavior and empty diets rather than silently acquiring new content.

Archive version 5 expands ecological cells from 512 to 608 bytes. The six added vec4 records are composition metadata, not additional nutrient inventories. Buffer-limit checks, memory estimates, managed-production buffers, viewing and readback all use the new stride. Versions 2–4 expand with empty composition records while preserving original stocks and clocks. Version-one import remains explicit. The extra ecological-buffer memory is 18.75%; terrain/environment buffers are unchanged.

## Evaluation

`cargo run --release --example foodweb_evaluate -- 120` runs seeds 17, 81 and 256 with single producers/fixed prey, competition/fixed prey, and competition/flexible diets. An optional month count allows longer checks, e.g. `1200`. Worlds use terrain edge 64, ecology edge 32, one geological epoch with one ecological year, followed by the requested ecological interval. Reports go to `output/foodweb/comparison-MONTHS.json` (the initial 120-month report is also retained as `comparison.json`).

Mixed-cell counts require two producer identities with 5–95% shares and non-negligible biomass. Guild biomass summaries are unweighted cell means, not total animal counts. Regional production and conservation reports use spherical cell areas. Runs concurrent with tests are diagnostics, not controlled performance benchmarks.

The controlled checks cover identical producers sharing one production budget, alternative aquatic food supporting consumers with conserved nutrients, catalog diet errors, and the existing darkness, phosphorus, chemistry, lake exchange, migration and checkpoint fixtures.

### Recorded results

Seeds 17, 81 and 256 completed all three controls for both 10 and 100 ecological years. All nine century runs retained finite, nonnegative inventories and passed the configured conservation tolerance. The largest relative C/N/P residual was 2.411e-5 (0.002411%); the largest water residual was 1.035e-8. These are numerical residuals, not exact conservation.

Century-end mixed communities with competition and flexible diets:

| Seed | Canopy | Understory | Root mat | Shallow underground | Deep underground | Aquatic |
|---|---:|---:|---:|---:|---:|---:|
| 17 | 1765 | 156 | 95 | 480 | 156 | 172 |
| 81 | 1796 | 196 | 100 | 532 | 200 | 182 |
| 256 | 1796 | 165 | 108 | 360 | 114 | 161 |

Many initially mixed communities became dominated by one producer; persistent coexistence was concentrated in canopy and subterranean communities. This is a selection response, not evidence of speciation.

The food-web result needs further calibration. At ten years, alternate foods supported more aquatic consumers. By a century, aquatic predators dominated: their unweighted mean carbon was 0.0087–0.0092 kg/m² versus roughly 3.9e-7 with fixed prey, while migratory animal carbon fell from roughly 0.0005 to roughly 5e-8. The broad predator diet includes direct plankton access with guild-wide assimilation, so it can bypass a trophic level and maintain pressure on scarce animal prey. Conserved inventories alone do not establish a balanced food web. Food-specific digestibility and better river/lake refuges should precede claims of calibrated aquatic size tiers.

Validation covered 105 distinct tests, including GPU tests, across suite runs and targeted reruns after fixing an invalid synthetic catalog and the lake iteration allowance. An existing version-four history also matched uninterrupted advancement after 24 months, including a checkpoint at month 12: history records and GPU fields were identical. Clippy and formatting checks passed.

Changed vegetation exposed the lake solver's previous 4,096-step ceiling during the long geography fixture. Its bounded allowance is now 16,384 steps, with the same convergence criterion and an explicit error if unresolved; it still exits early when converged. The previously failing long geography test passes with this allowance.

## Aquatic feeding revision

The next pass removes the aquatic predator's direct photosynthetic plankton food. Its diet is 80% preference for aquatic grazers and 20% for the accessible component of migratory animals. These are regional preference weights, not mandatory intake percentages or explicit age classes.

Each diet entry now has an optional `efficiency` multiplier (0–1) on the guild's carbon assimilation. Defaults preserve earlier catalogs. Sediment gives aquatic grazers 20% of their normal assimilation, and migratory omnivores 15%. Unassimilated material still follows the existing waste/respiration ledger; no C/N/P is discarded. The multiplier represents usable carbon energy; nutrient retention remains constrained by consumer stoichiometry.

Guilds also accept `food_half_saturation`, in weighted accessible kg C/m². With food availability F and this setting K, intake demand is multiplied by F/(F+K). Zero preserves the former behavior. The bundled aquatic guilds and waterbirds use positive settings: scarce food reduces capture, while maintenance and mortality continue. Preferences still distribute feeding according to actual local food stocks. This is a bounded first-pass functional response, not an individual hunt simulation.

Both additions fit the existing GPU catalog entry: four efficiencies are packed at 16-bit precision, and the unused guild parameter holds half-saturation. Ecological buffer size and archive version do not change. Archived catalogs without these fields keep efficiency 1 and K=0. Existing worlds retain their archived diets; new bundled defaults apply to new worlds.

Run `cargo run --release --example foodweb_evaluate -- 1200 aquatic` for matched previous/revised-diet controls on seeds 17, 81 and 256. Output is `output/foodweb/aquatic-diets-1200.json`; the earlier reports are preserved. Habitat refuges, explicit animal plankton, prey size classes and life-stage diets remain future work.

### Revised-diet century results

All six paired runs completed (seeds 17, 81, 256; 100 ecological years after initialization). Maximum relative C/N/P residual was 2.677e-5 (0.002677%); maximum water residual was 1.041e-8. The revised diets stopped the subsidized predator dominance, but the first encounter thresholds are too restrictive to maintain substantial predators or waterbirds.

Unweighted cell-mean carbon, kg C/m², previous → revised:

| Seed | Aquatic grazers | Aquatic predators | Migratory animals |
|---|---:|---:|---:|
| 17 | 2.43e-8 → 1.89e-6 | 9.21e-3 → 8.20e-9 | 4.78e-8 → 6.33e-7 |
| 81 | 2.43e-8 → 1.73e-6 | 8.73e-3 → 7.53e-9 | 4.70e-8 → 6.11e-7 |
| 256 | 2.42e-8 → 1.78e-6 | 9.07e-3 → 7.76e-9 | 5.33e-8 → 6.20e-7 |

Waterbirds also became negligible (roughly 1.4e-11 kg C/m²). Thus prey recovery and removal of the artificial subsidy are established, but balanced predator persistence is not. These low-resolution, regionally averaged results do not demonstrate healthy river populations. A subsequent calibration should lower encounter thresholds in suitable feeding habitats and add refuges/size-specific access, while retaining the removed plankton subsidy. The paired experiment changes diet and encounter settings together and does not isolate their individual contributions.

Validation for this revision: 16 ecology tests, 20 library tests and 13 managed-economy tests passed, including GPU feeding-efficiency/scarcity fixtures, conservation and checkpoint continuation. Clippy with warnings denied, formatting, and diff whitespace checks passed. Runs shared GPU time with tests; their wall times are not controlled benchmarks.

## Encounter-threshold calibration

A two-candidate seed-17 century comparison tested predator/waterbird half-saturation at 5e-6 and 5e-7 weighted kg C/m², against the first pass's 5e-4. All other feeding settings remained unchanged. The bundled default is now **5e-6** for these two guilds; grazer and migratory-animal thresholds remain 1e-3.

| Seed-17 century mean, kg C/m² | First pass (5e-4) | Selected (5e-6) | More permissive (5e-7) |
|---|---:|---:|---:|
| Aquatic grazers | 1.89e-6 | 2.82e-7 | 3.77e-8 |
| Aquatic predators | 8.20e-9 | 1.71e-7 | 2.41e-7 |
| Migratory animals | 6.33e-7 | 4.04e-7 | 1.12e-7 |
| Waterbirds | 1.43e-11 | 8.76e-10 | 2.41e-9 |

These are unweighted cell means. The selected setting restores predator feeding while preserving more prey than the more permissive candidate. Waterbirds improve but remain very sparse. This is a modest calibration, not a claim of realistic population densities or stable long-term equilibrium. Predators still cannot consume photosynthetic plankton; food efficiencies and scarcity-dependent capture remain active.

Reproduction: `cargo run --release --example foodweb_evaluate -- 1200 tuning 17` compares the two candidates. `cargo run --release --example foodweb_evaluate -- 1200 tuned` evaluates the bundled defaults over seeds 17, 81 and 256. Reports use separate `aquatic-tuning-1200.json` and `aquatic-tuned-1200.json` files and record both thresholds. The optional third argument selects a single seed.

The selected defaults completed all three century runs with finite, nonnegative state. Aquatic predator means were 1.67–1.71e-7 kg C/m², grazers 2.75–2.82e-7, migratory animals 3.93–4.04e-7, and waterbirds 8.39–8.84e-10. Maximum relative C/N/P residual was 2.574e-5 (0.002574%). All 16 ecology tests passed, including continuation and conservation fixtures; Clippy and formatting checks passed. The seed-17 bundled run reproduced the candidate result exactly.


Animal occupancy and movement now use [explicit founders and fine-edge habitat connectivity](wildlife-assembly.md). The ancestry tracer introduced there is not yet producer dispersal, species identity or ecotype evolution. Earlier calibration measurements above predate removal of spontaneous animal recruitment.
