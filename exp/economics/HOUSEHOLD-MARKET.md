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

The effective `NeedsFirst` policy authorizes purchases that improve members'
projected consumption. `NetOutput` and `PreserveCommittedWork` currently authorize
protected-surplus sales only in this adapter. They do not invent speculative demand
or a valuation policy. Existing dated governor instructions can switch the objective.
Order receipts retain the governor, term and policy; the settlement observer exports
this authority alongside the submission or rejection reason.

Consumption forecasts reuse household requests/allocation and permitted individual
recipes. Private member food reduces collective demand but never becomes household
sale inventory. Protection includes household claims and the uncovered portion of
supported member claims, plus consumption reserves. Claims are protected only where
the enclosing acquisition driver supports their composition; this does not enable
town-market credit or legacy land/forward drivers. Membership, rights and private
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

For this pilot, a person appearing in a household's membership history cannot also
register as a town trader. This deliberately conservative rule prevents collective
and private orders from covering the same needs; autonomous registration after exit
needs a dated adapter. Independent people and multiple household accounts can use
the book, but the exercised household scenario has one collective and two sellers.

The joint production-market forecast planner, credit/mortgage purchase drivers,
legacy exchange, pool markets and competing-access drivers remain excluded. Ordinary
productive work with contributed labor is exercised separately from that planner.
Outstanding work includes autonomous income plans, producer-input purchases,
employment and credit budgets, simultaneous member/collective participation,
market recruitment and market registration derived from membership or travel.
