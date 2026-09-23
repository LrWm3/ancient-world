# Four-person town market

Implemented: an opt-in monthly book with two buyers, two sellers and one food
listing. Participants generate orders from needs and protected surplus; the market
chooses counterparties. This extends the [need-order pilot](NEED-ORDERS.md) without
combining its previously excluded household, credit or legacy exchange drivers.

## Boundary and matching rules

The marketplace remains an ordinary agent with its existing type requirement and
trade catalog. The fixture requires person agents. Its town is a named agent with
a position; registered traders have positions in the market state. Coordinates
are one-dimensional abstract distance units, not a transport model.

At **Open**, record the living, permitted traders within the configured distance
of the town. Missing positions deny admission. This dated receipt stays fixed for
the month: moving closer after Open does not gain immediate entry, and moving away
affects the next admission check. Current permission and lifecycle are rechecked
at Acquire. Admission grants neither resources nor a new right to trade.

At **Acquire**:

1. Generate at most one whole-lot order per admitted trader from opening holdings.
   A buyer requests the lot only when it reduces a current need deficit. A seller
   offers it only when surplus remains after the shared need-order protection rule.
2. Freeze that month's quotes. Rank bids highest first and asks lowest first;
   ascending agent ID breaks equal-price ties. This is an explicit deterministic
   allocation rule, not a fairness or welfare guarantee.
3. For each buyer, try remaining sellers in ask order. Crossed quotes settle at
   their midpoint, rounded down to the listing's payment tick. Failed funding or
   storage leaves the seller available for subsequent attempts. An uncrossed best
   remaining ask ends that buyer's search; higher asks cannot cross either.
4. Reserve each successful trade against shared opening money/goods and net storage.
   Incoming proceeds cannot fund another outgoing transfer in the batch. Each trader
   fills at most one lot, even if it has stock or money for more.
5. Recompute the complete receipt and transaction list at settlement. Publish all
   transfers, price records and learning together, or publish none on validation
   failure. Consumption later sees the committed holdings.

Orders expire at the end of the book and are regenerated next month. The configured
roles, lot size, private price limits and registration list remain supplied. Need
and surplus determine whether to submit, not the willingness-to-pay limit. There
are no partial fills, simultaneous buying/selling by one trader, general order-book
persistence or multiple listings in this pilot. The bound is 32 registered traders.

## Prices, volume and learning

Each successful pair pays its own match price. The **last completed match price**
is the month's posted observation, not a uniform clearing price imposed on earlier
trades. If prices are 45 and then 40, the first buyer still pays 45. This explicit
first rule can be replaced in a later experiment.

The monthly receipt records submitted orders and protection, attempted matches,
actual transactions, grain volume, unfilled buy/sell quantities and an optional
posted price. No completed trade means `None`, even when earlier months had prices.
An unsuccessful crossed quote is not volume or a price observation. Old observations
remain in dated history. No-demand and ineligible traders submit no order and are
not counted as unfilled demand.

Fixed and ZIP quotes are selectable independently per trader. ZIP keeps the existing
market/participant/side memory; it does not attach memory to a counterparty pair.
Orders retain their opening quotes throughout the book. Learning processes attempted
pair events in order and affects later books, including when one seller encounters
an unfunded buyer before a successful trade. Unattempted orders produce no learning
event. This is a bounded bilateral event stream within matching, not a full broadcast
auction learning model. Concession rounds remain in the original bilateral pilot;
the monthly book accepts Fixed or ZIP only.

## Six-month CPU observation

Both buyers start with 100 coin ticks; both sellers start with ten grain. Each of
four people needs one nutrition unit per month. A two-grain lot satisfies two such
units through the existing consumption recipe. Sellers protect two months of needs.
Bids are 50 and 40 ticks per lot, asks 30 and 40. All begin at the town.

| Month | Posted ticks per lot | Grain traded | Unfilled buy | Unfilled sell | Total nutrition deficit |
| --- | --- | --- | --- | --- | --- |
| 1 | 40 | 4 | 0 | 0 | 0 |
| 2 | No new price | 0 | 0 | 4 | 0 |
| 3 | 40 | 4 | 0 | 0 | 0 |
| 4 | No new price | 0 | 0 | 0 | 0 |
| 5 | No new price | 0 | 4 | 0 | 2 |
| 6 | No new price | 0 | 4 | 0 | 2 |

The higher bid matches the lower ask; the remaining pair also matches at 40.
Each buyer ends with 20 ticks and each seller with 80. The initial twenty grain
are consumed: buyers consume eight and sellers twelve. Sellers have no nutrition
deficits; buyers each accumulate two. The fixture has no replenishing production
or deprivation condition rules. Reserve protection is demonstrated; sustainable
production and calibrated price discovery are not.

## Verification

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example town_market
cargo +1.92.0 test --locked --test town_market
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Controls cover competing buyers, stable ties under reordered registration,
permission and month-start locality, failed funding/storage followed by another
buyer, uncrossed/no-price books, actual conservation and per-match prices, forged
receipts, replay, buffer limits and invalid configurations. Fixed and ZIP CPU runs
match reference runs, batching and table reversal. Every monthly barrier supports
in-memory state reconstruction and continuation; ZIP also preserves sequential
learning across failed and completed attempts. All 12 new tests pass, alongside
61 selected acquisition, need-order, marketplace, negotiation, ZIP, condition and
core-economics regressions (73 total). Strict all-target Clippy passes.

## Scope and next integration

The implementation reuses permitted consumption projections, quote/ZIP policies,
marketplace memory, acquisition budgets and settlement validation. It introduces
an explicit monthly book and Open admission receipt, not a scheduler rewrite.
The main civilization scheduler is unchanged. World/state copies favor small-case
correctness over performance; this is not a device-resident matching benchmark.

Households, secured credit, legacy forwards/equipment exchange, competing-access
and pool-market drivers remain excluded from this book. The next integration should
supply household or other institutional orders through explicit budgets and mandates,
or add a production/credit case with resource reservations. Other open choices are
endogenous valuation, selectable matching/tiebreak policies, multiple lots, movement
and delivery costs, and the use of market observations in future projections.
