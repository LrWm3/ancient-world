# Domestic care comparisons

Domestic care now contributes to `History::resolution_report()` when the resolution
framework is enabled. It retains actual care assignments and existing labor
settlement. This is a diagnostic pooled-capacity counterfactual, not itself a communal
care service or a second demographic resolver.

## Opening inputs and actual outcomes

At the existing care reservation boundary, before learning services and workshop
staffing, capture each site's dependent demand, eligible known resident adults'
capacity, and available settlement service labor. Count each domestic member once,
including adults in units without dependents. Age, presence and the existing
illness-sensitive capacity rule constrain eligibility.

The aggregate expectation is `min(demand, pooled adult capacity, service labor)`.
It asks what could be covered if these known residents could freely help across
families. It does not claim they will help, and never reserves that extra work.
Sparse named populations still describe only observed domestic groups, not every
anonymous resident of the settlement.

Actual reservation first matches carers inside their own domestic unit and
enforces the settlement labor budget. [Neighbor assistance](neighbor-care.md) then
uses willing, connected adults and the remaining budget. Production and the existing care
settlement determine completed work. At monthly Close the adapter records:

| Comparison | Expected | Actual |
| --- | --- | --- |
| pooled_care | Dependent demand | Pooled feasible care |
| reserved_care | Pooled feasible care | Family and neighbor reservation |
| completed_care | Reservation | Completed care |

All values are worker-months. These distinguish opening scarcity, family matching,
and unused reservations. The second difference can identify an isolated dependent
household despite spare town capacity. Neighbor assistance can close part of that gap when relationships and willingness
support it. The report does not measure health consequences of unmet care.

## Boundaries and persistence

Captured inputs live with the dated care plan. Close requires a settled, reserved,
current-month plan, unchanged demand, valid sites and finite, bounded outcomes.
Receipt commits are staged, so a rejected batch does not partially change the
report. A second commit is rejected. Receipt recording changes no work, goods,
people, payments or random streams.

Actual mode is Individual because this care system uses known carers even when
demography is aggregate or ordinary named work participation is disabled. Turning
comparison metrics off retains guarded receipts without metrics. Turning the
resolution framework off leaves care execution unchanged. Old care plans default
to no projections; no earlier expectation is reconstructed. New monthly plans
capture inputs normally.

## Verification

Run:

```sh
cargo test --lib
cargo test --lib domestic -- --include-ignored --test-threads=1
```

The analytical fixture separates capacity, matching and execution shortfalls,
rejects invalid results, and checks duplicate protection after serialization.
The family fixture compares the same child and adults with and without family
connections: pooling predicts available care while actual unrelated adults provide
none. A half-completed reservation records the actual work without repeating it;
recording comparisons changes only resolution bookkeeping. The scheduler fixture
checks monthly versus batched care and checkpoint continuation with comparisons
enabled. These checks establish accounting and timing, not realistic care rates.

Verified 2026-09-11: 102 regular library tests passed (91 hardware tests ignored
in that run); all four focused domestic tests passed, including both hardware
fixtures on the available GPU. All-target Clippy with warnings denied and the
repository artifact check passed. No long-run care or mortality calibration was
performed in this change.
