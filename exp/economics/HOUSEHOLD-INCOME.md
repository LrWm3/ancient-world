# Household needs and market income

Implemented: opt-in `NeedsThenIncome` extends the existing contributed-labor
allocator. It compares allocations by members' current unmet needs, then expected
collective cash change through the next local town book. It uses the existing
process, household transfer, market and settlement machinery.

This is a bounded operating policy. Available member work and stock targets remain
supplied. It does not yet discover businesses, negotiate employment or jointly
optimize several months of production and credit.

## Authority and decisions

The constitution must explicitly permit `NeedsThenIncome`, and the charter or an
authorized dated governor instruction selects it. Existing founding templates and
default legal policy sets are unchanged. A state with explicit founding limits must
also recognize the new policy. It requires percentage-contributed labor and a town
book; the legacy spare-labor policy remains separate.

At the existing Productive reservation boundary, the household considers the same
recipient alternatives as before. Current permissions, the constitutional activity
mandate, real inputs, storage and contributed hours bound each alternative. Existing
commitment protection remains in force. Food, warmth and other fulfillment deficits
compare in their existing priority order; a better monetary return cannot override
a worse current-needs result. Equal needs and equal expected cash release the extra
labor rather than favoring output solely for its physical quantity.

Both `NeedsFirst` and `NeedsThenIncome` authorize collective consumption purchases.
The distinction is their secondary work objective: the former values net output;
the latter values expected net cash change. Financial-statement profit is reported
separately. This short policy does not price every private input's opportunity cost.

## Forecast and settlement boundary

Each alternative privately settles the candidate productive batch, pools its actual
outputs, satisfies current consumption, closes the month, and evaluates the next
Open and Acquire boundaries. It stops before another Productive phase, avoiding
recursive household planning. The standard forecast context removes unpublished
future fixture capacity and discretionary funding.

The forecast observes current balances, permissions, positions and accepted terms.
Counterparty decisions and quotes follow the existing market rules in that private
branch. The largest sum of completed volume and unfilled bids in the last six books,
including this month's already-completed book, bounds the next predicted volume.
Configured match limits still apply. This is a full-information, short-term
hypothesis: bids and previous sales are evidence of interest, not purchase promises.

All hypothetical trades still need a permitted, local buyer with actual opening
funds and storage, and crossed quotes. Current production cannot enter the market
that already cleared this month. Expected future receipts neither fund current work
nor appear in the journal. Live settlement repeats its normal checks, so a changed
permission or buyer budget can invalidate an earlier sales projection.

Labor receipts retain baseline and selected forecasts: observation month, next-book
month, demand caps, opening/closing coins, net cash change and traded quantities.
The external settlement observer exports them. Replay recomputes the forecasts and
rejects altered projections or transfers atomically.

## Original control and coordinated CPU scenario

Two adults form a household, while two independent people supply grain. The household
buys two grain for 40 ticks monthly. One external producer needs one fuel monthly
and bids 40 ticks for it. That producer replenishes grain through a supplied process
which consumes and returns seed. Fixed quotes isolate the work objective from price
learning; these prices are not demonstrated equilibrium values.

Each household member has five labor units and contributes one. A supplied member
fuel activity requires six units and produces two fuel: the allocator can direct
both contributions to that worker, adding one donor hour to its available five.
Half the fuel enters the collective account and half remains private. Existing
private work and commitment checks still apply. The collective unit can be sold
at the following month's book. No wage or private-to-household sale is invented.

| Months | Grain bought monthly | Fuel sold monthly | Closing household coins | Member food deficit |
| --- | --- | --- | --- | --- |
| 1 | 2 | 0 | 60 | 0 |
| 2–24 | 2 | 1 | 60 | 0 |
| 25 | 2 | 1 | 60 | 0 |
| 26 | 2 | 0 | 20 | 0 |
| 27–36 | 0 | 0 | 20 | 2 |

The original 12-month control completes with all household food needs met, unchanged
total coins and reconciled separate-agent statements. The longer run exposes a
remaining coordination failure: private fuel reaches its supplied target of 24
at the end of month 24. The member stops requesting work in month 25, although the
household still needs sales. The last unit sells in month 25; affordable food ends
after month 26. Unused private stock is not silently appropriated for collective
sales. The fuel buyer also lacks warmth in the opening month before the first sale.

The original failure remains a control. The coordinated variant now closes this
connection through dated, voluntary member surplus support: protect two private
fuel, offer at most one per month toward a one-unit collective target, and accept
only useful support before deciding whether extra labor is needed. The target of
24, technology, starting money and prices are unchanged. A 120-month CPU/reference
comparison keeps both members fed, conserves coins and reconciles separate books.
See the [completed initial loop](PERSON-HOUSEHOLD-LOOP.md) for consent, timing,
withdrawal, shock recovery, receipts and limitations. The example now defaults to
this coordinated variant for 36 months.

## Verification

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example household_income
cargo +1.92.0 run --locked --example household_income -- 36
cargo +1.92.0 test --locked --test household_income
```

Controls cover no demand, unfunded buyers, uncrossed prices, current food priority,
missing inputs, unlawful/out-of-mandate work, insufficient real contributions,
dated policy changes and lost counterparty permission. CPU/reference execution,
reordered tables and checkpoint continuation agree. Separate double-entry accounts
reconcile, observer output is read-only, and forecasts create no spendable money.
The finite private-target failure is retained as a regression test.

Verification on 2026-09-29: **162 distinct tests passed** across 11 selected
household, market, accounting, law and telemetry suites, including all 13 final
income-policy tests. Both CPU examples (12 and 36 months), strict all-target Clippy,
formatting and repository artifact checks passed. One slow annual household
accounting test remained ignored; the full crate suite was not run. Raw logs are
under ignored `output/economics/household-income-*.log`.

The original verification record above predates the coordinated variant; see the
[loop verification](PERSON-HOUSEHOLD-LOOP.md#reproduce-and-verification-scope) for that extension.

The static charter can now set `cash_target: Some(Amount)` in town-payment units.
Equal-needs alternatives compare projected closing cash capped at that target;
omitting it retains net-cash maximization. Both labor and voluntary support use
the same objective. This is a supplied founding parameter, not self-directed
personal policy revision. NeedsFirst support also operates without market income.

[Integration passes](INTEGRATION-PASSES.md) now cover outside member wages,
private member surplus sales and direct loans in the town budget. Remaining limits
include endogenous activity targets, speculative input purchases, multi-period
investment, household hiring and negotiated price calibration. Ordinary work and
the existing town account are reused; the separate joint production-market planner
remains excluded.

The [second integration batch](INTEGRATION-PASSES-2.md) adds partial support under
storage limits, physical barter proceeds and charter-directed purchase routing.
Collective input funding covers existing active processes; it does not infer new
investment plans from the income objective.
