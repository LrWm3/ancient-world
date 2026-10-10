# Choosing discovered market supply

Implemented 2026-10-10 after an independent review of the
[self-starting economy](ENDOGENOUS-DISCOVERY.md). The first gap selected for work
was supplier agency: previously, any recurring need withheld all of a person's
labor, while stock suppliers retained no need-derived reserve. That could prevent
a fed worker from earning coins and expose another supplier's productive inputs.

## Decision and execution

At Open, discovery builds eligible counterparties from the existing venue and
listings. Each person compares whole-lot outgoing resource portfolios with
declining all sales. A private forecast runs ordinary Open regeneration and Due
settlement before hypothetically subtracting the proposed outgoing resources at
Acquire. This month's known capacity overrides remain visible. Hypothetical
proceeds are recorded but never credited to the forecast or live balances.

The choice protects signed household labor contributions, horizon consumption
stock floors and accepted claims/entry inputs through the existing protected-stock
calculation. Forecast objectives protect deaths, need shortfalls and failed
processes, including active work with no need goal. For household members, these
comparisons include the other living members. Every measured loss must remain no
worse than declining. Earlier selected sales remain in the portfolio while later
resources are considered: independently useful substitutes cannot both be treated
as free surplus.

The selected quantity becomes a retained holding floor and an explicit quote lot
ceiling. Fixed and provisioning order generation both respect that ceiling, even
if inventory changes before submission. No candidate comparison reserves labor
or transfers property. Ordinary Acquire matching, finite budgets, reservations,
settlement and later process execution remain authoritative. Monthly phases and
market allocation policy are unchanged.

`World.discovery.supply` retains month, person, resource, available/protected
quantity, selected lots, baseline losses and tried alternatives, including failed
forecasts. These are decision diagnostics. Order plans, deals, transactions and
process status separately establish what was submitted, traded and completed.
Passive stock owners without an activity/consumption model retain their accepted
claims. Failure to compute the no-sale baseline aborts Open atomically; failure
of a candidate forecast rejects only that candidate.

## Controls

`tests/discovered_supply.rs` demonstrates:

| Opening condition | Observed result |
| --- | --- |
| Hungry person has two hours and a two-hour food recipe | No labor offered |
| Same recipe and needs, four hours | One two-hour lot offered |
| Four metal, food production needs two | One two-metal lot offered; inputs retained |
| Either two metal or two hours can produce food | Metal offered first; labor then retained |
| Known opening shock reduces four nominal hours to one | One hour available; no whole lot offered |
| Active non-need production requires the available hours | Labor retained to avoid process failure |
| Passive person owns stock but has no activity participant record | Stock supply is evaluated without requiring a consumption model |
| Worker begins with fourteen food and recurring nutrition need; competing people each have one hour; forward valuations omitted | Worker sells labor for coins, food remains covered and minting completes over fourteen months; CPU/reference, phase reconstruction and books agree |
| Same opening worker with forward valuations enabled | Two wheat are sold and delivered; worker develops food deficits in months thirteen and fourteen despite holding coins |
| Same fed worker, ordinary competing endowments | Worker offers labor but the lower-ID supplier fills the trades |
| Worker begins with only one food | Positive supply offers coexist with later food deficits |

The paid-worker control deliberately prevents competitors filling a two-hour lot.
It also disables their viable household cultivation; their observed food deficits
are asserted. It proves worker supply and payment, not general economic viability.
Its forward valuations are now explicitly empty while loan discovery remains enabled.
The paired full-finance control preserves a limitation exposed by whole-lot buying:
four-month forecasts approve one-wheat forwards in months five and eight, delivered
in months seven and ten. The later food shortfalls lie outside both forecasts.
Successful supply/settlement is not a guarantee of subsistence over the entire run.
The [twenty-iteration follow-up](REVIEW-BATCH-20.md) adds an optional independent
forward-assessment horizon: fourteen months rejects that harmful buffer sale
without inflating market stocking targets. The original default short-horizon
failure remains a supported control; longer forecasting is not a universal cure.
The unchanged `tests/discovery.rs` baseline remains the integrated cultivation,
annual payment, financing and forty-month renewal control.

`tests/mint_orders.rs` additionally checks that extra inventory cannot exceed an
authorized ceiling, invalid ceilings are rejected, and provisioning honors both
buyer and seller ceilings.

Independent follow-up review caught two integration issues before completion:
provisioning also needed to enforce buyer ceilings, and passive persons could
reach a consumption helper requiring an activity participant. Both now have
regression controls. Review also required actual worker trade/payment evidence
instead of treating an expiring labor balance as evidence of a sale.

Run from `exp/economics`:

```sh
cargo +1.92.0 test --locked --release --test discovered_supply --test mint_orders --test discovery --test mint_provision --test mint_cycles --test mint_finance --test agency --test integrated_agency --test commitment_agency --test forecast_context
cargo +1.92.0 run --locked --release --example discovered_economy
```

Verification: **97 tests passed across these ten targets**, including nine new
supply controls. Strict all-target Clippy, formatting and repository artifact
checks passed. Both CubeCL CPU examples (baseline and `--surplus`) retain the
fourteen-month outcomes recorded in `ENDOGENOUS-DISCOVERY.md`: paid annual dues,
two initial food deficits, baseline loan repayment and two delivered surplus
forwards. This is targeted regression coverage, not a new full-suite certification.

## Limits and next review

This is bounded conservative supply selection, not a general market planner.
It tries at most sixteen lots per market in stable market order, choosing the
largest passing amount before considering the next resource. It does not optimize
all portfolio permutations. Each welfare dimension is protected separately,
rather than allowing tradeoffs between them. Private consumption floors span the
configured forecast horizon and can withhold useful inventories.

Supply forecasts remove **all** supplier sell quotes, freeze future discovery and
organization choices, and give no credit for prospective wages. Consequently,
they cannot discover a plan whose benefit depends on using new wage income to buy
food. Prices remain supplied limits, actual fills depend on competition, and
future food availability is not guaranteed by possessing coins. Constitutions,
laws, technologies, objectives and endowments remain inputs.

Opt-in independent public surplus sales are now implemented; the issuer protects reserve
and commitment claims before selling whole lots even without a mint-funding gap.
The [review record](REVIEW-ITERATIONS.md) retains the original wage/food control,
and [financed circulation](FINANCED-CIRCULATION.md) combines it with household
farming, annual dues and lending. Its calibrated worker receives two jobs and later
food, but still has five fourteen-month nutrition deficits. The opt-in
[private circulation follow-up](PRIVATE-CIRCULATION.md) now adds person/household
asks and collective purchases on that food listing, with live claim protection and
external budget receipts. Household venue admission remains explicit. Sustained
worker income in this discovered control and broader market discovery remain open.
Successful
repayment without ore is retained as a counterexample to treating finance as proof
of completed productive work.

Voluntary land requests remain another bounded gap: distinguish choosing a
useful, fulfillable commitment from admission merely determining that an agent
could accept it. Broader financial use selection and recruitment remain later work.
