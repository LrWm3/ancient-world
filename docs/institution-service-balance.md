# Institutional service balance: matched 30-year histories

Source: `7675316`; Quadro RTX 5000 Max-Q, Vulkan. Both arms completed seeds
17, 81 and 256 through 30 years, with monthly accounting and samples at years
10, 20 and 30. Both processes exited successfully. All report numbers were
finite; seed lists, completion markers and sample boundaries were checked.
Configuration metadata matched except for the selected institutional work policy.

## Reproduce

Build `cargo build --release --example cultural_work_calibrate`, then run:

```sh
for policy in control essential; do
  extra=()
  if [ "$policy" = essential ]; then
    extra=(--essential-institution-work)
  fi
  target/release/examples/cultural_work_calibrate \
    --seeds 17,81,256 --years 30 --common-share 0.65 \
    --individual-demography --workshop-refinement \
    --agriculture-refinement --extraction-refinement --construction-refinement \
    --compare-resolution --operating-institutions \
    --named-institution-administration "${extra[@]}" \
    --output "output/service-room-${policy}-30.json"
done
```

The loop uses Bash or Zsh arrays. Defaults here are terrain 32, ecology 16, one
geological epoch, yield scale 0.5 and 16 founders. Common share 0.65 is a controlled
food-access intervention, not the project default. Essential-first changes duty
allocation, not scheduler order. These runs used the same release executable;
processes overlapped, so elapsed times are not a performance comparison.
Raw reports and logs remain ignored under `output/`.

## Outcomes

Food gaps are cumulative monthly gaps divided by cumulative food need, not crop
yield estimates. Physical shortage and inability to access food remain separate.

| Seed | Policy | Final population | Operational / active institutions | Physical food gap | Food access gap |
|---|---|---:|---:|---:|---:|
| 17 | Control | 1,951 | 0 / 32 | 0.0438% | 1.9543% |
| 17 | Essential-first | 1,960 | 7 / 30 | 0.0439% | 1.9414% |
| 81 | Control | 1,995 | 0 / 32 | 0% | 1.9597% |
| 81 | Essential-first | 1,981 | 8 / 32 | 0% | 1.9073% |
| 256 | Control | 2,066 | 0 / 32 | 0% | 1.9687% |
| 256 | Essential-first | 2,052 | 6 / 32 | 0% | 1.9463% |

Essential-first preserves some institutional operation and slightly improves food
access in these cases, but population effects are mixed (+9, -14, -14).
This is a small toy-game comparison, not empirical calibration, and does not
justify changing the default policy.

## What institutions actually completed

Counts below accumulate all 360 monthly boundaries. A lesson completion is one
teaching session, not necessarily mastery of a topic. Zero-request rows are
omitted. All other lesson/hearing rows, all heritage-study rows and religious
dispatch counts were zero.

| Seed | Policy | Service | Requested | Opening space granted | Work-backed | Used |
|---|---|---|---:|---:|---:|---:|
| 17 | Control | Hearing | 3 | 3 | 0 | 0 |
| 256 | Control | Hearing | 1 | 1 | 1 | 1 |
| 17 | Essential-first | Hearing | 1 | 1 | 1 | 0 |
| 81 | Essential-first | Lesson | 4 | 4 | 4 | 4 |
| 256 | Essential-first | Lesson | 8 | 8 | 6 | 5 |

Every recorded request used 0.2 room-months. Controls therefore requested 0.8,
received 0.8 opening space, retained 0.2 with work backing and used 0.2.
Essential-first requested 2.6, received 2.6 opening space, retained 2.2 with work
backing and used 1.8. Three control requests and two essential-first requests lost
work backing. Two essential-first requests failed after receiving both grants.
The aggregate counters do not identify the exact live eligibility failure.

No recorded request was space-denied. That does **not** establish abundant space:
requests pass operational and actor/topic eligibility gates before entering this
ledger. At year 30, 32 institutions per control had readiness below 0.25; the
essential-first counts were 22, 22 and 23. Whole-membership space coverage below
0.4, among institutions with at least two local adults, affected 27/24/23 control
institutions and 20/17/20 essential-first institutions. These are overlapping
snapshot conditions, not exclusive causal attributions.

The lesson planner also selects one rotating actor per site, requiring an
operational institution, mutual membership, a present teacher and knowledge the
actor lacks. Readiness and this opportunity selection both warrant review;
this run does not measure how many requests each upstream gate suppressed.
Heritage and relief require their controlled fixtures for verification: these
histories supply no balance evidence for those services.

## Verification and next boundary

All 126 regular library tests passed, with 109 hardware tests ignored in that
run. All 13 targeted room/teaching/hearing/heritage/relief/relocation/allocation
checks passed, as did all-target Clippy with warnings denied. Full frozen
monthly/batched/checkpoint equivalence passed for seeds 17/81/256.

Across all six histories, maximum reported population residual was zero;
maximum monthly food residual was below 2.16e-7 and maximum absolute reported
end-of-run economy residual below 2.58e-6, in their respective ledger units.
Conservation checks do not establish sensible service rates.

The next useful diagnostic is the request funnel before room reservation:
operational eligibility, eligible students/sources, actor selection and work
matching. Mixed-bundle cancellation still merits narrower attribution. Visitors,
relief labor/shared freight and a controlled working-core versus whole-membership
readiness comparison remain unfinished. This result completes the capacity
measurement increment, not the broader integration worklist.
