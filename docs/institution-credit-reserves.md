# Institutional credit reserve policies

The first institutional lender experiment inherited the council reserve floor of
100 money units. It made no loans in a six-arm seed-1024 comparison. Institutions
operate on a different scale, so a separate policy now makes that assumption
explicit without changing council reserves or borrower underwriting.

- `CouncilFloor` retains the greater of the existing council cash floor and one
  year of the institution's administration and building-repair quote. This remains
  the default and the migration behavior for archives without the new field.
- `AnnualOperatingCosts` protects the annual quote itself. Existing materials,
  local replacement prices and wear still determine building costs. The policy
  does not treat all treasury cash as surplus or count expected donations.

Both policies offer the same 25% default share of cash **above** the chosen reserve.
Actual transfers still pass shared lender, borrower and repayment-source caps.
Selection is a policy experiment, not a claim that one annual budget is empirically
sufficient. Unplanned spending, damage, losses and future prices can exhaust it.

The native switch is `--institution-credit-operating-reserve[=true|false]`, with
`--council-credit --institution-credit-lenders` required for these offers to run.
Omission preserves the archived policy. False selects `CouncilFloor`; true selects
`AnnualOperatingCosts`. Loans already made retain their terms when this changes.

Use `scripts/monetary_experiment.py --compare-institution-reserves` with the usual
checkpoint, years and ignored output arguments. It generates eight arms: four
baseline credit/issuance combinations, two with institutional lenders and the fixed
floor, and two with institutional lenders and operating-cost reserves. All controls
explicitly select the appropriate policy; enabling both institutional comparison
flags does not duplicate arms.

Judge this by financed work and borrower/lender outcomes, not loan counts alone.
Measure institutional upkeep, repairs, closures, unpaid service, repayments,
defaults, cash and money residuals. A lower reserve that produces loans but damages
institutional continuity is not automatically an improvement.

Controlled verification passed: the GPU fixture compares the same 25-unit
institution under both policies, checks bounded actual principal and retained
operating cash, and verifies accounting and checkpoint replay. Existing exclusion
cases for leadership, building access and failed repayment capacity remain in that
fixture. The CLI policy test, 19 active market tests, nine Python reporting tests,
and strict all-target Clippy also passed. The [eight-arm, 200-year comparison](institution-credit-reserve-evaluation.md)
completed: operating reserves enabled more requests but no loans, because net
repayment capacity remained zero. The fixture does not establish a balance benefit.
