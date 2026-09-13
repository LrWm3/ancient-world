# Demand-aware workshop staffing

An opt-in Reserve-phase policy limits workshop labor requests using the current
recipe orders already assembled by `plan_production`. The previous policy uses
lagged completed work, headroom, a minimum shift, and optionally due contracts.
Those remain the unconstrained request. The new ceiling is the operator's leased
share of the sum of requested recipe batches times recipe work in its family.
The lower of that ceiling and the original request proceeds to cash and labor
allocation. Rent remains payable; zero ordered work means zero wage reservation.

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
  unconstrained work, ordered-work ceiling and requested work before cash caps.
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
comparison is running from the same founding checkpoint as the prior staffing
screen, with demand staffing enabled in all arms. Its outputs are local under
`output/demand-staffing-screen/on/`; results are pending.
Multi-seed viability and input-aware staffing remain outstanding.
