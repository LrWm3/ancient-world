# Startup stocks and liquidity preferences

This follow-up to [startup forwards](STARTUP-FORWARD-PROPOSALS.md) separates two
questions: which supplied working stocks the acceptance economy still needs, and
whether each governor can distinguish plans that exhaust cash at different times.
It does not add money, resources, prices, employment, or automatic loan approval.

## Ten iterations

1. Matched workshop stock controls: retain ore and metal, either alone, or neither.
   Town ore remains zero; original cash and institutional terms are unchanged.
2. Shared completed-month observations and an opt-in reserve-exposure objective.
3. Consistent discovery, supplier, employment and finance forecast adapters.
   Changing the objective retains the same signed work/quote candidate catalog.
4. Separate baseline/proposed paths for each forward signatory's own forecast;
   an initial signed preference variant, with terminal-reserve behavior retained.
5. Competing prepayments, already-admitted seller claims, installment dates,
   delivery extensions/substitution, and frozen beneficiary controls.
6. Consent through leadership rotation and the existing one-shot rejection rule.
7. Opt-in ordinary path retention with independent tests that peer expectations
   still run for terminal-only planners.
8. Full 48-month CPU/reference, phase reconstruction, ledger replay and independent
   catalog-order comparison, plus a CPU command selecting stocks, authority and
   liquidity preference independently.
9. Eight 48-month stock/preference comparisons with pinned outcomes.
10. This report and updated entry-point documentation.

## What the preference measures

`Metric::Reserve` measures only the stock shortfall at the end of the forecast.
`Metric::ReserveExposure` sums the stock shortfall at each completed month's Close.
Its units are resource-unit months. Different resources remain separate entries
in the lexicographic objective vector. A governor compares alternatives over its
own equal-length horizon; the two governors' numbers are not added together.
Opening/emergency observations use a single boundary's shortfall. The fixture
replaces each organization's coin-reserve objective and any corresponding personal
preference before the initial mandate is sealed; it does not change preferences
at runtime. Targets and objective ordering remain the same.

The early zero-workshop-ore diagnostic illustrates the difference. The twelve-month
buyer forecast ends with zero coins in both branches, giving terminal loss 1,000
in each. Its baseline monthly cash is `7,6,5,4,3,2,1,0,0,0,0,0`; the proposed
branch is `7,4,3,2,0,0,0,0,0,0,0,0`. Exposure losses are 11,972 and 11,984.
Terminal scoring permits mutual consent; exposure refuses this candidate because
the buyer's projected loss is worse. This is a preference about liquidity timing,
not a universal solvency test or a requirement to refuse every investment.

## Evidence boundaries

`agency::path::Path` records after Close advances to the next Open, labeled with
the completed month. It retains dated stock balances and scheduled unpaid wages,
loan amounts and delivery quantities. Ordinary installments use the completed month's due
amount, not all remaining principal; delivery observations honor effective due
dates, substitution and relief. Later payment does not erase an earlier sample.

These are Close observations. They do not detect every intramonth missed payment
that another settlement cures before Close, and scheduled unpaid debt is not a
legal default finding. Comparisons share an opening boundary; if a caller starts
midmonth, the first remaining monthly interval contributes one Close sample,
without fractional-month weighting. Readiness continues to inspect historical ledger receipts.
There is no scheduler or collection-priority change.

Ordinary alternatives retain monthly paths only when an exposure objective or
preference requests them. Terminal-only alternatives keep an empty path. Bilateral
startup comparisons retain both parties' paths because their peer assumptions and
horizons can differ. Detail is bounded per forecast by its horizon, but retained
history and logs still grow with run length. Forecast observations are not assets,
reservations, or promises of counterparty action.

Member beneficiaries are frozen at the assessment boundary. Financial discovery
now also freezes beneficiaries for its existing objectives; this corrects cases
where a departing or dying member could previously disappear from the comparison.
The focused control removes the ending roster after freezing; it is not a new
membership-exit or death implementation.

## Measured outcomes

All cells run 48 months with startup-forward authority enabled. Workshop opening
ore and metal are each either the original four units or zero; town ore is zero.
Food deficit is cumulative reported nutrition shortfall, while wage arrears are
outstanding at the ending boundary. Production counts are completed batches.
Accounting reconciliation is required even for economically failing runs.

| Workshop stocks | Preference | Mints | Workshop | Town | Food deficit | Ending wage arrears | Forward signed / activated | Ready |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Ore 4, metal 4 | Terminal | 42 | 41 | 40 | 0 | 0 | 7 / 9 | Yes |
| Ore 4, metal 4 | Monthly exposure | 41 | 40 | 40 | 0 | 0 | 7 / 9 | No |
| Ore 4, metal 0 | Terminal | 0 | 0 | 0 | 67 | 2 | None | No |
| Ore 4, metal 0 | Monthly exposure | 0 | 0 | 0 | 67 | 2 | None | No |
| Ore 0, metal 4 | Terminal | 1 | 0 | 4 | 61 | 2 | 2 / 4 | No |
| Ore 0, metal 4 | Monthly exposure | 41 | 40 | 42 | 0 | 0 | 5 / 7 | Yes |
| Ore 0, metal 0 | Terminal | 0 | 1 | 4 | 67 | 2 | 1 / 3 | No |
| Ore 0, metal 0 | Monthly exposure | 0 | 0 | 0 | 67 | 2 | None | No |

Monthly exposure makes the metal-only economy viable under the existing readiness
gates. The governors sign later, in month 5 rather than month 2. These are outcomes
of changing both organizations' initial coin preferences; the experiment does not
isolate which governor or subsequent policy change caused the improvement.

With both stocks present, terminal scoring remains better on these gates. Exposure
has no food deficit or ending wage arrears, but fails the recurring-production-or-
demand gate: each full post-startup year must have at least twelve workshop batches,
twelve town batches and twelve metal sales. Other readiness checks pass. The CPU
command independently reproduces this exposure row, including signing in month 7,
admission scheduled for month 8, activation in month 9 and delivery due in month 10.

Neither preference bootstraps the cells without opening metal. The failing cells
have unmet nutrition, wage arrears and missing recurring production and ore supply.
Consent and completed initial deliveries are therefore insufficient acceptance
criteria. The original terminal preference remains the default; there is no
general claim that preserving cash earlier produces better plans.

## Commitment and governance controls

An earlier prepaid agreement consumes the same opening cash as a proposed startup
forward. An existing seller delivery prevents another admission under the ordinary
single-outstanding policy. Both projections and actual admission retain prior
terms; refusal does not cancel them. A second assessment begins with the prior
seller claim already admitted and outstanding.

A governor rotation between publication and activation preserves recorded consent
and the delivery obligation. The successor retains ordinary later policy authority;
there is no fresh signature requirement at activation. Failed admission leaves
productive programs inactive, but consumes the conservative one-shot startup
opportunity even after cash returns. Automatic retry or cancellation remains out
of scope.

## Reproduce

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example acceptance -- --startup-liquidity --workshop-stocks=both --monthly-exposure
cargo +1.92.0 run --locked --release --example acceptance -- --startup-liquidity --workshop-stocks=neither
cargo +1.92.0 test --locked --release --test financing_matrix --test forward_continuation
```

Stock choices are `both`, `ore`, `metal`, and `neither`. Omit `--monthly-exposure`
for terminal scoring; add `--no-startup-forward` to remove forward authority.
A successful CLI exit means the run completed, not that its readiness report passes.
Generated logs remain under ignored `output/`.

## Limits and next questions

Neither objective predicts future endogenous negotiation. Peer assumptions remain
frozen within a forecast. Exact prices, lot sizes, employment, initial money,
constitutions and permissions remain supplied. Removing one workshop input is
an ablation of this fixture, not proof that arbitrary supply chains can bootstrap.
A cash-preservation preference may also reject productive investment. Do not tune
stocks or widen rights just to make a preference win its comparison.

The next comparisons should keep the opening resources and readiness gates fixed:

- Change the workshop and town preferences separately in the metal-only and
  both-stock cells. The current comparison changes both, so it cannot attribute
  the improvement or lost production to one governor.
- Inspect refused proposals and projected purchases/sales in the no-metal cells.
  Compare a bounded counterparty response model or a longer explicitly signed
  planning horizon only after identifying the missing projected transaction.
- Distinguish improving proposal consent from improving actual timely payments,
  production and need satisfaction. A successful initial delivery is insufficient.

Exposure runs are substantially slower in this comparison. Private forecasts copy
the world's retained diagnostic history, and several phase adapters copy that
private world again. Monthly paths amplify that existing cost. Rejected proposals
can also remain eligible for repeated monthly assessment. Autonomous organization
reviews do not recurse inside forecasts, but old diagnostic copying is avoidable.
A future optimization could strip historical projections from private forecast
copies while preserving accepted programs, authority, commitments and live history.
That change needs an equivalence test before adoption; this batch retains the
existing copy semantics.

## Verification

The targeted verification set passes 127 distinct tests across 22 suites, including
25 library tests, existing agency/expectation/discovery/employment/publication
controls, the new path and commitment controls, four matrix tests covering eight
48-month cases, and the extended continuation test. This is a selected regression
run, not the entire crate's test suite. The matrix outcomes and agreement dates
are pinned as regression expectations; economically failing cells are expected
results, not ignored test failures.

The separate 48-month CubeCL CPU command completes and reproduces the both-stock
exposure result. Invalid documented stock choices are rejected. Formatting,
all-target release Clippy with warnings denied, and the repository artifact check
pass. Independent review checked the observation semantics, continuation limits,
result table and expected matrix values. Generated logs are kept outside Git.

The 48-month continuation test covers the original successful forward scenario:
both workshop stocks, terminal preference, and forward authority. CPU execution
reconstructs `Simulation` after every phase and matches the reference World,
State, ledger, reports and accounting audit. Replay reproduces State and the
audit using the reference World at each batch; it does not reconstruct World or
governance history from a bare ledger. Reordered independent catalogs preserve
State, ledger and audit. These are in-memory reconstruction checks, not serialized
checkpoint tests. The separate monthly-exposure CPU command checks reported
outcomes, not complete CPU/reference history equality for that preference.
