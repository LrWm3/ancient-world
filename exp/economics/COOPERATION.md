# Two ways to discover cooperative agreements

Opt-in `production_market::Policy::Cooperate` compares `Discovery::Mutual`
with `Discovery::Posted` in the two-person [calibration fixture](CALIBRATION.md).
Both use the same bounded terms menu, work candidates, six-month horizon, fixed
prices, individual acceptance criteria and dated settlement. The default individual
planner is unchanged.

**Posted acceptance has no centralized joint feasibility forecast.** Each person
consents using its own conditional projection. Mutual search remains a centralized
comparison. Neither mechanism guarantees performance after an unexpected shock.

## Shared terms and decisions

Each contract records parties, selected work preferences, start/end dates and
delivery-versus-payment obligations. Public offers publish deliveries and payments;
the recipient does not receive the proposer's work plan as a planning input.
Work choices and assessments remain available in diagnostic receipts.

The menu exchanges six grain and six fuel over six months, with twelve coins
flowing each way. Grain trades in two-unit lots for four coins; fuel trades in
one-unit lots for two coins. Both seller assignments are available. For each
assignment there are two schedules:

- Regular: grain every second month, fuel monthly.
- Deferred seller delivery: the proposing seller delivers all its goods in the
  final month; the other party delivers regularly.

Payment always occurs with delivery. Deferring goods also defers their payment.
Amounts and prices do not change. Mutual search receives the union of both
proposers' menus, deduplicated. These are fixture terms, not inferred quantities,
negotiated prices or ZIP.

Work candidates come from the enabled process catalog: ordinary need-directed
work, waiting, crop production and fuel production. Continuing processes retain
priority; a preferred activity does not create labor, seed, rights or outputs.

Each person's outside option is its best no-exchange candidate over six months.
Scores compare terminal outcome, priority-ordered deficits, missed loan payments, aborted processes,
a two-month closing need buffer, then productive labor. Both must be no worse
than their outside option; at least one must improve. Each must avoid projected loan arrears and finish with at
least its opening cash minus accepted loan debt, measured on the same basis at
the horizon. With no loans this is the original closing-cash test. This is a restrictive recurring-exchange
rule, not a general investment policy. Acceptance does not impose an absolute
zero-deficit requirement.

## Mutual search

The matcher enumerates schedules and pairs of work preferences, runs joint
forecasts from actual resources and checks both people's outcomes. Acceptable
candidates are ranked by score in sorted participant-ID order, then enumeration
order. This privileges the first person's outcome among acceptable alternatives;
it is not a fairness or social-welfare optimizer.

## Posted offers, rejection and revision

Within Acquire, proposers take turns in stable participant-ID order. A proposer:

1. Independently evaluates each schedule against its own work candidates.
2. Retains its best strictly beneficial, affordable plan for each distinct set of terms.
3. Ranks those offers by its own score. Equal scores retain menu order, with
   deferred delivery before regular delivery.
4. Posts its first offer. The recipient evaluates its own work candidates against
   those public terms and accepts its best acceptable response.
5. If rejected, posts the next ranked terms. If all are rejected, the next
   proposer takes its turn.

The first agreement with two consents is accepted. There is no joint rollout,
central comparison of the two submitted plans, or joint-score arbitration.
Revision exhausts a small fixed menu within one boundary; it does not learn new
terms from rejection, persist an offer book or negotiate across months. Only one
agreement may be active for this pair.

Individual forecasts assume the other party honors its promises. In a private
forecast clone, that party's balances are replaced by its promised outgoing
quantities and coins, its work becomes Wait, and its needs, active processes and
storage limit are removed. The evaluating person's resources and budgets are
unchanged. Even no-trade outside options use this individual projection path.
A test changes the other person's selected work, capacity, food and coins without
changing the evaluating person's assessment.

These hypothetical resources never enter live state. The code still uses the
existing world forecast engine and records both choices for execution; it is not
a separate distributed runtime or a general privacy boundary. This isolated
fixture has distinct plots and one active agreement. Conditional forecasts do
not solve competing future claims on shared resources or overlapping promises.

## Timing and consequences

Open establishes capacities and market admission. Acquire discovers an agreement
and settles due trades. Productive uses the accepted work preferences; production
and consumption follow the existing schedule. Future shocks are removed by
`ForecastContext`. Monthly, batched and checkpoint execution use the same phases.

Each monthly package reserves all outgoing legs against opening money and goods.
Incoming coins cannot finance another outgoing leg in that batch. Live storage,
market eligibility and finite budgets remain enforced. These are settlement
checks, not a prediction that future promises are feasible.

Any failed delivery cancels the entire current package and all remaining
deliveries. Earlier trades remain final. Receipts identify the failing resource,
party, date and available budget where applicable. There are no refunds, damages,
insurance, arrears collection or automatic seed replacement. Parties may try
again next month.

Committed receipts and transactions are re-evaluated before atomic publication,
including replay/forgery checks. For Posted, this repeats the independent
assessments; it does not restore a joint forecast. Future labor and stock are
not escrowed. Changing discovery mode preserves active agreements; abandoning
the cooperative planner while an agreement is active is rejected.

Both mechanisms use `cooperation::Contract`, finance transfers and existing
Acquire reservations. This remains a scoped contract domain rather than a new
universal agreement interpreter. Scheduled exchanges also differ from the earlier
spot-order planner's stock-protection rule, so that earlier baseline is not a
controlled comparison of discovery alone.

## CPU controls after removing the posted joint check

| Control | Mechanism | Food / warmth deficits | Grain / fuel traded | Ending coins 88 / 91 | Accepted / completed / failed |
| --- | --- | --- | --- | --- | --- |
| Normal, 72 months | Mutual | 0 / 0 | 70 / 71 | 22 / 26 | 12 / 11 / 0 |
| Normal, 72 months | Posted | 0 / 0 | 70 / 71 | 22 / 26 | 12 / 11 / 0 |
| Lost first harvest, 24 months | Mutual | 14 / 7 | 6 / 11 | 14 / 34 | 2 / 1 / 1 |
| Lost first harvest, 24 months | Posted | 14 / 7 | 6 / 11 | 14 / 34 | 2 / 1 / 1 |

Normal runs finish with grain/fuel stocks of 8/5 for person 88 and 4/5 for person
91. Every completed agreement restores coins to 24/24; 22/26 at month 72 is
within-cycle timing. Mutual uses 1,352 logical projections, including 1,248 joint
projections. Posted uses 384, all individual, and records two rejected offers.
These counts exclude repeated receipt validation and are not runtime benchmarks.

With the larger initial food reserves, both accept a deferred grain schedule.
The surprise loss of person 88's month-three labor destroys its first crop.
Delivery consequently fails in month six, after five fuel purchases, leaving
14/34 coins. The lost seed continues to limit recovery. Shock runs use 1,456
projections for Mutual (1,344 joint) and 660 for Posted (zero joint), with 31
rejected posted offers. The new schedule changes exposure relative to the
previous regular-delivery-only experiment; these are replacement results.

### Controlled rejection and revision

Give the wood producer two initial grain instead of six; keep all other
resources, prices and production rules unchanged. Its own forecast rejects
waiting until month six for grain. The same proposer revises to delivery in
months two, four and six, which the recipient accepts. Mutual search also selects
regular grain delivery. Both complete the six-month agreement with no unmet
food/warmth needs and coins restored to 24/24.

Posted receipts retain the proposer assessment, every recipient candidate
assessment, rejected terms and accepted revision. Proposer ranking is private
score followed by stable menu order: a first offer may be preferred by a tie,
rather than a strict improvement. This control demonstrates useful refusal and
revision without jointly inspecting the parties' plans. It does not establish
general convergence, fairness, shock resilience or shared-resource feasibility.

## Running and verification

From `exp/economics`, use fresh ignored output directories:

```sh
MONTHS=72 TELEMETRY_DIR=../../output/economics/cooperation-demo \
  cargo +1.92.0 run --locked --example cooperation
VARIANT=low-food MONTHS=6 TELEMETRY_DIR=../../output/economics/cooperation-revision-demo \
  cargo +1.92.0 run --locked --example cooperation
SHOCK=harvest MONTHS=24 TELEMETRY_DIR=../../output/economics/cooperation-shock-demo \
  cargo +1.92.0 run --locked --example cooperation
cargo +1.92.0 test --locked --test cooperation
cargo +1.92.0 test --locked --lib cooperation::tests
```

Observers expose dated terms, proposer/reply assessments, rejected and accepted
offers, total/joint projection counts, completed trades and failures. Financial
volume and posted-price observations include contract deliveries without
fabricating spot orders. Raw logs and JSONL stay under ignored `output/`.

Focused coverage includes repeated finite-money operation, rejection/revision,
zero-cash refusal, hidden-harvest failure, no same-batch money reuse, admission,
forgery/replay, individual forecast isolation and observer neutrality. CPU monthly
and checkpoint execution are compared with reference batches and reordered
participant/process catalogs for both modes, with and without the shock.

Validation passed: eight cooperative integration tests, one individual-projection
unit test, and 48 regression tests across calibration, production markets,
reciprocal markets, telemetry and town markets. The final receipt additions and
strict proposer-preference assertion were also checked directly. Strict
all-target Clippy and formatting passed. The full crate suite was not run.

## Configured agreements with ordinary lending

A supplied `Policy::Agreement` now settles after loan admission using the shared
Acquire budget. Loan receipts cannot pay a current delivery, and a participant
cannot lend and spend the same opening coins. Sufficient independent funding
settles both; retained borrowed cash may pay a later delivery after Due collection.
Forecast discovery receives the original acquisition state, while settlement uses
the quoted legal state and remaining reservations, including pooled storage.

The same cancellation consequence applies to supplied agreements: a failed
package leaves earlier exchanges final and cancels subsequent deliveries, even
when new loan money becomes spendable next month. Resuming a checkpoint cannot
restart that cancelled agreement. The separate loan continues to be serviced.

`tests/cooperative_lending.rs` checks these boundaries, CPU/reference equality,
reconstructed continuation, separate double-entry statements and tamper rejection.
Work is held at Wait in this control to isolate financing and delivery; this is
not evidence that its six-month plan meets needs or earns its repayments.
Individual cooperative discovery can now include configured borrowing under the
debt-aware acceptance test below. This adapter does not approve borrowing on an
agent's behalf or introduce new negotiation terms.

A household control additionally fills shared storage with seed while leaving
private room for a grain purchase. The required pooled contribution blocks the
whole delivery when shared room is exhausted; with room it pools exactly once.
The member's separate loan asset and the loan's unpooled principal remain intact.

## Loan-aware individual acceptance

Both discovery methods retain actual accepted loans and simulate the configured
admissions, installments and arrears. Receipts expose opening and closing debt and
missed payments; observers show the same fields. The acceptance floor is cash
minus principal and accrued interest. Newly borrowed cash cannot disguise interest
expense or unpaid principal as exchange income. A projected missed installment
rejects a candidate even if its no-exchange outside option also misses payment.

The controlled pair accepts an interest-free, externally funded loan plus mutual
exchange, but declines the same terms with uncovered monthly interest. Actual
borrowing is supplied consent and still executes even when exchange is declined;
these agents are choosing an exchange plan, not underwriting their loans.
CPU/reference and reconstructed continuation agree, and forged debt assessments
are rejected. This does not value receivables, inventory, collateral appreciation,
or unrelated claims as spendable cash. It is deliberately a conservative
recurring-exchange constraint, not a general solvency or investment test.

Conditional forecasts retain observed financial obligations; only the other
party's promised delivery resources are hypothetical. They do not guarantee that
competing financial claims will leave those resources available. Household
cooperative discovery with lending remains excluded until its conditional
forecast preserves governed allocations and separate financial scopes. Supplied
household agreements continue to use the tested settlement adapter.

## Accepted terms survive planner configuration changes

A checkpoint with an active cooperative agreement rejects edited price, delivery
date or work terms under `Policy::Agreement`. Changing the discovery mode keeps
servicing the accepted schedule. A regression reproduced silent price replacement
before this guard; `tests/cooperative_terms.rs` now checks refusal and CPU
continuation with the original consideration. This is immutability during the
accepted term, not a negotiated amendment mechanism.

## Common agreement inspection

`agreements::for_agent` now returns `View::Exchange` for accepted cooperative
packages. Committed boundaries retain the accepted schedule even if the first
package fails. The view aggregates actual completed deliveries and preserves
active, completed or failed status without another balance or contract book.
Unaccepted catalog schedules do not appear; either party receives the same view.

`View::parties()` is the common participation interface, including all three
parties to a guarantee. `holder()` and `grantor()` now return optional roles:
reciprocal exchange has no privileged holder/grantor. Asymmetric loan, land,
process and guarantee roles remain available.

Exchange `claims()` is empty deliberately: each goods/payment pair is conditional
and settles atomically. It cannot be split into two independently collectible
waterfall claims. Failure retains prior deliveries and cancels the rest under the
existing terms; it creates no damages, arrears or refund obligation. The complete
schedule remains inspectable in the typed exchange view. Tests check successful
completion, first-delivery failure, CPU/checkpoint parity, read-only inspection,
and rejection of missing accepted terms.


## Composition planner follow-up

The later [conditional spot adapter](PLANNER-POSTED.md) reuses `Delivery` terms and
normal town-book transfers for an atomic reciprocal package. It adds independent
composition assessments after public preliminary masks, without using this pilot's
joint discovery or prescribed work preferences. Only the current month's terms
bind; composing this document's future schedules and cancellation consequences
with the newer search remains a proposed next integration. The two reports use
different fixtures and are not matched performance comparisons.
