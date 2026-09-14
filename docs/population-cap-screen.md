# Tenfold settlement and named-population caps

The settlement capacity is now 2,560 (previously 256); the shared named-person
capacity is 500,000 (previously 50,000). Settlement admission, daughter founding,
GPU storage allocation and labor forecast validation use the same owning constant.
The GPU engine checks the enlarged history buffers against individual binding
limits and the existing memory budget before allocation.

These are admission/storage limits, not target populations. There is no comparable
hard maximum population per town in the aggregate demographic equations. The
90-person maximum daughter founding party remains a transfer size, not a town cap.
Initial population, birth/mortality parameters, land, food, money and founding
requirements are unchanged.

Candidate sampling was capped at 2,048 in the comparison below. It now shares the
2,560-settlement ceiling; geography can still supply fewer suitable sites. Existing
worlds retain their saved candidate lists rather than gaining new locations on load.
The comparison below predates this sampling change. This is not a demonstrated capacity or
performance benchmark at 2,560 inhabited settlements or 500,000 residents. Archive
size limits and other sparse-record limits also remain independent.

## Matched experiment

Old executable: bc927bb. New executable differs only in these caps, the shared
constant name and GPU allocation checks. Seeds 1024, 256 and 409 start from the
same stored founding worlds: five civilizations, 600 total people, terrain/ecology
32/32, frozen environment. Advance 100 years with the previous council-welfare,
progressive-tax, practical-research and household-clothing experiment enabled;
credit and minting remain disabled. Commands otherwise match
[council welfare reserves](council-welfare-reserves.md).

These archives use aggregate authoritative demography with sparse named people,
not complete individual rosters. Raising the registry cap tests that existing
path; it does not silently convert population authority or add initial residents.

## Population and the binding mechanisms

| Seed | Initial | Year 50 | Year 100 | Recorded sites | Named records at year 100 | Cumulative births | Cumulative deaths |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | 600 | 150.75 | 117.50 | 5 | 343 | 505.20 | 987.69 |
| 256 | 600 | 364.04 | 230.67 | 5 | 434 | 931.38 | 1,300.72 |
| 409 | 600 | 348.21 | 241.23 | 5 | 416 | 886.90 | 1,245.67 |

Year 50 values are from the preceding welfare experiment, with identical settings.
The population ledger is approximately initial + births - deaths; settlement
migration redistributes population rather than adding global population.

The old limits were not binding. All worlds retain only the five original site
records, and hundreds rather than tens of thousands of named records. Suitable
sampled candidate counts are 34, 37 and 44. At year 100 no surviving town even
reaches the preliminary 160-person threshold for daughter founding.

Daughter founding additionally requires twelve months of provisions, an unused
same-island candidate within 600 km, and a population threshold between 220 and
400 depending on relative crop potential. Those are colonization feasibility
rules, not the settlement-count cap. Lowering them is a different experiment.

Food access remains consequential even with reserves deployed. At year 100, seed
256's largest town has 153.93 people and 35,974 kg of aggregate food stock. Applying
the shader's demographic rates to its ending age mix and ration shortfalls gives
approximately, per month:

- Births: 0.303.
- Ordinary mortality: 0.171.
- Additional hunger mortality: 0.300.
- Additional illness mortality: 0.043.

This is an endpoint diagnostic using closing ages, not a replay of exact opening
stocks or attribution of every historical death. It shows the immediate mechanism:
ration deprivation can outweigh births despite local food stocks. Hunger also
suppresses births and increases illness. Adequate town-level food inventory is not
equivalent to adequate household entitlement and purchasing power.

Other sites genuinely run out of food and become abandoned. At year 100, four
sites in seed 1024 are abandoned; two each in seeds 256 and 409. Ordinary aging
and mortality continue even where food access improves. Increasing storage or
identity admission limits cannot change those demographic rates.

The next useful controlled intervention is the household food-access boundary:
compare current purchasing with needs-based access to *existing local food*,
holding production, total stocks and cash fixed at the opening boundary.
Measure funded demand, actual rations and age-specific mortality before attempting
another whole-economy tuning change.

## Verification result

All six century-long runs completed. For each seed, the **entire parsed exported
history is exactly equal** between old and raised caps, not merely the final
population. This rules out these caps as the cause of decline in these matched
worlds. It does not establish that no larger or differently configured world
could reach a cap.

The ordinary library suite passes: 199 passed, 155 hardware/extended tests ignored.
Native GPU runs exercised the enlarged allocations; strict all-target Clippy passes.
No full-capacity stress or individual-authoritative demographic comparison is
claimed. Raw outputs and the old executable remain ignored under
`output/population-cap-screen/`; no generated binary or history is committed.

Follow-up: [food access and nutritional stress](food-access-and-nutritional-stress.md)
tests the two mechanisms independently and together. Gentler health effects improve
survival across the three seeds, while fully common food exposes a municipal
revenue tradeoff.
