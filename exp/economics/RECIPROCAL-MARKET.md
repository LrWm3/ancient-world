# Grain and wood exchange

Implemented extension of [production-market planning](PRODUCTION-MARKET.md).
The same four people can now buy grain and sell fuel wood, or do the reverse,
through one marketplace. This is a bounded two-good pilot with fixed quotes;
it does not yet combine adaptive roles with ZIP.

## Shared boundary and separate books

The venue retains its existing person-type, permission and month-start locality
requirements. Its second listing exchanges one fuel wood for two coins. Grain
remains two units for four coins. Per-unit prices are equal; wood's smaller lot
allows selling a one-unit surplus above the two-month need reserve. Initial
holdings, competencies, labor, plot rights, storage and production definitions
are unchanged from the grain-only scenario.

All orders are generated from the same opening observation. The books then clear
against a single acquisition budget for actual stocks, coins and weighted storage.
Incoming coins cannot fund another purchase within this Acquire boundary. Sold
stock can free storage for a subsequent purchase, but newly bought goods cannot
be resold at this boundary. Orders and hypothetical sales never credit money.
The complete set of trades settles atomically before productive work.

`ClearingPriority::MarketId` gives the lower market ID first access to shared
resources; `ReverseMarketId` is a controlled alternative. Both use exactly the
same opening requests. This is an explicit allocation policy, independent of
monthly timing. Neither policy claims fairness or simultaneous market equilibrium.
Each person can fill at most one order per listed good per month. Price/agent-ID
ranking within each book is unchanged.

Orders carry a market ID. Monthly receipts keep price, completed quantity and
unfilled quantities in a map keyed by market ID: grain and wood quantities are
never added into a meaningless combined volume. Marketplace quote memory was
already keyed by market and side. One listing's price or volume cannot become the
other listing's observation. No successful match means no new posted price.

The pilot supports at most two distinct goods, a common payment resource, and the
same registered people across both listings. Historical state reconstruction
expects the configured listing set; dynamic listing addition/removal is not part
of this change.

## Work and purchase alternatives

The planner now crosses its four work choices with four purchase policies:
none, all listed goods, grain only, or wood only. This gives 16 alternatives.
A person can deliberately retain the ability to buy food while offering its own
wood surplus. The single-good case retains eight alternatives. Existing work,
seed requirements, rights and two-month seller reserves still constrain execution.
No job, permanent buyer or permanent seller is assigned.

Each market has its own six-month observation window. `DemandSignal` selects
`CompletedOnly` (retained by the original single-good scenario) or
`IncludeUnfilledBids` (used by all controls in this comparison). Completed lots and unfilled
bid lots are recorded separately. The larger historical maximum bounds potential
future matching. Unfilled bids represent interest, including bids which might
lack funding; they are not guaranteed demand. Every hypothetical match must still
find goods, a counterparty, money, permissions, locality and space in the rollout.
The last completed price values at most one closing surplus lot of that good;
without a completed price it contributes no surplus valuation.

This interest signal addresses a cold-start limitation in the preceding planner:
zero completed trades had implied zero future sales, even when requests went
unfilled. Both the grain-only and reciprocal controls use the new rule. The
change does not invent a price, a loan, a buyer or a spendable receivable.

Forecasts still assume other people use ordinary need-directed work and allow
purchases. All live people instead replan independently each month. Predictions
can consequently disagree with actual supply and demand. This remains an explicit
limitation, especially when several people simultaneously change activities.

## Reproduction

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --example reciprocal_market
cargo +1.92.0 test --locked --test reciprocal_market --test production_market --test town_market --test need_orders --test acquisition --test negotiation --test zip
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

The example runs 48 months on CubeCL CPU for no trade, grain only, both listed
goods, and a directed diagnostic. `CASE=none`, `CASE=grain`, or `CASE=both` selects a control; `MONTHS=12`
shortens it. `CASE=directed` selects the diagnostic: crop experts prefer
cultivation and buy wood, while wood experts prefer collection and buy grain.
Those fixed policies test whether complementary production can settle; they are
not an autonomous-planning success or permanent agent types. Annual summaries show need deficits, labor, each resource's completed
volume, failures and individual coins. Monthly receipts retain both books.
`DETAIL=1` includes balances and full candidate forecasts. Raw output belongs in
ignored `output/`.

## Findings

All four 48-month CPU controls retain 96 total coins, have zero nutrition deficits
and record no failed processes:

| Control | Warmth deficit | Grain traded | Wood traded | Labor | Ending coins (88, 89, 91, 92) |
| --- | ---: | ---: | ---: | ---: | --- |
| No trade | 50 | 0 | 0 | 382 | 24, 24, 24, 24 |
| Grain only, autonomous | 12 | 36 | 0 | 384 | 48, 48, 0, 0 |
| Grain + wood, autonomous | 8 | 32 | 0 | 384 | 48, 48, 0, 0 |
| Grain + wood, directed diagnostic | 0 | 100 | 94 | 380 | 30, 30, 18, 18 |

The autonomous two-good case never completes a wood trade during these 48 months.
Its reduced warmth deficit reflects different projected alternatives/work choices,
not realized reciprocal exchange. It has not solved the concentration of coins.

The unchanged no-trade control reaches 50 warmth-deficit units over 48 months.
The grain-only case reaches 12, with zero nutrition deficits and zero process
failures in both. Net grain buyers run out of coins by month 36. At month 48 the
coin distribution is `[48, 48, 0, 0]`; total grain traded is 36. Some later trade
between the remaining funded people is possible, so this is not a claim that
the whole venue stops trading.

Adding a wood listing alone did not produce wood trades in the twelve-month
trial. Allowing unfilled bids to signal future demand, and reducing wood lots
from two units/four coins to one unit/two coins, also did not activate exchange
within the first twelve months. These were bounded diagnostics, not evidence
that smaller lots or better demand information are generally ineffective.

The directed diagnostic does sustain reciprocal trade: 100 grain and 94 wood
exchange over 48 months, with no food/warmth deficits or failed processes. Coin
holdings settle at `[30, 30, 18, 18]` at each annual observation. Its total labor
use is 380, versus 382 with no trade and 384 in the grain-only case. It uses the
same initial assets, skills, needs, prices and settlement rules; only work and
purchase selection are fixed. This separates operational exchange feasibility
from the harder problem of independently discovering that arrangement.

The autonomous planner must not be described as having achieved specialization
just because this control works. Its conditional forecasts can show wood sales
which do not materialize when all people replan, and a later decision can decline
to offer the anticipated stock. Neither the marketplace nor the treasury fills
that gap. The next planning experiment should compare counterparty assumptions
and persistence of dated plans against these same physical and financial controls,
rather than adding more goods and declaring the coordination problem solved.

## Verification scope

Controls exercise competing purchases against one cash budget under both clearing
priorities, shared storage, exclusion of incoming sale proceeds until the next
month, buying one good while offering another, separate prices and observations,
replay and atomic rejection of altered second-market transfers or receipts,
incompatible catalogs, and unfilled interest with no funding. Test lots are
explicitly set to two units/four coins where needed to isolate equal-cost
competition; the demonstration's wood lot is one unit/two coins.

CPU execution with reversed people, process/resource catalogs and venue listings
matches reference batched execution and reconstruction at Acquire. Existing
single-market controls retain their economic assertions after migration to
market-keyed receipts. Historical forecast transcripts are omitted only from
private fixed-policy rollout copies; observations and accepted processes survive,
and the live audit trail remains intact.

All 63 focused tests passed, including nine new reciprocal-market controls and
existing production, town-market, need-order, acquisition, negotiation and ZIP
regressions. Strict all-target Clippy and the repository artifact check passed.
The full crate suite was not rerun for this extension.
