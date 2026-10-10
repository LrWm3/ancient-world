# Private food circulation

Status: bounded fourteen-month comparison with a forty-month continuation. All cases begin from
`scenario::financed_circulation()` with no authored memberships, household/land/loan
agreements, operational programs or counterparty quotes. The marketplace explicitly
admits household agents in **both** variants; discovery never bypasses its rules.

Each row compares `private_sales` off/on under identical endowments, listing size
and valuation. Wheat costs one coin per unit, quoted as either three-unit or
one-unit whole lots. The opt-in policy includes private person/household asks and
charter-directed collective buying. Therefore this compares that combined policy,
not an isolated change to seller eligibility. Initial public wheat is sixteen or
eight; its reserve remains four. All other financed-circulation inputs are retained.

| Public wheat | Lot | Private policy | Private food lots | Household → worker lots | Worker food bought | Worker deficit | Crops | Mints |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 16 | 3 | Off | 0 | 0 | 6 | 5 | 10 | 2 |
| 16 | 3 | On | 0 | 0 | 6 | 5 | 10 | 2 |
| 16 | 1 | Off | 0 | 0 | 4 | 7 | 10 | 1 |
| 16 | 1 | On | 0 | 0 | 8 | 3 | 10 | 2 |
| 8 | 3 | Off | 0 | 0 | 0 | 11 | 9 | 3 |
| 8 | 3 | On | 3 | 2 | 9 | 3 | 11 | 3 |
| 8 | 1 | Off | 0 | 0 | 0 | 11 | 9 | 3 |
| 8 | 1 | On | 12 | 5 | 12 | 1 | 13 | 3 |

Every case discovers a household and lease, repays its admitted loan and pays the
one-wheat annual due. Both household members retain one startup nutrition deficit.
The larger public store produces no private food fills; its one-unit improvement
cannot be attributed to private seller transactions. With the smaller public store,
household sales actually reach the independent worker. Smaller lots help in that
case, but the off-policy results show that smaller lots alone do not guarantee
better outcomes: planning, demand and financing interact.

The eight-unit/unit-lot/private case still has a worker deficit despite buying
twelve wheat, because receipts arrive after some earlier consumption boundaries.
Finite wages, access and timing remain material. This is not a steady-state result,
an optimized policy, endogenous pricing or evidence for unconstrained market access.

All eight cases compare uninterrupted reference execution with CPU reconstruction
at every phase, including world/state, transactions, reports and separate financial
books. Assertions pin actual trades, member/worker outcomes, process completions,
repaid debt and annual payment. Run in `exp/economics`:

```sh
cargo +1.92.0 test --locked --release --test discovered_circulation private_food_access -- --nocapture
```

## Forty-month continuation

The eight-unit public store and one-unit listing continue unchanged for forty
months, with private policy off/on. No new endowments, scripted employment or
renewal instructions are injected.

| Private policy | Worker food bought | Last food purchase | Earned coins | Last paid work | Ending worker coins | Worker deficit | Household food | Crops | Mints |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Off | 1 | Month 38 | 12 | Month 6 | 11 | 36 | 20 | 25 | 3 |
| On | 12 | Month 13 | 12 | Month 6 | 0 | 25 | 19 | 28 | 3 |

Both variants retain the household, repay one loan, accept the renewed lease in
month 26 and pay the annual dues in months 14 and 38. Member deficits remain one
each. Worker food and cash reconcile against actual accepted trades and consumption.
Uninterrupted reference and phase-reconstructed CPU execution agree, including
separate financial books.

The later receipts distinguish two constraints. With private sales disabled, the
worker submits funded bids when there is no public surplus available. With private
sales enabled, the household has eligible feasible supply at a later boundary but
the worker has no affordable bid. Private access spends the finite wage income
more usefully; it does not generate recurring demand for the worker's labor. The
fourteen-month improvement therefore does not extrapolate to sustained nutrition.
A next experiment should expose an additional useful paid activity or recurring
labor demand through ordinary opportunities and compare its actual compensation
and food purchases, retaining this finite-income control.

```sh
cargo +1.92.0 test --locked --release --test discovered_circulation private_circulation_continues -- --nocapture
```
