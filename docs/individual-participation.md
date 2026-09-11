# Individual participation

Shared participation covers cultural/research work, refined workshop staffing,
named expedition service and new military campaigns. Enabling it does not by itself
convert aggregate population into individual demography. Population authority
depends on the selected [demographic mode](individual-demography.md) and
[resolution framework](resolution-framework.md). Production and food accounting
still use settlement aggregates; named participants are not added to those stocks.

This guide includes the successive participation extensions. For current timing
and sharing rules, use the [monthly schedule](monthly-schedule.md) and
[allocation policy guide](service-allocation.md).

## Identity and presence

`History.participation` stores numeric person references, known household membership,
opening-month presence, available capacity and cumulative completed work. Existing heads and recorded kin supply membership. With participation enabled,
local cultural eligibility includes known adult family members, not only ownership
heads. The number of site action grants remains bounded; additional candidates do
not create additional labor. A living leader without a
household can be associated with the civilization's first active settlement. Unknown
residence stays unknown. Children and people aged 60 or older have no converted
service-work capacity, matching the adult cohort workforce. Household relocation removes
its known members from local eligibility; death and relocation are rechecked live.

These membership links refer to ownership households. The separate
[domestic-group adapter](domestic-participation.md) observes recorded family ties
and reserves caregiving; it does not replace economic ownership accounts.
New expedition crews reference shared person IDs; older crew archives retain their
separate identities. New military campaigns also have shared identities; legacy armies remain aggregate.
Participation alone does not establish a complete resident census; the
[resident baseline](resident-rosters.md) explicitly identifies whole anonymous slots. The explorer labels
it as a known-person roster and displays current presence separately from its opening
observation.

## Participation contract

The monthly reservation opening captures personal capacity once. The initial ceiling
is `0.8 * (1 - 0.5 * settlement_illness)`, with the same illness clamp as the existing
workforce calculation. It is an abstract effective-work allowance for converted
services, not a simulation of waking hours or a complete personal employment schedule.

Both gates apply: available settlement service labor and available named participants.
Every grant is deducted once from the existing settlement service pool. Personal shares
partition that grant; they do not add extra workers. Ordinary production still uses
aggregate labor after service reservations.

- Cultural bundles pin their actor, successor and institutional teacher. Their named
  team divides the grant equally. This conservatively reserves possible supporting
  participants for the bundle, rather than pretending each action has separate time slots.
- Research chooses at most four known local adults, ordered by curiosity plus bounded
  accumulated research experience, then by stable person ID. The same people may also
  be cultural actors; the later reservation sees only their remaining capacity.
- Research source workshops remain workshops. Their IDs are not treated as person IDs
  or as physical visiting teachers.
- Execution checks current presence; missing participants invalidate the team's grant.
  Partial work due to materials or other existing constraints remains possible.
- Settlement records actual completed work and each person's proportional contribution.
  Only completed research builds the experience used in subsequent team selection.
- Unused commitments expire. They cannot be reassigned after execution or credited as
  productive experience. The layer does not introduce wages or change ownership.

Research and culture now forecast demand together under an explicit site allocation
policy. Research-first remains the default; weighted sharing is opt-in. Personal
matching still reserves research participants before cultural participants, and
execution remains in its existing subsystem stages. Site shares are not a
simultaneous optimal assignment of people. No GPU readback or population update is
introduced by participation.

## Persistence and controls

New histories enable participation. Older archives without the field retain legacy
aggregate service work. `History::set_individual_participation(bool)` switches at a
completed personal-work boundary; enabling it establishes known membership on the next
reservation. It does not invent biographies or convert population. Disabling clears
personal plan links while preserving existing economic and cultural records.

The registry, current commitments and accumulated contributions are serialized.
`History::participation_report()` and the service work report expose them; the culture
explorer includes an Individual participation panel. Historical IDs are unchanged.

## Population authority and remaining work

[Domestic groups](domestic-participation.md), [whole-resident identification and
relocation manifests](resident-rosters.md), and [individual births, deaths and
birthday transitions](individual-demography.md) have subsequent implementations.
Individual demographic authority remains opt-in, with explicit fractional residuals
and conversion checks for older stocks. Ordinary labor and food accounting are
still aggregate; ownership-account relocation does not independently move domestic
families. Broader conversion coverage and additional sector participation remain
separate work.

## Expedition identity bridge

New voyages choose eight present adults aged 15–54, excluding civilization leaders
and people with any current-month service commitment. Existing identities are reused.
When the sparse named roster is insufficient, recruitment can individually record
unnamed adults already included in the town's adult stock. This is bounded by the
adult cohort minus known present adults; it is not a birth or population import.
The departure event explicitly records this initialization, unknown parents and an
estimated starting age of 25–39. These recruits attach to existing ownership
accounts; no domestic family relationships are invented.

`History.person_duties` stores current voyage, origin and ownership-account references
independently of the optional participation switch. Presence remains correct while
expedition processing temporarily owns the voyage list. Rescue transfers each live
person's duty to the rescuing voyage; historical manifests remain historical records.
Validation rejects missing duties, duplicate active identities and orphaned duties.
Legacy manifests without person IDs remain readable and are not retroactively turned
into witnessed biographies.

Travelers cannot perform local cultural/research work, marry or produce representative
birth records at home, or consume local mortality credits. Existing ownership-account
relocation waits if one of its members is on expedition or military service. This is a temporary restriction
of the ownership-household model, not the intended eventual treatment of independently
moving domestic families. Named crew deaths update `Person.died`, preserve event
subjects and allow household succession without a second population death. Survivors
return to their original town, retain accumulated competence, and remember a field
location they actually worked at. People crossing age 60 while away return to the
elder cohort.

The existing monthly voyage wage pool now remits to living participants' household
wallets, with matching wage ledger entries. It comes from the existing expedition
escrow. Legacy/unbanked crews keep the town-wallet path. Empty crews receive no wages.
Ordinary household payroll is still aggregate; this does not convert every job.

Launch requires installed, usable harbor capacity, not already-funded merchant
shipping crews. The voyage reserves its own eight adults and outfitting materials.
Ordinary shipping still requires funded vessel work. This distinction fixes a launch
deadlock at structurally adequate ports with no current merchant cargo demand.

See [travel verification](individual-travel-verification.md) for the new checks.

## Verification

Verification used Rust 1.89.0 and the Quadro RTX 5000 with Max-Q Design, Vulkan
backend. Generated logs and reports remain under ignored `output/`.

The first long-run diagnostic exposed an incorrect inference in the new ledger:
several existing cultural actions retain their initial grant array after recording
completed work. Contributions now use explicit per-site completion records from
maintenance, elections, petitions, heritage work, recoveries, pilgrimages and personal
choices. Quarterly cultural receipts also include election work, which previously
occurred before their measurement window. This fixes reporting; it does not invent
additional labor. Personal presence is checked before elections spend their grant.

The calibration runner now asserts that lifetime individual contributions match the
existing cumulative cultural and research work counters. It distinguishes a passing
no-contention history from a controlled fixture in which people are actually scarce.


### Results

- Ordinary library tests: **76 passed**, 69 hardware tests ignored by that command.
- Participation tests: **four passed**, including one GPU workshop fixture (the other
  three overlap the ordinary library suite).
- GPU culture tests: **13 passed**; research exchange fixture: **one passed**.
- History environment tests: **three passed**, including full-state batch/checkpoint
  comparisons and compact/full environmental readbacks.
- Clippy across all targets with warnings denied, formatting and artifact policy passed.

That is **94 distinct tests** across the selected suites, not a claim that every
hardware test in the repository ran. The fixtures verify shared personal capacity,
partitioned team effort, no repeated settlement, preserved archived commitments,
death/relocation invalidation, and real workshop output when people are available.
Precommitting the available people blocks sample processing while leaving the samples
intact. Cultural teaching has an explicit completion-to-personal-ledger assertion.

Three matched 100-year runs used seeds 17, 81 and 256, terrain edge 32, ecology edge
16, one geological epoch, 16 initial civilizations and default crop yield 0.5.
The runner enables society, politics, governance, offices, shipping, expeditions,
discoveries and living history. One arm enables participation; the other uses
`--legacy-participation`.

| Seed | Final population | Active institutions | Known person records | Eligible named adults at last opening | Completed cultural worker-months |
| --- | ---: | ---: | ---: | ---: | ---: |
| 17 | 2,760 | 32 | 3,165 | 2,044 | 2,354.175 |
| 81 | 3,165 | 34 | 3,263 | 2,108 | 2,464.125 |
| 256 | 2,418 | 32 | 3,242 | 2,065 | 2,274.525 |

All reported non-participation observations, event-type totals and final economic
residuals matched between arms. Personal cultural totals matched cumulative subsystem
work within 0.000007 worker-months at the endpoints; the runner checks agreement at
every quarterly observation. Known records include deceased people and are not a
second population count.

**None of these ordinary histories performed research work.** Thus they verify
non-contention behavior and accounting, not balance under substantial cultural/research
competition. That interaction is established by the controlled fixture, not by claiming
that matching seed trajectories prove it. Larger populations, full resident demography,
and sustained research-heavy histories remain uncalibrated. Concurrent run timings are
not performance benchmarks.

### Reproduction

Prefix Cargo commands with `CARGO_INCREMENTAL=0 mise exec rust@1.89.0 --` in this
workspace:

```sh
cargo test --lib
cargo test --lib participation -- --include-ignored
cargo test --lib culture:: -- --ignored
cargo test --lib discoveries:: -- --ignored
cargo test --test history_environment -- --ignored
cargo clippy --all-targets -- -D warnings
cargo build --example cultural_work_calibrate
```

Then run:

```sh
target/debug/examples/cultural_work_calibrate --seeds 17,81,256 --years 100 --output output/individual-participation-enabled.json
target/debug/examples/cultural_work_calibrate --legacy-participation --seeds 17,81,256 --years 100 --output output/individual-participation-legacy.json
python3 scripts/summarize_cultural_work.py output/individual-participation-enabled.json output/individual-participation-legacy.json
```

## Military identity bridge

New raids and political campaigns select present, uncommitted adults through the
same recruitment contract as expeditions. The food/tool/manpower limit is rounded
down to a whole-person maximum. Recruitment can return a smaller available force,
but requires at least three people; it never rounds up past the resource limit.
Known people are reused before identifying unnamed cohort adults. Identification
requires a kinship registry and an existing ownership account. With politics disabled,
simple raids can recruit existing eligible ownership heads but cannot create
additional kin records.

Each active army has a live roster and each member has one military duty with origin,
household and starting month. These duties remain authoritative even while the
monthly army loop temporarily owns the raid list. Local work, marriage, representative
births, local named mortality and ownership-household relocation exclude travelers.
The population withdrawal remains the existing cohort transfer, now equal to roster
length; the roster does not add a second stock.

Expected fractional attacker losses accumulate in an army-level remainder. Only
whole deaths remove people, clear duties and debit the existing population ledger.
Death events identify the people and link to the campaign's preceding cause. They
do not consume local demographic death credits. Survivors return to their previous
household and town; people reaching 60 return to the elder cohort. Death and return
events preserve identities after the live army record is removed.

Military careers retain campaign counts and actual monthly service. Mean experience
adds at most 15% to attack preparedness, reaching its cap after 120 service months
per member. This is a bounded toy-game parameter, not additional manpower or a claim
about historical combat. No military wage system is added here.

Old armies without rosters keep fractional aggregate losses and returns. Validation
rejects roster/manpower mismatches, dead or duplicate active members, missing/orphaned
duties and simultaneous military/expedition service. The explorer lists army members
and their careers. Local defenders, ordinary employment, domestic households and
demographic births/deaths remain aggregate; this completes another population-writer
adapter, not the complete-population conversion.

Verification and limitations: [military participation](individual-military-verification.md).

## Known domestic groups

The next adapter separates recorded domestic families from ownership accounts and
connects caregiving to local work and recruitment. See
[domestic participation](domestic-participation.md) for timing, controls and limits.
This supersedes the earlier statement that domestic groups are entirely absent;
full population rosters and household demographic authority remain outstanding.

## Population reconciliation and named mortality

[Population reconciliation](population-reconciliation.md) now distinguishes present
named age bands from travelers and unresolved identities. New histories limit
representative births to unrepresented child slots and assign some freshly counted
GPU deaths to named residents of all ages. These assignments consume existing death
credit without another population debit. Earlier age-70-only named mortality remains
a legacy mode. This improves alignment; it does not yet make named people the census.

## Reusing residents in succession

Ownership succession now prefers a present recorded child, another account member,
or another local adult before identifying an unnamed resident. It checks adult and
elder slots and preserves a vacant estate when no representative is available.
Succession references no longer assume that the successor was created after the
predecessor: valid references and an acyclic chain are required instead. See
[resident succession](population-reconciliation.md#resident-succession) and
[verification](resident-succession-verification.md). [Vacant ownership accounts](estate-vacancies.md)
now retain property without inventing a head. A single allocator for all population
adapters and complete rosters remain outstanding.
