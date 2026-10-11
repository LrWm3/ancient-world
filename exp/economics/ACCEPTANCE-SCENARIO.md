# Mixed-organization acceptance scenario

Status: six independently reviewed passes completed on 2026-10-11. This is one opt-in fixture built on the
integrated commitment economy, not a replacement scheduler or a claim of arbitrary
organization support. Run from this directory:

```sh
cargo +1.92.0 run --locked --release --example acceptance
cargo +1.92.0 run --locked --release --example acceptance -- --matrix
cargo +1.92.0 test --locked --release --test acceptance
```

The acceptance target is people, a household, a workshop, a town organization, a
state and a venue interacting through finite resources, lawful agreements and
separate accounting. Success requires actual outcomes, not merely named agents.

The final default runs **nine agents for fourteen months**: four people, a household,
a workshop, a municipal treasury, a state and a marketplace. It records zero
nutrition deficits, three completed mints (30 issued coins), timely delivery of
the two-wheat forward, a repaid state loan and one paid annual land due. All nine
separate books reconcile. Workshop wages total eight coins; municipal wages total
ten, with no closing wage arrears. The workshop sells three metal lots; household
and grower sell four and six food lots respectively, including five purchases by
the smith. Organization names describe actual separate operations, but workshop
and town operating mandates are supplied; their founding and governance are not
implemented.

This passes the bounded interaction and food-access gates. **Financial sustainability
remains unfulfilled:** the workshop loses eight in carrying-value terms, exhausts
its opening ore and has only a four-month employment term. The town consumes its
opening payroll reserve faster than rent replaces it. Neither has a demonstrated
self-financing continuation. One grower storage rejection remains.

The pass records below describe successive configurations; their earlier counts
are historical. The final comparison and verification are in pass six.

## Pass 1 — establish the existing chain

`acceptance::Run` reuses the elected state/household commitment fixture and normal
Audit stepping. It reports actual nutrition deficits, completed mint work, forward
deliveries, annual land payments, issuance and reconciled separate agent books.
It rejects evidence requests before a completed month and reconciles the entire
coin supply against issuance. The 14-month reference control has two opening
nutrition deficits, four completed mints, 40 issued coins, two delivered wheat and
one paid annual land due. A final delivered total alone does not prove timeliness;
the underlying commitment tests retain the month-four deadline assertion.

## Initial boundaries (pass one)

- Workshop: no distinct operating company yet in this initial pass.
- Town: no governed municipality yet; locality is not governance.
- Marketplace: the existing mint venue supports scoped posted quotes; it is not a
  universal market for all contracts. Do not combine incompatible acquisition drivers.
- Founding consent, constitutional menus, quotes and initial endowments are supplied.
- No automatic company ownership/dividends, municipal formation or dissolution,
  multiple jurisdictions or population/discovery integration is implied.

## Pass 2 — make metal supply an operating dependency

The workshop is a distinct permitted type with its own ore, coins, storage and
zero endowed hours. A four-month employment agreement buys two smith-hours each
month; ordinary productive work converts two ore into two metal. The workshop
replaces the original seller in the existing venue's metal quote. Output from
Productive can first be offered at the following Acquire. Opening metal is removed,
so the state cannot issue coins unless the workshop actually produces and sells it.

Employment is an explicit accepted commitment, not an optimizer that automatically
stops wages when no useful job exists. Missing ore can leave paid but unused hours;
missing labor cannot create output. Company founding, ownership and governance
remain open. The runner starts from this month-one fixture; restart verification
must preserve its Audit and opening-supply basis rather than construct a new book.

Measured reference outcome: four workshop batches, eight paid wages and four mint
batches; eight distinct agent books reconcile. Removing ore or smith hours stops
both refining and minting. Workshop cash returns to eight, but this is **not
profitability**: the supplied metal price covers wages and consumes the opening ore
endowment without replacing its carrying value. Financial sustainability remains a
separate acceptance gate.

## Pass 3 — municipal treasury and competing employment

A distinct municipal type owns the plot and receives its existing one-coin annual
lease payment. It has ten opening coins, no endowed hours, and an accepted contract
for one smith-hour each month to collect one unit of retained municipal fuel.
The workshop's employment rank precedes the municipality's rank. Both requests use
the same ordinary employment reservation boundary; timing is unchanged.

The first four months leave no town hours because the workshop buys both available
smith-hours. After its four-month contract expires, the town can buy one hour per
month. This is an operating mandate and real treasury, asset and output accounting,
not a municipality with implemented founding, elections or autonomous spending.
Retained fuel is not yet distributed public benefit, and annual rent does not cover
recurring payroll. Neither this treasury nor the workshop receives synthetic labor.

## Pass 4 — paid people must also eat

The smith and existing mint worker each require one wheat per month, start with
one wheat, and retain their own accounts. The smith starts with one coin; employment
income arrives through normal wage settlement. Wheat quotes use three-unit lots at
three coins. Public sales and explicitly authorized household/grower surplus asks
share the existing mint venue. The grower retains two private wheat; household
membership does not authorize sale of private inventory. Sale/claim protection
uses one month. Quote limits and stock targets are supplied, not ZIP learning.

Calibration is explicit: the broad fixture uses six wheat and two seed per harvest,
four hours, and a private grain work target of twelve. The original integrated
fixture remains unchanged. `TightFood` changes only the broad fixture's wheat yield
to four; `NoSalesTarget` removes only its commercial work target. Neither the work
target nor the quotes are autonomously discovered business plans.

The diagnostic run found that increasing yield alone did not feed the smith.
Storage blocked repeated harvests, and private wheat was not on the market. Adding
a protected private ask reduces the full-run food deficit to five (four for the
smith) and creates four household food sales and three smith purchases. This is
still a **failed full-nutrition gate**, not sustainable autonomous coordination.
Each sale/purchase count is a completed whole-lot deal, not a wheat-unit count.
The two-wheat forward delivers by month four, the state loan is actually accepted
and repaid, and annual rent is paid. Missing loans report `None`, not zero debt.

The household and grower feed different buyers: aggregate household sales do not
prove household-to-smith exchange. The test checks a committed private-seller route
to the smith explicitly. Crop accounting retains the inherited 4:2 wheat/seed cost
weights as a supplied carrying-cost policy; those weights are not output quantities
or market valuations for the calibrated 6:2 recipe.

## Pass 5 — matched controls and a viable food calibration

The CPU `--matrix` run exercises twelve cases on the same fixture. All cases pass
separate-book and coin-supply reconciliation, including those with economic failure.

| Pass-five case | Food deficits | Mint batches | Municipal work | Wage arrears |
| --- | ---: | ---: | ---: | ---: |
| Baseline (private wood target 100) | 5 | 3 | 10 | 0 |
| Four-wheat harvest | 17 | 3 | 10 | 0 |
| No commercial grain work target | 5 | 3 | 10 | 0 |
| No private wheat asks | 20 | 3 | 10 | 0 |
| Private wood target 4 | 0 | 3 | 10 | 0 |
| No workshop ore | 8 | 0 | 10 | 0 |
| No smith hours | 20 | 0 | 0 | 0 |
| Workshop excluded from venue | 8 | 0 | 10 | 0 |
| Refining legally prohibited | 8 | 0 | 10 | 0 |
| Municipality has first labor priority | 8 | 0 | 12 | 1 |
| No opening workshop payroll cash | 5 | 2 | 12 | 1 |
| No opening municipal payroll cash | 5 | 3 | 2 | 1 |

The bounded wood target reduces grower storage rejections from six to one and
eliminates all nutrition deficits in this fourteen-month control. This is a supplied
personal work target, not a new universal food-first policy. Removing the grain work
target has no measured effect in the original calibration; do not attribute its
outcomes to a planner improvement. Changing employer rank gives the workshop
partial hours that cannot complete its indivisible two-hour job: accepted wages
still accrue even when bought hours cannot produce. Lack of payroll funding retains
real arrears and suspends later delivery; it does not erase already supplied work.
Workshop exclusion permits refining but blocks sales; legal prohibition blocks
refining itself. The workshop's measured net income is negative eight in the
normal-price runs, so nutrition success still does not establish business viability.

## Pass 6 — fix the default and verify the whole boundary

The default now uses a private fuel stock target of four. `CrowdedWood` retains
the previous target of 100 as a control. No execution rule or original integrated
fixture changes. Every final case differs from this default by the stated control;
all retain separate accounting and coin-supply reconciliation.

| Final case | Food deficits | Mint batches | Municipal work | Wage arrears |
| --- | ---: | ---: | ---: | ---: |
| Baseline | 0 | 3 | 10 | 0 |
| Four-wheat harvest | 6 | 3 | 10 | 0 |
| No commercial grain work target | 0 | 3 | 10 | 0 |
| No private wheat asks | 20 | 3 | 10 | 0 |
| Crowded wood target 100 | 5 | 3 | 10 | 0 |
| No workshop ore | 7 | 0 | 10 | 0 |
| No smith hours | 20 | 0 | 0 | 0 |
| Workshop excluded from venue | 7 | 0 | 10 | 0 |
| Refining legally prohibited | 7 | 0 | 10 | 0 |
| Municipality has first labor priority | 7 | 0 | 12 | 1 |
| No opening workshop payroll cash | 2 | 2 | 12 | 1 |
| No opening municipal payroll cash | 4 | 3 | 2 | 1 |

Removing the commercial grain target changes who sells food (five lots each from
household and grower), but does not harm nutrition in this control. Its benefit is
not established. All twelve cases deliver the two-wheat forward on time; this is
not evidence that all their other obligations or needs succeed.

Nine focused tests cover actual committed deals, protected private asks, finite
shared employment, real input dependencies, venue versus legal permission,
liquidity and storage controls. Every case compares reference execution against
CubeCL CPU with an in-memory Simulation reconstruction after **every phase**,
retaining the Audit, ledger, reports and opening supply basis. The default also
matches month-at-a-time execution with reordered agent/resource/recipe/employment
catalogs. Reordering catalogs does not reorder causally dependent monthly phases.

A separate default replay reconstructs State and Audit from committed batches
using each batch's actual dated World context, including Open governance changes.
This proves replay under those contexts, not reconstruction from a bare ledger
or durable serialized restart. A runner rejects a noninitial opening rather than
silently initialize new books over a partially completed economy.

## Remaining boundaries

- Workshop/town founding, ownership, governance, recruitment and autonomous spending
  need shared organization adapters. Existing state/household controllers still run.
- Fixed recipe, employment terms, quotes, consent, stock targets and opening assets
  are supplied. This scenario does not demonstrate endogenous discovery or ZIP.
- The mint venue supports scoped commodities and spot capacity trades, including
  state procurement of worker hours. Workshop/town employment contracts remain
  outside it; rights, membership and assets do not yet share a universal interface.
- Retained municipal fuel is an accounted asset, not measured public welfare.
- Broader jurisdictions, reproduction and automatic estates are not exercised here.
- Longer operation needs viable margins, replenishment and recurring public funding;
  zero nutrition deficits over fourteen months does not establish those properties.

## Verification record

The focused acceptance target passes nine tests. The related ten-target release
regression run passes **92 tests, zero failed, one ignored** (including those nine):
`acceptance`, `integrated_agency`, `commitment_agency`, `employment`, `minting`,
`mint_orders`, `mint_finance`, `mint_cycles`, `mint_provision` and
`household_accounting`. The ignored annual household test was not rerun for this
fixture-only change. The full crate suite was not rerun in this batch.
Strict all-target release Clippy, formatting, diff checks and repository artifact
checks pass. Generated matrix/log artifacts remain under ignored `output/`.
