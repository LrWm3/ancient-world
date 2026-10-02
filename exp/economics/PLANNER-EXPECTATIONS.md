# Observed counterparty submission expectations

Status: implemented opt-in follow-up to [active planner exchange](PLANNER-EXCHANGE.md).
Planners can now use the last observed eligible counterparty order choice, with
configurable expiry, instead of always assuming ordinary order submission. This
reduces some optimistic fill errors but does not fix coordination. Longer memory
can entrench inactivity and worsen survival. **The ordinary expectation model
remains the default.**

## Model and information boundary

`composition::expectations::Policy` separates counterparty assumptions from the
agent's own order forecast, search strategy, economic score and live clearing:

| Policy | Hypothesis |
| --- | --- |
| `Ordinary` | Peers use ordinary eligible need-order generation, as before. |
| `RecentSubmission { memory_months }` | Use the last eligible submitted/withheld observation for each peer, listing and side while it remains fresh. Without fresh evidence, use ordinary generation. Zero memory is rejected. |

Configure `Persons::counterparty_policy` separately from `Persons::order_forecast`.
The latter controls the agent's own hypothetical submissions. Both current-boundary
and standing own-order forecasts support the new counterparty model. The pilot
selects one counterparty policy for the coordinator, with a separate actor-specific
snapshot for every search; it does not yet give every person an independent policy
configuration or learn probabilities.

At Acquire, `Snapshot::observe` reads only earlier settled town books. A `Submitted`
receipt is evidence of willingness even if the order was not filled. A
`PlannerWithheld` receipt is evidence of an eligible order deliberately withheld.
Ineligibility, protected stock, absent need and failed admission are not evidence
of unwillingness; they do not replace an earlier still-fresh eligible observation.
Each observation retains its month, submitted flag and actual filled quantity.
Fills are diagnostic evidence, not a separate probability or promised supply.

Evidence remains fresh when `forecast_month - observed_month <= memory_months`.
For example, a month-1 refusal with memory 3 applies in months 2–4 and expires in
month 5. Expiry operates inside the forecast as well as between live decisions.
A snapshot is frozen throughout candidate evaluation: hypothetical outcomes do
not train the model. Same-month and future books are excluded. Each actor receives
its snapshot before any live choice is committed, so search order cannot expose
another person's current proposal.

This uses the existing full-information experiment's detailed order receipts,
including eligibility and deliberate withholding. A real venue might publish
only orders and trades, in which case absent orders would not reveal this reason.
Private beliefs, partial observability and venue-specific disclosure are not modeled.
The experiment keeps venue/listing definitions fixed.

## Forecasts do not authorize live actions

Inside a private branch, the expectation supplies hypothetical peer submission
masks. Current eligibility, finite stock, money, storage and normal clearing still
apply. A predicted submission can fail to match or settle. Peer production is still
omitted; this extension does not promise another person's harvest or simulate that
person's decision policy.

The coordinator extracts only the searching actor's own chosen mask from its
private branch. Actual clearing receives the independently chosen masks from all
actors. It never receives another actor's hypothesis as live consent. New productive
work remains checked against actual fills before dated reservations are committed.
Open admission, Acquire clearing/reservation and Productive execution are unchanged.

`Exchange::expectations` stores the snapshot alongside the existing expected and
actual fill maps. Cloned coordinator checkpoints preserve these records. A failed
boundary publishes neither decisions nor learned snapshots. This is not a new
journal, durable checkpoint format, matching mechanism or centralized planner.

## Matched 24-month comparison

[The runner](examples/planner_expectations.rs) compares both search strategies,
ordinary expectations and recent-submission memories of 1 and 3 months, 1/2/6 coins
per person, and trading enabled/disabled: **36 runs, 72 person rows**. It retains
all [previous fixture settings](PLANNER-EXCHANGE.md#controlled-comparison): two
persons with complementary supplied rights, finite grain/fuel/seed and coins,
fixed one-coin lots, deprivation consequences, two-month buying/seller protection,
six-month forecasts, 256 expansions, 32 forecasts, seed 7 and stable work admission.
All arms use standing own-order forecasts. No resource, price, score or productivity
was retuned. The matrix uses the reference backend; CPU equivalence is tested
separately.

Trading-enabled results:

| Search | Peer model | Coins each | Total reported deficit | Terminal person-months | Grain / fuel volume | Expected but unfilled / unexpected fills |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Beam | Ordinary | 1 | 15 | 26 | 2 / 2 | 7 / 0 |
| Beam | Ordinary | 2 | 13 | 30 | 0 / 1 | 8 / 0 |
| Beam | Ordinary | 6 | 13 | 30 | 0 / 1 | 9 / 0 |
| Beam | Recent, 1 month | 1 | 13 | 30 | 0 / 1 | 2 / 1 |
| Beam | Recent, 1 month | 2 | 13 | 30 | 0 / 1 | 3 / 1 |
| Beam | Recent, 1 month | 6 | 13 | 30 | 0 / 1 | 2 / 1 |
| Beam | Recent, 3 months | 1 | 12 | 32 | 0 / 0 | 1 / 0 |
| Beam | Recent, 3 months | 2 | 12 | 32 | 0 / 0 | 1 / 0 |
| Beam | Recent, 3 months | 6 | 12 | 32 | 0 / 0 | 0 / 0 |
| Best-first | Ordinary | 1 | 2 | 0 | 20 / 20 | 2 / 0 |
| Best-first | Ordinary | 2 | 17 | 23 | 5 / 3 | 8 / 0 |
| Best-first | Ordinary | 6 | 13 | 28 | 3 / 0 | 11 / 0 |
| Best-first | Recent, 1 month | 1 | 4 | 0 | 19 / 19 | 1 / 2 |
| Best-first | Recent, 1 month | 2 | 17 | 23 | 5 / 3 | 1 / 3 |
| Best-first | Recent, 1 month | 6 | 14 | 27 | 2 / 1 | 5 / 3 |
| Best-first | Recent, 3 months | 1 | 4 | 0 | 19 / 19 | 3 / 3 |
| Best-first | Recent, 3 months | 2 | 15 | 25 | 3 / 2 | 7 / 5 |
| Best-first | Recent, 3 months | 6 | 12 | 32 | 0 / 0 | 1 / 0 |

All eighteen trading-disabled controls have zero volume, total reported deficit 12,
and 32 terminal person-months. Coins remain finite and conserved. The ordinary
arms reproduce the earlier standing-policy comparison.

Reported deficit sums food/warmth shortfalls across both persons. Terminal
person-months count repeated terminal reports, not distinct deaths. Post-terminal
reporting can lower reported deficits, so a lower deficit alone is not better
welfare. Volume is counted once per book; the CSV repeats it on each person's row.
The two forecast-error columns sum positive differences in each direction by actor,
listing and side at the current Acquire. These are one-unit-lot counts in this
fixture, not monetary values; the two sides of one trade can both count.

Best-first with one coin each preserves survival under both memories, but reported
shortfalls rise from 2 to 4 and exchange falls from 20/20 to 19/19. At two coins,
one-month memory reduces optimistic misses from 8 to 1 without improving survival
or volume, while adding 3 unexpected fills. At six coins, three-month memory nearly
eliminates optimistic misses by eliminating trade; both persons become terminal
earlier. Beam continues to fail in every arm.

The lesson is that accurate prediction of inactivity is not discovery of a mutually
beneficial plan. Remembered refusals can discourage the actions that would make
future trade useful. These runs also retain the earlier limits: private scoring,
short horizons, no peer-production forecast and supplied market roles. They do
not establish that observed expectations are generally harmful or that memory
alone caused every outcome. The results do not justify changing the default.

## Verification

Six new tests cover:

- Eligible withholding versus ineligibility and submitted-but-unfilled orders;
  expiry and exclusion of own, current-month and future evidence.
- Cold-start equivalence, snapshots from previous live books only, and live masks
  taken from their own actors rather than peers' predictions.
- Invalid memory and failed settlement leaving state and coordinator unchanged.
- Twelve-month reference/CubeCL CPU equality, reordered registrations, checkpoint
  resume before Productive, and reconciled separate double-entry books.
- Low-cash survival and the preserved high-cash coordination regression.
- Both own-order forecast variants, finite cash and unchanged search budgets.

Together with the existing planner, market, offer, allocation and settlement
regressions, **147 tests pass across 22 suites**. Formatting, strict all-target
Clippy, documentation links and the source-only artifact check pass. The full v1
runner and annual 32-person stress test were not rerun for this scoped extension.

From `exp/economics`:

```sh
mkdir -p ../../output/economics
cargo +1.92.0 run --locked --release --example planner_expectations > ../../output/economics/planner-expectations.csv 2> ../../output/economics/planner-expectations.log
cargo +1.92.0 test --locked --release --test planner_expectations --test planner_exchange --test planner_persons --test planner_continuation --test composition --test planner_calibration --test composition_market --test town_market --test production_market --test cooperation --test acquisition --test household_market --test household_offers --test process_offers --test forecast_context --test planning --test search --test financial_offers --test competition --test allocation --test resolution --test intermediary
cargo +1.92.0 fmt --check
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Raw results remain under ignored `output/`; this report is the committed record.
A useful next bounded comparison would let actors disclose an actionable conditional
offer, then test whether the existing agreement machinery can make complementary
plans reliable. Learning from completed books remains useful evidence, but cannot
by itself communicate a willingness to act if the other person also acts. That
extension is proposed, not implemented here.
