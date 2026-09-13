# Shared currency: two-century four-arm comparison

Eight runs completed: seeds 17 and 81, baseline/credit/issuance/combined, each for
200 years. Settings and checkpoints match the [30-year experiment](shared-issuance-smoke.md):
32/32 terrain/ecology, five civilizations, delivery-paid exports in every arm,
default living history. The executable is the fixed `0aa337c` build, before the
later term-aware underwriting correction and restructuring archive checks.
This is game-balance evidence, not historical calibration.

## Population and lending

| Seed | Baseline population | Credit population | Issuance population | Combined population |
| --- | ---: | ---: | ---: | ---: |
| 17 | 172.869 | 172.869 | 237.735 | 237.735 |
| 81 | 266.292 | 266.292 | 309.708 | 309.708 |

Issuance creates 1,250 units in each enabled world, entirely within the first
ten-year authorization. It creates nothing afterward. All arms make zero loans
and therefore have no repayments or defaults to evaluate. Credit-enabled arms
are not identical archives because they contain policy/decision records; compare
actual world fields as well as summaries. For each seed, baseline versus credit
and issuance versus combined differ only in the complete History `credit` field;
all other serialized fields match.

| Seed / credit arm | Tax requests | Export requests |
| --- | ---: | ---: |
| 17 credit | 22 | 0 |
| 17 combined | 6 | 0 |
| 81 credit | 1,016 | 4 |
| 81 combined | 759 | 4 |

These are requests, including repeated monthly attempts, not distinct borrowers.
The recorded decisions are `NoCapacity`. Issuance reduces tax-credit requests but
does not turn them into loans. The prior [commercial comparison](credit-commercial-balance.md)
examines seed 81's small receivable versus operating commitments. Counting failed
requests alone does not prove underwriting should be relaxed: both the operating
forecast and actual available repayment receipts need controlled review.

## Work, food and support

| Seed / arm | Cumulative operator work | Reported food production | Final need-weighted hunger | Cumulative council town support | Ending council cash |
| --- | ---: | ---: | ---: | ---: | ---: |
| 17 baseline | 107.36 | 13,863,726 | 0.0374 | 36,532.22 | 5,369.19 |
| 17 issuance | 125.57 | 14,896,522 | 0.0280 | 42,215.88 | 3,533.65 |
| 81 baseline | 721.42 | 16,637,439 | 0.0427 | 47,320.83 | 338.30 |
| 81 issuance | 301.55 | 15,828,234 | 0.0273 | 42,378.07 | 1,446.09 |

Definitions are unchanged from the short experiment: operator work excludes other
work; food uses the existing food-equivalent production ledger; hunger describes
the final month's household needs; town support is the annual council-to-town
transfer ledger, not all relief. Cash stocks cannot identify particular issued
coins. No result here establishes that a specific household survived because of
one grant: long trajectories diverge through many interacting decisions.

Population and terminal food access improve in both issuance cases, reversing the
short-run population sign in seed 17. However, seed 81's operator work and cumulative
food production decrease, and council support does not rise consistently. The
initial cash intervention has persistent consequences, but this is not a uniform
productivity improvement or proof that liquidity explains population decline.

## Verification, limits and next tests

All eight native runs completed validation. Monetary residual magnitude remained
below 4.11e-7 in the baseline/issuance cases; other managed relative residuals
reached 5.63e-5. The executable's issuance caps do not scale up with its enlarged
cash supply, so these cases do not exhibit repeated rescue or runaway issuance.
There is no debt-growth test in a world with zero lending.

The reusable runner was invoked with the two documented checkpoint paths,
`--years 200`, output directory `output/monetary-four-arm-century`, and
`--binary output/shared-issuance-smoke/ancient-world`. It copied the executable
and retained checksums, histories and logs under ignored `output/`. Runs took
roughly 70–88 seconds, with overlapping compilation and some GPU verification;
these timings are not performance comparisons.

The Stage 2 benefit gate remains unproven. Next evidence must include a controlled
cash-timing shortage with affordable repayment, failed tax bases, lost cargo,
held-out seeds and the specified longer runs. A viable loan should improve a
measured immediate service/work outcome without displacing essential spending;
a failed repayment should produce a bounded loss rather than refinancing forever.
Negotiated restructuring and closure handling remain implementation work. Do not
turn on issuance by default or infer an FX design is validated from these two seeds.

## Longer seed-81 follow-up (500 years)

The same fixed `0aa337c` executable completed a four-arm, 500-year follow-up from
the same seed-81 checkpoint. This is a longer tuning-seed run, not held-out evidence.

| Arm | Ending population | Loans / defaults | Issued | Final need-weighted hunger | Cumulative operator work |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | 180.167 | 0 / 0 | 0 | 0.0293 | 1,159.46 |
| Credit | 127.052 | 2 / 2 | 0 | 0.0229 | 1,126.24 |
| Issuance | 182.070 | 0 / 0 | 1,250 | 0.0194 | 592.76 |
| Combined | 182.070 | 0 / 0 | 1,250 | 0.0194 | 592.76 |

All native runs completed validation. The credit arm's two defaults are the
[numerical-residue cases](credit-precision-residuals.md), not two demonstrated
failures to afford repayment. Its lower population cannot be attributed to that
classification alone without a controlled rerun. Issuance's population advantage
is much smaller than at year 200, while operator work remains markedly lower.
Terminal hunger alone is not a survival or total-welfare measure: it omits the
needs of people no longer present. These results strengthen the case for looking
at work, population and food access together rather than choosing one endpoint.

The precision correction and new delayed-export negotiation were not present in
this executable. Raw follow-up artifacts are ignored under
`output/monetary-four-arm-five-century/`; the same runner used `--years 500` and
only the seed-81 checkpoint. These results do not pass the next-stage gate.
