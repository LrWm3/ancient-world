# Local food bottlenecks after institutional funding

Follow-up to [the institutional comparison](institution-growth-followup.md).
This pass adds the missing funding-only arm and inspects retained local food
boundaries. No production, mortality, relief or policy defaults changed.

## Funding-only comparison

Same settings and optional-policy bundle as the preceding comparison; fresh
200-year worlds, seeds 1024/409, 32/32 grids, aggregate demography. Both runs used
commit `22d0375`, including the deceased-household-head correction.

| Seed | Institutional policy | Population | Operating / active institutions | Final-decade births / deaths |
| --- | --- | ---: | ---: | ---: |
| 1024 | Neither | 1,185 | 1 / 16 | 272 / 238 |
| 1024 | Working core | 1,197 | 5 / 24 | 280 / 300 |
| 1024 | Operating funding | 1,158 | 5 / 18 | 268 / 267 |
| 1024 | Both | 1,156 | 13 / 20 | 272 / 298 |
| 409 | Neither | 1,219 | 1 / 20 | 280 / 243 |
| 409 | Working core | 1,116 | 4 / 20 | 260 / 252 |
| 409 | Operating funding | 1,190 | 7 / 20 | 276 / 257 |
| 409 | Both | 1,184 | 18 / 20 | 277 / 275 |

Funding alone improves operational counts, but joint funding and operating-space
accounting produces the largest improvement on both seeds. Counts have changing
denominators; these are different evolved populations of institutions, not matched
individual institutional treatment effects. More services do not reliably improve
population. The new runs took 130.6 / 120.7 seconds concurrently on the Quadro RTX
5000 Max-Q Vulkan backend; these are wall times, not isolated performance benchmarks.

## Where the funded seed 1024 loses food security

The retained monthly trace covers months 2341–2400. All physical food shortfalls
in that window occur at original sites 2 and 4. Other sites still have purchasing
shortfalls, which are counted separately.

| Site | Food need kg | GPU edible production kg | Physical shortfall kg | Access shortfall kg | Physical shortage months / 60 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 2 | 130,554 | 119,102 | 7,700 | 1,304 | 4 |
| 4 | 52,839 | 50,306 | 3,241 | 3 | 7 |

Site 2 ran short in months 2382–2383 and 2394–2395. Site 4's shortfalls likewise
clustered around months 6–7 of the model year. Both ended month 2400 with food
(13,232 / 5,543 kg): an endpoint stock is not evidence of uninterrupted food access.
Production over the whole window was below need, even before recorded spoilage
of 3,074 / 1,089 kg. This is a local supply and seasonal-buffer issue as well as
an affordability issue; giving these towns more purchasing money alone cannot
supply food that is physically absent.

Their recorded whole-month unclassified net food flows were approximately -0.65
and -0.58 kg over five years, tiny beside the shortages. This algebraic remainder
includes more than trade and cannot exclude balanced imports and exports. The
only recorded land route touching either town connects them to each other, about
1,037 km. Site 2's endpoint port has no vessels. Those observations suggest a
transport constraint worth testing, not proof that shipping is the sole cause.
Several other towns ended with large stores: sites 0, 1 and 3 had approximately
75,704, 80,484 and 76,943 kg respectively. Those stocks are not automatically
accessible to the deficit towns.

In the final decade the aggregate demographic accounting attributes 160.03 deaths
to base mortality, 122.06 to nutrition and 15.96 to illness (298.05 total), against
272.03 births. These are the model's arithmetic components, not independently
estimated causal effects. Do not infer that removing the nutrition term would
preserve the rest of the trajectory.

## Work actually completed

The last cultural receipt is dated month 2400: 3.90 requested and granted abstract
work units, 3.35 used and 0.55 released. Institutional upkeep plans used all 2.50
requested/granted units. One successor knowledge acquisition was recorded at that
boundary. These distinguish completed work from institution readiness, but do not
measure annual teaching, research or service benefits. No claim that a particular
school increased harvest follows from these endpoint records.

## Reproduction and verification

Reproduce the funding-only arm (repeat with seed 409):

```sh
target/debug/ancient-world --headless --seed 1024 \
  --resolution 32 --ecology-resolution 32 --epochs 1 \
  --civilizations 5 --history-years 200 \
  --farm-phosphorus-release 0.0000005 --farm-nutrient-retention 0.95 \
  --enable-system demographic-audit,institution-operating-funding,municipal-food-relief,food-solidarity,household-estate-inheritance,household-estate-reclamation,household-wealth-tax,council-welfare-reserves,named-office-service \
  --history-export output/funding-only-1024.json
python3 scripts/summarize_growth_funding.py output/funding-only-1024.json
python3 scripts/report_growth_bottlenecks.py output/funding-only-1024.json
```

The new report reuses monthly flow accounting, groups by stable site ID, exposes
missing months and incomplete boundaries, and dates cultural plans independently
of the current month. Missing service data remain unknown rather than zero.
Three new tests cover local versus world aggregation, production versus net flow,
missing observations, and stale service plans. Reporting tests passed (15 tests),
and the growth-focused discovery passed (8 tests, including the three new tests;
these counts overlap). Both native experiments completed with normal validation.
No Rust behavior changed in this pass; the full GPU suite was not rerun.

Next: compare a bounded seasonal-buffer or feasible inter-town food-delivery
intervention against the same opening state. Measure delivered food and the
pre-harvest deficit first; population is a downstream outcome. Do not solve a
missing vessel or food source by silently minting cargo, cash or labor. Larger
seed ensembles and default-resolution evaluation remain outstanding.
