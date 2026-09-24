# Physical minting: funded material and labor inputs

Implemented as an isolated CPU pilot. A state sells wheat for existing coins,
uses those proceeds in a later month to buy metal and labor, and executes an
ordinary production process that creates coins. This tests funding, permission,
physical inputs and supply accounting together. The fixed control supplies bilateral deals. An opt-in order policy now generates
state wheat asks and recipe-input bids and chooses counterparties from crossing
quotes. The production target and reservation prices remain configured; this is
not autonomous monetary policy or ZIP.

## Three economic agents and a marketplace

The state starts with six wheat and no coins. A metal supplier starts with two
metal and six coins; a worker starts with six coins and regenerates two labor
hours each month. A passive marketplace admits person and state types and lists
wheat, metal and same-month labor for coins. Coins require no storage; physical
stocks do. Initial private coins are explicit bootstrap money.

In month one, each person buys three wheat for three coins in a separate package.
The state's six coins become spendable after settlement. In month two, one
procurement package buys two metal for two coins and two labor hours for four
coins. The state then consumes the metal and hours to produce ten coins.
The worker also has a competing activity requiring two hours to collect one
firewood; paid hours cannot be spent again on that activity.

The recipe is supplied technology, with no promise to redeem coins for metal or
wheat. Mint authority comes from process permission plus the configured issuer;
the process is not available to every holder of metal. New coins enter treasury,
not automatically private circulation. The example runs three months and schedules
only one batch; it does not automatically repeat issuance.

## Boundaries and allocation

The existing scheduler remains in place:

1. **Open:** regenerate period capacity and expire unused hours. The state has no
   baseline labor, so its mint work requires acquired hours.
2. **Acquire:** collect dated deals; process packages by ascending package ID and
   deals by ID. Recheck venue admission, stock-trade permissions and, for labor,
   capacity-trade permissions. State procurement also checks mint permission.
   Reserve all outgoing balances and storage against the opening resource view.
3. **Acquire settlement:** publish an entire accepted package or none of it.
   The metal and labor purchases form one package. Independent wheat sales may
   clear separately. Receipt reasons identify the insufficient opening account.
4. **Productive:** ordinary process execution consumes acquired inputs and emits
   output. Generic process validation checks actual resources and effects; the
   minting guard reconciles the coin supply change to authorized completions.
5. **Next Open:** unused acquired labor expires. Later capacity is a new budget.

Incoming proceeds cannot fund another purchase in the same Acquire batch. Moving
wheat sales into the procurement month therefore fails to finance procurement.
Rejected packages release tentative reservations, leaving labor available to its
owner's competing work. This uses the shared acquisition resource view; it does
not change actor execution order to allocate labor.

Labor here is paid **capacity delegation**, fulfilled at Acquire. It is not a
future employment agreement or a guarantee of production. Procurement and later
production are separate commitments: if production subsequently fails, there is
no automatic wage refund. A richer labor agreement would need attendance,
completion and failure terms.

## CPU evidence

All cases start with 12 total coins and run for three months.

| Case | Closing coin supply | State / supplier / worker coins | Mint batches | Worker firewood |
| --- | ---: | --- | ---: | ---: |
| Complete cycle | 22 | 10 / 5 / 7 | 1 | 0 |
| State has only three wheat | 12 | 3 / 3 / 6 | 0 | 1 |
| Supplier has only one metal | 12 | 6 / 3 / 3 | 0 | 1 |
| Worker has only one hour in month two | 12 | 6 / 3 / 3 | 0 | 0 |

In the complete cycle, supply stays at 12 through both exchanges and increases
only on mint completion. In the treasury control, only one wheat sale clears;
three coins cannot fund the six-coin input package. Each procurement shortfall
rejects both purchases, leaving no partial metal purchase or wage payment.
The labor control also cannot complete the worker's two-hour firewood activity.

Eight focused tests cover this cycle, shortfall packages, opening-budget timing,
phase-by-phase CPU/reference agreement, checkpoint continuation, catalog and
actor reordering, replay rejection, forged receipts/inputs/issuance, permissions,
double-selling hours and observer transparency. An additional 77 existing tests
across acquisition, conditions, economics, negotiation, telemetry, town markets
ZIP and marketplace admission passed. Clippy passed for all targets with warnings denied.

Run from `exp/economics`, using a fresh output directory:

```sh
cargo +1.92.0 test --locked --test minting
TELEMETRY_DIR=../../output/economics/physical-minting-demo-new \
  cargo +1.92.0 run --locked --example minting
```

The runner prints the four comparisons and writes metrics, transactions and
settlement-observer JSONL under ignored output. No generated artifacts belong in Git.

## Generated order variant

`minting::orders::Policy` replaces scripted deals for dated mint targets.
The original variant supplies one target; the repeated variant adds later dates. It
reads the same immutable Acquire snapshot, with no extra execution phase:

- Sum the recipe's material and service requirements, subtract owned stocks, and
  calculate missing whole lots. Before the target month, current capacity cannot
  count as future labor. Budget at configured bid ceilings, not assumed cheap asks.
- Before the target date, offer enough whole wheat lots to cover the funding gap,
  capped by owned wheat. No sales are needed if opening treasury already covers
  procurement. Lot rounding may raise more coins than the exact gap.
- Counterparties generate bids toward configured stock targets and asks above
  protected reserves. They keep their own price limits; these are simple inventory
  policies, not full need/utility forecasts. Unclassified/ineligible actors cannot
  participate. Actual settlement still checks money, storage and labor permission.
- At the target date, submit input bids only if opening coins cover the bid-ceiling
  budget. Never finance these bids with same-batch sales or expected new coins.
- Match the issuer's buys against cheapest crossing asks and its sales against
  highest crossing bids, breaking price ties by agent ID. Trades execute at the
  seller's ask. Input markets resolve in market-ID order against one shared opening
  resource budget. Try another eligible counterparty if a candidate cannot settle.
- Commit procurement only if every missing input lot matches. Otherwise discard
  all tentative input trades and retain a reason identifying unfilled markets.
  Earlier wheat sales are not undone. After a target date, select the next
  configured date, or stop issuing orders if none remain. A missed target is not
  automatically retried or added to the next target.

The mint still starts through its existing dated production request. Acquiring
inputs is not completion; production validation and supply accounting are unchanged.
Configuration requires exactly one matching scheduled start per target, no scripted deals,
and one distinct listing for each recipe input. It bounds quote count and lots;
this is a small conditional procurement book, not an arbitrary order-book engine.

All four CPU controls reproduce the fixed variant's balances and outcomes above.
Additional controls show:

| Change | Generated response / outcome |
| --- | --- |
| State already has two metal and one coin | Funding requirement falls to four; sells one wheat lot and buys labor only |
| State starts with six coins | No wheat sale; waits until month two to acquire dated labor |
| Labor ask falls from four to three | Trades at three; state ends with 11 coins after minting |
| Another eligible seller offers metal for one | Chooses that seller; protecting its entire stock instead selects the original supplier |
| Labor ask exceeds the bid ceiling | No input purchases or minting; worker can collect firewood |
| No wheat demand, or no room for metal | No unsupported financing or partial procurement |
| Mint permission revoked or mint disabled | No issuer orders; no issuance |

Ten new tests cover these responses, shortfalls, participant admission and labor
permissions, supplied-price crossing, protected stock, same-month funding, receipt
tampering, invalid configuration, telemetry transparency, CPU/reference equality,
actor/quote reordering and restart after each phase. The original eight mint tests
remain unchanged and pass. One regression during development caught unsorted new
receipt data under reordered scripted deals; receipts are now canonically ordered.
The acquisition, marketplace and telemetry regressions also passed (42 tests
including both mint suites), as did all-target Clippy with warnings denied.

```sh
cargo +1.92.0 test --locked --test mint_orders --test minting
MINT_ORDERS=true TELEMETRY_DIR=../../output/economics/mint-orders-new \
  cargo +1.92.0 run --locked --example minting
```

`physical_minting_orders` telemetry records required funding, emitted orders and
the plan outcome, including insufficient treasury and unmatched input markets.
Package telemetry uses resolved dated deals, including generated counterparties.
The matcher does not record every suppressed counterparty order or every candidate
it tried; its failure reason is a bounded diagnostic, not a complete causal trace.

## Repeated issuance with finite replenishment

The six-month variant configures targets in months 2, 4 and 6. The same order
policy always considers the earliest remaining target. Its committed receipt
contains `target_month`; restarting from state at any phase selects the same
next target without a hidden cursor. Configuration rejects duplicate starts,
missing starts and additional dates at or before the first target.

The metal supplier now begins with finite ore instead of finished metal. An
ordinary stock-target activity consumes two ore and two supplier labor hours to
produce two metal whenever finished stock falls below two. Supplier labor
regenerates monthly. This is a supplied ore inventory and a processing recipe,
not geological extraction or an inexhaustible resource faucet. Metal produced
in Productive cannot be sold in that month's earlier Acquire boundary. Metal
sold at Acquire can trigger replenishment in the following Productive phase,
for availability in later months.

The worker similarly targets six firewood through the existing activity system.
On mint dates, sold labor leaves no hours for firewood. On other dates, or when
procurement fails, remaining hours may collect firewood. No scheduler or resource
allocation priority was changed to obtain these results.

| Six-month CPU case | Completed mint months | Final total coins | State coins | Worker firewood |
| --- | --- | ---: | ---: | ---: |
| Six opening ore, normal output | 2, 4, 6 | 42 | 18 | 3 |
| Only two opening ore | 2 | 22 | 10 | 5 |
| Worker has only one hour in month 4 | 2, 6 | 32 | 14 | 3 |
| Mint output four coins instead of ten | 2 | 16 | 4 | 5 |

The normal case ends with no ore or metal. Its 42 coins are held by the state
(18), supplier (9) and worker (15). Wheat sells only in month one; completed
issuance finances later input purchases. In the low-output case, the four coins
left after the first batch cannot meet the next six-coin input budget, and no
wheat remains to sell. In the labor-shortfall case, the full input package is
refused in month four; metal stays with the supplier for the next target.

Six new tests check finite resources, cycle outcomes, no same-month use of newly
produced metal, monthly versus batched execution, CPU/reference agreement,
restart after every phase, reversed actor/activity/start ordering, target
validation, telemetry transparency and no extra issuance after the last target.
All 24 tests across the repeated, generated-order and original minting suites
passed, together with all-target Clippy with warnings denied.

```sh
cargo +1.92.0 test --locked --test mint_cycles --test mint_orders --test minting
MINT_CYCLES=true TELEMETRY_DIR=../../output/economics/mint-cycles-new \
  cargo +1.92.0 run --locked --example minting
```

The example retains its previous modes; `MINT_CYCLES=true` selects the six-month
controls and takes precedence over `MINT_ORDERS`. JSONL remains under ignored
output. The order observer includes the selected target date, or null after
all targets expire.

## Purchasing power and leisure

The opt-in [food provision scenario](MINT-PROVISION.md) adds recurring nutrition,
independent state wheat sales, input asks tied to expected purchasing power and
an explicit leisure process. Its adequate/scarce/empty/endowed controls preserve
the previous scenarios as fixed-policy comparisons.

## Scope and next extension

The driver explicitly excludes credit, household pooling, negotiated/town/production
market drivers and collection-linked issuance. It uses existing agents, venue
catalogs, permissions, transfers, resource reservations, processes and CPU gather;
it does not establish that these excluded pilots compose with physical minting.
These variants have package receipts but do not update ZIP pricing memory or
the town market's price/volume history.

There is no geological mining model, mint equipment or tax collection. The
original controls omit consumption planning, state objective search and legal
delegation of mint authority. Generated orders provide a control alongside
supplied deals.
Reservation prices, private stock targets and the mint date are still supplied.
The state may sell wheat even when future input supply will prove insufficient;
it has no guaranteed counterparty commitments. Conservative ceiling budgets may
also defer a purchase that cheaper realized prices could fund. Repeated configured
issuance is now tested with finite ore processing. Changing
quote policies and future supply expectations remain separate extensions. The
normal case can finance later production because minted coin is accepted at fixed
quotes; that is not evidence of stable purchasing power or a sustainable monetary
policy. The original controls have no inflation response, issuance demand target,
worker
subsistence cost, renewed wheat demand or endogenous valuation of ore-processing
labor. The separate food-provision variant begins addressing subsistence and
willingness to accept coins; it still has finite endowments and fixed state prices.


## Financial reporting

The opt-in [issuance accounting adapter](FINANCIAL-STATEMENTS.md#physical-minting-and-collection-linked-issuance)
now recognizes authorized currency creation under an explicit non-redeemable
equity convention. Issuance is separated from income and external cash flows.
Physical minting retains actual material and purchased monthly-service expenses;
collection-linked tokens count only native-goods receipts. This reporting support
does not change settlement, admission, or acquisition-driver compatibility.
