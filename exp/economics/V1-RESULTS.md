# Economics v1 verification results

Candidate: `a1b99fa6707667ae0dc9ceec52b6c8e01f6ff88d` (2026-09-30).
Status: **all v1 release gates passed; the bounded checklist is complete**.

This certifies the bounded stand-alone experiment described in
[V1-RELEASE](V1-RELEASE.md), not the entire long-term roadmap. The candidate adds
household-aware explicit offer acceptance, freezes the scenario set and supplies
one verification entry point. Separate persons, household mandates and financial
claims remain explicit.

## Reproduce

From the repository root, in a clean checkout:

```sh
python3 exp/economics/scripts/check_v1.py
```

The [scenario manifest](V1-SCENARIOS.md) fixes actors, resource endowments, supplied
consent, policies, horizons and controls. The runner uses Rust 1.92.0, locked
dependencies and CubeCL CPU/reference execution. Source/catalog revision stays
constant across every gate. Generated settings, logs and detailed observations
are local under
`output/economics/v1-a1b99fa6-20260930T203301684101Z/`.

## Gate results

| Gate | Exit | Seconds | Passed / failed / ignored |
| --- | --- | --- | --- |
| S1-offers | 0 | 3.00 | 5 / 0 / 0 |
| S1-shared-payment | 0 | 2.00 | 1 / 0 / 0 |
| S2-continuing | 0 | 309.03 | 1 / 0 / 0 |
| S2-no-support | 0 | 6.00 | 1 / 0 / 0 |
| S2-interruption | 0 | 10.00 | 1 / 0 / 0 |
| S2-observer | 0 | 1.00 | 1 / 0 / 0 |
| S3-farm-finance | 0 | 42.00 | 1 / 0 / 0 |
| S3-buffer | 0 | 1.00 | 1 / 0 / 0 |
| S4-mint-finance | 0 | 2.00 | 16 / 0 / 0 |
| S5-mortgage | 0 | 8.00 | 5 / 0 / 0 |
| S5-native | 0 | 7.00 | 7 / 0 / 0 |
| S5-crop-recovery | 0 | 3.00 | 1 / 0 / 0 |
| S5-claim-budget | 0 | 3.00 | 1 / 0 / 0 |
| S5-exit | 0 | 3.00 | 18 / 0 / 0 |
| S5-observer | 0 | 2.01 | 1 / 0 / 0 |
| S6-population | 0 | 697.06 | 1 / 0 / 0 |
| full-suite | 0 | 1821.20 | 1033 / 0 / 1 |
| clippy | 0 | 1.00 | 0 / 0 / 0 |
| format | 0 | 2.00 | 0 / 0 / 0 |
| artifacts | 0 | 1.00 | 0 / 0 / 0 |
| diff | 0 | 1.00 | 0 / 0 / 0 |

Counts overlap: the named scenarios also run in the full suite. The ignored
population test is executed explicitly. No test totals should be added together
as a count of unique tests.

## Economic outcomes

| Family | Observed outcome | Failure control / limitation |
| --- | --- | --- |
| S1, 24 months | Four crops; 16 grain pooled exactly once; 48 member-month food and warmth requirements met; 4 grain land dues paid, none unpaid. 44 wood-collection completions. CPU, reference and post-acceptance continuation agree. | One plot grants one package; loser can collect wood. Missing permission, seed, labor/storage and stale/forged packages publish nothing. Both adults unavailable in month 2 abort both crops and lose two seed. |
| S2, 120 months | All 240 household food requirements met; 240 grain and 119 fuel traded; household ends with 60 coins. Coin supply conserved and eight separate statements reconcile. Continuation at month 60 agrees with uninterrupted execution. | No-support control misses two food units by month 27. Market interruption records six food and four warmth deficit units over 25 months across all persons, then household feeding recovers from month 15. The healthy run retains one outside-producer startup warmth deficit. |
| S3, 24 months | Two completed crops in the funded baseline; mortgage repaid, annual rent of 2 grain paid and both 2-grain prepaid deliveries performed. Nutrition deficit zero. Individual and household variants reconcile and resume before delivery. | Removing finite buyers preserves repossession. The fixed six-month stock-buffer control pays the same financial claims but misses one meal; this remains a failing economic policy, not a release-test failure. |
| S4, months 12–17 | Incremental provision completes four mint batches, issues 40 coins, repays the loan, delivers the forward and pays 2 rent; actual inputs/hours remain bounded. Five separate statements reconcile. | It still records four food-deficit units. Full-buffer provision issues zero, leaves 2 rent unpaid and records nine food-deficit units despite repaid loan/delivery. No claim of universal nutritional viability. |
| S5, short controls | Actual sale proceeds, custody, guarantee timing, claim assignment and losses reconcile through continuation and household exit. Face-6 mortgage control distributes 4 sale proceeds; early guarantee can add 2. Prices 3/7 retain discount/premium cost. | Unfunded sales leave claims/title intact; native repayment needs storage. The property control sells at 4/9 or remains unsold: principal ends 2/0/6, and excess 3 goes to borrower only for the funded price-9 sale. Maintained crop completes; neglected crop aborts at work. Recovery observer separately asserts 8 distributed and 12 written off. |
| S6, 32 persons / eight households / 13 months | All 416 person-month food and warmth requirements met; 13 crops completed; annual dues paid with none outstanding; 1,300 issuance reporting ticks; 41 separate statements reconcile. Runtime 695.92 seconds (697.06 including command overhead). | 64 shelter-deficit units remain. Work receipts requested/allocated/completed are 18,762/5,350/5,350; unused labor capacity is 4,250 ticks. Population regression, not an optimality or performance guarantee. |

The test-only reporter reads existing receipts, state and statements. S1 requested,
allocated and completed work-receipt units are 118/88/88; S2 1,800/486/486. These
are request-native units, not hours. Unused labor capacity is 152 in S1 and 954 in
S2, counting period expirations and the final remaining capacity. S4 has 10 unused
hours under incremental provision versus 24 under full-buffer provision. Town
volumes are separate from posted/mint financial drivers. Empty town-book data
must not be interpreted as zero transactions across the whole model.

## Completion and retained boundaries

V1-01 has [acceptance evidence](V1-ACCEPTANCE.md). V1-02 freezes the
[scenario manifest](V1-SCENARIOS.md). V1-03/V1-04 assert economic outcomes, real
failure consequences, separate accounting, CPU/reference comparisons and relevant
continuation boundaries. V1-05 supplies the runner and this human-readable result;
V1-06 passed every gate above on the recorded candidate. The subsequent changes
that publish this result are documentation-only.

The accepted exclusions remain: general household prerequisite/consequence-aware
search, autonomous underwriting and state government, cooperative household credit,
new institutional types, personal self-directed policy changes, children/death
estates, FX/consolidation/wider instruments, GPU certification and durable full-state
restart. No new work is added to complete a Fibonacci batch. Future improvements
belong in a separately bounded scope.
