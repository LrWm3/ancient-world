# Food access and accumulated nutritional stress

Two independent opt-in counterfactuals follow the
[population-cap comparison](population-cap-screen.md):

- `--needs-based-food[=true|false]`: full current dietary entitlement to finite
  town food, regardless of household cash.
- `--gradual-nutrition[=true|false]`: accumulated age-band dietary stress for
  aggregate mortality and shortage-driven illness.

Omission preserves archived settings; older histories initialize both disabled.
These are game-design experiments, not physiological or historical calibration.

## Access experiment

The existing Reserve-phase household retail preparation selects a common-food
share of one under needs-based access. This overrides the temporary founding
transition and political common-share setting for food only. Payroll, dividends,
taxation, common asset ownership and other retail keep their existing rules.

Full need is an entitlement ceiling, not a food grant. GPU production and retail
settlement consume at most actual available food. When food is insufficient,
households share realized common food proportionally to dietary need. There is no
food purchase payment for that common portion and no fabricated council subsidy.
The existing goods and C/N/P ledgers account for actual consumption once.

This is a strong ablation of household purchasing, not targeted emergency relief.
It removes food-sale revenue from towns and leaves more money in household wallets;
those economic consequences are part of the experiment.

## Health experiment

Each settlement persists three dimensionless age-band deficit memories in
`Demography.nutrition.xyz`; `w` tells the GPU whether this aggregate pilot is
enabled. Each monthly demographic pass observes actual age-band rations.

For deficit fraction h and previous memory m:

```text
tau = 6 months if h >= m, otherwise 3 months
m_next = clamp(m + (h - m) / tau, 0, 1)
mortality_stress = max(m_next², max(0, 2 × (h - 0.5)))
```

The existing hunger-mortality coefficient multiplies this stress instead of h.
Shortage-driven illness uses its dietary-need-weighted mean. Ordinary mortality,
contamination, illness recovery, aging and the birth equation are unchanged.
Births still respond to current adult ration shortfall and illness. Household
work penalties retain their existing rules.

This allows a modest temporary deficit to have a small initial mortality effect,
retains harm under chronic deprivation, and preserves acute danger above a 50%
ration deficit. Complete starvation still receives the original maximum hunger
penalty immediately. For example, persistent 5% deprivation approaches stress
0.0025 rather than 0.05. Recovery is gradual: restored food does not instantly
erase accumulated harm.

Memory is an abstract health indicator, not a store of edible calories or body
mass. Enabling an old world starts from its stored memory (zero for old archives).
Disabling freezes that memory; re-enabling resumes it. No geographical or temporal
resolution invariance is claimed.

The pilot is restricted to aggregate-authoritative demography. The CLI and
household validation reject combining it with complete individual demography.
A later individual version needs personal nutritional state and household exposure,
rather than silently substituting the town average for actual personal rations.

## Validation scope

The controlled fixture contrasts identical cash-poor households under market and
needs-based entitlement. Preparation preserves town food and total money; a 40%
food-supply fixture allocates exactly the available food without charging wallets.
The GPU memory update is compared against a separate arithmetic calculation using
observed rations, and memory/policy survive JSON round-trip. Full-world runs check
the combined GPU/Rust layout and existing conservation validation.

Raw experiment outputs belong under ignored `output/food-access-health-screen/`.

The aggregate resolution framework also reads the completed nutritional memory
when constructing its mortality projection, so enabling comparison receipts does
not silently revert to the old hunger coefficient. Frozen demographic snapshots
retain their resolved mortality rates. An analytical test checks both chronic
5% deficiency and complete starvation independently of GPU implementation.

An initial GPU fixture failed because its cash-poor setup erased the world's
initial money before running full conservation validation. The corrected fixture
uses the cash-poor clone only for the allocation comparison, and restores the
ledger-consistent founding world before GPU advancement. No production conservation
tolerance was relaxed.

## Three-seed century screen

Seeds 1024, 256 and 409; 100 years from the same stored five-civilization,
600-person founding worlds; terrain/ecology 32/32, frozen environment.
Baseline retains the previous tax, welfare-reserves, clothing and practical-research
settings. The nine new runs change only the two experimental switches.
Credit and issuance remain disabled. These are game balance observations, not
performance benchmarks or a calibration against human demographic data.

| Seed | Arm | Final population | Active sites | Endpoint need-weighted hunger | Births | Deaths | Operator completed work |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | baseline | 117.50 | 1 | 0.0167 | 505.2 | 987.7 | 22.87 |
| 1024 | food | 106.60 | 1 | 0.0000 | 505.2 | 998.5 | 6.89 |
| 1024 | health | 137.31 | 2 | 0.0259 | 541.1 | 1003.8 | 88.82 |
| 1024 | both | 115.56 | 2 | 0.0128 | 518.2 | 1002.6 | 18.08 |
| 256 | baseline | 230.67 | 3 | 0.0418 | 931.4 | 1300.7 | 4.69 |
| 256 | food | 279.16 | 3 | 0.0000 | 958.6 | 1279.4 | 16.96 |
| 256 | health | 297.47 | 4 | 0.0560 | 993.4 | 1295.9 | 8.91 |
| 256 | both | 288.80 | 3 | 0.0000 | 969.8 | 1281.0 | 25.80 |
| 409 | baseline | 241.23 | 3 | 0.0203 | 886.9 | 1245.7 | 314.17 |
| 409 | food | 238.64 | 3 | 0.0000 | 901.1 | 1262.5 | 177.86 |
| 409 | health | 257.52 | 3 | 0.0232 | 905.6 | 1248.0 | 251.43 |
| 409 | both | 251.01 | 3 | 0.0000 | 913.8 | 1262.8 | 149.32 |

## Interpretation and disposition

Health-only retains more people in every seed: about 17%, 29% and 7% above
baseline. It does not restore growth from the initial 600. Endpoint hunger is
higher in all three health-only worlds; lower mortality does not mean food access
has improved, and keeping vulnerable populations alive changes the population
over which hunger is measured. Cumulative deaths are not per-person mortality
rates: more surviving people and births also create more lifetime exposure.

Food-only removes endpoint hunger among surviving households but improves final
population only in seed 256. It abolishes all household food-sale payments.
Total town operating cash falls from approximately 4,797/9,177/10,621 to
39/5/16. Those are actual cash stocks, not merely a narrative explanation.
Operator work falls in two seeds. The cash loss is a direct consequence of removing
payments; its exact contribution to subsequent population differences is not
isolated by these long, diverging histories.

Combining the policies does not outperform health-only in any of the three seeds.
All worlds still record only five sites. More generous admission limits, gentler
mortality and full common entitlement therefore do not by themselves establish
a self-sustaining economy.

Keep both policies opt-in. The next food-access experiment should preserve a
funded producer/municipal operating path while targeting exclusion: common food
access cannot simply remove revenue and assume the remaining payroll and production
arrangements will fund themselves. Separately, test the nutrition parameters over
additional climates and earlier time windows before promoting health-only to a
default. Do not tune only for the final survivor count.

The GPU boundary test passes, as do 200 ordinary library tests (156 extended or
hardware tests ignored) and strict all-target Clippy. All nine native runs pass
existing conservation validation; independently audited endpoint money residuals
are below 6.4e-8 relative. These results do not claim full-capacity, cross-hardware,
or complete individual-demography verification.

Final-build regression: seed 1024's full century export with both switches disabled
exactly matches the previous version after removing the added default fields.
The final health-only build also exactly reproduces its earlier century export.
These two additional runs bring new native runs to eleven. The comparison is
same-backend repeatability, not a substitute for a save/resume or cross-GPU study.
