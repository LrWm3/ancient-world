# Natural-stock fishery counterfactuals

Follow-up: [adaptive fisheries](adaptive-fisheries.md) adds an opt-in crew and
equipment policy. The runs below retain legacy fishing. Their archived calorie
fractions use the nominal fish conversion of 0.5; the updated analyzer also applies
the production shader’s C/N/P constraint, giving 0.444444 for this catalog. Thus
these historical calorie fractions are upper bounds (multiply by 8/9). Catch,
population and budget measurements are unaffected.

The living-history intervention boundary now has a longer, naturally initialized
experiment. This extends the [single-source analytical fixture](living-scenarios.md)
without replacing it. No animal inventories, prey ratios, migration rates or
fishing coefficients were changed for these runs.

## Protocol

For each seed (17, 81, 256), generate one geological epoch at terrain resolution 64
and ecology resolution 16, using the bundled catalog. The epoch includes one year
of ecology; then advance another 120 ecological months. Found sixteen groups,
enable society and living history, and advance twelve history months to establish
household observations and receiving-water addresses. Save a common checkpoint.
The ecological warm-up is initialization, not a claim of ecological equilibrium.

Advance six checkpoint branches for 120 further monthly steps:

- **Baseline:** no intervention.
- **Sham:** restore the already-enabled guilds 8, 9 and 10 in the great lake.
  This appends the same number of intervention records without changing their stocks.
- **Removed:** remove those three catchable guilds from great-lake cells, retaining
  their C/N/P as aquatic organic matter and suppressing recruitment there.
- **Restored:** the same removal, followed by lifting suppression after month 60.
  Surviving populations must supply recolonization; restoration creates no biomass.
- **Closed:** disable managed fishing while wildlife continues evolving.
- **Removed + closed:** combine the removal with the harvest ablation.

The great-lake region selector uses the dominant area fraction of each ecological
cell. Mixed coastal cells assigned to land can retain aquatic populations and
continue supporting catch. Thus this regional experiment deliberately differs
from global removal of all catchable animals. Surviving biomass in a lake-weighted
regional report is not necessarily inside a treated cell.

Monthly records contain each town's catch increment, food reserve, demographic
ration need and intake, fish price, population, abandonment, household shortage
observations and receiving-cell guild carbon density. Annual records add regional
wildlife inventories and ecological ledgers. All monthly ecological snapshots are
checked for valid inventories; economic residuals are checked monthly and ecological
budgets annually. End-of-branch archives are saved locally.

## Ten-year results

| Seed | Baseline catch kg | Removed catch kg | Catch reduction | Restored catch kg | Baseline catch calorie equivalent / ration need |
|---|---:|---:|---:|---:|---:|
| 17 | 1,890.68 | 392.10 | 79.3% | 1,127.61 | 0.0262% |
| 81 | 2,159.74 | 383.40 | 82.2% | 1,261.45 | 0.0301% |
| 256 | 1,911.33 | 129.72 | 93.2% | 1,009.01 | 0.0267% |

All three sham branches reproduce baseline town and annual wildlife trajectories
exactly. Restoration branches match removal branches until restoration. Both closed
branches catch zero fish. These checks run in the analysis script, not just by visual
inspection of end states.

**Ecological recovery and economic recovery are different.** In the last year,
restored catches reach approximately 100% of baseline in all three seeds. Yet the
combined lake-associated carbon inventory of those guilds is only 26.8%, 18.9%
and 21.9% of baseline respectively. Enough prey has returned to saturate the tiny
fishery labor allowance; the underlying food web has not returned to baseline.

**Food and population effects are weak and not monotonic.** Baseline aggregate unmet
rations range from 1.18% to 1.93%; severe household-shortage observations range from
0.73% to 1.37%. Removing lake guilds changes final population by less than one person
in each seed. Food reserves can end higher or lower after removal. Different
consumption and production trajectories mean final stored food is not a cumulative
measure of the removed fish contribution.

Closing harvest is also not equivalent to deleting fish: it releases the fishery
labor reservation. Its final population effect ranges from approximately -4.32 to
+10.00 people. Do not attribute that entire effect to nutritional intake alone.
Closed and removed-plus-closed town trajectories match in this ensemble, even though
their ecological stocks differ.

These results establish a strong wildlife-to-catch connection but a very weak
fishery-to-diet contribution under current parameters. They do not support describing
these settlements as economically dependent on aquatic ecosystems.

## Fifty-year pressure follow-up

A separate exploratory run uses seed 17, crop-yield scale 0.33, and 600 history
months per branch. Restoration happens after month 300. All other initialization
and branch rules match the ten-year protocol. This one-seed stress case is not a
held-out calibration or a factorial estimate separating duration from crop yield.

The baseline catches 5,845.76 kg over fifty years, equivalent to only 0.0251% of
ration need. Aggregate unmet rations are 5.19%, and 7.55% of observed household-months
have severe shortages. Final population is 1,016.41. Permanent removal reduces
catch to 672.38 kg (88.5% lower), with final population 1,008.79. The restoration
branch catches 2,975.43 kg and ends at 1,006.80 people. No town is abandoned in these
three branches.

Restoration recovers catch capacity but does not rewind the intervening history.
Its final-year catch reaches 100.9% of baseline while its lake-associated stock is
about 81% of baseline, with a different guild composition. Population and food
access are not monotonic functions of cumulative fish catch. The stress case
therefore reinforces the small dietary role of present fishing; it does not
establish that fisheries cause or prevent settlement collapse. Both closed-harvest
branches catch zero and finish at 1,030.03 people; the sham again matches baseline
exactly. Releasing labor and later historical feedback prevent interpreting closure
as a pure food-removal experiment.

All 24 branches completed 5,760 measured monthly advances. The sixteen scripted
control checks passed. Maximum absolute relative ledger residuals were:

| Suite | Economic ledger | Ecological C/N/P | Ecological water |
|---|---:|---:|---:|
| Ten-year ensemble | 2.03e-6 | 2.65e-6 | 1.60e-6 |
| Fifty-year stress case | 1.30e-5 | 8.94e-6 | 6.48e-6 |

These are whole-system residuals; the earlier cell-level transfer fixtures remain
necessary because global totals can conceal local errors. Clippy, formatting and
48 ordinary tests passed. The experiment itself exercised the GPU on every month;
other ignored GPU suites were not rerun because production kernels were unchanged.

## Reproduce

```sh
mise exec rust@1.89.0 -- cargo run --example living_fishery_evaluate -- 120
python3 scripts/analyze_living_fisheries.py output/living-fishery-natural
FISHERY_OUTPUT=output/living-fishery-harsh mise exec rust@1.89.0 -- cargo run --example living_fishery_evaluate -- 600 17 16 0.33
python3 scripts/analyze_living_fisheries.py output/living-fishery-harsh
```

Arguments are history months, optional single seed, ecology resolution and crop-yield
scale. History months must be a multiple of 24; restoration happens halfway through.
`FISHERY_OUTPUT` selects the output directory. Use a fresh directory for a different
configuration. The analyzer rejects incomplete or duplicate branches and reordered
monthly trajectories. It accepts gzip-compressed trajectories as well as raw JSONL.
The calorie conversion comes from the archived fish catalog, not a fitted coefficient.

The [ten-year evidence](evidence/living-fishery-natural/) and
[fifty-year pressure follow-up](evidence/living-fishery-harsh/) contain all
monthly records, catalogs, summaries, execution logs and source hashes. The diagnostic
grid and one GPU backend constrain the claim: this is not a resolution study,
held-out empirical calibration or proof of equilibrium.

## Implications for the next implementation

The fishery currently reserves 0.1% of available workers at eligible sites, with a
catch labor cap of `workers × 0.02` kg/month. Stock availability further limits catch;
large river animals and predators are less catchable. Increasing wildlife alone
cannot make fishing a major livelihood once this cap binds.

A useful next extension is bounded, demand-driven fishing labor with maintained
boats/gear and access costs. It should compete with agriculture and crafts, make
catch sensitive to local stock density and effort, and retain the existing finite
C/N/P withdrawals. Calibration should compare per-worker catch, dietary contribution
and depletion/recovery trajectories, rather than raising animal biomass until towns
survive. No such coefficients or production behavior were changed in this experiment.
