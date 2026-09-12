# Family care and spare-time assistance

Local voluntary assistance previously excluded every domestic unit with dependent
care demand. A parent whose own child was fully cared for could not help an
isolated neighboring child or elder, even with spare time and a strong relationship.

The opening care reservation now protects family grants first, then admits helpers
from units whose own demand is fully covered. Eligibility is captured before neighbor
matching: receiving help does not qualify a household to pass that help onward.
The existing generosity, kinship, estrangement, presence and site-labor limits still
apply. Each helper offers to at most one other unit, using capacity remaining after
family commitments. The neighbor-help switch disables this additional allocation.

A person may consequently appear in a family row and a neighbor row. Validation
rejects duplicates within one row, assignments across towns, and combined care
above the maximum personal budget. Monthly participation subtracts the sum of
care commitments; settlement uses the same finite service work pool and records
actual completion. No new population, food, money, health effect or archive field
is introduced.

This remains local support. Cross-town help requires explicit travel or remittances;
the current recipient-site care rows cannot safely charge a remote helper's labor.
Earlier helper IDs retain priority when several helpers compete for unmet demand.

## Verification (2026-09-12)

On the Quadro RTX 5000 / Vulkan:

- Both hardware-backed domestic tests pass. The controlled sharing case preserves
  0.12 worker-months of family care and adds 0.10 for an isolated dependent. The
  helper's combined care is 0.16 and remaining participation capacity is 0.64.
- Disabling neighbor assistance restores the family-only allocation. With only
  0.06 site worker-months available, family care is underfilled and no help is
  diverted outward. A deliberately corrupted pair of individually valid rows
  exceeding one helper's combined capacity is rejected.
- Saved opening-state settlement agrees with uninterrupted settlement. The existing
  GPU monthly/batched/checkpoint fixture also passes over twelve continued months.
- Regular library suite: 127 passed, 110 hardware tests skipped. This is separate
  from the two explicitly executed hardware tests above.

These are boundary and continuation results, not a long-run balance assessment.
No claim is made yet about mortality, migration or multi-seed institutional effects.
