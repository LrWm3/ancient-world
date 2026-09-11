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
omits summary metrics. This increment first covered successor teaching; subsequent study and research
contracts are described below. Heritage interpretation and other cultural actions
still need their own output contracts.


Verification: the regular library suite passes 114 tests (103 hardware tests
skipped). The four service-allocation tests, including the three-seed GPU
fixture, pass with expected and delivered successor gain checked separately
from labor. Compare-on/off histories remain equal after excluding resolution
bookkeeping. The cultural request GPU fixture checks absent funding, actual
progress and saved outcome fields. A CPU projection fixture checks read-only
behavior and completion from prior partial study. All-target Clippy is clean.
The full frozen-history checkpoint/batch fixture also passes after this change,
including its three-seed monthly, batched and resumed comparisons.


## Institutional and object study outcomes

Requested study now captures the same conditional progress/acquisition projection
as successor teaching, plus its source: a surviving local object's ID, or a
local teacher and institution. Forecast and execution share the readable-object
selection helper. A readable object keeps precedence over institutional teaching;
this increment records that existing rule rather than changing it.

`study_learning_gain` and `study_acquisition` enter the same cultural receipt,
separately from successor metrics. Actual study is measured immediately around
the action and recorded only for the projected student, topic and source. Work,
fees, teaching experience, acquired knowledge and events still commit through
the original action. Missing staff, lost sources and insufficient work leave
zero actual learning. Old plans default to no study projection.

The projections describe individually feasible requests before competing actions
spend their shared grant; their sum is not a guaranteed joint outcome. A scarce
0.125-worker-month cultural grant can fund study but not the following 0.1-month
successor lesson. Their separate outcomes expose this ordering consequence while
the ordinary labor receipt still accounts for unused work. Heritage interpretation and other cultural actions remain separate follow-ups.
Research processing outcomes are described below.


The four service-allocation tests pass, including three GPU seeds and a two-action
scarcity case: funded object study delivers its projected gain, while the
following unfunded successor lesson records zero. Comparison-on/off histories
remain equal after removing resolution bookkeeping. The institutional GPU fixture
checks captured teacher/institution identity, absence, zero funding and delivered
progress, alongside the existing source/provenance checks. Regular library tests
pass 114 cases (103 hardware tests skipped); all-target Clippy is clean. The full
three-seed frozen-history fixture also passes monthly, batched and checkpoint
continuation comparisons with the new study outcomes.


## Specimen research outcomes

Research plans now retain requested and actual resin/crust study, remedy output
and phosphorus release in kilograms, plus acquisition of each processing method.
The read-only forecast consumes shared tools, fuel, writing supplies and work
locally; its outcomes are conditional on receiving the requested effort and
retaining those supplies and teaching contacts. Allocation does not erase unmet
expectations.

Execution records the actual study, method acquisition and physical processing
output at the existing transfer points. The comparison adapter only reads these
results at Close; it does not perform another transfer. Method copying and
destructive study remain distinct from manufacturing. A failed teaching contact
can leave study possible instead of the expected manufacturing, so actual output
need not occupy the same category as the forecast. Differences remain unexplained
unless a mechanism explicitly attributes them.

The research receipt now holds three work metrics and six outcome metrics. Its
bounded validator permits nine metrics for research; other receipt limits are
unchanged. Old research plans without outcome fields remain readable and do not
receive invented historical comparisons. Botanical comparisons are extended below; heritage actions and comparisons
between learning channels remain unfinished.

Verification for specimen outcomes: the GPU research fixture passes shared-tool
scarcity, post-reservation tool loss, closed teaching contact and known-method
production cases, including serialized outcomes. All four service-allocation tests
pass, including three GPU seeds checking study mass against completed effort and
unchanged histories with comparison disabled. The regular library suite passes
114 tests (103 hardware tests skipped); all-target Clippy is clean.
The full frozen-history fixture also passes its three-seed monthly, batched and
checkpoint-resumed comparisons with these outcomes present.


## Botanical study and application

Botanical plans distinguish destructive study from application for bast, pigment
and planting samples, using kilograms of input. Actual use is captured at the
existing material transfer, never inferred from a later inventory difference.
Application input is deliberately not a promise of output: a planting trial can
consume samples and labor while producing no seed in unsuitable habitat. The
existing botanical output and C/N/P ledgers retain that distinction.

Application requests now require the same destination catalog good as execution.
Missing goods leave collections untouched rather than reserving work for an
unexecutable application. Study remains possible before a destination exists.
Shared tools, fuel and requested effort still bound all research claims. Research
receipts now allow fifteen metrics; bounded summary capacity is 128 to accommodate
the additional names across aggregate and individual modes. Other receipts remain
bounded at eight metrics.

Older plans without the botanical capture marker omit these six comparisons,
rather than presenting zero as an observed result. The targeted GPU fixture checks
planned versus actual study/application masses over four months, and retains its
zero-work, unsuitable-habitat, resumed execution and material accounting cases.
The three-seed service-allocation fixture still checks comparison-on/off history
equivalence. Regular library tests pass 114 cases, with 103 hardware tests skipped.
Missing-catalog application requests and absent legacy capture markers also pass
the GPU fixture; all-target Clippy is clean after the compatibility change.
The full three-seed frozen-history monthly/batch/checkpoint fixture passes with
botanical comparisons enabled.
