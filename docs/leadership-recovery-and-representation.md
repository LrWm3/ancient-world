# Leadership recovery and political representation

These are bounded toy-game policies. They extend existing work reservations,
mandates and faction reviews; they do not add a new scheduler or election system.

## Institutional recovery

An active institution with a vacant mandate and no eligible local members can seek
one qualified local adult after six months of vacancy. Religious institutions
require matching affiliation. Relevant competence must reach 0.35; the highest
competence wins the invitation, with stable person-ID tie breaking. This is an
abstract willingness/qualification rule, not a personal recruitment negotiation.

Reserve proposes the recruit through the existing election request. No membership,
money or authority changes then. Respond rechecks local presence, eligibility,
affordability and the named commitment before spending the 0.05 worker-month
interview/selection grant. A successful recovery transfers approximately five money
from institutional treasury to the town account, using the receiver's representable
increment, and admits the recruit before selecting the new holder. It records a
recruitment event linked into the mandate's succession history.

No money, labor or available local adult means no recovery. Treasury loss or absence
between reservation and execution invalidates the proposal. Repeating the monthly
observation cannot recruit or pay twice. The selected holder does not bypass building
condition, readiness or later administrative work requirements.

Existing annual recruitment and funded institutional relocation remain available.
This addition neither teleports absent members nor automatically moves institutions.
Inactive institutions, temporary deputy powers and paid return journeys are outside
this increment. Relocation still requires an open route, funding and members already
present at the destination.

## Configurable political weight

`History::set_political_weighting(civilization, policy)` sets a civilization's
`leadership::PoliticalWeighting`. Call it after reserved participation is settled.
It records a policy-change event; subsequent annual faction and leadership reviews
use it without redoing past elections. The policy persists inside the civilization's
leadership mandate. No explorer dropdown is added in this increment.

| Policy | Household's base political weight |
|---|---|
| Legacy (default) | Faction mobilization: ownership share × town population; council election: one |
| Property | Share of the home town's private property |
| Household | One per eligible household |
| Resident | Count of recorded present residents represented by that household |

Resident representation includes dependents through their eligible head; it does
not give each person a separately simulated preference. Travelers, dead people and
future births are excluded. Sparse historical populations remain sparse: this mode
does not infer unnamed voters. Property shares are local shares, not cross-town
market-valued wealth. These limits are deliberate and visible in the reported weights.

Existing household eligibility still applies. Council candidates additionally need
adult local political eligibility. Faction support still multiplies base weights by
urgency and cohesion; those represent mobilization, not extra ballots in a council
election. Candidate preferences, heritage, accountability and succession rules are
unchanged. Property inheritance remains separate from political authority.

`History::political_weights(council)` exposes household-indexed weights.
`leadership_report()` includes both faction and council base vectors alongside each
civilization's policy and candidate totals. Compute the faction vector before taking
mutable politics state so kinship-derived resident membership is still available.

## Verification (2026-09-12)

- Three institutional succession tests pass, including a GPU fixture with funded
  recruitment, no treasury, insufficient work, and departure between reservation
  and execution. Successful recruitment preserves combined institutional/town
  money and all cases settle through the existing participation receipts.
- Four leadership tests pass, including GPU tests comparing the same electorate
  under all three explicit policies, checking published weights against ballot
  totals, unchanged estates, and serialized continuation equivalence. A controlled
  two-household fixture reverses relative weight between property and resident
  representation while household representation ties them.
- The expanded faction crisis/membership GPU regression passes.
- Ordinary library tests: 149 passed, 130 ignored. All-target Clippy passes with
  warnings denied. These tests establish bounded behavior and policy plumbing;
  long-run faction balance and institutional survival have not been recalibrated.
