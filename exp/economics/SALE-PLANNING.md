# Bounded sale planning

An opt-in sale policy now tests current stock-sale quantities through ordinary
production, consumption and loan settlement. The original fixed-reserve fixtures
remain controls; `stock_sale::forecast_scenario` enables this comparison with a
six-month forecast, zero fixed food reserve and zero allowed nutrition deficit.
Productive input reserves remain enforced before candidate generation.

## Decision and execution

Acquire first selects the financed-purchase branch, then bounds potential sales by
stocks, productive input reserves, monthly lot limits, the state's remaining
purchase allowance, opening cash and receiving storage. `sale_plan` evaluates
zero through that many lots using sanitized `ForecastContext` clones. Each
alternative reproduces the selected purchase or decline branch and settles the
specified quantity through existing stock-trade transactions. Recursive borrowing
and sale searches are disabled inside these inner rollouts.

The rollout includes ordinary planting, active crop work, consumption, interest,
payments and repossession. It assumes no further sales after the candidate. The
live system replans at each subsequent Acquire. Future scripted cash gifts and
capacity overrides are not forecast knowledge. Sale proceeds still cannot pay an
installment already due earlier that month.

An alternative qualifies only if the seller remains active and projected unmet
units satisfy every configured need cap. Choose the largest qualifying quantity
at the posted price. If no quantity qualifies, sell nothing and report
`feasible=false`; waiting is not described as a successful remedy. Receipts retain
each quantity, deficits, terminal outcome, qualification and selected quantity.
Normal credit boundary validation recomputes the decision and rejects tampering.

Caps use generic provision IDs, with the same cumulative-unmet-unit interpretation
as borrowing limits. Sale limits and borrowing limits are configured separately:
they govern different actions and horizons. Sale forecasts require at least the
longest enabled productive-process duration, at most 24 months, and nonempty,
nonnegative caps on the seller's positive needs. Existing lot limits bound the
number of candidates. This remains the one-seller credit pilot.

## CPU observations

In the funded 18-month fixture, the agent rejects selling opening food: zero, one
and two initial lots project respectively zero, one and two nutrition deficits.
It sells two grain in months 7, 8, 15 and 16, repays in month 13, and finishes with
55.80 coins and no nutrition deficits. These outcomes match the fixed six-month
reserve, while now deriving the choice from dated production and consumption.

A controlled month-5 snapshot with three grain and a crop completing in month 6
permits selling two grain. The fixed six-month reserve sells none from the exact
same snapshot. The forecast sees that one remaining grain covers this month and
that the harvest supplies later consumption. A no-land, low-food control finds
no qualifying option and sells nothing.

The limited-allowance and low-price fixtures still reject borrowing. Their live
sale forecasts approve selling one opening grain because slow gathering covers
the six-month window. They subsequently accumulate eight nutrition deficit units
over 18 months, versus seven under the fixed-reserve controls. Thus a locally
admissible sale can worsen later food security. This policy is opt-in and is not
claimed to dominate the fixed reserve.

## Limits and next questions

This is a bounded quantity comparison using the existing production policy, not a
joint optimizer over planting schedules, multiple sales, borrowing and saving.
It maximizes current sales subject to short-window needs, not lifetime welfare.
The no-further-sales continuation avoids speculative income but can pessimistically
project debt trouble. It does not require each inner branch to repay; origination
still has the separate longer-horizon borrowing guard. Known harvests can fail
under later shocks, and no risk margin has been introduced.

The observed horizon problem motivates comparing longer horizons or a terminal
food/production viability requirement before making this policy the default.
Changing the sale objective to penalize harm relative to waiting is another
possible extension, but a short comparison alone cannot detect harm beyond its
window. No scheduler or allocation priority was changed here.

## Verification

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --example sale_plan
cargo +1.92.0 test --locked --test sale_plan --test stock_sale --test borrowing --test credit_offers --test credit
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

31 tests and all-target Clippy passed. New tests cover opening food protection,
near-harvest sales versus fixed reserves, no safe option, forecast horizon/cap
validation, hidden future shocks, forged receipts and CPU/reference equality with
monthly, batched, checkpoint-resumed and reordered-catalog execution. Raw example
output stays under ignored `output/economics/`.
