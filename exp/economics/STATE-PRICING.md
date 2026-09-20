# State price spread, forwards and tool productivity

Implemented for the cash trading scenarios, including `trading-32-plots`.
The legacy output-share and smaller historical control scenarios retain their
original catalogs. This supersedes the one-to-one price assumptions in the
[earlier forward experiment](FORWARD-TRADING.md).

## Prices

The current fixture assigns a base value of one coin to a unit of each traded
commodity. Prices are explicit scenario constants, not a market-clearing estimate.

| Transaction | Coins per commodity unit |
| --- | ---: |
| State buys actual goods | 0.75 |
| State sells goods back to people | 1.50 |
| State prepays future goods through a forward | 0.50 |
| Existing peer stock trades | 1.00 |

Seller-specific state quotes apply to the actual transaction, affordability
reservation and replay validation. A common peer bid cannot buy state inventory
at the cheaper peer price. The state still requires physical inventory and buyers
require opening coins; state purchases and advances use finite treasury funds.
Posted bids and existing inventory targets determine which goods are requested.
This change does not make everyone automatically buy every commodity.

Tax quantities, anniversary dates, commodity-or-coin conversion and issuance
rules are unchanged. An additional plot still costs two grain or two coins each
year. Two units of collected native grain still issue one token; paying tax in
coins does not issue currency. Taxes are not repriced at 0.75 or 1.50.

For example, a one-coin advance pledges two units of future goods. Those two
units could sell for 1.50 coins on the spot market. Their foregone spot value,
not just the one coin received, is the borrower's economic financing cost.

## Making the tool purchase worth financing

Tool prices remain 25% of the spot-valued output assisted by the offered tool
over the next 12 months. Spot valuation uses 0.75; the separate forward schedule
uses 0.50. These are separate tables so lowering forward proceeds cannot also
silently lower the tool's output valuation.

Before purchasing, the buyer forecasts the same local work policy with and
without that tool. The difference in closing commodity stocks at spot prices,
plus any difference in coins, estimates its incremental economic benefit. A
purchase is accepted only when that benefit exceeds:

- the buyer's own coins spent; plus
- the spot value of goods promised to finance the shortfall.

The usual projected-surplus, finite treasury, receiving-storage and one-open-
forward constraints still apply. `NotEconomic` records affordable purchases whose
forecasted benefit is insufficient. Idle labor is not assigned an invented cash
value, and a tool is not purchased just because it reduces a recipe's labor cost.
The forecast is local, deterministic and bounded; future competition, price
changes and unexpected disruptions can defeat it. It is not an optimal plan or
a complete valuation of health, leisure and future assets.

## Productivity adjustment

Cash-market labor now has quarter-unit precision. All base labor capacities and
manual service costs are scaled together, preserving physical manual work.
Equipment-assisted variants then require half their previous labor: for example,
a one-unit tool requirement becomes 0.5 units, and a two-unit requirement becomes
one unit. Capacity receipts contain labor ticks; four ticks equal one labor unit.
Need fulfillment, process durations and tool lifetimes are unchanged.

In addition, a hoe used for the final crop stage doubles harvested grain, and a
comb used for livestock collection doubles milk/wool output. This is a generic
technique output multiplier applied to non-recycled output at completion. It
does not multiply the returned planting seed, manufacture ore from a doubled
mineral deposit, stack across stages, or award a harvest bonus when the final
stage is manual. Mining, fishing, construction and toolmaking benefit from the
lower labor requirement without an output multiplier in this calibration.

The resolver and settlement validator use the same output calculation, including
storage limits and overflow checks. Tool projections value actual forecast
outputs, so they include the yield benefit when it really occurs. Output-share
controls retain multiplier one.

## Reproduction and results

From `exp/economics` (optimization only speeds up host-side forecast rollouts):

```sh
cargo +1.92.0 run --locked --config 'profile.dev.package.economics-compute-smoke.opt-level=2' --example forward_audit -- 72 trading-32 cpu
cargo +1.92.0 run --locked --config 'profile.dev.package.economics-compute-smoke.opt-level=2' --example forward_audit -- 72 trading-32-no-forward
cargo +1.92.0 run --locked --config 'profile.dev.package.economics-compute-smoke.opt-level=2' --example forward_audit -- 72 trading-32 reference 3 old-tools
```

The comparison holds 32 people and three providers fixed. Under the new prices
and stronger tools, the 72-month run delivered 37 tools for 137.98 coins, including
15 forwards advancing 27.24 coins. All promised commodities were delivered; no
forward balance remained at the end. There were no nutrition/warmth deficits,
unpaid taxes or terminal people. Providers ended with 15/13/13 grain and
5.87/3.62/4.24 coins. Final state, every ledger batch and all monthly reports
matched exactly between CubeCL CPU and the reference backend.

Without forwards, the same stronger-tool scenario delivered 24 tools for 57.72
coins. It also ended with no nutrition/warmth deficits, tax arrears or terminal
people. Financing therefore enabled additional purchases under this fixture's
coin constraints. It does not prove that every tool is worth buying: the financed
run still declined 602 attempts as uneconomic and 1,532 for no projected use.
These are repeated attempts, not counts of people.

Under identical new prices with the old tool benefits restored, only eight tools
sold for 19.11 coins and none used forwards. That matched control also had no
nutrition/warmth deficits, tax arrears or terminal people at month 72. The revised
benefits therefore change the economic case for financed purchases; they are not
needed merely to force agents to remain alive.

A concrete accepted forecast is a six-coin hoe bought with four existing coins
and a two-coin advance. The advance pledges four grain, worth three coins at the
spot bid. The forecast estimates 12 coins of incremental stock value versus seven
coins of economic cost, leaving a five-coin margin. The future receipt is still
uncertain; actual repayments are tracked separately.

Provider counts are held fixed for this comparison; ending solvency does not
establish a long-run equilibrium or indefinite provider support. Posted seller
ordering also remains deterministic rather than a cheapest-seller search.

Validation includes direct state/peer prices, affordability at the state ask,
unchanged tax/issuance tables, discounted forward quantities, positive forecast
benefit after financing, rejection of unproductive tools, exact harvest effects,
recycled seed conservation, and the existing replay/CPU regression tests.
Generated logs remain under ignored `output/economics/`.

## Additional plots under the new prices

The matched 72-month additional-plot comparison uses two opening seed units per
person in both treatments. Without expansion, completed grain output is 1,864
units. Enabling expansion grants eight extra plots, produces 1,856 grain, and
collects 40 units of extra plot tax with no arrears. Total modeled need deficits
increase from 64 to 72, and one person reaches a terminal state versus none in
the control. Person 89 reaches the terminal state in month 71 because of warmth
deprivation. This person did not acquire an extra plot, so the outcome requires
an audit of indirect market/resource effects before attributing it to individual
over-expansion. The existing bounded expansion forecast does not establish
long-term safety for other participants. Both treatments match exactly between
CubeCL CPU and the reference backend. The pricing change does not relax taxes
or erase this adverse outcome.

The full regression suite passes 117 tests; formatting, strict Clippy and the
repository artifact check pass.
