# Regional places reference the history's surviving objects

Regional surveys now include the existing settlements and artifacts in their parent
planet cells. Site snapshots retain actual stocks, assets, lifecycle state and dates;
object snapshots retain material composition, ownership, custody, loss/destruction
state and provenance references. No second artifact inventory or procedural treasure
roll is introduced. Older regional files load with empty historical collections.

The regional inspector shows the sites and objects associated with the selected
parent cell. It explicitly does not place them in invented buildings or excavation
voxels. For an occupied site with an eligible resident, a lost surviving object can
be selected for recovery. Headless callers use
`Generator::request_local_recovery(&survey, site, person, artifact)`.

Requests validate the world's seed, both environmental clocks and history month,
plus the current artifact record. A stale survey cannot reintroduce a sold, moved,
recovered or destroyed object. There is at most one pending request per site and
per object. The archive stores the request and its causal event.

At a quarterly cultural decision, the selected resident must still be alive and
local, the settlement occupied, and the object lost at the same place. Invalid
requests are cancelled with an event. Insufficient reserved work leaves the request
pending. A successful recovery occupies the site's existing cultural action budget,
which was already withheld from GPU craft work; another cultural action cannot use
that budget again. There is no new automatic labor allowance.

Recovery updates the canonical object's custody and accessible site. Ownership and
competing claims remain unchanged. The event links both the prior object history and
the player's request. A fresh regional survey reflects that result; subsequent
manuscript access, dedication, inheritance or sale uses the same existing object.
This provides a local-to-history action, not a separate local simulation ledger.

The controlled GPU fixture exercises actual abandonment loss, survey/request,
insufficient work, cancellation after a resident's death, duplicate/stale rejection,
and exact save/resume versus monthly continuation. It verifies material composition,
claims, ownership and economic conservation after recovery.

Remaining scope includes unoccupied-site archaeological expeditions, exact building
locations, excavation geometry, dated sediment/burial layers and an active local
simulation clock. Recovering a listed historical object is not a claim that its
precise archaeological location has been simulated.

[Artifact retention policy](evidence/README.md)
include all nine cultural fixtures and the ordinary suite. The regional UI was
compiled; the action and result were exercised through the shared generator API,
not a manual click-through test.
