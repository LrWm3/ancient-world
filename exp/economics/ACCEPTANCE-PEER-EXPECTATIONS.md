# Acceptance follow-up: bounded peer expectations

This ten-iteration batch investigates the governed scenario's cash-first failure.
The prior [governed acceptance](ACCEPTANCE-GOVERNED-MARKETS.md) remains a control.
Prices, initial consent, employment and finite resources remain supplied; predicted
activity is not a financial claim or counterparty consent.

## Iteration record

1. **Frozen observations.** Organization configuration now supports an opt-in
   `RecentSuccessfulStarts` forecast policy; `AcceptedOnly` remains the default.
   A snapshot uses the last two accepted, completed one-month starts for each peer
   and recipe, strictly before the observation month. Both observations must fit
   the memory window (1–24 months). It excludes the observing actor, unfinished or
   failed latest work, unpublished starts and longer processes. The interval is
   observed rather than assumed monthly. This step adds observations only.

2. **Isolated future attempts.** Real issuance and hypotheses share dated-start
   plumbing. Applying evidence is allowed only with frozen controllers at a future
   Open, at the observed cadence and before expiry. Existing starts are retained
   without duplication. Current authority, enabled recipes, process permission and
   an unmet own output-reserve goal are necessary; normal execution still decides
   resource availability. Hypotheses add no decisions, cash or claims. Tests cover
   isolation, deduplication, satisfied goals and expiry.

3. **Candidate forecasts and evidence.** Each evaluated organizational decision
   freezes one peer snapshot shared by every candidate. Private simulations apply
   its attempts at future Open and settle normally. Alternatives retain closing
   stocks and whether each hypothetical attempt completed; decisions retain their
   observation evidence through the existing serialized observer interface. The
   live world receives only the selected organization's own authorized program.
   Tests exercise actual hypothetical completion and verify no future processes
   or peer decisions leak into the live Open.

4. **Matched cash-first experiment.** `acceptance::expectations::scenario_with`
   changes only the workshop's expectation policy between the two cash-first runs;
   goals, prices and four-month horizon / twelve-month reviews are identical.
   Both initially fail: 14 mints, 13 batches per organization and ending arrears 2.
   At month 13 the observation correctly sees consecutive successful starts in
   months 11/12, but only hypothesizes month 14. The state elects every two months;
   frozen future voting leaves the month-15 office vacant. Extra demand alone does
   not solve this second missing counterparty action. This remains a negative
   control rather than relaxing readiness or silently assuming authority.

5. **Explicit voting hypotheses.** A separate
   `RecentSuccessfulStartsAndBallots` variant observes only the current begun
   state-election cohort. It repeats individual choices, including abstentions,
   through ordinary ballot admission at future pre-term Open. It never extends an
   incumbent's term directly. Already accepted ballots win; freshness is measured
   from real issuance, not renewed by hypothetical voting. Each candidate reports
   hypothetical ballot admission outcomes. Stepped tests demonstrate ordinary
   elections and process completion across future terms, preserving the live world.
   Household elections are not projected by this adapter.

6. **Cover the policy's operating interval.** The comparison separates memory
   length from horizon length while keeping annual reviews and cash-first goals.
   `acceptance::expectations::scenario` now selects the measured passing treatment:
   a twelve-month workshop forecast and 24-month process/ballot memory. All other
   fixture terms remain the governed baseline's. Ordinary forecast defaults remain
   unchanged. Four-month and accepted-intent-only variants remain controls.

   | Workshop forecast | Peer hypothesis | Memory | Mints | Workshop/town batches | Final arrears | Ready |
   | --- | --- | --- | ---: | --- | ---: | --- |
   | 4 months | Accepted only | — | 14 | 13 / 13 | 2 | No |
   | 4 months | Processes | 6 | 14 | 13 / 13 | 2 | No |
   | 4 months | Processes + ballots | 6 | 14 | 13 / 13 | 2 | No |
   | 4 months | Processes + ballots | 24 | 27 | 26 / 26 | 0 | No |
   | 12 months | Accepted only | — | 24 | 24 / 24 | 0 | No |
   | 12 months | Processes | 24 | 24 | 24 / 24 | 0 | No |
   | 12 months | Processes + ballots | 24 | 46 | 47 / 47 | 0 | Yes |

   All rows have zero food deficits. Ending arrears alone obscure prior missed
   payroll in the twelve-month accepted-only/process-only controls. The passing
   row keeps replenishment at the month-13 review, pays 48 wage coins per
   organization and has workshop net income 44 over all 48 months. These results support the combined
   bounded treatment, not an assertion that longer forecasts always improve plans.

7. **Authority and finite-execution controls.** Recurring demand now uses the
   incumbent's supplied preferences when present, matching the live decision
   adapter's override of collective objectives. A satisfied incumbent goal cannot
   be replaced by a larger collective target. Private stepped controls retain
   zero-wealth, lost process permission, closed venue and disabled-recipe failures;
   attempted procurement does not manufacture input goods or money. Election tests
   cover abstention, insufficient turnout, dead/ineligible candidates and the
   explicit future-only boundary, asserting actual vacancies rather than merely
   counting admitted ballots. Eight focused tests pass.

8. **End-to-end failure matrix.** The passing forecast treatment runs against
   missing ore supply, missing replacement authority, closed metal-market access,
   insufficient margins and late payroll on both reference and CubeCL CPU. All
   five remain unready, with equal World, State, ledger, audit and readiness.
   Additional absent-buyer and already-satisfied-buyer controls mint nothing and
   generate no recurring peer attempts or later issuer purchases. Forecast demand
   therefore does not become self-fulfilling money or consent. Two integration
   tests cover these seven full 48-month failure runs on each backend.

9. **Continuation, replay and telemetry.** The cash-first treatment passes the
   same every-phase CubeCL CPU reconstruction and dated-World replay checks as
   earlier acceptance variants, matching uninterrupted reference outcomes and
   audits. Reordering independent catalogs preserves readiness. This is in-memory
   reconstruction, not a durable checkpoint format; replay includes dated World
   inputs rather than relying on the financial ledger alone. The organization
   observer exports the frozen evidence and private candidate outcomes without
   adding future live transactions or changing World, State, ledger or audit.
   Eight continuation tests and one observer test pass.

10. **Runnable comparison and documented boundaries.** The CPU acceptance example
    selects this treatment with `--expected-peers`; its existing
    `--continuation-matrix` option also accepts that flag. README, integration,
    operating-authority and forecast documentation distinguish the opt-in result
    from the unchanged governed control and default forecast policy.

## What remains assumed

This is the experiment's full-information model, including individual accepted
ballots and incumbent preferences. It does not infer secret ballots or model
private information. Repeated ballots preserve observed choices without computing
new platforms or political preferences. Recurring work uses a bounded own-output
goal gate, not the peer's optimizer or a forecast of competing goals. Neither
hypothesis promises performance under unforeseen political or economic changes.

Evidence is frozen at each real decision, expires without renewal from imagined
outcomes and applies only at future Open boundaries. Household voting and
multi-period process recurrence are not projected. The acceptance variant still
supplies identities, consent, employment, prices, endowments and goals. Cash-first
retains secondary metal/ore objectives for candidate discovery. The subsequent
[sale-linked discovery experiment](ACCEPTANCE-SALE-DISCOVERY.md) removes them in
a separate cash-only fixture; it needs finite working inventory and posted asks,
while its cold-start control still fails. The 24-month memory and twelve-month
horizon are a measured calibration, not a universal improvement. The old
`AcceptedOnly` default is unchanged; no personal self-directed policy changes or
new organization formation are introduced.

## Run

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example acceptance -- --expected-peers
cargo +1.92.0 run --locked --release --example acceptance -- --expected-peers --continuation-matrix
```

The baseline CLI passes on CubeCL CPU: 48 months, 46 mints, 47 workshop and town
batches each, zero food deficits and no wage arrears. Both employers pay 48 wage
coins; nine separate books reconcile. Three complete post-startup operating years
pass the existing economic gates. Permanent failure controls remain failures;
recovery from unforeseen shocks is not established for this cash-first variant.

## Verification

Final verification passed **148 tests**: 18 library tests and 130 integration
tests across 24 suites. This includes acceptance and governed controls,
organizational authority/discovery/programs, state and commitment agency,
peer-stock/mint orders, forecast context, the new peer expectation/election/resource
controls, the seven-case CPU/reference failure matrix, continuation and telemetry.
The standalone CubeCL CPU baseline also passed. All-target release Clippy passed
with warnings denied, formatting passed, and repository artifact checks passed.
No generated run artifacts are committed.

Focused reproduction of the new boundaries:

```sh
cargo +1.92.0 test --locked --release -j8 \
  --test agency_expectations --test peer_election_expectations \
  --test peer_expectation_constraints --test acceptance_expectations \
  --test acceptance_expectation_controls --test acceptance_continuation \
  --test expectation_telemetry
cargo +1.92.0 clippy --locked --release --all-targets -j8 -- -D warnings
cargo +1.92.0 fmt --check
```

Independent review found no blocking implementation issue; its requested scope
qualifications and actual vacancy controls are included above.
