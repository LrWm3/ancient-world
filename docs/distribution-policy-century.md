# Political distribution: matched century comparison

This experiment asks whether politically changing household distribution improves
population retention and the late trajectory. It does not change production,
mortality coefficients or the faction platforms to obtain a preferred result.

## Setup

Original simulation source: `789a419`; treatment replay uses the validation-only correction in `6d59482` (described below). Two paired seeds, 256 and 409, each run for 100 years
of living history at terrain/ecology 32/16, one epoch, sixteen founders and crop
yield scale 0.5. Individual demography and workshop/agriculture/extraction/
construction refinement, comparison receipts, monthly household diagnostics and
family support are enabled in both arms. Other runner defaults match.

The control disables only new political distribution proposals from founding.
Elections, taxes, family gifts and council relief remain active. The treatment uses
the new annual distribution decisions, effective next month. It is therefore an
ablation of one connection, not an experiment disabling all political feedback.

```sh
cargo build --example cultural_work_calibrate
target/debug/examples/cultural_work_calibrate \
  --seeds 256,409 --years 100 --resolution 32 --crop-yield-scale 0.5 \
  --individual-demography --workshop-refinement --agriculture-refinement \
  --extraction-refinement --construction-refinement --compare-resolution \
  --household-diagnostics --family-support --fixed-distribution \
  --output output/distribution-century-fixed.json
```

Repeat without `--fixed-distribution`, writing
`output/distribution-century-political.json`. Compare with:

```sh
python3 scripts/compare_food_access.py output/distribution-century-fixed.json \
  output/distribution-century-political.json --allow-difference political_distribution
```

Reports retain cumulative food demand, physical shortage, purchasing-access gap,
monthly conservation checks, decadal population and fiscal stocks, and council
policy snapshots. The original arms execute concurrently on the Quadro RTX 5000/Vulkan;
the treatment replay runs after the control finishes. Timings include differing contention and are not an isolated performance comparison. Raw
reports and logs remain ignored under `output/`.

## Interrupted run and audit correction

The first treatment attempt completed seed 256 but stopped seed 409 at month
1122 (93.5 years) with an invalid agriculture-plan error. Its report was marked
incomplete, and its failure history and completed decadal samples were retained
locally; it was not counted as a completed century.

Inspection found a numerical weakness in the grant audit. At a request of
106.666664 worker-months, 121 equal f32 grants sum to 106.666801 when reduced in
f32, but their stored contributions sum to 106.666665 in double precision. The
former falsely exceeds the existing 0.0001 worker-month tolerance. A regression
fixture reproduces that rejection and separately verifies that a real additional
0.001 worker-month and a NaN are still rejected.

Commit `6d59482` audits stored contributions in double precision while preserving
allocation, GPU inputs, production and demographic arithmetic. Error messages now
include request and grant totals. The runner counts cases where the old f32 audit
would reject an otherwise valid plan. Both treatment seeds are replayed from
founding with this correction. Seed 256's entire per-seed report matches its
original completed run after excluding runtime and the newly added audit counter.
The fixed-policy controls completed under the original audit, so their simulation
results do not require a changed allocation rule to remain valid.

The replay passes the original month-1122 failure: site 12 requested about
105.38615 worker-months; its f32 reduction was 105.38627, while the stored grants
summed to 105.3861945271492, within the unchanged tolerance. This is the only
recorded legacy-audit false positive in the completed treatment ensemble.
Seed 409's decadal samples through year 90 also match the interrupted attempt.

## Completed population and food results

Both reports are complete and the strict comparator acknowledges only
`political_distribution` as a configuration difference. Both fixed-policy
population and food trajectories reproduce the earlier family-support century
runs exactly. Family support remains enabled in every arm here.

| Seed | Year-100 population fixed → political | Population change, years 90–100 | Cumulative purchasing gap, % of need | Purchasing gap, years 90–100 |
| --- | ---: | ---: | ---: | ---: |
| 256 | 1,338 → 2,511 | −24 → +147 | 2.9343 → 2.0209% | 2.9330 → 2.0639% |
| 409 | 1,322 → 2,647 | −40 → +71 | 3.0461 → 2.0056% | 3.2622 → 2.1442% |

Physical food shortage is zero in all four histories. All sixteen sites remain
active. The intervention improves purchasing access and reverses the measured
late decline in both seeds without increasing crop-yield settings or reducing
mortality coefficients. Endogenous production, births, deaths, political choices
and spending can diverge after the intervention; these are not frozen mediators.

| Year | Seed 256 fixed | Seed 256 political | Seed 409 fixed | Seed 409 political |
| --- | ---: | ---: | ---: | ---: |
| 10 | 2,051 | 2,109 | 2,024 | 2,099 |
| 20 | 1,934 | 2,145 | 1,926 | 2,113 |
| 30 | 1,797 | 2,162 | 1,815 | 2,125 |
| 40 | 1,662 | 2,134 | 1,700 | 2,121 |
| 50 | 1,555 | 2,128 | 1,624 | 2,213 |
| 60 | 1,465 | 2,154 | 1,558 | 2,270 |
| 70 | 1,429 | 2,236 | 1,471 | 2,384 |
| 80 | 1,411 | 2,313 | 1,419 | 2,484 |
| 90 | 1,362 | 2,364 | 1,362 | 2,576 |
| 100 | 1,338 | 2,511 | 1,322 | 2,647 |

## What changed and what it costs

Fixed councils retain common coverage 50%, payroll 20%, dividends 1%, relief
spending 5% and target coverage 75%. At year 100, effective treatment policies
span these ranges across each seed's councils (not population-weighted means):

| Seed | Common coverage | Payroll | Dividends | Relief share | Food target | Activations |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 256 | 60.73–69.57% | 20% | 0.35–0.50% | 10.39–14.44% | 95.03–99.57% | 1,579 |
| 409 | 51.38–67.26% | 20–25% | 0.42–0.96% | 6.74–13.20% | 86.38–97.26% | 1,578 |

The fixed arm has zero distribution activations. The mechanism is active, rather
than just a changed label: public entitlement and targeted purchasing assistance
increase, while ownership dividends generally decline.

| Seed | Household council relief, lifetime | Family gifts received, lifetime | Council treasury, year 100 | Town operating cash, year 100 |
| --- | ---: | ---: | ---: | ---: |
| 256 | 15,550 → 47,751 | 246,843 → 161,132 | 11,322 → 1,217 | 41,780 → 52,919 |
| 409 | 17,369 → 45,560 | 267,682 → 154,857 | 12,515 → 2,311 | 39,638 → 52,981 |

These are abstract currency amounts. Household relief is taken from account
receipts, not the broader council relief total, which includes other channels.
Lifetime spending is not normalized for the larger population. Lower family
transfers are consistent with less unmet purchasing demand, not evidence that
family ties became weaker.

Council reserves fall roughly 89% and 82%. Town operating cash rises, and combined
council-plus-town cash rises slightly; funds have different locations and spending
permissions, so this is not a claim of general public insolvency. The government
may still have less cash immediately accessible to council-funded responses.
Active institutions rise from 35 to 50 and 35 to 54, but institution counts alone
do not establish service readiness or adequate upkeep.

## Verification and next decision

Maximum monthly population residual is zero; maximum monthly food residual is
below 1.2e-6. Final economic ledger residuals are below 1.3e-5 in their respective
native units, and there are no unresolved resident/cohort discrepancies. No
people are away in military, relocation or expedition compartments at these
endpoints, and these runs contain no expedition voyages or military service.
They are not a test of fiscal resilience during major campaigns.

The ordinary library suite passes 134 tests (114 hardware/long fixtures remain
ignored in that command); focused GPU policy tests, four comparison-script tests,
Clippy across all targets and repository artifact checks pass.

Keep the political-policy defaults for now. The result is substantially better
than fixed distribution in these two scenarios, but persistent late purchasing
gaps around 2% and thin council reserves remain. Two seeds at resolution 32/16 and
a century horizon do not establish a universal equilibrium. The [council-funding follow-up](council-funding-balance.md) now measures actual
requests: century-wide administrative coverage improves in both seeds, but late
coverage worsens in one. No petition closes for lack of council funds. Keep the
population gains while investigating local tax collection and comparing explicit
basic-service funding against household assistance; do not target a larger idle
treasury as an end in itself. These observations do not retune the policy targets.
