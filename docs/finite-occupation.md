# Finite occupation and return journeys

Successful new political campaigns can station their surviving troops for three
months before returning. `Politics.occupation_months` is an archived rule, bounded
at 0–12; zero retains immediate withdrawal. Missing fields in old histories default
to zero. The pre-existing campaign already budgets three extra supply months; this
change does not grant another food shipment, army, population or treasury.

The same campaign record holds its soldiers, remaining equipment and provisions
through muster, march, battle, occupation and return. Residents remain separate:
stationed soldiers do not contribute civilian labor or count twice toward population.
The ordinary monthly food debit and starvation mortality continue during occupation.
There is no automatic seizure of the town's subsequent harvests. The finite battle
loot remains the existing explicit transfer from the target's food inventory.

After each ration, the army checks its remaining food against one more occupation
month plus its return journey. It withdraws early if that reserve is unavailable,
if either end changes allegiance, or if the target is abandoned. Surviving people,
food and equipment return through the existing ledger path. New armies retain the
original itinerary duration, including multi-hop journeys; they no longer default
to a one-month return merely because no direct road exists. Old archives lack this
itinerary and retain their previous direct-road/fallback approximation.

Presence up to one soldier per ten residents can delay secession by up to three
crisis months. It also slightly worsens unrest and loyalty: coercive control does
not manufacture consent. These coefficients are game rules. Ordinary payroll,
local offices, autonomy, household pressures and cultural identity remain involved.
`occupation_started` links the conquest; withdrawal links the occupation; return
links withdrawal. The explorer labels stationed armies and their assignment end.

This adds a supply-limited occupation phase, not a permanent regiment system.
Recruitment/reinforcement, shipment-based military resupply, siege engines, blockades,
unit experience and separately resolved return-path hazards remain open. Historical
phases persist as events after the army demobilizes; no idle permanent manpower pool
is created. Existing civilian displacement continues through household journeys.

## Century observations

Final-build seeds 17 and 409, with offices enabled and the living environment,
recorded 14 and 15 occupations respectively. All 29 ended before the final census.
Final populations were 2,251 and 2,795; maximum observed relative ledger residual
was 1.96e-5. These are behavior checks, not a claim that occupation improves welfare.
Only one occupation appeared in the annual samples across both runs: short phases
must be counted through events, not inferred absent from an annual map.

The controlled fixture verifies finite food, starvation bookkeeping, early withdrawal,
political access loss, unchanged army allegiance after both towns are conquered,
non-duplication of soldiers/equipment, and serialized continuation. It also checks
that coercive presence lowers loyalty rather than masquerading as consent. Existing
political tests cover conquest, defeat, competing claims, genealogy and checkpoint
continuation; office regressions still pass.

[Artifact retention policy](evidence/README.md)
identify the exact build and the scope of each test. Future recruitment, resupply and
siege calibration should retain explicit phase events and monthly supply mediators.
