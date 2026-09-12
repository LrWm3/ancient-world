# Institutional working-core space pilot

`Culture.institution_working_core` is an opt-in game rule separating the space
needed to operate an institution from its aspiration to accommodate all local
members. It defaults to false, including archives without the field.

Existing readiness uses the minimum of funded upkeep, local staffing, building
support and maintenance work. For facilities, building support previously divided
usable space by whole-membership demand. A growing membership could therefore
make a maintained institution fail even though a small staff could operate it.

With this pilot, only the readiness denominator uses the existing kind-specific
space demand for at most two present adult members. Membership-based expansion
targets remain unchanged. No remote member supplies local staffing, and existing
work, money, ownership, damage and disruption checks remain in force.

This is not extra floor area. Lessons, study, hearings and heritage services still
compete for actual dated room grants and personal labor. Readiness permits a
request; it does not complete one. Larger membership can still justify expansion
without making it a prerequisite for all modest activity.

The existing institution report retains whole-membership `space_demand` and
`space_coverage`, adding `operating_space_demand` and `operating_space_coverage`.
The balance runner exposes `--institution-working-core` and records the setting
in matched-report metadata. Policy affects quarterly maintenance in Respond;
subsequent requests observe the resulting readiness. It does not revise earlier
work or room reservations.

## Verification

The Vulkan funding/space fixture passes all eight cases over twelve quarters.
Four local members in a maintained merchant room of capacity two end with
readiness 0.14 under whole-membership coverage and 0.74 under working-core
coverage. Both consume the same six cash of fees and 2.21 kg of repair timber.
The larger room remains operational under both policies. Both unfunded sizes
fail under both policies. Cash, embodied material, repair waste and labor checks
remain exact within the fixture tolerances. JSON continuation reproduces each
quarter. Expansion demand still reports eight while core operating demand is four.

The full frozen scheduler test passes on seeds 17/81/256, with the pilot enabled
in 17/256 and the old rule retained in 81. Monthly, batched and saved continuation
histories agree. Matched balance evaluation is reported below.
Balance comparison must measure completed lessons and knowledge, not just
operational institution counts. The two-member core and existing space coefficients are toy
parameters, not historical staffing estimates.

## Matched evaluation protocol

Source baseline `56cf027` plus this pilot, Quadro RTX 5000 / Vulkan. Both arms
use terrain 32, ecology 16, one epoch, sixteen founders, living history and crop
yield 0.5. Both enable individual demography, workshop/agricultural/extraction/
construction refinement, named institutional administration, operating funding,
essential-first work and alternate-quarter student selection. Family cash support
remains disabled in both arms. Only core-space policy differs.

```sh
cargo build --example cultural_work_calibrate
target/debug/examples/cultural_work_calibrate \
  --seeds 17,81,256 --years 30 --resolution 32 --crop-yield-scale 0.5 \
  --individual-demography --workshop-refinement --agriculture-refinement \
  --extraction-refinement --construction-refinement --compare-resolution \
  --household-diagnostics --named-institution-administration \
  --operating-institutions --essential-institution-work --institutional-students \
  --output output/institution-core-control.json
```

Repeat with `--institution-working-core` and output
`output/institution-core-enabled.json`. Compare complete metadata and food budgets:

```sh
python3 scripts/compare_food_access.py output/institution-core-control.json \
  output/institution-core-enabled.json --allow-difference institution_working_core
```

Raw outputs stay ignored; concurrent runs provide behavioral evidence, not
performance measurements. The regular library suite passes 129 tests (112 extended
or hardware tests skipped); the two named GPU checks above were run separately.
All-target Clippy passes with warnings denied.

## Completed thirty-year comparison

All six histories complete with sixteen active sites, zero maximum monthly
population residual and maximum food residual below 3.3e-7.

| Seed | Operational institutions control → core | Lesson requests | Completed lessons | Completed hearings | Endpoint knowledge links | Population |
|---|---:|---:|---:|---:|---:|---:|
| 17 | 6 → 30 | 3 → 76 | 2 → 69 | 2 → 60 | 5,883 → 5,880 | 1,516 → 1,514 |
| 81 | 7 → 27 | 4 → 161 | 3 → 133 | 7 → 53 | 5,881 → 5,884 | 1,534 → 1,526 |
| 256 | 4 → 30 | 0 → 105 | 0 → 94 | 1 → 37 | 5,881 → 5,900 | 1,662 → 1,666 |

Operational student-opportunities rise from 10/106/0 to 1,902/5,686/2,840.
All requested lessons receive opening room grants. Work-backed lessons are
69/137/96; actual completion is 69/133/94, so attendance/execution checks still
matter after a grant. Heritage study and opening religious relief remain
unexercised; these histories cannot establish their balance.

The controlled fixture identifies operating-space coverage as the immediate
mechanism. In coupled histories, greater readiness also permits hearings and other
services, affecting later decisions; endpoint population or knowledge differences
cannot be attributed solely to teaching. A completed lesson adds topic-specific
progress, not guaranteed mastery. Wider access without sustained enrollment can
spread small amounts of instruction among many learners. The mixed endpoint
knowledge counts leave that limitation visible.

Cumulative purchasing gaps (% need) change from 3.2233/3.2466/3.1142 to
3.1428/3.3441/3.0302. Population and access effects are mixed. This is a useful
correction to the operating-space requirement, not a general prosperity fix.
Keep the policy opt-in pending held-out/longer comparisons and joint funding
review. Sustained enrollment, visitors and multiple institutional officer roles
remain separate commitments; do not broaden this pilot by creating free labor,
space or automatic knowledge.
