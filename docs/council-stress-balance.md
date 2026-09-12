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
target/debug/examples/cultural_work_calibrate --seeds 17,81,1024 --years 200 \\
  --resolution 32 --crop-yield-scale 0.5 --individual-demography \\
  --workshop-refinement --agriculture-refinement --extraction-refinement \\
  --construction-refinement --compare-resolution --household-diagnostics \\
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

The one-year seed-17 smoke test completes with 1,955 residents and records all four forcing values correctly. Example Clippy passes with warnings denied. The longer ensemble is pending. Report failed or incomplete
histories separately; never count their last surviving sample as a 200-year result.
