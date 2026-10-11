# Shared forecast observation context

Implemented in `src/forecast.rs`. Need-directed search, its planning rollouts,
remaining-value work choices, collateral resale valuation, borrowing, sale-only,
joint production/sale planning and organizational agency construct their
observations through `ForecastContext`. `search::SearchContext` remains a public
alias, so existing search strategies keep their interface.

## What the snapshot contains

The context owns cloned world/state tables. Accessors are immutable;
`into_parts()` yields an isolated branch for explicit hypothetical changes.
`boundary()` records the observed month and next phase to execute. Construction
neither advances time nor accrues, pays, reserves or publishes anything.

| Information | Treatment |
| --- | --- |
| Current balances, capacity, conditions, ownership and process progress | Preserve committed state |
| Historical diagnostic payloads | Strip alternative projections and forward-review details from private copies; retain live diagnostics, decision rows, accepted commands, catalogs, scalar scores, ballots, authority and pending agreements |
| Accepted loans, land agreements, obligations and dated reserved work | Preserve, including known future due dates |
| Catalogs, rates, process recipes, rights and configured application | Preserve as known scenario terms/plans |
| Capacity override fixtures | Preserve only the current override after Open; hide unopened/future overrides |
| Process starts | Preserve accepted organizational starts and published mint-policy dates; other fixture starts survive only for the current month |
| Discretionary credit transfer fixtures | Retain current/past months; hide later transfers |
| Configured resale buyer | Exclude from rollouts; speculative bids must not recursively become forecast income |

Current-month process-start fixtures are treated as known scheduled inputs, including
when the snapshot is taken before Open. This preserves the pilot's existing
observation rules; it is not an agent-specific visibility or announcement system.
Past transfers do not replay because execution remains dated. Unaccepted offers
remain possibilities rather than contractual future receipts. Existing configured
credit applications remain known configured plans. The [borrowing comparison](BORROWING-DECISIONS.md)
explicitly forks accepting/declining that application; it does not search arbitrary loans.

## What stays separate

The context does not choose a horizon, candidate, score or execution policy.
Callers keep their existing limits and checked horizon arithmetic:

- Search generates need-directed candidates; planning scores their consequences.
- Work choice replaces recursive search with the selected bounded work plan and
  projects currently observed capacity using its existing rule.
- Resale forks the same observed context for with/without-asset comparisons. It
  explicitly moves the hypothetical evaluation to Productive, and only the
  with-asset branch changes title and attached crop control. These are valuation
  hypotheses, not live purchases or reservations.

Need consequences, subjective stock value and denomination-bearing bids retain
separate meanings. Domain hypotheses remain domain code; the shared context does
not introduce a universal action language. Normal simulation settlement still
validates and publishes actual decisions.

## Shared need accounting

Borrowing, sale-only and joint planning use `forecast::needs` to validate limits,
accumulate deficits and compare priority-ordered provisions. Their candidate sets,
horizons, objectives and fallback rules remain separate. See the
[planning contracts](INTEGRATION-STATUS.md#standardized-planning-contracts).

## Scope and verification

This shared boundary covers search/planning, work choice, resale, borrowing,
sale-only, joint production/sale planning and organizational agency. The older forward
production projection has its own specialized construction and is not migrated
in this increment. Full catalogs/state are visible: private information,
uncertainty distributions and alternative observation policies are not implemented.
New scenario fixtures must be classified explicitly before exposing them to these
forecasters; cloning a newly added field does not establish that agents know it.

Two focused tests check unchanged live state, isolated hypothetical edits,
idempotent sanitization, search compatibility, preservation of known future loan
payments and land commitments, exclusion of future discretionary funding from
work/resale decisions, and response to current buyer cash. Existing search,
planning, work-choice and resale suites check capacity-shock invisibility,
scenario outcomes, CPU/reference results, batching and checkpoint continuation.

Original context validation: **27 tests passed** across those five suites; all-target Clippy passed
with warnings denied. Run from this directory:

```sh
cargo +1.92.0 test --locked --test forecast_context --test search --test planning --test work_choice --test resale
```

## Organizational peer hypotheses

Organizational forecasts freeze autonomous controllers and preserve accepted work
and governance history through this context. Domain-specific hypotheses live in
`agency::expectations`, not in snapshot sanitization. `AcceptedOnly` is the default;
opt-in variants repeat bounded observations of completed one-month peer work and,
separately, individual state ballots through ordinary future admission. Only future
Open boundaries receive hypotheses, and imagined outcomes never refresh evidence.
They do not authorize live work, issue live cash or promise future counterparties.

The [peer-expectation acceptance report](ACCEPTANCE-PEER-EXPECTATIONS.md) records
the comparison and current verification. Observation still uses full information,
including accepted ballots and incumbent preferences. It does not infer secret
ballots or simulate a peer's future political/economic optimization.

Organization-decision observations now include signed sale opportunities and
accepted sale receipts from each private projection: date, market, buyer, resource,
quantity, payment resource and value. Receipts come from the existing mint/peer
market settlement batches; they do not cover every financial arrangement. They
explain a hypothetical forecast rather than promising demand or recording real
revenue. Observer-on/off comparisons preserve live state, history and accounting;
see [sale-linked discovery](ACCEPTANCE-SALE-DISCOVERY.md).

The [independent liquidity comparison](LIQUIDITY-PREFERENCES.md) verifies the private
diagnostic trim against rehydrated histories, including pending forward activation,
actual delivery, accepted non-mint starts and nonempty peer observations. The first
World clone still copies the payloads before trimming; subsequent private phase
copies avoid them. Live history retention remains unbounded with run length.

## Process completion diagnostics

Ordinary organization projections retain dated completions for their own operator.
Bilateral forward reviews retain each party's baseline and proposed completions.
`agency::completions::observe` reads only committed transactions in the requested
inclusive month interval, including employment adapter transactions. Nested
production plans, aborted/active work and repeated completed-state changes do not
count. The operator may differ from the output beneficiary. All process types are
included; use definition IDs to distinguish productive and consumption work.

Historical completed state does not create new forecast completions, and forecasts
carry no actual entitlements. These rows help distinguish forgone production from
failed processes; they do not change loss vectors. Existing private-history trimming
removes old completion diagnostics along with their enclosing projections/reviews.
See [input-buffer comparisons](INPUT-BUFFER-PREFERENCES.md) for a shared-opening
example where score priorities change but projected process flows remain identical.
