# Need-generated marketplace orders

Implemented as an opt-in, bounded consumption policy. Household integration remains
excluded pending its own redesign. This connects individual needs to the existing
person-only marketplace without changing its pricing or settlement rules.

## Decision and execution boundary

`World.need_orders = Some(Policy { reserve_months: 2 })` treats the existing bilateral
`Session` as a market template. Starting at its configured month, Acquire evaluates
one possible lot each month. Without this policy, the original one-shot session
behavior is unchanged. Both traders require participant components; the reserve
horizon must be between 1 and 24 months.

1. Check marketplace eligibility and the listed transaction terms.
2. Observe current holdings after earlier credit reservations. Project permitted
   consumption recipes in need-priority order, sharing one stock budget across
   needs and months. Substitutable foods use the same rules as actual consumption.
3. Generate a buy order only if receiving the lot reduces at least one current
   deficit without increasing another. Generate a sell order only when the whole
   lot remains available after protecting the seller's needs and commitments.
4. If both orders exist, run the configured fixed, concession or ZIP quote policy.
   Protect seller stock and buyer payment resources, then check actual funding,
   storage and permissions through the existing acquisition resolver.
5. Recompute the exact dated receipt at settlement; publish transfers and pricing
   memory atomically. Consumption sees committed goods later in the month.

Requests are not completed transactions: a useful food order can still fail because
a mortgage downpayment reserved the buyer's cash. Incoming transfers update the
holdings projection and storage, but cannot fund another outgoing leg in the same
Acquire batch. Credit retains first priority; no scheduler reordering was added.

Receipts record generated orders, before/after buyer deficits and protected
resources. `NoDemand` and `NoSurplus` produce no quotes or ZIP learning events;
previous pricing memory survives these skipped sessions.

## What is protected

The reserve covers consumption over the configured horizon without assuming future
production. It also includes dated commodity obligations within that horizon,
unpaid entry inputs of every remaining active-process stage, and currently
collectible, already-accrued loan payments. Already-consumed stage inputs are not
reserved again. Future mortgage installments and future interest require the
separate borrowing forecast; this policy does not forecast them.

Partial stocks below a recipe's minimum are retained. Recipe ordering and protection
across alternatives are conservative: they need not find the largest sale possible
under an optimal substitution portfolio. Active-process inputs remain protected
even when their stage lies beyond the consumption reserve horizon. These are
explicit allocation choices, not additional physical transfers or permanent locks.

Live consumption now also filters recipes by process permission before selecting
one. A forbidden high-yield recipe no longer blocks an available permitted recipe.

## CPU observation

Two people each need one unit of nutrition per month. The buyer begins with 200
coin ticks and storage for two grain; the seller begins with ten grain. The listed
lot is two grain, and the reserve horizon is two months. No production replenishes
stocks, and this fixture has no deprivation condition rules.

| Month | Buy order | Sell order | Outcome | Buyer / seller deficit |
| --- | --- | --- | --- | --- |
| 1 | Yes | Yes | Trade | 0 / 0 |
| 2 | No | Yes | No demand | 0 / 0 |
| 3 | Yes | Yes | Trade | 0 / 0 |
| 4 | No | No | No demand | 0 / 0 |
| 5 | Yes | No | No surplus | 1 / 0 |
| 6 | Yes | No | No surplus | 1 / 0 |

Concessions settle each lot at 40 ticks; the default ZIP control settles at 36.
Final buyer/seller balances are 120/80 and 128/72 respectively. Both consume the
same ten initial grain units. With generated orders disabled, the supplied session
trades only in month one and the buyer accumulates four deficit units instead of
two. The experiment demonstrates responsive orders and reserve protection; it does
not solve scarcity or establish sustainable production or equilibrium pricing.

## Verification

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example need_orders
cargo +1.92.0 test --locked --test need_orders
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

All 11 focused tests pass. Controls cover substitutes, shared recipe inputs,
partial stocks, permitted recipes, active-process inputs, dated obligations,
credit cash contention, unfunded/storage/ineligible orders, forged receipts and
replay rejection. CPU/reference, monthly/batched/checkpoint continuation and
reordered catalogs agree. The broader 74-test acquisition, market, ZIP, foraging,
permission and credit-planning regression selection also passes; all-target Clippy
passes with warnings denied. Another 30 membership, condition and core economics
regression tests pass (104 selected tests total).

## Scope and next work

The venue, counterparties, lot size and private reservation values remain supplied.
Only consumption-driven demand and protected-surplus supply are generated. There
is no order book, competing-seller selection, producer-input search, endogenous
valuation or general portfolio planner. ZIP learns quotes within supplied limits.
Existing credit/household and joint-plan/negotiation exclusions still apply.

The remaining review areas are deprivation consequences in credit fixtures,
controlled planner ablations, uncertain forecasts and competing offers. Household
integration is deferred for reworking rather than extending its current assumptions.

The separate [town-market extension](TOWN-MARKET.md) now matches multiple registered
buyers and sellers under month-start locality, while reusing this need/protection
policy. The bilateral credit-compatible pilot described above retains its supplied
counterparty pair; the new town book is not yet credit- or household-compatible.
