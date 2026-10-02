# Independent production and exchange planning

Status: implemented opt-in extension to [multi-person admission](PLANNER-PERSONS.md).
Two active persons now independently choose production and town-market order
submissions. A shared book settles their actual choices before admitting new work.
A 24-month low-cash best-first control sustains reciprocal exchange and both people
survive. Beam search and higher-cash controls still fail. This demonstrates one
compatible pair of plans, not reliable autonomous economic coordination.

The existing simulation defaults, scheduler and historical [v1 results](V1-RESULTS.md)
are unchanged. `StandingPolicy` is the default forecast variant for this newly
supported market mode of the opt-in `Persons` coordinator.

## Decision, clearing and execution boundaries

`composition::continuation::persons::Persons` retains one controller per person.
Each reads the same Acquire opening and scores only its own outcomes. A controller
selects productive requests and a set of `(listing, side)` order submissions.
It cannot choose another person's work or spend another person's balance.

The live boundary proceeds as follows:

1. Open establishes normal dated venue admission and current observations.
2. At Acquire, collect every participant's independently chosen order mask. An
   empty mask explicitly withholds that participant's orders; there is no automatic
   live counterparty submission in this mode.
3. `town_market::evaluate_selections` generates ordinary eligible orders from the
   common opening, applies the masks and clears the book. Masks cannot grant venue
   access, bypass order gates or supply missing money, inventory or storage. All
   registered traders must be covered. Listing order and matching policy remain
   the existing town-market rules.
4. `composition::market::prepare_all` exposes that acquisition to common offer
   preparation. The explicit allocation policy admits new work against actual
   fills, existing commitments and previously admitted packages. Failed work can
   receive a freshly validated fallback. Equal-score request-only alternatives
   remain available in non-market mode; market mode does not use them because they
   omit the accompanying order choices.
5. Commit one Acquire batch. Productive executes its dated reservations once;
   subsequent consumption, consequences and reporting use the normal phases.

Spot orders and productive requests are separate intents. If a valid spot trade
settles but the proposed new process is rejected, the goods remain acquired.
This is not an all-or-nothing contract combining a trade and future work. Conversely,
an expected but unfilled purchase cannot authorize work requiring that input.
Sale income cannot finance another purchase in the same book: buying remains
bounded by opening money even when listing order would otherwise permit recycling.

Settlement recomputes the book from the supplied masks and rejects altered receipts
or transactions. The entire boundary and coordinator update remain unpublished on
failure. `Round::exchanges` records each person's submitted sides, private expected
fills and jointly cleared actual fills. Existing town receipts explain ineligibility,
`PlannerWithheld`, failed matches, prices and volume.

## Forecast choices and their limits

Private market branches keep other persons' current stocks, ordinary consumption
and ordinary need-order generation, but remove their productive capacity and active
production. These are conditional forecasts. Actual peers independently choose
orders and production; neither their chosen masks nor future production are known
to this forecast. Current state is observable, so this is not a private-information
model. The direct single-person `choose` guard against active counterparties remains;
only the coordinator constructs these hypothetical branches.

`Persons::order_forecast` exposes two variants:

| Variant | Actor's own forecast orders |
| --- | --- |
| `CurrentBoundaryOnly` | Select currently eligible sides for this Acquire; later forecast boundaries return to ordinary order generation. This preserves the earlier adapter's behavior. |
| `StandingPolicy` | Select registered listing/sides, including those eligible only later. Apply that selection throughout the forecast; ordinary gates still determine whether an order can actually be placed. |

The second variant addresses an inconsistency: a candidate could withhold its ask
now while relying on its own automatically generated future asks to fund later
purchases. It holds the actor's submission policy consistent within one forecast.
It also changes the candidate set by including future-eligible sides, so this is
not a pure one-variable ablation of persistence. It adds no wealth objective and
changes no quote, matching, accounting or need-consequence rule.

Live decisions are still reconsidered monthly. A standing forecast neither retains
live orders nor commits future counterparties. Retain/repair and scheduled market
review are rejected before advancing the simulation. Non-market review modes
remain supported. Checkpoint the coordinator alongside the simulation; this tests
cloned in-memory continuation, not durable serialization.

## Controlled comparison

[The runner](examples/planner_exchange.rs) executes 24 runs and emits 48 person rows:
beam/best-first search, both forecast variants, 1/2/6 opening coins per person, and
trading enabled/disabled. Each run lasts 24 months with seed 7, stable-priority work
admission, a six-month planning horizon, at most 256 expansions and 32 complete
forecasts per personal search. The matrix uses the reference backend; separate
checks exercise CubeCL CPU execution.

The fixture supplies complementary access through generic rights and processes:

- Person 88 has woodland access, can produce three fuel units with one month's
  labor, and begins with three grain and four fuel.
- Person 89 has crop access and one seed, and begins with six grain and three fuel.
  A crop requires one seed and one labor unit in each of three months, returning
  eight grain and one seed.
- Each person has one labor unit per month, needs one nutrition and one warmth,
  and has storage capacity 64. Existing deprivation rules apply: shortfall damage
  2, recovery 1, impairment threshold 4, retained capacity 500 per mille and terminal
  threshold 12. Shortages can therefore reduce production and kill participants.
- Two supplied listings exchange one grain or one fuel for one coin. Person 88
  buys grain and sells fuel; person 89 does the reverse. Buying and seller protection
  both use a two-month horizon, distinct from the six-month planning horizon.
- No minting, loans, external traders, land acquisition or dues subsidize the run.
  Access and listing roles are supplied; occupational specialization is not discovered.
  Trading-disabled controls set match limits to zero with identical endowments.

Observed results with trading enabled:

| Search | Forecast | Coins each | 88 food / warmth deficit | 89 food / warmth deficit | Terminal person-months, 88 / 89 | Grain / fuel volume |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Beam | Current boundary | 1 | 7 / 1 | 0 / 7 | 13 / 13 | 2 / 2 |
| Beam | Current boundary | 2 | 7 / 0 | 0 / 7 | 14 / 14 | 1 / 1 |
| Beam | Current boundary | 6 | 7 / 0 | 0 / 7 | 14 / 14 | 1 / 1 |
| Beam | Standing policy | 1 | 7 / 1 | 0 / 7 | 13 / 13 | 2 / 2 |
| Beam | Standing policy | 2 | 6 / 0 | 0 / 7 | 16 / 14 | 0 / 1 |
| Beam | Standing policy | 6 | 6 / 0 | 0 / 7 | 16 / 14 | 0 / 1 |
| Best-first | Current boundary | 1 | 10 / 0 | 3 / 9 | 5 / 7 | 7 / 6 |
| Best-first | Current boundary | 2 | 8 / 0 | 0 / 7 | 11 / 14 | 3 / 1 |
| Best-first | Current boundary | 6 | 8 / 0 | 0 / 6 | 11 / 16 | 3 / 0 |
| Best-first | Standing policy | 1 | 1 / 0 | 0 / 1 | 0 / 0 | 20 / 20 |
| Best-first | Standing policy | 2 | 6 / 0 | 4 / 7 | 11 / 12 | 5 / 3 |
| Best-first | Standing policy | 6 | 7 / 0 | 0 / 6 | 12 / 16 | 3 / 0 |

All twelve trading-disabled runs have the same outcome: person 88 food deficit 6,
person 89 warmth deficit 6, the other two deficits zero, terminal reporting in 16
months for each person, and no exchange. Opening coins remain unchanged.

Deficits sum reported unmet need quantities. Terminal person-months count repeated
terminal reports, not distinct deaths. Post-terminal reporting means a lower deficit
alone does not establish better welfare. Market volume is counted once per listing;
the CSV repeats it on both person rows, so those columns must not be summed across
persons.

The successful control completes seven crops and nineteen fuel processes, exchanges
20 grain and 20 fuel, and ends with one coin each. There are two startup shortfall
units in total and no terminal conditions. Its private expected-but-unfilled metric
sums to 2; the six-coins standing best-first run sums to 11. This metric sums positive
expected-minus-actual quantities by actor/listing/side/month; it is a forecast error
measure, not aggregate economic shortage, and both sides of a trade can contribute.

A plausible explanation for the low-cash result is that future buying depends on
selling under the consistent own-order forecast. With more cash, private plans can
postpone selling while expecting ordinary counterparty supply; both actors' live
choices need not satisfy those expectations. The receipts demonstrate withheld
orders and expected/actual mismatches, but this comparison does not isolate every
cause or establish that more liquidity generally harms trade. Beam fails every
arm, and the standing variant worsens some controls. Search ordering, continuation,
finite horizon and private scoring remain material limitations.

## Verification and reproduction

**141 tests passed across 21 regression suites, including nine new exchange tests.**
The new coverage demonstrates repeated production and exchange with finite money,
the matched no-trade control, differing own-order forecasts, explicit withholding,
complete participant masks and admission gates, opening-cash limits, rejection of
unfilled production inputs, atomic rejection of forged receipts, and early rejection
of unsupported review modes.

The CPU/reference comparison runs twelve months, reverses participant/definition/
right/trader registration order, and resumes a checkpoint before Productive with
dated work pending. State, ledger, coordinator receipts and per-agent double-entry
audit agree; separate books reconcile without silently consolidating the persons.
Formatting, strict all-target Clippy, local documentation links and repository
artifact checks pass. The full v1 runner and annual 32-person stress test were not
rerun for this scoped change.

From `exp/economics`:

```sh
mkdir -p ../../output/economics
cargo +1.92.0 run --locked --release --example planner_exchange > ../../output/economics/planner-exchange.csv 2> ../../output/economics/planner-exchange.log
cargo +1.92.0 test --locked --release --test planner_exchange --test planner_persons --test planner_continuation --test composition --test planner_calibration --test composition_market --test town_market --test production_market --test cooperation --test acquisition --test household_market --test household_offers --test process_offers --test forecast_context --test planning --test search --test financial_offers --test competition --test allocation --test resolution --test intermediary
cargo +1.92.0 fmt --check
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Raw CSVs and logs stay under ignored `output/`. Forecast-month counts exclude
cumulative admission previews and are not runtime benchmarks.

## Remaining scope and next experiment

Supported integration is a plain individual town book with monthly review and
existing productive rights. Household/finance composition, market land acquisition,
retained market plans, conditional trade/work contracts, endogenous quotes and
counterparty-production forecasts remain outside this comparison. Fixed quotes
here do not establish performance with ZIP. No global welfare optimizer or new
centralized planner was introduced.

A next bounded experiment is to replace ordinary assumed peer submissions with
explicit, swappable counterparty expectations informed by completed order/fill
observations, while keeping these same endowments and budgets. Compare it against
this conditional baseline and preserve the failing beam/higher-cash controls.
Reliable posted commitments are another possible approach. Neither extension is
implemented in this original comparison; making forecasts agree with likely
counterparties is the gap these results expose. The subsequent
[observed-expectations comparison](PLANNER-EXPECTATIONS.md) implements an expiring
submission hypothesis and preserves the failed controls. It reduces some optimistic
errors but does not solve coordination; conditional commitments remain proposed.
