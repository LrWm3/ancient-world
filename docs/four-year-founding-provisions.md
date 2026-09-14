# Four years of founding provisions, with and without storage

The [monthly food trace](monthly-food-balance.md) showed that initial supplies
lasted through year one, but weak harvests and gaps before the next harvest exhausted
several towns during years two and three. This experiment tests the user's proposed
four-year arrival inventory, then repeats it with larger baseline granaries.

**Food plus storage substantially improves founding survival.** None of the three
seeds has a dietary shortfall greater than 0.01 kg per town-month through year five
in that arm. Population reaches approximately 654 from an initial 600. Later
shortages still develop, but year-twenty populations remain above the controls.
This is toy-world balance evidence, not historical calibration.

## Controls and implementation

Three arms use the same founding archives, seeds 1024, 256 and 409, frozen 32/32
worlds and preceding screen's policies. Production, trade, work allocation,
demography, ordinary spoilage and crop yields are unchanged.

| Arm | Declared food per person | Total food for 600 people | Baseline granary capacity |
| --- | ---: | ---: | --- |
| Baseline | 216 kg | 129,600 kg | 12 adult-ration months per current resident |
| Food only | 864 kg | 518,400 kg | Same as baseline |
| Food and storage | 864 kg | 518,400 kg | 48 adult-ration months per current resident |

“Four years” means 48 months at 18 kg per person per month. Mixed-age dietary need
is lower, but spoilage and other uses still apply. Initialization's existing 120 kg
seed allocation per town stays separate, leaving 517,800 kg edible stock in the
four-year arms before month one. Patron sustenance is a separate inventory and is
not increased. Daughter settlement founding provisions are unchanged.

Added options:

```
--founding-food-months 48
--base-granary-months 48
```

Both CLI overrides require a month-zero managed history. The food override requires
recorded patron arrivals and changes the declared arrival inventory, initial food
and C/N/P accounting, and the patrons' human provision records. It records an
explicit founding-provisions event. Repeating the same target adds nothing again;
a lower target cannot withdraw previously declared provisions. Food targets are
bounded to 12–120 months and do not create recurring supplies or money.

The storage option changes archived `production.base_granary_months`, default 12,
bounded to 12–120. It enters the existing GPU capacity equation; normal container
bonuses and spoilage remain active. It is a **baseline-capacity experiment**, not
construction purchased from timber, bricks or labor. Capacity scales with current
population just as the original granary rule did, so this arm changes long-term
storage rules too, not merely the size of a temporary landing cache. It does not
provide generic workshop/warehouse capacity. Normal defaults remain unchanged.

## Storage explains most of the food-only loss

In all three seeds the first-month global quantities are the same:

| Arm | Food spoiled in month one | Food held at month-one close |
| --- | ---: | ---: |
| Baseline | 1,146 kg | 118,736 kg |
| Food only | 350,102 kg | 158,578 kg |
| Food and storage | 4,599 kg | 504,081 kg |

The old capacity rule discards overflow after consumption. The food-only arm loses
about 349,000 kg more than the baseline immediately—about 90% of the additional
388,800 kg shipment. Ordinary spoilage continues in the larger-granary arm and is
higher in absolute kg because more food is stored. Overflow returns its embodied
nutrients through the existing detritus pathway; the experiment does not erase its
physical accounting. These nutrient returns are another downstream consequence of
changing the initial stock.

## Population and shortages

All arms begin with 600 people in five towns.

| Seed | Year 3 baseline | Food only | Food + storage | Year 5 baseline | Food only | Food + storage |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | 506.90 | 558.97 | 632.84 | 445.92 | 484.01 | 653.59 |
| 256 | 552.42 | 596.32 | 632.88 | 544.19 | 580.87 | 654.19 |
| 409 | 556.83 | 600.03 | 632.88 | 543.85 | 577.05 | 654.19 |

Food alone delays the earliest shortage from month 16 to 24 in seed 1024, 18 to 25
in seed 256, and 18 to 29 in seed 409. Food plus storage has no measured dietary
shortage through month 60 in any seed. Over years one through five, physical
shortfall as a percentage of integrated dietary need changes:

| Seed | Baseline | Food only | Food + storage |
| --- | ---: | ---: | ---: |
| 1024 | 15.384% | 12.263% | 0% |
| 256 | 7.680% | 5.325% | 0% |
| 409 | 7.858% | 5.288% | 0% |

The longer comparison remains favorable but does not establish self-sufficiency:

| Seed | Year 20 baseline population | Food only | Food + storage | Baseline physical shortfall | Food + storage physical shortfall |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 321.81 | 331.44 | 382.26 | 10.894% | 7.539% |
| 256 | 491.38 | 503.34 | 541.33 | 5.774% | 4.369% |
| 409 | 496.50 | 511.09 | 546.40 | 5.762% | 4.020% |

Shortfall percentages integrate the entire twenty years, not just the endpoint.
In the food/storage arm, seed 1024 first records more than 1 kg of annual unmet
need in year six. Seeds 256 and 409 first exceed that threshold in year seven,
initially through access shortfalls; physical deficits exceed 1 kg in year eight.
The annual record does not identify the exact first month after the five-year run.

Food-access deficits are not solved. Over twenty years they increase from
0.082% to 1.221% of need in seed 1024, 0.306% to 0.445% in seed 256, and 0.172% to
0.191% in seed 409. Exported food also varies rather than uniformly rising:
food/storage dispatches approximately 14,313 / 2,839 / 2,709 kg across the seeds,
versus baseline 9,752 / 6,745 / 4,879 kg. These are dispatches, not deliveries.

Higher population also means more people exposed to later risks. For example,
seed 1024 has more cumulative births and slightly more aggregate GPU deaths with
food/storage than the baseline by year twenty. Do not interpret its larger ending
population as uniformly lower mortality at every stage.

## Decision

Providing four years of food **with storage that preserves it** is a useful way to
make the founding period more forgiving. It resolves the observed first-five-year
shortages in this small seed suite without increasing yields. Extra food with old
storage still helps, but wastes most of the shipment immediately.

Before adopting it as normal founding, decide whether baseline granaries should
have this capacity permanently, or whether arrivals instead bring a finite protected
cache. The latter would be a different model and would need new comparisons. Later
production deficits and household food access remain separate issues. There is no
storage-only arm here, no claim that the two interventions have been fully separated,
and no larger-resolution, living-history or century validation of these settings.

## Verification and reproduction

There are 27 balance runs: three seeds × three arms × years 3, 5 and 20. All exit
successfully with native validation; maximum absolute reported managed C/N/P,
water, money or goods relative residual is below `6.1e-6`. Full commands, raw exports
and logs are local under ignored `output/four-year-provisions/`. Use the full prior
screen's settings and month-zero archives, adding the options above. Exact local
archives are not committed under repository artifact policy.

The final library suite passed 211 tests (156 extended/hardware tests remain
ignored). Strict all-target Clippy passed. Unit fixtures cover additional versus
duplicated provisions, invalid targets, archived storage defaults and capacity
bounds. The one-year provision override exactly reproduces the prior three-year
histories for all three seeds. After adding the storage parameter, seed 1024's
baseline still matches exactly after removing only the new default catalog field.
A month-zero save/load and repeat application of the 48-month settings also matches
the original initialized history exactly. This tests initialization idempotence,
not full long-run checkpoint or cross-GPU equivalence.
