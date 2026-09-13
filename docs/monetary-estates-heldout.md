# Held-out monetary comparison after estate integration

Status: seed 256 is complete; seeds 409 and 1024 are running. Do not interpret the
partial results as a completed experiment gate.

The copied executable was built from `547aaba`. Fresh founding checkpoints use
terrain/ecology edges 32/32, one geological epoch, one ecological year per interval,
five civilizations and 600 initial people. Each of the four arms advances 200
years with default living history. Hardware is the Quadro RTX 5000 Max-Q.

All arms enable delivery-paid exports. Baseline disables both credit pilots and
issuance; credit enables the council/commercial pilots; issuance enables only the
dated shared-currency policy; combined enables both. Issuance remains capped at
1,250 total units across the five councils. These held-out seeds were not used to
select the existing credit/issuance parameters. They are toy-balance comparisons,
not historical calibration.

## Seed 256

| Arm | Ending population | Completed operator work | Reported food production | Terminal need-weighted hunger | Loans | Issued |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Baseline | 82.475 | 19.133 | 8,225,840 | 0.03584 | 0 | 0 |
| Credit | 82.475 | 19.133 | 8,225,840 | 0.03584 | 0 | 0 |
| Issuance | 107.274 | 19.318 | 8,792,761 | 0.02896 | 0 | 1,250 |
| Combined | 107.274 | 19.318 | 8,792,761 | 0.02896 | 0 | 1,250 |

Whole exported histories differ only in the credit subtree for baseline versus
credit and issuance versus combined. Thus this seed shows **no realized credit
effect**. There were no defaults, debt balances or precision settlements.

Issuance improves these terminal population/access measures, with a small increase
in operator work and more reported food production. Cumulative council-to-town
support rises from 24,924.90 to 29,933.41. Ending council cash falls from 103.75 to
71.95; town cash rises from 2,945.93 to 3,874.20 and household cash from 46,948.16 to
47,299.68. Those stock differences do not trace the ancestry of issued currency
or establish that issuance directly caused every downstream change.

All four native runs passed validation. Maximum absolute monetary relative
residual was `1.72e-7`; other managed residuals were at most `1.52e-5` and ecological
C/N/P residuals at most `1.53e-5`. Conservation does not establish useful balance.
None of these arms exercised a realized loan, so this result does not validate
estate behavior beyond the separate controlled fixtures.

## Interpretation and reproduction limits

Reported food is the cumulative food-equivalent ledger, not a wheat harvest.
Operator work excludes other livelihoods. Hunger measures the final month's
household hunger weighted by need, not cumulative famine. All populations remain
well below the initial 600, so improved relative outcomes do not imply demographic
stability. The earlier seed 17/81 runs produced mixed issuance outcomes; one
positive held-out seed does not justify enabling issuance by default or opening
the multiple-currency gate.

Run artifacts remain under `output/monetary-estates-heldout-founding` and
`output/monetary-estates-heldout`. The runner records executable/checkpoint hashes,
settings and raw outcomes. Shader validation ran concurrently for part of this
ensemble; elapsed times are not controlled performance benchmarks. Constants edits
cannot affect the copied executable.
