# Recurring mixed-organization acceptance

This is the bounded continuation of the [fourteen-month interaction fixture](ACCEPTANCE-SCENARIO.md).
It tests four people, a household, a workshop, a town treasury, a state and a
marketplace operating together for **48 months**. Readiness means this configured
economy passes the gates below; it does not mean arbitrary organizations can
discover a sustainable economy from scratch.

The [voluntary-employment variant](VOLUNTARY-EMPLOYMENT-ACCEPTANCE.md) retains these
gates while replacing standing workshop/town jobs with dated mutual consent.
It explicitly redistributes three existing startup grain units and supplies
food-reserve-then-cash worker preferences; the scenario report records those
assumptions, failed trials and controls separately from this baseline.

The [inventory-driven follow-up](ACCEPTANCE-STOCK-MARKET.md) passes the same gates
with workshop/town ore bids and asks replacing the dated ore-forward schedule.
It also exercises temporary supply interruption and recovery. The baseline and
measurements on this page retain their original forward-based configuration.

The [working-capital variant](WORKING-CAPITAL-ACCEPTANCE.md) also passes these
unchanged gates after removing both workshop opening ore and metal. It combines
upstream discovery with an independently accepted loan and conditional productive
work. Its narrower assumptions and controls are documented separately.

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example acceptance
cargo +1.92.0 run --locked --release --example acceptance -- --continuation-matrix
cargo +1.92.0 test --locked --release --test acceptance_continuation
```

The default exits unsuccessfully on a failed readiness gate. The matrix also fails
if a negative control unexpectedly passes. The historical fixture remains available
with `--legacy`, its detailed receipts with `--legacy --details`, and its twelve
controls with `--matrix`.

## What changed

The original interaction fixture exhausted ore and municipal payroll reserves.
Here the town buys one smith-hour to extract two ore, and the workshop buys another
hour to refine two ore into two metal. Both draw from the same finite two-hour
personal endowment; neither organization receives endowed labor. Monthly prepaid
delivery agreements buy two ore for two coins, with delivery due the next month.
The workshop sells a two-metal lot for four coins through the mint venue. This
covers replacement ore and wages, leaving a margin. The town earns ore revenue
in addition to annual plot rent. Actual production, payment and delivery still run
through ordinary processes, employment, contracts, markets and double-entry books.

The workshop and town now use the shared agency controller under
[signed operating agreements](ORGANIZATION-OPERATING-AGREEMENTS.md). A fixed workshop
officer and rotating town officers authorize bounded work policies. Controllers
evaluate maintaining supply buffers against retaining current work policies and
commitments; their first accepted
policy takes effect at the following Open. Constitution, legal permission,
authority and actual resource feasibility remain separate checks. Existing state
and household governance continue alongside them.

Opening workshop inventory is four ore and two metal. The metal bridges the first
policy's next-month effective date. Crop output is six wheat and one replacement
seed per harvest, avoiding systematic seed accumulation. Organization stock targets
are four metal and four ore. The issuer's supplied coin reserve objective is 1,000,
providing continuing mint demand over this test. These are explicit calibration
choices, not discovered equilibrium prices or business plans.

## Acceptance gates

The first twelve months are startup. Each subsequent twelve-month window
(13–24, 25–36, 37–48) must independently demonstrate:

- Twelve completed refining jobs and twelve extraction jobs, twelve metal sale
  lots, and at least 24 replacement ore delivered.
- Twelve actual smith-hours and twelve paid wage coins for each employer.
- Strictly positive workshop and municipal net income, nondeclining cash reserves,
  and enough closing cash for the next ordinary operating payment.

Across the full run there must be no unmet nutrition, no late wages or loan arrears,
timely household and ore forward deliveries, a repaid bootstrap loan, and paid
annual rent in months 13, 25 and 37. All nine separate books and total issued coin
supply must reconcile. Each operating organization must retain its signed agreement
and make an authorized work-policy choice. A final zero arrears balance does not
erase an earlier missed wage payment: the checker inspects dated collection receipts.

Supplied employment, land and operating terms extend through month 60, beyond the
month-48 observation boundary. A month-48 ore prepayment due in month 49 remains a
real outstanding asset/liability, not an overdue delivery or a fabricated expense.
This avoids improving final cash merely by stopping replacement orders at the cutoff.
Automatic contract renewal is not tested.

## Measured baseline — 2026-10-11

The CPU runner passes all gates: zero nutrition deficits and wage arrears, 46 mint
batches issuing 460 coins, timely obligations, and nine reconciled entity books.
Workshop and town each complete 47 jobs and pay 48 wage coins across startup and
operation. The workshop sells 46 metal lots. Household and grower sell 14 and 20
food lots, with 17 purchases by the smith. Eleven grower storage rejections remain,
without unmet nutrition. These are lot counts, not resource-unit counts.

| Completed months | Workshop net income | Town net income | Workshop cash | Town cash |
| --- | ---: | ---: | ---: | ---: |
| 13–24 | 12 | 13 | 14 → 26 | 20 → 33 |
| 25–36 | 12 | 13 | 26 → 38 | 33 → 46 |
| 37–48 | 12 | 13 | 38 → 50 | 46 → 59 |

Each window contains twelve jobs per organization, twelve metal sales, 24 delivered
ore, and twelve actual hours with twelve wage coins per employer. Wage totals are
settlements of claims earned in that window; the separate full-history timeliness
gate excludes late payments. Net income comes from separate double-entry statements,
including production carrying costs. It is not inferred from cash balances.

| CPU control | Ready | Food deficits | Final wage arrears | Final-year workshop / town income |
| --- | --- | ---: | ---: | ---: |
| Baseline | Yes | 0 | 0 | 12 / 13 |
| No ore production | No | 43 | 2 | 0 / 0 |
| No replacement orders | No | 43 | 2 | 0 / 0 |
| Workshop excluded from venue | No | 67 | 2 | 0 / 0 |
| Metal lot price reduced to two | No | 5 | 2 | 0 / 0 |
| Startup payroll one coin short | No | 0 | 0 | 12 / 13 |

The last control eventually recovers financially but still fails the no-late-wages
gate. It replaces the first workshop wage bill with nine coins against eight
opening coins, then uses the ordinary one-coin monthly terms. Continued labor is
explicitly permitted. Its recovery does not rely on a hidden grant or waived claim.

## Verification scope

Tests compare reference and CubeCL CPU execution, including every-phase in-memory
Simulation reconstruction while retaining Audit, ledger and reports. Dated replay
reproduces State and Audit using each batch's actual World context. Monthly stepping
and reordering independent catalogs must agree with the normal batched run. This
does not establish a durable serialized checkpoint or replay from a bare ledger.

Negative controls remove ore production, replacement orders or workshop venue
access; reduce metal prices below sustainable margins; or supply a one-month startup wage bill one coin above opening workshop cash
while permitting continued labor. The last control tests historical lateness
even if wages eventually recover. Economic failure must remain distinguishable
from accounting failure.

The full crate release suite passes **1,293 tests, zero failed, one ignored**.
After final control and lint refinements, the six organization-authority and
recurring-acceptance tests pass again. Strict all-target release Clippy, formatting,
diff checks and repository artifact checks pass. The CPU runner and six-case matrix
produce the results above. The existing ignored test remains outside this result.

## Deliberate limits

- Identities, formation, consent, initial assets, recipes, contract terms, quotes
  and constitutional policy menus are supplied. The operating adapter does not
  establish company ownership, autonomous founding, recruitment or dissolution.
- Town extraction has labor limits but no finite geological deposit. It uses the
  same abstract environmental resource convention as this fixture’s wood collection.
- Mint demand depends on the configured issuer reserve objective. This is a finite
  operating test, not evidence of indefinite growth or general equilibrium.
- The venue trades scoped commodities and spot labor. Employment and ore delivery
  agreements remain outside it; this is not a universal rights/asset market or ZIP
  price-discovery test.
- Storage rejection may still occur. The nutrition gate measures its consequences;
  readiness does not require every proposed job to succeed.
- Multiple jurisdictions, reproduction, automatic estates and changing organization
  rosters are not exercised. Individual self-directed policy changes remain excluded.

Generated runner output and logs belong under ignored `output/`; only source,
tests and this human-readable evidence belong in Git.
