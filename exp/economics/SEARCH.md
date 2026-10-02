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

`SearchContext` is now an alias for the [shared forecast context](FORECAST-CONTEXT.md).
It owns a read-only snapshot of current state and known catalogs.
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

## Experimental explicit package composition

The later [planner comparison](PLANNER-COMPARISON.md) introduces a separate opt-in
`composition::choose(sim, scope, strategy, budget)` interface. Beam and best-first
share dated common-offer descriptions and a forward package evaluator. Unlike
`CandidatePlan`'s single work-policy proposal, the package can name several
productive lots and member-consented household prerequisites. It therefore does
not silently coerce those packages into the older trait or change its defaults.

`Selection::accept` checks the dated observation snapshot and uses ordinary
settlement. Callers may reconsider at Acquire; they must still collect independent
contested applications before using the allocator. A household mandate names
consenting members and does not imply permission over another household/person's
private work. Common-offer composition requires an isolated person or household
branch. The [market adapter](PLANNER-CALIBRATION.md) additionally supports one
planning person with passive town counterparties: it selects eligible need-order
subsets alongside productive offers, and scores only that person. Active
counterparty production remains excluded from direct `choose` calls; the `Persons`
coordinator below supplies private forecast branches for independently active
traders. Household market search and finance remain excluded.

`choose_with_scoring` exposes private-buffer and household-consumption buffer
variants for calibration. Fixed forecasts reproduce fixed execution, but monthly
replanning can shift harvests and shortages. These variants are experiments; no
new score or continuation policy becomes the default.

`composition::continuation::Controller` is a separate opt-in review driver for
isolated crop/household scopes, requiring explicit `ContinuingFirst` configuration.
It compares monthly composition search, retain-and-repair on observed deviation,
and scheduled search at the forecast horizon. Between searches the same cheap
policy used by the forecast prepares fresh dated work through ordinary settlement.
Retaining a forecast does not retain its grants or make its future resources real.
Checkpoint the controller alongside the simulation; cloning both is exercised,
not a durable serialization format. See [continuation results](PLANNER-CONTINUATION.md)
for costs, shocks, economic regressions and exclusions.

`composition::continuation::persons::Persons` coordinates one such controller per
person. All search from the same opening in separate branches; none directs another
person's production. It retains equally scored packages actually explored by each
search so that interchangeable plots need not create avoidable rejection. An
explicit allocation policy orders whole-package admission through common offer
preparation. Existing work and previously admitted packages constrain each next
admission; fresh dated work commits once. Rejected actors can request cheap fallback
work and reconsider at their next Acquire, including under scheduled review.
This pilot limits each reviewed package to one new land agreement. Its comparison
view ignores unrelated private changes and global IDs while retaining own resources,
commitments and observed shared availability; it is not a private-information model.
Checkpoint `Persons` with the simulation. See [multi-person results](PLANNER-PERSONS.md)
for two/four-person repeated harvests, scarce access, CPU/accounting checks and
exclusions. Household and finance composition remain unsupported here.

The [active exchange extension](PLANNER-EXCHANGE.md) supports a plain town book
with monthly review. Every person selects its own productive package and a mask
of listing/side order submissions. The coordinator clears all masks together,
then checks work against actual fills. Existing admission and settlement enforce
opening cash, stocks, storage and dated capacity. Work rejection can leave a valid
spot trade in place; these are separate intents, not conditional trade bundles.
Market packages do not use the request-only equal-score alternatives above, which
do not preserve order choices.

`Persons::order_forecast` selects `CurrentBoundaryOnly` (later hypothetical orders
use ordinary generation) or `StandingPolicy` (the actor retains its selected sides
through the forecast, including sides eligible only later). `StandingPolicy` is
the default for this new opt-in market coordinator; direct `choose` retains its
existing behavior. Neither option retains live orders between months. Peer forecasts
use current stocks, ordinary consumption/orders and no new peer production; live
peers decide independently. Receipts expose expected versus actual fills. No peer
forecast is a promise or an allocation. Retain/repair and scheduled market review
are rejected before advancing.

`Persons::counterparty_policy` independently selects ordinary peer submission or
`RecentSubmission { memory_months }`. The latter freezes each actor's view of earlier
settled eligible submission/withholding receipts, with expiry during the forecast.
Submitted-but-unfilled orders remain evidence of willingness. Ineligible orders do
not erase a still-fresh eligible observation. Only the actor's own resulting mask
enters live clearing; hypothesized peer masks never authorize peer actions.
`Exchange::expectations` exposes the evidence. The [expectation comparison](PLANNER-EXPECTATIONS.md)
records fewer optimistic misses in some cases but worse coordination under longer
memory. Ordinary expectations remain the default; peer production and reliable
mutually compatible plans remain gaps.

`composition::continuation::posted::Controller` optionally wraps `Persons` for a
two-person reciprocal spot offer. After public preliminary order masks, each actor
searches its own outside option and offered package. Two consents require exact
current deliveries and feasible combined work before publication; otherwise the
original spot decision remains the fallback. It reuses existing market transfers
and `cooperation::Delivery` terms. Future submission is only an explicit renewal
hypothesis, not a dated promise. See [posted-offer results](PLANNER-POSTED.md) for
extra search costs and failed high-cash controls.

`composition::continuation::scheduled::Controller` now assesses one public six-month
delivery-only schedule using independent own-work searches. Forecasts condition on
peer performance while retaining the actor's actual budgets. The shared cooperative
executor owns the accepted book, checks opening resources and cancels future terms
on a failed delivery. Terms persist independently of controller work choices.
[The dated comparison](PLANNER-SCHEDULED.md) sustains both persons at all tested cash
endowments for 24 months, but adds searches, assumes peer performance and uses a
single supplied terms menu. Low-cash best-first spot has fewer shortfalls. General
schedule optimization, residual spot clearing and risk estimation remain open.

Expansion and full-forecast budgets are distinct. Metrics include pruning,
exhaustion, rejection reasons and forecast-months. Failed economic outcomes remain
visible. This experiment does not supersede the historical verification above;
its successes, regressions and remaining adapters have their own report.
