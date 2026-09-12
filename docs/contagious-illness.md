# A first contagious illness

New histories enable one toy infection; older archives without the field retain
their previous health model. The generator supports `set_contagion` and
`introduce_infection`. The starting baseline converts one existing resident to
exposed, without adding population.

Each settlement has susceptible, exposed, infectious and recovered compartments.
Monthly transitions use the opening snapshot:

- Exposure: S × (1 − exp(−0.8 × I / population)).
- Progression: 60% of E becomes I; 50% of I becomes R.
- Waning protection: 1% of R becomes S.

These are deliberately slow monthly game parameters, not a fit to a named disease.
Compartment transfers conserve people. Census reconciliation follows authoritative
demography: net additions enter susceptible, net reductions remove proportional
shares. Addition, removal, incoming and outgoing ledgers explain each pool's
inventory. They are not an independent birth/death calculation or individual
diagnoses. Net census reconciliation does not reconstruct every simultaneous
birth and death.

Household departures remove a proportional health partition from the source;
journeys retain it, progress monthly, lose the same people as their actual travel
cohorts and merge it at arrival. Commercial cargo captures an attendant-exposure
estimate at departure's Close and applies a bounded contact only on actual
delivery. Longer journeys reduce that estimate. This is an abstract contact per
shipment, not infection in the goods or a named shipboard outbreak. Military and
expedition disease compartments remain outside this initial pathogen model.

Infectious prevalence sets a lower bound on existing remembered illness burden.
That same burden already influences nutrition-sensitive mortality, births and
work. No second casualty or labor debit is applied. Recovery of the burden still
uses existing health-memory rules; it is not identical to current prevalence.
An arriving exposed traveler cannot transmit locally until a subsequent disease
update.

Checks cover closed SEIR conservation over 1,200 months, travel splits/merges,
local exposure, actual cargo delivery, and twelve-month checkpoint/batch equality.
These establish accounting and timing, not long-run demographic balance or
epidemiological accuracy.
