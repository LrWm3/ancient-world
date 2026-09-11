# Neighbor assistance for domestic care

Known local relationships can now turn spare personal time into care for another
household. This is a small voluntary-help rule, not a general welfare institution
or a simulation of social obligation.

## Allocation policy

After normal family care is matched and limited by town service labor, consider
resident adults from domestic units with **no dependent-care demand**. Each helper
must have a cultural agent with positive generosity and a relationship above 0.2
toward a currently resident member of a household needing help. A common town or
religion alone does not establish that relationship.

A helper prefers the eligible household with the highest relationship strength
multiplied by its fraction of unmet need. Ties use stable domestic-unit IDs;
helpers are processed in stable person-ID order. Each helps at most one household.
The monthly offer is at most `0.1 × generosity × relationship` worker-months,
further capped by their illness-sensitive capacity, remaining need and town labor.
These are toy behavioral parameters, not empirical estimates.

Family care keeps priority. This first version does not take extra work from adults
already in a household needing care, even if those adults have spare time. It also
does not promise fair division between equally willing helpers or needy households.

## Integration and controls

Help is added to the existing care row with the actual helper's person ID. Personal
availability subtracts it alongside family care, and a committed helper cannot
leave before care settlement. GPU service labor limits completed care through the
existing settlement step. No extra wages, goods, population, or automatic health
benefits are created. Repeated calls cannot reserve or settle the work twice.

`History.domestic.neighbor_help` enables the rule by default, including when loading
older domestic state. Set it to false before the next monthly reservation for a
no-assistance comparison. Already reserved care is honored. Existing saved monthly
plans retain their assignments; help is considered when the next plan is built.

[Care resolution](care-resolution.md) now compares the pooled capacity ceiling
with the combination of family and willing-neighbor assignments. The remaining
matching gap can reflect missing relationships or willingness, in addition to
family structure. It is not solely a measure of isolated families anymore.

## Verification

```sh
cargo test --lib domestic -- --include-ignored --test-threads=1
```

The controlled child-care fixture checks disconnected and connected households,
disabled assistance, an unavailable helper and exhausted town labor. Positive help
reduces the helper's other-work availability, blocks departure until settlement,
and appears in completed care and resolution receipts. A serialized reservation
continues identically. The monthly/batch/checkpoint fixture uses a child outside
the helper's family so continued assistance exercises the full scheduler.

This change does not introduce new friendships, payment, cross-town travel,
reciprocity memory, or mortality penalties for unmet care. It establishes the
relationship → commitment → capacity cost → completed-care connection first.

Verified 2026-09-11: all four focused domestic tests passed, including the two GPU
fixtures; 102 regular library tests passed (91 hardware tests ignored in that
run). All-target Clippy with warnings denied passed. In the controlled fixture,
the connected adult supplies 0.1 worker-month, retains 0.7 for other activities,
and the care ledger completes 0.1. Disconnected, disabled, unavailable and exhausted
budget controls supply zero within numerical tolerance. Long-run assistance rates
and social consequences have not been calibrated.
