# Genealogy, factions and territorial warfare

Enable **Dynasties and politics** after social history in the civilization sidebar. Existing social archives remain unchanged until activated; the new baseline records unknown ancestry for existing adults. New worlds can use both flags:

```sh
mise exec rust@1.89.0 -- cargo run -- --headless --load output/refined.world --epochs 0 \
  --civilizations 8 --society --politics --history-years 100 \
  --save output/civilization-politics.world --history-export output/civilization-politics.history.json
mise exec rust@1.89.0 -- cargo run -- --load output/civilization-politics.world
```

To extend a saved social world, omit `--civilizations` and `--society`. To resume an already political world, omit `--politics` as well.

## Families and inheritance

Genealogy now stores two parent references for recorded new births, household affiliation, dated marriages, union dissolution on death, and children. Annual marriage matching pairs eligible local adults and excludes shared ancestors within three generations. Reproductive unions have a simplified age window, minimum three-year birth spacing, and at most four recorded children. These are explicit game rules, without sex-specific fertility, pregnancy, divorce or cultural marriage-law catalogs.

Named births consume credit from actual GPU cohort births; they do not add another resident or create food needs twice. Named figures remain annotations of the aggregate population, not a complete individual census. Deaths consume aggregate mortality credit. Catastrophic deaths and detailed migration of individual families are still coarse; this is not an age-exact registry of every inhabitant.

On a household head's death, an eligible living adult child can inherit the office and unchanged household share. A person already heading another house cannot inherit a second one. If no eligible child exists, the older representative-adult succession rule appoints an unrelated head with unknown ancestry. No fabricated parenthood is added to that appointment. Household shares remain pooled-property interests; partitioned estates and independent family budgets are future work.

The inspector lists people, parents, birth/death dates and unions. Existing cause-linked history records births, marriages, inheritances and council changes. Genealogy references are independent of the older predecessor/office field.

## Factions and control

Each civilization now has nine available interests, including three pressure-dependent movements. Household allegiance can change, cohesion affects support, and the governing interest changes tax and autonomy preferences. See [expanded political interests](faction-interests.md) for the intentionally unstable bread leagues, revivalists and warbands, migration/archival behavior, and limits. A challenger still needs a five-percentage-point lead to replace the governing council faction. Dissent remains diagnostic; faction fragmentation does not create a civil war.

Settlement **cultural affiliation** and **political administration** are separate persistent fields. Conquest changes the administrator. Annual taxes and road purchases then use that administrator's council; inhabitants, family identity, private property and inventories remain in place. Daughter villages founded after conquest inherit their parent's administration. Exiled cultural councils can persist without holding taxable settlements. Council faction turnover is implemented here; [governance](governance.md) adds legitimacy, autonomy and administrative secession. General electoral rules, minority rights, alliances and civil wars remain extensions.

Claims cover occupied cells, dry adjacent hinterland and surveyed road corridors. Overlapping claims by different administrations form disputes. This bounded regional footprint intentionally leaves unexplored land unclaimed; it is not a polygon filling each entire island. Every claimed cell must be dry central-island land, and road claims inherit the already validated connected terrain paths. The atlas overlay colors administration and marks disputed claims white; it can be switched off. The inspector shows claim counts, administrators and cultural affiliation.

## Campaigns and peace

A council can demand a neighboring settlement through the inspector or `Generator::declare_war(origin, target)`. A recent food crisis or incoming raid can also trigger autonomous territorial escalation. Demands require a contested corridor, an open surveyed land route under 1,500 weighted travel km, different administrators, and no active war or ten-year truce between the parties.

Mobilization removes actual working adults, food and tools from the origin. Armies are capped by adult manpower and equipment, leave three months of civilian food behind, and carry provisions for outward travel, return travel and a margin. Tools are a generic equipment proxy pending a weapons/armor catalog. Armies eat monthly, suffer starvation without supplies, lose real manpower in combat, and lose equipment into the material ledger. New trade and relief departures across the opposing administrations stop during war; cargo already dispatched continues.

The campaign resolves one aggregate battle at its objective. Equipped manpower is compared with the defending adult levy; success transfers administration, while defeat preserves the defender's territory and prevents food looting. Remaining supplies and equipment return with survivors after an actual return journey. Both results end the war and start a ten-year truce. If another campaign changes the origin or target administrator before arrival, the army withdraws without fighting the new owner. Battles, peace and the expedition return retain causal event links.

This is territorial warfare at the campaign level, not tactical combat: no siege engines, fortifications, reinforcement convoys, occupations requiring garrisons, army interception, naval warfare, atrocities or multi-front diplomacy yet. A campaign may not conquer the outer continent. Dense farming, production and demography remain GPU simulations; family graphs, factions, claims and campaign records run on CPU.

## Persistence and checks

Political schema version one is embedded as an optional extension in version-two archives. Parent references, marriages, memberships, councils, administrators, claims, wars, equipment and in-transit armies persist. Loading an older world does not initialize or replay these systems. Advancement and scenario commands commit only after validation.

Checks cover century-long family histories and real-child inheritance; missing/cyclic ancestry and invalid administrations; central-land claims; faction turnover; insufficient supplies and closed routes; conquest and repulse; food, population, C/N/P, water, money and goods accounting; and exact checkpoint continuation with different advancement batch sizes during campaigns. Marriage matching caches ancestry sets for each annual matching pass rather than repeatedly traversing the full family record for each potential partner.

The existing 256-site, 16-civilization, 200,000-event and 64 MiB archive-header limits remain. Recorded genealogy is additionally capped at 50,000 people; births continue in aggregate beyond that annotation limit. Broad multi-seed calibration and political balance remain future work.

## Saved-world continuation check

On 2026-09-06, the previous 100-year social archive was loaded, activated and advanced another 100 years on the Quadro RTX 5000 Max-Q. It reached year 200 with 33 settlements, 6,092 residents, 4,205 genealogy records (3,629 with recorded parents), 1,080 unions, 99 claimed terrain cells and eight faction changes. Loading, advancement and output took 14.10 seconds in the optimized development build. The largest managed-budget relative residual was 1.85e-5. This prosperous world generated no wars organically; controlled neighboring-polity fixtures separately exercised conquest, repulse, supply failure, embargoes and wartime checkpoint continuation. A separate competing-campaign fixture verifies withdrawal when another army captures the objective first. These are single-world observations, not a political-balance calibration.

The optional [governance extension](governance.md) adds administration costs, local legitimacy and autonomy, administrative secession, diplomatic trust and expiring non-aggression agreements. The limitations above describe the base political layer without that extension.
