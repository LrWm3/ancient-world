# Two ways to discover cooperative agreements

Implemented as opt-in `production_market::Policy::Cooperate` variants:
`Discovery::Mutual` and `Discovery::Posted`. They use the same two-person
[calibration fixture](CALIBRATION.md), candidate work policies, six-month horizon,
fixed prices, contract terms, acceptance criteria and settlement path. The default
individual planner is unchanged.

## Shared agreement and bounded search

Each candidate agreement names both parties, their selected work preferences, a
start/end month and dated delivery-versus-payment obligations. The public offer
contains only delivery/payment terms and its proposer/expiry; it does not publish
a counterparty's work plan. Contract terms are kept in committed market history.

The initial menu is deliberately small: one resource unit per month in each
direction, expressed using existing market lots. One person delivers two grain for
four coins every second month; the other delivers one fuel for two coins monthly.
The reverse assignment is also considered. A six-month agreement therefore has
nine deliveries and twelve coins flowing each way. Prices come from the fixture's
fixed trader quotes. Quantities are a supplied menu, not yet generated from arbitrary
need sizes, bargaining or ZIP. No one is preassigned the farming or wood role.

Candidate work preferences are ordinary need-directed work, waiting, crop production
and fuel production, derived from the enabled process catalog. Existing processes
retain continuing-work priority. An accepted preference does not create labor,
seed, rights or outputs, and ordinary fallback work remains available.

Each person's outside option is its best candidate without exchange over the same
six months. Scores compare terminal outcome, priority-ordered need deficits,
aborted processes, a two-month closing need buffer, then actual productive labor.
Both must be no worse by that lexicographic score, and at least one must improve.
Each must also end the agreement with at least its opening coins. This restrictive
cash-neutral-or-better gate is specific to this recurring-exchange pilot; it is
not a general investment valuation policy or an absolute zero-deficit requirement.

All actual monthly cash/material constraints are checked in the joint forecast.
Ending cash alone does not establish affordability. The same bounded resource and
storage checks run again against live holdings before each delivery batch.

## Mutual plans

The matcher enumerates both goods assignments and the Cartesian product of work
candidates. It projects the combined arrangement, evaluates each participant's
outcome against its own outside option, and selects a mutually acceptable candidate.
Both may reject. Candidate arbitration is explicit and deterministic: scores in
sorted participant-ID order, then stable candidate enumeration. This favors the
first participant among mutually acceptable alternatives; it is not a fairness
claim or a population-wide welfare optimizer.

## Market-posted agreements

Each person evaluates potential terms against its **own** candidate work plans and
posts at most one beneficial offer. Offers expire at that Acquire boundary. The
other person independently searches its own work choices against the published
terms and may accept or reject. A final joint feasibility/benefit check precedes
acceptance; only one agreement can be active for this pair.

Individual forecasts are explicitly conditional on promised deliveries. In a
private clone only, the counterparty is given enough hypothetical outgoing stock
and coins to honor the promise, its work choice is replaced with Wait, its active
processes are removed and its needs disabled. The evaluating person's endowments
and outgoing budgets are unchanged. A focused test proves changing the hidden
counterparty work choice cannot change this conditional assessment. These are
promise assumptions, not observations, live grants or guarantees.

The final joint check removes those hypothetical endowments and executes **both
actual selected plans** from actual opening resources. Thus an attractive proposal
can still fail acceptance. This check has full access to submitted plans; the
pilot does not establish distributed proof of feasibility or private-information
market clearing. Posted offers are bilateral and scoped to the existing pair,
not a persistent general marketplace offer book or common-offer adapter yet.

Both approaches use `cooperation::Contract`, the existing finance transfer primitive,
Acquire resource reservations and the ordinary CPU transaction commit. This adds
a scoped agreement domain, not a second universal contract interpreter.

## Timing, reservations and consequences

Open admits participants and establishes capacities. Acquire discovers/accepts an
agreement when none is active, then settles that month's dated trades. Accepted
work preferences guide Productive; production and consumption use existing rules.
Renewal is considered after expiration. Neither approach reads future capacity
shocks; forecasts go through `ForecastContext`.

All outgoing legs draw from opening balances. Incoming coins cannot fund another
outgoing leg in that same Acquire batch. Storage and participant eligibility are
checked as well. Transactions and the dated agreement receipt are re-evaluated
before atomic publication, so forged or replayed acceptance cannot reserve twice.
Only one active agreement is supported, preventing overlapping accepted proposals
in this isolated driver. Future labor and stocks are **not escrowed**: the selected
work policy persists, ongoing processes reserve through existing machinery, and
future delivery is an obligation subject to failure. This is not multi-agreement
future-capacity allocation. Switching between the two discovery variants preserves
an active agreement; removing the cooperative planner while one is active is rejected.

If any scheduled trade is unfundable or a participant is unavailable, none of that
month's agreement trades settle. The receipt records failure and cancels all remaining
deliveries. Previous trades remain final; no refunds, damages, arrears collection,
reputation penalty or insurance are invented. Existing production can continue under
ordinary work policy. Parties may propose a new agreement next month.

Scheduled delivery uses its accepted terms rather than the spot order generator's
six-month stock-protection threshold. The full-cycle forecast and closing buffer
are its acceptance test. Consequently comparison with the earlier autonomous spot
baseline changes both planning and interaction rules; only the two new mechanisms
are a controlled discovery comparison.

## CPU results

Both mechanisms produced identical economic paths in these deterministic controls:

| Run | Food deficit | Warmth deficit | Grain traded | Fuel traded | Ending coins 88 / 91 | Accepted / completed / failed agreements |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| Mutual, 72 months | 0 | 0 | 70 | 71 | 22 / 26 | 12 / 11 / 0 |
| Posted, 72 months | 0 | 0 | 70 | 71 | 22 / 26 | 12 / 11 / 0 |
| Mutual, lost first harvest, 24 months | 15 | 5 | 8 | 9 | 22 / 26 | 2 / 1 / 1 |
| Posted, lost first harvest, 24 months | 15 | 5 | 8 | 9 | 22 / 26 | 2 / 1 / 1 |

The unshocked agreement starts in month 1; month 7 renewal is declined; another
starts in month 8, then every six months. Month 72 is inside the last agreement,
so ending 22/26 cash reflects unmatched timing within that cycle. Every completed
agreement ends at 24/24. Final `(grain, fuel)` holdings are `(8, 5)` and `(4, 5)`;
total productive labor is 125 units. This is repeated feasible exchange under
finite money/storage, not a proof of indefinite sustainability.

Logical forecast counts in the accepted/rejected discovery receipts total **520
for mutual versus 436 for posted** over 72 months. The shock runs use 640 versus
480. Counts include outside-option searches and final checks, but exclude repeated
validation of the same receipt; they are not runtime benchmarks. Fewer candidates
are explored by posted offers, which can also miss an arrangement that a second
choice offer or reply would find. Equal outcomes here do not establish equivalence
between the mechanisms.

The shock removes person 88's labor in month 3 after the initial agreement has
been accepted, aborting its first crop. Month 4's grain delivery fails and the
agreement cancels, preserving the earlier coin transfers. One new agreement starts
in month 13 and completes, but recovery is incomplete. The aborted crop also loses
its seed input, so this is persistent productive damage from one missed harvest,
not merely a one-month income delay. No seed market, emergency financing or restart
assistance is present. Forecast acceptance never guaranteed resilience to this shock.

## Running and verification

From `exp/economics`, use fresh output directories:

```sh
MONTHS=72 TELEMETRY_DIR=../../output/economics/cooperation-demo \
  cargo +1.92.0 run --locked --example cooperation
SHOCK=harvest MONTHS=24 TELEMETRY_DIR=../../output/economics/cooperation-shock-demo \
  cargo +1.92.0 run --locked --example cooperation
cargo +1.92.0 test --locked --test cooperation
cargo +1.92.0 test --locked --lib cooperation::tests
```

The runner executes both mechanisms and writes ordinary observer JSONL. Cooperative
records include public offers, accepted dated terms, individual assessments, logical
projection counts, completed deliveries and failures. Market volume/price records
include agreement settlements; these do not fabricate spot orders or negotiation
attempts. Raw logs remain under ignored `output/`.

Focused tests cover 72-month finite-budget operation, surprise failure, zero-cash
rejection, no reuse of incoming money, marketplace admission, atomic rejection of
forged/replayed receipts, conditional-information isolation, and observer neutrality.
Both discovery modes compare CPU monthly/checkpoint execution against reference
batched execution, including the shock and reordered participant/process catalogs.

All 56 focused tests passed: seven cooperative integration tests, one conditional
forecast unit test, three calibration, nine production-market, nine reciprocal-market,
ten telemetry and seventeen town-market tests. The final checkpoint/receipt guards
were also rerun directly. Strict all-target Clippy and formatting checks passed;
the full crate suite was not run.

The next useful comparison is a richer but still bounded offer menu: quantities,
dates or cashflow timing that permit one mechanism to find an arrangement the other
misses. Shock recovery is a separate question involving seed replacement, reserves
or negotiated restructuring; discovery alone did not solve it.
