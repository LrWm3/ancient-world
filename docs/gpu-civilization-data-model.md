# GPU-native civilization data model

**Status: reference only — "break glass in case of emergency."**

Explore this architecture only if regular CPU improvements plus targeted GPU
kernels for measured bottlenecks prove insufficient to reach the large population
sizes needed later, after settlement limits are raised or removed. Those incremental
improvements remain the primary approach. This document is a contingency reference,
not an active implementation plan; it contains no runtime implementation or
performance claim.

Source baseline: `09d9150`, inspected 2026-09-13.

## Purpose

If the contingency above is reached, this design would make GPU state the
authoritative representation of the civilization simulation.
The design question is: **what is the GPU-native representation of each concept?**
The existing Rust object model becomes an import/export, inspection and reference
view. Simulation decisions operate on indexed data resident on the GPU across
months. The CPU handles command submission, capacity provisioning, persistence,
input and presentation; it does not have to calculate civilization decisions.

This design permits different algorithms and individual outcomes. The working
behavioral target is approximately ±5% of selected current outcomes, defined over
explicit metrics, horizons and scenarios. This is an acceptance target, not an
established property or a promise that every historical trajectory stays close.
Identity, causal timing, eligibility and accounting have separate correctness
requirements and do not receive a 5% error allowance.

The [monthly schedule](monthly-schedule.md) remains the timing contract. GPU
representation does not imply a global simultaneous market, a new social priority
policy, or one universal decision function. The core objective is to express the
model through a small set of reusable data structures and computational primitives.

## 1. Seven categories of simulation data

| Category | Meaning | Physical representation |
| --- | --- | --- |
| Entities | People, households, settlements, institutions, firms, political units, assets | Typed IDs and structure-of-arrays tables |
| Edges | Kinship, membership, ownership, contact, routes, dependencies | Typed edge columns; grouped offsets and secondary indexes |
| Vectors | Traits, skills, preferences, needs, beliefs, pressures | Fixed-width numeric columns with a versioned feature schema |
| Proposals | Candidate actions, offers, work requests, transfer requests | Bounded arenas with actor offsets, scores, requirements and status |
| Pools | Money, goods, personal time, room space, freight, source stocks | Authoritative balances/capacities plus dated allocation and reservation records |
| Events | Deferred transitions and committed causal facts | Typed records, phase-aware due indexes, payload tables and causal edges |
| Logs | Historical events, transfers, tenures, observations and receipts | Append-only columnar chunks with stable IDs |

An institution spans several categories: its entity row identifies it; edges
identify members and property; vectors describe characteristics; pools hold its
money and service capacity. These are conceptual categories, not seven giant
untyped buffers.

| Existing semantic shape | Proposed storage |
| --- | --- |
| `Vec<Entity>` | Entity columns plus active-row index |
| `Vec<Id>` inside an entity | Edge table or offset/count into a packed arena |
| `BTreeMap<Id, Value>` | Direct indexed column when dense; sorted key/value columns otherwise |
| `BTreeSet<Id>` | Deduplicated sorted membership edges; bitset for a small bounded domain |
| `Option<Id>` | ID plus validity bit or reserved invalid ID |
| Enum with payload | Numeric tag plus typed payload row |
| Inventory | Dense owner-by-good array for common goods, sparse holdings for uncommon items |
| Deferred action | Due month, due phase/substep, status, typed payload reference |
| Narrative text | String ID plus cold text; decision-relevant semantics in typed columns |
| Arbitrary JSON snapshot | Explicit dependency rows and captured values |

Shared execution primitives are map, filter, compact, sort-by-key, scan, segmented
scan/reduce, gather, controlled scatter, join, candidate generation, argmax or
sampling, grouped arbitration and append. Each subsystem composes these primitives
with its own eligibility, priority and settlement rules.

## 2. Storage and mutation contracts

### 2.1 Identity survives physical reorganization

A stable `PersonId` is not necessarily the current physical row. IDs are typed;
`PersonId(7)` and `InstitutionId(7)` refer to different entities. Use monotonic IDs
within an entity type, never recycled for another historical entity.

```text
person_directory[id] -> hot row, historical row, or invalid
person_id[hot_row]
person_born_month[hot_row] : i32
person_civilization[hot_row]
person_presence_kind[hot_row]
person_presence_subject[hot_row]
person_alive[hot_row]
person_traits[trait, hot_row]
person_skills[skill, hot_row]
```

Direct `id == row` indexing is a valid first implementation while entity rows stay
in place. Compact active worklists and edges first. If entity rows are later
compacted, update the directory and inverse IDs; persistent edges and logs retain
stable IDs. Cached row references carry a layout revision and expire on compaction.
An active bitset is distinct from a record-exists bitset: a dead ancestor still
exists and can still be queried.

Use `u32` local row indices with checked capacities. Existing `u64` event and loan
IDs remain logically 64-bit; represent them as two words where necessary. Never
cast IDs to floats. Signed prehistory birth dates remain signed. ID exhaustion is
an explicit boundary error, not wraparound.

### 2.2 Authoritative columns and derived indexes

Every quantity has one owner. A membership inverse, a town population total or a
debug object is either authoritative under a declared mode or a derived projection.
It cannot independently mutate the same fact.

Each table/index descriptor carries schema version, length, capacity, layout
revision and observation boundary. Boundary means month, phase and named substep;
some observations also carry the prior completed social/ecological month.
Consumers must know which version they read. Kernels exchange these descriptors
on the GPU; the host need not inspect individual rows.

Logical structure-of-arrays does not require a separate GPU binding for every
field. Related columns can occupy aligned ranges of a few buffers. Physical
packing must obey the backend's alignment, binding, buffer and dispatch limits.
Vector storage is dimension-major when adjacent lanes process adjacent actors;
tiled actor/feature layouts are alternatives to benchmark. `[actor][feature]` in
an equation describes logical indexing, not a mandated memory stride.

### 2.3 Edges, indexes and base plus delta

```text
relationship_id[E]
relationship_src[E]
relationship_dst[E]
relationship_kind[E]
relationship_strength[E]
relationship_updated_month[E]
relationship_valid[E]
src_offsets[source_count + 1]
dst_sorted_edge_rows[E]       # only when reverse queries justify it
```

Group immutable base edges by the common query key. Keep additions, replacements
and deletions in a delta arena; merge, sort, deduplicate and rebuild offsets at
declared maintenance boundaries. Deletion targets an edge ID or a complete unique
key. Replacements carry an operation sequence so conflicting updates have an
explicit winner under subsystem rules.

**Delta visibility is immediate at the next permitted consumer.** A query reads
base plus relevant deltas and suppresses deleted/replaced base records, or the
coordinator materializes a merged view before that consumer. Postponing physical
rebuilding cannot postpone a marriage, death, ownership transfer or departure.
Maintain a grouped delta index when a flat delta scan becomes expensive.

One-to-one current relationships can use direct arrays, with reverse edge indexes
derived from them. Keep multiple indexes only when they remove material repeated
work. Exact ancestry still involves irregular gathers and potentially multiple
graph traversals; flat storage improves representation without making every
genealogical query constant-time.

### 2.4 Dynamic capacity and finite working memory

Variable output uses count, exclusive scan, capacity check, then fill. Producers
receive disjoint ranges. Optional atomic appends reserve chunks and report overflow
before publishing accepted actions; an overflow must never silently drop a birth,
payment, event or competing candidate.

Pool growth is a host allocation operation at a safe boundary. If capacity is
insufficient, retain the uncommitted input boundary, enlarge storage and rerun the
affected preparation. Do not replay a committed debit. Large workloads can be
chunked, but all contenders in the same allocation window must participate before
committing its result. Chunk order is not an allocation policy.

Memory accounting includes base state, deltas, reverse indexes, sort/scan scratch,
candidate/requirement arenas, transaction staging and queued log export. Candidate
counts can dominate entity storage. Use sparse eligible actor-option pairs and
batched evaluation; a nearest-k or top-k candidate restriction changes behavior
unless it preserves all options the current decision could choose.

Historical chunks can leave VRAM after persistence and consumer acknowledgement.
All facts still used by decisions remain in resident summaries or indexed working
sets. Exhausted log staging applies backpressure. Memory compression, forgetting
and changing retained ancestry are separate model decisions.

## 3. Concrete mapping of the current civilization model

The following layouts specify logical columns and mutation responsibility. They
are a source-informed schema design, not final byte offsets or a generated ABI.
Catalog widths, numeric scales and optional components must be resolved before
each implementation slice is enabled.

### 3.1 World root, settlements and environment

Sources: [civilization.rs](../src/civilization.rs),
[economy.rs](../src/economy.rs), [society.rs](../src/society.rs),
[history_environment.rs](../src/history_environment.rs).

| Current structure | GPU tables / columns | Mutation and visibility |
| --- | --- | --- |
| `History`, `LivingHistory` | World header: seed, month, source epoch, terrain/ecology clocks, system flags, schema versions, initial ledgers, incomplete-boundary marker | Advance clocks once per month; preserve separate frozen/living close paths |
| `Civilization` | Civilization ID, language ID, name ID, leader person ID | Founding appends; leadership commits update leader at the existing review boundary |
| `Site` | Site ID, civilization, island, cell, founded, name ID, lifecycle state | Founding appends; abandonment changes state, never reuses identity |
| `Stocks`, `Demography` | Population/cohort, food, habitat, mortality, nutrition and ledger columns | Production/demographic settlement has explicit write ownership; population mode determines authoritative counts |
| `Economy` | Site-good stock/flow/price/target columns; site-crop, herd, workshop, labor, storage, housing, waterworks, extraction and fishery components | Existing GPU arithmetic becomes a consumer of committed plans; optional components use indexed tables |
| `SettlementLifecycle` | Current/pending size, persistence counters, last harvest/storage/housing observations, waterworks state | Execute records completed outputs; Respond updates lifecycle; Close records timeline |
| `Candidate` | Candidate cell, island, productive area/yield, score and landmark ID | Survey output and founding proposals, followed by spatial/capacity arbitration |
| `SocialCell`, `SocialState` | Site pressure, livelihood, distribution, ownership and affiliation summary vectors with observed month | Derived at Close; earlier consumers intentionally retain prior-Close evidence |

Terrain `gpu::Cell`, `ecology::EcoCell` and regional grids already have GPU
representations. Retain their authority and connect civilization tables through
cell IDs, active-cell indexes and transfer records. `Resources::Source`, regional
mines and discovery sources remain distinct pools even when they share a cell.

### 3.2 People, households, ancestry and demographic authority

Sources: [civilization.rs](../src/civilization.rs),
[politics.rs](../src/politics.rs),
[population_registry.rs](../src/population_registry.rs),
[individual_demography.rs](../src/individual_demography.rs).

| Current structure | GPU representation | Mutation rule |
| --- | --- | --- |
| `Person` | Stable ID, name ID, civilization, signed birth month, death month/validity, predecessor | Birth/identification appends; death marks once; historical identity survives |
| `Household` | ID, site, head, share, founded, parent household, generation, vacancy, name ID | Head succession and residence transfers update explicit columns |
| `Kinship` | Person-to-household association and two optional parent IDs; reverse child adjacency | Append ancestry links; maintain current residence separately from ancestry |
| `Marriage` | Union ID, two partner IDs, started/ended, children count, last birth; active-partner index | Resolve partner conflicts before forming a union; ending retains its record |
| `NamedDemography`, `ResidentSlots` | Mode flags, site/age-band named counts, anonymous slots, birth/death/defense remainders | Identification transfers anonymous representation to named representation without creating population |
| `DemographicProjection`, `Outcome`, `Snapshot`, `Comparison` | Dated site projections, person-risk rows, candidate death rows, outcome totals and comparison receipts | Select the configured resolution once; apply one population commit |
| `PopulationReconciliation` | Site cohort/named/residual/overhang columns and exception-ID arenas | Close reconciliation checks residence/travel/military/expedition partitions |

Do not turn every cohort into a person as an incidental storage conversion.
Aggregate, named and fractional residual authority are existing model choices.
In individual mode, personal outcomes plus the explicit residual determine the
population result. In legacy mode, assigning identities to already-counted losses
does not debit population again. Individual ancestry can be exact in either mode
for the people actually represented.

### 3.3 Presence, domestic groups and work

Sources: [participation.rs](../src/participation.rs),
[domestic.rs](../src/domestic.rs), [labor.rs](../src/labor.rs),
[agriculture_participation.rs](../src/agriculture_participation.rs),
[workshop_resolution.rs](../src/workshop_resolution.rs).

```text
presence(person, kind, subject, origin_household, effective_boundary)
domestic_unit(id, anchor_kind, anchor_id, home_site, formed, ended)
domestic_membership(person, unit, effective_boundary)
availability(person, month, capacity, care_reserved, committed)
commitment(id, boundary, site, activity, requested, granted, used, released, status)
commitment_member(commitment, person, granted_work, used_work)
care_request(unit, site, need)
care_assignment(request, carer, requested, reserved, completed)
```

`Household` is the current economic/property grouping; `domestic::Unit` is a care
and co-residence grouping. They remain separate entity types. Presence resolves
military duty, expedition duty, relocation, residence, death and unknown status
under an explicit transition policy. It is updated when movement/death commits;
an opening presence index cannot remain authoritative after those changes.

Participation's experience and completion vectors become optional person
components. `FarmPlan`/`FarmAssignment`, workshop `Staffing`, crew work and office
service attach typed detail rows to the common commitment header. Domestic care
keeps its own accounting and priority rather than being forced into an activity
variant that the current model does not have.

Caregiver/dependent and worker/job candidate edges share scoring and grouping
primitives. Qualification, location, available time, employer funds and equipment
are still joint constraints. Completion and released time settle once. Late release
never creates work in already-completed production.

### 3.4 Preferences, personality, beliefs and decisions

Current anchors: `culture::Agent.traits[6]`, `skills[4]`, relationship strengths,
`agriculture::RoleState`, political interest scores and relocation pressure/memory.
Additional vectors below are proposed behavioral extensions, not existing fields.

| Vector family | Example dimensions | Update boundary |
| --- | --- | --- |
| Stable preferences | Food security, wealth, kin proximity, status, autonomy, religious participation | Initialization and explicit slow adaptation |
| Behavioral traits | Risk sensitivity, patience, inertia, conformity, exploration | Stable or slowly changing under a declared rule |
| Needs | Hunger, care burden, housing shortage, illness, cash pressure | Derived from dated completed observations |
| Beliefs | Expected supply, perceived danger, seller reliability, destination prospects | Only when the actor receives relevant evidence |
| Sensitivities | Response to deterioration, uncertainty, losses and repeated hardship | Actor parameters used by the decision-family transform |
| Skills | Existing cultural/workshop/production practice and learned capabilities | Completed eligible work/learning |

For decision family `d`, actor `i`, option `a`:

```text
eligible(i,a) = hard_constraints(current state, dated plan, option)
features_d(i,a) = transform(perceived consequences, relationships, costs)
weights_d(i,t) = adapt(preferences, traits, needs, beliefs, recent changes)
score_d(i,a) = dot(weights_d(i,t), features_d(i,a)) - switching_cost_d(i,a)
```

Each family owns a versioned feature schema: names, units, normalization, sign,
missing-evidence handling and width. Equal widths do not mean equal semantics.
Cross-family consistency comes from shared actor traits and explicit transforms,
not interpreting “component 3” identically in unrelated feature lists. Preserve
existing trait meanings during import; do not relabel the six current traits as
new personality dimensions.

Use nonlinear feature transforms for diminishing marginal utility, thresholds,
loss aversion, interaction effects and risk. A dot product over transformed
features is not a requirement that behavior be globally linear. Actor-specific
beliefs prevent direct access to unknown destinations or unreceived testimony.

Select argmax, ranked proposals or a calibrated stochastic choice rule. For
softmax sampling use a positive temperature and numerically stable normalization;
define behavior for an empty feasible set and reject nonfinite scores. Include
stay/idle/decline options where semantically valid. Temperature, memory decay and
switching penalties are behavioral parameters, not GPU scheduling parameters.

When option features are shared, scoring can be matrix multiplication. When
features are pair-specific, use sparse batched dot products. Fuse feature
construction and scoring when materializing every feature vector wastes memory;
retain selected feature contributions for explanation and calibration.

The discrete-choice foundation is supported by
[McFadden's conditional logit work](https://eml.berkeley.edu/reprints/mcfadden/zarembka.pdf).
Longer-term predicted outcome vectors could later use
[successor features](https://arxiv.org/abs/1606.05312); estimating those predictions
is additional modeling work, not a prerequisite for this architecture. Neither
reference validates a particular personality schema for this world.

### 3.5 Money, inventories, property and productive assets

Sources: [household_economy.rs](../src/household_economy.rs),
[enterprises.rs](../src/enterprises.rs), [culture.rs](../src/culture.rs),
[facilities.rs](../src/facilities.rs), [resources.rs](../src/resources.rs).

| Concept | GPU representation and ownership |
| --- | --- |
| Account | Account ID, owner type/ID, currency ID, balance and revision; owner-to-account index |
| Household account | Cash-account reference plus need/common/purchased food, hunger, observed site/month and cumulative income/expense columns |
| Inventory | Owner type/ID, good ID, quantity; dense site-good tables where useful, sparse special holdings elsewhere |
| Household property share | Household/site share relation under the current share convention |
| Firm | Owner household, site, family, leased units, account, capital, wage policy, staffing and lifecycle columns |
| Artifact | Creator, actual owner, custodian, location/transit state, topic, tradition, lost/destroyed flags; material-component edges |
| Ownership dispute | Claimant-to-asset edges distinct from actual ownership; petition/remedy rows |
| Facility | Facility-to-room and room-to-component edges; method ID, capacity, remaining work, component mass/condition/wear |
| Resource source | Source ID/cell, mineral ID, initial/remaining/extracted quantities, depletion causes |
| Regional mine | Mine ID, source/site relation, opened/retired, monthly ceilings and cumulative extraction |
| Residue and environmental returns | Typed source-to-sink transfers and cumulative mass accounting |

An artifact currently has one `Owner` and potentially several claims. Encoding it
as fractional co-ownership would change the model. Use ownership fractions only
where the existing asset semantics support shares. Location, custody, title and
disputed claim are independent facts, including while property is in transit.

Unify cash authority through account IDs; compatibility fields such as council
treasury or firm cash become views. A household wallet and a town stockpile do not
become one pool merely because both are indexed. Every transfer names its actual
source, destination and currency/good.

### 3.6 Retail, trade, contracts and credit

Sources: [economy.rs](../src/economy.rs),
[export_contracts.rs](../src/export_contracts.rs),
[credit.rs](../src/credit.rs), [credit/state.rs](../src/credit/state.rs),
[credit/underwriting.rs](../src/credit/underwriting.rs).

```text
offer(id, seller, good, available_quantity, price, location, terms)
order(id, buyer, good, desired_quantity, limit_price, decision_boundary)
trade_candidate(order, offer, route, score, feasible_quantity)
contract(id, buyer, seller, good, remaining, escrow_account, expires, observations)
cargo(id, from, to, good, quantity, paid, route, due, delay, voyage_clock)
cargo_route_edge(cargo, edge, reserved_capacity)
loan(id, lender_account, borrower_account, currency, repayment_source, terms, status)
loan_balance(loan, principal, interest_due, accrued_through_month)
credit_evidence(source, beneficiary, observed_month, receipts, costs, risk)
credit_request / offer / grant / servicing_receipt / loan_entry
```

`RetailPlan` becomes site headers and household need/demand rows. Reserve captures
funded demand. Execute previews household nutrition for mortality and subsequently
settles wallets at the current retail step. Preview is not a second consumption
or payment. Family gifts, payroll, dividends and council relief retain their
separate funding windows and receipt categories.

Vector-based buying scores need satisfaction, delivered cost, delay, reliability
and risk. Quantity decisions use marginal utility or explicit demand curves.
Merchant opportunity scores can include margin, cash tied up and uncertainty.
These are alternative behavior policies; importing current target-stock and
supplier rules is a separate baseline mode.

Matching must resolve buyer money, seller stock, endpoint freight, intermediate
edges and sea capacity together. Contracts retain escrow, funded/unfunded orders,
observed delivery, planned and dispatched quantities. Goods and cash cannot be
counted at both endpoints while in transit.

Credit carries typed repayment-source records, protected cash, operating reserves,
receipt pledges, interest, arrears and write-offs. In the inspected coordinator,
servicing runs in Open after economy preparation; opt-in council bridge credit
runs at the start of Reserve. Annual tax observations arrive in Respond. Preserve
those boundaries and the distinction between expected receipts and available cash.
Currency IDs do not imply an implemented exchange market.

### 3.7 Culture, knowledge, institutions and religion

Sources: [culture.rs](../src/culture.rs),
[culture/work_requests.rs](../src/culture/work_requests.rs),
[culture/learning.rs](../src/culture/learning.rs),
[institution_services.rs](../src/institution_services.rs),
[discoveries.rs](../src/discoveries.rs).

| Current structures | GPU representation | Critical distinction |
| --- | --- | --- |
| `Agent` | Optional person component: traits, skills, occupation/goal IDs, action counters | Cultural agent component is not a second person identity |
| Knowledge, known places, studies, sources | Actor-topic/place edges; progress and source-event columns; bitsets for small fixed topic domains | Knowing a topic, partial study and possessing a teaching source differ |
| `Institution` | Entity columns; member/role edges; knowledge edges; account and property references | Institution knowledge does not imply an available qualified teacher |
| `Capacity`, `MeetingPlace`, `Mandate` | Readiness, building reference, current holder/vacancy, observed work and support | Money, room capacity, member labor and completed duty remain separate |
| `WorkPlan`, `InstitutionWorkPlan`, service `Plan`/`Receipt` | Dated action headers; participants; object/dependency rows; election/upkeep/admin details; room and work receipts | Preserve minimum useful grants and captured policy/order |
| `Patron`, `Tradition`, cultural `Account` | Entity columns plus voyage/witness, parent-tradition and account-fact/event edges | Cultural account is an authored narrative record, not a money account |
| `ReligiousDynamics`, `Persuasion`, `Reform` | Household-tradition influence rows, site-tradition reform counters, explicit themes | Affiliation changes use the intended observation boundary |
| `Discoveries`, `Workshop`, `ResearchPlan`, `Botanicals` | Source/sample pools, site workshop components, dated processing plans, progress/outcome columns | Samples, methods, curated objects and applied remedies have distinct ledgers |
| Heritage finds/studies/recognition, recovery requests | Find/artifact entities, author/funder/witness edges, pending recovery rows | Evidence and provenance remain accessible to later decisions |

The current `WorkPlan.identities` JSON is used to compare execution inputs with
captured inputs. Replace it with typed dependency slices for people/knowledge,
faith, buildings, recoveries, objects and institutions. For a baseline port,
compare the same captured values and sets that current code compares. Version
counters can accelerate unchanged checks, but a version change alone must not
cancel work that changed and returned to an equivalent state, or work unrelated
to the captured scope. Emit explicit cancellation reason and changed-field IDs.

Learning services retain research/culture sharing, per-person matching, minimum
election grants, room feasibility, Stable/Rotating member priority and
FullUpkeepFirst/EssentialFirst duty policy. They do not become interchangeable
weights in a generic allocation pass. Institutional relocation appends a move
record and portable-asset edges; arrival changes location in Open.

### 3.8 Government, political affiliation, law and leadership

Sources: [politics.rs](../src/politics.rs),
[governance.rs](../src/governance.rs), [leadership.rs](../src/leadership.rs),
[offices.rs](../src/offices.rs), [civic_petitions.rs](../src/civic_petitions.rs),
[artifact_petitions.rs](../src/artifact_petitions.rs).

```text
council(civilization, account, active_tax, active_distribution, policy_revision)
pending_policy(owner, kind, parameters, decided_month, effective_month, cause)
faction(id, civilization, interest, organizer, support, dissent, cohesion)
household_faction[household]
site_controller[site]
administration(site, controller, autonomy, loyalty, unrest, unpaid, crisis)
office(id, site, controller, selection_rule, current_tenure)
tenure(office, holder, began, due, ended, appointment_event)
leadership_mandate(polity, rule, weighting, vacancy, absence, review_dates)
treaty(id, party_a, party_b, signed, expires, status, cause)
petition(id, type, claimant, respondent, evidence, remedy, status)
```

Existing controllers, councils, civilizations and factions remain distinct roles;
a generic polity reference must not collapse them. Jurisdiction and territorial
claims use polity/site/cell edges. Relations use typed party-pair edges.

Candidate scoring for affiliation, organizers and leadership can use feature
vectors. Eligibility and the selected hereditary/faction/election rule still
determine which choices are legal. Property inheritance and political leadership
are separate transitions. Tenure history remains append-only.

The current tax and distribution policy records fit fixed parameter blocks with
activation events. Broader laws require typed rule schemas if later added; this
design does not claim that the repository already implements a general legal
system. Petitions retain consent, claimant identity, observed evidence, completed
office service, remedy and actual compensation transfers.

### 3.9 Warfare, armies, occupation and peace

Sources: [military.rs](../src/military.rs), [society.rs](../src/society.rs),
[siege.rs](../src/siege.rs), [peace.rs](../src/peace.rs),
[military_supply.rs](../src/military_supply.rs).

| Current structures | GPU tables |
| --- | --- |
| `War`, `Raid` | War parties/goals/status; army origin/target/phase, food, equipment, aggregate soldiers, named-members-present flag, loss remainder |
| `Duty`, `Career` | Person-army assignment, origin household, start month; cumulative service columns |
| `Defense`, `Siege` | Site/army/artifact relations, construction commitment, integrity, active siege interval and causes |
| `Supply` | Army shipment with route, quantity, due date, return/status fields |
| `Peace`, `Payment` | Agreement terms, payer/payee accounts, installment schedule, arrears and payment log |
| Occupation and territorial snapshots | Current controller/occupation state; append-only dated control changes |

Resolve conflicting casualty, recruitment, return and property transitions by
person/asset. Named members and aggregate soldier remainders follow the configured
resolution. Army supply arrives before rations; rations, casualties and returns
precede household succession; sieges affect subsequent deliveries. Independent
battles can execute concurrently, but shared forces and targets require arbitration.
Peace installments remain Open cash claims ahead of later council claims.

### 3.10 Routes, ships, expeditions and relocation

Sources: [navigation.rs](../src/navigation.rs), [shipping.rs](../src/shipping.rs),
[vessels.rs](../src/vessels.rs), [expeditions.rs](../src/expeditions.rs),
[relocation.rs](../src/relocation.rs), [freight.rs](../src/freight.rs).

| Current structures | GPU representation |
| --- | --- |
| `Route`, `SeaLane`, `FrontierRoute` | Route ID/type/endpoints, packed path-cell arena, distance, open/flood state, topology/environment revision |
| Navigation service | Grid/graph state, active frontiers, distances, predecessor arrays and route query/result arenas |
| `Port`, `HarborWork`, `Fleet`, `Vessel` | Site/access paths, assets, work/readiness, vessel ownership and commission columns; vessel-person crew edges |
| `CrewWork`, `VoyageClock` | Dated crew commitments/payroll/productivity and separately dated voyage progress |
| `Expedition`, `Crew`, `Charter`, `Find` | Expedition phase/objective/sponsor, route, provisions/cash/specimens, named or anonymous crew rows, find/provenance edges |
| `RelocationState`, `Pressure` | Site production-history rings, hunger bitfields, observation dates and departure cooldowns |
| `Journey`, `TravelRoster`, `Passenger` | Household journey state, age cohorts, named passenger edges, death remainders, inventories, infection and testimony |
| Institution `Move` | Institution/from/to/route/due/status and portable-asset edges |

Paths are variable-length packed data; a frontier traversal is a shared primitive,
not a requirement to replace the existing GPU navigation solver. Cached paths and
derived reachability must respect route closure, environment and siege revisions.
Route geometry, connectivity and capacity are separate arrays.

Relocation candidate edges are household-to-destination with route references.
Features can include food/housing gain, livelihood, known kin, cultural familiarity,
travel cost, perceived danger and uncertainty. Hard checks retain source provisions,
destination reserves/housing, route eligibility and population admission limits.
Evaluate staying as an option in a behavioral extension. Score proposals in parallel,
then commit accepted departures against live shared capacity and exact passengers.
The baseline can retain sequential departure priority on the device.

Sea progress uses the preceding funded crew interval. New Respond cargo does not
receive another Open arrival pass. Destination testimony and received warnings
cannot influence the departure that produced them.

### 3.11 Relief, memory, contagion and environmental response

Sources: [relief.rs](../src/relief.rs),
[religious_relief.rs](../src/religious_relief.rs),
[social_memory.rs](../src/social_memory.rs),
[contagion.rs](../src/contagion.rs), [hazards.rs](../src/hazards.rs).

| Current structures | GPU representation and update |
| --- | --- |
| `Appeal`, relief observations | Dated origin/host/household/route/report rows with answered status and captured evidence |
| `Shipment`, `Mission` | Relief transport/action rows, food/account/room references, promised/paid/delivered/returned amounts and causes |
| `Report`, `AidMemory`, `Warning`, `Encounter` | Observer/subject/type rows with observed and received months, value, source event and outcome |
| `TradeContact::Receipt`, heritage witnesses | Sparse dated contact/evidence edges; declared rolling aggregates |
| `Contagion::Pool`, `Exposure` | Site/journey SEIR vectors, imported exposure, partition transfer ledgers and observation dates |
| `FloodImpact`, living returns | Active cell/site disruption components and bounded source-to-environment transfers |

Relief allocation keeps donor food and both endpoint freight constraints, the
current secular window and religious fallback. A remembered favorable report
does not create available food. Answered appeals cannot be replayed.

Memory can use base/delta sparse records and compact summaries. Retain the current
retention and visibility semantics during representation conversion. New bounded
memory, decay, selective forgetting or embeddings change behavior and require a
separate policy and calibration. Outstanding obligations and evidence needed for
claims cannot disappear under a generic psychological forgetting rule.

Contagion advances once in Open against the prior completed population, transfers
exposure on arrival, feeds health burden and reconciles partitions at Close. A
vector representation does not introduce a second mortality debit.

### 3.12 Events, history, names, catalogs and diagnostics

Sources: [civilization.rs](../src/civilization.rs),
[history_timeline.rs](../src/history_timeline.rs), [naming.rs](../src/naming.rs),
[spatial.rs](../src/spatial.rs), [resolution.rs](../src/resolution.rs).

```text
event(id, month, phase, substep, kind, site, other, payload_type, payload_row)
event_subject(event_id, entity_kind, entity_id)
event_cause(event_id, cause_event_id)
event_path(event_id, path_offset, path_count)
event_anchor(event_id, role, grid_id, cell_id)
scheduled_action(id, due_month, phase, substep, kind, subject, payload, status)
consumer_cursor(consumer, last_consumed_event_sequence)
timeline(site, month, population, food, treasury, abandoned)
```

The current `Event` is primarily a committed historical fact; cargo, journeys and
policies already carry their own due fields. The design distinguishes scheduled
actions from historical facts rather than pretending the current code has one
universal event heap. Due indexes can reference authoritative cargo/loan/policy
rows instead of duplicating their mutable quantities.

Use fixed headers and per-type payload tables with offset/count arenas for causes,
subjects and paths. An arbitrarily large event does not have to fit a tiny fixed
payload or lose provenance. Allocate IDs before later same-phase consumers need
them, publish completed records after a dispatch dependency, and maintain consumer
cursors. Close records the completed boundary; it is not the only event-creation
point.

For monthly buckets, retain absolute due month and phase. A ring index alone cannot
distinguish events separated by its horizon. Use a sorted overflow queue for more
distant dates; promote into the ring as the horizon advances. Cancellation and
rescheduling invalidate the old entry. A consumer processes a captured range;
new same-month actions run only in explicitly permitted later substeps.

History is append-only columnar storage with site/entity/type/month indexes.
The existing bounded `Timeline.samples` keeps its current retention contract;
an extended full timeline is a separate feature. A historical record referenced
by current decisions must have an accessible typed projection before cold export.

Names use string IDs for presentation. `Language`, `Lexeme`, `WordUse` and
`NameRecord` also contain evolving semantic state: concept IDs, source language,
adoption dates, sound-change parameters and vocabulary relations remain typed
simulation data. GPU kernels can select concepts and transformation recipes;
rendering text can occur on demand. Any string currently compared by decision code
(occupation, artifact kind, action name, event kind, method) becomes an enum/catalog
ID, not a cold string whose meaning requires CPU execution.

Editable rock/mineral/plant/guild, economy, agricultural, material, method and
patron catalogs remain source data. Compile them into versioned immutable GPU
tables and stable lookup IDs. Grid and spatial feature representations retain
their actual grid identity and precision. UI camera/renderer state, archive
headers and human-readable reports remain host views.

Resolution receipts use numeric system/metric/unit/reason IDs, typed snapshots and
expected/actual/explained columns. Work, supply, care and travel comparisons can
share reduction machinery without taking ownership of the simulated quantities.

## 4. Proposal, pool and settlement protocol

```text
proposal(id, actor_kind, actor_id, action_kind, option_id, boundary,
         policy_id, priority_key, score, minimum_useful, requested, status)
requirement(proposal_id, pool_id, amount_per_unit, fixed_amount)
pool(id, kind, owner_kind, owner_id, unit_or_currency, boundary,
     opening, available, revision)
allocation(proposal_id, requested, feasible, allocated, reserved)
reservation(proposal_id, pool_id, amount, boundary, status)
settlement(proposal_id, completed, consumed, released, shortfall, reason)
transfer(action_id, source_pool, destination_pool, amount, unit, status)
```

Each pool declares scope and time units: personal worker-months, site service
worker-months, room occupant-time, road freight, source mass or account currency.
Related limits need not be interchangeable pools. The service ceiling is only part
of labor, and an administration allowance is not escrow.

The common protocol is:

1. Capture the correct dated inputs and collect feasible candidates/requests.
2. Apply the subsystem policy against all competing requests in that window.
3. Match eligible participants and evaluate all joint requirements, including
   minimum useful work and indivisibility.
4. Reserve a jointly feasible bundle; publish acceptance only when every required
   pool can honor it. A rejected bundle leaves no committed partial debit.
5. Execute in the existing phase, rechecking required live eligibility.
6. Settle actual use/output and record releases, shortfalls and causes once.

Single-pool divisible requests can use grouped sums, demand caps and proportional
or weighted allocation. Existing two-claim research/culture rules remain their own
policy. One-to-one matching, indivisible elections and multi-edge freight are
different constraint problems that reuse primitives, not one universal solver.

For joint constraints, an initial correct device implementation can process each
conflict component in explicit priority order while independent components run
concurrently. A component connects proposals sharing any required resource. A
highly connected component may be large and sequential. Later implementations
can use conflict-free rounds, tentative grants followed by all-resource validation,
and explicit release/refill rules. They must define progress, termination and
unfilled-demand handling; arbitrary iteration caps can change outcomes.

Atomic subtraction on individual balances does not make a multi-pool action
transactional. Commit stages use prevalidated reservations, disjoint writers or
grouped transfer reductions. Incoming funds are available only at the boundary
where the current rules make them available, not automatically within a parallel
clearing round.

Retain the [service allocation](service-allocation.md) windows, institutional duty
and member policies, secular relief priority, council allowance semantics and
existing work execution boundaries. Sorting for locality or race avoidance does
not silently choose a new social priority. Sequential operations are permitted
inside GPU execution when causality or the selected policy requires them.

## 5. Execution within the existing month

```text
Open -> Reserve -> Execute/settle -> Respond -> Close
           |              |           |
       dated plans     actual use   later decisions
```

Within a compatible decision window, the dataflow is:

```text
dated state -> needs/evidence -> candidates -> features/scores -> proposals
            -> policy allocation -> joint reservations -> committed actions
            -> settlement + causal events -> visible state and derived indexes
```

This is repeated where existing systems need it, not performed once as a universal
monthly auction.

| Phase | GPU work and boundaries to preserve |
| --- | --- |
| Open | Prepare current state; increment month; activate due policies; peace and institutional arrivals; contagion and disruption; due military/market/relief/household arrivals; appeals; economy and credit servicing; source preparation. Cargo progress retains prior-month crew funding. |
| Reserve | Council credit in the inspected baseline; clear old reservations and establish care/personal availability; committed crews; research/culture; office/defense; fisheries and source allowances; production/enterprises; additional crews; optional named production attendance; retail. Persist grants and dependency evidence. |
| Execute/settle | Run existing production/consumption with committed plans; settle care/office/defense; selected demographic authority; production participation, sources, enterprises and crews; asset outcomes; household retail. Nutrition preview and payment remain separate. |
| Respond | Market decisions from opening delivery evidence; release work; expeditions and relocation; lifecycle; social outcomes, genealogy, culture, offices, petitions and governance; annual decisions when due. Preserve named causal substeps and same-month event consumers. |
| Close | Synchronize projections and required indexes, settle comparison receipts, social/contact observations and validation. Frozen timeline here; living returns/reconciliation and timeline at the coupled boundary. |

The actual coordinator and subsystem call order remain authoritative when this
summary omits a nested operation. GPU dispatches provide dependencies between
producers and consumers. A workgroup barrier is not a device-wide barrier; do not
use cross-workgroup spin waits as a replacement for ordered dispatches. The host
may submit a prepared dispatch sequence; simulation data and decisions stay on
the device. GPU residency does not require a single persistent megakernel.

## 6. Numeric representation and randomness

Use `f32` initially for behavioral vectors, scores, rates and bounded physical
state where the existing shaders already use it. Keep probability and budget
validation explicit. Reduced precision is a separately measured choice.

The current CPU code mixes `f32` and `f64` balances and ledgers. Plain conversion
of all `f64` values to `f32` is not an accounting design. The proposed portable
cash representation is scaled integer units with a versioned per-currency scale,
checked range and explicit rounding residuals. Use multiword arithmetic when
native width is unavailable and one writer per accepted account update. Debits
and credits use the same quantized amount. Interest accrual retains its remainder.
Select scales from measured smallest meaningful transfers and maximum balances
before enabling this representation; no scale is fixed by this document.

Physical mass, fractional people and work remain continuous quantities with
subsystem-specific finite/roundoff tolerances. Cumulative ledgers may need native
double precision, multiword accumulators or compensated/hierarchical reductions.
Choose and test them against long-horizon range/error bounds. Never use end-of-month
clamping to hide an overspend or a broken conservation equation.

The portable [WGSL type and memory model](https://www.w3.org/TR/WGSL/) does not
provide ordinary `f64`/`u64` scalar types; host-shareable flags and IDs need explicit
representations and layouts. The repository uses `wgpu` 24 and currently requests
timestamp queries where supported, so newer specification features are not assumed
to be available in its backend. A CUDA backend can select different physical
arithmetic and primitives while implementing the same logical contracts.

Use counter-based random draws keyed by world seed, subsystem, stable actor/action
ID, month and draw index. Allocate distinct counter namespaces and archive their
schema. Row ordering, candidate chunking and compaction must not select another
actor's random stream. [Random123](https://github.com/DEShawResearch/random123)
provides an established counter-based approach; its stream construction would need
an implementation and tests on the selected backend. This improves reproducibility
without promising cross-hardware bitwise trajectories.

## 7. Persistence, inspection and failure handling

Checkpoint the authoritative columns, optional-component flags, policy and feature
versions, stable-ID counters/directories, live deltas, pending actions, dated plans,
receipt/consumer cursors, numeric residuals and ecology/history clocks. Persist a
completed boundary marker; reject ordinary continuation from an incomplete coupled
boundary unless an explicit recovery path exists. Derived indexes may be rebuilt
if their construction and visibility are fully specified.

Rust objects can be materialized from GPU tables for legacy exports and focused
debugging. New vector state and schemas require a new archive representation;
an old export may not carry enough information for new-mode continuation. Import
must explicitly translate anonymous populations, validity, ownership kinds,
catalog IDs and numeric residuals. Do not invent ancestry or evidence during import.

Inspect compact entity views, selected proposals and relevant feature contributions
on demand. A useful action explanation includes observed month, eligibility,
score contributions, policy priority, requested/allocated/reserved/completed values,
rejection reason and causal IDs. Narrative generation consumes that data.

Use staging and publication boundaries so rejected candidate generation or failed
capacity checks cannot expose half a committed action. Broader rollback semantics
must be specified for the whole coupled month; the current living path does not
promise rollback of already-submitted ecological work. This document does not
silently replace that contract with an unimplemented full-world transaction.

## 8. Implementation sequence and acceptance

The target is the GPU data model throughout. Temporary CPU reference views support
verification during the conversion; they do not define the final architecture.

| Slice | Concrete deliverable | Acceptance evidence |
| --- | --- | --- |
| 1. Schema and archive | Entity IDs/directories, table descriptors, catalogs, numeric conventions and import/export | Round trips preserve identity, modes, policies, relationships and ledgers; legacy data has explicit defaults |
| 2. Relations and presence | Membership, ancestry, duties, active worklists, base/delta queries | Current lookup precedence, death/travel partitions and same-boundary changes match the declared rules |
| 3. Work and finance | Proposals, requirements, reservations, account transfers and work receipts | Scarcity, unavailable participants, indivisible minimums, joint constraints, cancellation and no double settlement |
| 4. Resident month | GPU needs, care, demographic selection, production integration and retail settlement | Correct nutrition timing, residual population and completed work; no CPU civilization decision calculation in this slice |
| 5. Social/transport month | Markets, relocation, knowledge/institutions, governance, armies, credit and due queues | Same-month causal fixtures, live admissions, prior-month crews, current policy windows and annual transitions |
| 6. Coupled close and history | GPU validation, living returns, logs, checkpoints and debug projections | Monthly/batched/resumed continuation, event-cursor continuity and finite memory under long runs |
| 7. Behavioral vector policies | Versioned decision-family features, heterogeneous preferences, learning/memory transforms | Paired baseline and alternative outcomes on fixed openings, calibration and held-out scenario results |

Stages can overlap in development, but every migrated quantity has one authoritative
writer. A subsystem is GPU-complete when its observations, candidates, allocation,
execution and settlement run from device data; uploading CPU decisions is an
intermediate integration state.

### Correctness and outcome checks

Representation checks cover ID validity, base/delta equivalence before/after
compaction, reverse-index consistency, all-or-nothing joint commitments, due-queue
wraparound, buffer overflow, empty populations, isolated actors and historical
references. Conservation checks include cash by currency, stocks and transit,
source depletion, population partitions, labor and room/freight budgets. Reorder
independent actor storage and vary chunk sizes without changing policy order.

Timing checks cover monthly versus batched advancement, annual boundaries,
checkpoint continuation, current arrivals, live death/succession, retail nutrition,
prior-month crew progress and living environmental returns. Within a backend,
aim for reproducible continuation; if numeric comparisons need tolerances, preserve
structural and causal checks rather than replacing all comparisons with ±5%.

For behavioral evaluation, predeclare the metrics and horizons. Candidate metrics
are population and age distribution, household food access, production, trade
delivered, migration, completed care/institutional work and town survival. Compare
aggregate levels and distributions, including households/towns under scarcity;
a similar global population alone can hide a large distributional change.

Use identical opening states, mode flags and policy settings for paired baseline
comparisons. For stochastic or branching trajectories, report multi-seed means,
dispersion and discrete outcome frequencies. Define absolute bounds for near-zero
metrics and rare events; a 5% relative difference there may be meaningless. Keep
calibration seeds separate from evaluation seeds. Stress tests include famine,
war, disconnected routes, unavailable teachers/carers, depleted source stocks,
empty demand and insufficient money/materials. Report failures rather than tuning
only the visible global totals.

The approximate 5% target applies to selected behavioral outcomes after these
definitions. It is not yet demonstrated. Performance measurements must include
the complete month, feature generation, arbitration rounds, rebuilding, device
memory, log traffic and startup separately. The existing
[history performance profile](history-performance-profile.md) motivates removing
CPU lookup work but supplies no speedup estimate for this design.

Generated benchmarks, archives, profiles and raw results belong under ignored
`output/`; settings, results, failures and limitations belong in Markdown. Follow
the [repository artifact policy](../AGENTS.md).

## 9. Research basis and remaining implementation decisions

The source-to-table mappings above are design proposals for this repository.
External work supports component techniques:

- [FLAME GPU 2](https://eprints.whiterose.ac.uk/id/eprint/199416/) demonstrates a
  GPU agent-simulation framework with agent state and communication. It supports
  the feasibility of individual agents; it does not implement this civilization's
  causal and economic rules.
- [Gunrock](https://arxiv.org/abs/1701.01170) organizes GPU graph operations around
  active vertex/edge frontiers. This informs relationship and route processing;
  inheritance and institutional semantics remain custom.
- [WarpDrive](https://arxiv.org/abs/2108.13976) demonstrates device-resident
  multi-agent simulation and parallel environments. Its benchmark throughput is
  not an estimate for a single, interconnected civilization.
- The discrete-choice, successor-feature, random-number and WGSL references are
  linked at the design decisions they inform. They establish techniques and
  constraints, not behavioral validity or an end-to-end implementation result.

Before implementation, choose backend-specific primitive implementations, account
scales and accumulator widths; specify byte layouts and capacities; enumerate the
exact feature schemas and baseline scoring rules; and define conflict policies
for each migrated resource pool. These are bounded design decisions within this
architecture. A general learned embedding, universal market-clearing algorithm,
new scheduler and multi-GPU partition are not prerequisites.
