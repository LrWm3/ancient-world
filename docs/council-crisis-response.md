# Council response to an administrative crisis

The first joint century diagnostics found 40 crises and 40 secessions across three
baseline seeds (17, 81, 256), with no recovery events. This exposed a missing decision:
a ruler could lose control through sustained pressure, but the available autonomy
policy could only be changed manually. This is a mechanism gap, not evidence that a
particular empirical secession rate should be fitted.

Newly enabled governance now permits councils to negotiate local autonomy. At a
quarterly decision boundary, an occupied town with 3–11 consecutive crisis months
can receive 75% autonomy. Grower and merchant councils accept this concession;
retainer councils accept only after at least six consecutive underfunded payroll
months. Governing interests already exist in the faction model. These preferences
and thresholds are explicit game assumptions, not universal historical claims.

This sets the existing autonomy policy and records `autonomy_negotiated` linked to
the preceding crisis. It does not reset hunger, loyalty, unrest or cash. On the next
monthly governance update, the existing autonomy rule can interrupt the crisis,
recording recovery linked to the concession. Actual future tax collection falls to
43.75% of the standard rate, and payroll requirements fall to 62.5%; the existing
fiscal mechanisms apply those changes. A well-funded retainer council can still
refuse and lose the town. Existing already autonomous and domestic sites cannot
enter this foreign-control crisis.

`Generator::set_negotiated_autonomy(bool)` changes the policy at a consistent world
boundary. The inspector exposes a checkbox; history evaluation accepts
`--no-negotiated-autonomy` for a controlled comparison. Old archives without the
new optional field retain manual-only autonomy. New governance uses the response
by default. Disabling responses does not revoke already granted autonomy.

The existing controlled hardship fixture now compares willing councils, funded
retainer holdouts, disabled policy and manual autonomy. It verifies retained versus
lost control, causal links, unchanged stocks, cash conservation, legacy field
migration and identical serialized continuation. Integrated before/after outcomes
are reported separately; no population or war quota is imposed.

The completed ensemble and held-out results are in
[integrated calibration](integrated-calibration.md#september-2026-results). Concessions
resolved every sampled crisis; this is a remaining political-balance question, not
evidence that hardship disappeared.
