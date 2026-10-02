# Independent person planning with shared admission

Status: implemented opt-in follow-up to [plan continuation](PLANNER-CONTINUATION.md).
Two and four independent persons can search, acquire separate plots and complete
repeated harvests through the existing CPU settlement path. They can also compete
for the same plot or wood without publishing duplicate grants. This extends the
bounded P3/P7 experiments in [the planner plan](PLANNER-EXPERIMENTS.md); defaults,
the monthly scheduler and the historical [v1 verification](V1-RESULTS.md) are unchanged.

The subsequent [active exchange comparison](PLANNER-EXCHANGE.md) extends this
coordinator to monthly independent production and order choices. This report retains
the earlier non-market settings, results and exclusions.

## Independent intentions, shared resources

`composition::continuation::persons::Persons` contains one `Controller` per
participant. At Acquire, each reads the same opening state in a private forecast
branch containing its own production, needs and consumption. Other persons remain
named owners/counterparties, but the search cannot choose their work or count their
hypothetical production. Each controller has its own forecast budget and review date.

The branch can observe shared stocks and available offers. Its forecast assumes
access if those remain available; it does not predict competing intentions. These
forecasts are conditional proposals, not claims that every person's forecast can
happen together. Counterparty agreements remain in the branch; this is not an
information-access boundary or a general counterparty belief model.

The coordinator then:

1. Extracts intended membership, land and new process requests from every proposal.
   No forecast effects, reservations or future production are copied into live state.
2. Retains equally scored packages actually evaluated by each person's search.
   If its first plot is taken, another equally valued plot can be tried without a
   centralized search inventing a different plan. The pilot permits at most one
   new land agreement per reviewed package; this is an adapter limit, not a law.
3. Uses the existing `allocation` policy to order unit package claims. All claims
   have equal priority; `StablePriority` breaks ties by identity, while `Lottery`
   uses the configured seed and month. `PriorityLottery` has the same ordering as
   lottery here because priorities are equal.
4. Tries each person's primary package, then its equal-score alternatives, against
   all already admitted requests using common `offers::prepare`. Prerequisites
   precede process requests. Rights, seed, labor, shared pools and storage must fit
   jointly. A failed package acquires nothing and spends no inputs.
5. Gives rejected persons a cheap fallback opportunity, such as wood collection,
   in admission order. Each request is checked against existing work and all
   accepted packages. This does not run another full forecast. Rejection also
   triggers a fresh personal search at the next Acquire, even under scheduled review.
6. Publishes one ordinary Acquire batch with dated work for all persons. Productive
   executes that work once. Other phases continue through `Simulation::step`.

Existing active processes receive the configured `ContinuingFirst` treatment even
when every new package is empty. Timing is unchanged; the explicit allocation
policy decides whose competing new requests are tried first.

This is ordered whole-package admission, not optimal matching. It does not revisit
a winner to accommodate a later applicant, accept a lower-scored substitute, or
negotiate prices. Equal alternatives are limited to those explored within the
person's own bounded search. Scarcity can still reject a feasible individual plan.

`Round` records primary proposals, explored alternatives, accepted packages,
allocation receipts, fallback requests and rejection reasons. A package rejected
on its first choice but admitted on an alternative is distinguishable by comparing
its proposal and accepted requests. Allocation diagnostics live in the coordinator;
the economic ledger continues to contain ordinary settled batches.

## Review and continuation

The three existing modes remain available: `Monthly`, `RetainRepair`, and
`ScheduledReview`. Each person's saved expectation is compared using a view that
ignores unrelated private balances, global batch numbers and globally assigned
process IDs. Own resources, commitments, rights and observed public availability
remain relevant. A private stock loss can therefore trigger only its owner's
review. The comparison view is never executable state and is not a privacy system.

Retained work is always prepared against live availability. A different admitted
plot or changed shared stock may invalidate the saved forecast at the next review;
no future promise overrides actual resources. Scheduled review still checks live
feasibility and reacts to explicit admission rejection.

Construct `Persons::new(agents, strategy, budget, review, allocation_policy, seed)`
and call `persons.step(&mut sim)` instead of directly stepping the simulation.
It requires exactly one controller per participant, `ContinuingFirst`, and the
supported person-only drivers. Unsupported households, markets and financial drivers
are rejected before advancing. Checkpoint the coordinator with the simulation:
review frames, rejected intentions and admission history must resume together.
Cloning both is tested; durable serialization is not added.

## Controlled comparison

[The runner](examples/planner_persons.rs) executes 48 scenarios and emits 144
per-person rows:

- Two or four persons, with either one plot or one plot per person.
- Stable identity ordering or seeded lottery, seed 7.
- All three review modes; best-first search with 256 expansions, 32 complete
  forecasts and a twelve-month horizon per personal search.
- Twenty-four live months, with no shock or zero available labor for person 88
  during month 6. The future override is hidden until observed at Open.
- Identical per-person opening endowments; shared wood starts at twelve units per
  person and regenerates one unit per person each month. Plots vary independently.
- Reference execution for the matrix; CPU equivalence is checked separately below.

For seed 7, both allocation policies produced identical per-person CSV results.
This is not policy equivalence: a separate scarce-wood test over seeds 0–9 produces
both possible winners. All three review modes have the crop, food, terminal and
arrears results below. Warmth differs in the shock cases, as noted afterward.

| People | Plots | Month-6 shock | Completed crops by person (88 onward) | Total food deficit | Terminal person-months | Ending unpaid dues |
| --- | --- | --- | --- | --- | --- | --- |
| 2 | 2 | No | 3, 3 | 0 | 0 | 0 |
| 4 | 4 | No | 3, 3, 3, 3 | 0 | 0 | 0 |
| 2 | 1 | No | 3, 0 | 6 | 14 | 0 |
| 4 | 1 | No | 3, 0, 0, 0 | 18 | 42 | 0 |
| 2 | 2 | Yes | 0, 3 | 6 | 14 | 2 |
| 4 | 4 | Yes | 0, 3, 3, 3 | 6 | 14 | 2 |
| 2 | 1 | Yes | 0, 0 | 12 | 28 | 2 |
| 4 | 1 | Yes | 0, 0, 0, 0 | 24 | 56 | 2 |

Food deficit sums reported unmet nutrition quantities; terminal person-months count
monthly reports in a terminal state, not distinct deaths. Dues are native grain
units. Monthly search incurs one unit of warmth deficit in each shock scenario;
retain/repair and scheduled review incur none. Every no-shock scenario has zero
warmth deficit.

With one plot per person and no shock, all people acquire distinct rights at the
opening and complete three harvests. No one's seed is spent on another's crop.
With one plot, the rejected people can collect wood, but have no continuing food
source. In this fixture the plot goes to person 88 under both policies. Losing that
person's labor at the first harvest aborts the crop; neither review policy can
recover its lost seed or supply missing food. Other plot holders continue harvesting.
There is no trade, food sharing or automatic replacement seed in this adapter.

| People | Plots | Shock | Full searches: monthly | Retain/repair | Scheduled review |
| --- | --- | --- | --- | --- | --- |
| 2 | 2 | No | 48 | 6 | 4 |
| 4 | 4 | No | 96 | 12 | 8 |
| 2 | 1 | No | 48 | 5 | 5 |
| 4 | 1 | No | 96 | 11 | 11 |
| 2 | 2 | Yes | 48 | 7 | 4 |
| 4 | 4 | Yes | 96 | 13 | 8 |
| 2 | 1 | Yes | 48 | 6 | 5 |
| 4 | 1 | Yes | 96 | 12 | 11 |

Fewer searches are not a measured runtime speedup. The CSV separately counts
search forecast-months, selected-plan projection months and frame replay steps.
It does not count all cumulative admission previews. For example, four persons
with four plots and no shock use 3,648 search forecast-months under monthly review,
1,248 plus 144 projection months under retain/repair, and 1,152 plus 96 under
scheduled review. The latter two also replay 984 and 656 phase steps respectively.
Full context copies and repeated preparation are not established as scalable.

## Verification and reproduction

The focused regression passed **132 tests across 20 suites**, including **9 new
multi-person tests**. These cover:

- Same-opening applications, one contested plot, retained loser seed and fallback.
- Repeated two-person harvests under all three review modes.
- Four simultaneous distinct plot admissions and repeated harvests.
- Thirty-two applicants for one plot at the first Acquire/Productive boundary.
  This is not a 32-person annual or lifetime run.
- Shared wood consumed once, with the lottery able to change the winner.
- Reference versus CubeCL CPU settlement, reordered independent inputs, continuous
  versus checkpoint-resumed monthly stepping, and equal separate financial audits.
  The checkpoint includes already reserved Productive work.
- Atomic failed publication, invalid controller coverage/configuration and unsupported
  scopes; a private grain loss causing only its owner's observed-deviation review.

The final adapter-limit guard was also rechecked with all nine new tests. Formatting,
strict all-target Clippy, documentation links and repository artifact checks pass.
The full v1 release runner and previous annual 32-person stress scenario were not
rerun for this change; their historical results remain separate.

From `exp/economics`, keep generated output under the ignored root `output/`:

```sh
mkdir -p ../../output/economics
cargo +1.92.0 run --locked --release --example planner_persons > ../../output/economics/planner-persons.csv 2> ../../output/economics/planner-persons.log
cargo +1.92.0 test --locked --release --test planner_persons --test planner_continuation --test composition --test planner_calibration --test composition_market --test town_market --test production_market --test cooperation --test acquisition --test household_market --test household_offers --test process_offers --test forecast_context --test planning --test search --test financial_offers --test competition --test allocation --test resolution --test intermediary
cargo +1.92.0 fmt --check
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

## Remaining boundaries

This adds independently planning persons competing for existing opportunities. It
does not combine multiple households, negotiate active person-to-person trades,
compose finance, optimize global welfare, or teach forecasts the probability of
obtaining contested resources. The scalar score and the cheap continuation policy
are unchanged; neither is demonstrated to be an optimal decision policy.

A useful next bounded experiment would add two active trading counterparties to
this same collection/admission boundary, retaining separate budgets and exposing
failed expected trades to each person's next forecast. That would address the
missing food outlet for landless persons only if the supplied opportunities and
exchange terms make such a livelihood feasible. This follow-up is now implemented
in [active exchange](PLANNER-EXCHANGE.md), with mixed economic outcomes and explicit
forecast-versus-fill receipts; learning counterparty submission behavior remains open.
