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
