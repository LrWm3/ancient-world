# Occupation-specific production experience

Named farming, forestry, mining and construction assignments now distinguish real
worker-months from untrained-equivalent work sent to GPU production. Each resident
stores four independent completed-time counters; working in one sector does not
teach the other three. Old records begin at zero rather than inventing experience.

For each assignment, the bonus is frozen from previously completed time:

```
bonus = 0.5 * prior_sector_work / (12 + prior_sector_work)
effective_work = actual_time * (1 + bonus)
```

Twelve completed worker-months yield a 25% rate bonus; the limit is 50%. These are
bounded game settings. The existing aggregate production forecast remains the
request in untrained-equivalent units. Reserve scales available personal time by
competence, caps effective grants at that request, then reserves actual time only.
This can meet a request with fewer worker-months or improve fulfillment when
attendance is scarce. It does not raise the forecast itself.

GPU production consumes the effective allowance through its existing equations and
finite inventories. No shader buffer or additional readback is introduced. Farming
still needs land, water and nutrients; forestry and mining deplete finite stocks;
construction still consumes actual goods. GPU work-use fields now describe effective
work, while the CPU plan retains actual grants and each frozen multiplier.

Settlement applies the completed fraction to each worker's actual grant. Only that
actual completed time enters personal experience, commitments and resolution work
receipts. Unproductive reserved time remains unavailable until next month's boundary,
as before. Household earnings use actual reserved attendance weights and never
multiply those weights by skill. There is no automatic skill wage premium.

Resolution reports include a separate `effective_production_work` metric, with
untrained-equivalent units. Actual attendance and completed-work metrics remain in
worker-months. Legacy assignments default to zero bonus; saved new assignments
retain their rate through continuation. New records validate finite nonnegative
experience and bounded multipliers.

Workshops and merchant crews retain their own models. This does not add education,
forgetting, cross-sector transfer or skill-dependent project quality. Freed personal
capacity does not retroactively reopen earlier reservation windows.

## Verification (2026-09-12)

Four GPU forecast/attendance tests passed. The matched fixture supplies the same
small requests and stocks to novices and workers with 12 months in each occupation.
All four sectors completed positive work; effective allowances match within 1e-4,
while experienced workers used 80% of novice time within a 0.002 ratio tolerance.
Goods, cultivated area and housing output match within 0.002. Repeating settlement
on the serialized experienced state changes neither history nor experience.

Existing fixtures also cover absent workers, household wage allocation, finite
extraction, contracted workshops, and batched versus monthly checkpoint continuation
through the normal pipeline. Ordinary library tests: 151 passed, 130 ignored.
All-target Clippy passed with warnings denied. These are controlled comparisons,
not a new multi-seed or century-scale balance calibration.
