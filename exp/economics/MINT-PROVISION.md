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

The default `FullBuffer` policy distinguishes:

| Choice | Interpretation | Work response |
| --- | --- | --- |
| `Covered` | Food is held for this month, and the horizon is covered by held food plus affordable, accessible wheat | Withhold input offers; permit leisure |
| `SeekIncome` | Expected wheat access is sufficient, but coins do not cover the horizon's food gap | Offer an input or permit the configured earning activity |
| `NoFoodAccess` | Available wheat, market access or consumption permission cannot support the projection | Do not accept coins as a solution; no leisure credit for blocked work |
| `AwaitFood` | Cash could cover a useful purchase, but this month's food was not acquired before the market boundary closed | Wait for the next purchase opportunity; do not count it as covered leisure |
| `AwaitOpportunity` | Incremental mode has funded the currently reachable portion, but the full buffer is not covered | Wait and reassess; do not credit leisure or earn coins for nonexistent food |

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

## Incremental provision and changing circumstances

`Policy.goal` now selects `FullBuffer` (the existing controls) or `Incremental`.
Both retain the same full horizon and the same definition of `Covered`. The new
variant changes the size of the next income target when the full buffer is not
yet covered:

1. Current food purchases still clear first from opening money.
2. Count the whole food lots existing coins could buy.
3. Seek funds for at most one additional month's portion, capped by the remaining
   horizon deficit and expected accessible wheat. If less than a month's portion
   is available, that smaller improvement can still justify earning.
4. Set the input ask from this incremental cash gap and the existing price floor.
5. Recompute after committed income and at subsequent boundaries. There is no
   sticky refusal, promise of future replenishment, or saved plan to execute after
   it becomes stale.

A funded partial target produces `AwaitOpportunity` if current food is held, or
`AwaitFood` if the agent must wait for another purchase boundary. Neither counts
as full coverage or leisure. With no expected accessible wheat, additional coins
still have no modeled provisioning benefit. The policy does not invent food or
permit spending today's wage in today's earlier purchase window.

The comparison changes only the policy between paired runs. The tight-granary
case begins with four wheat rather than the six-wheat scarcity control above.
The recovery case adds four units of sealed, finite state grain reserve to those
same four market wheat. A normal process converts that reserve into market wheat
in month 3 Productive. It consumes the reserve; it does not generate free grain.
Agents do not count it before release, and month 3 Acquire cannot spend its
later output. Updated availability is observable at month 4 Acquire.

| Six-month case | Unmet food: FullBuffer | Unmet food: Incremental | Mint batches: full / incremental |
| --- | ---: | ---: | --- |
| Adequate wheat | 0 | 1 | 3 / 4 |
| Tight granary (four wheat) | 10 | 8 | 0 / 1 |
| Empty granary | 12 | 12 | 0 / 0 |
| Endowed people | 0 | 0 | 1 / 1 |
| Tight granary + delayed reserve release | 10 | 6 | 0 / 3 |

The tight case shows the intended improvement: earning for one more meal is
useful even without a full buffer. In recovery, incremental agents record no
food access in month 3 Acquire, seek income again in month 4, and complete later
mint exchanges in months 5 and 6. Production and purchase delays still cause
shortfalls; recognizing improved circumstances does not instantly deliver food.

The adequate case is a regression in welfare, retained explicitly as evidence.
Incremental suppliers accept smaller payments, work more and build less financial
protection. Minting completes in months 1–4, consuming all available metal/ore;
the supplier subsequently lacks income for one meal despite wheat being present.
The original full-buffer variant completes in months 1, 3 and 5 and avoids that
shortfall. Incremental mode takes no leisure in this case, versus six hours in
full-buffer mode. It is therefore an opt-in strategy, not a replacement default
or proof of generally improved behavior. A later policy could consider production
cadence and fallback options when choosing how aggressively to build reserves.

Run the paired CPU comparisons from `exp/economics`:

```sh
cargo +1.92.0 test --locked --test provision_policy
TELEMETRY_DIR=../../output/economics/provision-policy-new \
  cargo +1.92.0 run --locked --example provision_policy
```

Five new tests preserve both the gains and the adequate-supply regression. They
also check finite reserve release, no anticipation of same-month output, waiting
versus leisure, CPU/reference equality, reordered inputs, restart after each
phase and observer transparency. All 39 tests across this comparison and the
four previous minting suites passed, along with all-target Clippy. Observer
provision records now include `goal` and `purchase_target` (additional food units,
not the full horizon requirement); `cash_gap` refers to the selected target.

## Limits

This policy covers one recurring need, one food market and a short fixed horizon.
The default policy refuses when the full projected buffer is unavailable; the
incremental variant allows partial improvements but does not optimize a complete
future plan. Neither policy forages, migrates, bargains or borrows.
Leisure is an optional alternative once provision is covered, without a modeled
utility curve, fatigue recovery or personality. Nutrition deficits are reported;
this scenario does not attach mortality or health-condition rules to them.

State grain and ore remain finite initial endowments. State price, input ceilings
and mint dates remain configured. Equal-share expectations do not establish future
claims, and there is no price learning or inflation model. The experiment tests
whether goods access can ground willingness to accept currency, not whether the
resulting economy or monetary policy is sustainable indefinitely.
