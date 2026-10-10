# Twenty further independent review iterations

Starting revision: `78202fa`. Scope: post-v1 discovery, useful economic outcomes,
and composition of persons, households and the state. Each numbered iteration is
independently reviewed, tested and committed. This does not reopen the completed
bounded v1 release or promise completion of the broader ambitions. Individual
self-directed policy changes remain deferred. Generated runs stay under ignored
`output/`; this record retains the settings, findings and limits.

## 1. Separate forward assessment from market stocking

Added optional `FinanceRule.forward_horizon`, capped at 24 months. The assessment
of both counterparties covers the maximum of this setting, the existing discovery
horizon, and delivery duration plus the existing buffer. `None` preserves the
historical policy; a short override cannot truncate the delivery forecast. Market
holding targets continue to use the discovery horizon. Assessment diagnostics
record the inclusive interval and number of projected suppliers, including zero.

The fourteen-food worker with competing one-hour persons uses a fourteen-month
assessment while retaining a four-month market horizon. Its finite food buffer
is no longer a projected surplus to sell; no food forward is admitted and its
nutrition remains covered. CPU/reference state, ledger and audited books agree
through phase-by-phase reconstruction. An eight-month assessment still accepts
and fulfills delivery of unneeded stock from a funded bilateral surplus control; invalid horizons
are rejected. The original short-horizon late-hunger control remains supported.
This is a supplied static financial decision policy, not a self-modifying agent.

Validation: 25 discovery/supply tests passed, including the unchanged productive
baseline; strict all-target Clippy, formatting, diff and artifact checks passed.
Independent review found no blocker. Longer forecasts remain conditional and may
reject deals that a short forecast accepted; they are not a universal welfare rule.
