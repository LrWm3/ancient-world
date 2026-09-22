# Shared forecast observation context

Implemented in `src/forecast.rs`. Need-directed search, its planning rollouts,
remaining-value work choices and collateral resale valuation now construct their
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
| Accepted loans, land agreements, obligations and dated reserved work | Preserve, including known future due dates |
| Catalogs, rates, process recipes, rights and configured application | Preserve as known scenario terms/plans |
| Capacity override fixtures | Remove; observed capacity is already in state after Open |
| Scripted process starts | Retain this month's starts only |
| Discretionary credit transfer fixtures | Retain current/past months; hide later transfers |
| Configured resale buyer | Exclude from rollouts; speculative bids must not recursively become forecast income |

Current-month fixture actions are treated as known scheduled inputs, including
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

## Scope and verification

This extraction covers search/planning, work choice and resale. The older forward
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

Validation: **27 tests passed** across those five suites; all-target Clippy passed
with warnings denied. Run from this directory:

```sh
cargo +1.92.0 test --locked --test forecast_context --test search --test planning --test work_choice --test resale
```
