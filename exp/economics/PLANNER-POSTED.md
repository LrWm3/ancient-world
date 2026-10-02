# Conditional reciprocal spot offers

Status: implemented opt-in follow-up to [observed expectations](PLANNER-EXPECTATIONS.md).
Independent planners can now announce order intentions and offer a reciprocal
package: “I sell this lot if I also buy that lot this month.” Each person separately
assesses the offer, and the existing market settles both trades or neither.
Some previously withheld trades now happen, but higher-cash cases still fail to
sustain both people. Existing planner defaults remain unchanged.

This is a bounded spot-offer adapter, not the integration of multi-month delivery
promises into composition search. The older [cooperative agreement](COOPERATION.md)
pilot already has dated future delivery schedules and cancellation consequences,
but uses a different work-choice planner and different fixtures. That pilot remains
intact; its long-run results are not a controlled comparison with this experiment.
The later [dated-delivery adapter](PLANNER-SCHEDULED.md) now integrates that executor
with independent composition search; the spot results below remain unchanged.

## Public intentions and independent acceptance

`composition::continuation::posted::Controller` wraps the existing `Persons`
coordinator. It supports exactly two persons, two fixed-side listings and monthly
review. It makes at most one reciprocal proposal at each Acquire, with the lower
agent ID as proposer. These are explicit pilot limits, not a universal bargaining
or allocation policy.

1. Run ordinary independent planning on a clone of the same opening state. Its
   selected order masks become public preliminary intentions. The trial settlement
   remains unpublished and is retained as the ordinary fallback.
2. Quote a reciprocal package from the existing book with both listed sides
   submitted. Normal need gates, seller protection, venue admission, fixed opening
   budgets, quotes and match limits still apply. No proposal is assessed unless
   both current deliveries could settle. Terms reuse `cooperation::Delivery` and
   the existing market's goods/payment transfers.
3. The proposer searches its own production options twice: an outside option
   against the announced peer masks, and a candidate with both of its reciprocal
   orders required against the offered peer masks. The candidate must strictly
   improve its private score.
4. If the proposer agrees, the recipient performs its own two searches against
   the same public information. Its candidate must be no worse than its outside
   option. Neither assessment reads the other's private selected work. Scores,
   requests and search costs are retained for inspection, not passed as planning
   input to the other person.
5. Two consents are still subject to current shared feasibility: common offer
   preparation validates the combined new-work requests and continuing work
   against the actual acquired resources. If it succeeds, publish one Acquire
   batch with both trades and dated work. Otherwise publish the original ordinary
   fallback. A failed final settlement publishes neither decisions nor receipts.

The outside option is a bounded best response to the publicly announced masks,
not a guaranteed payoff or a globally optimal alternative. The announced choices
can differ from that best response. If the proposal fails, the original submitted
intentions remain the fallback; there is no iterative best-response loop here.
Stable proposer priority also means an offer improving only the other person's
score can be missed. There is no price or quantity negotiation, alternative terms
menu or general agreement discovery in this adapter.

## Current binding terms versus future assumptions

Both private forecasts extrapolate their respective public peer masks over the
six-month horizon, with the actor's own chosen order policy also held consistent.
Peer consumption and finite stocks remain visible, but new peer production is
omitted as in the earlier composition adapter. Forecasts contain no joint work
optimization and create no live resources.

**Only this month's two deliveries are conditional and binding.** Continued
submission in later forecast months is an explicit renewal assumption, not an
accepted six-month promise. No future coin, goods or labor is escrowed. An offer
can look beneficial under that assumption and still lead to poor subsequent
outcomes. This distinction is material to the results below.

`town_market::evaluate_conditional` checks exact participant consent masks and
requires the ordinary book's completed deliveries to equal both posted terms,
including quantities, dates and prices. A partial book cannot satisfy the package.
Sale proceeds cannot finance another outgoing payment in the same book; both
payments need opening coins. Completed spot trades retain their ordinary double-entry
accounting and price/volume observations. There are no new balances or synthetic
netting credits.

The committed `Round::conditional` retains the accepted spot terms. Settlement
recomputes the book, rejects changed terms/consents/transfers and preserves replay
checks. Reconstructed state checks receipt consistency. Unfulfilled spot conditions
publish no partial exchange and create no arrears, damages or future cancellation
schedule. This completed spot record does not add a persistent claim to
`agreements::for_agent`; multi-period `View::Exchange` remains the older agreement
adapter. Future productive execution is not a condition that can undo settled
spot trades.

Open admission, Acquire reservation/settlement and Productive execution retain
the existing timing. The wrapper records every offer boundary, its public terms,
independent assessments and outcome. Its contained `Persons` history records
ordinary fallback execution only; accepted offer execution is in the wrapper
history and normal simulation ledger. Checkpoint the wrapper with the simulation.

## Matched comparison

[The runner](examples/planner_posted.rs) executes 24 runs and emits 48 person rows:
beam/best-first, ordinary spot versus conditional offers, 1/2/6 opening coins per
person, and trading enabled/disabled. Each lasts 24 months. All physical settings,
rights, fixed one-coin lot prices, score, consequences and planning budgets match
[active exchange](PLANNER-EXCHANGE.md#controlled-comparison): six-month forecast,
256 expansions and 32 full forecasts per search, two-month buying/protection,
seed 7, ordinary counterparty expectations and standing own-order forecasts.
No endowment or productivity was retuned.

The offer mechanism performs additional searches. Each actor first announces
using its normal search budget, then can use that budget for an outside and an
offer assessment. This is a mechanism comparison, not an equal-total-compute
ablation. Forecast-month totals below include those extra searches and both
actors, but exclude repeated settlement validation and admission previews; they
are not wall-clock benchmarks.

Trading-enabled results:

| Search | Mechanism | Coins each | Total reported deficit | Terminal person-months | Grain / fuel volume | Accepted packages | Forecast-months |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Beam | Ordinary | 1 | 15 | 26 | 2 / 2 | 0 | 1,416 |
| Beam | Ordinary | 2 | 13 | 30 | 0 / 1 | 0 | 1,368 |
| Beam | Ordinary | 6 | 13 | 30 | 0 / 1 | 0 | 1,344 |
| Beam | Conditional | 1 | 20 | 22 | 3 / 3 | 1 | 1,674 |
| Beam | Conditional | 2 | 17 | 27 | 1 / 2 | 1 | 1,542 |
| Beam | Conditional | 6 | 18 | 26 | 2 / 2 | 2 | 1,620 |
| Best-first | Ordinary | 1 | 2 | 0 | 20 / 20 | 0 | 1,968 |
| Best-first | Ordinary | 2 | 17 | 23 | 5 / 3 | 0 | 1,536 |
| Best-first | Ordinary | 6 | 13 | 28 | 3 / 0 | 0 | 1,392 |
| Best-first | Conditional | 1 | 2 | 0 | 20 / 20 | 0 | 3,168 |
| Best-first | Conditional | 2 | 17 | 23 | 5 / 3 | 0 | 1,836 |
| Best-first | Conditional | 6 | 18 | 20 | 5 / 5 | 5 | 2,046 |

All twelve disabled-trading controls have zero volume, zero accepted packages,
reported deficit 12 and 32 terminal person-months. Money is finite and conserved.
Ordinary arms reproduce the previous standing-policy/ordinary-expectation results.

Total deficit sums reported food and warmth shortfalls across both people.
Terminal person-months count repeated terminal reports, not distinct deaths;
post-terminal reporting can reduce the deficit total. Lower deficits alone are
not evidence of better welfare. Volume, accepted packages and forecast-month totals
are repeated on both CSV person rows and must not be summed twice.

At six coins each, best-first accepts five reciprocal packages and increases trade
from 3/0 to 5/5, delaying terminal outcomes but not preventing them. Beam also
accepts a few packages without sustaining the pair. The viable low-cash best-first
case takes no offer and remains at 20/20 volume with both people surviving. It
spends more search effort assessing offers it does not need.

These results demonstrate independent conditional acceptance and actual atomic
exchange. They do not demonstrate robust mutually sustainable plans, economically
optimal bargaining or reliable future delivery. Announced-mask extrapolation,
missing peer-production forecasts, finite search/horizon and supplied market roles
remain material limits.

## Verification

Six new tests cover reciprocal consent, both opening payments, missing goods,
admission/date/price failures, partial-book rejection, altered receipts and replay,
reconstructed receipt checks, independently beneficial assessments and own-only
work, the surviving low-cash and failing high-cash controls, closed-book fallback,
failed-boundary atomicity, and CPU/reference equality with reordered registrations.
The twelve-month CPU control resumes before Productive after an accepted offer,
with dated work pending, and reconciles separate double-entry books.

**186 tests passed across 29 regression suites**, including the existing planners,
expectations, acquisition,
allocation, offers, town/production markets, cooperative schedules and lending,
reciprocal markets, calibration and telemetry. Formatting, strict all-target Clippy,
local documentation links and the repository artifact check passed.
The full v1 runner and annual 32-person stress scenario are outside this scoped run.

From `exp/economics`:

```sh
mkdir -p ../../output/economics
cargo +1.92.0 run --locked --release --example planner_posted > ../../output/economics/planner-posted.csv 2> ../../output/economics/planner-posted.log
cargo +1.92.0 test --locked --release --test planner_posted --test planner_expectations --test planner_exchange --test planner_persons --test planner_continuation --test composition --test planner_calibration --test composition_market --test town_market --test production_market --test cooperation --test acquisition --test household_market --test household_offers --test process_offers --test forecast_context --test planning --test search --test financial_offers --test competition --test allocation --test resolution --test intermediary --test cooperative_terms --test cooperative_lending --test production_lending --test reciprocal_market --test calibration --test telemetry
cargo +1.92.0 fmt --check
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Generated CSVs and logs remain under ignored `output/`. The next bounded question
is whether composition can assess and honor a small dated delivery schedule through
the existing cooperative agreement executor, with failed-delivery consequences,
instead of treating later trade as an assumed renewal. That integration remains
implemented in the separate [dated-delivery comparison](PLANNER-SCHEDULED.md);
current spot success still must not be treated as a future commitment.
