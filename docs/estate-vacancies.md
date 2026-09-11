# Vacant ownership accounts

An ownership account is a persistent claim on communal private stocks and a finite
wallet. It is not the same thing as a biological family or a complete resident
roster. A deceased head no longer forces creation of an extra adult merely to keep
that claim valid.

## Succession and timing

During the monthly social response, succession first searches present eligible
adults, then whole unrepresented adult or elder slots. When neither exists, the
account keeps its previous `head` ID as a historical reference and sets
`vacant_since` to the current month. `household_vacant` identifies the account and
deceased person. Repeated months do not repeat this event or create people.

The account remains eligible for a future representative. Current membership and
cohort slots are checked anew; no missed-successor debt accumulates. Recovery clears
the vacancy, preserves the account ID, property share, wallet and ancestry, and
records inheritance plus `household_represented`. One available whole slot can
restore only one account in that pass. Known people are not resurrected or silently
moved from another settlement.

Existing vacancies remain vacancies if named demography is subsequently disabled;
their resolution still uses resident/slot checks. Other legacy-mode successions keep
the older behavior. Archives missing `vacant_since` deserialize it as absent. A
vacancy must reference a deceased head, start no earlier than that death or the
account's founding, and be dated no later than the current month.

## Property and activity

Vacancy itself transfers no cash, goods, population or land. It does not liquidate
ownership shares, redistribute them to neighbors, or clear recorded kinship.

- Relocation requires a living head, so a vacant account cannot launch a journey.
- Dead heads cannot participate in cultural work or faction leadership selection.
- Owner-managed enterprises close at their next preparation stage while the account
  remains unrepresented. Existing company cash returns to the same wallet once;
  the equipment lease ends without duplicating communal equipment. The vacant
  account cannot found a replacement company.
- Household food purchases, cohort payroll, passive dividends and relief remain
  aggregate resident entitlements. They can support dependents and unnamed workers
  through the estate wallet. The deceased head supplies no new occupation-derived
  payroll weights. Previously recorded livelihood weights remain group proxies.

The latter is deliberately limited: there is no executor person, complete dependent
roster, probate fee or individualized wage accounting yet. An estate receiving food
is not a dead person eating. Total need still derives from resident age cohorts,
and the same finite cash and food budgets constrain allocation.

Existing companies can continue after immediate succession to a living head. The
closure rule applies when their account has no living representative at preparation,
not automatically on every historical death. Paid work completed before the death
is not retroactively canceled.

## Political vacancy is separate from inheritance

The ruler's civilization identity, rather than the household site's current
administrator, determines which office is inherited and who is eligible. Conquest
or relocation does not silently transfer that office to the host civilization.

If the deceased head was also the ruler, the vacant estate is not awarded to an
already occupied ownership head. Instead, the council may choose a present adult
head of the same civilization as caretaker (oldest birth date, then stable ID).
`interim_leadership` records this appointment. This is a simple toy succession rule,
not a claim about historical political legitimacy. Later estate recovery does not
automatically displace the caretaker; ordinary subsequent political turnover applies.

When no such person exists, `Civilization.leader` retains the deceased ruler's ID
as history. `living_civilization_leader` returns None; the explorer shows a vacancy.
Validation permits this only when that person heads a recorded vacant estate.
Recovery retries monthly. Abstract councils, existing budgets and policies remain
in force; their operations are not represented as acts by the deceased ruler.
Expeditions receive no deceased ruler's personal skill bonus, faction elections
exclude dead heads, and a war named during interregnum uses the origin settlement
instead of a deceased leader as that naming source.

This does not add constitutional offices, regency disputes or estate courts. It
removes another forced identity source while retaining the existing aggregate
population authority. Later increments add [complete rosters and roster-backed relocation](resident-rosters.md).
[Resident payroll](resident-payroll-balance.md) now excludes empty estates from
municipal wages in new histories; retained accounts can still serve dependents.
The older aggregate-entitlement description above describes the original vacancy
implementation, not the newer roster-based payroll eligibility.

[Verification](estate-vacancy-verification.md) records controlled boundary checks,
financial transfers, continuation, seed runs and remaining unrelated test failures.
