# Run health: growth limits and subsystem behavior

Status: proposal, grounded in the current evaluation tools. This document proposes
incremental diagnostics; it does not report a new experiment or implement checks.

The aim is to answer two questions for a particular run, seed and interval:

1. Are civilizations growing, stalling or declining before an explicit limit, and
   what is preventing further development?
2. Does each enabled subsystem ever act, does it act when its conditions call for
   action, and does it produce outcomes consistent with its contract?

Health here means accountable game behavior. A stable population, an inactive
subsystem or a failed settlement can be an expected outcome. Passing conservation
checks alone does not establish good balance; reaching a cap is not a success
criterion. Start with separate findings for run validity, growth and subsystem
behavior, rather than a single score that hides failures or missing coverage.

## Build on existing evidence

| Existing component | Reuse | Missing piece to add |
| --- | --- | --- |
| [History evaluator](../examples/history_evaluate.rs) and [evaluation guide](history-evaluation.md) | Seeded generation, annual population and settlement samples, production summaries, events, residuals and optional worlds | Sustained-stall analysis, cap metadata, per-civilization grouping and selected monthly decision observations |
| [Build wrapper](../scripts/build_history_evaluator.py), [integrated runner](../scripts/integrated_history.py), [analyzer](../scripts/analyze_integrated_history.py) | Frozen executable, provenance, exact commands, checksums, completeness checks and seed summaries | A common health report layered on validated results; retain partial-run diagnostics separately |
| [Integrated century diagnostics](integrated-calibration.md) and [pressure sweep](../scripts/pressure_sweep.py) | Baseline/stress comparisons, population loss, abandonment, food stress, freight and service mediators | Detect when decline or a plateau begins and connect it to local constraints |
| [Evidence runner](../scripts/evidence.py) and [model evidence](model-evidence.md) | CPU/GPU verification profiles, accounting checks, controlled interventions and explicit coverage limits | Link a run finding to the relevant mechanism fixture; distinguish unexecuted tests from passes |
| [Settlement audit](../scripts/settlement_audit.py), [economy report](../scripts/economy_report.py), [waterworks report](../scripts/waterworks_report.py) and other focused reports | Existing subsystem-specific observations and lifecycle checks | Small adapters with shared run/site/month identifiers and comparable result states |
| [Service allocation receipts](service-allocation.md), [credit capacity diagnostics](credit-capacity-diagnostics.md), [service-order observations](service-order-shortfall-observations.md) | Dated requests, grants, reservations, completion and some limiting constraints | Explain non-submission and pre-request demand; preserve simultaneous blockers and unknowns |
| [History replay](../examples/history_replay.rs) and existing boundary tests | Same-backend monthly, batched and checkpoint continuation checks | Verify that added observations do not change simulation outcomes or duplicate monthly counts |

Annual samples can screen long trajectories now. They cannot establish that a
system never activated between samples. Latest receipts are not a complete history;
events record selected committed actions and do not explain every rejected or
unsubmitted request. Old fields absent from archives must remain unknown.

## Identify the run before judging it

Every report should carry a run ID, seed, source/build identity, catalog hashes,
resolved system settings and policies, terrain/ecology resolutions, generation
epochs, history mode, demographic authority, backend and absolute month interval.
For resumed runs, include checkpoint identity and the configuration changes at
resume. A seed alone does not identify an experiment. Application startup defaults,
evaluator flags and saved settings must not be assumed equivalent; see
[system options](system-options.md).

Reuse existing manifests under ignored `output/`. Add a version for the diagnostic
schema and rule configuration so a changed detector can be distinguished from a
changed simulation. Record observation cadence, missing intervals, event retention
and whether counters are cumulative, interval totals or boundary snapshots.

Validate completeness, chronology, checksums and finite/accountable state before
balance interpretation. Preserve failures, interrupted runs and unsupported
schemas in the suite inventory. Show available evidence from partial trajectories,
but exclude them from completed-run rates and label their intervals explicitly.
An endpoint save supports endpoint checks; historical activation needs retained
evidence or a reproducible rerun, not reconstruction from ending balances.

## Gross health: find stalls, then investigate constraints

### Make “before the cap” explicit

The current history code has several different limits in
[`src/civilization.rs`](../src/civilization.rs): `MAX_CIVILIZATIONS` is 16, while
`LIMIT` is 256 site records. Daughter founding checks `self.sites.len()` against
`LIMIT`; abandoned records therefore still consume that limit. Neither number is
a population carrying capacity. Candidate geography, founding eligibility,
housing, food and productive capacity impose different constraints. History-month
and event limits can also terminate observation without representing a growth cap.

Report each relevant limit with its value, units, scope, current usage and source.
For site expansion, show total records, active sites, abandoned sites, remaining
slots and eligible destinations separately. Attribute the shared world limit as
shared; do not give every civilization 256 independent slots. A daughter settlement
is not an additional founding civilization.

Use “plateau below the site-record limit,” “candidate exhaustion,” or “population
plateau with unknown carrying capacity,” as appropriate. If an experiment supplies
a target population, label it as a diagnostic target and explain its derivation.
Do not infer an intended population cap from a past maximum or food stock alone.

### Screen trajectories at three scales

Compute world, civilization and settlement trajectories. World totals can hide
local collapse offset by expansion elsewhere. Keep stable identities and explicit
ownership transitions so conquest is not mistaken for demographic growth; report
births/deaths and transfers separately where the data supports them. Account for
people away on journeys or campaigns consistently with existing population ledgers.

The first annual screen should show initial, peak and final population, recent
growth, active and ever-founded sites, new foundings, abandonment/reoccupation,
food stress and recovery. Include counts and denominators, distributions across
towns and the worst affected examples. Falling shortage counts after depopulation
are not automatically improving food access.

A starting detector to calibrate, not a model rule: after 20 years of warm-up,
flag a population plateau when the last ten annual observations span at most 5%
of their positive mean. Flag expansion stalls separately when no new site is
founded for ten years while record slots remain. Also report sustained decline
and large oscillations, which a flat endpoint difference would miss. A run may
have population growth with stalled expansion, or the reverse. Zero population
is collapse, not a plateau; insufficient history is insufficient evidence.

Store these diagnostic thresholds in the eventual analyzer's versioned
configuration. Test sensitivity to window length and tolerance before treating
alerts as regressions. Mark the first qualifying observation and the full window;
annual data does not locate an exact monthly onset. A still-growing run at its
last sample has no observed plateau yet, not proof that it will reach a cap.

### Turn each alert into a constraint investigation

For a flagged interval, inspect the affected sites before, during and after it.
Rank candidate explanations by observed blocked opportunities, persistent
shortfalls and affected population. Keep multiple constraints when they bind
together; do not fabricate additive percentages of causal responsibility.

| Candidate limit | Evidence needed at the relevant boundary |
| --- | --- |
| Expansion eligibility/geography | Site slots, unused reachable candidates, origin population/food checks, flood restrictions, destination threshold and settler availability |
| Food supply or access | Ration need/eaten, stocks and production, household purchasing shortfall, crop inputs/weather, actual relief and deliveries |
| Workforce or allocation | Effective available labor, demand before funding caps where observable, earlier commitments, allocation policy, personal eligibility and completed work |
| Materials, tools or productive assets | Requested recipes/work, opening inputs, installed versus operating capacity, tool sufficiency and actual outputs |
| Money or financing | Dated cash, protected balances, obligations, requests, grants and transfers; distinguish unsubmitted demand from rejected credit |
| Distribution and transport | Feasible buyers/suppliers, market/route access, reserved capacity, blocked dispatches and arrivals; idle freight alone proves little |
| Demography and service deterioration | Birth/death flows, age structure, illness, household nutrition, housing/water coverage and completed maintenance |
| Knowledge or institutional continuity | Qualified participants, holders lost, elections/teaching due, minimum useful grants, completed duties and recovery |

Emit a short evidence chain, for example: “expansion stalled while slots remained;
origins repeatedly failed the food provision check; household food shortfalls
persisted; the production-input boundary was not observed.” This narrows the
question without claiming a proven root cause.

Use matched checkpoint branches or paired seeds to test a leading explanation.
Change one bounded policy/resource condition, retain the same opening state and
requests where applicable, and examine the immediate mediator before long-run
population. A funded intervention should first change the constrained work or
delivery. Later divergent histories alone do not isolate causation. If one change
reveals another limit, retain that chain and only then test an interaction. Existing
pressure sweeps and counterfactual fixtures provide the pattern.

## Subsystem health: opportunities, actions and outcomes

Add a small registry of check definitions, beginning with a few subsystems. Each
definition names its owning code/docs, enabled prerequisites, unit of observation,
decision cadence/phase, trigger, feasibility conditions, permitted delay and
completion evidence. Separate a deterministic obligation from a discretionary or
stochastic opportunity. “Expected” must mean a stated contract or calibrated
conditional expectation, not a desired count of wars, loans or expeditions.

Track the applicable funnel:

`enabled → trigger/opportunity → eligible → requested → allocated → reserved → executed → outcome`

Some systems skip stages. Keep stage units explicit: requests, worker-months,
cash and completed tasks are not interchangeable. Observe the pre-request gate
when needed: an already cash-capped request cannot reveal unaffordable demand.
For delayed actions, match requests and outcomes by identity and due month;
unfinished work not yet due is pending, not failed. Track partial completion,
release, rejection and expiry separately.

| Finding | Meaning |
| --- | --- |
| Disabled / not applicable | Configuration or structural prerequisite excludes this check |
| Unexercised | Enabled and observed, but no triggering opportunity occurred |
| Expected inactivity / blocked | A trigger occurred, but a recorded eligibility or resource condition prevented action |
| Expected activity | Due action and completion agree with the contract, including permitted partial outcomes |
| Unexpected inactivity | A due deterministic obligation with satisfied preconditions did not act, or a defined conditional-rate check fails |
| Unexpected activity / outcome | Action violates a prerequisite, timing or budget contract, repeats a boundary, or fails an explicit settlement rule |
| Unknown / pending | Required evidence is missing, or the outcome is not due yet |

Keep these findings per check/interval rather than forcing a whole subsystem into
one exclusive label. Separately flag persistent expected blocking as a balance
concern: lawful starvation can explain poor gross health. “Ever activated” should
include first/last activation, count and completed outcome, with opportunity counts,
conditional rates, latency and longest eligible inactivity interval. Zero eligible
opportunities gives an undefined rate, not 100% success. For stochastic decisions,
use enough opportunities and uncertainty bounds; one missed opportunity is not a
failure unless the contract guarantees action.

Good first checks are daughter founding, food access/production, and research or
institutional work. They connect directly to expansion and have identifiable
constraints. For institutional elections, for example, eligibility includes a
qualified available participant and the minimum useful grant; a partial unusable
grant is not successful activation. Later add credit non-submission, transport,
waterworks and rare event systems using their existing reports and fixtures.

Instrumentation must follow the [monthly schedule](monthly-schedule.md): observe
inputs in Open or the subsystem's actual input boundary, capture requests and
reservations in Reserve, actual use in Execute/settle, and due reactions in Respond.
Close can aggregate receipts but cannot reconstruct opening cash or move event
creation. Preserve quarterly/annual cadence, previous-month crew funding and living
history semantics. Emit each monthly observation once regardless of dispatch batch.
Prefer bounded counters plus selected traces over retaining every actor every month;
truncated traces must still disclose their coverage. Observation must not consume
resources, draw random numbers or change actor ordering.

## Iterative delivery

1. **Annual gross screen using existing outputs.** Add an analyzer alongside the
   integrated analyzer, reusing its validation. Produce one report for a selected
   run/seed and interval, plus a suite table of stalls, decline, cap usage where
   known, likely constraints and missing evidence. Begin with the established
   17/81/256 baseline and harsh profiles. Synthetic trajectories should cover
   growth, flatness, decline, oscillation, extinction and incomplete samples.
   Done when every alert cites its interval and raw observations without claiming
   monthly activation or unsupported cap headroom.

2. **Instrument the first unexplained stall and three subsystem funnels.** Add
   only the missing cap, founding, food/production and service-work observations.
   Extend the evaluator's sampling and adapters instead of building a second
   simulation runner. Test positive activation, no trigger, unavailable participant,
   scarce resources, minimum useful work and delayed completion. Verify unchanged
   simulation state with observations enabled/disabled and monthly/batched/resumed
   consistency. Done when inactivity in the pilot systems is either explained by
   dated evidence or explicitly unknown.

3. **Investigate one dominant constraint.** Reproduce a flagged checkpoint, add a
   targeted trace around the stall and run one matched intervention plus an
   inactive-demand or abundant-resource control. Follow requests through actual
   completion and population/service outcomes. Use the existing build provenance
   workflow. Done when the report supports or rejects a specific causal hypothesis,
   records remaining limits, and adds a focused regression test for any real bug.
   Observation changes and balance changes should remain separate changes.

4. **Broaden coverage and calibrate alerts.** Register additional subsystems from
   the [maintenance inventory](system-maintenance-inventory.md), prioritizing those
   implicated in stalls and those never exercised. Use controlled fixtures for
   rare mechanisms and natural runs for activation frequency. Seed 409 already has
   substantial historical coverage; use it as a regression case and reserve fresh
   untouched seeds for new held-out decisions. Include longer histories and a
   production-resolution spot check before generalizing from resolution 64.
   Done when the coverage matrix distinguishes tested behavior, natural activation,
   unexplained inactivity and systems still lacking observations.

5. **Make comparisons routine.** Keep cheap deterministic contract checks in CI;
   run GPU seed suites on supported hardware for periodic/release comparisons.
   Initially make heuristic balance changes review alerts, while malformed runs
   and deterministic contract violations fail checks. Promote a balance alert to a
   gate only after its thresholds and false positives are understood across seeds.
   Compare matched configurations and retain per-seed regressions rather than
   accepting an ensemble average that hides a collapse.

The report should open with run validity, growth/cap findings and the most useful
next investigation. Follow with a subsystem coverage table and a few concrete
site/month examples linked to local evidence. Keep generated JSON, logs, worlds,
plots and manifests under ignored `output/`; commit the analyzer/tests and curated
Markdown summaries of settings, findings, failures and limitations. This proposal's
first useful deliverable is an explainable annual screen, followed by targeted
observations where that screen cannot answer why.
