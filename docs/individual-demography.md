# Opt-in individual demography

`History::enable_individual_demography()` names available whole residents and makes
those identities own monthly birthdays, births and natural deaths. It is a toy
population model, not a reproductive or epidemiological model. Existing worlds
and the default scheduler retain their prior demographic behavior unless enabled.

The operation requires a completed monthly boundary, society and politics, no
known people in transit/service, sufficient membership capacity, and population
stocks that already contain their known residents. Failed conversion leaves the
history unchanged. Fractional anonymous residents are preserved, never rounded
into extra people. Old archives default to the previous mode.

## Monthly authority

1. After opening arrivals and reservations, observe resident IDs and their ages at
   the preceding completed month. Check each cohort contains those residents.
2. GPU production computes rations, disease exposure and weather normally. A mode
   bit suppresses the GPU's independent aging, birth and death transitions.
3. Natural mortality samples each observed person's preceding age band once, using
   a stable seed/person/month stream. Survivors enter the cohort implied by their
   actual birthday at the current month (adult at 180 months; elder at 720).
4. Birth expectations accumulate by site. Each whole birth creates one person,
   membership entry and population increment. Fractional expectation persists.
5. Anonymous residuals alone keep fractional cohort aging/mortality. Derived site
   population and ledgers combine named transitions and those residuals. Genealogy
   cannot spend another birth credit for the same newborns.

The retained monthly mortality probabilities are `[.0005,.0006,.003]`, plus unmet
ration fraction times `[.06,.025,.05]`, plus disease burden times `.01`. Birth
expectation is opening adults × `.004` × adult ration sufficiency × `(1-disease)`.
These are existing game parameters. Eligible recorded couples supply parentage
when available; other births record unknown parents. There is no pregnancy model.

## Transfers and persistence

Defensive casualties consume whole resident identities, with persisted fractional
expected-loss carry. Relocation starvation samples named passengers once and
subtracts whole deaths; only anonymous losses remain fractional. Arrival cohorts
account for birthdays completed in transit. An arrival-month birthday belongs to
that month's demographic execution, avoiding double aging. Transit ration bands
remain the embarkation bands.

State archives include mode, completed month, birth carry and defensive-loss carry.
Repeated/stale demographic settlement is rejected. Conversion and monthly roster
checks report incompatible stocks rather than silently killing people or changing
birth dates. Grouped birth/death events retain affected person IDs.

## Scope and reproduction

    cargo run --release --example cultural_work_calibrate -- --individual-demography --seeds 17,81,256 --years 10 --output output/individual-seeds.json

Labor, payroll and household consumption still use aggregate projections. Ownership
accounts are not yet independently splitting domestic families. Anonymous fractional
residents retain approximate aging. Full default activation needs broader coverage
of old archives, colony creation, warfare and long histories. No exact continuation
of the former fractional demographic trajectory is promised.

See [verification](individual-demography-verification.md).
