# Reconciling named people with population stocks

The GPU's child, adult and elder cohorts still own population. Named people are
incomplete historical identities with fixed birthdays, household links and physical
presence. The two representations previously drifted far enough that a town could
have more named adults than adult residents. This adapter measures that drift and
closes two sources without deleting inconvenient identities or adding population.

## Read-only reconciliation

History::population_reconciliation classifies every identity exactly once as a
resident in an age band, expedition member, soldier, relocating person, dead,
unresolved, or a resident with an invalid future birth. Residents use ages <15,
15–59 and 60+, matching the cohort labels. Site rows expose:

- Fractional cohort stock and whole known resident count for each age band.
- Unrepresented whole slots: max(floor(cohort) − known, 0).
- Overhang: max(known − cohort, 0).

Do not sum positive and negative site differences into a reassuring global result.
Surplus adults in one town cannot fill unnamed slots in another town. Travelers
are reported separately, and unresolved living people are not silently declared
dead. The participation explorer and cultural_work_calibrate reports expose this
audit. The audit itself changes no state.

## Birth identification

In the new adapter, genealogy needs both its existing accumulated birth credit and
an unrepresented child slot at the current settlement. Each newly recorded child
uses one slot; later families in the same pass see the reduced allowance. This
identifies a child already counted by the GPU, rather than adding a second birth.
Named birthdays remain an approximation until demographic authority is transferred.

## Assigning already-counted mortality

Previously, ordinary named non-heads and ownership heads normally consumed death
credits only after age 70. Younger people could therefore survive cohort famine and
disease outcomes merely because they had a name.

The monthly execute phase now observes the change in cumulative demographic deaths
across the production dispatch, before later expedition or military losses. After
care settles, an identity adapter distributes a bounded portion of these losses
among present named residents. It does not run another mortality simulation.

For each town, with age weights w, known counts K and completed cohort counts C:

    coverage = clamp(sum(K × w) / sum(C × w), 0, 1)
    target = fractional_carry + fresh_GPU_deaths × coverage
    named_deaths = min(floor(target), eligible_known_people, floor(unassigned_death_credit))
    next_carry = fractional_part(target)

Zero cohort exposure gives zero coverage. Weights follow the existing toy mortality
ordering: child/adult/elder base weights 0.0005/0.0006/0.003, plus unmet-ration weights
0.06/0.025/0.05 and disease burden × 0.01. These select which already-counted deaths
receive names; they do not create additional deaths. Candidates are sorted by a
seeded weighted sampling key, with stable person IDs breaking ties. There is no
age-70 exemption, and service travelers never enter the local candidate pool.

Whole assignments consume existing unassigned mortality credit. They change person
death dates and produce site events with person references, fresh loss and coverage
values. They never decrement resident population, food, money or nutrients a second
time. Only a fractional remainder carries forward; insufficient candidates cannot
build a backlog of hypothetical named deaths. A monthly stamp prevents repeated
assignment. Old accumulated archive credits alone cannot trigger new assignments.

Existing response stages handle the consequences: head succession, ended unions,
loss of knowledge holders, canceled personal work and subsequent family observation.
There is no additional head/non-head age-70 death pass while this adapter is enabled.
Local production has already happened when these deaths are assigned; later personal
work must still check whether its actor survives. This is an explicit monthly timing
choice, not a daily sequence.

## Compatibility and remaining gaps

New histories enable NamedDemography. Older archives missing the field retain the
legacy adapter until History::set_named_demography(true) is called at a completed
work boundary. It starts with no fractional credit or retroactive death assignment.
The --legacy-named-demography calibration option retains the previous birth and
named-mortality behavior for comparisons. Adapter state is serialized and validated.

This is still **not a complete resident registry**:

- Cohorts age gradually through fractional transfers; named people cross age bands
  on birthdays. Exact agreement cannot be expected from those two rules.
- Ownership-account relocation is not yet a roster of independently moving families.
- Birth identification, service recruitment and resident succession share admission
  checks, but initial ownership setup and society-disabled leader replacement still
  create representative identities outside that contract.
- Cohort losses may outrun the sampled named share locally. Existing overhang is
  reported, not resolved by inventing deaths or moving people between towns.
- Food consumption, ordinary employment, fertility and demographic totals remain
  aggregate. No complete biographies are invented for anonymous slots.

The next authority transfer needs admission checks for remaining representative
creation, explicit conversion of fractional stocks, and roster-backed movement
before individual births/deaths replace the GPU cohort calculation.

The new earlier death boundary also requires a relocation guard: an ownership account
with a deceased head cannot depart before succession supplies a living representative.
Otherwise the account could become stuck in transit while the succession pass skips
away households. Existing traveler guards remain in force.

The adapter reuses existing production readback; it adds no terrain snapshot or GPU
transfer. [Verification and paired results](population-reconciliation-verification.md)
record the improvement and remaining discrepancies.

## Resident succession

With the named-demography adapter enabled, ownership succession now searches
existing adults before creating a representative. It prefers a recorded child,
then a member of the same ownership account, then another resident of the town.
Candidates must be at least 18 and physically present. Existing account heads and
civilization leaders are excluded; birth date and stable ID break ties. A monthly
occupied set prevents two accounts choosing the same successor. This remains a
simple succession rule, not an individual consent or inheritance-law model.

An appointed head joins the account's membership record at the same site. Recorded
parents and unions are preserved; appointment does not invent descent. Population,
property shares and inventories are not increased. An heir whose home is local but
who is currently traveling is ineligible, including in the legacy heir path.

If there is no eligible existing resident, the adapter checks whole unrepresented
adult slots, then elder slots, using the same age-band reconciliation as birth
identification. Multiple identifications in a month see the reduced allowance.
An anonymous elder representative is initialized at 60 instead of identifying an
extra 25-year-old adult. These ages are estimates, not reconstructed birthdays.

**Estates can now remain vacant.** If no existing adult or whole anonymous
adult/elder slot is available, the account retains its last head as a historical
reference and records `vacant_since`. No person or population is added. Later
succession retries against current residents and slots, without accumulating a
backlog of hypothetical successors. Claims and wallets stay with the account.
[Estate vacancies](estate-vacancies.md) describes passive subsistence, business
closure, recovery and separately vacant political leadership.

Initial ownership setup and society-disabled leader replacement still retain
representative identity creation. Service recruitment already requires anonymous
adult slots. Ordinary employment and household consumption still use cohort
proxies, not complete person rosters.

## Shared admission contract

`ResidentSlots` now supplies the same whole-slot calculation to genealogy, service
recruitment and succession. It snapshots known resident counts before a subsystem
can temporarily remove household or kinship context from History. Each claim checks
current authoritative stock, not a saved allowance: a loss during the pass can
reduce availability. Reservations are all-or-nothing and consume only the requested
site and age band. Nonfinite or negative stocks cannot authorize identification.

The ledger is temporary and never serialized. A later subsystem constructs a fresh
ledger from identities actually created earlier, so it cannot reuse an anonymous
slot already named by succession or recruitment. Existing identities require no
new slot. Legacy creation/death observations adjust the temporary count only; they
never modify population stock. Legacy birth/succession compatibility remains explicit.

This is a shared admission rule, not a complete census or a new demographic model.
See [admission and regression verification](resident-admission-verification.md).

## Explicit roster follow-up

New relocation journeys now carry named passenger manifests, and an opt-in resident
observation baseline identifies all available whole residents without increasing
population. See [resident rosters](resident-rosters.md) for compatibility, age-band
constraints and the remaining cohort-authority gap.
