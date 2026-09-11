# Resident daughter settlements

The full-common-food century control exposed a population-authority mismatch at
seed 17/month 673: annual founding had deducted 20% of a parent town's cohorts,
but its named residents still lived there. The next monthly observation correctly
rejected the inconsistent state. Extra food made a previously rare growth path
reachable; this was not a reason to weaken population validation.

Individual mode now refines the existing annual founding target into whole
ownership households. The existing population, food, survey, distance, farmland
and flood checks still decide whether to propose a daughter town. Among present
households, the refinement selects stable-ID-ordered groups fitting the bounded
40–90 resident target. Households with members on expedition/military service or
in relocation cannot leave, and the civilization ruler's household stays behind.
The selected group must contain an adult and leave residents and ownership at the
parent. If no feasible group or insufficient provisions exist, founding makes no
changes and can be considered at a later annual boundary.

The commit moves existing household membership rather than creating new residents.
Birth dates, ancestry, person IDs, faith, wallet balances and household identity
persist. Age-band transfers are counted from those people at the completed month.
The new site receives the actual population, twelve months of existing provisions
at the existing conservative 18 kg/person-month rate, and the same population
fraction of the parent's municipal cash. Its seed stock is a transfer from those
provisions. Moved and remaining private ownership shares are normalized within
their respective towns, following the existing relocation convention. This is not
a model of selling each piece of land or moving physical buildings.

Founding remains the existing annual regional movement abstraction: it does not
create a second migration journey or give the daughter town production in the
month just completed. Routes and resource claims enter the existing initialization
path; actual monthly production starts afterward. Household presence derives from
its new location, while completed participation receipts retain their original
work site. An aggregate history retains its existing fractional founding path.

The migration event records the participating households and people, actual
population, provisions and municipal transfer. No new patron arrival or replacement
founding inventory is created. Historical objects left at the parent are not
silently teleported to the daughter town.

Verification is in `civilization::daughter::tests`: whole-roster and age transfers,
identity preservation, ownership totals, food/cash conservation, no-op controls
for absent households and missing provisions, serialization replay, normal site
initialization and full checkpoint versus monthly/batched continuation. The
common-food century rerun is required in addition to the fixture; fixing roster
movement alone does not establish viable long-run population balance.

The corrected seed-17 common-food century now completes with 7,019 residents,
27 active sites and zero population residual; see the [balance evidence](resident-payroll-balance.md).
This establishes continuation through repeated daughter founding, not the balance
of the default household purchasing policy.
