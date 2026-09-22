# Negotiated pricing: first bilateral pilot

Implemented opt-in, dated negotiation between two generic agents. One offers a
whole stock lot; the other pays in an existing stock resource used as coins.
The CPU example uses two people and grain. No state agent sets their price.

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example negotiation
```

## Terms and quote policies

`World.negotiation` holds one `Session`: month, buyer, seller, goods and quantity,
payment resource, and a bounded number of quote rounds. Each trader supplies an
opening quote, a private reservation limit, and a `QuotePolicy`:

- `Fixed`: retain the opening quote after rejection.
- `Concede { ticks }`: raise a rejected bid or lower a rejected ask by a positive
  number of payment ticks, clamped to that trader's own reservation limit.

The buyer's limit is the maximum payment for the entire lot. The seller's limit
is the minimum acceptable receipt. Neither update reads the counterparty's
private limit. Both updates follow the same public non-crossing quote pair, so
agent table order does not give one side an extra turn.

Quotes and limits use integer payment units **per lot**, not per grain unit.
This avoids floating-point settlement and allows fine prices by choosing small
coin denominations. The fixture treats balances as coin ticks; no conversion or
currency issuance occurs during exchange.

When bid reaches or exceeds ask, both parties accept the midpoint, rounded down
to a payment tick. That price remains within both quotes and both reservation
limits. This midpoint rule is an explicit bilateral price rule, not a claim to
implement an order-book auction. Stop on agreement, unchanged quotes, or the
configured round limit (at most 64). A limit on rounds counts quote observations,
including the initial quotes. It does not advance simulated months.

This is deterministic concession bargaining, **not ZIP**. Policy selection is
separate from transfer construction and settlement. A later ZIP policy will need
persistent trader learning state and a defined public market event stream; merely
renaming concession steps would not implement it.

## Timing, authority and settlement

The pilot owns an isolated Acquire window in the existing monthly scheduler:

1. Open makes the current boundary visible; any existing Due phase still precedes
   Acquire.
2. Acquire reads current holdings and the scheduled session. Both agents must be
   active and permitted to perform `StockTrade` under the existing state policy.
3. Quote rounds change no holdings and reserve no resources.
4. Crossing quotes must still pass full-lot seller stock, buyer payment and joint
   receiving-storage checks. Failure records a reason and emits no transfer.
5. Existing `finance::exchange_payment` builds both transfer legs. Existing
   settlement gathers their effects with the CubeCL CPU backend and atomically
   publishes goods and payment. Productive and later phases see the result.

The ledger's acquisition batch retains all quote pairs and the outcome. Commit
recomputes the expected negotiation against its dated opening state, comparing
both the receipt and the complete transaction list. Altered quotes, altered
prices, missing receipts, duplicate transfers and changed funding fail before
publication. Existing batch IDs and phase checks prevent replay. These are
trusted simulation records, not an authenticated external trading API.

There is one indivisible lot: no partial fills, no temporary credit, and no use
of incoming money to fund another purchase in the same boundary. No-trade rounds
advance the normal acquisition barrier without changing resources. The configured
session occurs only in its scheduled month; later months do not repeat it.

This driver is opt-in. Existing posted-price exchange, contested opportunities,
shared-pool markets, equipment offers, access offers and households cannot be
combined with it yet. Their acquisition requests need a common reservation
boundary before composition. Existing scenarios leave negotiation unset and
retain their prior behavior.

## CPU comparison

Buyer starts with 100 coin ticks and room for two grain units. Seller has two
grain units. Buyer bids 20 with a limit of 50; seller asks 60 with a limit of 30.
Both concede five ticks after rejection. The example uses ten possible rounds.

| Case | Observed quotes (bid/ask) | Outcome | Goods transferred | Payment transferred |
| --- | --- | --- | --- | --- |
| Concessions | 20/60 → 25/55 → 30/50 → 35/45 → 40/40 | Agreement in round 5 | 2 grain | 40 ticks |
| Fixed quotes | 20/60 | No agreement | 0 | 0 |
| Buyer limit reduced to 25 | Eventually 25/30 | No agreement | 0 | 0 |
| Only two rounds allowed | 20/60 → 25/55 | No agreement | 0 | 0 |
| Buyer has only 39 ticks | Reaches 40/40 | Insufficient payment | 0 | 0 |
| Buyer has no storage room | Reaches 40/40 | Insufficient storage | 0 | 0 |

Successful settlement leaves the buyer with two grain and 60 ticks, and the
seller with zero grain and 40 ticks. Each gains ten ticks of surplus relative to
its supplied reservation value. This is a valuation comparison, not newly issued
money. Total grain and total coins are unchanged.

All six comparisons ran on CubeCL CPU. Seven new tests cover these boundaries
plus missing goods, trading permissions, equal limits, initially crossed quotes,
large concession steps, invalid terms, tampered receipts, atomic rejection,
CPU/reference agreement and reordered tables. Monthly, batched and in-memory
checkpoint-resumed runs agree and do not repeat the trade. Another 32 regression
tests pass across finance, pricing, economics, conditions and consequence
priority. Formatting, Clippy with warnings denied and artifact checks pass.

## Limits and next extension

The scenario supplies orders and reservation values. Agents do not yet derive
willingness to pay from need deficits, replacement opportunities or expected
future income. The example isolates exchange and contains no production or
consumption. Quotes reset only when a new session is configured; there is no
learning across sessions, negotiation cost, strategic signaling, competing
counterparties or welfare claim about the resulting price.

The next useful extension is to generate a buy order from a food deficit and a
sell order from surplus after protected needs and commitments. Keep valuation,
quote policy, counterparty allocation and settlement separate. Once repeated
orders and observed market events exist, compare an implemented ZIP policy with
this simple concession baseline using the same starting resources and orders.
