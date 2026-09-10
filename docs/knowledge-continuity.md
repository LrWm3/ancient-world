# Knowledge continuity and actual teaching sources

This increment closes two inconsistencies between cultural history and practical
production. An institution's historical topic set previously supplied lessons
without a living local teacher or a surviving document. Separately, the production
knowledge mask could count a dead or underage household head, and could treat a
civilization leader as present in multiple towns before household simulation.

## Available practice versus recorded tradition

`Culture::available_knowledge(history, site)` now supplies the production topic
mask and the explorer's local practical knowledge list. It combines the knowledge
of the existing living, adult, locally present representatives returned by the
cultural actor lookup. Households in transit or lost on journeys do not supply
local knowledge. Abandoned towns supply none. Before households exist, a leader
represents the civilization's first site rather than every settlement.

These are named household representatives, not a full census of skilled workers.
One eligible knower still enables a technique; there is no proficiency-weighted
labor pool or gradual skill decay in this increment. The existing recipe gates
consume this mask through `Economy.management[3]`. Losing the last local knower
therefore closes relevant production recipes at the next reservation boundary;
it does not destroy existing goods or equipment.

## Institutional instruction

A lesson from an institution now requires all of:

- The institution is operational and located at the learner's site.
- The learner belongs to it.
- A different living adult, present at that same site, belongs to it and knows
  the topic.
- The topic is part of its recorded knowledge and unknown to the learner.
- At least 0.1 worker-month remains in the existing cultural action budget.

The institution retains its historical topics when teachers die or leave. Those
records are possible traditions to revive, not disembodied instructors. Remote
membership does not constitute a local teaching visit. Teacher schedules are not
modeled separately: this remains the existing bounded cultural instruction proxy,
with institutional upkeep and reserved cultural labor.

Institutional lessons identify the named teacher and institution and link to the
teacher's recorded learning event when available. Instruction records never invent
an earlier source if the teacher has no surviving provenance record.

## Documents and recovery

The existing physical-document study path remains available: the object must be
local, not lost, and not destroyed, and study needs reserved work. Reading can
restore practical knowledge when no teacher survives; an unread document does not
automatically open production recipes. Personal knowledge remains after the source
teacher dies or the institution closes. Recorded objects and institutional histories
are retained rather than erased when practical access disappears.

The production summary now includes per-site practical topic masks alongside the
existing total historical knowledge links. These quantities mean different things:
the latter includes historical agents and is not a count of current skilled labor.
No GPU buffers or archive schema change. Existing saved personal knowledge and
sources remain intact; the corrected eligibility rules apply when resumed.

## Verification and limits

The controlled fixture checks missing teachers, teacher death, departure to another
site, institution closure, unavailable work, a successful sourced lesson, continued
practice after the teacher's death, and document destruction versus readable survival.
It also checks the exact topic mask uploaded for production and serialized source
continuity. Production-order tests already exercise the downstream recipe knowledge
gates. Document contents in the fixture are explicitly assigned; the experiment is
not a naturally generated lost-library history.

```sh
mise exec rust@1.89.0 -- cargo test --lib institutional_learning_needs -- --ignored --nocapture
mise exec rust@1.89.0 -- cargo test --test culture -- --include-ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --lib institution_capacity::tests -- --include-ignored --test-threads=1
```

This fixes eligibility and provenance, not the whole learning model. Annual route
contact remains a coarse transmission mechanism. Aggregate proficiency, teacher
workloads, manuscript comprehension prerequisites, innovation and measured regional
skill-loss frequencies remain future work. No claim of historical calibration or
improved population outcomes follows from the controlled checks.

### Recorded validation

49 ordinary tests and 11 explicitly executed GPU tests passed. The latter include
the new source-continuity fixture, nine cultural integration tests (including
checkpoint continuation, artifact balances and political/cultural separation),
and institutional upkeep. Clippy with warnings denied, formatting and whitespace
checks passed. Hardware: Quadro RTX 5000 Max-Q using Vulkan.

[Evidence logs and source checksums](evidence/knowledge-continuity/) retain the
runs. This change does not introduce an optional alternate knowledge universe:
it corrects eligibility in the existing system. Old saves retain their knowledge
records but may lose recipe access previously supplied only by ineligible actors.

Personal teaching now [prioritizes locally scarce practices](knowledge-succession.md),
and the inspector reports single-holder succession risks.
