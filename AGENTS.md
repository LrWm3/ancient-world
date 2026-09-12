# Repository artifact policy

Keep commits source-only: source code, editable catalogs, tests, documentation and
human-readable Markdown test summaries. Do not commit experiment binaries,
compressed results, raw logs, trajectories, generated images, or run manifests.
Write generated experiment outputs under ignored `output/` and summarize settings,
results, failures and limitations in Markdown. Existing experiment scripts can
produce local artifacts but those artifacts must not be added to Git.

Run `python3 scripts/check_repository_artifacts.py` before committing.
Continue committing and pushing normal work as requested by the user. The
single-commit history rewrite was a one-time cleanup; do not routinely rewrite
published history.

# Tools for developing simulation systems

Use the existing monthly scheduler and explicit allocation policies to integrate
systems. Treat timing and allocation as separate design choices; do not introduce
a scheduler rewrite merely to change which competing claim receives resources.

## Monthly history phases

Read [the monthly schedule](docs/monthly-schedule.md) and the relevant phase in
`src/civilization.rs` before adding or moving monthly work. The coordinator uses
five stages:

| Phase | Integration responsibility |
| --- | --- |
| Open | Apply due arrivals and policies, establish current observations and availability. State which inputs describe this month versus a previously completed interval. |
| Reserve | Forecast requests, resolve competing claims, and commit bounded work/resources. Pass dated plans to execution rather than recomputing them after other claimants spend capacity. |
| Execute/settle | Execute granted work and settle actual use, outputs and shortfalls. Keep requested, allocated, reserved and completed work distinct. |
| Respond | React to completed outcomes and schedule subsequent actions. Preserve explicit same-month exceptions and causal event visibility; decisions cannot retroactively change completed production. |
| Close | Refresh derived summaries, reconcile and validate the completed boundary, and record history using the existing frozen/living-history rules. |

These are the existing timing contracts, not a requirement to move every legacy
operation into an idealized phase. When changing a subsystem, identify its input
boundary, reservation point, execution/settlement point and when its outputs
become visible. Follow the actual coordinator for due deliveries, prior-month
crew funding and living ecological returns. Events remain attached to committed
actions; do not defer all event creation to Close.

Run each monthly operation once per simulated month, independent of batch size.
Do not reuse stale grants, count transfers twice or backdate released work into
production. For timing changes, add focused boundary tests and check monthly,
batched and checkpoint-resumed execution where relevant. Reordering independent
actors can test accidental iteration dependence; shuffling causally dependent
phases changes the model's rules.

## Explicit allocation policies

Use [the service allocation pilot](docs/service-allocation.md) and
`src/service_allocation.rs` as a pattern for competing claims:

1. Identify the resource pool, its scope and the allocation boundary.
2. Collect feasible requests before either claimant consumes the shared pool.
3. Apply an explicit policy to obtain demand-capped allocations.
4. Match eligible participants and enforce money, material and capacity limits
   before committing actual reservations.
5. Retain receipts showing requests, allocations and reservations; use existing
   work receipts for actual completion and release.

The current pilot covers research and cultural work only. Configure it through
`History.service_allocation.policy`: `ResearchFirst` preserves existing priority;
`Weighted { research, culture }` provides positive relative weights and redistributes
entitlement above demand. Research-first is the default, including old archives;
weighted sharing is opt-in. Policies and latest boundary receipts persist with
history and appear in the service-work report.

Do not infer that all services now share fairly: care and committed crews still
precede this window, workshops/additional crews follow it, and individual
matching retains ordering effects. Site entitlement cannot create a qualified
teacher, available worker, money or materials. The service pool is only a bounded
part of total labor, not the entire workforce.

Extend this pattern selectively. Before applying it to a different pool, define
eligibility, joint funding/resource requirements, minimum useful grants and what
happens to unfillable allocations. Do not force the two-claim implementation onto
unrelated resources or change execution order to express a sharing preference.

For allocation changes, compare policies against identical opening requests,
include inactive-demand and unavailable-participant controls, and verify finite
budgets and continuation consistency. Inspect completed work as well as grants:
more even shares can fail to complete indivisible tasks. Record results and limits
in Markdown; conservation and determinism alone do not establish good balance.

For indivisible work, see institutional election requests in
`src/culture/work_requests.rs`: check the request's minimum useful grant against
both the shared allowance and an eligible participant's availability **before**
reserving. An unfundable election should leave capacity for divisible upkeep;
reserving and later expiring a known-unusable partial grant needlessly starves
other work. Execution still rechecks eligibility and the required grant.
