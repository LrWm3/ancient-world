# Purchasing-power-sensitive input offers and optional leisure

Implemented as an opt-in policy in the isolated physical-minting experiment.
Suppliers now have recurring nutrition needs and decide whether coin income can
help provide food. State wheat sales operate independently of its minting funding
gap. A covered participant can explicitly choose leisure instead of producing or
offering more inputs. Fixed-order and repeated-issuance controls remain available.

## Needs, provisioning and work

Each person needs one nutrition unit every month. The existing Consumption phase
consumes one wheat to produce one nutrition fulfillment; monthly reports retain
fulfilled and unmet quantities. Wheat purchases target the current month's deficit.
The policy looks ahead three months, including the current month, when deciding
whether more income is useful. The consumption recipe and participant requirements
supply the quantities; the policy does not identify people by occupation.

The policy distinguishes:

| Choice | Interpretation | Work response |
| --- | --- | --- |
| `Covered` | Food is held for this month, and the horizon is covered by held food plus affordable, accessible wheat | Withhold input offers; permit leisure |
| `SeekIncome` | Expected wheat access is sufficient, but coins do not cover the horizon's food gap | Offer an input or permit the configured earning activity |
| `NoFoodAccess` | Available wheat, market access or consumption permission cannot support the projection | Do not accept coins as a solution; no leisure credit for blocked work |
| `AwaitFood` | Cash could cover the projection, but this month's food was not acquired before the market boundary closed | Wait for the next purchase opportunity; do not count it as covered leisure |

Expected future access is deliberately conservative: each admitted configured
participant estimates an equal whole-lot share of the state's remaining wheat
following current food sales. This is a forecast from public inventory, not a
right, reservation or guarantee. Actual matching retains price priority and
agent-ID tie breaks. Expectations can differ from realized allocation, especially
under scarcity. No hypothetical replenishment or future minted income counts as
food provision.

A seller's minimum ask is the larger of its configured floor and the coin gap to
cover its horizon, rounded up to the market tick. One input lot is offered per
seller/listing. Higher wheat prices therefore raise asks; coins alone do not make
an empty granary valuable. This is a simple reservation-price rule, not ZIP or a
full labor supply/utility optimization. A participant selling several different
inputs can overfund the same gap; the initial scenario has one sale channel each.

## Independent food sales and conditional procurement

The Acquire boundary first previews current wheat purchases against opening
stocks, coins and storage. That preview informs input offers and reservation asks.
The resulting combined order book uses the existing transfer and reservation
primitives:

1. State food asks remain available even when treasury covers minting, mint
   permission is absent, or all configured mint dates have passed.
2. Current food sales settle in independent packages. They are preserved if an
   input package cannot match.
3. Mint inputs remain one conditional package: all required lots or none. State
   bids still require opening treasury sufficient for their configured ceilings.
4. Incoming food-sale proceeds and input-sale wages are never added to spendable
   resources in the same Acquire batch. Today’s wage cannot buy today's food.
5. Productive work sees committed balances. Consumption follows Productive.
   No work or food consumption is moved backward to change a completed boundary.

The scoped work selector chooses between each configured earning process and
leisure at Productive, using the committed state. This is one policy hook at
activity request generation; generic activity execution, process accounting and
capacity limits remain in place. It is not yet a universal needs-based policy for
all scenario drivers.

## Leisure is an explicit completed process

The leisure recipe spends two labor hours and emits two units of a perishable
leisure record. The record expires at the next Open; completed process history
and telemetry retain the actual hours taken. It creates neither coins nor food,
and is not a mandatory need in this pilot.

Sold labor cannot also be used for leisure. A metal supplier who sells inventory
may still have its own hours available and choose leisure after its income covers
food provision. A supplier facing blocked markets or hunger does not receive
leisure records merely because its hours remained unused. The selector does not
force work for its own sake.

## Six-month CPU controls

The state starts with 12 coins; each person starts with three, so total opening
coin supply is 18. Wheat costs three coins per unit. The state bids at most six
coins for each of the metal and labor lots. Minting creates ten coins per batch.
The supplier starts with one metal lot and six finite ore units and can replenish
metal using its own labor. Mint opportunities exist in every month from 1 to 6.

| Case | Opening state wheat | Mint months | Unmet nutrition units | Completed leisure hours | Closing coin supply |
| --- | ---: | --- | ---: | ---: | ---: |
| Adequate | 18 | 1, 3, 5 | 0 | 6 | 48 |
| Scarce | 6 | 1 | 6 | 2 | 28 |
| Empty | 0 | None | 12 | 0 | 18 |
| Endowed people, six wheat each | 18 | 6 | 0 | 22 | 28 |

Adequate supply meets all 12 nutrition requirements. People initially earn enough
to cover their horizon, then use leisure; later deficits prompt renewed production
and offers. The state's ending coin balance is 42 and each person's is three:
food purchases return coins to treasury rather than letting private money grow
without a use.

Scarce wheat allows the first exchange but eventually removes the expected benefit
of earning more coins. The empty-granary control refuses coin offers even with
additional private coin endowments. In the endowed-food case, no mint inputs are
offered initially because people can cover their needs. Income becomes useful in
month six as the rolling horizon extends beyond their remaining stocks; that
horizon does not stop at the example's reporting cutoff.

Additional controls raise wheat from three to four coins, raising both initial
input asks from six to eight. Those asks no longer cross six-coin state bids, so
food can sell while mint procurement fails. With no initial private coins but
funded nine-coin bids, wage exchanges can occur, yet current food remains unmet:
new wages only finance a subsequent boundary, and no leisure is credited for that
current shortage.

## Running and verification

From `exp/economics`, use a fresh ignored output directory:

```sh
cargo +1.92.0 test --locked --test mint_provision
MINT_PROVISION=true TELEMETRY_DIR=../../output/economics/mint-provision-new \
  cargo +1.92.0 run --locked --example minting
```

`MINT_PROVISION` takes precedence over `MINT_CYCLES` and `MINT_ORDERS`. The runner
prints coin balances, food deficits and leisure sessions and writes JSONL. Order
observer records include each participant's food target, held food, expected
access, coin gap and choice. These are decisions after previewing food sales;
subsequent income can change the Productive choice. Transaction/process records
show actual work and leisure. The observer does not alter decisions.

Ten focused tests cover the controls, reservation-price response, independent
food settlement, post-target food sales, permissions, wage timing, actual leisure,
finite labor, invalid policies, forged decision receipts, CPU/reference agreement,
reordering, restart at every phase and observer transparency. The existing minting
suites remain the regression controls for prior behavior. All 66 tests across
these four minting suites, activities, economics and telemetry passed, along with
all-target Clippy, formatting and the repository artifact check.

## Limits

This policy covers one recurring need, one food market and a short fixed horizon.
Food scarcity causes refusal when the full projected buffer is unavailable; the
policy does not rank partial improvements, forage, migrate, bargain or borrow.
Leisure is an optional alternative once provision is covered, without a modeled
utility curve, fatigue recovery or personality. Nutrition deficits are reported;
this scenario does not attach mortality or health-condition rules to them.

State grain and ore remain finite initial endowments. State price, input ceilings
and mint dates remain configured. Equal-share expectations do not establish future
claims, and there is no price learning or inflation model. The experiment tests
whether goods access can ground willingness to accept currency, not whether the
resulting economy or monetary policy is sustainable indefinitely.
