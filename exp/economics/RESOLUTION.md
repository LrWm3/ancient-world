# Resource resolution mechanisms

Implemented small experiment, not a universal market or dependency planner.

Ranking and resolution are separate choices. `allocation::RankingPolicy` still
orders applicants using priority, lottery, or stable priority. The new
`resolution::resolve` interface determines what can be accepted from those
applicants against one dated availability snapshot:

| Mechanism | Authorized result |
| --- | --- |
| `Immediate` (default) | As many whole lots as fit, at least the request's minimum. |
| `ConditionalBundle` | All requested lots with every immediate prerequisite, or nothing. |

A request contains a stable claim ID, priority, requested/minimum lots, and a map
of prerequisite accounts to quantity per lot. Account scope can identify shared
wood, an individual's stone, cash, or that individual's available labor. These
are current budgets; forecast output is not stock. Each accepted lot consumes
all of its listed inputs together under either mechanism. The difference is
whether a smaller number of complete lots is acceptable.

The result records the dated context, mechanism, offered and reserved quantities,
resource shortfalls (account, required, available), rejection reasons, and
remaining budgets. A domain callback checks permissions, storage and other
constraints against a cloned candidate plan. Only success retains that plan and
its resource reservations. A failed callback discards both, allowing later
applicants to use the same resources. Callbacks must keep side effects inside
independently cloned plan data; external mutation and shared interior mutation
would violate that contract. Resolving is a preview, not a live state commit.

## Integration and boundaries

`pool_market::Config.mechanism` selects either implementation. Discovery exposes
it in collection supply terms, and the dated round retains the actual resolution.
The existing wood scenarios default to immediate allocation. To opt in:

```rust
world.pool_market.as_mut().unwrap().mechanism =
    economics_compute_smoke::resolution::Mechanism::ConditionalBundle;
```

The wood adapter groups each person's new collection requests into one bundle.
Immediate mode keeps the previous privately feasible quantity ceiling. Bundle
mode retains the full requested quantity: insufficient labor cannot silently
turn a two-lot bundle into a one-lot acceptance. Its shared wood budget goes
through the generic resolver, while the existing process preview jointly checks
labor, permissions, storage and actual work completion. Domain failures are
rejections; resource-account failures report structured shortfalls.

Existing crop attendance and other base work precede this collection window, as
before. They are not rolled into the person's new collection bundle. Open still
regenerates capacity and stock once. Acquire can retain a dated production plan;
Productive executes that plan or resolves current work at its existing boundary.
Settlement reconstructs the expected resolution before accepting the batch.
Outputs become available through the existing commit, not as inputs to another
claim in the same resolution. Released inputs can serve later applicants in this
window, but never retroactively rerun prior work.

## Controlled checks

The multi-resource fixture gives both requests the same opening budgets: three
wood, one stone, and separate labor accounts. Applicant 1 requests two lots, each
using one wood, one stone and one labor. Applicant 2 requests one lot using two
wood, one stone and one labor. Ranking is stable priority in both runs.

- Immediate accepts one lot for applicant 1. Applicant 2 cannot obtain stone.
- Conditional bundle rejects applicant 1 because two stone are required, then
  accepts applicant 2. Applicant 1's labor remains entirely available.
- A separate domain rejection after tentative plan mutation discards that
  mutation and all prerequisites, allowing the next applicant to complete.
- Missing inputs, empty demand, duplicate identities, and reversed request order
  are covered.

The simulation fixture starts two admitted farmers in month 2 with two shared
wood available (one collection lot). The first person requests two collection
lots, the second one. With identical requests and stable ranking:

- Immediate grants one lot to the first person.
- Conditional bundle grants none to the first and one to the second. Both crops
  continue; the first retains two labor after crop attendance, the second one.
  Shared wood ends at zero, and CPU settlement matches the reference state.
- With ample wood but insufficient private labor, the full bundle is rejected
  rather than silently reduced. Later applicants can still use the wood.
- Tampering with the recorded mechanism is rejected without changing state.

A conditional-mode continuation test compares CPU/reference execution with
reversed participant order, then three months in one call versus a checkpoint
clone advanced one month at a time. Existing immediate-mode conservation,
scarcity, replay rejection and continuation tests remain in place.

## Limits and next extensions

This is atomic prerequisite allocation, not a general dependency graph. The
multi-resource interface is generic; the live adapter currently handles the
configured wood collection offer. It does not yet group different process
definitions into an agent's multi-stage plan, purchase inputs for toolmaking,
secure future supply, or expose explicit alternate plans for same-round retry.
Rejected requests leave resources available and provide feedback; automatic
counteroffers and negotiated acceptance are future work.

There are no changed prices, substituted resources, delivery dates, auctions,
ZIP negotiation, cooperation or new legal permissions. Those require richer
terms and explicit agent consent, beyond the authorized partial quantities in
`Immediate`. A future resolution protocol can add those outcomes while retaining
shared resource checks and settlement. Whole bundles can leave usable resources
idle; the controlled checks establish semantics and conservation, not superior
economic balance or globally feasible forecasts.

## Validation record

`cargo +1.92.0 test --locked` passed 187 tests. The final focused run of
`--test resolution --test pool_market` passed 11 tests, including one additional
private-labor regression added after the full run compiled. Clippy with
`--all-targets -- -D warnings`, formatting, diff checks and the repository
artifact policy passed. Raw logs remain under ignored `output/economics/`.
