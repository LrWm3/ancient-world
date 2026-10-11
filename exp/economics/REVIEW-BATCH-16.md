# Sixteen follow-up review iterations

Start: `80b513b`, following [the ten-iteration batch](REVIEW-BATCH-10.md).
Each iteration receives independent review, focused verification and a separate
source-only commit. Defaults and individual self-directed policy deferral remain
unchanged. Generated outputs stay under ignored `output/`.

## 1. Compose discovery with posted household hiring

Reuse the existing optional household employment adapter instead of introducing a
new wage or production model. Discovery accepts posted, worker-consented household
terms with land discovery disabled. Preaccepted employment and discovered-land
composition remain excluded at this boundary. Existing useful-work comparisons,
finite opening cash, charter budgets, real hours and normal payroll still apply.

Independent review found no blocker. All 18 household-hiring tests pass; the new
six-control test compares Reference/CPU, financial audit and continuation after
every phase. Useful work hires two hours; absent work, money or capacity and
unprofitable terms do not.

## 2. Share validated acceptance staging with hiring forecasts

Settlement and hiring previews use one membership/access staging helper. Newly
accepted productive rights are visible to the subsequent work forecast. Employment
eligibility still uses opening permissions; new citizenship cannot authorize a
same-window labor sale. Three audited Reference/CPU controls pass, including denied
membership and next-month labor eligibility. Independent review found no blocker.
Discovered land remains excluded: its prerequisite reservations need an explicit
handoff before optional employment can safely compose with them.

## 3. Optional worker-side opportunity-cost protection

Added a per-person, opt-in bounded assessment for posted jobs, sharing the real
post-Acquire preview with the employer. It protects survival, needs and failed
processes without hypothetical wage purchases. Both sides must approve the exact
final hours, including after discrete trimming. Supplied consent and preaccepted
jobs retain their defaults. Dated production plans remain binding and prevent
optional hiring. Independent review identified the discrete-quantity issue and
confirmed the decreasing intersection loop fixes it. All 20 hiring tests pass,
including food-vs-wages, enough spare hours, no capacity and default-consent controls.

## 4. Preserve worker assessment evidence

Employment boundaries retain each bounded assessment's horizon, objectives,
requested/search maximum, baseline, alternatives, errors and worker-approved
quantity. Final delivery remains separate, so trimming is not mistaken for an
accepted forecast. Settlement recomputes this evidence. Altered selection, horizon
and alternatives are rejected without publishing state. Independent review found
no blocker; all 21 household-hiring tests pass.

## 5. Observe worker decisions externally

The existing planning observer now emits worker comparisons, with either party's
filter, selected/alternative detail and actual delivery separate from approval.
No hypothetical execution occurs in the observer. The focused CPU test passes
with Off, Selected, Alternatives and an unrelated-agent filter; observed and plain
state/ledger agree. Independent review found no blocker.

## 6. Protect accepted forward delivery deadlines

Worker assessments freeze accepted forward obligations for the worker/household
scope, including current-boundary admissions. Required remaining delivery is
compared with actual performance at the deadline, never with a forgiven ending
balance. Out-of-horizon and unaccepted offers are not obligations. Three audited
CPU/reference controls pass: protect production for an admitted two-month delivery,
ignore an unfunded proposal, and expose the limitation of a one-month horizon.
Independent review found no blocker.

## 7. Distinguish missed deadlines from later catch-up and relief

A live audited CPU control delivers nothing by month two and catches up in month
three; the three-month worker comparison retains a four-unit deadline loss.
Separate reader tests cover opening write-offs/extensions versus later relief:
accepted opening changes alter the duty; later changes cannot count as performance
or rewrite the frozen deadline. These reader fixtures do not claim to exercise
recovery settlement. Both tests pass; independent review found no blocker.

## 8. Protect accepted loan performance in its own denomination

The optional worker policy freezes scoped accepted loan IDs and compares worst
recorded arrears separately per loan. Event evidence survives enforcement and is
not confused with a cleared ending balance. A grain loan and coin wage test checks
protected payment, missed payment under supplied consent, and an unfunded proposal;
all three audited CPU/reference controls pass. Independent review found no blocker.
This measures worst arrears within the horizon, not a complete credit-risk model.

## 9. Separate worse own arrears from unrelated distress

A two-loan control gives the worker a baseline shortfall of one unit; selling
hours would increase it to five. An unrelated borrower misses seven units without
changing the worker's comparison. Finite storage is held slack and identical to
isolate payment performance (the original seven-unit store raised the baseline to
three). Audited CPU/reference runs and decision equality pass. Independent review
found no blocker. A global missed-payment boolean would miss this distinction.

## 10. Preserve already-running work across restarts

A worker starts a two-month process through normal execution before employment
becomes available. Protected scarce labor finishes its process and declines the
job; default supplied consent hires and actually aborts it; spare capacity completes
both. Needs and debt are absent so this isolates process protection. Audited CPU
continuation rebuilt after each phase matches uninterrupted Reference, including
reports and world state. All three controls pass; independent review found no blocker.

## 11. Bound the search without changing existing consent

Invalid horizons and unknown participants are rejected. A 64-hour indivisible job
can pass; a 65-hour job cannot fit this opt-in search and is not partially hired.
Default supplied consent and preaccepted employment still deliver 65 hours without
worker-assessment receipts. Four audited CPU/reference cases and three validation
controls pass. Independent review found no blocker. Documentation now distinguishes
participant eligibility, the quantity ceiling, and the covered financial duties.

## 12. Competing employers share one worker's real hours

Two households each value a two-hour job; the worker needs two hours for food.
The first hire enters the second worker assessment, which rejects another harmful
sale. Explicit rank changes the winning household; reversing catalog order does
not. Food, paid wages, decision losses and both financial books agree between CPU
and Reference. Both rank controls pass; independent review found no blocker.

## 13. Protect other members after household labor contributions

A worker with no personal need belongs to a second household whose other member
needs five grain. Its charter reserves half the worker's six hours. Selling two
hours prevents completing both three-hour food processes: supplied consent leaves
one unit unmet, while the protected policy declines and feeds the member. Reserved
contributions and sold hours remain separately bounded. Both audited CPU/reference
controls pass; independent review found no blocker.

## 14. Exercise repeated hiring, food sales and interruption together

The eight-month town-market control has a preformed household, supplied job/work
terms, four circulating coins, two opening worker grain and a two-grain lot priced
at two coins. Optional jobs are identical with and without worker protection.

| Market / worker policy | Hours delivered, months 1–8 | Food sales | Paid wages | Worker food deficit |
| --- | --- | --- | --- | --- |
| Open / supplied consent | 2,2,2,2,2,2,2,2 | 7 trades / 14 grain | 16 coins | 0 |
| Open / protected | 2,2,2,2,2,2,2,2 | 7 trades / 14 grain | 16 coins | 0 |
| Closed month 3 / supplied consent | 2,2,2,0,2,0,0,0 | 2 trades / 4 grain | 8 coins | 2 in month 3 |
| Closed month 3 / protected | 2,2,0,2,0,2,0,0 | 2 trades / 4 grain | 8 coins | 0 |

Protection preserves self-provisioning during the interruption. Both interrupted
cases end with zero employer cash and no late hiring; protection does not restore
the circulation loop. Audited Reference/CPU states, ledgers, reports and month-four
continuations agree. All 31 hiring tests pass. Independent review found no blocker.
These are supplied jobs on the town market, not a resolution of the original
forty-month discovered-income gap or discovered-land/hiring composition.

## 15. Verify the integrated crate

At code/test revision `cad8448`, the complete release suite passes **1,243 tests,
zero failed, one ignored** across 146 result targets (including empty targets).
The ignored 32-person annual accounting integration passes separately; its result
is reported separately from the normal suite count. Earlier focused tests overlap
the full suite. Strict all-target release Clippy, formatting,
repository artifact checks and diff checks pass. Independent review found the
verification coverage sufficient and no new source blocker.

Reproduction from `exp/economics`:

```sh
cargo +1.92.0 test --locked --release -j 8
cargo +1.92.0 test --locked --release --test household_accounting specialist_households_reconcile_production_trading_and_annual_dues -- --ignored --exact
cargo +1.92.0 clippy --locked --release --all-targets -j 8 -- -D warnings
cargo +1.92.0 fmt --check
```

Run `python3 scripts/check_repository_artifacts.py` from the repository root.
Raw local logs remain under ignored `output/batch16-*.log`. Cargo reported an
unrelated cache-cleanup permission warning; all commands completed successfully.
These checks verify CPU/reference behavior, not CUDA or general economic viability.
