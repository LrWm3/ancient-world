# Contact and pilgrimage learning

Annual contact and successful pilgrimages now add partial practical learning
instead of granting a complete topic immediately. They use the same saved study
progress as paid lessons. Only completed topics enter practical knowledge.

## Channels and effort

A paid lesson contributes 0.1 learning input and can benefit from the existing
teacher-experience support. Annual contact contributes 0.025; observation during a
successful pilgrimage contributes 0.05. Informal channels get the learner's
curiosity effect but no paid-instruction bonus. With no curiosity or support,
these correspond to three lessons, twelve annual exposures or six visits from
zero progress. These are game pacing constants, not measured learning rates.

Informal input is an exposure proxy, not extra worker-months. Contact does not
claim that a named teacher spent funded instruction time. Pilgrimage retains its
existing travel, provision and offering costs; incidental observation does not
add a second lesson payment or instruction-experience reward. Paid instruction
remains the supported channel for faster deliberate learning.

One person receives at most one informal exposure per month across topics and
both channels. Duplicate passes, many incoming routes or a pilgrimage followed
by contact cannot multiply that allowance. The earlier eligible channel uses it;
this does not introduce simultaneous selection among learning opportunities.

Contact reads teachers' completed knowledge and its provenance at the opening
of the annual contact pass. A student who completes a topic cannot immediately
relay it through a second town during that same pass. Existing route/flood checks,
recent delivery evidence and the cross-faith contact delay still apply.
Scholarly membership offered by this contact path requires completed learning.

## History and persistence

Partial contact emits knowledge_contact_progress; completion retains the
knowledge_contact event kind. Both identify the teacher and link delivery evidence,
the teacher's source and prior study where available. Pilgrimage returns record
the topic, progress and whether practical access was acquired, with the same
source chain. Partial progress from one channel can be continued by another.

The per-person last-exposure month is serialized with the agent. Older agents
default to no recorded exposure. Existing knowledge is never revoked or converted
back into partial progress. Future learning trajectories change under the new rules;
exact continuation of the former instant-contact behavior is not promised.

## Limits

This does not make informal contact a personally staffed teaching service.
First eligible resident selection and the existing contact geography remain
coarse. Founding assistance and other special knowledge sources retain their own
rules. More gradual spread may create knowledge bottlenecks; long-run balance
needs evaluation separately from the boundary checks here.

For paid learning see [knowledge succession](knowledge-succession.md); for work
comparison receipts see [learning resolution](learning-resolution.md).

## Verification notes (2026-09-11)

The regular library suite passed 101 tests, with 91 hardware tests ignored.
All-target Clippy passed. The three-seed frozen scheduler check matched monthly,
batched and checkpoint-resumed histories under the new learning rules.

Focused checks cover:

- Partial informal progress, cross-topic/month exposure limits, serialized guards,
  mixing paid and informal study, and eventual practical completion.
- Flooded versus open routes, recent versus stale delivery evidence, repeated
  annual calls, and reopening without instant mastery.
- A three-town chain in which the middle learner completes a topic this year,
  but the distant town only begins learning during a later annual update.
- Pilgrimage progress and source recording, alongside existing finite travel,
  offering, closure and recovery checks.
- Existing conversion, syncretism, specimen-contact and naming-contact regressions.

Two contact fixtures initially failed: the reopening assertion still expected the
old instant mastery, and the chain fixture changed religious affiliation between
years, activating the unrelated five-year contact delay. The former now asserts
partial progress; the latter holds both site and household faith constant.

Reproduction commands:

```sh
cargo test --lib
cargo test --lib contact_ -- --include-ignored --nocapture --test-threads=1
cargo test --lib pilgrimage_and_recovery_use_real_routes_work_and_stocks -- --ignored
cargo test --test history_environment frozen_schedule_batch_and_checkpoint_equivalence -- --ignored
cargo clippy --all-targets -- -D warnings
```

These are mechanism and continuation checks. No long-run knowledge-diversity or
economic-balance calibration was performed in this increment.

The subsequent [encounter-selection increment](informal-learning-selection.md)
removes first-resident bottlenecks without changing these progress rates.
