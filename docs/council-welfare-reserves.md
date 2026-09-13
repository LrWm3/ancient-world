# Council reserves as household welfare

The opt-in `--council-welfare-reserves[=true|false]` selects the
`NeedsFirst` council allocation policy. Omission preserves the archive setting;
false selects the existing distribution policy. This is a toy welfare policy,
not an inferred historical institution.

## What changes

At the existing monthly household retail preparation boundary, after payroll,
dividends and family transfers, each household requests the cash needed for its
full dietary budget at the current food quote. Common provisions and its actual
wallet balance reduce that request. The policy replaces the lower political
food-target fraction and treasury-share ceiling for this scoped allocation.

Requests from all settlements controlled by a council are gathered together.
Available treasury above the current administration forecast funds these requests
proportionally. Payments transfer existing council money into household wallets.
The existing retail and production stages still determine actual food consumption:
purchasing power cannot create food, remove freight constraints or guarantee supply.

There is no second welfare payment pass, new currency or automatic annual treasury
liquidation. The administration allowance is a forecast, not escrow or protection
for every future public commitment. Existing escrow is already outside liquid
treasury. Competing discretionary public spending may receive less cash.

## Why aggregate treasury was misleading

In the preceding 50-year tax-plus-research runs, cash and unmet relief requests
belonged to different governments. Seed 256 had councils holding about 3,901 and
4,344 with negligible requests, alongside councils unable to fund requests of
about 29 and 25. Seed 1024 had a council holding about 6,975 meeting its requests
while a different council had no cash for a request near 29.

This policy improves local coverage; it does not pool separate governments'
treasuries or invent knowledge of distant need. Cross-government financial aid
would need its own observed request, political decision and actual transfer.

## Three-seed comparison

Native debug build, RTX 5000 Max-Q, terrain/ecology 32/32, frozen environment,
50 years, seeds 1024/256/409. Both arms enable household clothing, progressive
cash tax and practical research. Other settings match the
[civic production finance screen](civic-production-finance.md).
Baseline is the completed d48267d behavior; the intervention changes only
`--council-welfare-reserves=true`. Raw outputs remain ignored under
`output/council-welfare-screen/`. These are balance screens, not timing benchmarks.

| Seed | Policy | Population | Endpoint need-weighted hunger | Council cash | Cumulative household relief | Operator completed work | Operating margin |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | Existing | 148.20 | 0.0582 | 14,466 | 2,751 | 82.67 | 422.22 |
| 1024 | Needs first | 150.75 | 0.1170 | 4,342 | 14,480 | 22.87 | 97.88 |
| 256 | Existing | 367.29 | 0.0161 | 9,036 | 7,517 | 9.24 | 28.40 |
| 256 | Needs first | 364.04 | 0.0158 | 3,632 | 13,907 | 4.69 | 19.93 |
| 409 | Existing | 345.94 | 0.0451 | 3,848 | 14,730 | 175.40 | 868.44 |
| 409 | Needs first | 348.21 | 0.0471 | 2,300 | 18,318 | 179.37 | 929.77 |

The intervention deploys money: relief increases in every world and ending council
cash falls. It does **not** establish an overall improvement. Population increases
in two worlds, endpoint hunger increases in two, and operator work falls in two.
Research completions fall from 4/3/4 to 3/1/2. Welfare competes with future research
and other public spending; these divergent histories do not isolate that as the
sole cause of fewer completions.

At month 600 in seed 1024, councils 0 and 1 have no opening relief cash against
requests of 47.53 and 5.09. Councils 2 and 4 retain about 2,578 and 839 with no
request; council 3 covers its 0.30 request. Town 2's households hold about 7,416,
but receive no food and the town has zero food stock. That is a physical food
constraint, not a reason to issue more welfare money there. Town 0 retains ample
food while its council cannot cover poor households' purchasing gaps.

Seed 256 similarly has three councils with zero opening relief cash and unmet
requests, while another holds about 3,090 and fully covers its 0.73 request.
Aggregate public reserves conceal unequal fiscal jurisdictions. Household cash
totals also conceal unequal wallets.

Retain this as an **opt-in counterfactual**, not a new default or a solved
circulation claim. The next welfare question is sustained local fiscal funding
and observed intergovernmental assistance. Workshop customers, available inputs,
knowledge and working finance remain separate mechanisms.

## Verification and reproduction

Run the owning analytical and GPU boundary fixtures:

```sh
CARGO_INCREMENTAL=0 cargo test --lib council_allocation -- --include-ignored
cargo clippy --all-targets -- -D warnings
```

Five tests pass. The controlled low-target/low-share comparison verifies larger
actual household transfers, the protected allowance, monetary balance and archive
round-trip. Existing fixtures cover inactive demand and subsequent administrative
payment/continuation. This change does not claim a new full-world batch or
cross-hardware equivalence study.

All three native runs complete. Independently audited relative money residuals
are at most 1.91e-7 in the intervention runs. Use
`python3 scripts/audit_circulation.py PATH_TO_HISTORY.json` for the disjoint cash
audit. Cumulative relief is a flow and must not be added to ending cash stocks.
