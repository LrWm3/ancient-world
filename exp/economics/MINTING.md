# Physical minting: funded material and labor inputs

Implemented as an isolated CPU pilot. A state sells wheat for existing coins,
uses those proceeds in a later month to buy metal and labor, and executes an
ordinary production process that creates coins. This tests funding, permission,
physical inputs and supply accounting together. The issuance target, counterparties
and fixed prices are supplied scenario data, not autonomous state planning or ZIP.

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

## Scope and next extension

The driver explicitly excludes credit, household pooling, negotiated/town/production
market drivers and collection-linked issuance. It uses existing agents, venue
catalogs, permissions, transfers, resource reservations, processes and CPU gather;
it does not establish that these excluded pilots compose with physical minting.
The fixed exchanges have package receipts but do not update ZIP pricing memory or
the town market's price/volume history.

There is no mining process, mint equipment, tax collection, consumption planning,
state objective search or legal delegation of mint authority to another agent in
this scenario. The sensible next extension is replacing supplied procurement with
state-generated bids and wheat asks while keeping this same funding and production
boundary. Only then should repeated issuance targets and pricing feedback be
compared against these fixed-term controls.
