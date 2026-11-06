# Accepted loan repayment schedules

`LoanOffer.repayment` now chooses `RepaymentSchedule::Installments` (the default)
or `AtMaturity`. Origination copies that choice into the existing accepted `Loan`.
Later changes to an offer cannot change the accepted schedule. Both direct
advances and financed property purchases use the same book, collection executor,
forecast and financial reporting.

This removes one timing mismatch with main-world credit. The
[source claim inventory](HOST-CLAIM-INVENTORY.md) still rejects liquid imports
that would omit debts, receivables or default history. This is a contract capability,
not permission to translate or execute arbitrary source contracts.

## Monthly boundary and accounting

An active loan accrues simple interest on remaining principal once per month,
starting the month after origination. Fractional interest carries in the existing
integer remainder; unpaid interest does not compound. `Installments` retains
cumulative principal installments and monthly interest collection. `AtMaturity`
defers collection of both principal and accrued interest until
`opened + term_months`, then requests all remaining debt. Accepted maturity dates
must fit the month counter.

For example, a 100-unit advance opened in month 1 with a three-month term and
100 basis points per month has this fully funded maturity path:

| Due boundary | Interest accrued this month | Collection | Remaining principal | Remaining interest |
| --- | ---: | ---: | ---: | ---: |
| 2 | 1 | 0 | 100 | 1 |
| 3 | 1 | 0 | 100 | 2 |
| 4 | 1 | 103 | 0 | 0 |

Interest receivable/payable and income/expense appear when accrued. Deferring
cash collection does not hide the liability or book income twice at repayment.
Read-only projections advance a private loan copy through the zero-payment months
and report the maturity demand without changing live state or receipts.

The existing experimental `Due` phase collects opening claims before acquisition
and production. It is part of the host driver's Open stage. Due collects requests,
applies the configured ranked or proportional policy and existing stock protection,
then commits actual payments against finite resources. A future maturity claim
requests zero and consumes no collection allowance. Accrual and payment events
remain attached to the committed Due boundary and are visible to subsequent work.
Neither the experimental phase sequence nor the main-world coordinator changes.
The main engine continues servicing credit in Open after arrivals and economy
preparation, before Reserve.

At maturity, payment remains interest-first. A shortfall starts arrears then, so
the collateral grace clock does not begin during the earlier zero-payment months.
Existing enforcement and estate acceleration still apply; a repayment schedule
does not override a stay or change recovery authority. Unsecured arrears retain
the experiment's existing behavior rather than acquiring the main engine's
automatic default/write-off rules.

## Compatibility and scope

All existing offer constructors and fixtures explicitly select the default
installment schedule. Agency-generated working-capital terms and guarantee
recourse remain installment-only; their validators reject unsupported schedule
substitution. The new choice is available on explicit loan offers. It adds a field
to the Rust terms API, not a serialized Simulation/Audit checkpoint format.

The main-engine comparison uses its actual loan origination and monthly servicing
methods for the example above. It sets the native collection cash share to 1,
uses one source money unit per experimental tick, and matches a 0.12 annual simple
rate to 100 monthly basis points. Principal, interest and both parties' cash agree
after every servicing month. The main engine's default cash share remains 0.25.
This controlled fully funded case does not establish scarcity-policy parity,
general rate precision, calendar rebasing, assignments, defaults or later recovery.
Those remain requirements for a source-contract importer.

## Verification

The eleven schedule tests cover audited interest before maturity, immutable
projections and accepted terms, fractional carry, one-month terms, finite
interest-first partial repayment, a future claim sharing the proportional pool,
collateral grace, explicit acceleration, invalid maturity dates and the controlled
main-engine comparison. They also open an actual estate before maturity through
another overdue claim and verify that a guarantee waits until maturity and pays
only the unpaid residual. Reference, CubeCL CPU, monthly, batched and cloned
continuations agree on state and full receipts. Clone continuation is an in-memory
check; durable financial restart remains a separate integration gate.

On 2026-11-06, all 62 focused tests passed: 11 schedule, 9 credit, 18 lending,
22 host integration, and one each for working-capital consent and repeated
guarantee recourse. Strict release Clippy passed for all targets with the host
feature and warnings denied. All-target release checking without the host feature,
formatting, diff checks and the repository artifact check passed. Sub-agent review
found no correctness issues; its suggested estate and guarantee controls were
added and passed.

These are small controlled fixtures, not a generated-world rollout or evidence
that the new repayment choice improves economic balance. Generated build and test
logs remain under ignored `output/economics/maturity-*.log`.

```sh
cargo +1.92.0 test --locked --release -j8 --features ancient-world-host \
  --test loan_schedules --test credit --test lending
cargo +1.92.0 test --locked --release -j8 --features ancient-world-host \
  --test host_ancient_world
cargo +1.92.0 test --locked --release -j8 --features ancient-world-host \
  --test working_capital publication_admission_activation_and_authority_are_separate
cargo +1.92.0 test --locked --release -j8 --features ancient-world-host \
  --lib repeated_guarantee_calls_share_dated_claims_with_planning
cargo +1.92.0 clippy --locked --release -j8 --features ancient-world-host \
  --all-targets -- -D warnings
cargo +1.92.0 check --locked --release -j8 --no-default-features --all-targets
cargo +1.92.0 fmt --check
```
