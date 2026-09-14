# System generalization opportunities

Status: proposal for discussion only. These are options, not planned work, an
implementation commitment or authorization to change simulation behavior.

The opportunity is to share mechanisms that already recur across the simulation:
how assets wear, journeys advance, services meet needs, people learn, organizations
retain roles and information reaches decision-makers. A useful generalization
reduces duplicated rules while preserving the differences that give each system
its behavior. Fewer types alone would not establish a simpler or better model.

This proposal concerns components beyond credit. It describes current foundations,
possible generalized models and small comparisons that could establish whether
each abstraction is worthwhile. Source links identify the inspected implementation;
older linked experiment results apply to their recorded revisions.

## 1. Physical assets, construction and maintenance

### State today

[Institutional facilities](material-objects-and-facilities.md) already contain rooms
with material components, condition, useful capacity and unfinished construction.
Their [implementation](../src/facilities.rs) keeps components in the owning
artifact's material ledger rather than introducing another inventory.

Other infrastructure uses its own representations and construction rules: housing,
waterworks, storage, [road upkeep](../src/road_upkeep.rs) and fleet assets.
[Named vessels](../src/vessels.rs), for example, subdivide installed port assets;
their hull materials remain in the port ledger. There are shared concerns, but no
single asset lifecycle covering all these systems.

### Possible generalized model

An asset could expose:

- Stable identity, physical location, owner and authoritative inventory reference.
- Components with material quantities, condition and remaining useful life.
- Construction, repair, expansion and dismantling projects with bounded inputs
  and work remaining.
- Installed capacity and actually available service, with an asset-specific rule
  connecting condition, staffing, inputs and environment to output.

The common layer would handle project progress, component replacement, wear,
ownership and receipts. A waterworks adapter would still require water; a vessel
would still require a crew and a usable route. Installation would not automatically
mean operation. Material substitution would remain limited to supported methods.

### Why explore it, and a small first comparison

New infrastructure could reuse construction and maintenance mechanics instead of
acquiring another independent lifecycle. Start by comparing institutional rooms
with one other stationary asset, such as storage. Extract only matching component
and project responsibilities, preserving its current service equations and monthly
visibility. Check construction, partial completion, repair, destruction and waste.

Do not migrate every asset into individual objects merely for uniformity. Aggregate
GPU infrastructure may need packed data and aggregate component quantities. Shared
semantics need not mean a common in-memory layout or additional GPU readbacks.

## 2. Journeys and transport

### State today

[Household relocation](../src/relocation.rs),
[institutional relocation](../src/institution_relocation.rs),
[expeditions](../src/expeditions.rs), commercial cargo and relief have distinct
journey records and purpose-specific rules. Household journeys account for people,
provisions, blockage, return and loss. Institutional relocation preserves the
organization while leaving its buildings behind.

Some reuse already exists: [religious relief](religious-relief.md) funds shipments
through existing relief transport, and [participation](../src/participation.rs)
tracks named absence and travel duties. These are foundations to extend rather
than evidence that all movement already follows one model.

### Possible generalized model

A journey could describe its itinerary, departure boundary, travelers, cargo and
custody, provisions, reserved capacity, progress and arrival conditions. Common
operations could advance travel, apply route delays, consume provisions, record
losses and deliver surviving contents exactly once.

Purpose-specific behavior would select destinations, authorize departure and
interpret arrival. Migration changes residence; delivery transfers cargo;
exploration records encounters; military movement preserves its own supply and
combat rules. Transport modes would define speed, eligible routes and capacity.

### Why explore it, and a small first comparison

Shared movement mechanics could reduce inconsistent absence, arrival and loss
accounting, and provide consistent hooks for infection and witnessed information.
Start with two existing noncombat flows whose route and delivery semantics actually
match, such as ordinary relief and another compatible goods shipment. Share a
journey clock and delivery receipt before attempting a universal passenger manifest.

Test closure, destination abandonment, partial loss, in-transit saves and disabled
new departures. Existing travelers and cargo must still finish or fail under their
contracts. A generic journey must not create free freight, make all route networks
equivalent, or introduce a second population or inventory owner. Migration and
military adoption would require their own later accounting review.

## 3. Needs and service provision

### State today

Food access, shelter, water service, care and institutional operation define their
own demand, eligibility and responses. Some competition is already explicit. The
[essential-service recovery policy](essential-service-recovery.md) distinguishes
urgent shelter, eligible water repairs and optional housing headroom within a
bounded construction allowance. Research and culture have their own
[service allocation window](service-allocation.md).

These mechanisms do not form a general representation of a beneficiary's need
and the alternative ways to meet it. They also operate at different boundaries and
over different resource pools.

### Possible generalized model

A need could identify its beneficiary, service and units, required quantity,
urgency, duration, acceptable substitutes and consequences of remaining unmet.
A provision option would specify prerequisites, provider, inputs, cost, delivery
time and expected service. An explicit policy would choose among feasible options.

For example, unmet shelter might support a construction or repair proposal where
those alternatives are implemented. A food need could be met through existing
purchase or assistance paths. New alternatives would be model additions, not
automatic consequences of declaring a common need type.

### Why explore it, and a small first comparison

It would become easier to distinguish absent supply, unaffordable supply, failed
delivery and service that arrived too late. Gross-health diagnostics could identify
persistent needs and the unavailable response rather than infer them from balances.

Start with a read-only description of shelter and water shortfalls and their
existing provision options. Verify that it explains current decisions before
allowing it to generate requests. Preserve resource-specific allocation boundaries
and minimum useful work. Do not combine food and shelter into interchangeable
well-being points or implement a global optimizer as a side effect.

## 4. Skills, knowledge and learning

### State today

[Practical knowledge](knowledge-continuity.md) controls access to techniques and
depends on available local holders. Paid lessons and informal exposure already
share [study progress](informal-learning.md) in
[`culture/learning.rs`](../src/culture/learning.rs).

[Production experience](production-experience.md) tracks completed named work in
farming, forestry, mining and construction. Workshops and merchant crews retain
their own experience models. Knowledge of a topic, accumulated practice and
effective productive work are related but distinct representations.

### Possible generalized model

A capability could expose knowledge prerequisites, proficiency, completed practice
and learning provenance. Tasks would declare required capabilities and use their
own mapping from proficiency to effectiveness. Instruction, practice and observation
would contribute through explicit learning channels, with configured transfer
between related capabilities where supported.

Knowing a technique would remain distinct from performing it efficiently or owning
the necessary tools. Informal exposure would not automatically consume paid teaching
time, and reserved idle work would not earn completed-work experience.

### Why explore it, and a small first comparison

Teaching, recruitment and succession could refer to the same capability interface
without introducing another skill counter for each occupation. Start with a common
query and completed-practice receipt for production and workshop work, preserving
their existing learning curves and arithmetic.

Test unrelated-skill controls, absent teachers, partial learning, idle reservations
and continuation. Forgetting, cross-sector transfer and skill-dependent quality are
optional behavior experiments, not necessary parts of a structural extraction.

## 5. Organizations, roles and succession

### State today

Cultural institutions combine membership, facilities, treasuries, operating
[capacity](../src/institution_capacity.rs) and
[service duties](../src/institution_services.rs). Local
[offices](../src/offices.rs), institutional succession and political
[leadership](../src/leadership.rs) have different selection and tenure mechanisms.
Presence, eligibility, vacancy, duties and continuity recur across these systems.

### Possible generalized model

An organization could hold memberships, assets and roles. A role would specify
authority, qualifications, obligations, location requirements, tenure and its
selection/vacancy rules. Appointment, election and hereditary selection would be
separate policies operating on that common role lifecycle.

Organization-specific behavior would remain responsible for its decisions and
services. Holding an office would not automatically grant ownership of assets,
qualification for every task or personal liability for organizational obligations.

### Why explore it, and a small first comparison

The model could support new organization types with less duplicated vacancy and
succession handling. Start by comparing vacancy and holder-presence records for
one local office and one institutional role. Preserve their current selection,
funding, authority and action cadence.

Test death, sustained absence, relocation, replacement and unfinished duties.
A replacement must not inherit completed work as if they performed it. This is
broader than an identity refactor: merging whole organizations too early could
erase useful distinctions between a household, guild, temple and government.

## 6. Information, observations and memory

### State today

[Route warnings](../src/route_warnings.rs) describe encountered closures.
[Witnessed relief appeals](witnessed-relief.md) transmit dated hardship through
actual arrivals. [Social indicators](social-indicators.md) retain pressure memory,
while learning, trade and credit consume other forms of evidence. Freshness,
visibility and response rules are implemented in their respective systems.

### Possible generalized model

An observation could identify its subject, content, source, observed month,
received month, audience, provenance and uncertainty. Common operations could
deliver, deduplicate, retain and age evidence. Decision systems would query what
the actor knows at their decision boundary rather than the current global state.

Keep facts, testimony, beliefs and responses distinct. A remembered hunger pressure
is derived state, not the original hunger observation; a closure report can become
stale without proving that the route has reopened. Confidence and decay rules
would remain appropriate to the evidence type.

### Why explore it, and a small first comparison

Consistent information boundaries could prevent distant omniscience and retrospective
decisions, while making behavior easier to explain. Start with the shared envelope
for route warnings and witnessed appeals, preserving their existing expiry and
delivery rules. Check that reports cannot influence departures that preceded receipt.

Test stale information, repeated delivery, contradictory later observations and
missing provenance. Use bounded local records or indexes; a universal event scan
every month would be a poor trade for cleaner types. Do not replace the historical
event record with an actor's beliefs, or fabricate old knowledge when loading saves.

## Relative value and adoption criteria

| Opportunity | Strongest reason to explore | Main risk |
| --- | --- | --- |
| Physical assets | Existing component model and recurring construction/repair lifecycle | Duplicate inventories or expensive conversion of aggregate GPU state |
| Information and memory | Existing witnessed evidence with clear timing contracts | Accidental omniscience or treating belief as fact |
| Journeys | Repeated movement, absence, delay and arrival accounting | Population/cargo duplication and changed travel semantics |
| Skills and learning | Existing partial sharing, reusable capability queries | Flattening distinct learning mechanisms or rewarding idle work |
| Needs and provision | Better explanations of unmet service and feasible responses | Implicit global allocation or incomparable services collapsed into a score |
| Organizations and roles | Repeated vacancy, eligibility and continuity handling | Erasing authority and selection differences |

Physical assets and information/memory are the strongest initial candidates for
discussion: both have concrete existing foundations and small extraction boundaries.
Journeys offer substantial reuse but carry more accounting and timing risk. Needs
and organizations offer broader behavior possibilities and warrant a narrower
two-system demonstration before choosing an abstraction. This ordering is a
recommendation for evaluation, not a work schedule.

For any candidate, a possible adoption process would be:

1. Identify two concrete consumers and list matching responsibilities, differences,
   authoritative state and monthly boundaries.
2. Introduce the smallest shared interface or receipt while retaining existing
   behavior. Reuse the [resolution framework](resolution-framework.md) where its
   dated comparison/commit semantics apply; it is not a universal transaction engine.
3. Verify unchanged outcomes under compatibility settings, including monthly,
   batched and checkpoint-resumed execution where timing or persistent state changes.
4. Remove duplicated responsibility only once both consumers use the shared path.
   Keep specialized equations and policies in their owning subsystems.
5. Evaluate new behavior separately with controlled cases and matched seed runs.
   Use the [run-health proposal](run-health-evaluation.md) to distinguish actual
   improvement from merely increased activity or newly missing observations.

The [monthly schedule](monthly-schedule.md) and explicit allocation boundaries
remain authoritative. Generalization would not justify changing execution order
to express a sharing preference, moving event creation to Close, or releasing work
backward into completed production. New archive fields must preserve old identities
and quantities without inventing past observations, skill or ownership.

An extraction would be worth keeping if it makes a second consumer easier to
understand, prevents a demonstrated inconsistency or enables a concrete experiment
with less duplicated code. If it mostly adds adapters while leaving every operation
special-cased, retaining separate components may be simpler. Generated comparisons,
logs and trajectories belong under ignored `output/`; committed evidence should be
source, tests and curated Markdown findings with their limitations.
