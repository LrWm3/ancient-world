# Financed startup of the mixed economy

The configured nine-agent economy now passes the unchanged
[48-month acceptance gates](ACCEPTANCE-READINESS.md) with **neither opening ore nor
metal at the workshop**. People, a household, workshop, town, state and marketplace
operate through existing production, employment, spot markets, forwards, lending,
governance and separate double-entry books. This is a bounded integrated economy;
it does not establish that arbitrary populations discover viable institutions.

## The gap and the repair

[Upstream work discovery](UPSTREAM-WORK-DISCOVERY.md) already brought mining and
refining online. The issuer's original procurement date preceded that output, and
its remaining coins could no longer fund the full metal-and-labor package. Its
accepted bootstrap debt continued to collect normally. A later procurement date
alone could not repair this liquidity gap.

An optional agency financing mandate now permits a borrower to propose a bounded
advance tied to an authorized productive program. A separately consenting lender
must offer compatible terms and currently hold the denomination. The existing
credit adapter handles admission, transfer, servicing, arrears and accounting;
this is proposal discovery and authorization, not another lending implementation.
There is no additional endowment, free input, waived debt or scheduler change.

Both parties independently compare retaining current commitments, executing the
same authorized work without borrowing, and accepting the proposed advance with
productive action. Self-funded alternatives cover both the ordinary next Open
and the delayed financed date. They must actually complete and service existing
debt and wages; the borrower must prefer them to retaining before the lender can
count them as outside options. Receipts retain each party's best self-funded date
and score. This prevents an unnecessary loan from postponing already viable work.
Each uses its own governing objectives and peer-expectation policy. Neither may
score worse than its best outside option, at least one must improve,
and both must project completion and timely repayment. Projection covers the new
maturity and the existing loans of both parties, within the existing 24-month
forecast limit. It rejects projected loan arrears and either party's missed wages.
Actual counterparties outside the agreement are not guaranteed profitable outcomes.

One finite bridge is allowed per borrower. The pilot supports a fixed, authorized
single-process program, principal up to 16 units, terms up to 12 months and at most
64 candidate proposals per boundary. Exceeding the search bound fails explicitly.
The acceptance fixture permits up to eight coin units, six-month zero-interest
terms and a one-month first-payment delay. The state can borrow for its mint
program; the town can lend. Legal borrowing/lending permission remains separate
from these static mandates. Other scenarios default to financing disabled.

## Timing and authority

The sequence is explicit:

1. Open evaluates ordinary operating alternatives from the shared opening snapshot.
   The subsequent bilateral review can observe those now-public dated choices.
2. Both parties accept a proposal; publication records terms and seals their
   mandates. It changes no balances and creates no admitted loan.
3. The following month's Acquire rechecks law and current lender funds through
   ordinary credit admission. Incoming funds cannot finance another purchase in
   that same acquisition batch.
4. The next Open activates the captured productive program only if admission
   succeeded and current process permission allows it. Actual inputs, labor and
   capacity still determine completion.

The successful startup proposes a two-coin loan in month 3, admits it in month 4,
and activates mint work in month 5. Refusal of admission produces no funded work;
revocation of process permission after admission leaves a real debt but blocks
activation. Neither case silently cancels an accepted loan.

Historical authorization survives later office vacancy, just as other accepted
commitments do. Requiring a new governor at execution had postponed financing by
one month and caused a real workshop wage miss. That failed trial is not counted
as success. Pending agreements now hold competing operating choices while allowing
members to cast election ballots. Financing and startup forwards respect one
another's pending commitments and minimum-tenure windows. A second review can fund
a choice published at the same Open; older choices still impose tenure, and review
cadence is unchanged.

Real agreement history always requires the exact parties, their historical
governors and mutually accepted comparisons, even if future autonomous planning
is disabled. Only private, fully frozen forecast branches can carry an explicitly
hypothetical agreement without consent. Activated programs appear in the common
accepted-program reporting and forecast-provenance interfaces.

## Observed economy

The successful 48-month reference run has:

- 44 completed mints and metal sales, producing 440 coin units through physical
  minting; all nine agent books reconcile.
- 45 refining jobs and 46 mining jobs across startup and continuing operation.
- 48 paid wage coins from each employer, no historical wage or loan arrears,
  no nutrition deficits, and both household-forward units delivered on time.
- Full repayment of the initial state loan and the newly accepted financing.

Each of months 13–24, 25–36 and 37–48 independently completes 12 mining jobs,
12 refining jobs, 12 metal sales, 24 ore deliveries and 12 paid hours for each
employer. Workshop net income is 12 per window and town net income is 13. Workshop
cash rises 10 → 22 → 34 → 46; town cash rises 16 → 29 → 42 → 55. These are measured
outcomes under supplied prices and contracts, not inferred from conservation alone.

## Reproduction and checks

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example working_capital -- --cpu
cargo +1.92.0 run --locked --release --example working_capital -- 12 --details
cargo +1.92.0 run --locked --release --example working_capital -- --without-finance
cargo +1.92.0 test --locked --release --test working_capital
cargo +1.92.0 test --locked --release --test working_capital_controls
cargo +1.92.0 test --locked --release --test working_capital_continuation
```

The default run checks 48-month readiness and exits unsuccessfully if a gate fails;
the no-financing command is an intentional failing control. Raw logs belong under
ignored repository-root `output/economics/`.

Boundary tests distinguish publication, admission and activation, preserve election
ballots, reject forged consent and changed mandates, and test lender depletion,
revoked process permission and office vacancy. Continuation coverage compares CPU
with reference execution, reconstructs the simulation after every phase, replays
ledger state and audits, and reverses independent catalog order. The existing
startup-forward path also passes its CPU, phase-reconstruction, replay and catalog
reordering regression after the shared pending-agreement changes.

Matched controls keep prices, endowments and ordinary settlement unchanged except
for the named intervention:

| Configuration | Months | Completed mints | Outcome |
| --- | ---: | ---: | --- |
| Neither opening input; financing authorized | 48 | 44 | Passes unchanged readiness |
| Neither opening input; financing disabled | 48 | 0 | Fails readiness; input production alone cannot fund the issuer |
| Both opening inputs; financing enabled / disabled | 12 | 10 / 10 | Existing successful startup preserved |
| Ore only; financing enabled / disabled | 12 | 10 / 10 | Existing successful startup preserved |
| Metal only; financing enabled / disabled | 12 | 9 / 9 | Existing successful startup preserved |
| No ore supply | 12 | 0 | No financing accepted; scarcity remains |
| No metal-market access | 12 | 0 | No financing accepted; access restriction remains |
| Low metal price | 48 | 3 | Workshop loses money; fails readiness |
| Late payroll | 48 | 0 | Historical wage arrears fail readiness |

The twelve-month controls establish the stated startup behavior, not full
48-month readiness. Brief production in the low-price control does not establish
a viable chain. Readiness inspects historical missed payments as well as ending
balances, so later repayment cannot erase a failed boundary.

Final verification passed 77 distinct targeted tests: 41 library, 14 agency,
2 agency-program, 2 forecast-context, 2 forward-governance, 5 organization-forward,
3 working-capital boundary/readiness, 6 working-capital control, 1 financed
continuation and 1 existing forward-continuation test. The latter selects
`forward_startup_matches_cpu_phase_reconstruction_replay_and_catalog_reordering`
from `forward_continuation`. Formatting, all-target release Clippy with warnings
denied, diff whitespace checks and the repository artifact check also passed.
Independent review found no remaining material issue after the consent,
self-funded comparison and governance corrections. This was targeted verification,
not a rerun of the entire experiment test suite.

## Remaining scope

Initial agents, institutions, employment, land terms, the initial loan and household
forward, prices, recipes, objectives and constitutional authority are supplied.
The household and town already exist; this does not prove autonomous founding or
population growth. The one-bridge limit is deliberate, not a revolving-credit
policy. Loan terms and candidate order are supplied rather than negotiated; equal
candidates use stable enumeration. Lending funds are rechecked at admission, not
escrowed when proposed. Forecasts are bounded hypotheses, not guarantees of future
work, leadership or payment.

Arbitrary multi-party project finance, recursive economic discovery, market-driven
pricing across the whole chain, sustained shock recovery in this new financed
configuration, larger populations, performance at scale and CUDA execution remain
outside this acceptance result. Existing independent experiments cover some of
these mechanisms; they are not silently included in this certification.
