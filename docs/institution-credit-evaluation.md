# Institutional lender comparison

Seed 1024, five initial civilizations, 32 terrain/ecology cells per face, 200
history years, one saved founding checkpoint. Six arms ran: baseline, credit,
issuance, combined, credit with institutional lenders, and combined with
institutional lenders. The latter two are the only arms enabling the new offers.
All use the same delivery-paid export policy.

The copied executable contains the initial institution-offer implementation
committed as `0b7d449`. It predates the subsequent explicit building-access guard;
that guard has separate controlled verification. It also predates the parallel
managed-land constants extraction. Raw histories, commands, executable checksum
and checkpoint checksum are retained locally under ignored
`output/monetary-institution-lenders/`. Concurrent compilation and hardware tests
mean elapsed times are not performance comparisons.

| Arm | Final population | Loan contracts | Currency issued |
| --- | ---: | ---: | ---: |
| Baseline | 98.3713 | 0 | 0 |
| Credit | 98.3713 | 0 | 0 |
| Issuance | 109.256615 | 0 | 1,250 |
| Combined | 109.256615 | 0 | 1,250 |
| Credit + institutional lenders | 98.3713 | 0 | 0 |
| Combined + institutional lenders | 109.256615 | 0 | 1,250 |

The two new arms are exactly equal to their respective controls after normalizing
only the institutional-lender policy flag. This comparison includes full serialized
histories, not just the rounded population summaries. All six runs passed native
validation. The largest absolute terminal monetary residual is 1.35116e-7; this
is not a maximum over monthly residuals.

## Where the extension stops

There are zero institutional offers in recorded underwriting rounds and zero
institutional loans. The council construction counts are unchanged: credit has
one submitted council-month and 2,432 gaps without an eligible contacted lender;
combined has 53 submitted council-months and 2,124 such gaps. Their respective
six and 58 total requests include five export requests and all fail capacity.
An institution might have offered cash in a month with no submitted request, so
absence from recorded rounds does not prove absence of every potential offer.

At the endpoint, each new arm has two active institutions. The largest treasury
among all institutions is 2.43213 (credit) or 2.50018 (combined), far below the
initial inherited council reserve floor of 100. Endpoint stocks do not establish
the entire trajectory, but they identify an important scaling question: a council
cash floor may be inappropriate for an institution whose annual operating quote
is much smaller. This is a reason for an independently controlled institutional
reserve-policy comparison, not evidence that all institutional cash is free.

The controlled GPU fixture proves that a sufficiently funded institution can make
and service a loan without creating cash. This ensemble does **not** demonstrate
useful institutional lending, completed work financed by it, or safe default rates
for institutional lenders. The zero-loan sample provides no such rate estimate.

## Next evidence needed

Compare a reserve derived from institutional operating costs against the inherited
fixed floor, keeping borrower coverage checks intact. Report both issued loans and
subsequent upkeep, repairs, closures, arrears and actual council service completion.
Include a loss case and a constrained building budget; do not select a smaller floor
only because it generates more contracts. Operator service-order financing remains
a distinct unfinished connection. This null result does not pass the Stage 2 gate.
