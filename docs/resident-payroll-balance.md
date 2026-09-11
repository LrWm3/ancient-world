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
The original ensemble continues using its already-built debug executable; new
controls use the release executable, keeping the original conditions intact.
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
and full common-food access) are running sequentially for seed 17; results are
pending and the population-balance gate remains open.

Each uses `target/release/examples/cultural_work_calibrate --seeds 17 --years 100
--individual-demography --workshop-refinement --compare-resolution --output
output/<condition>.json`. Add no flag for resident payroll,
`--legacy-resident-payroll` for its control, that flag plus
`--no-individual-nutrition` for age-band exposure, or that flag plus
`--common-share 1` for the common-food control. The original debug ensemble remains
unchanged; these release runs are not used as timing comparisons.
