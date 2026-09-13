# Council credit: first matched smoke comparison

This is an integration probe, not the planned calibration or held-out evaluation.
Implementation: `09d9150`; Quadro RTX 5000 with Max-Q Design, native Vulkan.
Seed 17, 32 cells per face for terrain/ecology, one geological epoch with one
ecological year, five starting civilizations, 30 history years. Both arms loaded
one identical founding checkpoint. Default startup systems were retained. The
only intervention was `--council-credit=false` versus `--council-credit=true`.
Generated checkpoints, JSON histories and logs remain in ignored
`output/credit-council-smoke/`.

| Measure | Baseline | Credit |
| --- | ---: | ---: |
| Starting population | 600 | 600 |
| Final aggregate population | 551.852862 | 551.946047 |
| Loan contracts | 0 | 10 |
| Total principal transferred | 0 | 8.69464348 |
| Total principal plus interest repaid | 0 | 0.000003815 |
| Defaulted contracts | 0 | 10 |
| Credit rounds | 0 | 21 |
| Money residual reported by existing validator | 3.10e-8 | 2.11e-7 |
| History command wall time | 9.85 s | 9.45 s |

Timings are single observations, not evidence of a speed improvement. Population
change is negligible and cannot establish a credit benefit. Lending occurred, but
all ten loans defaulted. Eight subsequent requests were rejected by the existing
credit exclusion; three had insufficient capacity. The issuance gate has not passed.

## Immediate cause and correction

The first borrower pledged month-60 taxes. Its preceding month-48 collection was
29.85846; underwriting deducted 12.05755 in operating costs. The actual month-60
collection was 50.33089, yet cash at month-61 servicing was only 0.00001526. The
problem was not absence of the promised gross tax collection. Annual road building
spent from the same treasury before the next Open, and its feasible commitments
were missing from credit's cost evidence.

The correction records council-specific road requests and actual payments at that
existing spending boundary. Tax evidence now deducts annual road requests as well
as town support; automatic underwriting additionally retains its monthly operating
forecast. Archives without road evidence must wait for a new collection instead
of assuming zero road cost. No payment ordering, tax rate or lending cap is changed
by this correction. This can correctly suppress lending rather than improve a
population headline. A repeat of the same checkpoint comparison is required.

The broader work remains: successful and failed commercial repayment, institutional
closure, restructuring, bounded issuance and all four comparison arms, longer seed
ensembles, mediator measurements and currency exchange after acceptance gates.

## Repeat with annual road commitments

The same founding checkpoint and 30-year credit command completed after the
correction. All 22 requests were rejected for insufficient capacity. There were
zero loans, zero defaults and no cash transfers. Final population was 551.852862;
serialized sites, events, people, cargo and council records exactly matched the
baseline. The existing monetary residual also matched, 3.10e-8. Command-reported
wall time was 9.46 seconds (single observation).

This is a corrected rejection case, not a successful credit economy. It shows why
raising loan caps or minting to rescue these loans would have obscured a real
obligation conflict. Broader receipt-backed commercial cases and explicit temporary
liquidity shortages still need testing. No issuance or currency-stage gate is met.

Validation for the correction: two tax evidence tests, ten active market tests
(two extended cases ignored), strict all-target Clippy, and the full GPU-backed
30-year history validation passed. The receipt forecast control specifically
rejects a loan whose annual road requirements absorb the expected tax income.
