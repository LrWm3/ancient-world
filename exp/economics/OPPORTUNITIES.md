# Individual opportunity marketplace and state permissions

Status: implemented CPU experiment, `opportunity-farming`.
The current scenario extends this original slice with
[citizenship as an enabling agreement](CITIZENSHIP.md); its physical production
and tax terms are unchanged. This returns to one
person, cultivation and firewood collection. Earlier scenarios remain controls.

The individual discovers process and state-access offers through one catalog
interface. Starting at nutrition and warmth needs, backward resource links identify
productive opportunities and the asset access they require. A cultivation offer
is visible before the person has acquired its prerequisites. Discovery makes no
reservations and grants no rights. Seed/output cycles terminate through visited
resource IDs.

Offers reference existing process definitions and agreements rather than duplicating
terms. Physical stages still enforce inputs, duration and labor; accepted state
agreements still enforce access and annual payments. Environment offers have no
wallet, bargaining agent or promise of a guaranteed yield under arbitrary conditions.
This slice has deterministic yields.

| Opportunity | Contributions and terms | Outcome |
| --- | --- | --- |
| State plot access | Two grain annually, first due 12 months after acceptance | Exclusive usable plot right within its dated term |
| Cultivation | One seed at planting; labor 2, 1, 1, 1, 1, 2 across six months; plot access throughout | Eight grain and one seed at completion |
| Collect firewood | One labor and one unit from a finite shared wood pool | Two fuel |
| Consume grain | One grain | One nutrition for this month |
| Use fuel | One fuel | One warmth for this month |

The person opens with five grain, one seed, one fuel and three labor per month.
The wood pool holds at most 12 units and regenerates one per month. Nutrition and
warmth each require one unit monthly. Existing deprivation/capacity/death rules
remain active. Missing required process work aborts the process, gives no harvest,
and does not refund already consumed seed or labor. Unpaid land obligations block
new cultivation; the existing rule allows an active crop to finish.

## Planning and execution

The existing scheduler remains authoritative:

1. Open refreshes labor, expires last month's fulfillment, and regenerates wood.
2. Due settles dated annual obligations.
3. Acquire discovers linked access offers and forecasts acceptance plus productive
   schedules against the same opening resources. The 18-month rollout includes
   the first annual payment; governed access forecasts have a minimum 13-month
   window. Alternatives with forecast unpaid obligations are excluded.
4. The selected acquisition commits its agreement and dated current production
   plan. Productive executes that plan once, consuming seed/labor/wood and advancing
   crops. Future labor is projected, not withdrawn from future months today.
5. ClearArrears can settle claims from completed production. Consumption spends
   actual stocks, then Close records need consequences.

The agent compares the existing bounded portfolio of continuation, need-priority
and deferred-start policies. This can allocate warmth work alongside cultivation;
all alternatives execute through ordinary settlement in their forecasts. It is
not an arbitrary-depth optimizer. Backward reachability discovers prerequisites;
only actual dated forecasts establish this experiment's feasibility. It does not
provide a general seed-purchase search or jointly accept multiple prerequisite
agreements in one boundary. Forecast solvency is not a guarantee against future
shocks or obligations beyond the finite horizon.

## State permissions as a foundation for laws

`World.transaction_policy` identifies the issuing authority, maps agent IDs to
agent types and provides an explicit allow-list of transaction actions per type.
The original direct-permission fixture permits persons to accept state land
access, cultivate, collect wood, consume grain and use fuel. The current CLI
scenario grants land access and cultivation through Citizen membership instead,
and permits persons to accept the posted citizenship offer. Types and process IDs are data; the planner contains
no person/farmer decision branch. Unclassified agents and unlisted actions are
denied. The state type has no autonomous action permissions in this fixture.
Accepting land access authorizes the agreement's subsequent automatic payments;
period regeneration and physical consequences are system events, not voluntary
person transactions.

Discovery includes currently permitted offers and offers enabled by obtainable
membership. Actual permissions are checked again in process preparation,
access acceptance and settlement. A scheduled start or fabricated batch cannot
bypass them. Denied work has a `NotPermitted` receipt; rejected settlement publishes
no state changes. Revoking permission prevents subsequent execution, including
existing work; normal resolution can abort the affected process. A stale stored
plan is rejected atomically rather than silently altered.

This is a single-authority permission policy, not a complete legal system. There
are no jurisdictions, enactment dates, enforcement probabilities, penalties,
appeals or grandfathering rules yet. Those can later determine the permission
query without becoming branches in the individual decision-maker. Permissions
also do not create physical resources or replace right/stock/capacity checks.
Governed worlds support bilateral marketplace `StockTrade`, credit's finite state
stock bid (both parties checked), and `FinancedPurchase` origination. Existing
citizenship may grant these actions; accepted debt still settles if permission is
later removed. See the [integration matrix](INTEGRATION-STATUS.md). Older equipment,
specialist-market and household configurations still require permission semantics.
Worlds with no transaction policy preserve previous behavior.

ZIP is not implemented. It can later supply negotiable prices for trading offers;
physical input requirements remain process terms.

## Verification

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked -- opportunity-farming
cargo +1.92.0 test --locked --test opportunities -- --nocapture
```

The 36-month CubeCL CPU run completed harvests in months 6, 12, 19, 25 and 33,
with a sixth crop active at the end. Both taxes (months 13 and 25) paid two grain.
Nutrition and warmth deficits were zero throughout; no processes aborted and the
person remained active. Ending personal stocks were five grain, zero unplanted
seed and five fuel; the active crop had consumed the next seed. The state held
four collected grain. Firewood collection completed 20 times.

CPU and reference settlement produced identical state and ledgers. Monthly
continuation from a month-12 checkpoint matched the batched run. Controls cover
missing seed, unaffordable taxes, denied access, denied cultivation, denied wood
collection, agent-type differences, scheduled-start bypass attempts and permission
changes before settlement. These are controlled feasibility results, not evidence
of an optimal planner or calibrated economic balance. Raw output stays under
ignored `output/economics/`.

The full crate regression suite passed (140 tests); the seven opportunity tests
also passed after the final forecast-feasibility refinement. Formatting, Clippy
with warnings denied, and the repository artifact check passed.
