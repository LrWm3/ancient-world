# Economics integration and planning interfaces

Implemented: a shared acquisition boundary for secured credit, its finite state
stock bid, and one bilateral negotiated exchange. Borrowing, sale-only and joint
production/sale forecasts also share need-constraint accounting. The experiment
still contains several separately tested pilots; this is not a universal economy.

## Shared acquisition boundary

`acquisition::evaluate` reads one immutable Acquire boundary and returns a dated
batch. The explicit allocation rule is **credit first, negotiated exchange
second**. Within credit, the existing purchase/resale/state-bid order is preserved.
This changes neither monthly phase order nor when installments fall due.

1. Credit emits its transactions and receipts from opening balances.
2. The shared resource view subtracts every outgoing leg from spendable balances.
   Incoming cash or goods cannot finance another leg in this batch. Storage tracks
   net reserved effects, so a confirmed outgoing stock transfer frees room.
3. Bilateral quote discovery uses opening permissions and pricing memory, with the
   remaining money, stock and storage limits. Optional need-generated orders use net
   reserved holdings; crossed quotes can still fail funding.
4. Settlement recomputes both components together and checks the exact combined
   transaction list before publishing balances, loans, title or ZIP memory.

A 2,000-tick downpayment uses all of a buyer's 2,000 opening ticks; an otherwise
acceptable 40-tick grain purchase fails. With 2,050 opening ticks both settle and
10 ticks remain. A land seller receiving 10,000 ticks cannot spend those proceeds
inside the same batch. Rejected exchange does not undo a valid financed purchase;
a forged batch or buffer overflow publishes neither component.

The resource view is deliberately small: outgoing balance reservations, net holdings and net
storage usage. It is not a universal resource auction. Credit still owns title,
collateral and loan rules; negotiation owns quotes and learning. Individual domain
previews do not authorize the combined batch. Use the shared resolver for that.

## Permissions

`Action::FinancedPurchase` controls the buyer's discovery and origination. An
existing citizenship agreement can grant this action through the ordinary
membership permission table. An unpermitted configured application records an
ineligible rejection. Revoking permission after acceptance does not erase debt or
prevent due settlement. Credit stock bids require `StockTrade` permission for
both parties; venue exchange additionally retains its configured participant type.

This does not automatically acquire citizenship before applying for a mortgage.
The combined fixture can start with accepted membership; prerequisite search and
acceptance remain their existing process/access pilot. Permission-governed
collateral resale remains rejected pending explicit buyer/seller rules.

## Supported combinations

| Combination | Current status |
| --- | --- |
| Scripted secured purchase + bilateral fixed/concession/ZIP exchange | Shared Acquire reservations and CPU controls |
| Finite state stock bid + bilateral exchange | Shared stock, money and net storage; state bid retains its posted price |
| Credit origination + existing citizenship/type permissions | Supported; due enforcement remains independent of permission |
| Borrowing/sale-only forecast + configured negotiation | Uses the same resolver in hypothetical branches; optional bounded consumption orders |
| Joint dated production plan + negotiation | Explicitly rejected; future work reservations need their own shared budget contract |
| Credit/negotiation + households | Still rejected; pooled purchase resources and loan support need explicit receipts |
| Legacy equipment/forward exchange, competing-access or pool-market drivers + credit/negotiation | Still rejected |
| Need-generated marketplace orders | [Bounded consumption/surplus policy](NEED-ORDERS.md) implemented; bilateral parties, lot and reservation prices remain supplied |
| Four-person monthly town book | [Implemented separately](TOWN-MARKET.md): locality, generated orders, multiple counterparties, fixed/ZIP quotes; credit and households remain excluded |
| State posted bids learning ZIP prices | Not implemented; co-settlement does not change the price-setting policy |

These exclusions are intentional validation boundaries, not claims that every
agent system can now be combined. Extend one boundary at a time with the same
opening-resource, forgery and continuation controls.

## Standardized planning contracts

`ForecastContext` remains the shared observation constructor. `forecast::needs`
now supplies common operations for borrowing, sale-only and joint planning:

- Validate nonnegative cumulative limits against positive catalog needs.
- Accumulate monthly deficits in integer units, separately for each provision.
- Check absolute limits and report the first violation in priority/resource order.
- Produce a stable priority-ordered deficit vector for policy-specific scoring.

The public policy configuration and decision receipts stay domain-specific.
Borrowing permits an empty limit map; sale and joint policies require limits.
Horizon bounds, candidate enumeration, payment checks, economic scores and fallback
rules remain with their policies. This preserves their different questions:

| Planner | Objective retained |
| --- | --- |
| Borrowing | Accept only a repayable, admissible improvement over declining |
| Sale-only | Choose the largest admissible current sale; otherwise sell zero |
| Joint work/sale | Compare needs, failures, buffer and net wealth across bounded work/sale alternatives; explicit no-sale scarcity fallback |

No common simulator loop or universal utility score was introduced. The rollouts
make different hypotheses, and merging those loops would hide their policy choices.
These limits constrain decisions; they do not add missing deprivation consequences
to fixtures that have no condition rules.

## Verification

Run from `exp/economics` with Rust 1.92.0:

```sh
cargo +1.92.0 test --locked --test acquisition
cargo +1.92.0 test --locked --test forecast_needs --test borrowing --test sale_plan --test joint_plan
cargo +1.92.0 test --locked --test credit --test credit_offers --test stock_sale --test negotiation --test zip
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Validation: the full crate run passed 290 tests. After the final common-offer
dispatch refinement, all 35 focused acquisition, offer, need-accounting, venue,
membership and permission tests passed (including the additional eighth
acquisition test). All-target Clippy passed with warnings denied. Generated logs
remain under ignored `output/economics/`.

Acquisition controls cover cash contention, incoming-proceeds exclusion, state-bid
stock contention, net storage, both-party permissions, citizenship, continued debt
service after permission removal, forged component receipts, buffer exhaustion,
ZIP state, CPU/reference equality, monthly/batched/resumed execution and reordered
catalogs. Existing planner controls retain funded/scarce outcomes and repeated
harvests. These establish settlement compatibility, not economic calibration or
realistic emergent prices.

The other review work remains separate: deprivation consequences in the credit
fixtures, controlled planner ablations, uncertainty and competing sellers. The
broader architecture and speculative extensions remain proposals unless a linked
implementation report says otherwise.

## Production and market planning pilot

[Production-market planning](PRODUCTION-MARKET.md) adds an opt-in four-person
comparison of ordinary work, producer preferences, waiting and buying. Decisions
use bounded reference rollouts and preceding market observations; live clearing
reserves actual stocks, money and storage, and productive execution honors ongoing
work before new requests. Adaptive market sides use fixed supplied quotes. This
does not integrate household budgets, credit or legacy state trading into the town
book. The original fixed-side ZIP pilot remains available.

## Reciprocal-market extension

[Grain and wood exchange](RECIPROCAL-MARKET.md) adds a second listing with shared
opening money, stocks and storage, per-good observations and purchase choices,
and selectable market clearing priority. Historical unfilled bids can signal
possible future demand; actual and forecast settlement still require finite
resources. This remains a four-person fixed-quote experiment. Household budgets,
credit and adaptive ZIP are not integrated by this extension.

The directed complementary-work control sustains grain/wood exchange and keeps
all four people funded. The autonomous planner has not demonstrated that result;
its forecasts and monthly replanning can fail to provide anticipated supply.
Use the linked results to distinguish settlement support from emergent behavior.
