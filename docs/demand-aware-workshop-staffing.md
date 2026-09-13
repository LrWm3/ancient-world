# Demand-aware workshop staffing

An opt-in Reserve-phase policy limits workshop labor requests using the current
recipe orders already assembled by `plan_production`. The previous policy uses
lagged completed work, headroom, a minimum shift, and optionally due contracts.
Those remain the unconstrained request. Originally, the ceiling was the operator's
leased share of requested recipe batches times recipe work in its family. The
[current input-feasibility extension](workshop-input-feasibility.md) also caps it
against a scratch-stock forecast and excludes food recipes that cannot earn
operator fees. The lowest ceiling proceeds to cash and labor allocation. Rent
remains payable; zero usable ordered work means zero wage reservation.

This is a demand forecast, not an input reservation. A recipe order can still fail
because of competing inputs, equipment, labor or downstream limits. Contracts
remain due and report incomplete work/refunds normally. Suppressed attendance is
not counted as completed service. A later positive order can request workers again
while the operator remains open; the existing prolonged-idleness closure still
applies. No materials, town cash or income are created.

## Controls and records

- `Enterprises.procurement.demand_staffing`, default false including old archives.
- Native `--demand-workshop-staffing[=true|false]`; omission preserves archive policy.
- Experiment runner `--demand-workshop-staffing` applies the setting to all monetary
  arms and records it in local metadata. Without the flag it explicitly disables
  the pilot, just as the existing contract-staffing comparison does.
- `staffing_observations` retains the latest Reserve boundary's firm, month,
  unconstrained work, ordered-work ceiling, optional input-feasible ceiling and
  requested work before cash caps.
  Operator `last_requested_work` remains the post-cash request. Observations are
  forecasts, never additive work inventory.

This does not enable procurement, credit, or contract-aware staffing. It does not
alter phase order or individual work arbitration.

## Verification

The automatic-procurement GPU fixture is extended with a due funded contract and
zero current orders: no operator wages or worker reservation should occur; money
is conserved and the contract remains unearned. Restoring recipe orders permits a
positive request again. The enabled policy also runs through the existing batched
versus checkpoint-resumed execution comparison. The GPU fixture passed (one test, 4.50 seconds after compilation). The independent CLI override test passed, as did strict all-target Clippy, the
native build and all 12 monetary report tests. A four-arm seed-1024, 50-year
comparison completed from the same founding checkpoint as the prior staffing
screen, with demand staffing enabled in all arms. All four exited successfully.
Outputs remain local under `output/demand-staffing-screen/on/`.

| Measure | Prior, no issuance | Demand cap, no issuance | Prior, issuance | Demand cap, issuance |
| --- | ---: | ---: | ---: | ---: |
| Operator completed work | 21.652 | 21.575 | 22.013 | 20.415 |
| Operator paid work | 38.556 | 31.023 | 35.482 | 27.003 |
| Revenue minus wages and rent | -587.78 | -269.35 | -425.18 | -135.10 |
| Operator records | 10 | 12 | 11 | 10 |
| Operators still open at year 50 | 0 | 0 | 0 | 0 |
| Population | 156.000 | 156.688 | 156.676 | 157.926 |
| Terminal need-weighted hunger | 0.06117 | 0.06939 | 0.07178 | 0.06544 |

Credit creates no loans in either enabled arm. Full histories match within each
issuance pair after removing only the credit records. The maximum endpoint
absolute relative money discrepancy for the new runs is 1.556e-7. That is an
endpoint check, not a bound on every monthly residual.

The cap reduces paid work and operating losses. Food access worsens without
issuance and improves with it. All firms still close, and output falls in the
issuance comparison. This is not sufficient evidence of viable workshops or an
economy-wide improvement; keep the pilot opt-in. Next inspect input availability,
forecast overcommitment and customer payment against wages/rent, retaining the
original demand and suppressed/uncompleted service in diagnostics.

The table above describes the original demand-only implementation. Input-aware
staffing is now implemented in the linked extension; multi-seed viability and
satisfactory economy-wide outcomes remain unproven.
