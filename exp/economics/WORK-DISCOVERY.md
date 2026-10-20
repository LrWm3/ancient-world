# Work discovery: requests, requirements and productive offers

Status: six implementation/review passes completed in a bounded, opt-in pilot.
The default discovery policy remains unchanged. Requests explain potential work;
accepted contracts and ordinary execution still determine actual outcomes.

## Why this direction

The preceding [integration batch](REVIEW-BATCH-24.md) made production and signed
output offers work together, but work candidates still came from static sale
options. This pilot asks whether organizations can use visible buying interest
and dated procurement to discover more relevant work.

The research suggests three useful separations:

- [Smith's Contract Net protocol](https://cse-robotics.engr.tamu.edu/dshell/cs631/papers/smith80contract.pdf)
  separates task announcement, bidding and award. Here a request is only observed
  interest; this pilot does not implement Contract Net negotiation or awards.
- [Walsh and Wellman's decentralized supply-chain formation](https://arxiv.org/abs/1107.0021)
  treats complementary production inputs and resource contention together. Our
  interpretation is to expose missing prerequisites before treating output as a
  viable offer, while retaining independent counterparty decisions.
- [Coles et al.'s LP–RPG resource-flow work](https://arxiv.org/abs/1402.0564)
  addresses optimistic planning that spends resources repeatedly. Here explanations
  allocate observed input stock once within each route, while the existing forecast
  and settlement remain responsible for feasibility. No LP solver was added.

## Six passes and independent review

| Pass | Change | Verification and review outcome |
| --- | --- | --- |
| 1 | Observe bounded, dated requests from signed stock-buy policies and the issuer's next pending procurement. | Unfunded interest stays visible without creating coins, stock, orders or commitments. Withdrawal, signed bounds and access affect fresh observations. |
| 2 | Explain recipes, first-batch input shortages, monthly services, timing, authority, storage and price compatibility. | Reviewer prompted explicit first-batch timing, raw stock targets versus protected stock, and alignment with the existing input-claim horizon. Multi-stage/duplicate-input controls preserve these distinctions. |
| 3 | Add observation-only and request-filtered discovery, with frozen evidence per decision. | Six-month observation-only control preserves choices, state and ledger. History regenerates each catalog from its own snapshot; later quote withdrawal cannot rewrite it. |
| 4 | Compare identical 12-month startup fixtures across four opening-stock conditions. | Request filtering alone regresses Both and OreOnly to zero mints. Early decisions and independent review identify missing one-shot work-plus-quote choices. Retain this failed variant for comparison. |
| 5 | Add a separate policy composing authorized one-shot work with signed quote portfolios. | Both and OreOnly recover to 10 mints, versus 9 under Static. Tests preserve static forward proposal IDs and signed process authority. Neither remains unsuccessful. |
| 6 | Verify continuation, replay, ordering and request lifecycle controls; document the result. | 24-month Reference/CubeCL CPU comparison reconstructs at every phase, including next-Open activation before Acquire. Expired dates roll forward; terminal/recovering participants cannot publish current interest. |

An independent review agent reviewed each stage and the final evidence. This is
six bounded passes, not six claims that arbitrary startup now works.

## Policy and timing contract

`agency::Config.work_requests` selects:

| Policy | Request evidence | Candidate generation |
| --- | --- | --- |
| `Static` (default) | None | Existing static sale options and own objectives. |
| `Observe` | Frozen requests and route diagnostics | Exactly the static catalog. |
| `Respond` | Same evidence | Sale-derived work must match visible interest, signed prices and the first-output deadline; own-goal work remains. |
| `RespondWithWorkQuotes` | Same evidence | Respond plus one-shot `StartProcess` combined with signed quote portfolios. |

At Open, after due policies activate, controllers read the same opening snapshot.
Requests describe current signed buy targets and the issuer's **first** still-pending
procurement date. They do not sum every future requirement against the same stock.
Standing stock targets have no invented delivery date. Future service requirements
are not covered by last month's unused hours. Marketplace eligibility, legal
permission and recovery restrictions apply to observers and requesters.

Fresh observations use the current Open; disabled controllers and those awaiting
startup-forward activation are skipped by the existing controller loop. A request
is not a funded bid, reservation, accepted
forward or promise of purchase. It remains visible when the buyer lacks coins;
funding is deliberately unverified. The pilot exposes observed stock gaps from
these public policies; it does not model private inventory disclosure or beliefs.

Candidate programs retain ordinary independent forecasts, governance, law and
admission checks. Accepted commands take effect next Open. Existing allocation,
Acquire, productive execution and accounting decide what actually happens. A
request disappearing does not automatically cancel already accepted work.

The new variant combines existing work with complete signed portfolios. Such a
portfolio can replace buy quotes or clear quotes as well as post asks. Nonempty
bundles precede work alone among equally improving candidates, as they already do
for recurring work. Consequently this experiment changes available combinations
and their tie order; it is not a pure speed optimization. Startup-forward discovery
continues using the old static catalog so its program identities remain stable.

Requests are capped at 64; route explanations and program catalogs at 16. Exceeding
a bound returns an error rather than silently truncating opportunities. These are
small exhaustive menus, not scalable search or centralized feasibility planning.

## What the explanation establishes

`Decision.work_requests` freezes dated observed interest. `work_routes` shows:

- the signed recipe and stock target, illustrative batch count and first output date;
- first-batch entry inputs and stage-specific monthly service requirements;
- current uncommitted input coverage, with duplicate inputs drawing from one pool;
- price crossing, process permission, occupied governance office and market eligibility;
- required site/equipment kinds, current storage room and protected output stock.

It does **not** certify a feasible schedule. Batch count uses the same raw-stock
target semantics as existing `WorkTargets`; protected output is separately shown.
First-output timing is not a promise that all requested batches finish then. Site
and equipment kinds do not prove access to a particular asset. Current storage
room is not a forecast of room after consumption. Future labor is unreserved;
stock coverage is local to a route, not reserved across alternatives or agents.

Historical validation checks frozen identity, structure, signed standing terms and
recipe quantities, then reproduces the candidate catalog. It does not authenticate
old scheduled-publication dates/prices against an independent immutable publication
ledger. Coordinated tampering with historical evidence is outside this trusted
in-memory model; current mutable procurement terms cannot validate past intent.

## Matched 12-month results

All rows use `Case::Baseline`, production-first workshop preferences and the existing
productive-ask publication policy. Only discovery policy varies within each opening
stock fixture. Prices, physical inputs, legal permissions and financing are unchanged.
Counts come from completed work and audited settlement, not projected production.
Food deficit sums reported unmet nutrition across the run; wage arrears are unpaid
earned claims remaining at the ending boundary.

| Opening workshop stock | Policy | Mints | Workshop batches | Town batches | Food deficit | Wage arrears |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Both | Static | 9 | 8 | 7 | 0 | 0 |
| Both | Respond | 0 | 2 | 0 | 5 | 2 |
| Both | RespondWithWorkQuotes | 10 | 11 | 10 | 0 | 0 |
| Metal only | Static | 8 | 7 | 8 | 0 | 0 |
| Metal only | Respond | 8 | 7 | 8 | 0 | 0 |
| Metal only | RespondWithWorkQuotes | 8 | 7 | 8 | 0 | 0 |
| Ore only | Static | 9 | 10 | 10 | 0 | 0 |
| Ore only | Respond | 0 | 2 | 0 | 5 | 2 |
| Ore only | RespondWithWorkQuotes | 10 | 11 | 10 | 0 | 0 |
| Neither | Static | 0 | 1 | 3 | 5 | 1 |
| Neither | Respond | 0 | 1 | 3 | 5 | 1 |
| Neither | RespondWithWorkQuotes | 0 | 2 | 4 | 5 | 1 |

In the ore-only Respond run, month-one and month-two workshop decisions select a
one-shot refinery start without a metal ask. Its month-three quote-only decision
arrives after issuer procurement has disappeared. The first observed procurement
is too early for newly started output; filtering correctly declines that deadline,
but the old catalog could not pair the remaining own-goal start with selling.
The new variant supplies that combination without declaring the deadline feasible.

Over the two organizations' 12-month histories, total candidate counts are
168/96/168 for Both and OreOnly under Static/Respond/bundled, and 154/104/154 for
MetalOnly. Neither has 154/90/154. The successful refinement does not reduce catalog
size here. Request and route totals count repeated observations, not unique demand,
fulfilled orders or additional production.

This supports a specific composition repair. It does not establish general welfare
improvement, profitable arbitrary supply chains or autonomous startup without
physical/financial prerequisites. The new variant has 24-month continuation
coverage, not the separate 48-month acceptance-readiness certification. The
24-month ore-only example completes 22 mints, 23 workshop batches and 22 town
batches, with zero cumulative food deficit and no outstanding wage arrears;
workshop net income is 20 in the fixture's accounting units.

## Reproduce and inspect

Run from `exp/economics` with Rust 1.92.0:

```sh
cargo +1.92.0 test --locked --release -j8 --test work_discovery_comparison -- --nocapture
cargo +1.92.0 test --locked --release -j8 --test work_discovery_continuation
cargo +1.92.0 test --locked --release -j8 --test work_requests --test work_request_batches --test request_discovery
cargo +1.92.0 run --locked --release -j8 --example work_discovery -- ore bundled
```

The example accepts `both|metal|ore|neither` and
`static|observe|respond|bundled|upstream|input-bids`,
runs 24 months and prints accepted programs, request snapshots and route diagnostics.
Redirect raw output under ignored `../../output/economics/` when retaining runs.

The continuation tests compare state, world including decisions, ledger, reports
and double-entry audit between uninterrupted Reference and CubeCL CPU reconstructed
at every phase. Ledger replay reproduces state and audit; reversed independent
catalog order preserves state, ledger and audit. Controls include ore-only success
and neither-stock failure. CUDA remains unverified.

Final targeted verification passed 75 tests: 34 library, 3 organization discovery,
5 organization forwards, 1 publication execution, 12 startup publication,
7 work requests, 3 first-batch explanations, 4 request discovery, 4 matched
comparisons and 2 continuation tests. Formatting, strict all-target Clippy and the
repository artifact gate also passed. This is targeted regression coverage, not a
new run of the experiment's entire test suite.

## What remains

The [upstream extension](UPSTREAM-WORK-DISCOVERY.md) now announces missing
first-batch inputs from explicit governing production goals and separately tests
funded input-bid publication. It improves the metal-only startup modestly, but
neither-stock output still misses downstream procurement. The original six-pass
results above remain a historical comparison of their own policies.

Further work includes multi-batch scheduling with shared capacity, input bundles,
conditional acceptance/decommitment, financing and expected counterparty response.
The original request adapter reads signed stock policies and issuer
procurement; the extension also reads explicit governing production goals. It is not yet a general marketplace for arbitrary announced tasks,
negotiated delivery terms or recursively discovered production chains.
