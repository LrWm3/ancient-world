# Planner experiments after economics v1

Status: first bounded implementation/comparison published in
[planner comparison](PLANNER-COMPARISON.md), using the
[frozen benchmark](PLANNER-BENCHMARK.md). The [follow-up](PLANNER-CALIBRATION.md)
adds continuation calibration and a viable finite wood/buy control through a
bounded market-order adapter. P0–P3 still have partial coverage: general market
composition, active counterparty planning and forecast calibration remain limited.
The [continuation comparison](PLANNER-CONTINUATION.md) adds a bounded P7 pilot:
monthly search versus observed-deviation repair and horizon reviews, using the
same cheap continuation in forecast and execution. General local plan repair,
selective materiality triggers and market continuation remain open. P4–P6 and P8
remain proposals. The [multi-person follow-up](PLANNER-PERSONS.md) extends P3/P7:
independent searches and review schedules feed explicit admission against shared
resources, retaining equal-score alternatives. Two/four-person repeated harvests
and scarcity controls are exercised; this is not active counterparty market planning
or a general solution to complementary-input allocation. No default was replaced.
Baseline: verified v1 code `a1b99fa`, results published at `b3cbcef`.
This is a bounded research plan, not an extension of the completed
[v1 release checklist](V1-RELEASE.md) or a commitment to implement every technique.

The question is whether an agent can discover useful combinations of our existing
opportunities without a separate strategy for each arrangement. Keep resources,
actors and institutions small while varying prerequisites, timing, scarcity and
information. Compare approaches before selecting a new default.

## What already exists

Reuse [opportunity search](SEARCH.md), [common offers](MARKET-AGREEMENTS.md),
[forecast context](FORECAST-CONTEXT.md), [joint planning](JOINT-PLANNING.md),
[telemetry](TELEMETRY.md) and the [integration boundaries](INTEGRATION-STATUS.md).

- `search::OpportunitySearch` generates read-only candidates with an explicit
  candidate budget. Current plans contain a limited set of typed prerequisite
  steps plus a work proposal; they do not represent arbitrary dated plans.
- `planning::evaluate_candidates` forecasts through existing execution. Current
  continuation uses a static rollout policy rather than recursive search.
- `offers` provides discovery/preparation/acceptance over typed resolvers.
  `acquisition` and ordinary settlement enforce shared opening budgets and timing.
- Explicit household prerequisite/process packages now execute through the
  household allocation envelope. General household search remains unsupported;
  existing guards must remain until a covered adapter replaces each restriction.
- Financial reporting and external observers already supply outcome evidence.
  Planning must not become another settlement or accounting implementation.

The older joint forecast planner is not a decentralized planner. Do not silently
reuse its access to multiple participants as authority for a person to direct
others. A household can direct only its delegated resources and permitted work.

## Research ideas to test

These are adaptations to investigate, not guarantees that published results carry
over to this model.

| Research | Useful idea | Experiment |
| --- | --- | --- |
| [Fox & Long, PDDL2.1 (2003)](https://strathprints.strath.ac.uk/1846/) and [Do & Kambhampati, SAPA (2003)](https://www.scs.cmu.edu/afs/cs.cmu.edu/project/jair/pub/volume20/do03a.pdf) | Durations, numeric resources, deadlines and resource-aware search estimates | P1–P2: dated opportunities and feasible prerequisite chains |
| [Chen & Thiébaux, numeric search portfolios (2024)](https://ojs.aaai.org/index.php/SOCS/article/download/31559/33719/35616) | Different search queues/heuristics can explore alternatives missed by one ranking | P2: beam versus heuristic search; diversity is an optional ablation |
| [Nau et al., SHOP2 (2003)](https://www.cs.umd.edu/~nau/papers/nau2003shop2.pdf) | Reusable task decompositions reduce search, at the cost of supplied domain knowledge | P4: optional generic methods versus primitive search |
| [Kiekintveld et al., Deep Maize (2006)](https://strategicreasoning.org/wp-content/uploads/2010/03/ControllingSupplyChainAgent.pdf) | Projected marginal resource values coordinate procurement, production and sales | P5: planner-derived reservation values and pricing |
| [Walsh & Wellman, supply-chain formation (2003)](https://arxiv.org/abs/1107.0021) | Complementary inputs can strand purchases; allocation and decommitment rules matter | P3/P6: contested inputs, forecast exposure and conditional acceptance |
| [Schut & Wooldridge, intention reconsideration (2001)](https://www.cs.ox.ac.uk/people/michael.wooldridge/pubs/agents2001a.pdf) | Persistence and replanning have costs that depend on environmental change | P7: monthly replanning versus event-triggered plan repair |
| [Silver & Veness, POMCP (2010)](https://papers.nips.cc/paper_files/paper/2010/file/edfbe1afcf9246bb0d40eb4d8027d90f-Paper.pdf) | Sample beliefs and simulated outcomes to plan with hidden information | P8: a small forecast-scenario comparison first; full POMCP deferred |

## Shared design boundaries

Introduce a read-only planning description over existing offers. Proposed fields:

| Information | Meaning |
| --- | --- |
| Identity and authority | Offer/version, actor, performer, beneficiary, membership, rights and household mandate |
| Requirements | Stocks, dated capacities, storage, prerequisites and conditions that must remain true |
| Effects | Dated consumption, outputs, rights, obligations and lifecycle transitions |
| Dependencies | Which effects require other acceptances, allocations or deliveries |
| Knowledge | Observed offer, owned resource, accepted claim, forecast opportunity or sampled outcome; an accepted claim can still default |
| Execution adapter | The existing resolver that prepares and revalidates an actual action |

Derive descriptions from authoritative terms where possible. Check descriptions
against resolver outcomes on small fixtures; duplicated estimates must not become
an independent source of permissions or spendable resources.

Keep search, outcome scoring, counterparty expectations, replanning and market
allocation independently configurable. A new search strategy does not change a
person's preferences, household charter, legal permissions or allocation policy.
Person self-directed policy changes remain excluded.

Candidate plans contain dated intentions and dependency explanations. Search may
reserve resources in its private projection; only ordinary acceptance can create
real reservations and commitments. Preserve current phase visibility, including
explicit financed-purchase exceptions to opening-budget rules. Never treat a
projected sale as current cash or promise another agent's capacity.

Hard feasibility checks cover physical resources, authority and valid execution.
Desired outcomes can remain unmet: a scarcity case must select a lawful fallback
and record deprivation/default, rather than report every possible action invalid
because no plan can satisfy all needs. Scoring and failure consequences remain
explicit policies. Search failures and unsupported adapters remain errors.

## First bounded batch: P0–P3

Complete these four experiments, publish results, then stop for review. Later
experiments are contingent options, not automatic continuation requirements.

### P0 — Freeze a small planner benchmark

Reuse v1 S1 and the smallest existing market/commitment controls. Create a manifest
of exact fixtures and commands before comparing new strategies. Opening terms,
prices, consent and technologies may be supplied; chosen action sequences may not.

| Case | Question and expected evidence |
| --- | --- |
| B1: prerequisite chain | Can an actor discover citizenship → land access → cultivation, while meeting current food/warmth requirements? |
| B2: complementary resources | Does abundant seed without land, or land without seed, prevent a false farming projection? Include a feasible wood/buy alternative and a true scarcity control. |
| B3: dated conflict | Two opportunities fit total labor but overlap at a critical crop stage. Does the planner find the feasible schedule or decline the additional obligation? |
| B4: produce versus buy | With finite buyers and storage, can the agent compare cultivation against wood sales funding later food purchases? Disable the buyer in a paired control. |
| B5: household authority | Repeat prerequisite discovery with two members and delegated labor/stocks. Private resources and consent limits remain binding. |
| B6: contested access | Two independently planning people seek one plot or one seed lot. Both may project success, but only the existing allocator can award it; the loser must use a lawful fallback. |

Start B1–B4 with one planning person and configured finite counterparties; introduce
the household only at B5 and independent competition at B6. Use supported drivers
per case rather than require an immediate universal market merger. Freeze a
24-month horizon for farming cases, sufficient for repeated crops and annual dues;
short cases may stop earlier once the relevant consequence is observed.

**Exit:** the current policies have recorded outcomes for every case, including
unsupported combinations. Tiny deterministic cases have hand-checked witness
plans or bounded exhaustive comparisons over the same action set and horizon.
These are offline test references, never a centralized runtime planner or a proof
of global optimality outside that finite search space.

### P1 — Expose dated opportunities without changing decisions

Describe membership, land access, cultivation, wood collection and the selected
existing food/wood trade driver. Use monthly periods and existing contract terms;
no new industry, solver dependency or scheduler is needed.

Initially run descriptions beside current decisions. Cover start requirements,
ongoing crop work, seed return, annual dues, perishability/capacity expiry where
configured, storage, permissions and observed market admission. Receipt comparisons
must detect stale terms and requirements omitted from the description.

For quantities, use a finite candidate set from existing lots, deficits and resource
limits. Record that discretization in the manifest. Do not enumerate every integer
amount or claim continuous quantity optimization.

**Exit:** accepted descriptions predict the relevant execution effects; missing
authority/resources and stale plans reject atomically. Describing an opportunity
alone changes no state, journal or existing decision.

### P2 — Compare two bounded composition searches

Implement both behind a shared search/evaluation contract:

1. **Beam composition:** keep a bounded set of promising partial arrangements.
2. **Resource-aware best-first composition:** prioritize arrangements using
   estimated prerequisite cost, time to useful output and dated resource deficits.

Generate relevant actions backward from needs/obligations, then validate proposed
arrangements forward. Include continuing existing work and waiting. Keep cheap
optimistic heuristics separate from exact feasibility: an optimistic estimate may
rank a candidate but cannot authorize execution or prove it feasible.

Use the same configured objective, observations, forecast continuation and action
descriptions for both strategies. Retain the legacy policy as a third control.
An optional diversity ablation keeps candidates from different acquisition routes
instead of allowing one route to consume the whole beam.

Proposed starting budgets are 64, 256 and 1,024 expanded nodes, paired with caps of
8, 32 and 128 full forecast evaluations; freeze any calibration changes before
comparison. Also count simulated forecast-months, peak live candidates and actual
runtime. Node counts alone do not imply equal computational cost. Deterministic
budgets govern decisions; wall time is measured rather than used as the replay key.

Use the same 12-month forecast initially, then a 6/24-month sensitivity comparison.
Carry unfinished processes and outstanding obligations into ending-state assessment;
a loan or crop extending past the horizon must not disappear from the score.
Distinguish exhausted search from a proven absence of feasible alternatives.

**Exit:** both searches discover a useful multi-step arrangement without a scripted
chosen package, pass B1–B4, expose budget exhaustion and retain lawful fallbacks.
Publish which succeeds under which budget, including failures. Neither becomes
the default merely because it is more sophisticated.

### P3 — Exercise households and competing plans

Connect selected candidates through the household acceptance envelope for B5.
Keep charter, objective and consent fixed. Remove a compatibility guard only with
a focused mixed test covering the formerly rejected case; do not globally disable
household/consequence-planner restrictions.

For B6, collect independent current requests before existing allocation resolves
them. Run the same opening requests through the selected existing allocation
controls. The search cannot inspect private rival plans or assign their resources.
After rejection, record remaining resources, lost opportunity and fallback work.
Changing an allocation policy is a distinct experiment from changing the planner.

**Exit:** discovery works through delegated resources; no private stock is silently
appropriated, and competing projections create no duplicate rights or grants.
CPU/reference, reordered independent inputs, continuation across acceptance and
separate financial statements agree for the selected strategy. Different strategy
outcomes need not be identical. If household discovery cannot be integrated within
this scope, record that result and retain the guard rather than claiming completion.

## Contingent experiments after the first review

Run only a newly selected bounded subset. Each must have a paired baseline and
result record; a negative result is useful and can end that experiment.

| ID | Experiment | Comparison and decision criterion |
| --- | --- | --- |
| P4 | Reusable generic methods | Add optional acquire-by-purchase, acquire-by-production and acquire-prerequisite methods. Compare primitive search with method-assisted search on identical budgets, including a novel opportunity combination. Retain methods only if they save work without excluding useful primitive alternatives in the benchmark. Avoid occupation-specific scripts. |
| P5 | Internal resource values and pricing | Estimate the incremental plan benefit of one feasible lot or dated capacity unit. First use it only to rank candidates; then test reservation limits with fixed quotes versus existing ZIP. Preserve hard liquidity and need safeguards. Compare realized benefit, accepted volume, stranded inventory and forecast error. Internal value is neither a journal valuation nor a market price. |
| P6 | Complementary purchases and financing | First expose existing exposure: obtaining only part of a useful package. Add one existing tool offer and one configured loan or forward, separately, without changing the search algorithm. Then, only where supported, compare ordinary acquisition with explicit conditional acceptance. Cancellation requires accepted terms and consequences; no free rollback of completed trades. Terms/counterparties remain supplied; underwriting is excluded. |
| P7 | Plan persistence and repair | Compare monthly reconsideration, persistence until invalidation, and event-triggered repair under identical fixed preferences. Trigger on failed prerequisites, threatened needs/commitments or a configured improvement threshold. Measure switching, wasted inputs, default, welfare and search work. A revised intention cannot cancel a binding contract or change a charter. |
| P8 | Uncertain counterparty forecasts | Compare a single point forecast with a small shared set of plausible buyer/price/supply scenarios. Start with full-sale, partial-sale and no-sale hypotheses; freeze weights/robustness rule rather than infer certainty from them. Evaluate unseen realized shocks, adverse outcomes and spending safety. Full POMCP, learned opponent models and neural policies remain deferred. |

P4 follows P2; P5/P6 follow P3; P7 requires persistent dated intentions from P2;
P8 follows a validated deterministic forecasting path. P5 can inform P6 valuations
but neither is a prerequisite for completing the first batch.

## Measurement and decision rules

For each pair hold opening state, opportunities, permissions, supplied consent,
allocation, scoring policy and observations constant unless that factor is the
declared treatment. Use deterministic controls first. When randomized allocation
or shocks are introduced, freeze ten paired seeds (0–9) and hold out a separate
ten-seed set (10–19) from tuning; report every failure, not only the mean.

Record through existing observers, adding a planning observer field only when a
missing distinction prevents diagnosis:

- Outcome: dated food/warmth deficits, condition losses, completed crops/work,
  performed/defaulted claims, real trade and ending assets/liabilities.
- Planning: generated/expanded/rejected candidates, rejection reason, forecast
  evaluations/months, horizon, budget exhaustion and chosen prerequisite chain.
- Coordination: requested versus granted resources, stranded inputs, unfilled
  orders, forecast-versus-realized sales and plan revisions.
- Correctness: shared-resource bounds, no same-window double spending, authority,
  atomic rejection, separate accounts, replay and relevant continuation equality.
- Cost: runtime and peak search storage, separating search/forecast time from
  settlement/accounting where measurements permit. No claim of GPU scalability.

Policy objectives determine tradeoffs; do not collapse every need into coin value.
Count unavoidable default as an economic outcome, while a lost claim or invented
resource is always an implementation failure. Compare ending obligations and
committed work so a planner cannot win by moving costs beyond the horizon.

Retain an experimental strategy as useful if it finds a predeclared beneficial
chain the baseline misses within the same resource and computation constraints,
without correctness failures; record any economic regressions. Recommend a new
default only after the frozen controls and held-out variants show acceptable
tradeoffs. An inconclusive or negative comparison can finish the batch: publish
why and leave the current default unchanged. Passing safety tests alone does not
establish better planning.

## Deliverables and stopping rule

The first batch produces a frozen benchmark manifest, read-only opportunity
descriptions, two swappable experimental searches, mixed household/competition
evidence and one Markdown results report with reproduction commands and sources.
Update the integration matrix to distinguish descriptions, discovery and execution.
Keep raw runs under ignored `output/economics/`; commit only source, tests and
human-readable documentation.

Run focused checks as each adapter changes, then the full economics suite, explicit
population regression, strict Clippy, formatting and repository artifact checks
before proposing a replacement default. Preserve the verified v1 result as a
historical baseline rather than relabeling new code as already verified.

Stop after P0–P3 and their comparison report. Do not automatically add resources,
institutions, a centralized economic optimizer, new financial instruments, a
universal contract interpreter, personal self-government or all later research
options. The review should decide whether the next constraint is discovery,
forecast accuracy, market coordination or computational cost.
