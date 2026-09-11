# Civic petitions and public reputation

This is an additional game loop, not an attempt to simulate political science.
It connects existing household factions, institutions, public finance, religious
accounts and naming. Numerical thresholds remain explicit design choices.

## Representation

Quarterly cultural work can fund one institution-backed petition per town.
The institution must be operational, have a living leader, and include a
resident household head belonging to the represented faction. The petition
costs 0.1 of the same cultural worker-month budget used by other activities.

Growers, congregations and bread leagues request relief; artisans and scholars
request learning support; merchants and revivalists can request more local
autonomy. Hunger favors relief, scholarly/craft institutions favor learning
when hunger is low, and unrest under centralized administration favors autonomy.
Hospitality and reciprocity themes modestly favor a religious sponsor's relief
petition. Retainers and warbands do not sponsor these particular demands.

Only the strongest qualifying local request opens. A town has at most one open
petition, with a five-year cooldown between openings. This is deliberately a
small agenda, not a complete ideology or legislative simulator.

## Responses and real costs

The controller at submission remains the addressee. Earliest response is three
months later. Government interests influence willingness; severe recorded
hardship can persuade a government outside the requesting faction.
An unresolved request closes after twelve months, or earlier after three months
if its settlement, sponsor or addressed administration becomes unavailable.

* Relief transfers 5–100 abstract currency units from the council to resident
  household wallets, according to town population. Equal cash entitlements use
  existing retail purchases later; they do not create grain or guarantee food.
* Learning transfers the same bounded sum to the sponsoring institution.
  Its existing upkeep and activity systems determine subsequent use; a grant
  does not automatically discover knowledge.
* Autonomy increases the site's autonomy by 0.15, capped at 0.85. Existing tax
  collection uses the changed value. This respects the negotiated-autonomy
  option and never reduces autonomy already granted. There is no invented fiscal transfer.

Insufficient funds never produce a partial or fictional payment. Requests can
wait for finances or political willingness to change. They are requests, not
contracts promising a specific delivery date.

## Memory, religion, diplomacy and names

Honoring or failing to deliver a request changes local administrative loyalty
slightly. The latest resolved request for each represented interest provides a
bounded positive or negative contribution to subsequent household faction
choice, fading with age. Credit does not transfer to a different controller.
This models perceived effectiveness of representatives, not a universal theory
of voter behavior.

Religious sponsors record a separate attributed account tied to the factual
petition and response. Affiliation and doctrine are not overwritten. An
institution's successful civic work supplies a naming association for gift,
learning or home for ten years, alongside existing lexical candidates.
Existing names remain unchanged; adopted names retain causal event references.

Separately, witnessed cross-border food deliveries improve diplomatic trust.
They do not count as commercial contacts, bypass war, or automatically create a
treaty. This gives existing relief another consequence beyond local survival.

## Persistence and inspection

Petitions archive opening pressure, sponsor, represented faction, controller,
requested and paid amounts, dates and causal events. Older archives initialize
with no invented petitions. The governance inspector lists recent requests and
outcomes. Sparse CPU records use the same monthly history boundary.

## Limits

No coalition negotiations, legislative votes, earmarked grant accounts, new
religious authorities or linguistic demographics were added. Priorities,
grants, delays and reputation weights need further playtesting. A successful
grant is evidence of a transfer, not evidence that hunger or ignorance ended.


## Verification

Controlled 64/16 terrain/ecology fixtures use seeds 7 and 17. They exercise
paid representation, the open-request limit, delayed relief transfers,
learning grants, autonomy, insufficient funds, fading credibility, attributed
religious accounts, event validity, old archive defaults and continuation of
pending requests. Cash across councils, households and institutions remains
constant to 1e-8 in the controlled transfers; food inventories are unchanged by
cash grants. Delivered aid increases trust exactly once in the controlled
comparison; lost shipments do not, and neither creates commercial contacts.
The ordinary library suite passed 59 tests (58 GPU tests remain ignored in
that ordinary invocation); the selected GPU fixtures above were run separately.
The final naming regression run passed nine CPU tests, with three GPU naming
cases left unselected.

The existing GPU governance suite passed all three cases: treaty expiry and
checkpoint/batch continuation, unfunded occupation versus devolved autonomy,
and delivered trade building trust and agreements. The first run identified
that the common event validator did not recognize faction subjects; support
was added rather than dropping the new causal references.

These are verification and integration fixtures, not ensemble calibration of
petition frequency or political balance. Test outputs stay in ignored output/.

Reproduce with:

```sh
CARGO_INCREMENTAL=0 cargo test --lib representation_delivery -- --ignored
CARGO_INCREMENTAL=0 cargo test --test governance -- --ignored
```
