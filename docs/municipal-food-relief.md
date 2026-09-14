# Municipal purchasing assistance experiment

Enable with `--enable-system municipal-food-relief`; disable with the corresponding
`--disable-system`. Registry defaults and older archives leave it off. Society and
household accounts are required. It does not enable solidarity, council welfare,
credit, common-food entitlement or nutrient overrides automatically.

The purchasing follow-up found towns with substantial operating cash beside
unfunded household dietary needs. This experiment redistributes existing town
cash rather than adding food, currency or household debt.

## Monthly boundary and limits

At the end of Reserve's household retail preparation, after council relief and
household solidarity, gather remaining private food-purchasing gaps locally.
Protect ten currency units per resident of current town cash; at most 5% of any
surplus is available this month. The protected amount is a game working-capital
allowance, not a claim to fully cost every future service or construction project.

Cap the transfer by both the remaining purchasing gap and food already present
at that boundary after accounting for existing entitlements. Newly produced food
and later shipments are not pledged. Share actual withdrawal among households in
proportion to their gaps; no ownership weighting applies. Each household receives
cash and the existing relief income counter records its source as relief, with
municipal source details in the new dated receipts. The town account loses exactly
the representable amount credited. Ordinary purchasing then consumes actual food
and pays the town; this can circulate the same money rather than destroy it.

The existing GPU entitlement upload and subsequent retail settlement remain in
place. Receipt fields retain opening cash, protected cash, requested assistance,
food backing, allowance and payment. The policy's monthly guard prevents repeated
payment at the same boundary; archived state retains that guard. Disabling stops
future payments and preserves previous receipts. Earlier reservations retain their
priority; this is not a new scheduler or a guaranteed institutional funding pool.

Evaluate household access, physical scarcity, population trends, municipal cash,
and service readiness together. A rise in population alone is insufficient to
establish a balanced policy. Lending remains a separate potential experiment:
recurring underpayment should not automatically become subsistence debt.

## Reproduction

Use the [growth investigation wrapper](../scripts/run_growth_investigation.py)
with terrain/ecology 32/32, one geological epoch, five civilizations, seeds 1024
and 409, and 200 years. Retain the previous purchasing-solidarity bundle:

```text
--enable-system demographic-audit,household-estate-inheritance,household-estate-reclamation,household-wealth-tax,council-welfare-reserves,named-office-service,food-solidarity
```

Add `--enable-system municipal-food-relief` for the treatment. Explicit nutrient
settings remain retention 0.95 and geological release 5e-7; the wrapper supplies
these. No needs-based-food, population-cap, harvest, demographic or founding
threshold changes accompany this policy. Use fresh worlds and distinct ignored
history export paths. The history summary script reports the latest municipal
payment boundary separately from lifetime tax and income totals.

## First 200-year comparison

| Seed | Population without / with municipal relief | Active towns without / with | Final-decade births / deaths with relief | Access gap without / with | Physical gap without / with |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 887.5 / 1,185.5 | 5 / 7 | 271.70 / 237.53 | 1.229% / 1.176% | 0.364% / 0.141% |
| 409 | 979.0 / 1,219.1 | 4 / 10 | 280.20 / 243.39 | 1.236% / 1.090% | 0.251% / 0.232% |

Food gaps are demand-weighted across 200 years, not final-month gaps. Treatment
histories finish with eight/eleven total sites, including one abandoned site in
each seed. Final-50-year births exceed deaths by 85.12 / 48.43 as well. This is
positive late growth and settlement expansion under ordinary purchasing, without
a common-food entitlement override. It is not a promise of continued growth.

At the last relief boundary, municipal requests are 859.08 / 1,110.10 and actual
payments are 605.10 / 813.66. Most remaining food-backed needs exceed the 5% cash
allowance; the trace does not show a general lack of food backing. These receipts
are monthly transfers, not additional money or lifetime relief totals.

Town cash ends at 34,647 / 36,061, compared with 28,669 / 32,067 in the solidarity
controls. Payment did not deplete town accounts in these worlds: subsequent food
purchases return money and larger populations change turnover. Long-run differences
are nonlinear and do not identify a unique causal path from every grant to a birth.
The direct transfer fixtures and actual dated receipts establish the immediate
cash-to-purchasing connection; the matched worlds evaluate its broader consequences.

Institutional health remains poor: only one institution is operational in each
treatment, out of sixteen/twenty marked active. Controls had one/two operational.
Cash on hand is therefore not sufficient evidence of good public services. Keep
this policy opt-in; subsequent service comparisons must inspect actual work,
usable space, membership and paid upkeep rather than declare the economy solved.

The three focused CPU fixtures cover finite cash/food constraints, existing
entitlements, zero surplus, fractional cash rounding, household ordering, monthly
duplicate execution, disabling, archive continuation and legacy defaults. The
ordinary library suite passes 216 tests, with 157 hardware/long fixtures ignored.
The native binary builds and library/binary Clippy passes with warnings denied.
The two treatment runs complete normal native validation at monthly boundaries.
Treatment elapsed times are 151.0 / 143.9 seconds, overlapping compilation and
each other; these are not isolated performance benchmarks.

Disabled controls were then regenerated on the new executable (122.9 / 117.4 s).
For both seeds, the entire JSON history exactly matches the prior solidarity
history after removing the newly introduced, disabled `municipal_relief` field.
This is stronger than endpoint agreement for the disabled path. It does not claim
cross-GPU identity or whole-generator checkpoint equivalence with the policy on;
the focused continuation test covers the new transfer state itself.
