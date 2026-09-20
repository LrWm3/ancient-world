# Storage, annual collection and currency

Implemented CPU experiment, September 2026. The [four-person extension](FOUR-PEOPLE.md)
adds three people and joint settlement of seller subsets. Fixed storage capacity is a generic
constraint on resource holdings. Tokens require no space. This extends the existing
annual plot agreement and acquisition boundary without adding a monthly phase.

## Rules and boundaries

- Each resource has a configurable nonnegative space weight; each constrained agent
  has a shared capacity. Usage is the sum of quantity × weight across its stocks.
  Unspecified weights are zero; unspecified agent capacities are unlimited, preserving
  earlier fixtures. These defaults require deliberate catalog configuration.
- The new fixtures allocate 15 storage units to the person and 32 to the state.
  Grain, seed, fuel and harvested wild food each occupy one unit. Tokens occupy zero.
  Raw wood and unharvested wild supply are outside this covered store in this pilot.
  Initial personal grain is five, plus one seed and one fuel.
- Productive resolution reserves output space alongside inputs and services, updating
  a shared space budget for accepted actions. Consumed inputs can release space;
  newly produced inputs still cannot fund another action in the same batch.
  Settlement checks the complete resulting state atomically.
- An action whose output cannot fit is rejected with `InsufficientStorage`.
  Existing processes retain the experiment's abort-on-failure behavior, so a blocked
  harvest loses its earlier sunk inputs. There is no overflow pile or automatic sale.
- Annual tax is two grain, first due in month 13 and then every twelve months.
  Due and ClearArrears transfer only available grain that fits in the treasury store.
  Uncollected amounts remain obligations.
- Each two grain actually paid against an annual installment authorize one new token
  in the state's treasury. Partial payments use the increase in `floor(paid / 2)`;
  replaying settlement cannot mint again. Late payment issues at actual collection.
  Purchased grain never counts as tax collection.
- At Acquire the state posts a bid: buy one grain for one existing treasury token.
  The person may accept one bid as a forecast candidate. Both parties' opening funds,
  goods and receiving space must suffice. Grain and token transfers settle together.
  Tokens issued at Due are available at that month's Acquire; tokens issued after
  production wait until the next acquisition boundary.

A token therefore buys the state one grain at this posted price. It is not a promise
that the state will redeem tokens for grain. There is currently no token spending
outlet or intrinsic token preference. The person sells only when the forecast finds
that exchanging food for space improves outcomes. Currency issuance alone cannot
satisfy nutrition or warmth.

## CPU comparison

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked -- storage-exchange
cargo +1.92.0 run --locked --example storage_currency_audit
cargo +1.92.0 test --locked --test storage_currency
```

The audit runs each arm for 60 months and compares full state, transaction ledger
and reports with the Rust reference. All four arms share opening stocks, storage,
foraging, productive capacities and a six-month decision horizon. The one-grain
arm is a control; the other three use the requested two-grain annual tax.

| Scenario | Tax collected | Grain sold | Tokens state / person | State grain | Harvests | Food / warmth deficits |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| storage-tax-one | 4 | 0 | 0 / 0 | 4 | 8 | 0 / 0 |
| storage-tax-two | 8 | 0 | 0 / 0 | 8 | 9 | 0 / 0 |
| storage-issuance | 8 | 0 | 4 / 0 | 8 | 9 | 0 / 0 |
| storage-exchange | 8 | 2 | 2 / 2 | 10 | 9 | 0 / 0 |

All four two-grain bills are collected. Four tokens are created; two are transferred
for two additional grain. The state ends with ten grain and two tokens; the person
holds the other two tokens. These results describe this fixture, not a general
benefit from higher taxation or currency.

A focused harvest test isolates the storage incentive: with tight storage, selling
one grain admits a harvest; with the bid disabled, that same harvest cannot fit.
With a larger store, the person declines the sale and still harvests. This separates
the space benefit from an assumed preference for accumulating money.

Seven focused tests cover zero-space tokens and weighted shared stores, partial
collection and late issuance, a full treasury, atomic exchange and finite funds,
storage-motivated choices, tampering/overflow rollback, CPU/reference equality,
ledger replay and checkpoint continuation. The four fiscal fixtures are exercised
for 36 months with replay and continuation checks, plus the 60-month CPU audit.

## Limits

Storage is a fixed per-agent constraint, not yet an asset that can be built, rented
or damaged. Its shared space is separate from labor allocation and ecological pool
capacity. No spoilage, transport, multiple storage locations or currency redemption
is modeled. The stock bid is generic but this experiment has one active seller and
one state buyer, with one indivisible posted purchase considered per decision.

The planner evaluates current acquisition alternatives and their consequences;
it does not assume additional purchases inside future forecast months. Six-month
lookahead can miss longer-term effects. Space is reserved at the current production
boundary only: future output can still encounter a full store.

Issuance rounds down independently for each annual agreement installment. Fractional
receipts are not pooled across taxpayers or years. This is exact for the current
single two-grain bill; a broader fiscal model would need an explicit aggregation
and rounding policy. Historical annual-access fixtures retain their one-grain rent
as controls; the new fiscal fixtures carry the requested two-grain setting.
