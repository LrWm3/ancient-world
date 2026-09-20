# Experiments for the generic agent model

[Specialized activities](ACTIVITIES.md) adds mining/refining, produced and repaired
tools, livestock, plot-attached housing, work targets and coin payment alternatives.

[Four people](FOUR-PEOPLE.md) extends the storage/currency fixture with three additional
people, shared-budget tests and a 60-month capacity comparison.

Latest extension: [storage and currency](STORAGE-CURRENCY.md) records fixed storage,
two-grain annual taxes, treasury issuance and grain sales, with a 60-month CPU comparison.

Status: scenario A, repeated harvests, competing warmth, generic consequences and
bounded consequence-aware forecasts are implemented and
have run through CubeCL CPU settlement; see [results](RESULTS.md). Scenarios B–F
remain proposals.
Expand only when the current scenario answers its architectural question. Record
negative and inconclusive findings as carefully as successful ones.

## First implemented slice

Use one generic agent with person components and one passive state agent owning a
plot. Give the person a valid use right, finite grain/seed stocks, monthly labor,
a nutrition need and access to growing and consumption process definitions.
A bounded planner selects a process chain, emits requests and starts, and the
resolver produces validated transactions and gathered state updates. No occupation
class or farming-specific phase is needed. Run scenario A before adding trade.

One tick in these scenarios is now a modeled month with the stages specified in
[the design](DESIGN.md#monthly-execution-and-visibility-barriers). Unused stages
remain empty. Production outputs can be sold at the later goods stage that month;
incoming proceeds cannot fund another transaction within the same matching round.
This supersedes the earlier blanket rule that offers and all tradeable production
had to wait until the next tick. Timing variants must be labeled and compared
explicitly rather than changed through agent iteration order.

The first result is a runnable scenario with a Markdown decision/transaction trace
and a curated CPU results summary. It does not include credit, demographic growth,
geography, institutions or a GUI. Barrier checkpoints are currently in-memory
clones, not files; deterministic conditions require no random seed.

## A. One person selecting and executing a process chain

Question: can a bounded planner connect a nutrition deficit to a multi-month
process through resources and rights, without occupation-specific behavior?

Use these intentionally abstract, deterministic scenario quantities. They test
execution and accounting, not agricultural yield realism.

| Item | Initial value or rule |
| --- | --- |
| Person | 5 grain units, 1 seed unit, 2 labor units per month |
| Need | 1 nutrition unit each month; no mortality model |
| Plot | One unit of exclusive capacity, owned by a passive state agent |
| Right | Person may use the plot through month 9 and owns the process outputs |
| Consumption | 1 grain produces 1 nutrition fulfillment for that person/month; integral lots |
| Growing: planting | Month of start: 1 seed and 2 labor; occupy the plot for six months |
| Growing: growth | Four subsequent months, each requiring 1 labor and continuing plot occupancy |
| Growing: harvest | Sixth month: 2 labor, producing 8 grain; release occupancy after settlement |
| Conditions | Always suitable in baseline; no stochastic yield |
| Failure | A missed required service or invalid right aborts the instance with no output or seed refund; release all remaining commitments |

At each planning boundary, forecast the current and next five months of nutrition.
Search only the known growing-to-consumption chain, credit active expected output
at its due boundary, and start at most one instance when it reduces a projected
shortfall. Use stable IDs for ties. Seed is distinct from consumable grain. Future
labor requirements are forecasts, re-requested each month; plot occupancy is a
firm dated reservation. There is no automatic replanting or seed creation.

Run nine months. At month 1, six months of needs exceed the five grain units on
hand, so planting is selected. Initial stock covers consumption in months 1–5.
Growth executes in months 2–5; harvest settles before consumption in month 6.
All nine monthly needs can be met: `5 + 8 - 9 = 4` grain remains. Exactly one seed
is consumed, productive work totals 8 units, and no output exists before month 6.
The exhausted seed stock prevents another start even if a later forecast needs it.

Use controls with explicit expected differences:

- Two initial grain units: the same harvest cannot repair shortages in months 3–5.
- No seed, missing right or a right expiring before month 6: no growing start;
  distinguish each rejection reason.
- Zero labor in month 3: abort the instance, retain sunk inputs/work in receipts,
  release future plot occupancy and never emit its harvest.
- A second competing process request at harvest: compare explicit priority policies
  on identical opening resources. A harvest needing 2 labor cannot run on a grant
  of 1; the resolver must not consume an unusable partial grant.
- Duplicate plot requests, including overlapping valid rights: at most one full
  occupancy reservation succeeds; permission does not multiply land capacity.
- No nutrition need or disabled growing definition: explain non-submission rather
  than invoking a farming update unconditionally.

Check period-specific fulfillment, expired unused labor, process advancement once
per month, output ownership, reservation release and no premature production.
Replay all committed process and consumption records from initialization; compare
inventories, rights, occupancy, progress and fulfillment. Add rights revocation and
cancellation controls when those commands exist, before permitting dynamic tenure.
Do not add mortality to make the first food shortfall harder to interpret.

## A2. Repeated harvests before adding agents

Implemented as `repeated-harvests` for 60 months. Retain A's opening person stocks
and process requirements; add 1 seed to each completed harvest's outputs and extend
the use right through month 240. The horizon stays six months. The original A
fixture is retained as a regression control.

Check repeated planting and harvest dates, no food deficits, seed in storage versus
seed committed to an active crop, no overlapping occupancy, and future completion
past the reporting cutoff. Include no-initial-seed and too-short-horizon controls.
The observed sequence completes seven crops and starts an eighth without losing
or accumulating seed; detailed balances are in [results](RESULTS.md#repeated-harvests-60-months).

## A3. A competing essential need

Implemented with 1 warmth/month alongside nutrition. Give the person 1 initial fuel
and 40 raw wood units. A one-month process consumes 1 raw wood and 1 labor to make
2 fuel; another consumes 1 fuel for 1 warmth fulfillment. Raw wood remains finite,
not an implicit regenerating forest. The existing labor budget is still 2/month.

Compare food-first and warmth-first need ranks on identical initial stocks, then
protect active work before ranking new starts. Include 3-labor controls for both
need orders, zero warmth demand and unavailable wood. Inspect completed harvests,
per-need deficits, fuel buffers, seed loss and unused partial labor grants. Compare
an identical opening harvest boundary and its requests to isolate policy effects.

Food-first meets both needs in all 60 months. Warmth-first loses the first harvest
to fuel-buffer replenishment; protecting active work prevents that loss but cannot
undo late planting. These are recorded failures of the heuristic under the chosen
strict process-failure rule, not grounds to backdate work or erase unmet needs.
A future experiment should separate essential deadlines from buffer targets before
adding agents or concluding that one fixed need ranking is generally best.

## A4. Generic requirement consequences (implemented)

Question: can unmet requirements affect future operation through the same rules
for a person and an institution, while provision, condition and decision policy
remain separate?

Compare the condition-enabled food/warmth cases with the existing controls.
Run independent institution fixtures with two initial upkeep supplies, a fully
supplied control, and finite resupply completing in month five. Observe provision,
deprivation, adverse duration, opening capacity, process failure and terminal date.
Do not score an early death as a reduction in unmet needs: demand ceases after the
terminal month and must be reported alongside survival duration.

The implemented evaluator accumulates deprivation from shortfall, permits explicit
recovery, applies the most restrictive capacity modifier next month and records a
configured terminal transition. It contains no agent-kind or occupation branch.
These controls use fixed priority and inventory forecasts. The A5 experiment
below reuses the condition evaluator for consequence-aware decisions. Parameter
sensitivity remains follow-up work.

Checks include recovery, missing provision despite available inventory, finite
resupply, inactive requirements, modifier composition, terminal cleanup, invalid
rules/records, CPU/reference parity, replay, batching and boundary continuation.
Findings and abstract parameter values are in [RESULTS](RESULTS.md).

## A5. Consequence-aware productive allocation (implemented)

Compare optional bounded forecasts with matched fixed-priority controls. Keep
initial resources, condition rules and phase timing identical within each pair.

- Repeated harvests: can forecasted food harm outrank unnecessary fuel buffers?
- Severe cold at a harvest boundary: can imminent warmth harm reverse that choice
  despite food-first rank? Record both immediate survival and lost seed viability.
- Temporary upkeep shortage: does the agent preserve the known finite resupply and
  forecast gradual recovery without starting duplicate supply processes?
- Exhausted inputs: do all candidates report terminal harm when no production can
  supply either need? Check that optimistic promises never replace actual provision.

The implementation compares continuation-first and each-need-first rollouts, with
and without deferring new work now. It commits only current productive work and
replans next month. Scores prioritize terminal outcomes, impairment, normalized
deprivation, then buffers and work. This is bounded policy comparison, not general
optimization. See [the design](DESIGN.md) for observation and scoring contracts.

Record all alternatives, requested/completed first work, predicted monthly state,
selected policy and actual outcomes. Validate current-month prediction agreement,
future-shock ignorance, seed accounting, CPU/reference and replay agreement,
monthly/batched/checkpoint continuation, and catalog-order invariance. Results
must include delayed failure rather than treating immediate survival as a permanent
solution. The cold fixture intentionally tests this distinction.

## A6. Durable-tool barter (implemented)

Before adding another autonomous participant, give the passive state one tool and
a fixed offer for three grain. A generic stage technique reduces harvest labor
from two to one. The tool lasts six completed uses; other stages do not use it.

Compare identical four-grain, cold-impaired openings with and without the offer.
Also test an affordable purchase that jeopardizes pre-harvest food, absent stock,
exhausted equipment and a tool with one remaining use. Record ownership/payment
at Acquire, dated productive grants, completed use, wear, manual fallback and both
need consequences over sixty months. Keep the policy and work timing unchanged
between paired controls except for the acquisition settlement boundary.

Validate atomic failed purchases, duplicate buyers, shared equipment contention,
idle/rejected work without wear, CPU/reference equality, replay and checkpoints
between purchase and execution. Compare short versus longer forecasts with ample
opening food to expose investment value beyond the horizon. This probe does not
implement a general market or autonomous state policy; see [RESULTS](RESULTS.md).

## A7. Experience alongside tools (implemented)

Reuse the generic stage-technique mechanism, allowing a competency requirement
in place of equipment. Award one cultivation point for a completed harvest and
unlock a one-labor manual technique at four points. Compare manual learning,
tool-assisted learning and a no-seed control over sixty months.

Observe threshold crossing, actual labor costs, food/warmth consequences and
remaining tool life. Verify that the fourth harvest still uses its opening
technique, later equal-cost choices preserve equipment, partial/failed work earns
nothing, and forecasts include future practice gains. Check CPU/reference equality,
ledger replay, monthly continuation, threshold-boundary checkpoints and reordered
technique storage. Reject ineligible technique claims and overflowing competency
updates without partial publication.

This is learning through productive work. A standalone training planner,
forgetting and knowledge transfer remain proposals, not planned commitments.

## A8. Annual access agreements (implemented)

Start with accepted plot access activated in month 1 for one grain annually.
First payment is due in month 13, irrespective of completed or failed harvests.
Compare the same opening resources and forecast policy with free access.
Measure annual payments, arrears, blocked starts, continuing crop completion,
need shortfalls and survival; collecting rent is not itself a success criterion.

A separate due-month fixture opens without grain but with a harvest due. Verify
that it records arrears before productive planning, allows the committed crop to
finish, and clears arrears from output before consumption. Clearing debt must
not reopen productive planning that month.

Check partial payment, oldest-first arrears settlement, duplicate/forged/missing
settlements, right expiry, CPU/reference equality, forecast visibility, replay,
monthly batching and continuation across every month-13 barrier. A9 adds offer acceptance. A10 adds protected payment allocation; negotiation remains unimplemented.
See [RESULTS](RESULTS.md) for the observed food cost of the current priority.

## A9. Access acceptance and forecast horizon (implemented)

Make the plot right dormant until an agent accepts its fixed annual terms.
Compare accept-now with no acquisition using the existing strategy portfolio.
Check useful access, an unusable offer without seed, and competing eight-grain
and one-grain offers for the same plot.

Hold the process candidate horizon and inventory target at six months while
comparing six- and eighteen-month decision forecasts. The longer evaluation
includes the first payment and following harvest. Retain all candidate scores:
a stable-ID tie before any bill is visible must not be mistaken for an economic
preference for the more expensive contract.

Run sixty months after selection, recording food consequences, payments, arrears
and survival. Test acceptance dating, dormant rights, incompatible/duplicate and
out-of-phase grants, first-month forecast agreement, CPU/reference equality,
replay, acceptance-boundary continuation, monthly batching and storage-order
invariance. No negotiation or later cancellation is implied.

## A10. Competing payments and essential needs (implemented)

Compare DebtFirst with ProtectEssentials against identical agreements, balances,
capacities, need rules, forecast settings and process catalogs. Only allocation
policy changes; Due and ClearArrears stay in their existing positions.

Use repeated harvests as the ordinary pair. Use a second pair beginning at the
first due boundary with one grain, one seed and no active crop to expose the
arrears/access trap. Measure food and warmth deficits, survival, creditor receipts,
active months in arrears, blocked new work and completed harvests. Report continuing
post-terminal debt separately. Delaying failure does not establish recovery.

Controls cover abundant resources, zero or fulfilled needs, absent/disabled
consumption recipes, shared consumption stock and integer lot rounding. Compare
opening allocations directly; test forged receipt rejection, harvest-funded cure,
CPU/reference equality, first-month forecast agreement, replay, monthly batching,
reordered catalogs and checkpoints around both payment boundaries.

Protection uses actual stock for this month's consequence-bearing requirements.
It does not escrow stock against other spenders or guarantee future planting
inputs. Keep the protective policy opt-in; the observed results are mixed.

## A11. Combined production and surplus audit (completed)

Compare manual production, a tool offer, experience and both under each payment
policy, holding initial resources and eighteen-month decision forecasts fixed.
Record crop dates, idle-plot receipts, crop/fuel/unused labor, grain accounting,
arrears and tool use. Separate willingness to buy from equipment efficacy with
an already-owned-tool diagnostic.

Use dated starts and a work calendar as feasibility witnesses, clearly labeling
endowment, planning-window or allocation changes. Execute actual monthly work
rather than inferring capacity from aggregate labor. Findings and reproducible
source are in [PRODUCTION-AUDIT](PRODUCTION-AUDIT.md). The follow-up is implemented and measured in [DATED-CANDIDATES](DATED-CANDIDATES.md):
accepted dated payments and replenishment lead time now inform candidates;
normal food deficits disappear without changing physical resources.

## A12. Alternative food and finite foraging (completed)

Add a less labor-efficient, immediate producer of a distinct edible stock. Test
substitution and shared stock accounting, abundant-food controls, a shortage with
spare labor, and contention with crop stages. Compare six- and eighteen-month
foresight on identical openings; include a lethal-need control and a ready-harvest
boundary. Use finite shared supply and explicit capped Open regeneration.

[FORAGING](FORAGING.md) records the measured CPU outcomes. Immediate fallback can
bridge a shortage, but short foresight can sacrifice the crop and shift failure to
warmth. No new agent class or monthly scheduler was required.

## B. Two complementary producers exchanging

Question: can two differently equipped agents use the same decision/action model
for production, selling and buying?

Give one agent a food process and another a tool process, finite inputs, food needs
and different inventory targets. Seed the buyer with existing currency and the
seller with a tradeable surplus so the first exchange does not depend on an
unfunded future sale. Define a useful tool effect explicitly if studying gains from
specialization; otherwise measure exchange and target fulfillment only.

Start with fixed posted prices and integral lots. Compare no exchange, mutually
acceptable prices and non-overlapping reservation prices. Relabel the agents while
keeping IDs and economic state fixed: outcomes should not depend on occupation
names. Swap capabilities and resources to check that available behavior follows
the inputs rather than hard-coded party classes.

Check both trade legs, currency conservation, stage visibility, refused trades,
consumption and each party's inventory target. Do not require exchange to improve
every agent or impose an arbitrary successful-trade count.

## C. Several agents competing for scarce supply

Question: does competition have an explicit explanation rather than depend on
which agent happens to be stored first?

Start with two buyers claiming the same seller stock, then add several buyers
with heterogeneous needs and cash. Use identical opening proposals to compare
allocation policies, retaining the same monthly stage timing. Include duplicate
or alternative offers on the same goods, insufficient buyer funds and an indivisible trade whose
minimum lot cannot be filled.

Include orders in different commodity partitions sharing one buyer's cash. Reject
or downscale gross overspending before emitting transactions, even when hypothetical
same-round sale proceeds would make the buyer's final net balance nonnegative.

Permutation of record storage order should preserve outcomes with fixed IDs and
seed. Changing the configured priority or tie-break seed may change who wins;
report that as policy behavior. Measure unmet needs by agent, waiting time, unused
stock/cash and allocation-to-completion gaps. Conservation alone is not evidence
that a distribution policy is desirable.

## D. Adaptation through information and policy

Question: can agents change behavior through the common decision interface without
introducing a new subsystem for each economic role?

After establishing fixed-price behavior, compare it with one bounded rule that
adjusts future offers from observed stock and completed trades. Hold initial state
and shocks fixed. Test an input shortage, delayed price observation and absent
counterparties. Use only information actually delivered to the agent.

Measure whether adjustment reduces unmet demand or merely increases prices,
oscillation or exclusion. Record who gains and loses. An omniscient diagnostic
can describe the true shortage but must not become policy input accidentally.
Freeze tuning settings before evaluating additional seeds and initial endowments.

## E. Organization and delegation

Question: is an organization usefully represented as an agent, and can it coordinate
people without duplicating their work, assets or needs?

Compare the independent producers with a workshop that owns inputs and cash and
acquires workers' actual time. Give a representative bounded authority over that
account. Preserve total initial inventories, currency and human work capacity;
the organization itself receives no free work capacity or food requirement.

Controls: exhausted workers, absent representative, unauthorized spending and two
representatives attempting to spend the same budget. A transfer between a worker
and the organization must appear once in each relevant ledger. Compare a separate
organizational decision-maker with a simple delegated account; keep the simpler
representation if the extra agent adds no useful decisions.

## F. Optional contract extension

Only after production and exchange work, test a small cash loan through the same
agent IDs and policy interface. Compare allowed pairings with policy restrictions,
and a productive timing gap with an inability to repay. Disabling origination
must not erase existing obligations. This would assess whether credit composes
with the agent model; it would not implement the full credit proposal at once.

## Common evidence and decision criteria

The current transaction-pipeline tests compare grouped effects with a Rust reference
and raw-effect audit, reject duplicate batches, reject invalid process effects and
gross overspending without publishing any leg, and detect insufficient effect-buffer
capacity before publication. Generic terminal lifecycle checks now run; full market trade and typed relationship
checks remain for their scenarios. The add-one smoke test is retained separately.

Every implemented scenario should have analytical or controlled checks for its
accounting and timing. Compare repeated-seed execution, different month batch sizes,
checkpoint continuation across stage barriers and observation enabled/disabled.
For stochastic choices, run multiple seeds; also vary initial resources and
policies so seed variation is not mistaken for broad model coverage.

Retain a per-run summary of completion/validation status, resources, requested and
completed activity, unmet needs by agent, refusal reasons and runtime. Put raw
traces, manifests, checkpoints and binaries in ignored `output/economics/`; keep
curated findings here as Markdown. Implemented findings are recorded in RESULTS.md.

The model earns its genericity if another role can use existing agent, action and
settlement interfaces by adding capabilities, objectives or a policy. Specialized
process and contract execution remains legitimate. A growing switch on agent kinds,
duplicated balance ownership or a framework requiring many unused features is
evidence to simplify the design before adding more scenarios.
