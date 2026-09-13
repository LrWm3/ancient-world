# Aggregate municipal wages follow represented adults

Inspection of the year-50 circulation runs found both physical food shortages in
shrinking towns and purchasing exclusion in stocked towns. For example, seed 409
site 0 retained about 22,155 food-equivalent kg but its poorest observed household
ended with zero cash and roughly 13% unmet food need. This endpoint does not prove
a lifetime cause, but it identifies an access problem alongside physical scarcity.

Municipal aggregate wages previously used occupation weights per household,
without multiplying by represented adult members. Food need already used an
age-weighted projection of the settlement cohorts onto household membership. Large
families could therefore have greater needs without greater aggregate earning
weight even when they contained more adults.

With resident payroll enabled, aggregate sector weights now include the household's
projected adult share. The projection anchors known adult members, scales them down
if sparse identities exceed the cohort, and distributes unrepresented adults
uniformly, matching the existing food-need convention. It does not add population
or increase the payroll pool. Complete-roster households must have an actual adult;
legacy vacant-household eligibility remains restricted. Children and elders do not
become wage workers merely because they consume food.

Named farming, extraction and construction earnings override the aggregate weights
as before. Private workshop and vessel wages remain separately paid and their work
is removed from municipal payroll. Common provisions, relief and property dividends
are unchanged. Old archives with resident payroll disabled retain their prior
allocation; no new persistent fields or currency sources are introduced.

This remains an aggregate distribution rule. It does not turn every adult into an
explicit employee or guarantee that wages meet each household's dependent needs.
Household health, available assignments and named earnings are distinct mechanisms.

## Controlled verification

The numerical fixture assigns three existing adults as two, one and zero across
three households. A 90-unit farming payroll becomes 60, 30 and zero, preserving the
budget exactly. Sparse and overrepresented membership fixtures preserve the adult
cohort at six, 1.5 and zero adults respectively. Unknown aggregate adults remain
explicitly shared; this is not invented genealogical detail.

All 13 CPU household-economy tests pass. Three hardware retail tests pass (3.38 s),
covering wallet/common-food reconciliation, occupational income/access with
serialization, and founding transition/resume. The funded-family-food health and
next-month-capacity test also passes (2.86 s). Other ignored GPU tests were not
included in this focused run.

## Matched 50-year comparison

Seeds 1024, 256 and 409 reuse the native arguments and frozen 32/32 founding
checkpoints from `output/quote-ownership-screen`. Only executable and export paths
change. Credit/issuance remain off; adaptive prices, networked trade, procurement
at 0.25, demand/contract staffing, inheritance, named office service and abandoned
stock recovery remain on. Local outputs are under ignored
`output/adult-payroll-screen`. The first run overlaps compilation; timings are not
isolated benchmarks.

### Long-run failure exposed during the screen

The initial seed-1024 run stopped at month 444 with an inherited household's cash
at -2.7755575615628914e-17 and a tiny negative food purchase. Account validation
correctly rejected it. Dividend allocation computed each proportional payment
without bounding it by the remaining budget; cumulative floating-point rounding
could leave a negative final payment, particularly visible for an empty estate.
Each proportional dividend is now capped at the remaining balance. A numerical
regression reproduces a negative remainder under the former arithmetic and checks
nonnegative bounded payments under the correction. Validation remains strict;
negative balances are not silently clamped after settlement. Error messages now
identify the account, month and invalid state to make future failures actionable.

After the dividend correction, all 14 CPU household tests and the three hardware
retail tests pass again (3.42 s for the hardware group). The failed initial run is
not included as a completed balance result; the screen is rerun from its original
founding checkpoints.

## Completed results

All three corrected runs complete. Native build and strict all-target Clippy pass.
The final source additionally changes a test assertion to the equivalent range
form required by Clippy and relocates a comment; benchmark production behavior is
unchanged by those edits. Maximum absolute relative cash residual is 8.05e-8.

| Seed | Population before → after | Completed operator work before → after | Operating margin before → after | Terminal need-weighted hunger before → after |
| --- | ---: | ---: | ---: | ---: |
| 1024 | 160.980 → 160.170 | 11.060 → 11.060 | 48.342 → 48.342 | 0.0356 → 0.0371 |
| 256 | 314.377 → 335.503 | 3.809 → 3.809 | 15.860 → 15.860 | 0.0533 → 0.0374 |
| 409 | 311.528 → 347.072 | 49.862 → 49.216 | 188.416 → 190.581 | 0.0840 → 0.0702 |

Two seeds improve substantially in population and terminal food access without
new issuance or higher total payroll allowances. Seed 1024 changes little and is
slightly worse. Operator margins remain positive and completion remains above
99.6% of paid attendance. The comparison includes the dividend rounding correction;
its identified effect is preventing an invalid tiny negative remainder, not a
material redistribution subsidy.

These results support retaining the adult-based aggregate distribution, but do
not establish that all dependent households can afford sufficient food or that
physical food supply reaches declining towns. Terminal hunger is not cumulative
lifetime deprivation. The broader circulation/recovery objective remains open;
next examine purchases or pooled procurement by cash-holding households in towns
whose operating account cannot fund food imports, subject to real routes and
source stock rather than inventing food or confiscating unrelated savings.
