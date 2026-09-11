# Domestic groups and caregiving

Domestic groups now connect recorded families to individual work availability and
service recruitment. These are observations of **known people**, not a complete
resident census. Settlement cohorts still own population and ordinary employment;
existing households still own money, shares and goods.

## Membership

A group has a stable identity, a home, known members and a person or recorded-union
anchor. Recorded partners in the same settlement share a group even when they own
separate economic accounts. Known children join a co-resident parent's group.
Unrelated owners do not become relatives. At 18, an unmarried known child becomes
an independent group. Widowhood retains the union anchor; death removes the member.
A whole group's observed move preserves its identity when possible; split residence
can produce separate groups. Service travelers retain domestic membership but are
absent for local work. Membership and home changes preserve observation dates.

No additional residents, parentage, goods or money are created. This cannot identify
anonymous cohort dependents or caregivers. Orphans retain an observed family anchor
where available; the adapter does not invent guardians. It also does not turn an
ownership-account relocation into an independently selectable family migration.

## Monthly timing and effects

Opening participation reserves care before vessel crews, research and cultural
work. The toy monthly demand is 0.12 worker-months per known child under five and
0.04 from five through fourteen. Present known adults aged 15–59 provide capacity,
reduced by the existing local illness factor. Grants are proportional within each
family and scaled to the town's remaining service allowance. These rates are game
parameters, not measured time-use estimates.

Care subtracts from the same GPU external-service reservation used by other local
work, and from each caregiver's personal cultural/research capacity. After production
readback, actual care is bounded by granted and supplied service work. Its reservation
and used labor are removed before subsequent local service settlement. Unused time
expires; it is not retrospectively spent on another activity. A monthly receipt
prevents duplicate reservation and settlement. Care does not independently pay wages
or consume/create food or nutrients, and unmet care has no new mortality penalty.

Military and expedition recruitment refresh the observed family map and evaluate
proposed departures together. A candidate cannot leave while carrying an unsettled
care commitment, or if their departure would leave insufficient known caregiver
capacity for present dependents. Recruitment can still identify an anonymous adult
already counted in the cohort; it cannot call that adult a relative or caregiver.

## Controls and archives

New histories enable this adapter. History::set_domestic_households(false) disables
it at a settled work boundary; observations and provenance are retained. Old histories
without the field stay in legacy mode until explicitly enabled, recording a new
observation baseline rather than invented earlier families. Care plans, receipts,
memberships and transitions serialize with History. The participation explorer
shows group counts and requested, granted and completed care.

The cultural_work_calibrate example supports --no-domestic-care for matched runs
and reports known domestic membership and care. Raw outputs belong in output/.

## Remaining population work

Complete member rosters, household nutrition, individual births/deaths, ordinary
employment/payroll and domestic-group migration still need their own authority
transfers. This increment makes recorded family responsibilities consequential
without claiming those larger conversions are complete.

[Verification and paired seed results](domestic-participation-verification.md)
cover the work boundary, continuation, recruitment and the remaining census gap.
