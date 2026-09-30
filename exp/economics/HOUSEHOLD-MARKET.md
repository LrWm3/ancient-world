# Household participation in the town marketplace

Implemented as a bounded integration of household governance, member needs and the
existing local town book. The household trades through its own agent account; its
members remain individual people with separate needs, private holdings and financial
statements. No aggregate person or duplicate population is created.

## Authority, demand and budgets

The venue must accept the household type and state law must permit its stock trades.
The household has an explicit registered position; member locations are not averaged
or used to create transport. Open records local admission. Acquire rechecks current
permission, operational membership and the presence of a governor. Wind-down and
governance vacancies prevent new collective orders.

With the default `Purchasing::Collective` charter, the effective `NeedsFirst` or
`NeedsThenIncome` policy authorizes purchases that improve members' projected
consumption. `Purchasing::Members` instead delegates buying to registered members
using private funds, and disables collective bids. This is a static founding
parameter. Personal policy revision remains deferred. `NetOutput` and
`PreserveCommittedWork` currently authorize collective
protected-surplus sales only in this adapter. They do not invent speculative demand
or a valuation policy. Existing dated governor instructions can switch the objective.
Order receipts retain the governor, term and policy; the settlement observer exports
this authority alongside the submission or rejection reason.

Consumption forecasts reuse household requests/allocation and permitted individual
recipes. Private member food reduces collective demand but never becomes household
sale inventory. Protection includes household claims and the uncovered portion of
supported member claims, plus consumption reserves. Claims are protected only where
the enclosing acquisition driver supports their composition; this does not enable
mortgage-purchase configuration or legacy land/forward drivers. Direct loans now
share the town acquisition budget. Membership, rights and private
holdings are frozen over the bounded consumption horizon, without assumed harvests,
future purchases or earnings. Recipe protection is conservative, not optimal.

All listings share the household's actual opening money, stock and storage budget.
Member coins cannot finance its purchases. Incoming proceeds cannot finance another
outgoing leg in the same batch. A useful order may be submitted but fail funding or
storage. Fixed and ZIP quotes retain the existing town-market semantics and supplied
private price limits. ZIP memory belongs to the household/market/side.

## Monthly integration

1. **Open:** apply existing household lifecycle/governance work and record local
   market eligibility.
2. **Acquire preparation:** existing household allocation can reserve collective
   goods to members. Order projections see those updated private and collective
   balances, so current rations do not generate duplicate demand.
3. **Acquire settlement:** generate collective orders, match counterparties and
   reserve finite stock, money and storage. Recompute the receipt and commit trades
   and household transfers atomically through existing settlement.
4. **Later work and consumption:** the existing household resource and labor
   mechanisms distribute purchased food and execute contributed work. The household
   does not consume a second copy of its members' nutrition.
5. **Reporting:** journal market trades, internal transfers and consumption in each
   agent's separate books. Membership does not imply accounting consolidation.

Timing is unchanged. Purchase decisions and labor allocation reuse the current
policy but remain distinct decisions; there is no joint work/trade optimizer here.

## CPU observation

The fixture groups the two buyers from the four-person town example into one
two-adult household. Two independent sellers remain. The household starts with 100
coin ticks; each adult needs one grain-equivalent nutrition unit monthly. One lot
contains two grain. The household bids 50 ticks, sellers ask 30 and 40, and midpoint
matching settles the first seller's lot at 40 ticks.

| Month | Posted ticks per lot | Grain traded | Household coins at close | Member nutrition deficit |
| --- | --- | --- | --- | --- |
| 1 | 40 | 2 | 60 | 0 |
| 2 | 40 | 2 | 20 | 0 |
| 3 | No trade | 0 | 20 | 2 |

The third-month shortage is expected: remaining coins cannot fund the lot. This
demonstrates finite funding, not a sustainable household economy. Initial funding
and seller stocks are supplied; neither income nor credit is fabricated to sustain
the example. Separate household/member statements reconcile on CubeCL CPU.

## Verification and limits

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example household_market
cargo +1.92.0 test --locked --test household_market
```

Controls cover collective consumption, private-stock offsets, protected sales,
dated policy changes, permission/locality, governance vacancy, two goods sharing
cash/storage, and production with real contributed labor. CPU/reference runs agree;
fixed and ZIP continuation and reordered catalogs retain results. Forged orders,
authority and transfers fail atomically. External observation preserves execution.

Verification on 2026-09-29: **195 distinct tests passed** across 16 selected market,
household, accounting, law and telemetry suites. The final focused rerun passed all
12 household-market tests. The CPU example, strict all-target Clippy, formatting and
repository artifact check passed. One slow annual household accounting test remained
ignored; the full crate suite was not run. Raw logs remain in ignored
`output/economics/household-market-*.log`.

Current membership and the static charter determine who may submit consumption
bids. Registration alone grants no buy authority. Explicit exit permits registered
former members to buy privately; accession restores the household route. Adaptive
members with a blocked buy side may still sell protected surplus. Legal permission,
locality and own opening funding continue to apply.

Private purchases and sale proceeds share half their eligible stock receipts with
the household. Need projections count only the retained purchase portion as
assured personal fulfillment. Town matching reserves raw storage and the resulting
collective contribution separately, with exact relationship-specific fractional
carry across the whole book. Physical barter payment is supported; a trade that
fits privately but overflows the collective is rejected with a storage outcome.
Incoming proceeds cannot finance another outgoing leg in the same Acquire batch.

The opt-in static `fund_committed_inputs` charter parameter extends collective
demand to missing entry inputs of active member processes. Stock-resource keys in
order deficit maps identify these shortfalls; fulfillment keys still identify
consumption. Private inputs and collective opening holdings reduce demand. Existing
Productive allocation supplies purchased inputs, and production follows its
existing timing and accounting. This does not plan new businesses, buy speculative
work-order inputs, or promise future output. The flag applies with collective
purchasing and a needs-first objective; delegated member buying remains consumption
only. Fixed whole-lot terms and supplied reservation prices still apply.

Direct accepted loans now share credit-first Acquire reservations with the town
book. Mortgage-purchase configuration, recovery proceedings, joint production
planning, legacy exchange, pool markets and competing-access drivers remain
excluded. Outside member wages compose with collective budgets. Explicitly budgeted
household hiring and costed delegation to member work now compose with this book;
autonomous hiring discovery remains unsupported.

The [income-aware work policy](HOUSEHOLD-INCOME.md) includes optional static cash
buffers. The [first](INTEGRATION-PASSES.md) and [second](INTEGRATION-PASSES-2.md)
integration batches exercise these combinations. Outstanding work includes
endogenous private work targets, speculative investment/input planning, negotiated
hiring, market recruitment and registration derived from membership or travel.


## Current loan-payment demand

Static charter `fund_due_loans` allows collective needs-first orders to acquire
missing denomination stock for currently collectible loans. It covers the
household's own dues; member dues also require `support_member_loans`. Current
private holdings offset combined enabled requirements once. Future installments,
expected wages and hypothetical financing do not become purchasing resources.
Accepted arrears remain demand, subject to current eligibility/stays. Existing
admission, quote, opening payment budget and storage checks still gate settlement.

Due collection precedes Acquire. Purchased stock can therefore support the next
Due, with no backdated payment. The support charter remains distinct from purchase
permission: acquiring stock does not itself assume or discharge a member's debt.
See [combined verification and limitations](INTEGRATION-PASSES-3.md).

## Earned payroll demand

Static `fund_earned_wages` adds the household's own earned wage obligations, plus
member wage obligations when `support_member_wages` is enabled, to collective
needs-first buying requirements, alongside separately enabled process
inputs and current loan dues. Each member's private holdings offset combined enabled
requirements once; collective holdings reduce the remaining demand. Future
undelivered work creates no demand under the default `EarnedOnly` outlook.
Opt-in `CurrentDelivery` also targets current-month estimated payroll; see
[pass 26](PAYROLL-OUTLOOK.md) for its separate projection/earning boundaries.
Member purchasing delegation does not acquire
this responsibility. Orders still require legal access, payment stock, space and a
matching counterparty. Incoming market receipts cannot fund hiring in the same
Acquire boundary, but actual holdings can settle arrears at Close.

The [fourth integration batch](INTEGRATION-PASSES-4.md) connects hired hours,
member production, pooled output, sales and payroll. A temporary market closure
releases worker hours for self-production and changes subsequent demand; fixed
contract terms and conserved cash do not guarantee sustained household employment.

The [fifth integration batch](INTEGRATION-PASSES-5.md) runs private member hiring,
production, output pooling, collective coin purchases and payroll assistance through
this book under earned-only demand. Current earned claims generate bids; future payroll does not. With finite
initial funds, work alternates with months clearing arrears. Explicit member exit
removes collective demand for that member without erasing their wage debt. Quotes,
work targets and counterparties remain supplied configuration.
