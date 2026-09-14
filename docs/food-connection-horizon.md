# Planning food connections before a shortage

The [local bottleneck investigation](growth-local-bottlenecks.md) found recurring
pre-harvest shortages while other towns retained food. This experiment asks whether
an infrastructure decision looks far enough ahead, rather than increasing food,
cash, farmland or baseline storage.

## Timing mismatch and bounded intervention

`History::food_connection_investments` identifies a prospective surplus/buyer pair
for annual harbor construction staffing. Previously its buyer target was fixed
at three adult-ration months. A buyer with six months of food can fail that test
at the annual review even though it may run short before the next review.

The target is now the validated, archived production catalog field
`food_connection_target_months`, between 0 and 24. Missing fields and bundled
catalogs retain three months. Native runs can set `--food-connection-months 12`.
This scalar configures the existing `food_connection_investment` mechanism; it
does not introduce another optional system or change registry enablement.

The selection still requires an open prospective sea lane, affordable buyer,
source surplus after its existing food reserve, acceptable distance and no hostile
pair or flood closure. Incoming food cargo reduces the projected shortage. It
still picks one best pair and preserves support for already active cargo routes.
A zero horizon eliminates the prospective shortage score, not existing-cargo
support; disable `food_connection_investment` for the full mechanism ablation.

Only demand recognition changes. Reserve-phase harbor staffing is bounded by
available construction materials; annual installation still consumes those
materials and actual work. This is not a promise to purchase a year's food, a
forecast of future yield, an escrow, or free fleet/cargo capacity. The three-month
legacy rule is preserved exactly when the new field is absent.

## Verification design

A controlled GPU-founded fixture gives a reachable buyer six months of food.
Three-month planning selects no pair; twelve-month planning selects it. The query
leaves the entire history unchanged, and serialized continuation gives the same
selection. Existing fixture controls cover an empty source, cashless buyer,
closed lane and disabled investment, plus finite construction/work and resume.
A pure catalog test covers old defaults, bounds, nonfinite rejection and roundtrip.

Matched 200-year runs use the institutional funded bundle from the preceding
investigation: seeds 1024/409, terrain/ecology 32/32, one geological epoch, five
foundings, aggregate demographics, initial nutrient retention .95 and finite
phosphorus release 5e-7/month. Add `--food-connection-months 12` to the joint
institutional funding command. The previous three-month arm is the baseline.
Raw exports remain under ignored `output/growth-rounds/`.

Inspect commissioned ports, assets, vessels, actual cargo and local physical food
gaps before interpreting population. Longer lookahead can move work away from
other uses and cannot fix missing material supply. In particular, harbor equipment
currently requires generic tools; a town's stock of copper tools is not a substitute
in this construction path. Substitution needs explicit material provenance and
wear accounting, not renaming copper as generic tools.

## Results: infrastructure worked; welfare did not reliably improve

| Seed | Horizon | Population | Active towns | Final-decade births / deaths | Whole-run physical / access gap |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 3 months | 1,156 | 9 | 272 / 298 | 0.470% / 0.882% |
| 1024 | 12 months | 741 | 7 | 175 / 204 | 0.129% / 1.628% |
| 409 | 3 months | 1,184 | 10 | 277 / 275 | 0.229% / 1.124% |
| 409 | 12 months | 1,269 | 10 | 295 / 299 | 0.215% / 1.054% |

Seed 1024 demonstrates a real intermediate effect. Ports at sites 0 and 2
commissioned in months 348 and 408, and the existing site-3 port remained
maintained. All three ended with four vessels. The history recorded 5,742 sea
arrival events versus zero in the baseline. Those events include multiple goods;
they must not be reported as 5,742 food deliveries. There were 271 `market_arrival`
food events across all transport modes in the treatment history.

During the last retained 60 months, all physical food gaps disappeared, including
at the original deficit sites. However, purchasing gaps remained: sites 1 and 4
accounted for about 5,209 and 10,389 kg of unmet access respectively. Site 4 ended
with roughly 94,288 kg of food and 306 people, compared with 5,543 kg and 55 people
in the baseline. Major population redistribution and different historical paths
accompany this intervention; town identities alone are not matched populations.

Food prices did not exhibit a simple world-wide blow-up at the endpoint: site 4's
quote was about 1.377 versus 1.726 previously. Aggregate household cash was higher
(19,395 versus 14,553), but that does not establish that the households needing
food held it. The evidence supports examining distribution and household
entitlements alongside infrastructure, rather than inferring a universal food
production deficit or prescribing additional money indiscriminately.

Seed 409 commissioned one port at month 360, but no second port completed a sea
connection; sea arrivals remained zero. Its higher population cannot be attributed
to delivered maritime food. Changed construction spending/work and subsequent
history can matter even when the intended network does not operate.

Both native runs completed with validation, in 142.2 / 132.6 seconds concurrently
on the Quadro RTX 5000 Max-Q Vulkan backend (other machine load was present).
The pure catalog test, expanded hardware harbor fixture, regular library suite
(217 passed, 158 ignored), reporting suite (15 passed), Clippy and repository
artifact check passed. The native help exposes the new option. Full GPU-suite and
cross-resolution balance verification were not performed.

**Decision:** retain this as an explicit experiment with the three-month default
unchanged. Earlier planning is capable of enabling actual transport, but twelve
months is not a justified universal improvement. Next comparisons should start
from a shared pre-connection checkpoint and measure household food budgets,
actual transfers, employment and migration over a shorter window. A later material
substitution improvement should preserve installed tool identities and scrap,
and be tested separately. No population target was fitted in this pass.
