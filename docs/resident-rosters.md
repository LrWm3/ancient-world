# Resident rosters and explicit relocation passengers

This increment makes named passengers explicit and provides an opt-in observation
baseline for all currently unrepresented **whole** residents. Population still
belongs to fractional GPU age cohorts. This is not yet individual demography.

## Observing residents

`History::household_resident_roster(id)` returns present, living known members of an
ownership account, including its head and recorded members. It excludes travelers,
service personnel and invalid future birthdays. Membership does not establish kinship.

At a completed work boundary, `History::identify_resident_baseline()` identifies
whole anonymous slots in each inhabited site's child/adult/elder stock. It preserves
existing IDs, relationships and stocks. New ages are deterministic estimates within
the relevant band; new members join the least populated available ownership account
with stable-ID ties. Parents are unknown. An observation event records the baseline,
not fictional earlier births or marriages. Fractional remainders and existing named
overhang remain visible in population reconciliation.

The operation preflights ownership availability and the 50,000-entry membership
limit before creating people. A repeated call at the same unchanged boundary creates
nothing. It needs society and politics membership records. It does not run implicitly
for older archives or continuously manufacture residents as cohorts fluctuate.

To exercise an expanded starting population:

    cargo run --release --example cultural_work_calibrate -- --resident-baseline --seeds 17,81,256 --years 10 --output output/roster-seeds.json

This option can change cultural opportunity and care demand because more residents
become visible to those systems. It is an experimental authority-transition aid,
not the default for all worlds. Ordinary labor, payroll and food needs remain cohort
calculations; no second workforce or population is created.

## Relocation

New journeys automatically record their known passengers and embarkation age bands.
Their payload must include those people. Anonymous passengers draw only from cohort
space not occupied by other named residents. The prior small-group target still
applies to anonymous demand; an explicit ownership group may exceed it, up to 75%
of the town, while the existing rules retain local ownership representatives.
Route distance, destination capacity, provisions and political access still gate
movement. A group that cannot fit leaves nobody behind silently.

Present passengers are unavailable for local work. Historical identity, membership
and ancestry survive arrival or return. Missing manifests in old archives retain the
previous implicit account-travel behavior; no retrospective passenger list is claimed.
Serialized manifests reject duplicate passengers, invalid ages/IDs, overlapping service
and impossible age-cohort counts. Travel age bands remain frozen just as the existing
in-transit cohorts do; birthdays are not another population transfer.

Provisions still feed the actual travel cohort and shortages debit its population.
A bounded per-band carry assigns a portion of those already-counted deaths to known
passengers, weighted by their coverage. It never debits population twice. Complete
journey extinction marks remaining passengers dead; they no longer remain immortal
traveling identities. A dead account head leaves a vacant estate, preserving property
until arrival permits ordinary succession. Death events reference the passenger IDs
and original departure. Surviving passengers remain in the persisted manifest.

## Remaining authority changes

Ownership accounts are still the relocation unit; independent domestic families do
not yet split from them. The resident baseline does not invent guardians for unknown
children. Complete ongoing resident authority needs individual births, aging and
deaths, explicit family splits/merges, and roster-based labor/food accounting. Fixed
birthdays and fractional cohort aging can still diverge, especially after a fully
named baseline. The reconciliation report must expose that discrepancy.

See [verification](resident-rosters-verification.md).
