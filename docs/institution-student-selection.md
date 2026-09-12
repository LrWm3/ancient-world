# Institutional student selection pilot

Earlier opening diagnostics found many residents with an institutional teaching
source, but few selected as the quarterly cultural actor. Ordinary selection
rotates across every eligible resident, without considering whether a lesson is
available during that institution's operational window.

The opt-in `Culture.institutional_students` policy uses alternate site-quarters
(`(month / 3 + site) % 2 == 0`) to scan from the ordinary rotating position for a
student with an operational local institution, a present member who knows an
unknown institutional topic, and the existing household-affiliation eligibility.
Other quarters use ordinary rotation. If no eligible student exists, selection
also falls back to ordinary rotation. This is a toy opportunity policy, not a
model of academic admissions or individual preference.

The planner and request query share selection. The dated work plan captures the
actor and policy; execution uses that captured actor. A selected student requests
the same generic action bundle as before. Selection does not reserve money, grant
work, supply a teacher, bypass room capacity, or acquire knowledge. Those existing
allocation and execution checks still decide whether anything is accomplished.
Institutional lessons remain member-based; public enrollment is not added here.

Old archives default to ordinary rotation. Existing in-flight plans keep their
captured actors. The calibration runner exposes `--institutional-students` and
records it as report metadata. Default game behavior is unchanged pending evidence.

## Verification protocol

Source baseline `61a5784` plus this pilot. Quadro RTX 5000 / Vulkan, development
profile. Matched thirty-year seeds 17/81/256 use terrain 32, ecology 16, one epoch,
sixteen founders, living history, crop yield 0.5, individual demography and workshop,
agricultural, extraction and construction refinement. Both arms enable named
institution administration, operating funding and essential-first institution work.
Only the treatment adds `--institutional-students`.

```sh
cargo build --example cultural_work_calibrate
target/debug/examples/cultural_work_calibrate \
  --seeds 17,81,256 --years 30 --resolution 32 --crop-yield-scale 0.5 \
  --individual-demography --workshop-refinement --agriculture-refinement \
  --extraction-refinement --construction-refinement --compare-resolution \
  --household-diagnostics --named-institution-administration \
  --operating-institutions --essential-institution-work \
  --output output/institution-students-control.json
```

The treatment uses `output/institution-students-enabled.json`. Raw outputs remain
ignored. Concurrent execution is for behavioral comparison, not benchmarking.
Opening observations count student-opportunities, not unique people. Service
counts distinguish requests, opening space grants, work backing and completion.

## Boundary checks

- The hardware-backed institutional learning fixture passes. Its ordinary actor
  already knows the institutional topic, while another member can learn it. The
  opt-in policy selects that member and captures the correct teacher/topic; the
  plan still has zero granted labor. Closure and a fully knowledgeable membership
  restore ordinary selection. The next general-purpose quarter retains rotation.
- Culture round-trip retains the enabled policy and reproduces the entire plan.
  Missing policy fields load as ordinary rotation. Existing source, room, work,
  instruction progress and provenance checks in the same fixture also pass.
- The full frozen scheduler fixture passes on seeds 17/81/256, enabling the policy
  in 17/256 and retaining ordinary rotation in 81. Monthly, batched and saved
  continuation histories agree.
- Regular library tests: 127 passed, 110 hardware tests skipped. The two explicit
  hardware test commands above run separately. Library, runner and scheduler-test
  Clippy passes with warnings denied.

## Completed thirty-year comparison (2026-09-12)

All six histories complete. Each retains sixteen sites, zero maximum monthly
population residual, and maximum food residual below 3.3e-7.

| Seed | Operational student-opportunities, both arms | Selected source, ordinary → pilot | Lesson requests | Work-backed / completed lessons | Population, both arms |
|---|---:|---:|---:|---:|---:|
| 17 | 10 | 0 → 3 | 0 → 3 | 0 → 2 | 1,516 |
| 81 | 106 | 3 → 4 | 3 → 4 | 2 → 3 | 1,534 |
| 256 | 0 | 0 → 0 | 0 → 0 | 0 → 0 | 1,662 |

All requested lessons receive opening room grants; one in each exercised pilot
seed lacks work backing. Completed lessons rise from two to five overall, using
0.4/0.6 room-months in the two affected seeds. Petition completion remains 2/7/1,
and heritage study remains unexercised. Endpoint operational institution counts
remain 6/7/4. Population and the rounded cumulative food gaps are unchanged.

Endpoint knowledge-link counts remain 5,883/5,881/5,881. A completed lesson is
instructional progress, not necessarily acquisition of a discrete topic; these
results do not establish additional mastered knowledge or downstream productivity.
The policy is useful as a local opportunity correction, but cannot repair missing
operational teaching windows. Seed 256 is a negative control: 3,086 source-available
student-opportunities never coincide with an operational institution, and changing
selection achieves nothing. Review working-core readiness before broadening the
student-selection policy or making it default. Visitors and enrollment remain open.

```sh
python3 scripts/compare_food_access.py output/institution-students-control.json \
  output/institution-students-enabled.json --allow-difference institutional_students
```

This command verifies complete matched metadata, horizons, finite values and food
gap partitioning. No change to yield, relief or common-entitlement policy is part
of this comparison.
