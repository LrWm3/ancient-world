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
   organization and has workshop net income 44. These results support the combined
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
