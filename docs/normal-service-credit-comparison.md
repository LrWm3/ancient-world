# Ordinary service fees: partial funding does not justify full-order receipts

## Reproduction and scope

Run `cargo test --lib normal_service_fee_credit_comparison -- --ignored --nocapture`.
This hardware fixture uses seeds 17, 81 and 256, terrain resolution 32, ecology
resolution 16, five starting civilizations and the existing installed-workshop
fixture. It restricts recipes to tools and declares extra workshop/input inventory;
these are controlled production cases, not untouched natural seed economies.

At month 3 the fixture returns the selected firm's remaining opening capital to
its household, preserving cash and equity accounting. It moves existing town cash
to fund a next-month service order and leave a potential lender surplus. It retains
the normal quoted fee of 45 per worker-month. Orders request 0.25, 1 or 2
worker-months. Each opening is saved and loaded into matched service-credit on/off
arms, then advanced for twelve actual GPU monthly production steps. Commercial
credit is enabled in both; only operator service underwriting differs. There is
no issuance, new rescue policy, fee increase or free production.

Source baseline: `fe70d69`, plus the comparison fixture in `src/enterprises.rs`.
The test passed all 18 branches. Raw output stays in ignored
`output/normal-service-credit.log`.

## Results

The loan and contracted-payment results were identical across the three seeds:

| Requested work | Conservative expected fee | Estimated operating costs | Principal approved | Fee actually earned | Fee refunded |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0.25 | 10.6875 | 11.88 | 0 | 0 | 11.25 |
| 1 | 42.75 | 38.88 | 1.829102 | 0 | 45 |
| 2 | 85.5 | 74.88 | 5.019348 | 2.674255 | 87.325745 |

All no-credit controls issued no loans and earned no service fees. Every selected
firm closed at month 6, in both arms. In the two-worker-month case, cumulative
firm work reached 0.09517–0.09674 worker-months with credit versus zero without;
that total includes later ordinary work, not just the contracted month's work.
The other order sizes completed no work in either arm. Reported wage totals also
include pre-intervention fixture payroll and must not be interpreted as wages
financed by these loans.

Monthly credit and enterprise validation passed. The maximum absolute change in
the canonical relative money residual was 5.50e-9. Two-unit orders retained about
1.12e-7 cash in escrow because the town's f32 account could not represent that last
refund; the ledger retains it rather than dropping or duplicating it. This report
does not establish eventual loan repayment/default outcomes or general population
benefits.

## Implication and next correction

Small orders do not cover payroll and rent even before debt service, so rejecting
them is expected. Larger orders expose a more serious underwriting mismatch:
the forecast uses full-order work, but source-coverage limits approve only a
fraction of the operating gap. Rent is paid before wages. In the one-unit case,
the approved principal is below the 2.88 rent bill and cannot fund any production.
In the two-unit case, some work is possible, but the actual fee is far below the
full-order forecast.

The next correction should make attainable work and receipts conditional on the
approved grant, including fixed rent and existing cash. Options include an
explicit minimum useful financing requirement or a bounded second feasibility
check after joint allocation. Rejected/unusable grants should leave lender cash
available, and existing source/exposure limits must still hold. Do not solve this
by increasing fees, dropping real rent, or relaxing the repayment gate.

Automatic procurement and normal-economy long comparisons remain unfinished.
This experiment provides a failing economic case to improve, despite passing
accounting checks. Stage 2 remains no-go.

## Follow-up: grant-dependent feasibility

The [funding check](service-credit-funding-check.md) now conditions receipts on
opening cash plus the proposed grant. Rerunning all eighteen branches rejects
the former one- and two-unit loans before any transfer. All arms now have zero
new loans and zero earned order fees, with full escrow refunds and no residual
order escrow. Selected firms still close at month 6. The maximum monthly change
in the relative money residual is 6.11e-9.

The separate valuable-contract GPU fixture still receives credit, funds and
completes more work than its no-credit control, and matches saved continuation.
An analytical case also accepts a profitable *partial* grant. The correction
therefore addresses the unsupported forecast, rather than requiring every loan
to fund the entire request. These fixtures do not demonstrate a broad normal-fee
benefit; automatic procurement and sustained balance evaluation remain pending.

Corrected raw output: ignored `output/normal-service-credit-corrected.log`.
