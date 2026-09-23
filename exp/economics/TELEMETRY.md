# External metrics and committed-event logs

Implemented first step. Decision explanations and specialized subsystem observers
are deferred. The observer wraps the public `Simulation::step()` boundary and reads
new ledger entries and reports. No telemetry calls were added to agents, planning,
settlement, or the monthly scheduler. Private forecast simulations are not observed.

## Use

From `exp/economics`, select a fresh output directory:

```sh
MONTHS=3 CASE=directed TELEMETRY_DIR=../../output/economics/telemetry-demo \
  cargo +1.92.0 run --locked --example reciprocal_market
```

This writes `directed.jsonl`. Omitting `TELEMETRY_DIR` leaves the runner's original
execution path intact. Existing files are never overwritten. Outputs belong under
ignored `output/`, never in commits. The example's existing console summaries and
legacy `DETAIL` output remain separate; `DETAIL` is not needed for telemetry.

| Environment variable | Default | Meaning |
| --- | --- | --- |
| `TELEMETRY_MODE` | `both` | `metrics`, `logs`, or `both` |
| `TELEMETRY_AGENTS` | All | Comma-separated agent IDs |
| `TELEMETRY_FIRST_MONTH` | 0 | Inclusive first observed month |
| `TELEMETRY_LAST_MONTH` | Unbounded | Inclusive last observed month |
| `TELEMETRY_EVERY` | 1 | Metric sampling cadence relative to first month |
| `TELEMETRY_LOG_LIMIT` | 100000 | Maximum batch/transaction/error log records per file |

Configuration is also available through `telemetry::Config`. `Observer<W: Write>`
accepts a file, buffered writer, or in-memory destination. Use its `step` or
`run_months` instead of calling the simulation directly while observing. Finish
explicitly to write omission counts and check buffered output errors.

## Records and interpretation

Every line is an independent JSON object with `schema: 1`, `run`, and `kind`.
`start` records configuration; `attached` records backend, starting month/phase,
next batch ID, and agent/resource catalogs. These metadata records and `finish`
are always emitted, irrespective of filters or log limits.

| Kind | Boundary and meaning |
| --- | --- |
| `batch` | A committed ledger batch, with ID, phase and transaction count |
| `transaction` | Batch ID and transaction index, cause, all account effects, and optional resulting process status |
| `step_error` | A failed public step, including the number of batches actually appended |
| `need` | Monthly report's desired, fulfilled and deficit quantities per agent/resource |
| `agent_status` | Monthly report's terminal flag |
| `balance` | Closing account quantities, including institutional accounts present in state |
| `account_flow` | Gross credits/debits from observed committed transaction effects, grouped by month/agent/resource |
| `process_count` | Closing counts of retained active/completed/aborted processes per operator |
| `market` | Town market clearing's per-market volume, posted price and unfilled buy/sell quantities |

Balances and flows use each resource's native simulation units. Do not sum
unrelated resources or interpret gross credits as income: transfers and capacity
regeneration are included. Likewise capacity debits include expiry as well as
productive use. Process counts describe retained state, not new failures during
the month. Missing accounts/process statuses are sparse observations, not an
explicit row of zeros. Market price is `null` when no trade establishes a price;
previous prices are not carried forward. Current market metrics cover the town
market books, not every older exchange subsystem.

Agent filters select individual metrics and transactions involving that agent's
accounts or process operation. A selected transaction retains **all** counterparties
and effects, enabling reconciliation. Batch envelopes remain visible. Market
metrics explicitly retain `scope: whole_market`; filtering to a person does not
relabel market volume as that person's trades.

Log limits do not suppress metrics. `finish.omitted_logs` counts eligible records
excluded by the limit, not those excluded by filters. The limit bounds record
count, not bytes or the simulation's own retained ledger. A missing finish record
means output may be incomplete. Records are diagnostic exports, not a replay
archive: not every agreement, membership, credit or maintenance field is serialized.

## Continuation and failures

Attach one observer per real simulation/continuation segment. Only steps executed
through that observer are exported; preexisting ledger history is not replayed.
Do not reuse it for another simulation or interleave unobserved steps. A resumed
state can start a new output file with its original month and batch IDs. Attaching
mid-month gives full closing balances and reports but only the observed portion of
that month's account flows. A segment ending mid-month has no closing metrics for
that incomplete month. Initial attachment metadata makes that boundary explicit.

A successful simulation step is already committed before export. An output error
cannot undo it. The observer returns an explicit error and refuses further steps;
replace it with a new segment after correcting the output problem. There is no
transactional durability guarantee between the ledger and the external file.
This observer changes neither scheduler timing nor allocation policy.

## Verification

Seven focused observer tests cover CubeCL CPU observed/unobserved state, ledger and
report equivalence; metrics reconciled against receipts; JSON escaping; filters,
cadence and omission limits; mid-month checkpoint continuation and repeated calls;
failed settlement and writer failures; output modes; and exclusion of private
planning runs while retaining counterparties in filtered transaction logs.

Validation: all seven telemetry, nine reciprocal-market, and twelve town-market
tests passed (28 total), along with `cargo +1.92.0 fmt --check` and
`cargo +1.92.0 clippy --locked --all-targets -- -D warnings`. The full crate suite
was not rerun for this observer-only change.

A three-month `directed` reciprocal-market CPU run produced 15 committed batches,
70 transaction logs, six per-market observations and 24 need observations. All
need deficits were zero and no logs were omitted. Grain volume was four in month
one; wood volume was two in each of months two and three. This validates export
against the existing directed diagnostic, not autonomous specialization.

Later subsystem observers can inspect existing receipts for payment/default
outcomes, order rejection details, or forecasts versus realized actions. Add
missing facts to domain receipts only when a concrete diagnostic needs them;
avoid scattering logging calls throughout the economic code.
