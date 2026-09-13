# Retaining useful tool substitutes

The production planner previously subtracted held bronze/copper tool service from
the desired generic tool reserve, but never requested those held substitutes.
Their target therefore remained zero. Ordinary export surplus calculations protect
75% of the local target, so these useful alternatives had no such protection.
Planning could also reuse their unreserved availability for another request.

Tool planning now assigns existing generic, bronze, then copper tools to the
required service, decrementing scratch availability and recording targets for the
actual goods. Only unmet service requests new output from the previously selected
local production chain. Copper retains its existing lower service factor. Alloy
substitution still requires the existing alloy-enabled setting. This does not
manufacture tools, change wear, alter material efficiency or require one reserve
of every tool type. Unneeded stock remains available for other demands and trade.
The same helper serves urgent replacement forecasts and normal reserve planning.

This is forecast allocation; export policy still retains its existing 75% reserve
fraction. It does not create exclusive physical custody. Incoming cargo is included
as expected supply using the existing planning rules and becomes usable only on
arrival.

The change does not yet choose an unfamiliar imported substitute merely because a
nearby ruin or supplier stocks it. That remains a separate procurement comparison.
It also does not establish workshop profitability or fix household affordability.

## Verification

The focused fixture checks mixed-stock service coverage, positive targets for
held substitutes, no duplicate production orders, residual tradable stock, bounded
reuse across a second request, alloy-disabled behavior and partial-stock upstream
orders. Runtime and broader comparisons are recorded after execution below.


All six production planner unit tests pass (0.01 s). The existing hardware
replacement-tool/experience integration test passes (47.51 s), including finite
materials/work, no free zero-skill advantage, economy ledgers and three-month
batch versus serialized continuation. This validates the existing integration
fixture, not every alloy/import combination. Native build passes.


## Seed-1024 screen

Four 50-year runs completed, compared with the preceding recovery-enabled screen.
Use `scripts/monetary_experiment.py`, the same 32/32 founding checkpoint and
common flags documented in [abandoned stock recovery](abandoned-stock-recovery.md).
Recovery remains enabled in both versions. Local outputs are under
`output/tool-retention-screen`; previous outputs are
`output/recovery-border-screen/on`. The script freezes the tested executable.

| Arm | Population before → after | Completed operator work before → after | Cumulative operating margin before → after | Recovery arrivals before → after |
| --- | ---: | ---: | ---: | ---: |
| Baseline | 163.044 → 163.451 | 4.450 → 6.411 | 25.76 → 41.68 | 14 → 8 |
| Credit | 163.044 → 163.451 | 4.450 → 6.411 | 25.76 → 41.68 | 14 → 8 |
| Issuance | 162.117 → 162.906 | 4.456 → 6.251 | 25.89 → 39.88 | 5 → 7 |
| Combined | 162.117 → 162.906 | 4.456 → 6.251 | 25.89 → 39.88 | 5 → 7 |

All operators remain closed at year 50. Operating margin excludes financing,
capital, dividends and liquidation. Maximum absolute relative cash residual in
the four new runs is 2.31e-7. Strict all-target Clippy passes. Compilation overlaps
part of the baseline experiment, so these runs do not establish isolated timing.

This is modestly favorable in one seed, not a demonstration of general viability.
It does not identify every mediator behind nonlinear endpoint changes. The
controlled fixture establishes the concrete planning error independently of those
outcomes. Held-out seeds and a separate comparison of importing substitutes
against requesting the local production chain remain outstanding.
