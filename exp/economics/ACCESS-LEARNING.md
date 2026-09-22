# Learning about access to shared resources

Implemented opt-in extension of the [intermediary experiment](INTERMEDIARY.md).
The result is mixed: the estimator responds to losses and changes forecasts,
but did not improve completed outcomes in the tested 12-month comparisons.

Run the CPU comparison from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example access_learning
```

`Experiment.access_mode` selects `Optimistic` (the existing default) or `Learned`.
Both modes collect the same kind of realized-access observations. Learning affects
future projections, never eligibility, ranking, prices, current stock or accepted
reservations. The usual Acquire and Productive boundaries remain in place.

## Observation and update

The memory is keyed by person and shared resource account, not occupation or
activity. A tool request and a fuel request drawing from the same wood account
inform the same estimate. Record quantities of shared inputs actually consumed,
not tool/fuel output quantities.

After successful Productive settlement, record each person's initial requested
quantity and realized use. Where a rejected action is replaced by a fallback,
the requested quantity is the maximum of the two alternative requests for that
account. Thus a two-wood tool request followed by a successful one-wood fuel
fallback records **two requested, one received**. It does not count three wood of
demand, or mistake the fallback for complete fulfillment of the tool request.
Permission/storage/domain rejections do not contribute a failed initial request.
No request produces no observation, rather than a failure or a success.

Only new actions from the current decision are measured. Continuing commitments
are outside this allocation window. Forecast simulations never update memory.
The experiment stages physical settlement and memory together; a failed update
publishes neither. Dated observations reject duplicate/stale boundaries. Cloned
checkpoints include both pending productive work and access history.

At month m, use observations from the preceding six months:

```text
expected fraction = (2 + received units) / (2 + requested units)
```

The two successful prior units start an unobserved account at full expected
access and soften isolated rejections. Success increases the estimate; old
losses expire. With no recent evidence it returns to the optimistic prior.
This is a deliberately simple quantity-weighted heuristic, not a calibrated
probability or an entitlement to a fraction of global supply.

## Applying the expectation

Each decision retains its dated estimate snapshot. All initial candidates and
fallback candidates at that boundary use the same history; this month's outcome
cannot retrospectively affect its own forecast.

The current candidate's work is first checked against actual opening budgets
and retained reservations. Only afterward, in the private forecast, scale
uncommitted remaining shared stock and subsequent regeneration by the estimate.
Carry fractional units between forecast months: a 3/4 fraction of a one-unit
monthly flow supplies the deterministic sequence 0, 1, 1, 1, instead of truncating
every month's supply to zero. Expectations stay fixed within one projection and
are recomputed at the next live decision.

These private forecast budgets are not changes to the physical world, transfers,
escrow, or promised future deliveries. The regular resolver still grants whole
feasible bundles from actual resources. Private fractional timing is an
approximation; it does not model the exact timing of future competing bids.

## Controlled CPU comparisons

All runs cover months 2–13 of the same admitted two-person scenario. Within each
pair, opening state, replenishment, ranking policy and seed are identical. Scarce
supply starts with three wood, capacity three, regeneration one/month. Ample
supply starts with six wood, capacity six, regeneration three/month. Tool terms,
needs and taxes are unchanged.

Totals across both people; **optimistic and learned modes gave the same values**:

| Allocation | Supply | Unmet nutrition | Unmet warmth | Tools created | Harvest grain | Deaths |
| --- | --- | --- | --- | --- | --- | --- |
| Stable priority, seed 7 | Scarce | 1 | 2 | 1 | 24 | 0 |
| Lottery, seed 7 | Scarce | 0 | 1 | 1 | 32 | 0 |
| Lottery, seed 19 | Scarce | 0 | 0 | 1 | 32 | 0 |
| Each of the three policies/seeds | Ample | 0 | 0 | 2 | 32 | 0 |

Under scarce stable priority, person 89's expected fraction goes from 1 at month
2 to 3/4 at month 3 after partial access, then 3/9 at month 8 after repeated
losses. It recovers to 5/8 by month 13. Later forecast scores differ from the
optimistic forecasts. At month 9 the learned person first requests fuel, loses,
and falls back to planting; the optimistic person requests planting directly.
The final work and measured shortfalls are unchanged in this comparison.

Neither mode buys an imaginary tool, and the ample control still produces both
useful tools. These runs do not demonstrate a welfare improvement, nor prove
learning cannot discourage good investments elsewhere. With few alternatives
and unchanged allocation rules, knowing access is unreliable may not provide a
better feasible action. More broadly, request fulfillment is a censored signal:
agents only learn about requests they submit, and a changing mix of large and
small requests can move the estimate without a change in market supply.

## Validation and limits

Focused tests cover losses, recovery through successful access, expiry, inactive
demand, account/person isolation, invalid and duplicate observations, fractional
supply and wide arithmetic. Integration tests check learning only after execution,
fallback accounting, read-only previews, forecast changes after accumulated losses,
CPU/reference equality, participant order, batching and checkpoint continuation.
An ample control compares actual ledgers and state between forecast modes.

This remains scoped to the intermediary driver and its shared input accounts.
It does not learn prices, counterparties, permission risk, or future contractual
delivery reliability. The six-month window and two-unit prior are explicit
experiment parameters, not empirically estimated coefficients. No parameter
search was performed to make the comparison look beneficial. Raw comparison
output and test logs remain under ignored `output/economics/`.

Validation: 11 focused tests passed across access expectations, intermediary
planning and resource resolution. The CPU comparison completed all 12 runs
(two forecast modes × three policy/seed settings × two supply settings).
Clippy with warnings denied, formatting, diff checks and the repository artifact
policy passed. A failed observation update is also tested to leave physical state
and ledger unchanged.
