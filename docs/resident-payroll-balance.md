# Resident payroll and century population decline

The monthly-observed broad balance suite revealed a major unresolved difference.
At terrain edge 32, ecology edge 16, one geological epoch, yield scale 0.5 and
living history, all five individual runs nearly depopulated within a century.
Aggregate histories retained more residents. Both modes enabled the founding food
transition and the wider political, cultural, economic and expedition systems.

| Seed | Individual residents at year 100 | Aggregate residents | Individual physical / access shortfall (% of food need) | Aggregate physical / access shortfall (%) |
|---|---:|---:|---:|---:|
| 17 | 57 | 1,247.7 | 0.039 / 5.811 | 0.015 / 2.940 |
| 81 | 39 | 1,148.5 | 0.018 / 5.847 | 0.000 / 3.066 |
| 256 | 47 | 1,382.4 | 0.002 / 5.631 | 0.000 / 2.757 |
| 409 | 26 | 1,331.9 | 0.010 / 5.929 | 0.000 / 2.835 |
| 1024 | 47 | 1,295.4 | 0.089 / 5.729 | 0.037 / 2.851 |

Individual population residuals were zero; maximum absolute normalized food
residuals ranged from 2.12e-7 to 5.62e-7. Food shortfalls are sums of monthly,
local shortfalls, not annual global supply deficits. Clean ledgers do not make
this a satisfactory balance result. The mode comparison changes both demography
and workshop participation, so it does not isolate a cause on its own.

## A verified income connection

Inspection found municipal payroll distributing wages across historical household
accounts, including empty retained estates. As rosters age and estates become
vacant, these accounts can siphon wages from living households. Their ownership
and stored cash are historical state, but they are not resident employees.

New household economies enable `resident_payroll`. With complete individual
rosters, only accounts with observed living resident members share municipal
payroll. Sparse histories exclude marked vacant accounts unless known resident
members remain. A household with children but no adult representative is not
silently treated as empty. Empty estates retain their cash, property and existing
dividend rules; this change removes their wage allocation, not their identity.
Rounding residue goes to the last eligible recipient. With no eligible recipients,
the municipal payroll stays in the treasury.

This is residency-based payroll eligibility, not the unfinished full worker
assignment conversion. Occupied households still receive aggregate sector pay;
adult availability, actual individual sector assignments and dependent support
need the broader production/income work in the integration list. No mortality,
fertility, crop yield, common-food share or price rule was tuned in this fix.
Older archives default to the legacy allocation; `resident_payroll` can be changed
at a completed monthly boundary for a declared comparison.

## Controlled follow-up

The cultural-work runner now exposes `--legacy-resident-payroll`,
`--no-individual-nutrition`, and `--common-share`. These isolate the payroll bug,
personal exposure and food access without changing crop production directly.
The original ensemble used its already-built debug executable; new controls
use the release executable, keeping the original conditions intact.
Reports record the selected switches. Controls and their results must be evaluated
before attributing the century collapse to this one income defect.

Seed 17's same-state demographic receipts provide a useful further discriminator:
individual births total 2,334 against an aggregate expectation of 2,312.3, and
deaths total 4,194 against 4,172.0 (military losses are separate). Childhood-to-adult
transitions total 1,375 against 1,736.3 expected by the fractional cohort model.
These sums compare each realized state's own monthly projection, not a parallel
world's total. The evidence points toward persistent exposure and age/survival
structure, not a large stochastic birth/death counting bias. It does not yet prove
which economic intervention will restore viable long histories.

Verification: all ten household-economy tests passed with hardware fixtures enabled,
including the retained-estate negative control, no-recipient treasury case,
legacy-field loading and the existing continuation/accounting fixtures. All-target
Clippy with warnings denied, formatting and the repository artifact check passed.
The four century controls (resident payroll, legacy payroll, age-band exposure,
and full common-food access) run sequentially for seed 17. Results below supersede
the initial pending status; the population-balance gate remains open.

Each uses `target/release/examples/cultural_work_calibrate --seeds 17 --years 100
--individual-demography --workshop-refinement --compare-resolution --output
output/<condition>.json`. Add no flag for resident payroll,
`--legacy-resident-payroll` for its control, that flag plus
`--no-individual-nutrition` for age-band exposure, or that flag plus
`--common-share 1` for the common-food control. The original debug ensemble remains
unchanged; these release runs are not used as timing comparisons.

## First matched control results

The seed-17 release legacy-payroll control reproduced the earlier debug baseline's
57 residents and cumulative food-gap totals. Resident payroll retained 101 people
at year 100. Physical shortfall was 0.0417% versus 0.0391% of need; access shortfall
was 5.6485% versus 5.8108%. This confirms a beneficial contribution in this seed,
not a resolution of the population collapse. Neither comparison changed crop
production or mortality parameters. Further exposure and common-food controls
remain necessary; do not infer a cross-seed effect from this single matched pair.

The original scarcity ensemble is **stopped**, not still running: seed 17 completed
100 years, then seed 81 failed at month 740 with `invalid civilization leadership`.
The static-founding condition was never started. Completed seed results remain in
the partial report. A focused reproduction uses seed 81, yield 0.33, individual
demography/workshops and legacy resident payroll; the validator now identifies the
failing civilization, leader, affiliation, death month and estate vacancy.

Removing individual nutrition while retaining legacy payroll produced **zero**
residents by year 90 (18 at year 70, one at year 80). Cumulative access shortfall
rose to 9.7104%, physical shortfall to 0.0482%; population residual remained zero
and maximum normalized food residual was 3.09e-7. This ablation removes both
household-specific mortality exposure and the next-month hunger/work penalty;
it does not isolate those two channels individually. It rules out treating pooled
age-band exposure as a sufficient fix. Persistent household differences protect
some families from shortages that the pooled control spreads across the age band.
That mechanism is an interpretation, not yet an isolated causal result.

## Succession away from the home polity

The focused scarcity reproduction failed at exactly month 740: civilization 10's
leader 1306 died that month, with no retained vacant estate satisfying validation.
Inspection separately exposed a territorial-identity bug: household succession
looked up the ruler using the site's current civilization. Moving the household to another civilization's settlement could therefore
change which office it was considered to hold and which successors were eligible.

The regression places a ruler's household in another civilization's settlement
and gives the nearest heir a foreign affiliation, while retaining an eligible
household member of the ruler's own civilization. It failed before the correction. Succession now follows the
ruler's own polity, preserves both people's affiliations, leaves the host civilization's
ruler unchanged and creates no people. All seven resident-registry GPU fixtures
pass, including vacancy recovery and checkpoint continuation. The century rerun
must still establish whether this correction also resolves the seed-81 failure;
the narrower fixture alone does not prove that connection.

## Common-food control stopped at daughter founding

Full common food with legacy payroll passed 50 years, then failed at month 673.
Site 6's opening resident roster contained `[110, 160, 46]` people by age band,
while its cohorts had become `[88, 128, 36.8]`. The preceding month recorded a
`migration` event: the legacy daughter-settlement path deducted a 20% fractional
cohort without relocating the corresponding individuals. This control has **no
century outcome** and cannot be presented as proof that common access fixes balance.

The next population-authority connection is therefore actual resident/household
transfer during daughter founding, including the existing finite provisions and
cash transfer. Disabling founding or relaxing roster validation would hide that gap.
The runner now preserves failed history diagnostics beside its output, with the
seed, attempted month, error and completed decadal samples. These are marked
`diagnostic_only`: GPU ecology may have advanced while the history transaction
rolled back, so they are not resumable world checkpoints. Raw files stay ignored.

Follow-up checks: all seven resident-registry GPU fixtures passed after the
relocated-household regression; all-target Clippy with warnings denied, formatting,
Git whitespace checks and repository artifact policy passed. The focused scarcity
rerun is in progress; its result will update this record rather than silently
replacing the failed baseline.

The corrected seed-81 scarcity run completed 100 years with 23 residents, zero
population residual and maximum normalized food residual 6.45e-7. This verifies
that the displaced-ruler succession correction resolves the observed month-740
failure, while the scarcity population outcome remains poor. Daughter founding
is addressed separately in [resident daughter settlements](resident-daughter-founding.md).

The runner's opt-in `--household-diagnostics` adds account-level observations to
its decadal samples: dated food site, need, common/purchased food, hunger, cash,
current sector wages and cumulative wages, relief, dividends and food spending.
These are closing observations, not a sum of monthly cash balances. A three-seed
30-year run (17, 81, 256; resident payroll, individual demography/workshops, yield
0.5) is collecting them to distinguish distributional access from total production.
Raw account rows remain ignored; conclusions will be summarized here.

## Household distribution at year 30

The three-seed diagnostic completed. Each row describes the **single closing
month at year 30**, not the fraction of families ever hungry. Hungry here means
more than 10% of that household's food need unmet. Only accounts with positive
observed food need are included; currency is the game's abstract unit.

| Seed | Population | Hungry / resident accounts | Hungry share of food need | Hungry share of current sector wages | Hungry cash | All resident cash |
|---|---:|---:|---:|---:|---:|---:|
| 17 | 1486 | 61 / 240 | 31.26% | 23.85% | 0.0059 | 109818.0 |
| 81 | 1506 | 63 / 240 | 31.21% | 25.06% | 0.0077 | 112207.4 |
| 256 | 1683 | 59 / 240 | 29.57% | 23.24% | 0.0069 | 106287.2 |

The similar distribution across these seeds points toward household income access.
It is an association, not an isolated wage-policy effect: local output, available
work, household composition and prior saving all contribute. In seed 17, the top
food-need quartile averaged 127.2 kg/month and 11.5% hunger; the lowest averaged
59.3 kg and 2.2%. Their current aggregate sector wages differed by only 1.37 times
while food needs differed by 2.15 times. Municipal payroll is still allocated
largely by account/occupation proxies, not the number of actual contributing
workers and dependents. The unfinished production/income integration should be
evaluated against this pattern; the results do not justify an unconditional
food-yield increase or claiming that current wages are individual earnings.

Reproduction: `target/release/examples/cultural_work_calibrate --seeds 17,81,256
--years 30 --individual-demography --workshop-refinement --compare-resolution
--household-diagnostics --output output/household-distribution.json`.
All-target Clippy and the source-only artifact check passed for this diagnostic
increment. It changes observations, not model decisions.
