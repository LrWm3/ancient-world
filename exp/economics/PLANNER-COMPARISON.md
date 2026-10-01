# Bounded planner comparison

The first comparison supports keeping best-first composition as an experimental
option, **not replacing the default**. It discovers useful household arrangements,
but both searches regress on some controls and a larger horizon is not reliably
better. The market-trade adapter remains missing, so the original P0–P3 exit
criteria are not all satisfied.

Settings were frozen in [the benchmark manifest](PLANNER-BENCHMARK.md), commit
`a1627fd`, before search implementation. This report implements the first review
point in [the research plan](PLANNER-EXPERIMENTS.md); P4–P8 have not been started.
Implementation and focused verification are at `c6f774b`. Historical v1
verification at `a1b99fa` remains separate from this experiment.

## What was implemented

`composition` exposes dated, read-only membership/land/process descriptions,
including nominal stage inputs/services, seed/crop outputs, annual payments and
original consequence terms. Descriptions use the existing catalogs and agreement
schedule. Town listings expose current admission, lot terms and prior completed
price/volume, but do **not** pretend those observations are executable orders or
promised future buyers. Storage, rights, legal permissions and household allocations
remain authoritative in the observation snapshot and ordinary resolver.

Two strategies share one package evaluator:

- Beam search keeps eight partial packages at each depth.
- Best-first search orders partial packages by relevant production, prerequisites,
  output delay and resource pressure. This heuristic is optimistic and is not an
  optimality guarantee.

Both search backward from needs through consumption/input recipes, compose up to
six current-boundary offer requests, then validate forward with `offers::prepare`.
One process lot per definition/person/boundary is the finite action discretization.
Existing work and a no-new-work candidate are included. Feasible packages roll
through the existing simulator with ContinuingFirst continuation; only the chosen
current package is published. Both strategies use the existing consequence score,
with aborted processes recorded as broken commitments. Ending active processes,
land payment schedules and arrears are retained in the result.

These are **current package searches with dated consequences**, not arbitrary
multi-month schedule search. The caller replans at Acquire each month; forecasts
use the simpler fixed continuation. `Scope::Household` requires an explicit set
of consenting members for private member requests. Membership alone is not that
consent. The household envelope still controls delegated hours and shared stocks.
No charter, scheduler or allocation default changed. The legacy household
ConsequenceAware guard remains in place.

## Main results

All rows below cover 24 realized months. New-search limits are 256 attempted
expansions and 32 full forecasts per decision, with a 12-month forecast.
Each cell is **completed crops / accumulated food deficit / terminal people**.
A terminal person stops generating ordinary need demand, so a smaller raw deficit
can accompany a worse outcome; terminal outcomes must be read alongside deficits.

| Fixture | Existing control | Beam | Best-first |
| --- | --- | --- | --- |
| B1: citizenship → land → farming | 3 / 0 / 0 | 3 / 0 / 0 | 3 / 0 / 0 |
| B2: seed without land | 0 / 6 / 1 | 0 / 6 / 1 | 0 / 6 / 1 |
| B2: land without seed | 0 / 6 / 1 | 0 / 6 / 1 | 0 / 6 / 1 |
| B2: no seed, land or wood | 0 / 2 / 1 | 0 / 2 / 1 | 0 / 2 / 1 |
| B3: competing crop schedules | 4 / 0 / 0 | 3 / 0 / 0 | 3 / 0 / 0 |
| B5: household, two seeds | 0 / 12 / 2 | 6 / 0 / 0 | 7 / 0 / 0 |
| B5: household, one shared seed | 0 / 12 / 2 | 3 / 6 / 0 | 3 / 6 / 0 |

All these runs finish with zero land arrears and zero aborted crops. Warmth deficit
is zero except B2's full scarcity control, where it is six for every strategy.
This does not make scarcity successful: its person reaches the terminal state
four months earlier than in the missing-land/seed controls.

The B5 control is the existing **fixed household driver**, which does not discover
these missing prerequisites; it is not a failing run of the already verified v1
person–household scenario. The new strategies find the missing membership/land
arrangements without supplying a chosen package. Their first-month searches have
only posted terms and explicit member consent.

B3 is a real regression against the individual legacy planner: it ends with nine
grain versus one for both new searches. In the focused witness, the two crops fit
total six-month labor but collide at harvest. Both new searches prefer a package
with a better forecast than the simultaneous conflicting starts. Avoiding that
failure is not enough to discover the legacy planner's better realized schedule.

B5 ends with 16 pooled-plus-private grain under beam and 24 under best-first.
With one seed, both avoid terminal outcomes but incur six units of food deficit
and finish without grain. No extra seed or private resources are invented to
make that control succeed.

## Cost and budget sensitivity

Approximate release-build timings from one local reference-backend run, excluding
compilation; these are small-fixture measurements, not a scalability benchmark.
Foreground test/build activity can affect absolute wall times. No wall clock enters
search decisions.

| Fixture | Existing control seconds | Beam seconds | Best-first seconds | Beam / best-first full forecasts over run |
| --- | --- | --- | --- | --- |
| B1 | 0.144 | 0.072 | 0.071 | 64 / 64 |
| B3 | 0.175 | 0.185 | 0.191 | 173 / 173 |
| B5 two seeds | 0.017 | 2.385 | 2.276 | 176 / 168 |
| B5 one seed | 0.016 | 6.927 | 4.649 | 484 / 356 |

B1 uses 107 attempted expansions and 768 forecast-months in either new search.
The legacy decision ledger retains 156 successful forecasts spanning 2,028 months;
it does not count discarded full rollouts separately, so those are a lower bound
on its actual search work. Its annual-contract minimum expands the requested
12-month horizon to 13 months. It also chooses work priorities and automatic
work proposals, while the new searches compose explicit lots. These comparisons
therefore assess whole policies, not identical-action-set algorithm efficiency.
The exhaustive B1 test is the fair, finite action-set reference for the two new
searches; both match its best score at the large budget.

Budgets were also run at 64 expansions / 8 forecasts and 1,024 / 128, keeping
physical settings unchanged:

- B1 is unchanged across all three budgets.
- At eight forecasts, **both household searches fail to discover a viable
  arrangement**: no crops and both members become terminal. Exhaustion is recorded,
  not interpreted as proof that no useful arrangement exists.
- At eight forecasts, B3 beam gets four crops but one food deficit; best-first gets
  three crops without deficit. At 32/128 both get three without deficit.
- Raising forecasts from 32 to 128 does not improve B5 realized outcomes. Best-first
  does more work (264 versus 168 forecasts in the two-seed case); beam still prunes
  to width eight and returns the same outcome.

At the standard budget, two-seed B5 has two exhausted/pruned monthly searches for
beam and one for best-first. Their peak live search-node counts are 16 and 12,
respectively, including the current layer before pruning. This is a node count,
not a measurement of total simulator/forecast memory.

## Horizon sensitivity and diagnosis

Same 256/32 budget, same 24 realized months. Cells retain the crops / food deficit /
terminal people convention.

| Household forecast horizon | Beam | Best-first |
| --- | --- | --- |
| 6 months | 0 / 12 / 2 | 7 / 0 / 0 |
| 12 months | 6 / 0 / 0 | 7 / 0 / 0 |
| 24 months | 5 / 2 / 0 | 4 / 6 / 0 |

Opening diagnostics make the beam failure inspectable. At six months it chooses
no new requests after evaluating 24 packages and pruning nine nodes. Best-first
finds membership, land and an explicit crop start within 32 forecasts. At twelve
months beam chooses membership and land at opening, while best-first also starts
a crop. With a 24-month forecast, best-first instead opens two land agreements
and requests wood work; its forecast contains later aborted crops even though
actual monthly replanning avoids crop abortion.

The observed mismatch demonstrates why a longer fixed-policy rollout is not the
same as better prediction of a replanning agent. Beam pruning and the inherited
score also matter: the latter measures individual ending buffers and does not
value household-held buffers as a complete collective balance sheet. These are
limitations to isolate in another paired experiment, not reasons to change the
frozen fixtures until this result looks better. Future contracts remain inspectable
but there is not yet a general terminal liability valuation beyond the horizon.

## Competition and the market boundary

B6 creates each person's application independently against the same opening.
StablePriority and PriorityLottery then resolve the same applications. Across
seeds 0–9 and held-out 10–19, exactly one person receives the plot; reversing
application order preserves the batch. The loser retains their seed and performs
lawful wood fallback. Search neither awards the plot nor reserves it twice.
StablePriority awards person 88 all ten plots in each seed group. PriorityLottery
awards 88 three and 89 seven in each group; equal turns were not guaranteed or
used as a correctness criterion. These are opening-allocation checks, not a
24-month economic study of B6.

B4's existing reciprocal-market control produces zero food deficit, four warmth
deficit and 16 grain traded over 24 months; wood volume is zero. Removing market
access for people 91/92 yields zero grain traded and 26 warmth deficit, still zero
food deficit. Because these people can also sell under adaptive order selection,
this is an access-removal control, not a pure isolated buyer-demand intervention.
It does not establish a viable wood-sale-to-food-purchase chain.

Both composition strategies reject this driver explicitly. Town listing
**observation is implemented; order composition is not**. B2's feasible buy
alternative and B4's requested produce-versus-buy search comparison remain open.
No baseline market decisions are attributed to either new search.

## Verification and reproduction

The focused run passed **59 tests across ten integration suites**: composition,
search, household offers, process offers, competition, forecast context, planning,
opportunities, membership and agreements. Final all-target Clippy, formatting,
local document links and the repository artifact check also passed. The full
v1 suite and ignored population stress were not rerun: defaults and existing
execution code are unchanged, and no default replacement is proposed.

Focused tests cover the exhaustive B1 reference, atomic stale-state/term rejection,
future fixture information exclusion, zero-expansion fallback, input reordering,
missing complements, the harvest collision, explicit household consent, market
observations and contested access. The household comparison runs through the
ordinary accounting observer on CPU and reference, with monthly replanning and
continuation from a checkpoint after acceptance.

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example planner_comparison
FORECASTS=8 EXPANSIONS=64 cargo +1.92.0 run --locked --release --example planner_comparison
FORECASTS=128 EXPANSIONS=1024 cargo +1.92.0 run --locked --release --example planner_comparison
HORIZON=6 cargo +1.92.0 run --locked --release --example planner_comparison
HORIZON=24 cargo +1.92.0 run --locked --release --example planner_comparison
CASE=B4 MARKET=1 cargo +1.92.0 run --locked --release --example planner_comparison
CASE=B5 MONTHS=1 DETAIL=1 cargo +1.92.0 run --locked --release --example planner_comparison
cargo +1.92.0 test --locked --test composition -- --nocapture
```

Redirect local CSV/log output under root `output/economics/`; do not commit it.
`MODE=legacy`, `MODE=beam` or `MODE=best` selects a control; `CASE` selects a fixture.
Legacy forecast counts in CSV are retained successful alternatives, not a complete
counter of attempted evaluations. CPU/reference equality is tested separately;
the performance matrix uses the reference backend to avoid JIT setup effects.

## Review decision

Retain both as opt-in experiments and leave the production default unchanged.
Best-first is the stronger household candidate at the tested middle budget, but
has no demonstrated universal advantage. Before expanding the opportunity catalog,
compare continuation/ending-state assessment against actual monthly replanning
and test whether preserving prerequisite diversity fixes beam's missed chains.
Completing the original market comparison also requires a real bid/ask action
adapter and a feasible, separately controlled wood/buy fixture. Neither issue is
solved by simply increasing the node budget.
