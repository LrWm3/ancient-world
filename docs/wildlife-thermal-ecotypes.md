# Regional thermal ecotype pilot

This opt-in game mechanism adds one persistent regional thermal preference per
animal guild. It is not individual genetics, a new species identity, or a claim
that all animals have the same physiology. Fixed guild diets and body sizes remain.

`Config.wildlife_ecotypes` defaults to false. The wildlife runner accepts
`WILDLIFE_ECOTYPES=1`; it enables the pilot after generating the same opening stocks
as its control. `WILDLIFE_CLOSED_ONLY=1` retains the normal geographic barriers.

## State, behavior and boundaries

Ecological pools 38–40 hold twelve temperature preferences, encoded as Celsius +81
in the range 1–141. Zero denotes an uninitialized preference. Metadata is excluded
from C/N/P inventories. The ecological cell grows from 608 to 656 bytes; its two
buffers require 36 MiB more at ecology resolution 256. Existing allocation and
binding-size checks use the actual new stride. All consuming shaders share it.

New occupied founder populations receive local mean temperature plus a smooth,
seeded regional offset of at most 8 °C, bounded to −80…60 °C. Existing populations
without traits receive that explicit baseline on their first enabled biological
step. Turning the pilot off stops thermal feeding effects and adjustment; existing
traits still travel with biomass. Neither mode spontaneously creates wildlife.

During the monthly biology pass:

- Feeding demand is multiplied by `1 / (1 + ((T − preference)/15)^2)`.
  A 30 °C mismatch admits 20% of otherwise identical demand. Food availability,
  capture, nutrient stoichiometry and maintenance still apply.
- Actual assimilated growth replaces a fraction `growth / (opening carbon + growth)`
  of the population. Its mean preference shifts 2% of that fraction toward local
  mean temperature. Starvation produces no adjustment. This is a deliberately
  simple growth-linked local-adjustment rule, not an explicit genetic-selection
  calculation. The 15 °C width and 2% response are game parameters, not fitted data.
- Mortality and respiration retain the surviving population's preference. Numerical
  extinction and explicit guild removal clear it. Restoring a guild only permits
  recolonization; it does not restore a dead local ecotype.

During transport, the same pairwise, area-accounted carbon flux carries a thermal
moment. Recipients mix preferences by incoming and resident carbon before paying
movement respiration. Uninitialized legacy stocks do not count as a fictitious
−81 °C population: the mean uses only known-trait carbon. When the pilot is enabled,
biology initializes occupied unknown stocks before the normal transport pass.
The pilot does not add temperature-based route selection or relax geographic
barriers. Isolation can preserve different preferences; contact mixes them.

These traits can influence food-web stocks through feeding shortfalls. They do not
confer an Ancient World multiplier. Initial differences follow climate and regional
founder variation; subsequent differences depend on growth and actual migration.
Species identities, reproductive isolation, multiple ecotypes within one guild/cell,
body-size evolution and season-specific thermal physiology remain unfinished.

## Persistence and inspection

Archive version 8 stores the additional metadata. Versions 2–7 preserve their old
fields and gain zero-filled trait slots; older configuration defaults leave the
pilot off. No past evolutionary history is reconstructed. Normal enabled archive
continuation retains the actual traits. Cell inspection shows occupied guild
preferences; wildlife reports provide carbon-weighted regional means and spatial
standard deviations (null when no trait was recorded).

## Verification and evaluation

On the available Quadro RTX 5000 Max-Q / Vulkan, the eight wildlife GPU fixtures
and twenty ecology integration tests pass, alongside 127 regular library tests
(111 hardware tests are excluded from that ordinary library command). The focused
thermal case checks an analytical feeding response and growth-linked adjustment,
a starvation negative control, disabled-effect control, C/N/P accounting,
checkpoint continuation, removal and non-resurrection. The migration fixture
checks blocked coastal edges with the pilot enabled, colonization retaining the
source preference, and the analytical carbon-weighted mean of two populations.
The existing fine-edge CPU reference separately covers cube-face conductance.
Archive fixtures cover v3, v6 and v7 expansion and continued evolution.
The living-history conservation/clock/checkpoint fixture also passes, exercising
the updated farming and managed-return buffer consumers.

Matched comparisons used seeds 17, 81 and 256; terrain resolution 32; ecology 16;
one initial geological epoch with twelve ecological months; then 1,200 additional
months with terrain geology frozen and normal ecological weather/water updates.
Both arms use the same generation settings and have identical opening wildlife
reports. The enabled arm
establishes traits at the first comparison month. No civilization history runs in
this experiment. All results below are coarse regional estimates, not fine-grid
habitat assignments.

| Seed | Inner wildlife carbon, pilot/control | Ancient World wildlife carbon, pilot/control | Lake predator carbon, pilot/control | Inner / outer small-herbivore preference SD |
| --- | ---: | ---: | ---: | ---: |
| 17 | 1.068 | 1.037 | 0.491 | 2.23 / 4.08 °C |
| 81 | 1.050 | 1.035 | 0.382 | 2.82 / 3.87 °C |
| 256 | 1.057 | 1.057 | 0.377 | 1.87 / 4.11 °C |

The aquatic predator guild remains above the reporting occupancy threshold across
the great lake in all three enabled endpoints. Its share of lake aquatic consumer
carbon falls from 0.97/1.02/0.81% to 0.47/0.39/0.30%. Ocean wildlife carbon falls
5–8%. Land predator responses are mixed on the inner continents and increase in
the Ancient World; the endpoint differences must not all be described as a simple
predator-release cascade. These are coupled changes, with only the immediate
thermal feeding mediator isolated by the analytical fixture.

Maximum enabled relative C/N/P residuals are 1.74e-5 / 3.68e-6 / 2.75e-7;
maximum water residual is 1.41e-7. Runs remain finite and nonnegative. The broad
thermal preference distributions have not homogenized after a century, but this
does not establish speciation, equilibrium, or centuries of historical balance.

Runs take 13.4–15.5 seconds after initial generation in the optimized development
profile. Accumulated transport GPU time is 410–449 ms per 1,200-month run; biology
is 353–389 ms. Transport remains the largest measured ecological kernel. These
small-grid toggle comparisons do not measure the overhead of expanding the buffer
against the pre-change executable, nor establish performance at default resolution.

Keep the feature opt-in: the strong aquatic predator response merits diet- and
thermal-width comparisons before broader use. Future work should separate
physiological adaptation from species identities and permit multiple coexisting
regional traits rather than treating a mixed guild mean as a full population model.

Reproduce each arm with `WILDLIFE_CLOSED_ONLY=1 WILDLIFE_ECOTYPES=0` (or `1`), a
separate `WILDLIFE_OUTPUT=output/wildlife/thermal-0-1200.json` (or `thermal-1-1200.json`),
and `cargo run --example wildlife_evaluate -- 1200 32`. Raw outputs remain ignored.
The fixture commands are `cargo test --lib ecology::wildlife_tests -- --ignored`
and `cargo test --test ecology -- --include-ignored --test-threads=1`.
