# Local hardship and political consent

Governance now reads existing social observations as well as current aggregate food
shortage. A stocked town can lose consent when households cannot obtain its food;
restocking alone does not erase remembered deprivation. Housing repair, household
relief and recovery from displacement can improve governance through their effects on
these observations. They do not grant a direct loyalty bonus.

## Monthly rule and update order

All inputs are bounded fractions in [0,1]:

- `H = max(current town shortage, remembered hunger, remembered severe household shortage share)`
- `D = max(remembered disruption, remembered unsheltered share)`
- `F = wages paid / wages required` (one if no wages are required)
- `A = autonomy`, `T = council tax rate * (1 - 0.75*A)`

The existing loyalty/unrest update becomes:

```text
loyalty' = clamp(loyalty + .008*F + .006*A - .012*(1-F)
                 - .020*H - .008*D - .080*T, 0, 1)
unrest'  = clamp(unrest + .030*H + .012*D + .012*(1-F) + .100*T
                 - .012*loyalty' - .012*A, 0, 1)
```

Maxima avoid adding several measurements of the same hardship together. Disease
and ownership inequality do not enter as separate penalties here. These response
coefficients are game assumptions, not fitted historical probabilities.

Governance reads the last completed social observation. Social memory is updated
later at the monthly boundary; governance does not update or decay it again. Current
aggregate shortage remains an immediate input. Histories without social indicators
retain the former shortage-only behavior. No archive fields or monetary, population,
food or nutrient stocks are added.

Foreign control still requires twelve consecutive months below 25% loyalty and
above 65% unrest to fail, and autonomy of at least 75% prevents this specific crisis.
Domestic hardship changes loyalty and unrest but does not introduce domestic
revolutions in this change. Early improvement can interrupt a crisis; accumulated
loss of consent can make late relief insufficient.

## Evidence and inspection

Crisis notices report payroll coverage, effective tax, hunger, disruption, loyalty
and unrest instead of asserting that administration must be unpaid. A new
`governance_recovery` event records interruption of a crisis and links its preceding
cause. This means the crisis condition ended, not necessarily that hardship ended:
autonomy can interrupt a crisis while residents remain deprived. The inspector
shows current pressure inputs; these may include observations completed after the
last political update.

The controlled GPU-initialized history fixture starts an occupied town with full
payroll, zero tax and zero aggregate shortage. Four matched branches compare:

1. Secure households: retained control.
2. Sustained severe household deprivation and crowding: eventual secession.
3. Early removal of the observed hardship: consent recovery before secession.
4. Autonomy during hardship: retained nominal control with an explicit recovery event.

The observation removal is an **ablation**, not a claim that an implemented relief
program instantly clears social memory. The fixture holds observations fixed to
isolate the governance consumer; it does not establish long-run seed frequencies.
It checks measured event text, causal links, unchanged population/stock vectors,
cash conservation within 0.02 abstract currency, and exact JSON continuation during
the pre-intervention interval. Ordinary tests verify pressure selection, fallback,
no double counting and a negative control for unused disease/inequality fields.
Existing full monthly governance tests cover treaties, occupation, autonomy and
archive continuation. The GUI is compiled but not manually exercised in this pass.

Reproduce with:

```sh
mise exec rust@1.89.0 -- cargo test --lib governance::payroll_tests -- --include-ignored
mise exec rust@1.89.0 -- cargo test --test governance -- --ignored
mise exec rust@1.89.0 -- cargo test --all-targets
mise exec rust@1.89.0 -- cargo clippy --all-targets -- -D warnings
```

Captured logs and source hashes are in [Artifact retention policy](evidence/README.md).
