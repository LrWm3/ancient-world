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
4. Choose an eligible learner with the fewest known topics, using the existing seeded
   preference and then person ID to resolve ties.
5. Spend 0.1 reserved worker-months, transmit the topic and its provenance, and record
   the teacher/learner relationship through the existing teaching path.

The actor schedule and other cultural opportunities are unchanged. Reading or an
institutional lesson can consume work before personal teaching; there is no free or
guaranteed apprenticeship. Less than 0.1 available work prevents instruction. Dead,
underage, absent or in-transit people do not count as local successors. Local scarcity
is not a global ranking: distant teachers cannot make a town's knowledge secure.

`Culture::knowledge_holders(history, site)` returns twelve fixed counts. Practical
production access is derived from nonzero counts, with the same eligibility as before.
These counts represent named adult household representatives, **not all skilled
workers or the size of a profession**. There are no proficiency levels or multi-year
qualification requirements yet. Topic acquisition remains the existing binary,
quarterly game abstraction.

The explorer identifies topics with one holder as succession risks. Teaching events
state the pre-instruction holder count, name teacher and learner, and retain causal
links to the teacher's learning source. A surviving student retains practical access
after the teacher dies. Without instruction, death or migration of the last holder
can still remove the topic from the mask supplied to GPU production.

No archive fields or GPU layouts change. Existing histories preserve acquired
knowledge; future teaching choices intentionally change under this priority rule.

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
