# Alternative food and foraging on CPU

Implemented: one nutrition requirement can be met by grain, wild food, or both.
Foraging supplies immediate food at higher labor cost, and the planner can compare
it with crop work. All behavior uses generic processes, stock accounts and needs.

## Model and monthly boundaries

| Process | Inputs | Output | Duration |
| --- | --- | --- | --- |
| Grow grain | One seed, plot access, eight labor across stages | Eight grain and one seed | Six months |
| Forage | Two labor and one unit of shared wild supply | One wild food | One month |
| Eat grain | One grain | One nutrition | Consumption boundary |
| Eat wild food | One wild food | One nutrition | Consumption boundary |

Foraging therefore costs two labor per provision versus one for the complete crop,
but delivers at the current Productive boundary. It can be eaten at the subsequent
Consumption boundary. No scheduler phase or farmer class was added.

A generic Pool holds finite harvestable stock in a designated account. This fixture
uses a state-held environmental account, capacity six, initially full, regenerating
one unit per month up to its cap at Open. It is not food already owned by the person.
A PoolInput catalog permission binds the foraging process to that account. All
operators of this definition are currently eligible; there is no ecological access
market. The account owner receives no automatic payment.

At Productive, requests draw from the same opening pool and labor budgets. The
existing explicit request order awards whole feasible actions; no partial foraging
reservation wastes the pool. Successful work emits the shared-stock debit, own-labor
debit and owned-food credit in one validated transaction. Failed requests spend
nothing. Multiple claimants cannot each spend the opening supply. Stable agent IDs
break equal-priority ties; this is deterministic priority, not fair sharing.

Regeneration is a dated source event, verified at Open and replayed exactly once.
It never replenishes a pool between requests in the same boundary. The fixture's
one-person extraction rate does not exhaust the pool; separate two-claimant tests
with a one-unit, nonregenerating pool establish the actual bound and later refill.

## Substitution and decisions

Consumption shares one stock budget across recipes and needs. It first uses food
not earmarked for known payments within the configured buffer window, then releases
those earmarks if necessary to eat. Whole lots, actual stocks and already supplied
nutrition bound consumption. More output per input unit is preferred, then stable
recipe ID. The efficiency comparison assumes these editable input quantities use
comparable abstract units; this is not a general preference or price model.
Different foods have no taste, spoilage or nutritional-composition differences yet.

Alternative recipes using the same resource do not multiply its availability.
Buffer scoring allocates stocks month by month: surplus fulfillment expires and
cannot be counted as food for a later month. ProtectEssentials uses the same food
substitution, so wild food can protect nutrition while grain pays rent. Earmarks
are consumption preferences, not escrow or a payment-policy override.

For a need with alternative recipes, candidate generation compares productive
chains against a shared projected food budget, including dated outgoing claims.
A wild-food buffer can suppress unnecessary grain production. This remains an
optimistic two-link candidate heuristic; ordinary forecasts validate actual work,
production, resource conflicts and consequences.

The decision portfolio now includes a current-action preference for each enabled
substitute producer. That preference can put foraging ahead of an active crop, so
its cost is evaluated rather than silently excluded. It applies only to the first
productive boundary; future rollout months use the existing fixed allocation policy.
The selected action and forecast retain its preferred process ID. Existing offer
acceptance still stores the exact dated productive plan. Search is capped at four
substitute producers; with four needs and four offers the upper bound is 150
alternatives. It is not exhaustive joint-action search or recursive optimization.

## Measured comparison

Each arm executes nine months on CubeCL CPU and on the Rust reference. The active
crop in shortage cases is created by ordinary prior simulation; then a documented
opening food shortage is applied. No future rescue or fixture shock is visible.

| Scenario | Opening / difference | Forages | Harvests | Food / warmth deficits | Outcome |
| --- | --- | ---: | ---: | ---: | --- |
| forage-abundant | Month 1, 80 grain, normal two labor | 0 | 0 | 0 / 0 | Ends with 71 grain; no unnecessary food work |
| forage-bridge | Month 5, zero grain, active crop, **three labor**, six fuel | 1 | 1 | 0 / 0 | Forages alongside tending, harvests in month 6 |
| forage-conflict | Same shortage, **two labor**, six-month decisions | 8 | 0 | 1 / 2 | Forages now, aborts crop, later runs short of heat |
| forage-conflict-long | Identical state and rules, **eighteen-month decisions** | 0 | 1 | 1 / 0 | Accepts the month-5 food deficit, preserves harvest in 6 |
| forage-urgent | Conflict opening; one missed food provision is lethal | 8 | 0 | 1 / 2 | Forages to avoid immediate death; dies in month 13 |
| forage-harvest | Month 6, zero grain, crop ready, normal two labor | 0 | 2 | 0 / 0 | Harvest itself feeds the person immediately |

The bridge changes available labor and is not evidence that foraging is free under
normal capacity. Its otherwise identical disabled-foraging control misses one food
provision. In the urgent control, disabling foraging causes immediate death at the
first Close; foraging extends survival but does not establish sustainable recovery.

The conflict pair changes only decision horizon. Six-month foresight values immediate
relief and cannot see the later warmth shortage after its fuel buffer runs out.
Eighteen-month foresight sees that conflict and preserves productive capacity.
The harvest-month control matters: harvest output is edible later in the same month,
so zero opening grain alone is not a reason to abandon a ready crop.

## Reproduction and checks

From exp/economics:

```sh
cargo +1.92.0 run --locked --example foraging_audit
cargo +1.92.0 run --locked -- forage-conflict-long
cargo +1.92.0 test --locked
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
cargo +1.92.0 fmt --check
```

The example checks CPU/reference state, full ledgers and reports, plus independent
pool accounting. Seven new tests cover mixed foods, recipe aliases, overlapping
needs, indivisible lots, fulfillment expiration, payment-stock preservation,
finite shared extraction, capped regeneration, invalid regeneration, duplicate
replay, crop contention and urgent consequences. Every new scenario is checked
against reversed catalog order, monthly stepping, ledger replay, selected current
forecast outcomes and continuation from every opening-month boundary.

The full suite has 76 passing tests; formatting and strict all-target Clippy pass.
Raw traces and audit output stay under ignored output/economics. These are
short deterministic comparisons, not a long-run balance or ecological calibration.
Further work should test seasonal/depleting supply and longer trajectories before
interpreting foraging as a generally sustainable fallback.
