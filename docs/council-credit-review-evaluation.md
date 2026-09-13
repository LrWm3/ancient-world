# Council credit opportunity comparison

A repeat of seed 1024 uses its saved founding checkpoint, 32 terrain/ecology cells
per face, five initial civilizations, and 200 history years. The runner compares
baseline, credit, issuance and combined arms with the same delivery-paid export
policy. It uses a copied executable containing the council review changes committed
as `0899e0a`; raw metadata, histories and logs are ignored under
`output/monetary-council-reviews/`. Concurrent compilation/tests make elapsed times
unsuitable as a performance comparison.

## Completed four-arm comparison

| Measure | Baseline | Credit only | Issuance only | Combined |
| --- | ---: | ---: | ---: | ---: |
| Final population | 98.3713 | 98.3713 | 109.256615 | 109.256615 |
| Loan contracts | 0 | 0 | 0 | 0 |
| Currency issued | 0 | 0 | 1,250 | 1,250 |
| Council-months reviewed | 0 (disabled) | 12,000 | 0 (disabled) | 12,000 |
| No cash gap | — | 9,231 | — | 9,587 |
| Missing usable tax evidence | — | 336 | — | 236 |
| No eligible contacted lender | — | 2,432 | — | 2,124 |
| Submitted council decisions | — | 1 | — | 53 |

The six rejected requests in the credit arm include five export requests and one
council request. All failed underwriting for capacity. A council review is not a
loan request: a single decision can produce requests to multiple lenders.

Of 2,769 council-month cash gaps, 2,432 never reach underwriting because there is
no contacted council with an eligible surplus offer. That combines financial and
route restrictions; it does not prove roads are missing. Another 336 lack usable
tax evidence. Raising underwriting caps cannot address these skipped requests.
Nor does this establish that a different lender would safely fund them: the
skipped cases have not passed net-receipt coverage checks.

All pre-existing serialized values in all four arms match the corresponding
previous held-out seed-1024 exports recursively. New fields account for the
schema differences, including estate/account additions introduced since that run.
This supports unchanged behavior in these controls, not equality across hardware
or every seed. All four runs passed the native history validator. The maximum
absolute terminal money residual was 1.35116e-7; this is an endpoint measurement,
not the maximum monthly residual.

In the combined arm, 53 council decisions produce 53 tax requests; together with
five export requests, all 58 are rejected for capacity. Issuance makes more requests
possible but does not make any loan acceptable under the current rules. Workshop
completion is 18.1846 in baseline/credit versus 18.8257 in issuance/combined; terminal
need-weighted hunger is 0.03614 versus 0.02341. These reproduce the prior result,
not a new independent seed or evidence of credit benefit.

The previous Stage 2 no-go conclusion remains in force. The next lending review
should examine the availability and ownership of voluntary offers, then the
realized receipts and operating costs of potentially feasible projects. It must
separate lender cash availability from contact restrictions, and temporary relief
from recurring services. These counts alone do not justify lowering reserves,
ignoring costs or assuming a skipped borrower is solvent.
