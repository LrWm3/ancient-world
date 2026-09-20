# Specialist trading and tool output shares

Retained as the `trading-32-share` control. The default `trading-32` now uses
[upfront coin prices and prepaid production forwards](FORWARD-TRADING.md).
Implemented in the stand-alone CPU experiment. The original four-person and
32-person specialized-activity controls are unchanged.

From `exp/economics`:

```sh
cargo +1.92.0 run --locked -- trading-32-share
cargo +1.92.0 run --locked --example trading_audit
cargo +1.92.0 run --locked --example trading_audit -- 3 cpu
```

The scenario runs 72 months with 32 people plus the state. Three tooling providers
is the selected baseline. Five of the original eight tooling profiles become
farmers: the resulting distribution is 13 farming/fishing, eight mining/refining,
eight husbandry and three tooling profiles. These remain generic agents with
configured work orders, not occupation classes.

## Contract and quantities

A provider transfers a produced tool to a customer for a share of **realized
output from tool-assisted work**. There is no upfront purchase price. The default
`DEFAULT_CAPTURE_PERCENT` is 25; the scenario builder accepts integer percentages
from zero through 100.

Stocks and money use fixed-point integers: **100 ticks = one physical unit**.
Stock inputs, outputs, deposits, opening balances, storage capacities and annual
tax amounts scale together. Labor, time, durability and need fulfillment retain
their previous units. A husbandry operation produces two milk and one wool:
with a supplied comb, the provider receives 0.50 milk and 0.25 wool, while the
worker retains 1.50 milk and 0.75 wool. No floating-point settlement or rounding
loss is involved. Configurations requiring finer precision than a stock tick
are rejected.

The share applies after replacing inputs of the same commodity. A crop returning
one seed after consuming one seed owes no seed royalty; its eight grain split
into two for the provider and six for the operator. This preserves replanting.
It is a share of attributable process output, **not** a measurement of the
counterfactual extra output caused by the tool, a markup, or a percentage of all
the customer's production.

The first contracted tool actually used in a process binds that process to its
provider. The binding survives intervening stages and manual final work; completed
stock output is charged once. Later tools cannot stack additional charges.
Aborted processes and idle tools earn nothing. Processes yielding only durables
or need fulfillment have no divisible stock royalty in this version. The contract
persists with that tool through repairs; replacement assets receive new contracts.
Resale, contract renegotiation and apportionment among multiple contributing
providers are not modeled.

Consumption still uses the existing whole-recipe lots: fractional stocks are
stored and can accumulate or trade, but a quarter food unit alone does not
satisfy the current one-unit consumption recipe.

## Demand, exchange and monthly boundaries

Customers request tools relevant to their configured work and a hoe for possible
subsistence production. Each customer is assigned to a provider by stable
round-robin order. Providers produce against unmet requests, rather than building
an arbitrary stock of every tool. Exhausted customer tools reopen demand.
Providers retain an internal hammer stone for production.

This is an explicit demand/matching policy: all eligible customers accept the
posted output-share terms. They do not negotiate a rate or forecast whether
acceptance is individually beneficial. Existing manual techniques remain
available.

The existing monthly pipeline is retained:

1. Open refreshes capacities and ages assets. Due settles existing tax bills.
2. Acquire collects eligible tools and posted stock bids from this opening
   boundary. Stable IDs resolve competing requests. Delivery transfers ownership
   and records the output-share contract.
3. Stock exchange reserves each party's opening holdings and receiving space.
   Goods received in this batch cannot finance another purchase in the same batch.
4. Productive work reserves labor, inputs and equipment normally. Validated
   completion routes stock output directly to customer and provider. Both must
   have storage. If a tool-assisted option cannot fit, the existing resolver may
   choose feasible manual work, which produces no tool share.
5. ClearArrears, Consumption and Close retain their existing timing. Royalties
   can supply consumption or arrears at these later barriers. Newly crafted tools
   become available for delivery at the following month's Acquire.

The simple needs planner projects retained output rather than promising the
provider's share to the customer. Production and trade stay in the transaction
ledger, with explicit delivery and royalty receipts; gather commits balances.
No agent changes another agent's holdings directly.

Providers can barter grain for stone, wood, copper and iron through fixed posted
one-unit-for-one-unit bids. Other individuals offer stocks above a two-unit
reserve. Providers replenish toward four units of each input. These are abstract
fixture prices, not discovered exchange values.

Providers may also sell grain above a six-unit reserve to the state for coins,
up to four coins each. State purchases spend only treasury tokens actually
available. Existing issuance remains one token for two units of native grain
tax collected; trades, mineral extraction and coin tax payments do not mint money.
The annual tooling-provider tax remains two coins. Former providers reassigned
to farming instead pay the farming grain tax and receive its coin alternative.

## Selecting the provider count

The audit compares counts one through eight with the same 32 people, per-person
endowments, shared deposits and labor budgets. Reassigned people farm, so customer
mix changes as provider count changes. This is a controlled scenario choice,
not endogenous occupational entry or an equilibrium solver.

The last 24 months provide the recurring-income screen. Every provider must:

- Earn enough edible output shares, less grain actually spent on material barter,
  to cover monthly food and the two annual tax bills in that interval.
- Meet food and warmth requirements and remain active without overdue tax bills
  throughout the interval.
- End with enough coins for its next annual bill.

Unsold minerals, wool, hay and tools are not given hypothetical food purchasing
power. Beginning food and coins cannot make the income test pass. The selected
count is the largest tested count passing every provider's screen; the selection
helper returns **one** when none passes. The selected default is a scenario
constant, not an automatic population change during simulation.

The screen does not price the initial plot, house, capital, fuel endowment or
opportunity cost of labor, and does not prove indefinite viability. It deliberately
separates having inventory from earning usable food or tax liquidity. Customer
shortfalls and total arrears are reported alongside the provider screen.

## Observed 72-month comparison

| Providers | Tools delivered | Food deficit units across all people | Ending unpaid tax | Lowest provider net food income/month, final 24 months | All providers supported |
| ---: | ---: | ---: | ---: | ---: | --- |
| 1 | 70 | 6 | 0 | 1.438 | Yes |
| 2 | 69 | 4 | 0 | 1.625 | Yes |
| **3** | **72** | **0** | **0** | **2.062** | **Yes** |
| 4 | 68 | 0 | 4 | 1.229 | No |
| 5 | 73 | 0 | 0 | 1.208 | No |
| 6 | 73 | 0 | 4 | 0.646 | No |
| 7 | 74 | 0 | 10 | 0.771 | No |
| 8 | 72 | 0 | 15 | 0.771 | No |

Income is in edible physical units, after actual grain spending on materials and
before food/tax costs. Tools delivered includes replacements. Provider count
changes client assignment as well as the producer mix, so results need not be
monotonic. The provider screen alone is not a customer-welfare measure: the one-
and two-provider cases have customer food shortfalls.

At three providers, their ending grain stocks are 96, 46 and 24 units, and each
holds four coins. There are no food deficits or unpaid taxes in this run. The
72-month CPU state, ledger and reports match the reference exactly.

Five providers narrowly cover the measured food-and-tax income threshold, but
one ends with only one coin against the next two-coin bill. Four, six, seven and
eight also have overdue taxes. The higher-count cases therefore do not justify
retaining the original eight tooling providers.

As an inactive-income control, run:

```sh
cargo +1.92.0 run --locked --example trading_audit -- 1 reference 0
```

At 0% capture, the provider earns no shares, ends with zero grain and coins, and
the run has four unpaid tax units. The support screen fails and selection retains
one provider as the required fallback, without claiming that provider is viable.

## Validation and limits

Focused tests cover 0%, 25% and 100% shares; fractional commodity payments;
replanting-input protection; idle/manual work; provider storage; finite material
barter; tampered deliveries and receipts; the minimum-one fallback; and CPU,
reference, reversed-order, replay and midmonth continuation agreement.

The full regression run passed 102 tests. The final focused exchange run passed
all six tests, including the subsequently added provider-selection test. Formatting
and strict all-target Clippy also passed.

The full 72-month audit additionally compares the selected scenario's complete
CPU state, ledger and reports against the reference backend. Generated logs stay
under ignored `output/economics/`.

This remains a provisioned economy with fixed recipes, assignments, posted barter
prices and storage. It has no general resale market for royalty ores/wool/hay,
no transport costs, no endogenous provider choice and no price negotiation.
Capture rates influence resources and income, but customer acceptance is still
hardcoded. These limits matter when interpreting the tuned provider count.
