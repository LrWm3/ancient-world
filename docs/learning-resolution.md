# Research and cultural work reconciliation

Research and cultural work now participate in the shared resolution receipt
framework when History has resolution enabled. Existing allocation, named
participation and execution remain authoritative. This adds work comparisons, not
a new religious or knowledge-diffusion model.

## Boundary and quantities

The reservation window captures site, month, feasible requests, allocated caps,
actual reservations and whether named participation is enabled. Aggregate here
means aggregate service labor; it is independent of the demographic mode. A
history can use aggregate demography and individual cultural participation.

At monthly Close, after personal work settles, the adapter reads the actual
research and cultural plans and records three comparisons in worker-months:

| Metric | Expected | Actual |
| --- | --- | --- |
| allocated_work | Feasible request | Policy allocation |
| reserved_work | Allocation | Actual reservation |
| completed_work | Reservation | Completed work |

The differences identify allocation, reservation and execution shortfalls. They
do not attribute every cause to a particular person, missing material or canceled
action. In research-first policy the final research allocation is reduced to the
matched reservation before spare capacity is offered to culture; its shortfalls
therefore cannot all be classified as policy effects alone.

Research results must be settled. Plan dates and grant amounts must match the
opening allocation. Nonfinite, negative and over-grant results are rejected.
Boundaries use the subsystem, month, site and opening work quantities. Duplicate
receipt commits fail. The adapter stages the receipt batch before replacing state,
so a failure cannot leave half a batch of comparisons.

Receipt recording never repeats production, teaching, payments or mortality.
The existing subsystem ledgers own those transfers. Zero-demand categories emit
no receipt. The latest input window is saved in service allocation state; old
receipts without a captured resolution mode are skipped rather than inventing
earlier comparisons.

## Controls and interpretation

Use the existing resolution API and compare flag described in
[the framework guide](resolution-framework.md). With comparison disabled, guarded
receipts have no metrics. Enabling comparison records bounded running summaries
and the latest month's receipts. Both modes execute the same actions.

These are conditional work expectations, not independent predictions of knowledge,
beliefs or future productivity. Only demography currently has replayable
same-input aggregate/individual outcome snapshots. [Contact and pilgrimage](informal-learning.md) now share partial learning progress.
Research and culture receipts do not yet replay alternative participant assignments,
compare knowledge outcomes between channels, or
resolve competition for individual teachers simultaneously.

For sharing rules see [service allocation](service-allocation.md). For dated
learning and provenance see [individual participation](individual-participation.md).

## Verification (2026-09-11)

- Regular library suite: 100 passed; 90 hardware-dependent tests ignored.
- Analytical tests distinguish allocation, reservation and execution differences;
  reject nonfinite and over-grant results; and preserve duplicate-commit guards
  through serialization.
- The service allocation fixture runs seeds 17, 81 and 256 with an actual teaching
  opportunity and finite specimen processing. It executes research and culture,
  requires positive completed work for each, and checks their three metrics.
- Running those same actions with comparison metrics enabled versus disabled
  produces identical history after excluding resolution bookkeeping: inventory,
  people, events and learning are unchanged. A repeated receipt commit fails
  without partially changing the comparison state.
- Full frozen-history scheduler comparisons use the aggregate demographic resolver
  with named participation, comparison enabled on seeds 17/256 and disabled on 81.
  A 24-month batch matches single months, followed by 12 months matching a resumed
  5+7-month sequence. Checks include serialized history and terrain/ecology state.
- All-target Clippy with warnings denied passed.

Reproduce with:

```sh
cargo test --lib
cargo test --lib service_allocation -- --include-ignored --nocapture
cargo test --test history_environment frozen_schedule_batch_and_checkpoint_equivalence -- --ignored --nocapture
cargo clippy --all-targets -- -D warnings
```

Zero unexplained work residual follows from the stage partition; it does not prove
that the behavioral rules or allocation weights are good. This is integration
evidence, not long-run calibration of education or religion. Learning-channel
comparisons, action-specific completion forecasts and individual worker integration
for other sectors remain separate follow-ups.


## Successor lesson outcomes

Successor teaching now captures the selected student/topic, opening progress and
the expected gain and acquisition from one requested lesson before allocation.
The projection uses the same partial-learning rule as execution on a copy of
the student's progress. It is conditional on the lesson receiving work and
remaining eligible, not a prediction that all requests will succeed.

Execution measures progress immediately before and after that specific lesson.
Only that increment enters `successor_learning_gain`; `successor_acquisition`
records whether the lesson completes the topic. Canceled, unfunded or displaced
actions retain zero actual outcomes. Earlier contact or another study channel
cannot be misattributed as successor-teaching output. Work comparisons remain
separate, in worker-months; progress uses topic fractions and acquisition uses
completed topics. Differences are retained rather than assigned a speculative
explanation or converted into additional teaching.

The dated WorkPlan persists these values, with absent expectations for older
archives. Compare-off mode retains the same action execution and work plans but
omits summary metrics. This first action-specific comparison covers successor
teaching only: manuscript/institutional study, research processing, heritage
interpretation and other cultural actions still need their own output contracts.


Verification: the regular library suite passes 114 tests (103 hardware tests
skipped). The four service-allocation tests, including the three-seed GPU
fixture, pass with expected and delivered successor gain checked separately
from labor. Compare-on/off histories remain equal after excluding resolution
bookkeeping. The cultural request GPU fixture checks absent funding, actual
progress and saved outcome fields. A CPU projection fixture checks read-only
behavior and completion from prior partial study. All-target Clippy is clean.
The full frozen-history checkpoint/batch fixture is being rerun separately.
