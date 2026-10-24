# First boundary for embedding the economics engine

This increment adds a checked, headless single-site driver around the existing
simulation and financial audit. It is preparation for an Ancient World settlement
pilot. It does **not** yet import a real settlement, replace the main economy,
couple ecological stocks, or provide a complete serialized checkpoint.

`host::Settlement` owns an existing `Simulation` and `Audit`. Attachment requires
a local Open boundary and a matching, reconciled book. Attachment may occur after
startup: it retains ongoing work, historical costs, accepted contracts and decision
history. Local agreement dates remain local. A separate positive host month and
site identify every call and receipt; advancing the host calendar does not rewrite
contract maturity dates.

## State and resource contract

The experiment remains the sole writer of its economic balances, obligations,
processes and books. Callers get immutable views. Completed receipts are derived
observations, not transfer instructions: applying their stock changes back to the
same engine would double-count settlement. Host site summaries will eventually
need to be derived from these authoritative accounts.

Each opening supplies exactly one capacity limit for every existing participant,
including explicit zero availability. Unknown people, missing entries, duplicate
entries, wrong resource IDs, negative quantities and grants above the configured
endowment are rejected before publication. An existing scenario override is
intersected with the host limit, including an override above the catalog ceiling.
The ordinary condition modifier can reduce capacity further. Bought labor remains
an ordinary transfer, not another personal endowment.

Inputs are in the experiment's existing integer resource units. The harness's
site ID is a supplied label, not an imported Ancient World identity. Physical kg,
worker-month and currency conversions are not inferred. The future importer must
define stable identity mappings, exact scales, rounding/remainders and the single
owner of every imported stock before moving resources.

The caller must have exclusive authority over the supplied capacity. Merely
reading a site's total workers does not reserve them against care, crews, culture
or other services. The main engine's allocation windows still need an explicit
integration policy; this API does not establish one. Autonomous demography is
rejected, including configurations with new births disabled, because existing
children may still mature during Open and invalidate the host's capacity binding.
Existing terminal/condition consequences retain their normal behavior.

## Timing and visibility

The grouping was checked against [the main schedule](../../docs/monthly-schedule.md)
and `src/civilization.rs::history_*_month`. It preserves the experiment's current
execution order and does not change the main coordinator.

| Host-facing call | Existing experiment phases | Visible result |
| --- | --- | --- |
| `open` | Open, then optional Due | Current endowment and opening obligations settle; existing organizational reviews retain their timing |
| `reserve` | Optional Acquire | Existing acquisition, hiring and dated-plan admission; ends before Productive |
| `execute` | Productive, optional ClearArrears, Consumption | Actual work, deliveries, shortfalls and consumption; ends before Close |
| `respond` | No engine phase | Explicit observation boundary after execution; no production rerun or invented response pass |
| `close` | Close | Ordinary wages, maintenance consequences and monthly reports; both calendars advance exactly once |

Reserve is not a universal production reservation: configurations that skip
Acquire have an empty Reserve receipt and retain their existing Productive-phase
planning. The maintained/skilled fixture exercises its ordinary acquisition and
dated work path. Event/transaction creation stays at the original committed phase.

Capacity inputs reuse dated `capacity_overrides`. They affect this month's actual
Open regeneration, and post-Open forecasts observe them. Organizational reviews
inside Open retain the existing forecast rule that excludes unopened overrides;
they can still forecast catalog capacity during that review. Future host inputs
are never placed in the forecast. Distinguishing already observed host grants from
unpublished fixture events is an outstanding forecast adapter requirement.

This grouping is not permission to call it blindly beside all main-engine work.
The main Reserve stage currently grants several competing services, while this
pilot requires its own capacity limits at Open. A real settlement integration must
resolve that grant/observation boundary explicitly and preserve due arrivals,
prior-month crew funding and existing same-month exceptions. Main demographic
settlement must consume economic outcomes without independently reproducing the
experiment's population or deprivation effects.

Each staged method validates site, date and expected stage, works on a private
copy, and publishes the simulation, audit and driver cursor together. An error
leaves the original boundary unchanged. `advance_month` additionally makes the
five calls atomic as one unit. Successful earlier staged calls remain committed
when a later staged call fails; joint rollback with a future host requires a
joint host/engine transaction.

Stage receipts identify half-open committed batch-ID ranges. Closing receipts
contain changed stock accounts, actual productive completions and completed need
outcomes. They do not collapse requested, allocated and completed work: the
original ledger receipts remain available through the immutable simulation view.

## Headless harness and checks

Run from `exp/economics`. Initial agents, inventories, equipment, institutions and
prices are the supplied [skilled-workshop](SKILLED-WORKSHOP-ACCEPTANCE.md) fixture.
Local month one corresponds to host month 101. Standard runs grant catalog
capacity; the interruption removes the named workshop worker's own endowment for
one local month and restores ordinary availability the next month.

```sh
mkdir -p ../../output/economics/host-boundary
cargo +1.92.0 test --locked --release -j8 --test host
cargo +1.92.0 run --locked --release -j8 --example host_boundary -- --months 12 \
  > ../../output/economics/host-boundary/baseline.jsonl
cargo +1.92.0 run --locked --release -j8 --example host_boundary -- \
  --months 8 --unavailable-local-month 3 \
  > ../../output/economics/host-boundary/worker-interruption.jsonl
```

The focused tests compare ordinary execution with the staged CubeCL CPU driver
and monthly Reference driver, including world, state, ledger, reports, books and
receipts. They exercise mid-month in-memory reconstruction, annual timing, a
worker interruption and recovery, reversed independent inputs, later attachment
with active production and historical costs, an absent Acquire phase, stale and
duplicate calls, malformed grants, clock overflow and accounting rollback after
feasible physical execution. Both excess scenario capacity and demographic
rebinding have explicit rejection/bounding controls.

Verification on 2026-10-23 used Rust 1.92.0, the locked dependencies and release
builds. The twelve-month ordinary/Reference/CubeCL CPU comparison passes with
eight refining completions, zero food deficits and zero wage arrears. World,
state, ledger, reports and financial books agree exactly, including the annual
boundary. The initial six-test run took 447.53 seconds while other checks were
running; this is correctness-test elapsed time, not engine throughput.

All ten host tests passed across the verification runs: the twelve-month
comparison and nine focused controls. The final nine-control run took 85.28
seconds. The month-three worker interruption produced no workshop completion,
practice or delivered workshop hours, and workshop work resumed during the next
five months. The controls also pass both capacity-review regressions, later
attachment with active costed work, missing Acquire, invalid/stale inputs and
physical-execution/accounting rollback.

The two-month CLI smoke run emitted ten stage receipts for host months 101–102
and local months 1–2, three productive completions and zero need deficits. The
output was parsed to check its dates and stage sequence. Strict all-target
release Clippy, formatting and the repository artifact check pass. This is
focused validation, not a rerun of the experiment's entire suite or a new
48-month readiness certification.

Independent review identified two real capacity escapes before commit: an
above-catalog scenario override could exceed a full-ceiling host grant, and
demographic maturation could change a zero-capacity participant after validation.
Both are now bounded/rejected with regression controls. An added accounting
rollback test initially expected the wrong error wording; its assertion now
checks the actual explicit-accounting-adapter rejection. Review found no remaining
runtime blockers after the fixes and scope documentation.

## Remaining replacement gates

1. Import one real settlement with stable person/household/organization identities
   and declared unit conversions. Define which engine owns population, physical
   inventories, money and economic deprivation outcomes.
2. Integrate dated environmental availability and jointly bounded labor into the
   actual five-stage coordinator; return actual physical use and production to
   the ecological ledgers exactly once. Include shared-service scarcity and due
   cargo controls. The host's existing production/consumption must not also run
   for resources assigned to the replacement engine.
3. Persist the complete engine, audit and driver cursor and verify fresh-process
   load/continue. The serializable opening/result records added here are boundary
   DTOs, not a save format. Clone-based tests do not satisfy this gate.
4. Establish per-month time/memory limits and representative population/history
   benchmarks. This driver intentionally uses cloning for atomicity; it is not a
   scaling optimization or evidence that full-world forecasts will scale.

All generated output stays under ignored `output/`. This increment adds no
dependency from the main engine to the experiment and changes no production
default or scheduler allocation policy.
