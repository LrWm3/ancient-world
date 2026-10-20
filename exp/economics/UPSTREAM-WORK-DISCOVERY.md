# Upstream input interest and bid publication

Status: opt-in extension of [work discovery](WORK-DISCOVERY.md). This adds
missing-input discovery and a separately configurable publication policy. It does
not establish autonomous supply-chain startup or change default policies.

## Conditional interest

`agency::Config.work_requests = Upstream` retains request-filtered work plus
signed work/quote bundles. It also exposes missing first-batch stock inputs for
an organization's explicit, active governing `ProcessShortfall` goals.

The requester must be autonomous, discovering, eligible for the marketplace,
legally permitted to execute the recipe, governed by an occupied office, and
signed for both the recipe and the input buy options. A pending startup forward
suppresses this interest while its conditional program awaits activation. A cash
goal alone does not manufacture input demand, and an inactive governor's preferences
do not override the active governor.

Each `ProspectiveInput` record freezes the recipe, governing goal, observed stock,
protected stock, signed target/ceiling and planning horizon. It derives demand from
one batch's missing first-stage stock inputs, rounds up to market lots, and caps
it by the signed target gap (also rounded to lots) and lot ceiling. Duplicate input entries add within a batch; alternative goals/recipes
do not add demand at the same requester/market/date. An existing concrete request
wins an identical key, and sufficient standing interest suppresses repetition.

The output window is the observation month plus the requester's horizon. The input
window subtracts recipe duration and must allow next-Open activation. This is an
**aspiration**, not an accepted delivery date or a promise of continuous work.
Fresh observations renew the window; old decision evidence keeps its original date.
No request reserves stock, cash, labor or a counterparty. Unfunded interest remains
visible and explicitly unverified. Existing caps fail closed rather than silently
truncating requests.

This version covers own explicit production goals and first-stage stock inputs.
It does not recursively expand a customer's requests, later-stage inputs, services,
assets, rights or joint input bundles. Different dates can describe alternative
plans; they are not summed into a guaranteed stream of purchases.

## Separately governed input bids

`publication::Policy::InputBidsOnEqualProjection` extends productive-ask
publication. When no improving program is selected, a quote-only candidate may
break an exact forecast tie with waiting if:

- existing eligible stocked-sale publication cannot already break that tie;
- every existing quote in the replaced portfolio is preserved;
- all added quotes are signed buys matching the actor's own upstream interest;
- the entire proposed buy portfolio fits current uncommitted payment stocks,
  including retained bids and accepted claims, separately by denomination.

The input request identifies relevant markets and price ceilings. A bid may
replenish the **full signed stock target**, which can exceed the request's single
first-batch quantity. The affordability check covers that full target, subject to
its lot ceiling. This is bounded inventory exploration, not a same-quantity
conversion of each request into an order.

The forecast is still independent. Equal projected losses do not establish that
unmodeled counterparty responses will be beneficial. Publication does not escrow
funds; next-Open activation and ordinary Acquire recheck eligibility, funding,
stock and storage. Other accepted work can still consume resources before a trade.

An initial trial let candidate order choose between a stocked sale and an input
bid. In the metal-only case, the bid won at month 1, delaying the metal ask until
month 2 and missing the issuer's month-2 procurement. Mints fell from 8 to 0.
The final policy explicitly tries stocked sales before input exploration. A
reversed-candidate-order test guards this priority; no scheduler change was made.

## Controlled comparisons

All runs use the existing production-first acceptance fixture, prices, resources,
finance and law. Within each opening-stock case only the named discovery or
publication policy changes. Metrics describe completed production and audited
settlement, not forecast output. Food deficit is cumulative; wage arrears are
outstanding at the ending boundary.

Announcing upstream interest alone leaves **all recorded 12-month evidence**
unchanged versus `RespondWithWorkQuotes`:

| Opening workshop stocks | Mints | Workshop batches | Town batches | Food deficit | Wage arrears |
| --- | ---: | ---: | ---: | ---: | ---: |
| Both | 10 | 11 | 10 | 0 | 0 |
| Metal only | 8 | 7 | 8 | 0 | 0 |
| Ore only | 10 | 11 | 10 | 0 | 0 |
| Neither | 0 | 2 | 4 | 5 | 1 |

Prospective-input records occur three times in metal-only histories and twice in
neither-stock histories. These are repeated observations, not unique obligations
or fulfilled orders. The town can now see missing ore interest before a standing
buy policy exists. Visibility alone does not make supplying it worthwhile.

With explicit stocked-sale priority, input-bid publication improves metal-only
startup to **9 mints, 8 workshop batches and 9 town batches**, versus 8/7/8 without
it. Workshop net income rises from 4 to 6 and town closing cash from 12 to 14;
Issuance rises from 80 to 90 and workshop metal sales from 8 to 9; food deficit
and wage arrears remain zero. All other recorded 12-month evidence is unchanged,
including the neither-stock failure.

Equal totals conceal a different path in the neither-stock case. With input bids,
the workshop posts an ore bid in month 1; the town chooses mining plus an ask in
month 2, produces ore in month 3, and the workshop buys and refines in month 4.
It refines again in month 5. No workshop startup forward is accepted on this path.
The issuer retains only its original month-2 procurement date and adds none through
the 12-month run. The workshop finishes with four unsold metal units, no metal
sales and no cash. At Open 5, when the first metal becomes available, the issuer
has seven coins; its complete metal-and-labor input package requires eight.
It had eight at Open 4, before that output was available. The order adapter emits
no input bids when current opening funds cannot cover that package. This is a
specific timing-and-budget mismatch; workshop cash exhaustion follows later.
Upstream response succeeds, but another ore request or an unfunded procurement
retry cannot by itself establish downstream demand.

## Verification and reproduction

Run from `exp/economics` with Rust 1.92.0:

```sh
cargo +1.92.0 test --locked --release -j8 --test upstream_requests --test upstream_comparison
cargo +1.92.0 test --locked --release -j8 --test upstream_bid_comparison -- --nocapture
cargo +1.92.0 test --locked --release -j8 --test upstream_continuation
cargo +1.92.0 test --locked --release -j8 --lib agency::publication
cargo +1.92.0 run --locked --release --example work_discovery -- neither input-bids
```

Use `upstream` instead of `input-bids` for interest without the new publication
policy. Raw output belongs under ignored `output/economics/` at repository root.

Focused controls cover active preferences, inadequate horizons, process permission,
already sufficient stocks, duplicate inputs/goals, concrete-request precedence,
frozen dates, publication opt-in, exact ties, no added work, quote preservation,
protected payment claims, shared portfolio budgets and pending-forward suppression.
Continuation compares Reference with CubeCL CPU reconstructed at every phase,
including activation before acquisition. Ledger replay, audit reconciliation and
reversed independent catalogs check state, decisions and completed outcomes.

## Remaining boundary

A missing input request is useful explanatory evidence, but is neither a funded
award nor a viable multi-party plan. The neither-stock fixture still has a startup
timing problem: initial issuer procurement is due before newly produced metal can
arrive, while later metal output does not by itself sustain issuer demand.
The issuer also lacks the full input budget when output first arrives. The
input-bid path uses spot purchases without a workshop startup forward.

The next bounded experiment should compare a downstream procurement retry with
an explicitly funded later delivery window or independently accepted forward.
Keep money conserved and show whether the buyer preserves its future input budget;
a later date alone cannot cover the observed shortfall. Preserve affordability,
withdrawal/default consequences and a no-supply control. Recursive request
propagation, joint funding,
capacity reservations, arbitrary chain formation and 48-month acceptance readiness
remain unverified for these policies. CUDA is also unverified.

Final targeted verification: 79 tests passed (41 library, 3 organization discovery,
5 organization forwards, 1 publication execution, 4 request discovery, 7 work
requests, 3 batch explanations, 5 upstream requests, 4 upstream comparisons,
4 input-bid comparisons and 2 continuation tests). The two 24-month continuation
cases use input bids with metal-only and neither-stock starts. Formatting, strict
all-target Clippy and the repository artifact gate passed. This is targeted
coverage, not a fresh run of the entire experiment suite. Independent review
covered implementation, budget controls, priority and the failure diagnosis.
