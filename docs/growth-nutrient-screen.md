# Growth screen: phosphorus throughput before demographic tuning

This continues the [four-year founding comparison](four-year-founding-provisions.md).
A safe founding period does not establish long-term carrying capacity. The question
is whether population can grow and finance daughter settlements, rather than just
whether the initial five towns remain occupied. These are toy balance experiments,
not empirical agricultural or demographic calibration.

## Controls and iteration record

Fresh worlds use terrain/ecology 32/32, one geological epoch, five civilizations,
600 initial people, current standard system defaults, living history, and the new
48-month food/storage defaults. Crop yield scale stays 0.5, plot area 160 hectares,
and island phosphorus scale 0.25. `demographic-audit` is enabled. Recent optional
policies remain off except those identified below. There are no population imports,
mortality discounts, raised crop yields or lowered daughter-founding thresholds.
Results concern aggregate-authoritative demography.

| Iteration | Change | Seed 1024, people at year 50 | Seed 409, people at year 50 |
| --- | --- | ---: | ---: |
| 1 | Current defaults | 465.3 | 437.9 |
| 2 | Progressive household wealth tax + council welfare reserves | 453.2 | 436.8 |
| 3 | Gradual nutritional stress | 463.3 | 448.4 |
| 4 | Both policy changes | 468.7 | 449.0 |
| 5 | Nutrient retention 0.95 instead of 0.85 | 697.9 | 777.0 |
| 6 | Geological P release 5e-7 instead of 1e-7 monthly | 715.3 | 778.3 |
| 7 | Both nutrient changes | 715.3 | 775.6 |

All these runs still have five active towns at year 50. An additional baseline,
seed 256, ends with 442.2 people. Improvements in ending population do not yet
prove settlement expansion or continuing positive growth.

## Why change the nutrient mechanism?

At year 50, all five seed-1024 baseline towns report phosphorus-limited production,
with almost no available soil phosphorus. Geological phosphorus reserves still
contain approximately 4–8 million kg per town. Most available work already goes
to farming; simply assigning more farmers would not remove this cap.

The default monthly geological release is 1e-7 of the remaining source. A town
with four million kg therefore releases about 0.4 kg per month. Meanwhile, the
food accounting returns 85% of consumed N/P to detritus. A rough 100-person,
1,500-kg monthly food budget at 0.003 kg P/kg food loses 0.675 kg P even before
runoff and other exports. This is an accounting illustration, not a precise
prediction for mixed crops, age bands, livestock or each town's water conditions.

Both mechanisms retain finite accounting: release moves P from geological stock
to soil; retention reduces the existing exported fraction and moves nutrients into
detritus, which still needs to decompose. Neither multiplies crop output or adds P
from nowhere. Source extraction depletes its source over time.

The interventions also change the proposed mediator. In seed 1024, retention-only
leaves positive soil P in three towns and removes their closing P constraint.
The physical dietary deficit over 50 years falls from 1.66% of need to effectively
zero; access-related deficit remains 1.73%. Nutrition-attributed deaths fall from
476 to 289. Faster release similarly leaves virtually no physical deficit, but
still 1.64% access deficit and 275 nutrition deaths. For seed 409, physical deficit
falls from 2.57% to approximately 0.3%. This motivates examining food access again
*after* relieving nutrient scarcity; its failure alone does not rule out an
interaction. The combined nutrient arm's small extra benefit also argues against
indefinitely increasing nutrient supply.

## Reproduction and controls

Build the native binary, then run the baseline:

```sh
CARGO_INCREMENTAL=0 cargo build --bin ancient-world
target/debug/ancient-world --headless --seed 1024 \
  --resolution 32 --ecology-resolution 32 --epochs 1 --civilizations 5 \
  --history-years 50 --enable-system demographic-audit \
  --save output/growth-baseline.world --history-export output/growth-baseline.json
```

Repeat with distinct output paths and either or both numeric controls:

```text
--farm-nutrient-retention 0.95
--farm-phosphorus-release 0.0000005
```

Retention uses the existing site-policy setter and records policy events. It
changes existing towns only; future daughter towns retain their own initial
policy. Release is an archived economy-catalog parameter,
`production.phosphorus_release_monthly_fraction`, used by every managed town,
including daughter towns. Defaults remain unchanged. Missing fields in old saves
retain the previous 1e-7 monthly rate. Release is bounded to [0, 1e-4], retention to
[0, 1]. These numeric controls are not additional boolean registry systems.
Omitting them preserves saved values. Changing release takes effect at the next
production boundary; it does not retrospectively alter existing stocks.

The release parameter uses a previously unused lane of the existing GPU uniform,
without another buffer or readback. The GPU still performs the transfer. The
parameter is a game calibration control, not a measured rock-weathering rate.

## Verification

```sh
CARGO_INCREMENTAL=0 cargo test --test farm_nutrients -- --include-ignored --nocapture
```

The ordinary test checks invalid rates and old-catalog defaults. The optional GPU
test completes land claims, then checks the actual geological reserve against the
one-month analytical answer `P_next = P - P * rate`, with both zero release and the
maximum supported rate. It checks managed conservation and exact serialized
history equality between batched advancement and checkpoint-resumed monthly steps.
Both passed on Quadro RTX 5000/Vulkan; the GPU test took 59 seconds including shader
startup while the seed screen was also running. Clippy passed with warnings denied.
Raw histories, world archives and logs are under ignored `output/growth-rounds/`.
Only source, tests and this summary belong in Git.

## Longer follow-up

Iteration 8 extends retention-only and combined nutrient arms to 200 years on
seeds 1024 and 409. Retention-only ends at 407.8 and 518.9 people, respectively;
seed 409 has one abandoned town. This contradicts sustained growth despite the
promising year-50 response. In seed 1024, physical shortages remain effectively
zero throughout, while access shortfall is 2.95%, 2.53% and 2.23% of dietary need
in the subsequent fifty-year windows. The corresponding births/deaths are
741/909, 575/655 and 503/545. Better nutrient retention alone does not solve food
access. The combined nutrient arm and further access comparisons are pending.
