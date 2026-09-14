# Growth screen: phosphorus throughput before demographic tuning

Twelve iterations found a reproducible growing scenario: improved nutrient
throughput plus needs-based access to finite food grows 600 people/five towns
into roughly 2,000–2,400 people and 10–11 active towns over 200 years. Ordinary
purchasing defaults remain unchanged: this demonstrates feasibility, not a fix
for every economy. Institutional service funding remains weak even in the growing
worlds. The final comparison confirms that actual tax administration can return
some household cash to government without preventing growth.

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
access. The combined nutrient arm ends at 360.8 and 511.1 people, also with no daughter
towns and one abandonment in seed 409. More release does not rescue the later
access failure. Iterations 9–11 compare wealth-tax/welfare, household solidarity,
and needs-based access against that combined-nutrient baseline.

A separate endpoint inspection at year 50 finds 2–6 eligible nearby unoccupied
candidates per parent in these worlds. Every selected candidate gives a 280-person
founding threshold. Several towns have 12–17 adult-ration months of stock but only
160–192 residents. Geography is not the immediate founding blocker there;
population thresholds are. This does not establish that lowering the threshold
would make a viable daughter town. Future checks should distinguish a demographic
constraint from affordable land, labor and food capacity.


### Iteration 9: welfare after nutrient relief

Wealth tax plus needs-first council welfare now raises year-200 population from
360.8 to 536.2 in seed 1024 and from 511.1 to 697.1 in seed 409. Both still have
five sites; seed 409 retains one abandonment. In the final fifty years,
births/deaths improve from 466/530 to 672/726 and from 623/660 to 834/845.
Access shortfall falls from 2.50% to 2.02% and from 2.12% to 1.84%.
Physical shortfall stays small (0.16% and 0.01% in the intervention).
This is useful interaction evidence: welfare alone failed in iteration 2, but
works better after the physical bottleneck changes. It still does not establish
positive late growth or successful daughter founding.


### Iteration 10: household solidarity after nutrient relief

Local cash contributions improve year-200 totals to 693.0 and 731.9 people,
respectively. There are still five sites, with four active in seed 409. In the
last fifty years, births/deaths are 867/940 and 899/955, so this does not demonstrate
continuing growth. Access deficits remain 2.17% and 2.05% of need, while physical
deficits are just 0.056% and 0.011%. Solidarity moves existing money and preserves
food-sale revenue, but its bounded donor allowances do not eliminate deprivation.

The unchanged-default five-year founding regression also passes all twelve arms,
including frozen/living histories and old/new provisions. It reports the same
15 legacy shortage town-months and the same six new-default populations of
654.19. Runtime was 119 seconds during concurrent 200-year screens; these elapsed
times are not isolated performance benchmarks. Final Clippy checks also pass.


### Iteration 11: needs-based access to finite food

This is the first scenario with sustained population **and active-settlement**
growth. Nothing changed the daughter threshold, its required provisions, mortality,
yields, geography, or starting population. Only the existing needs-based food
policy is added to the combined nutrient arm.

| Seed | People at year 200 | Sites ever founded | Active sites | Final-decade births | Final-decade deaths |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 1,988.5 | 13 | 10 | 444 | 349 |
| 409 | 2,368.2 | 14 | 11 | 534 | 423 |

Both begin with 600 people and five sites. Births exceed deaths in **each**
fifty-year window, not just at the endpoint. New sites continue to appear in the
second century. There are also three abandoned towns in each run: the policy does
not protect a failing settlement from physical shortages or remove mortality.
Whole-history food access deficit is effectively zero, but physical deficits
remain. Final-fifty-year physical deficits are 0.76% and 0.92% of need.
The largest absolute managed relative conservation residual in these two endpoint
reports is 1.07e-5; population/food residual magnitudes are below 0.000083%.

This is **not a fix for household purchasing**. The policy makes all current food
need eligible for common stocks, removes food-sale revenue, and changes monetary
circulation. It is a strong counterfactual showing that viable population and
settlement expansion are possible under current farming and founding rules once
nutrient throughput and food access both improve. It does not justify silently
turning this policy on in normal histories.

Institutional identity is also weaker evidence than service. Both worlds have
four institutions still marked active, but seed 1024's four have zero readiness,
zero treasury and exhausted building condition. Thus “active institutions” must
not be reported as proof of a healthy service economy. This prompted iteration
12's circulation experiment rather than a claim that every downstream system was
fixed.

Reproduce the growing scenario by adding these flags to the command above and
changing `--history-years` to 200:

```text
--farm-nutrient-retention 0.95 --farm-phosphorus-release 0.0000005
--enable-system needs-based-food
```

An independent API-path guard regenerates seed 1024 from scratch with these
settings, runs 200 years, and requires more than 1,500 people, at least eight
active towns, a surviving second-century daughter, positive final-decade
births-minus-deaths, and bounded food/population/economic residuals:

```sh
CARGO_INCREMENTAL=0 cargo test --test farm_nutrients   nutrient_and_food_access_scenario_supports_continued_expansion -- --ignored --nocapture
```

This is a regression for the demonstrated scenario, not an empirical calibration,
all-seed guarantee, default-economy claim, or institutional-health check.


The API regression passed: 1,988.48 people, ten active towns, and final-decade
births/deaths of 444.27/348.63, matching the CLI scenario's reported totals. Runtime
was 107.75 seconds during concurrent seed runs. The retained institutional identity
assertion is not a service-readiness assertion. Clippy passes with warnings denied.

In both iteration-11 worlds, every institution still marked active has zero
readiness and treasury. Their host towns have roughly 556–1,854 kg timber and
large remaining woodland carbon stocks, but zero municipal cash. This rules out
an absence of wood at those endpoints as a sufficient explanation for failed
maintenance; it motivates examining funding instead of increasing timber yield.


### Iteration 12: circulation in the growing scenario

The first circulation bundle enables estate inheritance, estate reclamation,
wealth tax and council welfare on top of iteration 11. Population and site results
are identical to iteration 11. Inheritance transfers 1,822.79 and 2,882.64 between
household accounts, but tax and reclamation transfer zero. Almost the entire
50,000-unit initial monetary inventory ends in household accounts. Municipal and
institutional cash remain effectively zero, and surviving institutions still
have zero readiness.

Inspecting the consumer, rather than trusting the enabled flag, revealed that
wealth tax and reclamation both require completed named-office service. The first
bundle left that separate experiment off. This is **not** evidence that a funded
tax collector failed: the policy had no collection work. The registry documentation
now states this operational dependency explicitly. A repeat with
`named-office-service` added is the final check. Such service still needs real
funding, eligible officeholders and taxable balances; enabling it is not a promise
that every assessment will be collected.


With named office service enabled, the same circulation bundle does execute:

| Seed | People | Active/total towns | Tax collected over 200 years | Estate cash reclaimed | Ending council cash | Ending town cash |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | 2,013.2 | 11/12 | 5,592.45 | 0.00 | 2,339.80 | 205.62 |
| 409 | 2,390.9 | 11/14 | 7,186.20 | 716.34 | 6,835.28 | 0.02 |

Final-decade births/deaths are 453.06/350.03 and 538.71/423.47. This retains the
demographic and settlement growth while returning some money to public accounts.
It is a bundled circulation comparison, not identification of each policy's
independent long-run effect. Surviving institutions still have zero readiness and
treasury. Council cash is not an automatic institutional service payment.

Reproduce this final comparison by additionally enabling:

```text
--enable-system household-estate-inheritance,household-estate-reclamation,household-wealth-tax,council-welfare-reserves,named-office-service
```

## What this establishes and leaves open

- Population caps, elapsed time and a lack of nearby sites were not the main
  blockers in these seeds. The unchanged founding thresholds can be crossed.
- Finite nutrient throughput first constrained production; after improving it,
  access to food and cash circulation became the consequential constraints.
- Two seeds sustain population and settlement growth through 200 years, with
  positive final-decade demographic balances and an independently rerun scenario
  regression. This meets the positive-growth demonstration requested for this
  screen; it is not an assurance of indefinite exponential growth.
- Needs-based food is an intentionally strong policy counterfactual. Defaults
  still decline, and a healthy household-purchasing economy remains unresolved.
- Public and institutional operating finance is the next useful problem. Actual
  tax service matters, but resulting council reserves do not automatically fund
  upkeep or compensate towns for communal food. More crops, population slots or
  an untested reduction in death rates would miss that connection.
- These are small 32/32 worlds and two primary seeds, with an additional baseline
  seed. No larger-resolution, cross-hardware or empirical calibration claim is
  supported by this screen. All run outputs remain ignored; no raw artifacts are
  committed.
