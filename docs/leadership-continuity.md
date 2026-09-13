# Political leadership continuity

This first pass covers explicit succession rules, replacement within a governing
faction, and the political influence of locally known heritage recoveries.
It does not model universal historical constitutional behavior.

New political baselines use `GoverningFaction`. Old archives without the new
leadership state use `Hereditary`; this is an explicit compatibility baseline,
not a reconstruction of their constitutional history. Set a civilization's rule
with `Generator::set_succession_rule` at a completed boundary. Changes are recorded
and affect subsequent decisions; they do not transfer property or force an
immediate election.

- **Hereditary:** on death, the oldest eligible recorded child succeeds. Without
  a present eligible child, a governing-faction representative becomes caretaker.
  No relatives are invented. This first version does not later displace the
  caretaker automatically when a previously unavailable child returns.
- **GoverningFaction:** present adult household representatives in the governing
  faction are eligible. Death, six months of unavailable residence, or annual
  internal competition can change the ruler without changing the faction.
- **CouncilElection:** each eligible household representative casts one ballot,
  preferring affiliation and then candidate traits/skills and witnessed heritage.
  Candidates span factions. This household ballot applies to personal selection;
  the existing inter-faction property/urgency support calculation remains separate.

Candidates require age 18 or older, life, local residence, an active settlement
and political access in their own civilization. Estates, travelers and absent
representatives do not supply candidates. If none qualify, the vacancy is recorded
without fabricating a leader or transferring a household's goods. The existing
leader ID remains a historical reference; consumers must use living/present
eligibility rather than assuming it identifies an available officeholder.

Monthly society Respond settles estates first, then reviews political vacancies
and absence. Annual politics updates governing factions, then performs personal
selection. Review clocks prevent duplicate decisions at the same boundary.
Household inheritance no longer conveys the political title when politics is
enabled. Non-political legacy history keeps its existing representative behavior.

Internal selection uses the existing ambition, loyalty, skill and locally known
heritage score, with a challenger margin to prevent ties from removing an
incumbent. Personal accountability now adds two local, dated signals:
completed named office work, and civic petitions answered by a present ruler.
A response records that person at resolution, not whoever holds office later.
Delivered petitions earn credit; political refusal incurs blame only if funds
and the delivery channel were available. Unfunded or impossible responses remain
neutral for personal selection (existing faction grievance rules are unchanged).
Relief here means delivered purchasing power, not a claim that food was purchased.

Petition effects decay exponentially with a 60-month scale and saturate at ±0.5.
Completed named office work accumulates bounded credit up to 0.30 with the same
decay scale. Scarce labor, illness, and failed eligibility do not earn work
credit, but do not incur blame. Office credit is retained per person/site while
named office service remains enabled. Both signals are local to the candidate's
site; no global reputation or transmission is inferred. Hereditary rulers remain
exempt from annual performance contests. Candidate quality can now be negative;
council ballot counts remain nonnegative.

These are game rules, not calibrated political psychology. They do not cover
every relief shipment, treaty obligation, or administrative failure. Older archives
have unknown personal attribution and start without historical service credit.

Heritage recognition adds at most 0.20 to faction appeal, before the existing
bounded affiliation update. It uses current eligible member households, known
local witnesses, the existing aging function, and saturation. A recovery shared
by multiple members is counted once. Recognition cannot add population, money,
property or votes directly. Unknown and remote recoveries have no local effect.
Annual recognition inputs use opening membership to avoid earlier household
switches changing the evidence available to later households in the same pass.

`History::leadership_report` exposes rules, vacancy/absence clocks and latest
candidate scores (ballot counts for council selection). State persists in ordinary
history archives. Political succession events identify both people and the
retained governing faction. The existing office synchronization then follows the
new ruler; this change does not add free administrative labor.

Remaining extensions are broader personally attributed commitments, institutional
acting leaders/return recruitment, configurable franchise for inter-faction
support, and a viewer control for these constitutional policies. Existing funded
institution relocation remains available independently.

## Verification

- Ordinary library suite: 146 passed (128 extended/hardware cases ignored).
- Market integration target: 6 passed, 2 hardware cases ignored; explicit legacy
  political fixtures initialize the new optional leadership state.
- Explicit GPU runs: 3 new leadership cases, 7 resident-registry cases, the expanded
  faction case and the occupation case all passed (12 total).
- New comparisons cover identical estates under hereditary/faction rules, internal
  replacement with the faction unchanged, absence grace and restoration, one
  household/one ballot accounting, unknown/remote/aging recognition, dead-member
  exclusion, repeated-boundary idempotence and serialized continuation.
- Existing registry tests retain identity-slot and population checks. Their obsolete
  assumptions that estate heirs must become rulers were replaced with independent
  property/title assertions; an estate heir may now be recruited if not ruler,
  but recruitment still cannot identify an extra person from an exhausted slot.
- All-target Clippy with warnings denied, repository artifact policy and whitespace
  checks passed. Raw output is ignored under `output/leadership-*`.

These are controlled integration checks, not long-run balance calibration. Personal
selection remains a small game rule; no claim of historically calibrated political
behavior is made.

### Personal accountability follow-up

Controlled GPU fixture (seed 17): a ruler with delivered-petition credit retained
office against a challenger with a 0.4 skill advantage. Removing only the personal
attribution let the challenger win; household/property state remained identical.
Funded opposition earned negative credit; the same opposition with no treasury
funds remained neutral. Other people and remote sites received no credit.
Changing ruler did not transfer responsibility, and serialized continuation,
repeated resolution and aging preserved the expected scores.

Named office service tests verify positive credit for completed work, no extra
credit on repeated settlement, no credit for unavailable personal time or exhausted
town labor, and identical save/resume outcomes. Production and finance accounting
are unchanged.

Verification: 146 ordinary library tests passed (129 extended/hardware cases
ignored); all 9 civic-petition tests passed with ignored cases enabled; 3 leadership
fixtures and the named-office service boundary fixture passed. These controlled
comparisons establish the new causal connection, not long-history balance.
