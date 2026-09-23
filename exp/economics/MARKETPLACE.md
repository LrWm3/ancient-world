# Marketplace agent: person-only exchange venue

Implemented as a component on an ordinary agent. The initial marketplace exists
at initialization, has no membership agreement, admission fee, formation or
dissolution flow, and owns no participant inventory. It facilitates exchange
between eligible people through the existing Acquire and settlement boundaries.

## Initial venue and trade catalog

The controlled negotiation scenario contains a state agent, two person agents,
and marketplace agent 90. Its catalog is:

| Market | Facilitates | Lot size | Price precision | Access requirement |
| --- | --- | --- | --- | --- |
| 1 | Grain in exchange for coin ticks | 2 grain | 1 coin tick per lot | Person agent |

An eligible person can be buyer or seller. Only this catalog entry is enabled:
wood, seed, tools, land rights and services are not yet listed. The catalog is
editable `Marketplace.markets` data, with stable market IDs, stock commodity,
lot quantity, payment resource and price tick. Adding a catalog entry declares
support; it does not create goods, counterparties or orders.

`marketplace::discover` returns the sorted catalog for an eligible participant.
The example displays the requirement and facilitated trades:

```sh
# From exp/economics
cargo +1.92.0 run --locked --example negotiation
```

## Eligibility and existing state rules

The venue's `required_type` is `PERSON_TYPE`. Classification comes from the
existing state transaction policy's `agent_types` table; there is no second type
registry and no membership list. An unclassified agent is ineligible. A state,
marketplace or other non-person remains ineligible even if its type has permission
to perform stock exchanges.

Both parties must also be active and permitted to perform `StockTrade` under the
existing state policy. The venue cannot grant an exception to state rules. These
checks apply to discovery and negotiation and are repeated during settlement.
The marketplace itself must be active. No new lifecycle or membership machinery
is introduced.

## Routing, recording and pricing state

A negotiation session names a marketplace and market ID. Its proposed goods,
lot size, payment resource and price precision must match that entry. Unlisted
terms produce `UnsupportedMarket` without quotes or transfers. Unknown venue
references are invalid configuration. Quote limits and increments must be aligned
to the listed price tick; a crossing price rounds down to that tick.

The venue's committed state stores:

- History of dated negotiation outcomes and quote pairs, including buyer/seller
  identity, market ID and failed attempts against supported venue identities.
- Pricing records keyed by `(participant, market, buy/sell side)` within that venue.
  Each record holds the quote policy, last quoted price and observation month;
  [ZIP records](ZIP.md) also hold margin, momentum and seeded random state.

There is no buyer/seller-pair pricing key: another counterparty in the same market
would encounter that participant's same side-specific pricing state. Buy and sell
strategies remain separate. Implemented strategies are fixed quotes, bounded
concessions and the opt-in [bilateral ZIP margin policy](ZIP.md).

When another session is explicitly scheduled, the same policy resumes its last
quote, clamped to the participant's current reservation limit. ZIP instead resumes
its learned margin and reprices it against the current limit. Changing policy
starts from the new session's opening quote. Quotes observed during no-agreement
or failed-settlement outcomes can be retained; an ineligible/unlisted attempt has
no quotes and therefore does not overwrite pricing records. No orders are
implicitly renewed and no further sessions are automatically generated.

Quote history and pricing updates are published on the same staged boundary as
transfer effects. Tampered receipts, changed eligibility or changed catalogs
reject the whole commit, including memory changes. A successful exchange debits
and credits the two people directly. The venue receives neither goods nor money.

## CPU results and checks

The existing six negotiation comparisons retain their results through the venue:
the baseline exchanges two grain for 40 coin ticks after five quote rounds;
fixed quotes, disjoint limits, a short deadline, missing payment or missing storage
leave goods and money untouched.

A two-session control starts with four seller grain and buyer storage for four.
The first exchange settles at 40 ticks. For month two the buyer lowers its limit
to 35: remembered quotes open at 35/40, then converge at 35/35. The buyer finishes
with four grain and 25 ticks; the seller has 75 ticks. The marketplace's balances
remain zero. CPU and reference checkpoint continuation match, including history
and pricing records.

Six marketplace tests cover eligibility, catalog discovery, unsupported terms,
price ticks, catalog validation, atomic revalidation, persistent side-specific
state and continued CPU settlement. The seven negotiation tests and five existing
finance/pricing tests also pass (18 total). Clippy with warnings denied and the
repository artifact check pass. Generated output remains under ignored
`output/economics/`.

## Current scope

This is a venue around the existing one-lot bilateral experiment, with supplied
orders and reservation values. There is no order book, simultaneous matching of
many participants, fee model or automatic market-making. ZIP learning is scoped
to the bilateral event stream described in [ZIP.md](ZIP.md). Existing
restrictions still apply except for the supported credit/state-bid combination
through the [shared acquisition resolver](INTEGRATION-STATUS.md). The public discovery
function exposes supported trades; need-driven opportunity search does not yet
create buy/sell orders from that catalog.
