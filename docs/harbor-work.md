# Harbor construction and repair work

Ports already held finite timber, tools and masonry, with 2% annual deterioration
and transport capacity limited by the least complete material component. New ports
now also require spare craft labor for construction and repair. This closes the
previous unlimited-labor path through otherwise resource-limited port building.

The annual public-works sequence remains roads followed by harbors. Both consume
`economy.logistics[2]`, the unused craft worker-months remaining after production.
Harbors install 100 kg timber, 10 kg tools/rigging or 100 kg masonry per
worker-month. A complete new port requires four worker-months; ordinary full-port
wear requires 0.08 worker-months annually. These are explicit game-balance rates,
not historical shipbuilding estimates. Existing population-scaled material
reserves, policy eligibility and abandonment restrictions still apply.

Desired material additions are scaled together to the available workforce.
Construction can span years, and commissioning still requires all three material
components. The goods withdrawn are credited to the existing port inventory.
Labor is recorded cumulatively, consumed once and cannot also repair roads.
Material contributions retain the existing in-kind public-works model; this does
not introduce a wage or council-money payment for harbor labor.

A commissioned port with less than 50% structural capacity records deterioration;
recovery to at least 80% records restoration. The events share the port identity
and causal links. Temporary flood closures are separate from structural condition.
The inspector displays cumulative harbor work and deterioration. Existing cargo
reservations and new-shipment capacity checks use the same canonical port assets.

Work state is archived, validated and guarded against repeat annual calls in the
same month. Older ports without this state retain their legacy labor rules.
This remains an annual proxy using December's spare labor, not a year-round crew
scheduler. Road-first priority can disadvantage harbors and is exposed in the
integrated results rather than concealed by free labor.

## Verification

Controlled tests cover exact work costs, no construction without labor, partial
construction, finite material transfers, commissioning, decay after abandonment,
condition recovery, event causality, repeated-call idempotence and serialized
continuation. The existing GPU shipping integration test exercises actual coastal
routes, organic sea cargo, conservation, closures and full checkpoint continuation.

```sh
mise exec rust@1.89.0 -- cargo test --lib harbor_ -- --include-ignored
mise exec rust@1.89.0 -- cargo test --test shipping -- --ignored
```

## Reproduction

The evaluator now retains final harbor assets, commissioning dates and cumulative
work. The comparison uses the previous road-upkeep build under matching seeds and
settings, so it measures the added harbor-work constraint with roads already
consuming labor in both versions.

```sh
python3 scripts/build_history_evaluator.py --output output/harbor-build-new
python3 scripts/evaluate_enterprises.py --binary output/harbor-build-new/evaluator.bin \
  --build-manifest output/harbor-build-new/manifest.json --output output/harbor-runs-new \
  --seeds 17,81 --years 50 --modes operators-linked
python3 scripts/summarize_harbor_work.py --current output/harbor-runs-new \
  --baseline docs/evidence/road-upkeep/final --output output/harbor-runs-new/harbors.json
```

These are game-balance and integration checks at resolution 64 with yield scale
0.33 and a living environment. Later population differences include nonlinear
feedback; the controlled fixture isolates the immediate labor and capacity effects.

The first ensemble attempt exposed a validation failure in seed 81 at month 539:
the event subject validator did not recognize `road` or `port`. This also affected
the preceding road-condition feature, although its two seed runs had not triggered
those notices. Validation now checks both subject types against their canonical
collections. Regression fixtures accept real condition events and reject invalid
port IDs. The failed attempt is retained separately from completed-run evidence.

## Retained results

[Artifact retention policy](evidence/README.md) contains completed raw
trajectories, exact source/executable hashes, verification logs and the separately
marked incomplete first attempt.

| Seed | Commissioned / surveyed | First opening | Cumulative work | Structural capacity kg | Deteriorated at end | Population, prior → current | Max relative residual |
|---|---:|---:|---:|---:|---:|---:|---:|
| 17 | 3 / 5 | Year 3 | 28.20 | 2,273 | 0 | 1,803 → 1,811 | 1.20e-5 |
| 81 | 3 / 5 | Year 3 | 29.56 | 2,450 | 1 | 1,528 → 1,554 | 1.32e-5 |

Both retained three operational ports, as in the preceding build. Seed 81 recorded
one harbor deterioration and no restoration; seed 17 recorded neither. Structural
capacity is summed across commissioned ports without temporary flood closures.
An impaired port is still usable at reduced capacity. This demonstrates exercised
maintenance pressure, not an empirically validated failure rate. The initial work
rates are retained; larger ensembles and alternate public-works priorities remain
open calibration work.

64 ordinary tests passed (136 GPU fixtures ignored by the ordinary command).
The harbor command passed one pure and one GPU fixture, the road regression command
passed one pure and two GPU fixtures, and the full shipping GPU integration test
passed. Clippy and formatting passed. Runs used Vulkan on the Quadro RTX 5000 Max-Q;
shared host activity means elapsed times are not isolated performance benchmarks.
