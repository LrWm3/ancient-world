# Two people competing for a productive intermediary

Implemented bounded experiment. Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example intermediary
```

The example uses CubeCL CPU settlement for months 2–13. Month 1 admission and
crop starts use the existing reference-backed two-plot fixture, as in the wood
market experiment. It keeps citizenship, land agreements, annual taxes, farming,
wood collection, nutrition/warmth consequences and storage. No price, borrowing,
barter or negotiation is introduced.

## Scenario and observed result

Two admitted people each start the comparison with five grain, two fuel and an
active crop. The shared wood pool starts with three units, has capacity three,
and regenerates one unit per month. Ordinary collection uses one wood and one
labor to produce two fuel. Each person has three monthly labor; the ongoing crop
uses one during growth.

A discovered one-month process consumes two shared wood and two private labor to
create a wooden cultivation tool. It uses the existing durable outcome machinery.
The tool lasts for two harvest uses. At harvest it lowers labor from two to one
and multiplies grain output by two; recycled seed is not multiplied. Only an
actually completed tool enables the technique, and a newly created tool cannot
be used during the same productive boundary.

With stable priority and identical opening requests:

| Event | Person 88 | Person 89 |
| --- | --- | --- |
| Month 2 private projection | Prefers toolmaking | Prefers toolmaking |
| Joint input resolution | Reserves two wood and two labor | Rejected: only one wood remains |
| Fallback before execution | Keeps accepted work | Chooses wood collection |
| Month 2 completed work | Crop attendance and one tool | Crop attendance and two fuel |
| Month 6 harvest | 16 grain, one seed | Eight grain, one seed |
| Total unmet nutrition, months 2–13 | Zero | One |
| Total unmet warmth, months 2–13 | Zero | Two |
| Terminal by month 14 opening | No | No |

After month 2 work, shared wood is zero. Person 88 has spent all three labor;
person 89 retains one labor. Only person 88 owns a tool. After the first harvest,
that tool has one use remaining. Scarce subsequent regeneration does not grant
the other person a tool just because their initial projection included one.

The later shortfalls matter: atomic resolution prevents overcommitment, but
stable priority and optimistic future-access forecasts do not guarantee healthy
outcomes for both people. This is a mechanism test, not a balanced economy.

## Planning and resolution

`intermediary::Experiment` is an opt-in driver around `Simulation`. It supplies a
plan at the existing Acquire boundary and delegates all other phases to the
existing coordinator. `Experiment::run_months` preserves this driver; calling
`Simulation::run_months` directly would use the ordinary planner instead.
Cloning the experiment retains the simulation, dated work, ranking policy, seed,
and diagnostic rounds for checkpoint continuation.

The planner uses catalog connections rather than occupations or resource IDs:

1. Discover ordinary processes that can serve current needs through the existing
   need-chain lookup. Also discover single-month durable creation processes whose
   equipment enables a technique on a need-serving producer. Already usable tools
   suppress another creation candidate.
2. Compare waiting and each feasible action using the existing consequence score.
   Forecasts include actual process settlement, equipment creation/use, need
   consequences, future crop work and obligations. They use a 12-month horizon;
   future scripted shocks are hidden. Tied consequence/work scores prefer better
   immediate need-buffer coverage, then stable action identity.
3. Each person evaluates against the same opening snapshot and existing crop
   work. These forecasts condition on obtaining the inputs. They are recorded as
   projections, never inserted as holdings or future delivery promises.
4. Collect the selected actions' immediate stock and labor prerequisites. Use
   `resolution::ConditionalBundle` with a separately chosen ranking policy.
   Existing crop attendance has already been reserved. Domain preview additionally
   checks rights, permissions, storage and actual process feasibility.
5. In the same ranking order, rejected applicants get one fallback decision
   against retained work, excluding the action just rejected. Trial work cannot
   consume another applicant's reservation or spend this boundary's new outputs.
6. Pass the final batch to Acquire as `production_plan`. Productive executes the
   exact dated plan through normal settlement. Forecast work never runs live.

The shared resolver's remaining budgets describe the initial allocation window;
subsequent fallback reservations are in the final dated work, not retroactively
inserted into that initial resolution. Each diagnostic round records proposals,
forecast alternatives, allocations and fallbacks. Actual completion, durable
creation and wear remain evidenced by ordinary ledger process transactions.
Diagnostic rounds are not a new externally accepted settlement instruction.

## Checks and controls

The focused tests verify:

- Both initial projections prefer tools; only one complete input bundle clears.
- The loser switches to fuel, keeps unspent labor, and gains no speculative tool.
- Accepting the dated plan does not create equipment; productive completion does.
- Altered dated work and stale execution cannot change state.
- Both existing crops continue. The first harvest is 16 versus eight grain, with
  one returned seed each and exactly one tool use charged.
- Zero opening fuel makes both people decline tools in favor of current warmth.
- A tool with no output or labor improvement is declined.
- Four opening wood admits both tool requests against otherwise identical terms.
- CPU/reference runs, reversed participant ordering, monthly execution and cloned
  checkpoint continuation agree. Seeded lottery can change the recipient while
  keeping the opening projections unchanged.

## Limits

This deliberately supports at most four participants and 16 candidate actions
per person. Each person initially chooses one new action, followed by at most one
fallback if rejected. Existing work continues first. Multiple new actions,
alternative bundles, future input delivery, multi-step acquisition chains and
joint future-market forecasts are not implemented.

The tool is made directly from the shared environmental wood account, just as
collection draws from that account. This is not yet a separate wood-purchase
agreement followed by a contingent manufacturing agreement. It establishes the
first intermediary link: secured inputs → completed durable → later productive
benefit. The tool's useful connection is discovered from catalog techniques; the
material/labor quantities and productivity benefit are explicit scenario data.

Forecasts assume future access based on observed supply and omit other people's
unknown future demands after the immediate work. Later scarcity can invalidate
those expectations; it never creates a secured input. The experiment also uses
manual first-stage service requirements as conservative new-action claims when
an optional technique could reduce them. It does not optimize all possible
technique bundles. Existing scenarios retain their own planners unchanged.

## Validation record

The CPU example completed 12 comparison months. Targeted regression execution
passed 35 tests across intermediary planning, resource resolution, wood allocation,
equipment, process offers and opportunities. The final three-test intermediary
run also checks that the cold control explicitly selects fuel collection.
Clippy (`--all-targets -- -D warnings`), formatting, diff checks and the repository
artifact policy passed. Logs remain under ignored `output/economics/`.


An opt-in [access-learning extension](ACCESS-LEARNING.md) now discounts future
speculative shared supply using recent realized access. The original optimistic
mode remains the default; the controlled comparison reports no improvement in
completed food/warmth outcomes for the tested settings.


An independent opt-in [consequence ranking mode](CONSEQUENCE-PRIORITY.md) uses
acceptance-versus-denial forecasts to rank submitted bundles. It protects urgent
warmth in the tested contested-input scenario while leaving investment admissible
when immediate needs are covered; other markets retain their existing ranking.
