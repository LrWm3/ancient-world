# Agent-centered economics: processes and transaction-first execution

Status: architecture proposal with a first implemented CPU slice. Generic agents,
generic participant components, rights, staged processes, bounded planning and transactional
settlement now run in the single-harvest, repeated-harvest and competing-warmth
scenarios described in [the results](RESULTS.md). Generic requirement consequences
also run for people and a separate institution fixture.
Finite durable-tool barter, substitutable food, shared-pool foraging, fixed storage,
collection-linked token issuance, posted grain purchases and catalog-driven craft,
extraction, livestock and housing activities are implemented; general markets and the broader monthly pipeline remain proposals,
not planned commitments.

The base implementation uses four barriers: Open, productive processes (with planning
and reservation), consumption, and Close. Equipment-offer and stock-bid worlds add an Acquire
barrier after Open. Agreement worlds also settle annual dues before acquisition
and arrears after production; other market stages remain empty. It
supports deterministic two-link planning, abort-on-failure processes and in-memory
checkpoints and an optional bounded consequence-aware policy; the richer
policies below are extensions unless noted otherwise.

The economic model is built from **needs, resources, rights, capabilities and
processes**. Processes describe available transformations; agents select them;
transactions record their actual execution. There is no `Farmer` type, `farm()`
method or farming-specific monthly phase. Farming is a process choice.

The central computational pattern is:

```text
finalized state -> needs / process candidates / plans -> intents and orders
                -> matching and reservation -> validated transactions
                -> grouped effects -> reduction -> committed state
```

Agents make decisions from immutable observations. They do not call one another
to change balances or inventories. Resolution stages coordinate their intentions,
and settlement applies the resulting effects at explicit visibility barriers.
The same pattern supports production, consumption and lifecycle changes even when
no market or counterparty is involved.

## Implemented durable equipment and finite barter

Equipment is a separate mutable asset table: identity, owner, kind, remaining
uses, last-use month and optional plot attachment. Ownership is independent of the
plot use right. Kind specifications can also apply monthly condition decay. A generic
stage technique associates a process definition/stage with an equipment kind,
wear per use and an alternate service requirement. Techniques can instead require
a competency threshold, or require both equipment and competency. Inputs, outputs and
stage timing remain those of the process. The tested technique changes harvest
labor from two to one while preserving eight grain and one seed.

A posted offer identifies a seller, a specific asset and a positive stock price.
There is no state/person type check: any eligible agent can be either party.
The fixture's passive state offers its one tool for three grain. Acquire validates
opening food, ownership, seller/buyer lifecycle and offer availability, then commits
both food legs, ownership and offer exhaustion together. Incoming receipts cannot
fund simultaneous outgoing spending. Duplicate sales and jointly unaffordable
purchases reject the entire batch without publishing partial changes.

The forecast compares each available purchase with retaining food and deferring
new production, using the same condition consequences as ordinary work. It commits
the chosen acquisition and stores the exact dated next Productive batch. Execution
must match that plan; it cannot spend the remaining capacity on a newly recomputed
alternative after purchasing. Acquisition, production and consumption each expose
outputs at their own settlement boundary.

Reservation checks both the continuing plot occupancy and an optional tool for
this month's stage. Eligible variants prefer lower total service requirements,
with stable ties, preferring no equipment at equal cost. A tool is exclusive to one completed use per month. Idle,
rejected and aborted work cause no wear. A last valid use completes normally and
reduces remaining uses to zero; later work falls back to the manual variant.
Transactions record the technique and asset as well as actual labor and outputs.

The original tool fixtures use wear per action, not accounting depreciation.
The activity extension adds process-driven tool creation and capped repair, generic
mandatory productive assets alongside the optional technique, and configurable
monthly condition decay (used for herd upkeep). Houses attach to plots and provide
expiring shelter service through the same process/consumption path. One optional
labor-saving tool and one mandatory productive asset can support an action; arbitrary
equipment bundles, negotiation and a resale planner remain unimplemented.
The state has finite stock but no autonomous selling policy. Forecasts assume no
later purchases, then reconsider next month; they do not value unused equipment
at the horizon or optimize saving wear for more valuable future work. A six-month
forecast can decline a useful tool when ample food hides eventual seed loss.
These limitations are tested and reported rather than assuming all investments
are recognized. See [the tool results](RESULTS.md#durable-tool-barter-on-cpu).

## Implemented activity targets, durable outcomes and housing

The [specialized activity catalog](ACTIVITIES.md) configures four people with work
priorities rather than occupation classes. Requests for stock targets, asset creation
and condition maintenance compete with need-driven work in Productive. All six
extraction activities draw on finite deposits; refining metal is distinct from
mining and from currency issuance. Tool recipes consume materials and labor before
creating durable assets. Repair consumes its declared inputs/labor and cannot
restore more than the configured lifetime. Process records supply deterministic
asset identities and replay provenance.

Open expires service tickets and applies configured asset condition decay once per
month. Productive reserves current inputs, labor, site access, optional tools and
mandatory assets before executing outcomes. Newly created or repaired equipment
cannot be used again in that month. Consumption uses shelter tickets just like
other need satisfiers; Close applies the existing generic need consequences.

Site permission is separate from exclusive work-site occupancy. The `shared_sites`
policy allows housing use to coexist with farming on the same plot, while the house
itself remains an exclusive monthly asset. Construction still reserves exclusive
site work. Homes retain their plot attachment and require current access; portable
equipment offers reject attached assets. Food, warmth and shelter remain distinct
requirements. Workshop recipes remain TBC.

The 36-month fixture is provisioned and uses fixed priorities. It demonstrates
activation, wear, upkeep, construction and annual output/coin taxes, not endogenous
specialization or a sustainable market economy.

## Implemented practice and competency

Experience is an agent-attached competency account, not a transferable asset.
Catalog practice rules identify a process definition, stage, competency kind and
positive number of points earned on completing that stage. This keeps learning
independent of agent roles and of who owns the output. Awards go to the operator.

The validated process transition is the immutable provenance for an award.
Settlement derives awards from those records using the fixed run catalog and
gathers them into competency accounts with checked integer addition. Partial
stage progress, requests, rejection, idle time and aborts award nothing. A
multi-month stage awards once when it completes, not once for each month worked.
Replay reproduces practice from the same catalog; it does not require a separate
unvalidated practice instruction.

Technique eligibility always reads the opening committed competency state.
Practice earned by any process in a batch cannot unlock techniques for another
process in that batch. Publication makes it available at the next planning
boundary. Forecast copies contain competencies and use the same award and
eligibility rules, including across the acquisition-to-production dated plan.

The cultivation fixture awards one point per completed harvest; four points
unlock a manual technique requiring one labor rather than two. The fourth
harvest still pays its original cost. Tool assistance also earns practice. On
later harvests, equal-cost experienced manual work preserves tool life.
Techniques are complete alternatives: labor discounts do not stack implicitly.

These abstract thresholds test gradual accumulation with a discrete productivity
step, not calibrated learning curves. There is no forgetting, instruction,
transfer, practice from failure, fractional proficiency or autonomous training
objective. Forecasts account for learning from candidate work, but the current
candidate generator does not invent training projects solely for future skill.
Skills are not granted merely because time passes. Missing seed or rights can
prevent all relevant practice. See [experience results](RESULTS.md#experience-on-cpu).

## Implemented annual access commitments

An accepted agreement references a use right, creditor, debtor, activation month
and stock-denominated annual payment. The first fixtures start with acceptance
already recorded in the initial world. The offer fixtures below add dynamic
acceptance; negotiation remains unimplemented. Access activates at month 1, and one grain becomes due at months
13, 25, 37 and so on. Payment is independent of harvesting or crop success.

Rights remain separate from obligations. Each annual obligation is keyed by
(agreement ID, due month) and retains original amount owed and cumulative amount
settled, plus native commodity units actually received. Configured contracts may
settle remaining whole units in coins at a fixed quote after native payment. Only
native collection can authorize linked issuance; coin payments transfer existing
currency. Unpaid remainder is arrears. Recurrence creates each dated obligation once,
including overdue anniversaries in a resumed initial boundary. Expired rights
stop new billing, but previously due debts remain. There is no proration or
early termination command.

Worlds with agreements insert Due after Open and before acquisition/productive
planning. Due creates installments and settles from opening stock. A second
ClearArrears barrier follows Productive and precedes Consumption, allowing
committed harvest output to clear unpaid amounts. There is no second production
round: restored eligibility cannot backdate planting into completed work.
Worlds without agreements retain their existing timing.

New process starts using a linked right require no unpaid obligations under its
agreement. Existing processes keep their dated right/plot reservation and may
finish despite arrears, subject to ordinary resource and right-expiry checks.
The resolver enforces this restriction and settlement rejects forged new starts.
Payment is automatic, including partial payment, with oldest due first and
agreement ID breaking ties. Each debtor spends only opening resources; incoming
payments cannot finance another payment at the same boundary. DebtFirst gives obligations precedence over food consumption. The opt-in
ProtectEssentials policy below limits those payments while preserving the same
due dates and oldest-first ordering. There is no discretionary choice to default.

The ledger stores the dated obligation settlement and its exact transfer
transactions. Commit recomputes both, rejects omissions or alterations, gathers
stock effects through the existing CPU kernel and publishes balances/debt
together. Forecasts use these same phases and include obligations in report rows;
annual payments beyond the short horizon remain invisible to the current score.

Scope is one accepted agreement per linked right, fixed annual installments and
fixed parties, with payment to any valid creditor that owns the asset. Terminal
debtors retain debt and may accrue installments until right expiry, but automatic
collection stops; estate resolution is not modeled. No cancellation, debt
forgiveness, penalties, interest, bankruptcy or general enforcement machinery is
implied. See [annual results](RESULTS.md#annual-access-commitments-on-cpu).

## Implemented access-offer acceptance and horizon comparison

Access offers use the same annual terms as accepted agreements but remain dormant
catalog entries. Their referenced rights cannot authorize process work until an
Acquire batch accepts the offer. Acceptance publishes the agreement, with its
actual activation month, in state; the dormant right then becomes usable. The
first bill is twelve months after that actual month, not the catalog opening
date. Immutable catalog terms and accepted state records support replay.

Acquire compares no acquisition with each available equipment or access offer.
An access candidate must name this participant as debtor, have an active owner,
be within its availability window and not conflict with an accepted agreement or
active process on that plot. Acceptance rejects duplicate or incompatible grants.
The current portfolio accepts at most one offer per month; it does not explore
joint equipment-and-land purchases or negotiate prices. Declining is a no-op and
offers can be reconsidered next month. Accepted agreements cannot be cancelled
or switched merely because another offer later looks better.

Each candidate first settles acceptance in its private forecast, then resolves
current productive work using the newly available right. The chosen real Acquire
batch retains the exact dated productive plan, as equipment acquisition already
does. Authority validation rejects processes using unaccepted rights, including
in checkpoints. Forecast report rows include accepted agreements and resulting
obligations; acceptance by itself neither transfers grain nor creates a due debt.

World.decision_horizon optionally overrides rollout length without changing the
configured two-link buffer horizon or inventory-buffer score target. Dated claims
can extend the candidate window as described below. This isolates
foresight from changes in candidate generation. Forecasts assume no later
acquisitions inside a rollout, but real decisions re-evaluate offers monthly.

The controlled comparison offers identical plot access at annual prices of
eight and one grain. Within six months both score equally: neither first payment
is visible. Stable IDs pick the expensive offer. Eighteen months include the first
bill in month 13 and production around that deadline, and select the cheaper
offer. This deliberately exposes a tie caused by truncated foresight, not a
finding that expensive contracts are intrinsically attractive. Reversing catalog
storage leaves the result unchanged; changing IDs in the short-horizon tie could
change it. Longer forecasts remain bounded and are not a proof of affordability
throughout the agreement. See [offer results](RESULTS.md#access-offer-decisions-on-cpu).

## Implemented payment allocation policies

World.payment_policy separates payment allocation from agreement terms and phase
timing. DebtFirst remains the default. ProtectEssentials withholds current-period
consumption stock from the payment budget at both Due and ClearArrears; remaining
stock pays obligations in the existing oldest-due-first order. No obligation is
forgiven, rescheduled or made contingent on a harvest.

Here an essential requirement means a positive participant requirement with a
configured deprivation consequence. For each such need, use the same substitution allocation as the consumption
resolver, subtract already committed fulfillment, and reserve actual whole lots
across alternative stocks. A sole recipe retains partial-stock protection if its
whole lot is unaffordable. Shared inputs draw from one budget in need-priority order.
Protection is capped by stock actually present. Nutrition protects grain and
warmth protects fuel in the fixtures; neither resource nor agent type is hardcoded.
Zero demand, already satisfied needs, disabled/missing recipes and terminal
participants do not create protection for that need. Multiple essential needs
sharing an input sum their demands before capping against the one stock pool.

Protection is recomputed from each boundary's actual state. It is a limit on
obligation payments, not escrow: it does not reserve future harvests or prohibit
productive work/equipment acquisitions from spending the same stock in later
phases. It also does not guarantee that an indivisible consumption lot can be
completed when available stock falls short. It protects only this month's direct
consumption inputs, not seed, annual buffers or future needs. Broader joint
resource reservation remains outside this experiment.

Settlement receipts retain the policy, per-account protected stock, updated
obligations and actual transfers. Commit recomputes these records and rejects
forged policy/protection or payment effects before publication. Both policies
share opening balances, dated obligations and phase order. Forecasts copy the
configured policy and use the same evaluator, including lost planting access
while arrears remain. An UnpaidObligation work receipt distinguishes this block
from missing authority.

The test compares a repeating-harvest opening and a scarce opening with one grain,
one seed and no crop underway when payment falls due. Report food deprivation,
survival, creditor receipts, active months in arrears, blocked work and completed
harvests. Separate active arrears from debt retained after terminal transitions.
Neither policy is presumed better: the tested protective policy shifts shortages
without curing the normal case and delays failure by one month in the scarce case.
See [payment results](RESULTS.md#payment-policy-comparison-on-cpu).

## Generic agents, with persons as the first specialization

An agent is a persistent decision-making identity. Its economic role follows from
capabilities, resources, objectives and relationships. A farmer and a trader should
not require separate update loops merely because their labels differ. Type tags
select relevant component tables and policy families, not permission to bypass
common accounting.

Keep the common agent record small: stable `AgentId`, kind, lifecycle state,
policy identifier and references to its parameters and component rows. Expectations,
preferences and memory can occupy separate columns or tables rather than nested
objects. The implemented `Participant` component supplies a list of `Requirement`
records and one capacity source to any agent. Person and institution fixtures use
the same component, planner, processes and condition evaluator; there is no
agent-kind branch for biological versus organizational consequences. Biological
parameters are scenario data, not defaults inherited by all agents.

A firm or household could later use the same agent interface with different
components. An organization gets no free human work capacity or biological food
requirement. If persons belong to a household, define whether consumption is
budgeted by the household or individuals; membership must not duplicate needs,
inventory or work. Whether an organization needs its own decision policy rather
than just a delegated account remains an experiment.

Assets, process definitions, markets and contracts are records, not automatically agents.
An agent may control them without each object needing goals or a policy.

## Data-oriented state and authoritative ownership

Use typed tables keyed by stable IDs, with structure-of-arrays layouts where useful
for batched computation. CPU-side indexes map IDs to compact rows; row positions
are not persistent identities. Compacting a table must not change who owns an asset
or consume a different behavioral random stream.

| Table | Responsibility |
| --- | --- |
| Agents | Identity, kind, lifecycle, policy and component references |
| Participants | Generic recurring requirements and capacity source (implemented) |
| Condition rules and state | Requirement-specific deprivation, recovery, capability effects and terminal transitions (implemented) |
| Person-specific components | Future biological attributes beyond generic condition dynamics |
| Accounts and inventories | Authoritative balances by owner, resource and unit |
| Assets | Ownership, location, quantity/condition and valuation references |
| Needs | Beneficiary, service, desired amount, priority, deadline and fulfillment |
| Capabilities | Agent, method/service, available capacity and cost information |
| Rights | Holder, asset/service scope, issuer, validity interval, transfer/revocation terms |
| Process definitions | Typed inputs, services, conditions, stages, outputs and failure rules |
| Process instances | Operator, beneficiaries, bound assets, stage, progress and dated commitments |
| Environment | Actual conditions and separately delivered observations |
| Relationships and mandates | Membership, employment, ownership and bounded authority |
| Contracts and obligations | Parties, terms, outstanding claims, schedules and due amounts |
| Observations and beliefs | Source, observed/received boundary, visibility and derived expectations |
| Orders and reservations | Desired actions, limits and resource commitments |
| Transactions and effects | Validated changes, their provenance and settlement status |

Cash belongs to an authoritative account. A fast agent-cash column may be that
account's storage or a derived view, but cannot be an independently mutable copy.
Likewise, asset records and inventory views must not count the same goods twice.
Available stock is owned stock less reservations; a reservation is not new stock.
Valuation, equity holdings and receivables are not spendable cash.

Separate ownership from authority. A manager may spend a firm's account only
within a mandate; multiple managers share the same budget. Closing an agent must
leave retained claims and assets with a defined estate or successor. Its ID cannot
be reused for a new unrelated agent.

## Stocks, capacities, assets and rights

Stocks are stored quantities such as grain, seed and currency. Capacities are
period-bounded services such as person-hours, machine-hours or hectare-months.
Unused monthly labor expires; it does not accumulate into next month's allowance.
Stock accounting does not imply physical storage: grain needs space, tokens do not.
The implemented `Storage` catalog gives resources space weights and agents shared
capacity limits. Productive allocation accounts for space freed by inputs and used
by accepted outputs; settlement rejects overflow atomically. Capacity is checked
at actual completion, not reserved for every future month of a process. Storage
construction, spoilage and overflow handling remain extensions. See
[storage and currency](STORAGE-CURRENCY.md) for the covered-store fixture and controls.
Assets persist and supply capacities subject to condition and occupancy. Rights
permit a holder to use a specified asset or service for a stated purpose and period.

Ownership, permission, reservation and actual use are separate. A use right is
necessary but not sufficient to occupy a plot: the resolver must also check its
available capacity and competing reservations. Monthly regeneration renews service
capacity without erasing continuing occupancy. Labor acquired from another agent
and that person's own work draw on the same underlying time allowance.

For the first scenario, a state agent owns one plot and grants a person its use.
The state can be a passive owner with a fixed initialization policy. This is a
scenario choice, not a hard-coded ownership rule. Later leases or communal tenure
can change rights allocation without changing the growing process. Rights specify
whether crops already in progress survive expiry/revocation; the first scenario
requires the right to cover the entire process and rejects shorter grants.

## Process definitions and instances

A process definition describes a transformation:

```text
consumed stocks + required services + conditions + elapsed time -> typed outcomes
```

Definitions identify input units, minimum useful scale, compatible assets, required
rights, execution barrier, stages and allowed transitions. Each stage declares its
input consumption point, service requirements, duration, environmental conditions,
output rule and response to unmet requirements. Recipes are simple process
definitions; a multi-month growing process uses the same interface as a short
manufacturing process. Typed domain functions may compute yields or wear. The
initial catalog is not an arbitrary scripting language or universal optimizer.

A process instance records its definition/version, operator, input owners, output
beneficiaries, attached assets, start boundary, current stage, progress, realized
conditions and reservation references. Operator and asset owner need not coincide;
a land-use right alone does not determine who owns the harvest. Starting an instance
validates authority over every input and records output ownership explicitly.

Consumption is a process too: owned grain is consumed and a typed fulfillment
effect credits a named person's nutrition need for the current month. Fulfillment
is not tradable inventory or a permanent stock. Need-period IDs prevent this
month's satisfaction from satisfying next month's requirement. Persistent condition
consequences are separate typed effects at Close, described below.

A growing instance might move through planting, growth and harvest. Starting it
reserves plot occupancy for its full interval, consumes seed at planting execution,
and consumes only the labor actually provided at each stage. Future labor is a
dated forecast unless a firm commitment explicitly reserves it. Forecast yield
never enters available inventory. Output is credited only by completed execution.

Each stage declares whether insufficient service pauses progress, reduces output
or fails the instance. Cancellation/failure records sunk inputs, recoverable stock,
any salvage and released reservations; previously consumed seed cannot simply be
refunded. Specify a maximum pause/deadline so failed instances cannot occupy assets
forever. Right loss is resolved at a stated boundary before further use, using the
scenario's termination or grandfathering rule.

## Bounded planning over process chains

Known processes form a transformation graph with joint inputs and possibly joint
outputs. Agents can reason backward from nutrition to grain consumption, then from
missing grain to growing. They may use only known definitions and delivered
observations; actual future weather is not policy input.

Start with a bounded greedy planner: forecast deficits by due month, search a small
fixed-depth set of known process chains, check stocks/rights/current services and
future requirements, then rank candidates by expected reduction of weighted unmet
needs before their deadlines. Specify the horizon, search limit and stable tie
break. Prevent repeated cyclic expansion and count shared inputs and joint outputs
once across selected candidates. Recompute residual budgets after each selection.
Costs with unlike units require explicit normalization if added to the score.

A harvest six months away cannot eliminate a shortage next month. Keep interim
shortfalls visible, preserve seed or other committed inputs from consumption, and
account for outputs of already active instances when planning new starts. Distinguish
firmly feasible current work from a plan depending on unreserved future labor or
uncertain output. The first deterministic scenario needs only a two-process chain;
branching, stochastic expectations and learning come later.

Plans emit `StartProcess`, `SupplyProcessStage`, `Consume` or `CancelProcess`
intents with resource requests. Existing and new processes compete for the same
finite services under an explicit policy; continuing work does not silently win
because it was visited first. Minimum useful grants apply to indivisible work.
An 80-hour request cannot execute on 30 hours unless that process permits scaling.

Missing inputs can produce acquisition requests through the existing market
protocol. With no market enabled, record an unavailable-source shortfall rather
than inventing resources. A later labor or seed market can fulfill the same
requirements without changing the process definition.

## Needs, plans, orders and outcomes are different records

An initial person policy can prioritize essential consumption, a desired buffer,
then discretionary production and exchange. It considers only delivered information
and declared capabilities. Other policies can be substituted without changing
settlement. No opportunity, deliberate saving, lack of authority and unaffordable
need are different reasons for doing nothing.

The common interface is conceptually:

```text
observe(agent, visible committed state) -> AgentView
expect(AgentView, memory) -> Expectations
identify_needs(AgentView, Expectations) -> Needs
candidates(Needs, known processes, bounded horizon) -> Process chains
plan(AgentView, Process chains, active instances, Expectations) -> Plans
emit(Plans, current authorized budgets) -> Intents / Orders
resolve(Orders, resource pools, market rules) -> Match proposals / Reservations
validate_and_emit(feasible matches or operations) -> Transaction batch
reduce_and_commit(Transaction batch) -> next stage state / Outcomes
respond(delivered Outcomes, previous memory) -> recorded belief / policy updates
```

These are responsibilities, not mandatory class hierarchies. A one-person scenario
can use a few ordinary functions and typed arrays with empty market stages.

A need is not a purchase order. Plans can contain unfunded wishes; funded orders
must respect actual resources and financing already settled. An order identifies
its actor, authority, month/stage, market/resource, quantity, price/rate bounds,
priority, expiry and minimum useful fill. Mutually exclusive alternatives share a
choice-group ID. Requested, affordable, reserved, matched, delivered and consumed
quantities remain distinct.

For example, demand for 100 units and a delivery of 72 leaves 28 units of unfilled
purchase demand. A delivery alone does not establish consumption or need
satisfaction: later consumption may use existing stock, or acquired goods may be
reserved, stored or lost. Food deficit is measured against actual eligible
consumption, not automatically against purchase quantity.

## Implemented generic requirement consequences

A requirement and a condition are distinct. A `Requirement` requests a fixed
monthly quantity of a fulfillment resource; its priority remains a decision-policy
input. An optional `ConditionRule`, keyed by subject and provision resource,
binds that requirement to persistent deprivation and its consequences. Rules do
not inspect agent type. Food, warmth and institution upkeep use the same path.
The component's `needs` vector now stores generic `Requirement` records.

| Record | Implemented role |
| --- | --- |
| `Participant` | Any agent's recurring requirements and one capacity source |
| `Provision` | Month, subject, resource, actual quantity and completed source-process ID |
| `ConditionRule` | Shortfall cost, recovery, impairment threshold, retained capacity, affected capability and terminal threshold/name |
| `Condition` | Deprivation points and consecutive months with positive deprivation |
| `ConditionChange` | Required and supplied quantities, before/after condition and resulting per-condition capacity modifier |
| `TerminalTransition` | Month, subject, triggering requirement and configured terminal state |
| `MaintenanceSettlement` | Close-boundary provisions, condition changes and lifecycle transitions retained in the ledger |

Monthly dynamics are deliberately small and deterministic:

- Shortfall is `max(required - supplied, 0)`; each missing provision unit adds
  the rule's shortfall cost to deprivation.
- A fully supplied positive-demand month removes the configured recovery amount,
  down to zero. Extra provision does not accelerate recovery or bank satisfaction.
- Zero demand neither adds deprivation nor repairs previous deprivation.
- Adverse duration counts consecutive closes with positive deprivation, including
  partial recovery. It resets only at zero; it is diagnostic, not a second penalty.
- At the impairment threshold, the affected capacity receives a configured
  permille modifier. Multiple conditions use the most restrictive modifier,
  rather than multiplying penalties. Integer capacity rounds down.
- At the terminal threshold, the agent enters the named terminal state. All
  future capacity is zero and autonomous planning/consumption stops. Multiple
  simultaneous causes retain their condition changes; the lowest resource ID
  supplies the single transition's primary reason.

Timing uses the existing four barriers. Open expires previous fulfillment and
regenerates capacity using conditions from the preceding Close. Exogenous capacity
overrides supply the base amount and cannot bypass condition penalties. Productive
and consumption execution are unchanged. Close obtains provision receipts from
completed consumption processes for the current month, checks their totals against
fulfillment balances, and evaluates conditions once. Inventory and reservations
are never provision. Commit validates the entire expected maintenance settlement
before publishing balances, condition changes and lifecycle transitions together.
Missing, altered, duplicated or out-of-phase condition settlements are rejected.
Replay and checkpoints include condition/lifecycle state.

A terminal transition cannot undo this month's work. Remaining active processes
abort at the next Productive barrier, retaining sunk inputs and releasing their
occupancy; they cannot generate outputs after the terminal transition. Retained
stock/claims stay with the inactive identity. Estate transfers, succession,
reactivation and general asset-condition subjects are not implemented. Reports
retain the terminal event and frozen condition; demand becomes zero in subsequent
months, so shortage totals must be read alongside survival duration.

The evaluator's pure `advance` function is shared through ordinary settlement by
the optional consequence forecasts described below. Legacy policies still use
fixed need ranks and projected inventory shortfalls; their behavior is unchanged. All rules currently use accumulating deprivation, fixed monthly demand
and one condition per requirement. Buffered depletion, wear, deadlines, multiple
satisfiers, environmental requirements and richer lifecycle rules remain proposals.

The experimental catalog charges two deprivation points per missing unit,
recovers one point per fully supplied month, halves capacity at four points and
uses a terminal threshold of twelve. These shared fixture values demonstrate the
mechanism, not equivalent biology and organizational behavior. Each rule can have
different values. No empirical calibration or acute within-month physiology is
claimed. See the paired controls and institution recovery in [RESULTS](RESULTS.md).

## Implemented bounded consequence forecasts

`Priority::ConsequenceAware` is optional. It selects productive allocation at the
Productive boundary, or at Acquire when equipment offers or stock bids exist;
consumption policy is unchanged. The bounded implementation supports one to four
participants with at most four requirements each, regardless of participant type.
Unsupported scope is rejected explicitly. With multiple participants it chooses a
common priority/defer/producer policy using aggregate forecast consequences; this
is a coordinated group forecast, not independent optimizing agents.

Stock-bid alternatives enumerate subsets of eligible sellers (at most sixteen
subsets for four people), each offering one lot. Joint opening treasury funds and
receiving storage are checked before forecasting. Accepted trades settle together
at Acquire, followed by one dated productive plan for all participants. Productive
requests share the existing pool budget; stable IDs break equal-priority ties.
These rules prevent duplicate spending but do not guarantee fair outcomes. Other
acquisition alternatives still consider one equipment/access offer at a time.
See [four people](FOUR-PEOPLE.md) for shared-capacity controls and limitations.

The candidate generator remains a bounded two-link planner. For each need's
consumed stock it now includes known outgoing obligations: outstanding amounts
count once at the opening boundary and accepted contracts contribute dated annual
installments through right expiry. Already-paid amounts and unaccepted offers do
not add demand; expected incoming payments are not treated as inventory.

For a contract-linked input, the candidate window is the greater of the configured
buffer horizon and the longest enabled matching producer duration plus one month.
The extra month exposes a bill at the opening boundary after a possible completion;
a six-month crop can therefore be proposed in month 7 before rent in month 13.
Uncontracted inputs retain the configured window. This is a bounded shortage
heuristic: it optimistically credits active output and compares needs separately,
not a promise that all stages or payments will succeed. It currently generates
need-driven production through a positive need's consumption chain. Separate
configured work orders now supply stock, durable-asset and maintenance targets;
obligations do not automatically invent these specialization goals. It does not
optimize repayment, resolve future resource conflicts or perform arbitrary graph
search. The ordinary consequence forecast tests actual timing, resources,
payment priority and arrears; the generator only admits candidates.

The allocation layer
compares a portfolio of future allocation strategies: continuing processes first,
and each positive-demand need first in turn (`NeedFirstFor(resource)`). Each is
also evaluated with new autonomous starts deferred in the current month. Deferral
does not cancel existing processes or dated external intents. Without offers there are at most
ten alternatives, in stable order. Up to four posted offers add mutually exclusive
accept-one-now choices (equipment or access) beside no acquisition, for at most
fifty alternatives before alternative productive chains. Substitute-producer
choices expand that bounded portfolio as described below. It does not enumerate all joint actions or
find a globally optimal plan; identical first actions can have different assumed
future strategies, which are retained as distinct alternatives.

Each alternative copies the current committed opening state and simulates the
configured decision horizon, defaulting to the production horizon. Most fixtures
use six months; the access comparison uses six versus eighteen months while
keeping the configured candidate buffer and score target at six. Contract-linked
inputs extend the candidate window to cover production lead time. Rollouts use the ordinary request,
reservation, execution, consumption, Close condition and next-Open capacity code
with the Rust reference gather. Feasibility includes shared current budgets,
rights, exclusive occupancy, stage deadlines, sunk seed, actual future production,
recovery and terminal cleanup. Outputs cannot be spent before their settlement
barrier. Forecasts never mutate the real state or recursively invoke forecasting.

The observation boundary is explicit: current balances, conditions, rights,
process definitions and active process instances are known. Current dated external
intents are known. Future fixture capacity overrides and scheduled starts are
removed from the forecast world; a future shock becomes known only when actual
Open establishes availability. Already active delayed supply processes, such as
the institution's finite resupply, are known commitments, not hidden future events.
There is no probabilistic belief or learning system.

Scores are lexicographically minimized:

1. Terminal participant-months, making survival preferable and later death better
   when every candidate fails within the horizon.
2. Participant-months whose closing condition crosses an impairment threshold.
3. Accumulated deprivation, normalized by each condition's terminal threshold.
4. End-of-horizon inventory coverage gap, capped at the horizon's demand for each
   need's alternative consumption recipes, allocated from one stock budget per
   month and normalized per need. Surplus fulfillment does not carry forward.
5. Total completed productive work; remaining ties use stable alternative order.

Normalization uses fixed-point integer scores. Condition domains are equally
weighted after normalization. This is a declared experimental preference rule,
not a universal utility function. Buffer scoring is only a tie-break after harm;
it assumes the simple one-input consumption chains and does not solve substitution
or shared-stock attribution across different needs. Ongoing output beyond the
forecast boundary receives no terminal value.

Only the selected alternative's first productive batch is committed, with the
configured real backend (CubeCL CPU in the CLI). Next month, the policy replans
from actual observations. `Decision` records retain the forecast interval, all
scores and first-work receipts, selected index, and each alternative's predicted
monthly balances, needs, conditions and lifecycle. The CLI compares the selected
forecast with observed reports. The current month must match in deterministic
fixtures; later months may differ because of replanning or unobserved shocks.
These are trusted diagnostic records, not promises of future grants.

The experiments demonstrate both protecting a harvest over fuel buffers and
sacrificing that harvest to avoid imminent cold death. The latter loses the only
seed and causes later starvation: generic foresight cannot create missing recovery
options. Short horizons can miss productive chains; fixed future rollout policies
can miss better mixed sequences. More agents, uncertainty, broader search,
parameter sensitivity and improved terminal valuation remain future experiments.

## Implemented repeated production and competing needs

The long single-person fixtures consume seed at planting and produce seed alongside
grain at harvest. Seed is a stock, not a reusable capability: failed crops cannot
refund it and anticipated harvest seed cannot fund a same-round start. Rights cover
the entire future occupancy interval, including processes completing after a run's
reporting cutoff.

Participants now carry a list of requirements, each with a fulfillment resource, monthly amount
and priority rank. Requests, process instances and receipts retain the motivating
need. Warmth uses fuel production and consumption definitions, without a warming
method on the person or a dedicated monthly phase. The resolver checks both needs'
requests against the same stock and service budgets; fulfillment remains separate
by resource and month.

The candidate generator proposes one new process per need per participant per month,
deduplicating a producer selected by several needs. It forecasts each chain
separately; global joint planning and future labor commitments are not implemented.
`NeedFirst` ranks requests by need, then continuing work and stable IDs.
`ContinuingFirst` and `NewFirst` rank continuation status before need. Consumption
also uses need rank when inputs compete. Reordering a needs vector cannot change
that policy. Policy comparisons hold opening resources and requests fixed.

The [60-month results](RESULTS.md#warmth-competing-for-the-same-labor) expose a
limitation: a high-priority need's forecast buffer can displace a critical harvest
even when that need is currently well supplied. Food-first is feasible for the
fixture, while warmth-first loses the crop and planting seed. Distinguishing current
deficits, deadline-critical work and buffer targets is now tested by the optional
consequence forecasts above; the legacy policies preserve the original behavior.

## Implemented substitutable provisions and shared extraction

Multiple consumption definitions can satisfy one provision. They allocate against
one finite stock budget, using unencumbered food before payment-earmarked food,
then releasing earmarks if necessary to eat. Whole-lot feasibility, yield per input
unit and stable recipe IDs determine the order. This first efficiency ranking assumes
comparable catalog input units. Equivalent recipes do not duplicate stock coverage.
Buffer scores allocate stocks per month, so expiring surplus nutrition is not stored.

Needs with alternatives use a shared projected consumption budget when comparing
productive chains. The ordinary forecast additionally considers a current-boundary
preference for each substitute producer, ahead of continuing work. This exposes the
choice to interrupt a crop for immediate food. Preferences do not persist into
future rollout months; those retain the selected fixed allocation policy. Four
substitute producers cap the expanded portfolio at 150 alternatives with the
existing need/offer limits. One new autonomous process per need remains the bound.

A Pool is a stock account with finite capacity and fixed monthly regeneration.
A PoolInput maps a productive definition's stock input to that shared account.
Catalog permission currently admits all operators of that definition. Open emits
validated, capped regeneration; Productive reserves shared stock and private labor
jointly in existing request order. Whole feasible actions consume stock and emit
owned output together. Output becomes consumable after the productive settlement.
Stable priority is deterministic but does not promise fair allocation.

The foraging catalog uses two labor and one wild-supply unit to yield one wild-food
unit in the current month. The existing crop yields eight grain for eight labor
across six months. Either stock can produce nutrition. No forager class or new
monthly phase is involved. See [FORAGING](FORAGING.md) for CPU results: foraging can
bridge a shortage with spare labor, but short foresight can abandon the crop and
cause later warmth deficits. Longer foresight changes that choice on identical
resources; immediately lethal hunger can still justify sacrificing the crop.

## Markets share a protocol, not one matching rule

Goods, labor, credit, housing and equipment can use the same outer pipeline:

```text
generate -> partition / group -> match -> reserve -> emit -> reduce -> commit
```

Each market defines eligibility, pricing, settlement units, divisibility, minimum
fills and obligations. A credit match needs underwriting and consent; a labor
match needs available qualified work; a goods trade needs inventory and funding.
Some operations, such as consumption or spoilage, enter at validation/emission
without a buy/sell matcher.

The first goods experiment can use fixed reservation prices and integral lots.
A documented seeded priority over stable order IDs is a possible initial contention
policy. It is deliberately priority-sensitive but independent of collection order.
Proportional allocation is a separate option for divisible claims, not a reason
to change monthly execution order or partially fund an unusable indivisible task.

Partitioning by region or commodity does not make resource pools independent.
One buyer's cash may fund orders in several markets; the same worker may receive
several job matches. Assign bounded budgets to partitions or perform a shared
reservation pass before accepting results. Check joint cash, goods, time, authority
and collateral constraints before committing any leg. Gross outgoing claims must
be affordable: a nonnegative net delta cannot justify spending anticipated receipts
within the same settlement round.

## Transactions, effects and atomic settlement

A resolved match is provisional until its preconditions and all legs are validated.
A transaction names its ID, month/stage, causal order or obligation IDs, participants,
operation, units and actual quantities. Its effects are immutable records once the
batch commits. Failed candidates retain rejection receipts, not successful trades.

A local purchase of 5 food units at 4 currency units produces one transaction:

```text
transaction T: buyer 123, seller 91, food 5, value 20

account effects:
    (123, cash)   -20
    (91, cash)   +20
    (123, food)   +5
    (91, food)    -5
```

Each effect retains `transaction_id`, target table/ID/field, resource/unit and a
stable leg index. Validate both trade legs together. If either is infeasible,
neither transfers. Scheduled delivery uses in-transit custody and a later arrival
transaction; it cannot credit the destination's available inventory at dispatch.

Financial transfers are one operation family. Other families include process input
consumption and output, person consumption, labor reservation/completion, spoilage,
asset wear, interest accrual, claim repayment/write-off, and relationship or agent
creation/retirement. Each family declares its own accounting rules. Production
transforms goods; consumption and destruction are explicit sinks. Claim accrual
creates a receivable and obligation, not physical goods or cash.

Numeric effects are grouped and reduced by target. Structural operations use typed
commands such as `CreateAgent`, `StartProcess`, `AdvanceProcess`, `AddRelationship`
or `TransitionLifecycle`, with
explicit conflict resolution; arbitrary metadata cannot be added numerically.
For additive fields the invariant is:

```text
balance at next boundary = opening balance + sum(committed effects)
```

For the full state, including structural changes and memories:

```text
S[next stage] = apply_validated_batch(S[current stage], committed transactions)
```

Compute candidate balances and structural changes in staging buffers, validate
whole-batch budgets and references, then publish the next state. Atomicity means
no partially visible trade or batch; it does not require one GPU atomic instruction
or promise rollback after an arbitrary device failure. Retain the prior committed
boundary until publication succeeds. Batch IDs and applied markers prevent a retry
or checkpoint reload from settling the same effects twice.

## Gather-based execution on CPU and CubeCL

Logically, transactions scatter effects to many owners. Physically, write effect
records into separate slots rather than letting many threads update the same
owner's balance. Group by `(table, target_id, field, resource)`, reduce each segment,
then assign each destination to one apply operation.

Variable output needs explicit capacity management: count outputs, prefix-sum their
sizes and assign disjoint ranges, or use bounded per-source slots followed by
compaction. An overflowing queue must fail or retry before publication; silently
dropping orders or effects would change the economy. Event IDs should derive from
stable source IDs and local ordinals, not nondeterministic append order.

Prefer scaled integer units for conserved money, goods and time in the initial
model. Define price multiplication, rounding and residual ownership explicitly;
check integer overflow and use a stable rounding-remainder rule. This supports
exact reductions within range. Floating-point forecasts, valuations or production
functions still require specified reduction order and tolerances; sorting alone
does not guarantee bitwise CPU/CUDA equivalence.

Sorting and materializing every leg has a memory/bandwidth cost. Use sparse touched
accounts, deterministic bucketing, batched reductions or local pre-aggregation when
measurement justifies them, retaining transaction provenance. A workload of millions
of events is a benchmark question, not an assumed benefit. Bounded chunking must
preserve whole-round budgets and publication semantics across chunks.

CubeCL currently supplies a CPU integer segmented-gather kernel for settlement;
host code groups effects and validates prefix arithmetic before launching it. The
original add-one smoke test remains. Simulation checks now compare CPU-kernel
settlement with Rust reference execution and independent raw-effect accounting.
Sorting, planning, reservations and structural transitions remain on the host;
market clearing is not implemented. Move additional work to kernels where tested.
Global settlement stages may contain independently processed partitions, rather
than one serial matcher or unrestricted shared-state writes.

## Monthly execution and visibility barriers

One simulation tick now denotes a modeled economic month, with explicit stages
inside it. This replaces the earlier single opening snapshot and next-tick-only
trade/production rules. Each stage reads a committed boundary `S[t, stage]`.
Expectations begin from opening information; later orders can be adjusted using
actual earlier-stage financing or purchases without pretending those outcomes were
known when the initial plan was made.

The following is a baseline timing proposal for this stand-alone model. It does
not reorder or replace Ancient World's [monthly coordinator](../../docs/monthly-schedule.md).
Disabled subsystems contribute no transactions but retain the same visibility
contract for the remaining stages.

| Stage | Reads and decisions | Settlement and visibility |
| --- | --- | --- |
| Open | Prior Close, due arrivals, expiring records and time-dependent rules | Emit and settle arrivals, environmental changes/observations, capacity resets, wear and accrual once. Retain continuing asset occupancy and validate current rights. Due obligations become claims, not payments. No resource-dependent process advancement occurs here. |
| Process requests, expectations, needs and plans | Committed Open state, active instances and observations actually received | Existing instances emit current-stage requests without advancing. Agents forecast dated needs, generate candidate processes and propose starts/consumption plus funding gaps. Future harvests are not stock. |
| Credit | Planned funding gaps, voluntary offers, current liquidity and evidence | Underwrite, reserve, emit actual advances and claims, then commit. Later input orders see actual funding. Rejected credit creates no cash. |
| Labor and production inputs | Current financing, existing/new process requests, worker offers and material orders | Match and reserve joint money, stock, time and asset occupancy under explicit priorities. Commit input purchases and dated commitments; validate rights and minimum useful grants. Initial sellers offer current stock. Unsupported plans downscale or are rejected. |
| Productive process execution | Acquired inputs, valid rights, authorized assets, environment and committed work | Start/advance eligible instances once, including declared failure transitions. Emit actual input use, work, progress, outputs and reservation releases; commit before sales. Wages follow employment terms. |
| Goods and services | Committed production and remaining buyer budgets | Generate or revise consumer/output orders, match, validate and commit transfers. Current output can be sold now; sale proceeds cannot reopen earlier credit/input/production stages. |
| Consumption and service use | Delivered services and actually owned consumable stock | Run consumption processes through the same resolver; atomically commit input use and beneficiary/period fulfillment. Record shortages separately from unfilled orders. Reserved inputs are unavailable for consumption. |
| Obligations | Due wages, debt service, rent, taxes, benefits and declared dividends; current cash | Allocate bounded payments by explicit priority and emit actual transfers, partial payments and missed-payment events. Commit before distress review. |
| Distress | Completed shortfalls, assets and outstanding claims | Compact affected agents into work queues. Bounded refinancing or asset-sale rounds may settle new resources and pay remaining claims; they cannot rerun this month's production or consumption. |
| Entry and exit | Accepted formation/closure decisions and settled estates | Allocate stable IDs, transfer funded capital and create/retire relationships atomically. New agents first decide next month. Unresolved estates remain addressable. |
| Close | All committed stage receipts | Record realized need deficits and expectation/memory updates, apply closing-only effects, validate accounting and record checkpoint/report. Previously settled effects are not applied again in a final gather. |

Existing instances are inspected during planning and advanced only at their declared
execution barrier. Each `(instance, month, stage)` execution has an idempotent receipt;
the baseline allows at most one productive stage per instance per month, including
the planting stage of a new start. A one-month stage completes at that month's
execution barrier; the next stage becomes eligible next month. Environmental
exposure and elapsed-time progress are recorded there once, even for a stage with
no labor requirement. Open does not perform a second advancement.

Instant consumption executes at its own barrier with no additional elapsed month.
Zero-duration transformations still require an authorized barrier and bounded
round; they cannot recursively trigger unlimited process chains. A future timing
variant permitting multiple productive stages per month must specify its rounds
and visibility explicitly.

The ordering has economic consequences. In this baseline, end-of-month wages
cannot fund this month's shopping. Goods-market sale proceeds can pay later
obligations, but cannot fund purchases in the same matching round; absent another
explicit shopping round, they support shopping next month. Credit and input-market
receipts become available only at their declared later barriers.
If a scenario needs wage advances or payroll before shopping, give that payment an
explicit earlier stage and compare the timing as a separate model variant.

Initial orders can be emitted and matched within the same market stage, after all
participants have read that stage's immutable view. Counteroffers or repricing need
a bounded subsequent round; they cannot secretly observe another thread's partial
execution. Within a round, incoming transfers never enlarge its opening allowance.
Across a committed barrier, new funds may support the next authorized round.

Released production time cannot be sold backward into the completed labor market.
Repairs currently compete within Productive. Any additional same-month repair or
production sub-round would need an explicit place in the schedule.
Pending cargo, multi-month projects and employment contracts carry dated reservations
and obligations, not stale one-month grants. Advancing twelve months runs all twelve
monthly sequences, rather than aggregating away intermediate decisions.

## Labor, obligations and distressed-agent queues

A labor match reserves a person's actual time and creates the appropriate work
relationship/commitment. It does not mint a second labor stock at the employer.
Production consumes that commitment and records actual completion. Contract terms
state whether wages pay for reserved attendance, completed work or output; wages
must not be inferred inconsistently from whichever metric a subsystem uses.

An obligation is a scheduled claim, not a successful future transfer. At its due
stage it emits a payment intent. Creditors share the debtor's affordable allowance
under explicit priority, with proportional treatment of equal-ranked divisible
claims and a deterministic rounding rule. Essential protected balances and any
escrow remain explicit. Missed payment, arrears and write-off are distinct events.

Distress can use state transitions and compact queues, for example:

```text
operating -> liquidity shortfall -> credit review -> asset-sale review
          -> recovered / arrears / insolvency review -> liquidation / closure
```

These are conditional transitions, not an inevitable chain. Lack of current cash
does not by itself establish insolvency. A scenario supplies valuation, eligibility,
consent and bounded retry rules. Exhausted same-month rounds leave unresolved cases
for later; they do not loop until some rescue succeeds.

Liquidation records actual proceeds and allocates them among ranked claims.
Payments, collateral recovery and guarantor payments share outstanding-debt limits;
write-offs record only the remaining loss. Creditors receive effects through the
same settlement mechanism, never by direct callbacks mutating their accounts.

## Financial scope and balance-sheet meanings

The storage/currency fixture issues one treasury token for every two grain actually
collected on an annual tax installment. A finite posted bid transfers one existing
token to a person selling one grain to the state. Only collection authorizes issuance;
purchases do not. This is an explicit source event in the ledger, and other phases
must conserve the issued currency. Tokens use no storage and currently have no
spending outlet. Earlier fixtures retain their original accounting rules.

A later ordinary loan
moves existing creditor cash to a debtor and creates corresponding debt/receivable
records. It does not also create a bank deposit. The
[general credit proposal](../../docs/system-generalization-opportunities.md#7-general-credit-contracts)
provides optional contract terms, permissions and negotiation ideas.

Deposit-creating banks would be a distinct model extension: origination creates a
bank loan asset and matching deposit liability, with the borrower holding that
deposit asset and loan liability. Transfers between banks would also need reserve
or clearing rules. Do not mix that mechanism with fixed-supply cash lending or
assert money conservation across an explicitly authorized issuance event.

Interest accrual, principal repayment, interest payment and default loss require
separate legs and metrics. Principal is financing, not production revenue. Derived
net worth depends on the selected valuation basis; marking an asset's value up
cannot create currency or increase physical production.

Firm formation could transfer a founder's cash into a new firm's account and issue
an ownership claim. The founder's equity investment and the firm's contributed
capital represent different sides of the ownership relation. Consolidated wealth
must eliminate internal claims rather than sum them as newly created resources.
Similarly, dividends transfer resources; they do not produce them. Tax, wage and
benefit rules generate obligations whose payment remains bounded by actual funding.

## Ledger, replay and diagnostics

The committed transaction/event ledger is the canonical record of changes between
boundaries. Account tables are the current state; deltas are derived from uniquely
identified transactions, not independently editable records. Include cause IDs,
month, stage, round, participants, quantities, units and the rule/schema version.
Creation, retirement, expectations and memory updates also need replayable typed
effects if full-state replay is claimed.

Replay has two meanings: applying recorded committed transactions reproduces state;
rerunning decisions from a checkpoint must regenerate those transactions. The latter
also requires initial state, catalog/policy versions, pending orders, reservations,
agent observations/memory, active process stages, rights, future occupancy and
deterministic random state. Stochastic process outcomes must use stable instance,
stage and month keys so table compaction does not change realized yields. Neither claim follows
from retaining cash deltas alone. Checkpoint at completed barriers; a mid-stage
interruption discards unpublished work or resumes from explicitly saved stage state.

Store detailed ledger chunks and checkpoints under ignored `output/economics/`.
Keep bounded in-memory indexes and summaries. If a retention policy drops records,
state the earliest replayable checkpoint and remaining diagnostic coverage.

Reports should connect needs -> plans -> orders -> grants -> transactions -> actual
consumption/work, with non-submission, refusal and shortfall reasons. This answers
where cash went, why inventories fell, who absorbed a default, or why an agent
could not act. Economic aggregates need explicit definitions: GDP is not the sum of
all cash transfers, and changes in financial claims are not automatically output.
The [run-health proposal](../../docs/run-health-evaluation.md) supplies complementary
questions about activation and persistent limits.

## Scope and staged validation

Keep the independent Rust crate and CubeCL CPU setup. No dependency on `History`,
the main GPU buffers or world catalogs is required. Reuse lessons from
[participation commitments](../../src/participation.rs),
[explicit allocation](../../src/service_allocation.rs) and
[resolution receipts](../../docs/resolution-framework.md) without importing their
world-specific state.

The first implemented model is one person, a passive state owner, one plot/use
right, one deterministic multi-month growing process and one consumption process.
A bounded planner connects grain to nutrition. It emits process/consumption
transactions and gathers effects. Repeated harvests, warmth, consequences and
finite tool barter now extend this slice. A four-person storage/tax/currency
fixture adds three people with separate plots and stores but shared state budgets
and wild-food supply; general markets and finance remain proposals. [Experiments](EXPERIMENTS.md) remain small probes of this
architecture, not a requirement to build banking, firms or bankruptcy immediately.

The first slice checks the applicable invariants below; market, dynamic-right and
other extension checks remain requirements for those future mechanisms:

- Rights versus ownership versus occupancy; harvest ownership; dated future
  requirements; expired rights, missed stages and cancellation releases.
- Exactly-once process advancement, no early harvest and period-specific need
  fulfillment; no accumulation of unused labor or future satisfaction.
- Joint reservation across markets, no spending incoming same-round proceeds, and
  no double-committed work or inventory.
- Full-trade atomicity, explicit source/sink accounting, partial fills and priority.
- Duplicate batch rejection, failed batch leaving committed state unchanged, and
  effect-buffer overflow before any partial publication.
- Storage-order permutation with stable IDs, repeated seeds, replay, month batching
  and checkpoint continuation at financing/production/settlement barriers.
- Matching CPU reference and CubeCL results for supported operations, with explicit
  integer rules and floating tolerances rather than assumed cross-backend parity.
- Stage visibility: input purchases affect production, production affects later
  sales, and late receipts cannot repair already completed consumption retroactively.

Architecture, scheduling and policy changes should be compared separately. Passing
accounting and replay checks establishes consistency; useful economic behavior
still requires examining needs, completed activity and distribution across agents.

## Tool-output exchange pilot

The [specialist trading experiment](TRADING.md) adds Acquire-phase tool delivery
against a fixed output share, plus bounded posted stock barter. Productive
settlement splits actual output through ledger effects; a process retains its
first supplying-tool contract across stages. Stocks and money use hundredths in
this scenario, with corresponding input, storage and tax scaling. Fixed provider
assignments and a measured provider-count screen precede any endogenous market
entry or negotiated rates. Existing control scenarios retain their units.

## Upfront tool financing pilot

The default 32-person exchange now uses [upfront coin purchases and state
production forwards](FORWARD-TRADING.md). A bounded local process rollout prices
the next year's tool-assisted output and estimates deliverable surplus after
inputs, needs and taxes. Buyers pay their coins first; the state can advance the
shortfall at its posted spot commodity price. This uses finite treasury funds,
not new issuance. Tool transfer, provider payment and forward creation commit
together. Due commodity deliveries occur at the start of Acquire after annual
taxes, with actual stock/storage bounds and persistent shortfalls. There is no
ongoing output share on a cash-purchased tool. The preceding output-share pilot
is retained as a control, including its separate provider-count assessment.

The forward is a dated claim attached to a ledger transaction, rather than an
agent method that directly changes another balance. It feeds production claims
and protects promised stocks from spot sale. Forecasting remains host-side and
local: future competition and shocks may defeat the forecast. This pilot does
not implement a general loan, collateral or insolvency system.

## Additional productive sites

[Additional plot requests](ADDITIONAL-PLOTS.md) compare local production with and
without one more use right, including the first annual tax. A rotating application
queue allocates a finite parcel pool; accepted rights and taxes use the existing
access-agreement tables. Acquire reviews the reserved market outcome before
Productive execution, without adding a scheduler phase. Generic work orders may
start another exclusive-site instance on a different available parcel, with
shared labor, input and tool constraints. Forecast receipts explain why a request
was accepted or declined; acceptance does not create seed or capacity.

## State spread and financed tool value

[State pricing](STATE-PRICING.md) now separates spot valuation, seller-specific
state resale quotes and prepaid forward prices. Tax denomination and issuance
remain independent. Buyers compare local work with and without a tool and require
the resulting stock/coin benefit to exceed cash paid plus the foregone spot value
of pledged goods. Generic completion-stage technique multipliers improve net
harvest output without multiplying recycled inputs; fractional labor allows
smaller equipment service requirements. These calibrated benefits are explicit
catalog choices, not changes to monthly timing or resource allocation.


## Household agreements and collective decisions

The [household pilot](HOUSEHOLDS.md) composes ordinary agent identities through an
accepted formation agreement. One to four adults retain individual needs,
processes, rights and debts while half of actual receipts enters a separate
household inventory. Half of each member's storage becomes a shared capacity
pool; a functioning dwelling can provide a non-rival occupancy service. An active
household requires a surviving adult. Future children have no numeric limit,
with four additional adult places reserved for members growing up in the home;
demographic transitions are not implemented yet.

Household resource requests carry personal and collective benefit. A finite
allocation ranks collective benefit and then reservation sequence. Household
support can precede outside claims without forgiving them, and can fund the
existing personal tax/forward settlement. A bounded household decision lends
only otherwise unused labor to feasible member activities, retaining the other
members' existing work. The household waits when the comparison finds no gain.

This adds explicit before/after sub-boundaries to existing ledger batches, rather
than a new monthly scheduler. Canonical reservation and decision receipts,
contribution carries, storage checks, and CPU gathers make the complete action
atomic and replayable. Current household planning is a net-output heuristic;
collective production targets, outside pooled sales, joint credit underwriting,
and general multi-period bargaining remain future extensions.
