# Production and purchase planning through a town market

Implemented bounded CPU pilot. The original single-good results below describe
that version; [reciprocal-market planning](RECIPROCAL-MARKET.md) extends it with
wood trading, per-good purchases and an optional unfilled-bid forecast signal. Four people can cultivate grain and collect fuel
wood, choosing work and purchases from the same opening observation. This extends
[the town market](TOWN-MARKET.md); it does not replace the other experiment drivers.

## Scenario

All people begin with four grain, one reusable seed, two fuel and 24 coins. Each
has two labor units per month, storage capacity 32 and an existing plot-use right.
Each needs one nutrition and one warmth per month. Cultivation consumes one seed,
takes three months and returns four grain plus one seed. Wood collection yields
two fuel in one month. Both normally cost two labor per month of work.

Two people initially have crop expertise: cultivation costs one labor and yields
eight grain. The other two have wood expertise: collection costs one labor. These
are competency/technique records, not occupations or restrictions on available
processes. Practice accumulates through the existing mechanism; the high expertise
threshold keeps initial differences stable during this comparison.

The marketplace lists two-grain lots for coins. Fixed reservation quotes are four
coins per lot on both sides. It admits nearby permitted people at Open. Each may
submit at most one order per month; their side follows stocks, projected needs
and the chosen purchase policy. Wood is collected for personal use, not traded.
Money is a finite initial endowment: no mint, lender or state buyer replenishes it.

## Decision and execution boundary

1. **Open:** regenerate capacity and capture local market admission.
2. **Acquire:** each person compares eight alternatives from the same snapshot:
   ordinary need-directed work, waiting, preferring cultivation, or preferring wood
   collection, each with buying allowed or declined. Buying considers consumption
   across the six-month planning horizon. Legacy need orders retain their immediate
   consumption horizon. Sellers protect two months of consumption and existing
   input/payment claims. Choosing not to buy can leave an otherwise surplus lot
   available for sale.
3. **Clear and settle:** collect actual orders, match, reserve real stocks/coins/
   storage and commit transfers. Projections do not create inventories or cash.
4. **Productive:** execute the accepted work choice against post-trade resources.
   Existing processes receive priority; waiting means no new work, not abandoning
   an accepted crop. An explicit producer preference is followed by other ordinary
   need-directed requests, subject to the existing resolver's feasibility checks.
5. **Consumption and close:** satisfy needs, record deficits and retain actual
   price/volume and decision receipts for subsequent planning.

Every alternative runs the existing reference simulation for six months. The
person's candidate policy is held for that hypothetical horizon; other people are
assumed to follow ordinary need-directed work and allow purchases. Live people
replan monthly, so those assumptions can be wrong. Decisions rank terminal state,
priority-ordered need deficits, failed processes, a two-month closing need buffer,
closing coins plus bounded surplus valuation, and labor used. Stable candidate
order breaks ties. This is a named bounded policy, not a general optimizer.

Market beliefs use only preceding months, over a six-month observation window.
The last completed price values at most one closing surplus lot; this valuation
is separate from spendable coins. Future hypothetical clearing is capped by the
largest completed monthly volume in that window. With no prior transactions,
future clearing is capped at zero; current opening stocks can still trade. Every
hypothetical trade also needs a willing modeled counterparty, goods and payment.
The volume ceiling is optimistic historical capacity, not guaranteed demand or a
probability estimate. Supplied live quotes do not change with these observations.

The accepted decision and all alternatives are recorded with their month and
horizon. Settlement recomputes the market decision and validates productive work
against the accepted choice. State reconstruction checks decision indices and
boundaries. This extends the current monthly phases and allocation resolver; it
introduces no scheduler rewrite.

## Controlled observations

The identical twelve-month controls produced:

| Measure | No trade | Trade |
| --- | ---: | ---: |
| Nutrition deficit | 0 | 0 |
| Warmth deficit | 14 | 2 |
| Labor used | 96 | 94 |
| Grain exchanged | 0 | 16 |
| Failed processes | 0 | 0 |
| Total coins | 96 | 96 |

Deficits are summed unmet units across people and months. The trade case does not
assign permanent sellers: the skilled growers initially buy grain, then later sell.
By month twelve they each hold 32 coins and the other people each hold 16.
Buying can defer immediate self-production and free capacity for warmth, while
later crop output supplies the market. This is evidence for a useful production/
purchase choice, not proof of stable specialization or calibrated welfare.

At 24 months, no trade has 26 warmth-deficit units; trade has two. Both have
zero nutrition deficits, zero failed processes and 190 labor units used. The trade
case has exchanged 24 grain. Crop experts each end with 40 coins; the other people
each have eight. Cash is concentrating among net grain sellers, so this one-way
market is not evidence for an indefinitely sustainable circulation of money.

When the two net buyers leave market reach after month twelve, no further trades
complete: the 24-month total stays at 16 grain. Warmth deficits rise to 12, labor
use to 192, and nutrition deficits and failed processes remain zero. The people
continue producing and consuming outside the market. This controlled departure
shows lost market access changing work/need outcomes without creating speculative
seller revenue; it is not a general recession or default model.

An initial control using only immediate food-shortage orders produced no trade:
self-production maintained food stocks, so agents never submitted a purchase
request early enough to compare freeing future labor. The opt-in purchase horizon
addresses that specific mismatch; it does not relax finite settlement resources.

## Verification and reproduction

From `exp/economics`, with Rust 1.92.0:

```sh
cargo +1.92.0 run --locked --example production_market
cargo +1.92.0 test --locked --test production_market --test town_market --test need_orders
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

The example runs 24 months on CubeCL CPU for no trade, trade, and trade with people
91 and 92 leaving the market's reach after month twelve. `MONTHS=12` runs the
shorter comparison. `DETAIL=1` prints the alternative forecasts as well as selected
choices. Redirect raw output into ignored `output/`; keep only summaries in Git.

Focused controls cover changing sides, purchases ahead of immediate shortage,
seller reserves, absent seeds, continued work while waiting, missing participants,
expiration of demand history, finite money, repeated harvests, hidden future
capacity overrides, forged decisions/work, observed price sensitivity without new cash, invalid
configurations and reconstruction
at Productive. CPU monthly/checkpoint continuation with reversed catalogs matches
reference batched state, ledger and reports.

The full crate run passed all 323 tests, including nine production-market tests.
Strict all-target Clippy and the repository artifact check passed. The three
24-month example runs used CubeCL CPU settlement. Raw outputs remain ignored.

## Limits and next questions

The search is limited to four people, three productive definitions and at most
12 forecast months; the horizon must cover every enabled productive duration.
It copies state and runs reference rollouts, while actual settlement runs on CPU.
It is not a GPU planning benchmark. Forecasts observe the current complete state;
private information, uncertain weather and strategic opponent models are absent.

Adaptive sides currently require fixed quotes at supplied reservation values.
The earlier fixed-side town pilot still supports ZIP. Combining endogenous
valuations, adaptive roles and ZIP is separate work. There is one grain market,
one lot per order, stable ID matching ties and no institutional production budget.
Households, loans and legacy state-trade drivers remain outside this pilot.

Need deficits affect ranking but this fixture has no deprivation condition rules.
Unsold output consumes storage and labor, with no automatic guaranteed resale.
Finite buyer money and market departure should therefore be studied before treating
higher production as enduring profit. A subsequent step could add a wood market
and test reciprocal exchange, then compare quote policies under the same production
and need constraints.
