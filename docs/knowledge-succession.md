# Teaching successors for scarce practices

Quarterly personal teaching now prioritizes techniques with the fewest living adult
holders in the town. A sole metalworker can teach that vulnerable practice before a
common crop-calendar topic, even when the common topic appears earlier in the catalog.
This extends existing instruction, personal knowledge, reserved cultural work and
production access; it does not introduce a second skill inventory.

## Selection and costs

For the scheduled teacher:

1. Count topic holders among present, living adult local household representatives.
2. Consider only topics the teacher knows and another eligible local person lacks.
3. Select the topic with the fewest holders, breaking ties by stable topic ID.
4. Favor an eligible learner already studying that topic, then the fewest known
   topics, using seeded preference and person ID to resolve ties.
5. Spend 0.1 reserved worker-months, advance partial learning and preserve lesson
   provenance. Grant practical knowledge only when learning completes.

The actor schedule and other cultural opportunities are unchanged. Reading or an
institutional lesson can consume work before personal teaching; there is no free or
guaranteed apprenticeship. Less than 0.1 available work prevents instruction. Dead,
underage, absent or in-transit people do not count as local successors. Local scarcity
is not a global ranking: distant teachers cannot make a town's knowledge secure.

`Culture::knowledge_holders(history, site)` returns twelve fixed counts. Practical
production access is derived from nonzero counts, with the same eligibility as before.
These counts represent named adult household representatives, **not all skilled
workers or the size of a profession**. Partial learning is personal, while practical access remains binary. There are no
professional licenses or empirically calibrated qualification requirements.

The explorer identifies topics with one holder as succession risks. Teaching events
state the pre-instruction holder count, name teacher and learner, and retain causal
links to the teacher's learning source. A student who completed learning retains practical access after the teacher dies.
A partly trained student does not yet replace the missing practitioner. Without instruction, death or migration of the last holder
can still remove the topic from the mask supplied to GPU production.

Existing histories preserve acquired knowledge. New optional personal fields store
partial studies and completed instruction work; old archives initialize both empty.
GPU layouts remain unchanged. Future knowledge transmission can take longer than
in earlier software; no identical-future compatibility is promised.

## Accumulating paid learning

Successor lessons, institutional instruction and accessible-document study use
`progress += 0.1 / 0.3 * (1 + 0.5 * curiosity + 0.5 * teaching_support)`, capped
at one. Curiosity is the learner's existing trait. Teaching support is the teacher's
previous completed instruction work divided by that work plus one; independent
reading has zero teacher support. Ordinary novices need three lessons; strong
curiosity and experienced instruction can reduce this to two. These constants are
explicit toy pacing choices, not measured pedagogy.

A successful paid teaching action adds 0.1 to the teacher's instruction record.
This is an experience measure of already charged work, not additional available
labor. Existing participant reservations, source-presence checks and material limits
still apply. An unavailable teacher or destroyed/absent document cannot advance
progress. Partial studies are included in planned-identity guards and persist with
the learner, so a new valid source can continue earlier work.

Each paid lesson records progress and links to its source and prior lesson.
Completion grants the topic and makes the completion event its provenance. Partial
progress does not count as a holder or unlock recipes. The read-only culture report
exposes unfinished study counts and instruction work. Previously acquired knowledge
is not revoked. Other routes such as founding instruction, trade contact and
pilgrimage retain their existing transmission rules; this is not yet a universal
model of all learning or forgetting.

## Controlled evidence

The new fixture supplies one teacher with common crop calendars and sole-holder
metalworking, plus local potential learners. It verifies:

- Scarce metalworking is selected before the common lower-ID topic.
- Insufficient work prevents teaching; sufficient work consumes the existing budget.
- The event names the learner and preserves the teacher's source event.
- Teaching changes no goods or private cash inventories.
- Teacher death removes metalworking in the untaught comparison, while instruction
  preserves it in both local access and the production input mask.
- A dead teacher cannot propose instruction; invalid sites have zero holders.
- Serialized continuation produces identical production-input preparation.

The fixture initializes a world on GPU, but teaching itself is CPU history logic.
It verifies the input passed to production, not a new cell-level shader comparison.
Existing culture and institutional-instruction tests cover further transmission,
attendance, material transfers and checkpoint behavior. No long-run retention-rate
calibration or manual GUI exercise is claimed.

```sh
mise exec rust@1.89.0 -- cargo test --lib scarce_practices_receive_successors -- --ignored
mise exec rust@1.89.0 -- cargo test --lib institutional_learning_needs_a_present_source -- --ignored
mise exec rust@1.89.0 -- cargo test --test culture -- --include-ignored
mise exec rust@1.89.0 -- cargo test --all-targets
mise exec rust@1.89.0 -- cargo clippy --all-targets -- -D warnings
```

Captured logs and source hashes: [Artifact retention policy](evidence/README.md).

## Incremental-learning checks

The updated fixtures require multiple separately funded lessons, verify progress
before completion, and show that the selected successor remains preferred while
studying. No goods or cash are created. A teacher's death while a student is only
partly trained removes the production topic; further unfunded/unsupported study
makes no progress. Restoring the source permits completion. Partial-state checkpoint
continuation matches uninterrupted completion, including causal event chains.
Document study also requires repeated paid work and stops when the text is destroyed.

An initial unit run exposed completed progress at `0.99999994` rather than exactly
one. Completion now normalizes that field to one after the existing small floating
point tolerance check. The first full library run had 182 passes and that one failure;
the verification below reports the corrected run separately.

Three ten-year smoke runs (seeds 17, 81, 256; terrain 32, ecology 16, one geological
epoch; individual demography and workshop refinement enabled) retained final
populations 2,173 / 2,127 / 2,178. All had zero cohort overhang and unresolved
identities, with maximum absolute normalized economic residual at most `7.06e-7`.
However, none scheduled paid study or successor teaching during those ten years.
They demonstrate continued compatibility of ordinary history, **not learning-rate
calibration**. The controlled fixtures, rather than these short runs, establish the
learning effect and its source/participation requirements.

A fifty-year seed-17 extension did exercise the new path: 115 partial successor
instruction events, 42 completed successor teachings, and one partial document or
institutional study event. There were also 70 contact-knowledge events using the
unchanged contact mechanism. Final resident population was 2,970 with 5,887 known
person/topic links, zero cohort overhang, zero unresolved identities, and maximum
absolute normalized economic residual `2.75e-6`. Runtime was 92.4 seconds on the
available Vulkan GPU. This establishes that ordinary history uses the new path;
a single long seed does not establish a desirable learning rate or knowledge-loss
frequency, and unchanged contact channels remain another route to knowledge.

After the normalization fix, **183/183 library tests passed** including hardware
checks (84.76 seconds). **13/13 culture integration tests passed** (20.86 seconds).
All-target Clippy with warnings denied, formatting, whitespace and repository
artifact checks passed. Reproduction:

```sh
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib -- --include-ignored
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test culture -- --include-ignored --test-threads=1
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo run --example cultural_work_calibrate -- --individual-demography --workshop-refinement --seeds 17,81,256 --years 10 --output output/learning-progress-seeds.json
# Repeat the example with --seeds 17 --years 50 for the longer check.
```

Only this Markdown summary is committed; raw outputs remain ignored under
`output/learning-progress-*`.
