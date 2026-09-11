# Cultural work cancellation calibration

This is game-behavior calibration, not a fit to historical labor data. The question
is whether identity guards unnecessarily discard useful reserved work while still
preventing substitution of absent actors and changed teaching targets.

## Experiment

The diagnostic runner is `examples/cultural_work_calibrate.rs`. Every quarterly
boundary contributes requested, granted, completed and cancelled worker-months,
funded/cancelled site bundles, changed identity categories and candidate actions.
Cancelled work is the remaining grant at the first failed guard, not necessarily
the bundle's original grant: some institutional work may already have completed.
Repeated validation preserves that first observation instead of overwriting it
with zero. Action/category counts can overlap and do not represent distinct lost
executed actions. Zero-grant bundles are excluded from cancellation-rate counts.

Use the existing scarce-inner-continent default crop yield of 0.5, 16 initial civilizations,
one geological epoch, terrain edge 32 and ecology edge 16. Society, politics,
governance, offices, shipping, expeditions, discoveries and living history are
explicitly enabled. Other settings retain bundled defaults. Samples are taken every
three months; reports retain cumulative observations at ten-year intervals.
These small grids exercise the social schedule; they are not production-resolution
world calibration or performance benchmarks.

Seeds 17, 81 and 256 supplied the initial 100-year comparisons. Seeds 409 and 1024
were reserved for 200-year checks after choosing the change. No parameter fitting
or requirement for universal population growth was imposed.

## Change being compared

The former guard captured every local resident's identity, knowledge and faith.
A bystander learning something or a household changing its representative could
therefore cancel work assigned to someone else. The initial runs lost about 8%
of granted cultural work this way, with peaks around generational turnover.

The default guard now captures the selected actor, the planned teaching recipient
and selected institutional teacher. The institutional lesson's source is pinned,
so a newly available teacher cannot silently replace the original. Live checks
still require the named teacher and institution to be eligible. Institutions,
objects, recovery requests and their existing identity guards remain unchanged.

`Culture::focused_work_identities = false` retains the broad guard as an explicit
comparison control; the runner exposes it as `--strict-identities`. Old saved plans
without a participant list retain their broad observation until the next normal
reservation. New plans use the selected setting. Births, deaths, knowledge,
materials and wages are not altered to make either arm succeed.

## Results

Remaining unused work includes unmet material requirements, minimum action sizes
and competing actions; it is not all cancellation waste. A lower cancellation rate
is useful only while actual participant and target changes continue to invalidate
work.

### 100-year development comparisons

| Seed | Broad guard: cancelled grant | Focused guard: cancelled grant | Broad: used / granted | Focused: used / granted | Completed work change |
| --- | ---: | ---: | ---: | ---: | ---: |
| 17 | 8.16% | 2.83% | 76.44% | 80.55% | +4.96% |
| 81 | 8.05% | 2.92% | 78.57% | 82.45% | +4.85% |
| 256 | 8.10% | 2.82% | 75.01% | 79.00% | +5.17% |

The broad guard cancelled 7.95–8.07% of funded bundles; the focused guard cancelled
2.83–2.98%. For seed 17, cancellations involving people fell from 393 to 43.
Category counts overlap: losing a participant can change both people and faith.

Population did not increase universally: seed 17 finished at 2,760 rather than
2,686, seed 81 at 3,165 rather than 3,203, and seed 256 at 2,418 rather than 2,490.
The focused arm retained 32, 34 and 32 active institutions, versus 32 each under the
broad guard. These downstream differences are outcomes of diverging histories,
not evidence that the guard directly controls demographic growth.

A seed-17 replay of `--strict-identities` on the updated code reproduced every
reported sample, event-type count, residual and cancellation-category count from
the original broad-guard run. This checks that the instrumentation/control did not
silently change that baseline; it is not a claim about unreported whole-world bytes.

### Held-out 200-year comparisons

| Seed | Broad guard: cancelled grant | Focused guard: cancelled grant | Broad: used / granted | Focused: used / granted | Completed work change |
| --- | ---: | ---: | ---: | ---: | ---: |
| 409 | 9.70% | 4.18% | 72.73% | 76.71% | +5.22% |
| 1024 | 9.76% | 4.31% | 75.61% | 79.48% | +9.70% |

Seed 409 finished with 1,807 people and 36 active institutions under the focused
guard, versus 1,881 and 36 under the broad guard. Seed 1024 finished with 3,164 and
46, versus 2,584 and 42. Its greater total completed-work increase includes a
larger grant pool in the diverged history; the utilization change is the more
direct comparison. People-related cancellations fell from 923 to 111 and from
1,012 to 116. Institution changes remain the largest cancellation category in
both focused runs (383 and 475 bundles).

Retain the focused guard as the default. There is no reason to target zero
cancellations: actual participants, objects and institutions can become unavailable.
Unused grants also remain legitimate when an action cannot afford its materials.
All these runs granted the full requested cultural work, so they do not establish
behavior under a binding cultural labor shortage. They also do not establish
cross-hardware or production-resolution equivalence. Timings were collected with
concurrent workloads and should not be used to compare performance.

## Verification

On the Quadro RTX 5000 with Max-Q Design, Vulkan backend, Rust 1.89.0:

- Ordinary library suite: 73 passed; 68 GPU tests remained ignored in that command.
- Explicit GPU culture suite: 13 passed, including the changed-participant fixture.
- History environment suite: three passed, covering compact/full readback,
  batching, checkpoint continuation and due-food timing.
- Clippy across all targets with warnings denied, formatting and repository
  artifact checks passed.

The focused fixture changes only a bystander's knowledge and verifies that actual
crafting still consumes materials and labor; the broad control cancels it. Changing
the teaching recipient still cancels work. Existing death, migration and stale-plan
checks remain. A second validation cannot erase the recorded cancellation loss.
Saved work plans preserve their participant identities. The history environment
fixtures compare complete state; the long calibration reports are aggregate checks.

## Reproduction

Run from the repository root. The runner writes raw results only under ignored
`output/`; commit this summary rather than the generated reports.

```sh
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo build --example cultural_work_calibrate
target/debug/examples/cultural_work_calibrate --legacy-participation --strict-identities --output output/cultural-work-baseline.json
target/debug/examples/cultural_work_calibrate --legacy-participation --output output/cultural-work-focused.json
target/debug/examples/cultural_work_calibrate --legacy-participation --strict-identities --seeds 409,1024 --years 200 --output output/cultural-work-heldout-strict.json
target/debug/examples/cultural_work_calibrate --legacy-participation --seeds 409,1024 --years 200 --output output/cultural-work-heldout-focused.json
python3 scripts/summarize_cultural_work.py output/cultural-work-baseline.json output/cultural-work-focused.json output/cultural-work-heldout-strict.json output/cultural-work-heldout-focused.json
```

The summarizer refuses incomplete reports. Event totals, cancellation categories,
ten-year observations and final economy residuals are retained locally for inspection.
Knowledge-link totals include historical agent records, not just living teachers;
artifact totals likewise count records rather than only usable surviving objects.

The commands explicitly disable the subsequently added individual participation
pilot, keeping this comparison focused on identity guards rather than personal
capacity allocation.

The individual-participation delivery subsequently included previously omitted
institutional election work in the quarterly receipt. The table above retains the
measurements made before that reporting correction; current reruns can therefore
differ in work totals even with legacy participation selected. See
[individual participation](individual-participation.md) for the new verification.
