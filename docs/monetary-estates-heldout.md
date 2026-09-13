# Held-out monetary comparison after estate integration

Status: all twelve 200-year runs are complete and passed native validation.
The multiple-currency decision gate remains unmet.

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

## Seed 409

| Arm | Ending population | Completed operator work | Reported food production | Terminal need-weighted hunger | Loans | Issued |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Baseline | 177.847 | 403.131 | 10,034,429 | 0.02832 | 0 | 0 |
| Credit | 177.847 | 403.131 | 10,034,429 | 0.02832 | 0 | 0 |
| Issuance | 115.272 | 490.844 | 9,449,558 | 0.03879 | 0 | 1,250 |
| Combined | 140.234 | 492.293 | 9,548,205 | 0.02424 | 1 | 1,250 |

Baseline and credit histories differ only in the credit subtree. Issuance has
adverse population, food and terminal hunger outcomes despite more completed
operator work. Combined improves access relative to issuance alone, but remains
below baseline population and reported food production.

Combined made one export-backed town loan at month 698, with principal
2.023681640625 and maturity 713. It paid 2.0235210217927633 principal and
0.26627390008223684 interest; the remaining 0.00016061883223672524 principal
was explicitly precision-settled. There was no default or ending debt.
This demonstrates a realized contract, not proof that a roughly two-unit loan
directly explains the later population difference. Immediate funded activity and
the divergence path still need examination before attributing those outcomes.

All four seed-409 runs passed native validation. Maximum absolute monetary
relative residual was `2.77e-7`, other managed residuals `4.71e-5`, and ecological
C/N/P residuals `1.62e-5`.

## Seed 1024

| Arm | Ending population | Completed operator work | Reported food production | Terminal need-weighted hunger | Loans | Issued |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Baseline | 98.371 | 18.185 | 5,543,195 | 0.03614 | 0 | 0 |
| Credit | 98.371 | 18.185 | 5,543,195 | 0.03614 | 0 | 0 |
| Issuance | 109.257 | 18.826 | 5,630,954 | 0.02341 | 0 | 1,250 |
| Combined | 109.257 | 18.826 | 5,630,954 | 0.02341 | 0 | 1,250 |

Both matched pairs differ only in the credit subtree. No realized credit effect,
defaults or outstanding loans occurred. Issuance improves the listed measures;
cumulative town support increases from 18,177.40 to 20,222.11.

All four runs passed native validation. Maximum absolute monetary relative
residual was `1.36e-7`, other managed residuals `3.34e-5`, and ecological
C/N/P residuals `1.54e-5`.

## Decision and next evidence

Credit-only is a null result in all three held-out seeds: underwriting produces
no contracts, and whole-history comparisons show no changes outside credit
records. Combined produces only one loan across the three seeds. This ensemble
therefore provides little evidence about credit helping ordinary timing gaps,
even though controlled fixtures establish transfers and funded GPU work.

Issuance outcomes are mixed across seeds and metrics. Keep both pilots opt-in and
do not open the multiple-currency gate on this evidence. Next inspect rejected
requests and immediate work/payment timing in constrained fixtures, and trace
seed 409's one accepted export loan before claiming a causal benefit. Estate
succession, post-default recovery and other design checklist items remain separate
unfinished work. Do not raise issuance caps simply to improve a terminal metric.

## Interpretation and reproduction limits

Reported food is the cumulative food-equivalent ledger, not a wheat harvest.
Operator work excludes other livelihoods. Hunger measures the final month's
household hunger weighted by need, not cumulative famine. All populations remain
well below the initial 600, so improved relative outcomes do not imply demographic
stability. The earlier seed 17/81 runs produced mixed issuance outcomes; two
positive held-out seeds do not justify enabling issuance by default or opening
the multiple-currency gate.

Run artifacts remain under `output/monetary-estates-heldout-founding` and
`output/monetary-estates-heldout`. The runner records executable/checkpoint hashes,
settings and raw outcomes. Shader validation ran concurrently for part of this
ensemble; elapsed times are not controlled performance benchmarks. Constants edits
cannot affect the copied executable.
