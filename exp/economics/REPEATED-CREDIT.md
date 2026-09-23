# Repeated production after a financed plot purchase

The two missed meals in the production-funded credit pilot came from a fixture
permission limit, not a failure to plan another crop. Its ownership-following use
right was copied from the nine-month baseline. At month 9, the person had five
grain and the returned seed, but a new six-month crop would finish in month 14.
The right ended in month 9, so the existing feasibility check excluded cultivation.
Slow gathering was feasible and consumed the available labor, but yielded too
little to maintain food supply. Deficits appeared in months 16 and 18.

The correction gives this pilot an explicit cultivation right through month 120.
It still follows plot ownership, including repossession. Finite rights remain
finite: buying an asset does not globally override permissions or extend every
right. The generic permission, planning and settlement code is unchanged. Other
credit and baseline fixtures retain their original terms.

## Controlled results on CPU

Only the right's expiry differs between these runs. Starting food, seed, coins,
loan, prices, sale limits, storage and monthly labor are identical. Production
still uses a six-month crop with twelve grain and one seed output. The existing
six-month food reserve and the state's total 120-coin purchase allowance remain.

| Outcome | Right ends month 9 | Right ends month 120 |
| --- | --- | --- |
| Harvest months through month 60 | 6 | 6, 14, 22, 32, 44, 56 |
| Nutrition deficit units through month 18 | 2 | 0 |
| Nutrition deficit units through month 60 | 30 | 0 |
| Mortgage repayment month | 13 | 13 |
| Total state grain purchases through month 60 | 4 grain / 48 coins | 10 grain / 120 coins |
| Person's closing coins at month 60 | 7.80 | 79.80 |

The corrected agent plants again in months 9 and 17. After the state's purchasing
allowance is exhausted, it retains more food and plants less frequently: months
27, 39 and 51. No additional income, seed or labor is injected. At every committed
boundary, the original seed is either held or invested in one active crop; all
started crops finish. Continued production does not require an infinite state bid.

The corrected 18-month funded case now sells eight grain for 96 coins and closes
with 55.80 coins. Limited purchasing allowance and the lower-price case still
reject borrowing for installment shortfalls. Removing the food reserve still
allows harmful sales: the corrected unprotected control incurs nine nutrition
deficit units, even though it repays. Thus the cultivation fix does not resolve
the separate need for an absolute deprivation constraint on borrowing.

## Verification and limits

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example repeated_credit
cargo +1.92.0 test --locked --test stock_sale --test borrowing --test credit --test credit_offers
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

The 24 tests passed. The stock-sale suite retains the short-right control,
checks seed conservation at every boundary over 60 months, verifies completed
harvests and zero deficits, and compares CPU/reference, monthly/batched and
checkpoint-resumed execution with reordered catalogs through month 60. All-target
Clippy passed. Raw audit output is ignored under `output/economics/`.

This establishes repeated production under deterministic conditions for five
years. It does not establish resilience to poor harvests or labor loss. Rights
still expire in month 120; renewal, perpetual owner permissions and a generalized
relationship between title and cultivation rights are separate design choices.
The borrowing forecast remains 18 months; the 60-month audit observes continued
ordinary decisions rather than granting the agent additional foresight.
