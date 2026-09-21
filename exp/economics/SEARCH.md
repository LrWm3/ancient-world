# Swappable opportunity search

Status: implemented. The current strategy is named
`NeedDirectedOpportunitySearch`: it starts from needs and proposes opportunity
bundles through resource, land and membership prerequisites. The default preserves
the previous bounded search, forecast scoring and committed behavior.

## Boundaries

The interfaces are in `src/search.rs`:

```rust
pub trait OpportunitySearch {
    fn name(&self) -> &'static str;
    fn search(
        &self,
        context: &SearchContext,
        budget: SearchBudget,
    ) -> Result<SearchResult, String>;
}
```

`SearchContext` owns a read-only snapshot of current state and known catalogs.
Future fixture capacity overrides and future scripted starts are removed before
any strategy can see it. This is the existing observation model, not a new
partial-knowledge or beliefs system. A strategy has no live simulation handle.

`SearchResult` contains ordered `CandidatePlan` records and an explicit
`budget_exhausted` flag. Each candidate contains:

- Ordered typed steps: accept membership, accept land, buy equipment, sell stock.
- A monthly work proposal: allocation priority, whether to defer new productive
  starts, and an optional preferred productive process.
- An explanation of the proposed steps and work choice.

For example, citizenship and farming are represented as membership acceptance,
then land acceptance, followed by a work proposal that the ordinary resolver
expands into dated crop and firewood reservations. Plans contain no raw account
deltas, invented permissions or pre-granted resources.

`planning::evaluate_candidates` expands these proposals and forecasts their
consequences through existing settlement. It owns feasibility, annual-payment
checks and the unchanged lexicographic score: survival, impairment, deprivation,
ending need coverage, then productive work. Invalid proposals are rejected;
prerequisite order matters. The best surviving candidate becomes the same dated
batch used before this refactor. Stable candidate order still breaks score ties.

Settlement independently validates acceptance, permissions, resources and execution.
Search cannot bypass those checks. Membership and land settlement remain atomic.
A search error, over-budget response or set with no feasible candidates leaves the
proposed batch and live state unchanged and returns an error to its caller.

Decision traces now include the search name, configured budget, exhaustion flag,
number generated/rejected, candidate plans and forecast scores. Forecast records
contain the common plan rather than separate optional acquisition fields.

## Configuration and replacement

For the current person scenario:

```rust
use economics_compute_smoke::search::{SearchBudget, SearchConfig, SearchStrategy};

world.agent_search.insert(person_id, SearchConfig {
    strategy: SearchStrategy::NeedDirectedOpportunitySearch,
    budget: SearchBudget { max_candidates: 4096 },
});
```

Omitting an agent entry selects that same default. Configuration lives in `World`
and survives in-memory checkpoints. The strategy is used by the consequence-aware
planner; static allocation policies and specialized market resolvers retain their
existing execution paths.

`ExistingCommitmentsOnly` is a second, deliberately restrictive control. It offers
no new acquisitions or productive starts this month, while ongoing work and
ordinary consumption still go through the shared resolver. It is useful for
checking the strategy boundary, not a recommended survival policy.

To try an external Rust implementation without modifying the simulation loop,
implement `OpportunitySearch` and pass it to `planning::choose_with_search` at a
decision boundary. That entry point returns a proposed batch through the same
evaluator; commit it through normal settlement. Add a named `SearchStrategy`
variant and dispatch arm when the implementation should be selected in persistent
agent configuration.

The older one-to-four-person consequence planner evaluates a joint work portfolio.
Its active participants currently must select the same strategy and budget.
Conflicting per-agent configurations return an explicit error; this refactor does
not reinterpret the joint planner as independent simultaneous planners. Supporting
that would require an explicit proposal composition/allocation rule.

## Bounds and limitations

The default strategy keeps the existing limits: at most four forecast participants,
four requirements each, four posted acquisition offers and four substitute
producers. It retains the existing bounded membership-plus-land chain and work
policy portfolio. It does not search arbitrary process graphs or solve an optimal
schedule. Candidate work proposals are expanded by the existing need-directed
monthly resolver, rather than enumerating every possible labor assignment.

The configured candidate limit is 1..=4096. It bounds returned candidates and thus
expensive forecasts; it is not a wall-clock limit or a graph-edge budget.
Truncation preserves a deterministic prefix and may omit a beneficial membership
or land chain. An exactly complete result at the limit is not marked exhausted.
Direct strategy calls can use zero to obtain an empty, exhausted result; normal
agent configuration requires a positive limit. The evaluator checks that a custom
strategy did not exceed its supplied limit.

After the candidate's first month, forecast continuation uses the same static
rollout policy as before. The evaluator does not recursively invoke the new search
in every predicted month. Forecast limitations and horizon effects therefore
remain unchanged.

## Verification

Four 18-month comparisons against snapshots captured immediately before refactoring
produced identical state, monthly reports and committed batches (excluding the
intentionally changed decision diagnostics): citizenship farming, beneficial tool
purchase, long-horizon land-offer choice, and storage/currency exchange.

New tests cover deterministic budget truncation, exact-limit reporting, sanitized
observations, unchanged live state, per-agent strategy selection, continuing an
existing crop, injecting a custom strategy into the same evaluator, invalid
prerequisite order, over-budget responses, checkpoint continuation, and explicit
rejection of conflicting configurations in legacy joint planning.

Run from `exp/economics`:

```sh
cargo +1.92.0 test --locked --test search
cargo +1.92.0 run --locked -- opportunity-farming
```

Temporary comparison snapshots and logs remain in ignored `output/economics/`.

Final validation: all 153 crate tests passed, along with formatting, Clippy with
warnings denied, and the repository artifact check. The 36-month CubeCL CPU
scenario still completed five harvests, paid both annual taxes and recorded zero
food or warmth deficits.
