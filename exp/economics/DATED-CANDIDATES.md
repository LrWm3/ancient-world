# Dated obligations in production candidates

Implemented in the standalone economics experiment. The agent now considers
known payments when deciding which productive actions to offer to its forecast.
With the same physical resources, the normal sixty-month scenario meets food,
warmth and all four annual payments. No farmer-specific behavior was added.

## Change

For each positive need's consumption input, candidate generation combines its
recurring consumption with outstanding obligations and future installments on
accepted agreements. It subtracts amounts already paid, dates future bills from
actual activation, stops new bills at right expiry, and retains old arrears once.
Unaccepted offers and hoped-for incoming transfers do not create spendable stock.

For an input covered by an accepted payment agreement, its candidate window is
at least the longest enabled matching producer's duration plus one month, or the
configured buffer window if longer. Other inputs retain their configured window.
The extra month includes the opening payment boundary after a possible harvest.
At month 7, seven grain covers seven food provisions but not those provisions
plus rent due in month 13. A six-month crop is now proposed and can finish in 12.
Previously the six-month consumption-only check omitted this action entirely.

This is candidate demand pressure, not a reservation or an alternative settlement
engine. Ordinary forecast execution still checks labor, stocks, rights, payment
priority, stage failure and consequences. Same-month harvest can clear arrears
only at the existing ClearArrears boundary. The decision portfolio can defer a
new candidate. Physical outputs, capacities, prices, monthly phases and allocation
policies are unchanged.

## Controlled CPU comparison

Run from this directory:

```sh
cargo +1.92.0 run --locked --example production_audit
cargo +1.92.0 test --locked
```

The eight main arms use identical initial resources, eighteen-month decisions,
a configured six-month buffer, and sixty months of execution. Every arm matches
CubeCL CPU against reference state, complete ledger and monthly reports; replay
and independent grain/labor accounting are also checked. These results supersede
the historical [production audit](PRODUCTION-AUDIT.md).

| Payment policy | Variant | Harvests | Food / warmth shortfall | Rent paid | Crop / fuel / unused labor | Ending grain |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Debt first | Manual | 8 | 0 / 0 | 4 | 66 / 32 / 22 | 5 |
| Debt first | Tool offered | 8 | 0 / 0 | 4 | 66 / 32 / 22 | 5 |
| Debt first | Experience | 8 | 0 / 0 | 4 | 62 / 32 / 26 | 5 |
| Debt first | Both | 8 | 0 / 0 | 4 | 62 / 32 / 26 | 5 |
| Protect essentials | Manual | 8 | 0 / 0 | 4 | 66 / 32 / 22 | 5 |
| Protect essentials | Tool offered | 8 | 0 / 0 | 4 | 66 / 32 / 22 | 5 |
| Protect essentials | Experience | 8 | 0 / 0 | 4 | 62 / 32 / 26 | 5 |
| Protect essentials | Both | 8 | 0 / 0 | 4 | 62 / 32 / 26 | 5 |

All survive with no closing arrears or debt-blocked requests. Previously debt-first
arms completed seven harvests and missed four food provisions; protected arms
completed eight but missed two. The new manual harvest dates are
6, 12, 20, 27, 35, 42, 49 and 57. Its next crop starts in 60, accounting for two
labor beyond the eight completed crops. Grain reconciles as 5 + 64 - 60 - 4 = 5.

No arm buys the three-grain tool. Experience saves four labor without additional
harvests. The already-owned-tool diagnostic saves six labor, uses the tool six
times, and still completes eight harvests. Scheduled-start diagnostics also
complete eight. The explicit crop/fuel calendar still demonstrates ten harvests,
zero deficits and 21 ending grain; it remains a separate feasibility witness,
not an autonomous result.

The normal six-month-decision payment-policy tests now also require zero food
deficits and zero closing arrears. The severe opening-shortage controls still
fail: anticipating bills cannot replace absent food or bypass debt-blocked access.

## Verification and limits

69 tests pass, including claim-window boundaries, partial/already-paid bills,
expiry with arrears, activation dates, and an identical-stock comparison proving
early planting is admitted and pays the first bill without deficits. Existing
checks cover CPU/reference equality, exact replay, forecast outcomes, reordered
catalogs, batching and continuation at payment boundaries. Formatting and strict
all-target Clippy pass. Generated traces remain under ignored output/economics.

The candidate heuristic still follows two-link need-to-consumption-to-production
chains, credits active outputs optimistically, and examines each need separately.
It does not create a production goal for payment commodities unrelated to any
positive need, plan arbitrary process graphs, or guarantee future inputs. The
lead-time window is a simple contract-linked buffer rule, not optimal inventory
control. Forecasts evaluate actual feasibility, but remain bounded. The result
shows sustainable needs and rent for this deterministic sixty-month fixture,
not maximal surplus or demonstrated robustness to shocks and heterogeneous agents.
