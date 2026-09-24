# External metrics and committed-event logs

Implemented: base metrics/logs plus optional production-market planning and
settlement observers. Broader subsystem coverage remains incremental. The observer wraps the public `Simulation::step()` boundary and reads
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

## Planning and settlement observers

Enable these independently of `TELEMETRY_MODE`:

```sh
MONTHS=6 CASE=both TELEMETRY_AGENTS=88 TELEMETRY_PLANNING=selected \
  TELEMETRY_SETTLEMENT=true TELEMETRY_DIR=../../output/economics/observer-demo \
  cargo +1.92.0 run --locked --example reciprocal_market
```

`TELEMETRY_PLANNING` accepts `off` (default), `selected`, or `alternatives`.
`TELEMETRY_SETTLEMENT` accepts `false` (default) or `true`. Both use the existing
inclusive month/agent filters and shared log limit; metrics cadence does not sample
these records. They can run with base logs disabled. Settlement also enables
`step_error` records. Full transaction effects remain in the base logs.

The implementation lives in `src/telemetry/observers.rs`, outside economic code.
It reads real committed records and never reruns planning or matching.

| Kind | Content |
| --- | --- |
| `plan` | Production-market selected candidate, candidate index/count, dated price/demand beliefs, needs/priorities, ranking rule, and forecast |
| `plan_alternative` | Optional candidate choice and forecast, keyed by plan batch, agent and candidate index |
| `market_order` | Submitted side/quote and protected balances, keyed by batch and order index |
| `market_attempt` | Buyer/seller, goods quantity, quotes, recorded outcome, actual completed quantity and nullable price |
| `work_receipt` | Agent, need/process definition, recorded reason, requested/allocated/completed work |
| `plan_outcome` | Original selected forecast versus actual sales, purchases and need deficits over the **same full horizon** |

Batch IDs link plans, orders, attempts, work receipts and base transaction records
at a committed boundary. Order/attempt indices are local to their record kind;
they are not a fabricated one-to-one transaction attribution. Work in a later
phase has its own batch; join by month/agent/definition where available. Reasons
are the domain's recorded enum values, not generated explanations. Protected
balances are policy guards, not escrow. Large integer buffer gaps and protected
quantities are decimal strings to preserve `i128` values.

The ranking is lexicographic: avoid terminal outcomes, minimize need deficits in
priority order, minimize process failures and buffer gaps, maximize closing coins
plus bounded stock value, minimize capacity debits, then break ties by candidate
index. Alternatives expose the inputs to that ranking. They do not claim that a
selected candidate is globally optimal. Fixed diagnostic policies have no
search transcript and therefore emit no `plan` records.

Forecast sales/purchases are quantities per market over the entire forecast
window, not current-month promises. Actuals accumulate only completed matches;
failed attempts contribute zero. An outcome appears at Close of the forecast's
end month. Empty market maps mean zero completed quantity. The real agents can
replan every month, whereas the original forecast holds a candidate fixed; a
difference alone does not prove a settlement bug or identify a causal explanation.

An observer tracks only plans it encounters while attached. It does not recreate
old plans on restart or infer a full-horizon result from partial observations.
`finish.pending_plan_outcomes` reports outstanding comparisons when a run ends
before their horizons. If the ending month is outside the configured export
window, its comparison is filtered out. Continue stepping through the observer
even during filtered months to preserve the actual path for tracked plans.

Current coverage is deliberately explicit: production-market search, town-market
order-generation gates, orders/match attempts, and common batch work receipts.
Legacy search variants, credit/default waterfalls and agreements are not yet
fully exported. An absent order does not acquire an invented rejection reason.
Specialized observers can extend these records as concrete diagnostics require.

Observer-extension verification: ten telemetry tests passed, including full-horizon
CPU observation equivalence, optional alternatives, filtered counterparties,
shared log limits, and exact funding/storage rejection outcomes. Formatting and
strict all-target Clippy checks passed. The full economic suite was not rerun.

A six-month autonomous two-market CPU run filtered to agent 88 exported six plans,
ten submitted orders, two match attempts, 22 work receipts and one completed
forecast comparison; five later plans still awaited their horizons. No logs were
omitted. The initial forecast predicted no trade and one unit of warmth deficit;
the realized six-month path bought two grain units, sold two grain units and had
zero food/warmth deficits. This demonstrates dated forecast/outcome inspection,
not evidence that autonomous reciprocal wood trading has been solved.

The [twelve-month observer review](OBSERVER-REVIEW.md) applies these observers to
the missing autonomous wood trades, with a directed control and remaining gaps.

## Order-generation receipts

Settlement observation now includes `order_generation`: one receipt per configured
agent/market/side at Acquire. Adaptive books inspect both sides; fixed-side books
record only the configured side. Reasons are `Submitted`, `NotAdmitted`,
`Inactive`, `Ineligible`, `PurchasePolicy`, `OtherSideSelected`,
`NoNeedImprovement`, `InsufficientOpeningStock`, or `ProtectedStock`.

These are first decisive gates, not exhaustive hypothetical failures. The existing
buy-first rule still selects at most one side. When a buy is submitted, its sell
receipt is `OtherSideSelected`, with no pretend reserve evaluation. Likewise,
policy/eligibility exclusions carry null evaluation fields. Evaluated receipts
contain goods resource, lot size, opening availability, protected quantity and,
for buys, need deficits before/after one hypothetical purchase. Protected quantities
include needs and known commitments; no goods are reserved by recording a receipt.
Insufficient opening stock takes precedence when there is less than one lot;
otherwise a rejected sale is classified as protected stock.

Inputs are the same opening balances and dated admission used by order generation,
before either book settles. Receipts do not add a second evaluation or shift work
between phases. Funding/storage constraints still belong to matching; a submitted
buy can subsequently fail either check. Receipts are stored in the domain round,
recomputed during atomic settlement validation, and exported only after commit.
Stable side/agent ordering preserves catalog-reordering equivalence. Detailed
exclusions now appear in the [review follow-up](OBSERVER-REVIEW.md#order-generation-follow-up).

Order-generation receipts also identify their effective buy/reserve windows. The
[shared order-horizon comparison](ORDER-HORIZONS.md) tests 6/2, 2/2 and 6/6 months
without changing the six-month production planner.

[Planning variants](CALIBRATION.md) add selection reason, hold-through date and
dated assumed counterparty choices to `plan` records. A retained candidate may
not be the current wealth-maximizing alternative: the explicit persistence policy
allows safety improvements to override a hold, but not wealth-only improvements.

Credit boundaries also export `loan_state`, `loan_event` (accrual, payment, arrears,
fixed-value enforcement), `credit_stock_sale` limits and `collateral_process_transfer`
when settlement observation is enabled. These distinguish cash repayment from debt
cleared through repossession even when both end with status `Repaid`. They are
filtered by either counterparty and read committed receipts; broader credit/resale
coverage remains incremental. See [credit stress controls](CREDIT-STRESS.md).

The [recovery extension](CONTRACT-RECOVERY.md) adds settlement records
`guarantee_payment` (requested/paid amounts and recourse identity) and
`estate_recovery` (opening/rejection, funded/rejected sales, distributions,
write-offs and closure). Land/forward admission adds `Admitted`,
`LandDistributed` and `ClosureDeferred` details, preserving original claim units,
actual tender quantities and the non-loan creditors in observer filtering. Distribution details preserve requested, allocated and
paid quantities, including zero grants. These read committed receipts under the
existing filters and log budget; they do not run recovery or infer a payment from
a balance change. The recovery observer control verifies unchanged simulation
state/ledger and reconciles distributions and losses.

The opt-in cooperative planner emits `cooperation` records with agreement identity,
public offers, dated terms, selected work preferences, acceptance scores, logical
forecast counts, completed deliveries and failures. They are enabled by settlement
or planning observation. Agreement trade volume appears in ordinary market metrics;
there are no synthetic spot-order or negotiation-attempt records for these deliveries.
Posted offer receipts also include acceptance/rejection and proposer/recipient
assessments for each revision. `joint_projections` separates centralized forecasts
from individual forecasts; it remains zero for Posted discovery.
See [cooperative discovery](COOPERATION.md) for the information and failure policies.

## Physical minting observer

With settlement observation enabled, the [minting pilot](MINTING.md) emits
`physical_minting_market` records for accepted/rejected packages, with dated
counterparties, market IDs, prices and rejection reasons. `physical_coin_issuance`
records identify the issuer, completed process, coin resource and quantity.
Generic transaction logs retain all material, money and capacity legs. Agent
filters select packages involving a selected buyer/seller and issuance by a
selected issuer. Observation does not participate in reservations or settlement.

The generated mint-order variant also emits `physical_minting_orders` for a selected
issuer: required funding, orders (agent, market, side, limit and lots), and the
plan outcome. Package records now resolve trades from the committed boundary,
so generated counterparties are visible even with no scripted deals. Failure
reasons identify unfunded targets or unfilled input markets; they do not enumerate
every suppressed quote or attempted counterparty.

For repeated mint targets, `physical_minting_orders.target_month` identifies the
next configured attempt, including a target that cannot be funded or matched. It
is null after the final date. Missed dates do not accumulate a retry backlog.

With the [food provision policy](MINT-PROVISION.md), mint-order records also include
`provision`: per-participant required and held food, expected future access, cash
gap and `Covered`, `SeekIncome`, `NoFoodAccess` or `AwaitFood` choice. These precede
input settlement; actual leisure and earning work remain ordinary process records.
Unused labor alone is not recorded as leisure.

Provision records include `goal` (`FullBuffer` or `Incremental`) and
`purchase_target` in additional food units. `food_required` remains the complete
horizon target, while `cash_gap` belongs to the selected purchase target.
`AwaitOpportunity` means a reachable partial target is funded but the full buffer
is not covered; it must not be aggregated with completed leisure.

Forward relief emits `DeliveryRelief` with accepted term ID, forward, creditor,
application/rejection result, effective due date, newly written-off quantity and
remaining claim. These are legal claim changes, not physical delivery metrics;
creditor filtering includes them. See [explicit delivery relief](DELIVERY-RELIEF.md).
