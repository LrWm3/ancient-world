# Input buffers and completed-work evidence

This follows [independent liquidity preferences](LIQUIDITY-PREFERENCES.md). The
previous cash-exposure preference helped a metal-only startup but interrupted one
production month when the workshop began with both ore and metal. This batch
asks whether an explicit preference for retaining inputs helps, and what it costs.
It does not change the default objective, settlement, scheduler, or readiness gates.

## Ten changes

1. Read dated process completions from committed ledger transitions.
2. Attach operator completion evidence to ordinary organization forecasts.
3. Add opt-in founding input-buffer priorities using existing reserve exposure.
4. Retain each party's baseline/proposed completions in forward assessments.
5. Verify preference scope and unchanged resources, rights and discovered catalogs.
6. Provide a reproducible input-buffer comparison command.
7. Compare startup stocks and priority orders, including a common-opening decision.
8. Exercise resource, market-access, finance and payroll controls.
9. Extend CPU/reference, phase reconstruction, replay and catalog-order checks.
10. Record measured results and remaining limits in the documentation.

Independent review checked the reader, preference construction, experimental
controls and interpretation. The helper consumes trusted committed batches; it
is not an admission validator. It includes consumption as well as production,
attributes work to its operator rather than beneficiary, uses the completion
batch's month, deduplicates process IDs and ignores nested pending production
plans. Forecast consumers can select the relevant process definition. Completion
evidence is diagnostic; it is not a new objective or a promise to live agents.

## Controlled variants

All comparisons run 48 months. Workshop cash preference is monthly exposure;
town cash preference is terminal reserve. Initial town ore is zero. Workshop
stock ablations retain or remove the same opening four units of ore and metal.
All other endowments, prices, horizons, employment terms, permissions and startup
forward options remain fixed within each comparison.

The optional workshop buffer is the gross entry-input requirement of one REFINE
recipe: two ore. It uses `ReserveExposure`, accumulating each completed month's
shortfall from that target. `InputsFirst` places it before the organizational cash
goal; `CashFirst` places it after. `Off` is exactly the preceding experiment.
These are initial signed preference variants, not amendments to live mandates.
Applicable personal preferences are treated similarly; personal goals without
an organizational cash goal remain unchanged.

This is an inventory preference, not a production-continuity constraint. Holding
two ore does not guarantee labor, storage, legal access, buyers, cash or a completed
batch. Conversely, consuming ore productively can increase the measured shortage.
The recipe helper sums entry inputs once per stage, not per elapsed month. It is
not net-material planning for arbitrary multistage recipes.

## Forty-eight-month outcomes

Production counts are completed batches, food deficit is cumulative, and wages
are outstanding arrears at the end. All rows reconcile through the ordinary audit.

| Opening stocks | Buffer priority | Mints | Workshop | Town | Food deficit | Wage arrears | Signed / activated | Ready |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Ore and metal | Off | 41 | 40 | 40 | 0 | 0 | 7 / 9 | No |
| Ore and metal | Inputs first | 2 | 0 | 0 | 52 | 2 | — | No |
| Ore and metal | Cash first | 41 | 40 | 40 | 0 | 0 | 7 / 9 | No |
| Metal only | Off | 41 | 40 | 42 | 0 | 0 | 5 / 7 | Yes |
| Metal only | Inputs first | 1 | 0 | 3 | 61 | 2 | 2 / 4 | No |
| Metal only | Cash first | 41 | 40 | 42 | 0 | 0 | 5 / 7 | Yes |
| Ore only | Off | 0 | 0 | 0 | 67 | 2 | — | No |
| Ore only | Inputs first | 0 | 0 | 0 | 67 | 2 | — | No |
| Ore only | Cash first | 0 | 0 | 0 | 67 | 2 | — | No |
| Neither | Off | 0 | 0 | 0 | 67 | 2 | — | No |
| Neither | Inputs first | 0 | 1 | 4 | 67 | 2 | 1 / 3 | No |
| Neither | Cash first | 0 | 0 | 0 | 67 | 2 | — | No |

Cash-first reproduces the Off outcome totals and agreement dates in all four
stock variants. Both retain the prior both-stock readiness failure: eleven
workshop batches in months 13–24 rather than twelve; the next two annual windows
complete twelve. Metal-only passes under both orderings without an input-first
priority.

Input-first is not a successful general fix. In the both-stock start it completes
no workshop work and signs no forward. In metal-only it signs in month 2 instead
of month 5 and subsequently stalls. Neither-stock obtains a small amount of work
but remains unsustainable. The local mature-boundary score reversal therefore
does not generalize to a desirable preference from founding. Preserving an input
stock may discourage its productive use, but this comparison does not isolate
that mechanism from startup consent and subsequent decisions.

## Reproduction

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example input_buffers -- both inputs
cargo +1.92.0 run --locked --release --example input_buffers -- metal cash
cargo +1.92.0 run --locked --release --example input_buffers -- metal inputs low-price
cargo +1.92.0 test --locked --release --test input_buffer_outcomes -- --nocapture
cargo +1.92.0 test --locked --release --test input_buffer_controls -- --nocapture
```

The command accepts stocks `both|metal|ore|neither`, priorities `off|inputs|cash`,
and optional controls `baseline|no-ore|no-access|low-price|late-payroll`. It prints
actual accounting evidence, unchanged readiness checks, annual production/payroll,
forward dates and month-12 private alternatives. Each command's month-12 snapshot
belongs to that run's own history; the separate common-opening test isolates
scoring differences using identical state, peers and candidate actions.

## Why the common-opening choice changes

The common-opening test starts at month 12 of the previous both-stock,
cash-exposure trajectory. It compares retaining the existing program with
program 1, which retains work and a metal ask but drops the ore bid. Applying
either scoring policy to the same action produces exactly the same forecast
path, completed processes, sales and ending stocks.

| Action | Cash exposure loss | Ore exposure loss | Projected completed workshop batches |
| --- | ---: | ---: | ---: |
| Retain | 11,910 | 2 | 4 |
| Drop ore bid | 11,870 | 24 | 1 |

Cash-only scoring selects the lower cash loss. Input-first scoring selects the
lower ore loss before considering cash. The four retained-branch completions
occur in months 12, 17, 20 and 23; the drop-bid branch completes only in month 12.
These are private forecasts with frozen future autonomous decisions, not the
actual economy's subsequent monthly output. No universal production objective
or forecast guarantee follows from this comparison.

## Adverse controls

These controls use the metal-only start. All three priorities fail readiness under
each control. Off and cash-first have identical listed outcomes; input-first differs
only in the lower-price row.

| Control | Priority | Mints | Workshop | Town | Food deficit | Wage arrears |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| No ore production | All | 2 | 0 | 0 | 52 | 2 |
| No workshop market access | All | 0 | 0 | 0 | 67 | 2 |
| No smith labor | All | 0 | 0 | 0 | 88 | 0 |
| No workshop ore storage | All | 2 | 0 | 0 | 52 | 2 |
| No startup-forward authority | All | 2 | 0 | 0 | 52 | 2 |
| Lower metal sale price | Off / cash-first | 2 | 0 | 0 | 58 | 2 |
| Lower metal sale price | Inputs first | 5 | 3 | 5 | 40 | 2 |
| Unfunded startup payroll | All | 2 | 0 | 0 | 52 | 41 |

The checks require no workshop metal sales without market access, no delivered
workshop hours without smith labor, no positive ore receipts without storage,
and no signed/admitted workshop ore forwards without authority. Readiness and
financial totals are also pinned. More activity under low prices reduces the food
deficit but does not eliminate arrears or establish recurring viable operation.

The storage impossibility fixture deliberately makes metal volume-free globally
and gives the workshop zero capacity while ore has positive volume. This allows
its opening metal to exist but forbids acquiring ore. It is not a one-variable
capacity calibration, and its financial effects cannot be attributed solely to
changing capacity. The labor control removes the smith's entire endowment,
including potential personal work; its nutrition effect is likewise not solely
an employment-policy effect. These controls establish feasibility boundaries.

## Verification scope

Verification passed 78 selected tests across 18 Cargo targets: 30 library tests,
31 focused observer, forecast, forward, discovery and preference regressions,
four startup-stock comparisons covering twelve runs, seven adverse controls
covering twenty-one runs, and six continuation cases. The matrix tests pin
readiness, production, nutrition deficits, wage arrears and forward dates; the
controls also check their direct resource or authority constraints. This was a
targeted regression run, not the entire crate's test suite. Formatting, strict
all-target Clippy and the repository artifact check passed. The example command
also completed a 48-month both-stock/input-first run.

The six 48-month continuation cases compare the reference backend with CubeCL
CPU execution, including both successful and unsuccessful readiness outcomes.
They reconstruct the CPU simulation after every phase while preserving its world,
state, ledger, reports and effect limit, and compare the final world, state,
ledger, reports and audit exactly. Replay compares state after each phase and the
final audit using the live reference world. Reordered catalogs compare state,
ledger and audit. These are in-memory continuation and state-replay checks; they
do not establish durable checkpoint serialization or reconstruction of autonomous
world decisions from the ledger alone. CUDA was not exercised.

## Remaining work

There is still no universally preferable founding objective. Cash-first leaves
the previous both-stock replenishment failure; input-first harms startup. A next
comparison should distinguish productive use from idle input retention, with
explicit dated semantics for any production-flow objective. `FailedProcesses`
cannot represent work that was never started. A measured input buffer should not
be relabeled as productive continuity.

Continue comparing identical opening alternatives and complete founding-to-
operation paths. Do not repair a policy result by adding money, relaxing readiness,
or treating a signed forward as completed production. Startup prices, constitutions,
labor terms and substantial initial resources remain supplied; these fixtures do
not establish autonomous institutional formation or general economic viability.
