# Informal learning finds eligible participants

Contact and pilgrimage already shared bounded partial learning with paid lessons:
0.025 exposure for annual contact and 0.05 for a funded local pilgrimage, compared
with 0.1 work for paid study. Informal exposure is capped once per person per month,
across both channels. This increment changes selection, not those pacing constants
or the production labor ledger.

Previously both channels examined only the first resident at the source. Annual
contact also examined only the first destination resident. An uninformative source
or fully informed learner could therefore hide a useful encounter elsewhere in the
same community.

The shared selector examines eligible local people, skips learners who have already
received exposure this month, and finds a source for a missing practical topic.
For the selected learner it prefers the topic with the most existing study progress;
stable topic and person IDs break ties. Eligible residents come from canonical
presence queries, so absent and dead people cannot become encounter sources.
Candidate-list order does not change selection. This is a deliberately simple
selection policy; it does not model friendship networks or guarantee equal learning
opportunities. Learner priority still favors lower stable IDs. Each directed contact pair can
produce one encounter per annual pass; finding another eligible learner can
increase total exposure in well-connected towns, without bypassing personal caps.

Annual contact reads source knowledge and provenance captured at the opening of
its pass. Acquiring a topic does not allow it to relay through a second route in
that same pass. Pilgrimage reads the knowledge of people at the visited destination
at the time of the funded visit. Both keep the existing incremental progress,
provenance and acquisition events; finding a source does not grant instant mastery.

`Culture.contact_learning_month` records the last completed contact-selection
boundary. Repeating it, including after serialization, cannot simply select another
learner and generate additional contact exposure. Older archives default to no
recorded boundary. This guard covers informal knowledge selection; it is not a
claim that every legacy annual cultural operation is independently idempotent.
The normal monthly coordinator remains the scheduling authority.

## Verification

All 23 culture tests pass, including the explicitly run GPU fixtures. They cover a later
eligible source and learner, continuing study, reordered candidate lists, a source
that learned only after the opening snapshot, same-month exposure, serialization,
route closures and a pilgrimage whose first destination resident knows nothing.
Long-run effects on knowledge distribution and institutional membership remain
unmeasured. There are no new materials, money, paid work or population transfers.

The three-seed frozen scheduler comparison passes for monthly versus batched
advancement and checkpoint continuation. The archive-focused contact test passes
again with the old-record default check. The ordinary library suite passes 132
tests with 113 hardware fixtures ignored, and all-target Clippy passes with
warnings denied. Generated logs remain under ignored `output/`.

```sh
cargo test --lib culture:: -- --include-ignored
cargo test --test history_environment frozen_schedule_batch_and_checkpoint_equivalence -- --ignored
cargo test --lib
cargo clippy --all-targets -- -D warnings
```
