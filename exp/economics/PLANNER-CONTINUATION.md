# Planner continuation and review experiment

Status: opt-in follow-up to [forecast calibration](PLANNER-CALIBRATION.md).
The simulation default, monthly scheduler, household charter, search score and
resource quantities are unchanged. This exercises a bounded portion of P7 in
[the experiment plan](PLANNER-EXPERIMENTS.md); it does not complete that roadmap
or replace the historical [v1 verification](V1-RESULTS.md).
The later [multi-person follow-up](PLANNER-PERSONS.md) adds independently reviewed
persons and shared admission. The settings, results and isolated-scope limitations
below describe this earlier comparison.

Keeping forecast and execution policy aligned removes one source of harvest
timing drift and greatly reduces search work. It does not consistently improve
household outcomes. A more stable plan can preserve a poor allocation; repairing
after an observed shortage can help without recovering monthly search's outcome.

## Policies and execution boundary

`composition::continuation::Controller` separates review frequency from the
existing beam/best-first package search:

| Policy | When it searches | Between searches |
| --- | --- | --- |
| `Monthly` | Every Acquire | Control: ordinary explicit composition selection |
| `RetainRepair` | Initially, when the forecast horizon ends, or when observed context differs from its dated forecast | Use the forecast's cheap `ContinuingFirst` continuation and verify its expected starts |
| `ScheduledReview` | Initially and when the forecast horizon ends | Use the same cheap continuation with actual resources; record deviations without advancing the review |

Review intervals equal the forecast horizon: six, twelve or twenty-four months.
A repair selects a new bounded package and restarts that interval. It is full
reselection, not a minimal edit to the old plan. Both alternatives coincide in
the absence of unexpected observations; shocks distinguish their review behavior.
Scheduled review still reacts to actual availability through the cheap policy.
It never blindly spends a predicted resource.

The stored plan consists of dated observation frames and expected process starts
from the selected forecast. Existing process instances retain their ordinary
stages and obligations. At each live Acquire, fresh preparation previews ordinary
acquisition and productive allocation and attaches that month's dated work plan.
Productive consumes it once through existing settlement. No future forecast batch,
resource grant, permission or household allocation is copied into live execution.
Other phases use `Simulation::step` unchanged.

The controller requires explicit `ContinuingFirst` configuration, an isolated
person or household scope, and every household participant's private consent.
Existing composition guards still apply. Market continuation, active counterparties,
finance composition and changing household membership are outside this comparison.
It does not authorize a person to plan someone else's work.

Deviation detection compares the complete sanitized `ForecastContext`, including
observed state and terms. Future fixture capacity changes remain hidden until
Open makes them observable. This is deliberately conservative full-information
comparison, not agent-private knowledge or a selective economic materiality test.
Initial/monthly receipts have no saved prediction to compare; their false
`observation_changed` flag does not establish forecast accuracy.

Checkpoint the controller **with** the simulation. Cloned continuation is tested;
there is no new durable serialization format. The selected forecast is projected
once more and replayed to obtain observation frames. Receipts count this additional
projection and replay separately from search's forecast budget, as well as the
two-boundary cheap previews. Full context copies are acceptable for these small
fixtures, not evidence of scalable memory use.

## Controlled comparison

`examples/planner_continuation.rs` runs 162 configurations: three unchanged fixtures,
two search strategies, three horizons, three observation conditions and three
review policies. Every run executes **24 actual months** with 256 expansions and
32 full forecasts allowed per search; beam width remains eight. Search uses the
original private-buffer score. Less frequent reviews spend fewer total forecasts;
saved computation is not reassigned to larger searches.

- B3: one person, two crop options competing for dated labor.
- B5: two consenting household members and two pooled seeds.
- B5-one-seed: the same household with one pooled seed.
- No shock, zero labor for person 88 in month six, or all stored grain removed
  at month-six Acquire. The grain removal is an explicit external fixture
  intervention, not a financial transaction or a claim of journal reconciliation.
  The unshocked household separately receives financial-audit coverage.

The matrix uses the scalar reference backend; focused tests also exercise CubeCL
CPU settlement. All arms use the same explicit cheap continuation configuration. Quantities,
permissions, household allocation policy and preferences are held constant.
The stock loss happens before planning and before that month's harvest. These
are deterministic stress cases, not a distribution of plausible crop risks.

## Results

Retain/repair and scheduled review have identical unshocked economic outcomes
in all 18 fixture/search/horizon combinations. These selected results use
**best-first with a twelve-month horizon, over 24 actual months**. Terminal
months are summed agent-months after the terminal condition, not counts of people.
Food deficit alone becomes misleading when a person stops participating after death.

| Case / shock | Policy | Crops | Food deficit | Aborted crops | Terminal months | Ending grain dues | Active crops at end |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| B3 / none | Monthly | 3 | 0 | 0 | 0 | 0 | 2 |
| B3 / none | Retain/repair | 4 | 0 | 0 | 0 | 0 | 0 |
| B3 / none | Scheduled | 4 | 0 | 0 | 0 | 0 | 0 |
| B5 / none | Monthly | 7 | 0 | 0 | 0 | 0 | 1 |
| B5 / none | Retain/repair | 4 | 0 | 1 | 0 | 0 | 1 |
| B5 / none | Scheduled | 4 | 0 | 1 | 0 | 0 | 1 |
| B5 / labor | Monthly | 3 | 8 | 1 | 0 | 0 | 1 |
| B5 / labor | Retain/repair | 3 | 12 | 1 | 0 | 0 | 0 |
| B5 / labor | Scheduled | 0 | 12 | 1 | 18 | 2 | 0 |
| B5 / stock | Monthly | 7 | 0 | 0 | 0 | 0 | 1 |
| B5 / stock | Retain/repair | 6 | 2 | 1 | 0 | 0 | 0 |
| B5 / stock | Scheduled | 4 | 10 | 0 | 0 | 0 | 2 |
| B5-one-seed / none | Monthly | 3 | 6 | 0 | 0 | 0 | 1 |
| B5-one-seed / none | Retain/repair | 3 | 6 | 0 | 0 | 0 | 1 |
| B5-one-seed / none | Scheduled | 3 | 6 | 0 | 0 | 0 | 1 |

For B3, monthly search harvests in months **6, 13 and 17**, ending with two
unfinished crops. Both alternatives harvest in **6, 12, 18 and 24**, with no
unfinished crops. They restore the regular supply predicted by the selected plan.

The household reverses that advantage. Unshocked B5 monthly search completes
seven crops and ends with 20 pooled grain; both alternatives complete four, abort
one and end with two pooled grain. All meet food needs, but those are materially
different outcomes. At a **24-month** best-first horizon the tradeoff changes:
monthly search has four crops, six food deficit and no aborts; both alternatives
have four crops, zero food deficit and two aborts. Review frequency and horizon
cannot be selected independently of the objective.

After B5's month-six labor loss, immediate repair keeps terminal months at zero,
where scheduled review accumulates 18. Nevertheless, monthly search has fewer
food deficits and impaired months: **8 / 4**, versus repair's **12 / 16**.
After the stock loss, repair also improves on scheduled review, while monthly
search retains the strongest food outcome. B3's labor-loss cases can be fatal
under every policy; a review trigger cannot recreate a lost crop or seed.

Search work falls substantially in the unshocked twelve-month best-first cases:

| Case | Monthly search forecast-months | Retain or scheduled search forecast-months | Extra projection months | Extra replay boundaries | Cheap preview boundaries |
| --- | ---: | ---: | ---: | ---: | ---: |
| B3 | 2076 | 456 | 24 | 164 | 44 |
| B5 | 2016 | 768 | 24 | 164 | 44 |
| B5-one-seed | 4272 | 672 | 24 | 164 | 44 |

These alternatives perform two searches rather than 24, but each also projects
the selected plan and replays it for inspection. The CSV keeps both extra costs
separate; they are not hidden inside the 32-forecast search cap. Wall times are
single-run diagnostics, not a controlled performance benchmark. Changes in the
monthly process-start set include legitimate crop/wood transitions and must not
be interpreted as counts of unnecessary abandonment.

The conclusion is to retain both policies as experiments, leaving defaults
unchanged. Consistency repairs one mismatch; it also makes weaknesses of the
cheap household continuation visible.

### Complete matrix of economic outcomes

Each cell is **crops / food deficit / aborted crops / terminal agent-months /
ending grain dues**. All rows cover 24 actual months; horizon controls forecasting
and review intervals. All configurations remain visible, including cases
where lower reported deficits accompany death. Warmth deficit is zero except
B3 labor-shock monthly search at horizons 12 and 24 (both strategies): one unit;
and B5 stock-shock monthly best-first at horizon 24: two units.
The CSV additionally records dated harvests, impairment, active crops, holdings,
buffer scores and computational costs. Detailed review reasons are in the
companion log.

| Case | Search | Horizon | Shock | Monthly | Retain/repair | Scheduled |
| --- | --- | ---: | --- | --- | --- | --- |
| B3 | Beam | 6 | none | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | Beam | 6 | labor | 0 / 6 / 2 / 14 / 4 | 0 / 6 / 2 / 14 / 2 | 0 / 6 / 1 / 14 / 2 |
| B3 | Beam | 6 | stock | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | Beam | 12 | none | 3 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | Beam | 12 | labor | 0 / 6 / 1 / 14 / 2 | 0 / 6 / 2 / 14 / 2 | 0 / 6 / 2 / 14 / 2 |
| B3 | Beam | 12 | stock | 3 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | Beam | 24 | none | 3 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | Beam | 24 | labor | 0 / 6 / 1 / 14 / 4 | 0 / 6 / 2 / 14 / 4 | 0 / 6 / 2 / 14 / 4 |
| B3 | Beam | 24 | stock | 3 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | BestFirst | 6 | none | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | BestFirst | 6 | labor | 0 / 6 / 2 / 14 / 4 | 0 / 6 / 2 / 14 / 2 | 0 / 6 / 1 / 14 / 2 |
| B3 | BestFirst | 6 | stock | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | BestFirst | 12 | none | 3 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | BestFirst | 12 | labor | 0 / 6 / 1 / 14 / 2 | 0 / 6 / 2 / 14 / 2 | 0 / 6 / 2 / 14 / 2 |
| B3 | BestFirst | 12 | stock | 3 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | BestFirst | 24 | none | 3 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B3 | BestFirst | 24 | labor | 0 / 6 / 1 / 14 / 4 | 0 / 6 / 2 / 14 / 4 | 0 / 6 / 2 / 14 / 4 |
| B3 | BestFirst | 24 | stock | 3 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 | 4 / 0 / 0 / 0 / 0 |
| B5 | Beam | 6 | none | 0 / 12 / 0 / 18 / 0 | 0 / 12 / 0 / 18 / 0 | 0 / 12 / 0 / 18 / 0 |
| B5 | Beam | 6 | labor | 0 / 12 / 0 / 18 / 0 | 0 / 12 / 0 / 18 / 0 | 0 / 12 / 0 / 18 / 0 |
| B5 | Beam | 6 | stock | 0 / 12 / 0 / 28 / 0 | 0 / 12 / 0 / 28 / 0 | 0 / 12 / 0 / 28 / 0 |
| B5 | Beam | 12 | none | 6 / 0 / 0 / 0 / 0 | 4 / 0 / 1 / 0 / 0 | 4 / 0 / 1 / 0 / 0 |
| B5 | Beam | 12 | labor | 6 / 0 / 0 / 0 / 0 | 5 / 0 / 1 / 0 / 0 | 4 / 0 / 1 / 0 / 0 |
| B5 | Beam | 12 | stock | 6 / 2 / 0 / 0 / 0 | 6 / 2 / 0 / 0 / 0 | 4 / 10 / 0 / 0 / 0 |
| B5 | Beam | 24 | none | 5 / 2 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 |
| B5 | Beam | 24 | labor | 5 / 2 / 0 / 0 / 0 | 5 / 0 / 1 / 0 / 0 | 3 / 6 / 0 / 0 / 0 |
| B5 | Beam | 24 | stock | 6 / 6 / 0 / 0 / 0 | 5 / 4 / 1 / 0 / 0 | 3 / 16 / 0 / 0 / 0 |
| B5 | BestFirst | 6 | none | 7 / 0 / 0 / 0 / 0 | 5 / 0 / 0 / 0 / 0 | 5 / 0 / 0 / 0 / 0 |
| B5 | BestFirst | 6 | labor | 3 / 8 / 1 / 0 / 0 | 3 / 12 / 1 / 0 / 0 | 3 / 14 / 1 / 0 / 0 |
| B5 | BestFirst | 6 | stock | 7 / 0 / 0 / 0 / 0 | 7 / 2 / 0 / 0 / 0 | 7 / 4 / 0 / 0 / 0 |
| B5 | BestFirst | 12 | none | 7 / 0 / 0 / 0 / 0 | 4 / 0 / 1 / 0 / 0 | 4 / 0 / 1 / 0 / 0 |
| B5 | BestFirst | 12 | labor | 3 / 8 / 1 / 0 / 0 | 3 / 12 / 1 / 0 / 0 | 0 / 12 / 1 / 18 / 2 |
| B5 | BestFirst | 12 | stock | 7 / 0 / 0 / 0 / 0 | 6 / 2 / 1 / 0 / 0 | 4 / 10 / 0 / 0 / 0 |
| B5 | BestFirst | 24 | none | 4 / 6 / 0 / 0 / 0 | 4 / 0 / 2 / 0 / 0 | 4 / 0 / 2 / 0 / 0 |
| B5 | BestFirst | 24 | labor | 4 / 6 / 0 / 0 / 0 | 4 / 0 / 2 / 0 / 0 | 4 / 0 / 2 / 0 / 0 |
| B5 | BestFirst | 24 | stock | 5 / 10 / 0 / 0 / 0 | 5 / 2 / 1 / 0 / 0 | 4 / 10 / 2 / 0 / 0 |
| B5-one-seed | Beam | 6 | none | 0 / 12 / 0 / 18 / 0 | 0 / 12 / 0 / 18 / 0 | 0 / 12 / 0 / 18 / 0 |
| B5-one-seed | Beam | 6 | labor | 0 / 12 / 0 / 18 / 0 | 0 / 12 / 0 / 18 / 0 | 0 / 12 / 0 / 18 / 0 |
| B5-one-seed | Beam | 6 | stock | 0 / 12 / 0 / 28 / 0 | 0 / 12 / 0 / 28 / 0 | 0 / 12 / 0 / 28 / 0 |
| B5-one-seed | Beam | 12 | none | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 |
| B5-one-seed | Beam | 12 | labor | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 |
| B5-one-seed | Beam | 12 | stock | 3 / 16 / 0 / 0 / 0 | 3 / 16 / 0 / 0 / 0 | 3 / 16 / 0 / 0 / 0 |
| B5-one-seed | Beam | 24 | none | 3 / 8 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 |
| B5-one-seed | Beam | 24 | labor | 3 / 8 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 |
| B5-one-seed | Beam | 24 | stock | 3 / 16 / 0 / 0 / 0 | 3 / 16 / 0 / 0 / 0 | 3 / 16 / 0 / 0 / 0 |
| B5-one-seed | BestFirst | 6 | none | 4 / 4 / 0 / 0 / 0 | 4 / 4 / 0 / 0 / 0 | 4 / 4 / 0 / 0 / 0 |
| B5-one-seed | BestFirst | 6 | labor | 0 / 12 / 1 / 18 / 2 | 0 / 12 / 1 / 18 / 2 | 0 / 12 / 1 / 18 / 2 |
| B5-one-seed | BestFirst | 6 | stock | 4 / 14 / 0 / 0 / 0 | 3 / 16 / 0 / 0 / 0 | 4 / 14 / 0 / 0 / 0 |
| B5-one-seed | BestFirst | 12 | none | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 |
| B5-one-seed | BestFirst | 12 | labor | 0 / 12 / 1 / 18 / 2 | 0 / 12 / 1 / 18 / 2 | 0 / 12 / 1 / 18 / 2 |
| B5-one-seed | BestFirst | 12 | stock | 4 / 14 / 0 / 0 / 0 | 3 / 16 / 0 / 0 / 0 | 3 / 16 / 0 / 0 / 0 |
| B5-one-seed | BestFirst | 24 | none | 3 / 8 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 |
| B5-one-seed | BestFirst | 24 | labor | 3 / 8 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 | 3 / 6 / 0 / 0 / 0 |
| B5-one-seed | BestFirst | 24 | stock | 3 / 16 / 0 / 0 / 0 | 3 / 16 / 0 / 0 / 0 | 3 / 16 / 0 / 0 / 0 |

## Verification and limits

The focused suite passed **106 tests across 15 suites**, zero failures or ignored
tests. Six new tests cover:

- Exact stable forecast/realized state and report equality through a review
  horizon, and review at the following dated boundary.
- Monthly control equivalence to the existing composition API.
- Hidden future labor loss, observed repair and lawful failure under scarcity.
- Observed seed loss without reuse of saved grants.
- CubeCL CPU/reference equality, cloned checkpoints with pending productive
  work, and separate double-entry household reconciliation.
- Unsupported configuration and missing private mandate rejected without mutation.

Strict all-target Clippy, formatting and the repository artifact check passed.
The 162-run matrix is additional economic evidence, not 162 independent test
functions. The full v1 suite and 32-person population regression were not rerun;
no replacement default is proposed.

Keeping a continuation policy consistent is useful, but consistency alone does
not make it economically sound. The next bounded question is whether the cheap
household continuation can preserve critical crop work and need coverage without
expensive monthly reselection. A later selective review trigger could ignore
irrelevant changes and respond to threatened obligations. Active trading
counterparties remain a separate adapter/coordination problem.

## Reproduce

From `exp/economics` (raw outputs remain ignored):

```sh
cargo +1.92.0 run --locked --release --example planner_continuation > ../../output/economics/planner-continuation.csv 2> ../../output/economics/planner-continuation.log
cargo +1.92.0 test --locked --release --test planner_continuation --test composition --test planner_calibration --test composition_market --test town_market --test production_market --test cooperation --test acquisition --test household_market --test household_offers --test process_offers --test forecast_context --test planning --test search --test financial_offers
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
cargo +1.92.0 fmt --check
```

Optional `CASE=B3`, `CASE=B5` or `CASE=B5-one-seed` limits the comparison.
Run `python3 scripts/check_repository_artifacts.py` from the repository root.
