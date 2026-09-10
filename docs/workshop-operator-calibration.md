# Workshop operators: implementation and matched evaluation

The employer model now connects household ownership and wages, separate operating
cash, GPU production, service-fee payment and business closure. The final rules
are in implementation commit `2ce7ed2`; exact executable and source hashes are
retained with every evaluation suite. See the
[model and transfer specification](workshop-operators.md) for equations, units,
update order, ownership semantics, and limits.

## What changed during evaluation

The first pilot used seeds 17 and 81 for twenty years with four combinations:
equal/industry-linked wages and communal/private workshop operation. It exposed a
financing defect: a cash-surplus dividend rule could return founder capital as
“profit” while the business was losing money. Dividends now require accumulated
realized profit, recovery of prior losses, and an operating cash reserve.

The subsequent fifty-year comparison exposed another avoidable source of churn.
Founding required recent work and a solvent owner, but not enough expected fees
to pay a minimum shift and rent. The final entry rule compares the locally
observed work forecast against those costs and requires three shifts' working
capital. This is a local, backward-looking forecast: it cannot foresee subsequent
material shortages, falling orders, changing prices, or customer nonpayment.

With industry-linked wages, closures in seed 17 fell from 160 to 55 over fifty
years after the entry screen; seed 81 fell from 333 to 148. Starts also fell,
from 174 to 66 and from 352 to 163 respectively. This chiefly removes investment
in predictably weak opportunities; it is not evidence that every operating firm
became more productive. No parameter was fitted to a claimed historical survival
rate. Operator failures remain common in these harsh, changing economies.

All development results are retained:

- [Initial twenty-year pilot](evidence/workshop-operators/initial-pilot/summary.md): eight runs.
- [Profit-only refinement](evidence/workshop-operators/profit-pilot/summary.md): four matched operator runs.
- [Fifty-year comparison before the entry screen](evidence/workshop-operators/before-entry-screen/summary.md): sixteen runs.
- [Final four-way comparison](evidence/workshop-operators/final/summary.md): sixteen runs.

The 44 trajectories include earlier unsuccessful designs; they are not pooled
as if they represented one model. Compressed raw JSON and run logs sit beside
those summaries. Each run manifest contains build provenance and raw checksums;
the [Artifact retention policy](evidence/README.md) hashes the
retained artifacts and analysis scripts.

## Final settings and results

All final comparisons use one verified executable: seeds 17, 81, 409 and 1024;
fifty years; sixteen founding civilizations; terrain and ecology resolution 64;
yield scale 0.33; living ecology, discoveries, offices and waterworks recovery
priority enabled. All runs used the available Quadro RTX 5000 Max-Q through
Vulkan. Other processes and some verification work overlapped execution, so wall
times are not isolated performance measurements.

Below, both columns retain industry-linked household income. Only private
workshop operation changes; equal-wage controls remain in the full report.

| Seed | Population, communal / operators | Shortage site-years, communal / operators | Operators started / active / closed | Completed / paid work |
|---|---:|---:|---:|---:|
| 17 | 1740 / 1760 | 148 / 150 | 66 / 11 / 55 | 85.4% |
| 81 | 1595 / 1561 | 172 / 183 | 163 / 15 / 148 | 82.6% |
| 409 | 2046 / 1999 | 246 / 268 | 137 / 15 / 122 | 83.4% |
| 1024 | 1626 / 1619 | 129 / 123 | 145 / 16 / 129 | 82.9% |

Companies tie up household savings, pay idle time and rent, and can lose expected
fees. Their effects on population and shortages are consequently mixed. The
controlled fixture proves the immediate capacity effect of removing operating
cash. These later population differences include the rest of the coupled history;
they do not isolate a single mechanism or establish empirical realism.

The wage-allocation rule itself lowered final population in all four communal
controls, by roughly 39–133 people relative to equal wages. This result matters:
industry-linked income and its subsequent feedback are more consequential here
than simply introducing operator records. It remains a deliberately unequal
income proxy, not a measured historical wage distribution or household employment
model. The [Artifact retention policy](evidence/README.md)
separate each intervention and their interaction within matched seeds.

Across the eight final operator cases, firms completed 82.4–88.0% of funded work.
Unused labor is paid idle time, not fictitious output. Five-year survival counts
exclude firms founded too recently to have reached five years; those denominators
and closure reasons remain in the structured summaries. Annual settlement
observations can miss short episodes; firm closure counts use persistent records.

Maximum reported relative conservation residual across all sixteen final runs:
**1.537e-5**, below the established **1e-3** failure tolerance. Every firm and owner
capital link also reconciles independently. This establishes accounting within
the stated numerical tolerance, not exact arithmetic or empirical calibration.

## Verification

Final-code checks passed:

- Ordinary all-target suite: **58 passed**, **133 hardware fixtures ignored**.
- Employer tests, explicitly including GPU fixtures: **5 passed**. They cover
  finite funding, productive and unproductive entry, realized-profit dividends,
  cash-to-capacity effects, closure, ownership reconciliation and saved/batched
  continuation.
- Existing GPU economy regression suite: **18 passed**, including overstock
  limits, recipe/material conservation, specialized capacity, mining limits,
  tool recycling and archived workshop assets.
- Household income/retail tests, including hardware fixtures: **5 passed**.
- Reporting tests: **4 employer-report tests** and **3 existing integration-report
  tests** passed; they check incomplete samples, invalid accounts, survival
  eligibility and complete four-way comparisons.
- Formatting, whitespace checks and all-target Clippy with warnings denied passed.

These are execution counts, not unique coverage totals; some focused tests also
appear in the ordinary suite. Verification logs are retained in the evidence
folder. The full set of ignored tests, other GPU backends, resolution convergence,
and empirical wage/business-survival validation were not covered by this pass.

## Reproduction and remaining scope

The commands in the [model document](workshop-operators.md#reproduction) regenerate
the final matrix; use `--seeds 17,81,409,1024 --years 50` for this report. Existing
output directories are rejected. `scripts/publish_enterprise_evidence.py` verifies
completed suites and checksums before retaining compressed trajectories and
regenerating summaries and factorial effects.

The four requested additions have first implementations and evaluation evidence.
These are household-owned operators **leasing communal equipment**, with one
canonical inventory of materials and products. Private physical factories,
independent input purchasing and product ownership, creditor/debt recovery,
multiple shareholders and household-specific employment contracts remain future
extensions. Local workshop layoffs are presently changes in aggregate income and
capacity, not individually scheduled worker dismissals.
