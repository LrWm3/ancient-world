# Expanded political interests

This is a small game-rule expansion, not a model of historical party formation.
Each civilization has nine available interest groups. Some can remain unsupported.
They do not yet generate their own ideological programs, alliances or organizations.

| Interest | Appeal or organizing condition | Council tax rate |
|---|---|---:|
| Growers | Household food access and remembered town food pressure | 2% |
| Merchants | Recorded sales relative to starting treasury | 3% |
| Retainers | Active war | 6% |
| Artisans | Share of completed household wages from crafts and ownership inequality | 3.5% |
| Scholars | A household head's curiosity, reduced by hunger | 4% |
| Congregations | A household head's piety | 2.5% |
| Bread leagues | Hunger and inequality | 1% |
| Revivalists | Piety combined with hunger/disruption | 5.5% |
| Warbands | War and disruption | 8% |

The names, response weights and tax preferences are intentional game choices.
Scholar and congregation membership is not equivalent to belonging to a scholarly
or religious institution. Cumulative sales are a rough commercial-activity proxy,
not a measure of monthly profitability.

## Membership and instability

Once a year, one of three rotating household cohorts reconsiders allegiance. It
scores all interests using local conditions, the head's traits and faction cohesion.
Existing membership receives a small inertia bonus. Allegiance, voting, organizers
and council nominees use the same eligibility rule: an occupied household at a
non-abandoned site, a living head not on service, and no active relocation or recorded
loss. Historical estates retain ownership but do not vote while vacant. Membership changes select an existing faction; they do not create
population, money or new named organizations.

Bread leagues, revivalists and warbands deliberately depend on sustained organizing
pressure. Their cohesion increases by `0.35 × pressure` and declines by
`0.18 × (1-pressure)` per annual update, bounded to 0.05–1. Revivalists and warbands
also lose 0.4 when their selected organizer changes. Organizers are chosen from eligible faction supporters. An incumbent receives a
0.5 continuity bonus, so a minor change in appeal does not automatically replace
them. An unsupported movement has no organizer; pressure can still attract its
first members before the next annual organizer selection.
When cohesion falls below 0.25, remaining households can reconsider immediately.
A supported movement crossing below 0.6 records `faction_fragmentation`, before
its remaining members have all dispersed.
This means loss of cohesion and supporters, not the creation of named splinter parties.
The faction record persists and can regain support if its conditions recur.

Annual voting weights retain household shares and resident population. Cohesion
reduces the effective voting weight of fragmented movements. The winning faction
still needs a five-point support lead and an eligible household head to supply its
leader. Government identity changes only when that nominee exists.
Council turnover changes the existing tax policy. Retainers, revivalists and warbands
also resist negotiated autonomy until administrative payroll has failed long enough.
No additional unbudgeted violence, relief spending or religious conversion is created.

## Compatibility and inspection

Version-one political records expand at the next political preparation boundary.
New factions are appended; existing IDs, memberships and governing factions remain
intact. Expansion receives a dated event, without fabricated earlier supporters.
Migration looks up the destination civilization's matching interest instead of
assuming that IDs repeat in groups of three. The explorer shows all faction names,
support, dissent, cohesion and local affiliation shares. Social summaries retain
the original three slots and add six archive-defaulted shares.

```sh
cargo test --lib faction_interests
cargo test --lib expanded_faction_tests -- --ignored --nocapture
cargo test --test politics -- --ignored --test-threads=1
```

Fixtures exercise crisis appeal, cohesion decay, organizer-loss sensitivity,
legacy-ID preservation, household realignment, tax consequences and recovery.
Results demonstrate these rules working, not a scientifically grounded prediction
of politics. Long-history outcomes and balance can change from earlier revisions.

## Initial checks, 2026-09-10

On Quadro RTX 5000/Vulkan, the controlled pressure/recovery test passed, including
version-one ID preservation, bread-league council control and its 1% tax policy,
fragmentation during recovery, and serialized continuation. Four existing politics
GPU tests passed, including a century of genealogy and subsequent checkpoint/monthly
continuation. The ordinary library run passed 37 tests with 46 hardware tests skipped;
all-target Clippy and formatting passed.

Three separate 30-year runs used seeds 17, 81 and 256, terrain 64, ecology 16, eight
founding groups and society/politics enabled. They produced these final household
memberships (all households, including any travelers):

| Seed | Growers | Merchants | Scholars | Congregations | Council shifts | Fragmentations |
|---|---:|---:|---:|---:|---:|---:|
| 17 | 24 | 36 | 29 | 31 | 13 | 0 |
| 81 | 12 | 60 | 17 | 31 | 13 | 0 |
| 256 | 8 | 51 | 22 | 39 | 17 | 0 |

Other interests had no final memberships in these runs. Thus this is evidence of
some additional diversity, not evidence that all nine groups routinely matter.
The crisis fixture demonstrates instability under imposed pressure; these seed
runs did not produce supported movements fragmenting organically. Artisans and
military interests may need further tuning for their intended settings. No faction
or fragmentation quota is enforced.

Reproduce the seed report with
`cargo test --test politics expanded_interests_seed_comparison -- --ignored --nocapture`.


## Household access and political accountability, September 2026

Annual Respond reads retail outcomes completed in Execute/settle, and town pressure
from the preceding Close. When a household has a current-month food observation
at its present site, political hunger is 75% its own unmet food fraction and 25%
remembered town food pressure. Without that observation it uses the town proxy.
This permits access-related discontent with unchanged food production or stocks.
It does not assume every shortage is a harvest failure.

Artisan appeal uses the craft share of this month's paid sector wages when the
same current household observation exists, falling back to town planned labor in
legacy/no-retail histories. Paid work is not necessarily productive work; this
is an earnings-interest proxy, not measured artisan output. Merchant appeal still
uses the bounded cumulative-sales proxy and needs a separate recent-trade measure.
Household voting remains property weighted, not one-person-one-vote.

Petitions now retain the responding faction at resolution. Honored petitions give
credit to their advocates and the responding administration; opposition refusal
or council underfunding penalizes the responsible administration, not an opposition
advocate. Invalidation through site/controller/institution loss and unavailable
delivery channels do not assign faction blame. Credit fades and only the latest
relevant local resolution counts. Legacy records do not fabricate a responsible
government. This remains a simple attribution rule: it does not model propaganda
or determine who caused a council's financial trouble.

No production or monthly phase order changed. These weights are toy-game choices,
and changed political trajectories are expected.

### Verification of the update

- Faction-filtered library suite: six tests passed, including actual allegiance
  changes under household hunger with identical town inventories, stale-observation
  control, crisis recovery, vacant/dead-head exclusion and serialized continuation.
- Civic-petition suite: eight tests passed. Controlled refusal leaves opposition
  advocate credit unchanged, assigns negative credit to the responding faction,
  and retains that attribution after turnover and serialization. Existing funded
  learning, autonomy, local-contact and finite-money tests remain passing.
- The first food-access fixture failed because newly founded histories had no
  initialized retail account rows; it was corrected to establish explicit completed
  observations before the intervention. This was a fixture correction, not tuning
  the political threshold to force an outcome.
- All five hardware politics integration tests passed, including seeds 17, 81
  and 256 for 30 years, a 100-year genealogy/politics history followed by 24 months
  of batch-versus-checkpoint continuation, and finite territorial campaigns.
  Terrain resolution was 64 and ecology resolution 16. This pass checks viability
  and invariants; it does not recalibrate faction prevalence or assert ideological
  realism.
- Formatting, all-target Clippy with warnings denied and repository artifact checks
  passed.

Reproduce with:
`cargo test --lib faction -- --include-ignored --test-threads=1`,
`cargo test --lib civic_petitions -- --include-ignored --test-threads=1`,
and `cargo test --test politics -- --include-ignored --test-threads=1`.
These suites overlap; their counts should not be added as unique test counts.
Generated logs remain local under ignored `output/`.
