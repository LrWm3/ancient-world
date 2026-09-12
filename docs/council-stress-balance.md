# Council policies under broader environmental pressure

This follow-up extends the small-grid council comparison to held-out seeds and a
longer horizon. It tests a toy budget policy, not historical calibration.

The evaluation runner's `--weather-stress` option sets drought probability 0.5,
drought severity 0.9, storm probability 0.5 and storm multiplier 4. These are the
same forcing values used by the existing gathered/full-readback scheduler fixture.
The complete weather parameters are recorded in report metadata. No food, cash,
people, equipment or institutions are added to help the scenarios survive.

Both arms use the same weather stress and political distribution. Only annual
town support differs: existing support versus a request for the actual gap to
the town's working-cash target. The monthly administrative allowance stays off.
This separates the two budget policies after the allowance-only comparison's
mixed outcome.

## Protocol

Build `cargo build --example cultural_work_calibrate`, then:

```sh
target/debug/examples/cultural_work_calibrate --seeds 17,81,1024 --years 200 \
  --resolution 32 --crop-yield-scale 0.5 --individual-demography \
  --workshop-refinement --agriculture-refinement --extraction-refinement \
  --construction-refinement --compare-resolution --household-diagnostics \
  --family-support --weather-stress --output output/council-stress-existing.json
```

Repeat with `--cash-gap-town-support` and output
`output/council-stress-cash-gap.json`. Compare with both
`scripts/compare_food_access.py` and `scripts/compare_council_funding.py`, allowing
only `cash_gap_town_support` to differ. Raw results stay ignored under `output/`.

The runner already records actual expedition voyages and crew identities, military
service and deaths, and event counts. An enabled system with zero voyages or
service does **not** count as an exercised fiscal pressure. These runs impose
weather stress; they do not force wars or finance expeditions. Any unexercised
conflict/expedition path remains an explicit evidence gap requiring a separately
funded scenario fixture. Grid-resolution and hardware comparisons also remain
separate work.

## Status

The one-year seed-17 smoke test completes with 1,955 residents and records all four forcing values correctly. Example Clippy passes with warnings denied. The six-run, three-seed 200-year ensemble has started; results remain pending. Report failed or incomplete
histories separately; never count their last surviving sample as a 200-year result.

## Funded campaign boundary check

`cargo test --test governance council_allocation_campaigns_conserve_and_resume -- --ignored`
passes on Vulkan. A real affordable campaign is launched before saving in each of
three variants: existing support, cash-gap support, and cash-gap plus the monthly
administration allowance. Thirty-six months in one batch match thirty-six single
steps after reload. The war ends; complete histories match, tax observations are
present, and economic residuals remain below 0.001 in their native units.

This exercises resource-funded war under the policies. It does not establish
long-run wartime fiscal balance or prove that one policy improves war outcomes.

## Detected failure: actual farm-work over-allocation

The existing-support seed 1024 stops at month 1688 (140 years plus eight months).
Its requested farming allowance is 106.666664 worker-months, but the stored
individual grants sum to 106.66677019000053. The float32 display sum is lower,
106.66662; this is actual accumulated over-allocation, not merely a display/audit
sum problem. Seeds 17 and 81 complete 200 years in that arm. The failed seed's
140-year sample is not treated as a 200-year result.

A 399-worker fixture reproduces the fault with an identical 160/1.5 allowance and
0.8 capacity per worker. Repeated float32 subtraction can leave positive apparent
capacity after actual commitments exceed the total. The correction retains the
remaining allowance in double precision, rounds each individual grant downward,
and similarly bounds the GPU allowance by the double-precision sum of actual
commitments. It preserves the validation tolerance rather than accepting excess.
Both the new true-overgrant fixture and the previous false-audit-sum fixture pass.

For the corrected stress comparison, use isolated source `b9ed7bf` plus the
`src/agriculture_participation.rs` allocator patch. Do not include the intervening
husbandry, continuing-study or captured-route changes when interpreting the council
policy comparison. Corrected runs and final results are pending.

The cash-gap seed 1024 also fails, at month 2200, with stored grants
106.66678148508072 against the same 106.666664 request. Both policies therefore
expose the allocator fault. The corrected three GPU agriculture/extraction/
construction fixtures pass, including wage attribution and saved continuation.
