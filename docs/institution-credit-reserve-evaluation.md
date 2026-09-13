# Institutional reserve comparison: seed 1024

Eight matched 200-year arms completed successfully on the Quadro RTX 5000 with
Max-Q Design, using the same debt-free founding checkpoint, terrain/ecology edge
32, five civilizations, and delivery-paid export contracts. There was no crop-yield
override. This tests a lending reserve policy, not a different harvest or tax policy.

The copied executable contained the institutional operating-reserve change later
committed in `c943f45`. Its build metadata records base `02cecdd` plus the four
modified implementation/reporting files; it predates the subsequent constants
extractions. The executable SHA-256 was
`5223e63a0da8600c43dc2264fa51313ea9c2ed4102895b9a46ba51bd04d89794`.
Generated artifacts remain under ignored `output/monetary-institution-reserves`.

Reproduction with a corresponding founding checkpoint:

```sh
python3 scripts/monetary_experiment.py \
  --checkpoint 1024=output/monetary-estates-heldout-founding/1024.world \
  --years 200 --compare-institution-reserves \
  --output output/monetary-institution-reserves
```

## Results

| Arm | End population | Requests | Loans | Money issued |
| --- | ---: | ---: | ---: | ---: |
| Baseline | 98.3713 | 0 | 0 | 0 |
| Credit | 98.3713 | 6 | 0 | 0 |
| Issuance | 109.256615 | 0 | 0 | 1250 |
| Combined | 109.256615 | 58 | 0 | 1250 |
| Credit + institutional lenders, fixed floor | 98.3713 | 6 | 0 | 0 |
| Combined + institutional lenders, fixed floor | 109.256615 | 58 | 0 | 1250 |
| Credit + institutional operating reserves | 98.3713 | 35 | 0 | 0 |
| Combined + institutional operating reserves | 109.256615 | 58 | 0 | 1250 |

Every submitted request in the lending arms had zero net repayment-source
capacity. The operating-reserve credit arm recorded institutional offers in 22
underwriting rounds, and its council submissions rose from 1 to 30 (the remaining
five requests were exports). The combined operating-reserve arm recorded no
institutional offers in its underwriting rounds. These records do not cover offers
in months with no submitted requests, so they cannot establish that institutions
never offered in those months.

An illustrative actual request at month 25: institution 4 held 21.58796 money,
protected 6.78663 in annual operating costs, and offered 3.70033. Council 4 requested
0.93292, but its tax forecast was only 1.46497 against 1077.38410 in operating
costs. The rejection came from the repayment source, not unavailable lender cash.
This is evidence against solving this case by lowering lender reserves again.

## Effects and accounting

Comparing each operating-reserve history export with its ordinary credit/combined
control, the only changed top-level record was `credit`. All other serialized
history state matched exactly, including institutions, people and production.
The credit diagnostics changed, but there were no actual financing transfers.

Non-issuance arms ended with two active institutions, cumulative institutional
expenses 3007.82646, recorded upkeep work 439.28665 and institutional cash 4.36322.
Issuance arms also had two active institutions: expenses 3018.29443, upkeep work
435.37009 and cash 4.50043. All ten institution records had capacity records.
These are cumulative/endpoint measures, not evidence of uninterrupted service or
monthly minimum reserves. Inactivity is not automatically dissolution.

All eight native commands exited successfully. The maximum absolute terminal
money residual was 1.35116e-7, taken from the money component of the managed
ledger (index 4), not its water component. This is an endpoint relative residual,
not a maximum over all monthly boundaries. With zero loans, this ensemble cannot
establish repayment/default behavior or whether lending harms institutional upkeep.
The separate controlled loan fixture covers actual transfers and continuation.

## Next decision

Keep the policy opt-in and keep Stage 2 currency exchange gated. The reserve
change makes a previously excluded class of lender available, but does not yet
produce useful credit. Inspect the composition of council repayment forecasts:
annual town support and road requests, plus annualized administration and recent
relief. Distinguish recurring needs from temporary demand before changing any
forecast; do not erase costs merely to obtain loans. Then demonstrate an automatic
bridge funding actual work in a controlled solvent case, with an otherwise matched
insolvent rejection. Operator service-order financing remains separate unfinished
work requiring an actual funded order and payment milestone.
