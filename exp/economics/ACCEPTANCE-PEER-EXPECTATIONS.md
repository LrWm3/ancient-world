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
