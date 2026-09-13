# Selective food-connection investment

## Retained mechanism and control

The 5% variant is retained as a bounded connection mechanism, not as a claim
that overall economic balance is solved. `production.food_connection_investment`
is enabled by default and archived in the economy catalog. Set it to `false`
through the existing `--economy-catalog` option to disable supplemental eligibility;
ordinary harbor planning and construction remain. Old catalogs default to enabled.

The food-request audit found usable surplus behind unfinished harbors, especially
in seeds 256 and 409. This experiment gives supplemental annual work and a bounded
investment of held tools to a selected connection rather than every surveyed port.
It does not change food production, protected seller reserves, purchasing ceilings,
prices, household funding or the physical requirements for commissioning a harbor.

At the annual production-planning boundary:

- Consider surveyed, open, unflooded sea lanes between their actual endpoint towns.
  Include inland approaches and the existing sea-distance advantage in the market
  distance limit. Exclude abandoned/non-trading towns and active enemy pairs.
- Require actual food above the seller's catalog reserve and a buyer below three
  months of food, after subtracting pending food cargo.
- Bound the prospective quantity by buyer cash at the current seller quote. Rank
  quantity divided by one plus travel-equivalent distance. Select one pair; stable
  lane order breaks equal scores. Existing cargo also makes its endpoints eligible
  for supplemental maintenance, even when current food opportunities are absent.
- Only the eligible ports forecast supplemental work. All other ports keep ordinary
  construction behavior. Missing materials still use the existing production plan.

This is an aggregate game-planning heuristic, not a knowledge or diplomacy model.
It considers only direct surveyed endpoint towns, not all possible hinterland
customers or multi-leg routes. The score does not estimate construction payback,
future harvests or future population; small current opportunities can therefore
justify work that later proves unproductive. Affordability is an observation, **not a loan,
escrow, funded export contract or promise that the buyer will survive until opening**.

## Finite work and tools

Supplemental work is at most 2% of the annual boundary's available monthly
workforce, inside the existing 20% public-service ceiling. It enters adaptive
staffing, then actual building work and prepaid industrial attendance retain their
protection. It cannot impersonate named builders. Requested, protected, used and
released work are recorded in `Economy.harbor_work`; old archives default to zero.

Eligible construction can invest at most 5% of held generic tools below the
ordinary working reserve. Above the reserve, the existing investable surplus
remains available. Production continues targeting the full ordinary working
reserve; this does not declare working equipment obsolete or create replacements.
For example, 22 kg held can contribute at most 1.1 kg from scarce equipment,
leaving 20.9 kg before other actual uses. Timber and masonry reserves are unchanged.

Actual installation consumes finite inventory and work. Annual wear, weather,
commissioning requirements, hull backing, paid crews, route capacity and delivery
remain in force. Unused annual work is released; it is not backdated into recipes
that already ran. No new money is issued.

## Fifty-year screening

Baseline `f8c83b0`; identical founding archives, 600 months and settings as the
[food-request screen](food-request-diagnostics.md). Terrain/ecology 32/32,
Quadro RTX 5000 with Max-Q Design. First test a 10% scarce-tool share, then halve
it to 5% with the same selected-connection rule. All six runs completed.

| Seed | Arm | Population | Ending hunger | Operator work | Operator margin | Food dispatched kg | No-route requests | Commissioned ports |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | Baseline | 159.871 | .03695 | 11.219 | 47.42 | 8,844.7 | 94 | 2 |
| 1024 | 10% | 154.159 | .12318 | 14.089 | 55.70 | 10,145.0 | 74 | 3 |
| 1024 | 5% | 154.171 | .04330 | 12.306 | 48.34 | 10,044.6 | 79 | 3 |
| 256 | Baseline | 337.571 | .03323 | 4.500 | 18.50 | 0 | 651 | 0 |
| 256 | 10% | 351.039 | .01774 | 4.555 | 19.74 | 12,414.3 | 417 | 3 |
| 256 | 5% | 350.482 | .02812 | 4.696 | 19.97 | 11,789.8 | 422 | 3 |
| 409 | Baseline | 352.819 | .05919 | 41.532 | 134.22 | 0 | 621 | 1 |
| 409 | 10% | 336.570 | .03921 | 202.842 | 990.23 | 12,601.2 | 360 | 3 |
| 409 | 5% | 342.345 | .03661 | 187.326 | 951.31 | 10,821.9 | 310 | 3 |

Hunger is need-weighted at the ending boundary. Operator work and margins are
cumulative; margin excludes financing/capital. Food is dispatched, not necessarily
delivered or consumed. Across the six runs, absolute relative cash residual is
at most 2.06e-7. More shipping or conserved money is not sufficient evidence of
successful economic recovery.

The smaller share softens the adverse hunger result in seed 1024 while retaining
new trade in the previously disconnected seeds. It still does not restore that
seed's population, and seed 409 trades lower ending hunger against lower population.
Seed 256's workshop operators remain very small despite the trade improvement.
The circulation, productive-firm and abandoned-deposit objectives remain open.

## Verification

Four focused harbor tests pass for both shares, including source/work limits,
no premature commission, serialization continuation, repeat-boundary behavior,
selection of only an eligible pair, and removal of eligibility when surplus,
buyer cash or the open lane is absent. A separate GPU production fixture protects
prepaid industrial work against harbor reservations and checks adaptive staffing
without inventing output or workers. The final ordinary suite passes
195 tests with 151 hardware tests ignored; relevant GPU tests were run explicitly.
Native builds and strict library Clippy pass. Raw outputs and rejected variants
remain under ignored `output/selective-harbor*-screen/` directories.

## Century follow-up and limits

Extend the adverse seed 1024 and the initially favorable seed 256, each with a
matched baseline and the 5% candidate, to 1,200 months. All four runs completed.
These are extensions of tuning seeds, not independent held-out validation.

| Seed | Arm | Population | Ending hunger | Operator work | Operator margin | Food dispatched kg |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | Baseline | 119.293 | .03532 | 11.219 | 47.42 | 8,844.7 |
| 1024 | Selective | 117.815 | .01519 | 12.306 | 48.34 | 10,044.6 |
| 256 | Baseline | 229.014 | .03279 | 4.500 | 18.50 | 0 |
| 256 | Selective | 218.345 | .01584 | 4.696 | 19.97 | 12,772.3 |

The largest absolute relative cash residual in these four runs is 2.16e-7.
Seed 1024 has no additional ordinary food dispatches or operator work in the
second fifty years under either arm. Seed 256's early population advantage
reverses, although ending hunger remains lower. Ending hunger among surviving
households does not establish higher lifetime welfare; population loss can itself
reduce pressure. This is why the retained feature is described as a finite route
connection capability, not a solved circulation policy or universal improvement.

The original 10% candidate is not retained. The final source avoids computing
the pure eligibility forecast outside annual boundaries and reuses the market's
shared civilian ration constant. Those cleanups do not change annual selections.
The control is checked by the focused eligibility test; broader balance remains
under review. Century outputs are ignored under
`output/selective-harbor-century-screen/`. No runtime comparison is claimed,
because some verification compilation/tests overlapped frozen experiment runs.

Next inspect why existing connections stop carrying useful trade, and why small
operators have almost no repeat customers even after connections open. Do not
interpret additional port assets or early cargo as proof of viable private firms.
Abandoned stock recovery remains available; working unmined abandoned deposits
still requires a separate conserved extraction and travel path.

The final executable also ran a paired fifty-year seed-256 catalog control.
Both arms applied a catalog update; only `food_connection_investment` differed.
Disabled reproduces baseline population 337.5712832, zero ordinary food dispatch
and 651 no-route requests. Enabled reproduces the 5% candidate's population
350.4822661, 11,789.846117 kg dispatched and 422 no-route requests. This brings
the comparison count to twelve completed runs. Controls are in ignored
`output/selective-harbor-control-screen/`.
