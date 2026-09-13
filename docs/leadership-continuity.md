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
incumbent. It is **not yet personal performance accountability**: administration,
relief and promises still need dated officeholder attribution before they can
fairly distinguish avoidable failure from unavailable resources.

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

Remaining extensions are personally attributed service accountability, institutional
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
