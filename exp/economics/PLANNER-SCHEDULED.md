# Independently planned dated delivery agreements

Status: implemented opt-in composition adapter. The same two people who often
fail under spot-only planning can now independently accept a six-month delivery
schedule and keep choosing their own productive work. In the matched 24-month
controls, both survive under beam and best-first search at all three tested cash
endowments. This is bounded evidence, not a new default or a general bargaining
solution. The low-cash best-first spot control still has fewer need shortfalls.

This follows [conditional spot offers](PLANNER-POSTED.md) and reuses the
[cooperative contract executor](COOPERATION.md). The earlier executor combined
terms with prescribed work choices; `Contract::independent` now identifies a
schedule with empty work choices. Its parties come from the delivery terms.
There is no second delivery ledger, balance sheet, default engine or monthly phase.

## Discovery, assessment and execution

`composition::continuation::scheduled::Controller` wraps `Persons`. The pilot
requires two people, two fixed-side listings, ContinuingFirst and monthly review
with six-month composition forecasts. It excludes household/financial acquisition
combinations already outside the composition adapter.

At an uncommitted Acquire boundary, the public menu contains one proposal: one
existing lot in each direction per month, starting next month and ending in the
sixth month inclusive. Thus there are five pairs of dated deliveries. Price comes
from matching fixed quotes and limits. No immediate trade is bundled into
acceptance. Closed listings prevent admission. The menu does not search quantities,
prices, alternative start dates or counterparties; the lower-ID person proposes.

Each person separately searches its own production and prerequisites against:

- An outside forecast using the ordinary standing spot-order policy and observed
  peer stocks/needs, with no new peer production.
- The offered schedule, conditional on the peer supplying its promised goods and
  payments. The person's own stocks, coins, needs, capacity and commitments remain
  real constraints. No private peer work plan is read.

The proposer must strictly improve its score; the recipient must be no worse.
New consent also requires zero forecast broken commitments under that score,
including process failures. This conservative pilot test can reject agreements
for a person with an earlier aborted process; it is not a probabilistic credit
assessment or a calibrated default tolerance.

The conditional branch budgets only the peer's remaining promised outgoings,
disables peer needs/work and removes its storage ceiling. Those are **forecast
assumptions**, never live endowments. Incoming goods or money are usable only after
their dated Acquire settlement. The evaluating person's opening cash still funds
its own payments, and forecast failure cancels remaining deliveries. Searches and
cheap candidate checks use the same hypothesis. Peer performance is assumed,
not established by inspecting or optimizing its future work.

Both consents enter the normal Acquire batch. Combined current work is checked
against actual settlement before publication. Declined proposals fall back to
ordinary `Persons` execution. Accepted terms live in `town_market` history and the
common `agreements::View::Exchange`, independently of the controller checkpoint.
Changing to ordinary simulation stepping cannot discard them, rewrite them or
withhold a due delivery by submitting an empty spot mask.

During a live schedule, the dated executor owns this bounded town book. There is
no residual spot book in the same boundary. Each person replans its own work each
month under the remaining terms; this is not a new acceptance vote. The outside
score retained on continuing receipts is diagnostic. Continuing receipts mark
`acceptable` because the earlier consent already binds. Fresh work is reserved
against actual current deliveries. If requested new work is infeasible, continue
existing work only; do not undo a due exchange to rescue an optimistic plan.

The normal timing remains: Open establishes admission/capacity; Acquire executes
due exchange and reserves current work; Productive executes that reservation;
need consumption and consequences follow. A crop harvested in Productive cannot
fund a delivery earlier in that same month's Acquire.

## Consequence and accounting

The shared executor uses opening goods, coins, storage and market admission.
Both payments need opening coins: same-boundary sale receipts cannot fund the
other payment. An unavailable independent delivery listing also prevents delivery.
All deliveries due in that month succeed together or none settle. If any cannot
settle, retain a failure receipt and **cancel the remainder**. Prior completed
trades stand. There are no automatic damages, refund claims or arrears under these
terms; these remain conditional exchanges, not independently collectible debts.

Settlement validates consents, dates, immutable accepted terms and actual effects.
Reconstruction rejects missing intervening boundaries, changed deliveries,
resurrected cancelled schedules and inconsistent completed-transfer/market records.
Replay checks remain in the common commit path. Later new agreements require fresh
consent and a new start month; restoring goods cannot restart a failed old schedule.

Actual goods/payments use the existing accounting path, with finite conserved
coins and separate person books. Executory conditional deliveries do not create
synthetic cash, a loan or unconditional receivable/payable claims. Future terms
remain visible through `View::Exchange` even though its waterfall `claims()` is
empty. The controller's history records offer/continuation assessments and failures;
its contained `Persons` history covers ordinary fallback execution.

## Matched comparison

[The runner](examples/planner_scheduled.rs) performs 24 runs: ordinary versus dated
agreements, beam/best-first, 1/2/6 coins per person, and trading on/off. Each lasts
24 months, with seed 7 and the unchanged [active-exchange fixture](PLANNER-EXCHANGE.md):
three-month crops produce eight grain plus seed; wood produces three fuel per
month; each person has one monthly labor, separate supplied rights, finite storage
and food/warmth needs. Both listings use one-unit lots at one coin. No issuance,
credit, new resources or productivity retuning is introduced.

Each search retains 256 expansions, 32 full forecasts and a six-month horizon.
Assessment adds searches, so this is not an equal-total-compute comparison.
Forecast-month counts include both assessments and ordinary fallback searches;
they exclude admission previews and settlement validation. The conditional arm
also changes the public terms and counterparty performance hypothesis. Results do
not isolate the benefit of binding terms from those other mechanism changes.

| Search | Mechanism | Coins each | Total reported deficit | Terminal person-months | Grain / fuel volume | Accepted schedules | Forecast-months |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Beam | Ordinary | 1 | 15 | 26 | 2 / 2 | 0 | 1,416 |
| Beam | Ordinary | 2 | 13 | 30 | 0 / 1 | 0 | 1,368 |
| Beam | Ordinary | 6 | 13 | 30 | 0 / 1 | 0 | 1,344 |
| Beam | Dated | 1 | 8 | 0 | 17 / 17 | 4 | 2,784 |
| Beam | Dated | 2 | 8 | 0 | 17 / 17 | 4 | 2,784 |
| Beam | Dated | 6 | 8 | 0 | 17 / 17 | 4 | 2,784 |
| Best-first | Ordinary | 1 | 2 | 0 | 20 / 20 | 0 | 1,968 |
| Best-first | Ordinary | 2 | 17 | 23 | 5 / 3 | 0 | 1,536 |
| Best-first | Ordinary | 6 | 13 | 28 | 3 / 0 | 0 | 1,392 |
| Best-first | Dated | 1 | 8 | 0 | 17 / 17 | 4 | 2,784 |
| Best-first | Dated | 2 | 8 | 0 | 17 / 17 | 4 | 2,784 |
| Best-first | Dated | 6 | 8 | 0 | 17 / 17 | 4 | 2,784 |

The twelve trading-disabled controls have zero volume/accepted schedules, deficit
12 and 32 terminal person-months. Ordinary arms reproduce the earlier spot baseline.
Deficit sums food/warmth shortfalls across both people; terminal person-months count
repeated terminal reports, not distinct deaths. Post-terminal reporting limits
comparisons based on deficit alone. Whole-run volume, contracts and search counts
repeat on each CSV person row and must not be summed twice.

The unchanged high-cash fixture now sustains both people over the measured window.
The low-cash best-first spot control remains better on deficits (2 versus 8).
Fixed public terms are therefore useful here, not uniformly preferable.
Each trading-enabled dated run completes three schedules without failure and
accepts a fourth. At the cutoff six individual deliveries (three reciprocal pairs)
remain due. These are not counted as completed success. The runner emits completed,
failed and outstanding counts separately.

## Verification and remaining scope

Focused tests cover explicit consent, next-month availability, actual opening
budgets, missing goods/cash/storage/admission, default cancellation, prior-payment
preservation, fresh work after a shock, stale/altered receipts, replay, common views,
reconstructed histories and a closed-book fallback. A unit test verifies that
conditional peer resources never endow the actor or mutate live state. A CPU versus
reference run compares reordered registrations, continuation from an accepted
schedule with pending productive work, complete ledger/controller histories and
separate double-entry audits. The 24-month matched survival/regression controls
also run as tests.

Verification: **198 integration tests across 31 suites and 13 library tests passed**,
including the CPU kernel smoke test. Formatting, strict all-target Clippy, local
Markdown links and the repository artifact check passed.

No new defaults, negotiated prices, general multi-party schedules, residual spot
clearing, risk probabilities, collateral or financial composition are introduced.
This is not a full v1 or annual 32-person stress rerun. A useful next bounded
comparison is alternative delivery dates/quantities and controlled production
shocks, to separate menu choice from the benefits and risks of conditional peer
performance assumptions.

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example planner_scheduled > ../../output/economics/planner-scheduled.csv 2> ../../output/economics/planner-scheduled-run.log
cargo +1.92.0 test --locked --release --lib --test planner_scheduled --test planner_posted --test cooperation --test cooperative_terms --test cooperative_lending --test composition --test composition_market --test planner_persons --test planner_exchange --test town_market --test agreements
cargo +1.92.0 fmt --check
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Generated CSVs and logs stay under ignored `output/`.
