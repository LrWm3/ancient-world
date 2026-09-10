# Persistent institutional operating capacity

This implements the institutional-capacity idea from the [archived history design menu](archive/history-simulation-considerations.md) using existing institutions, treasuries and reserved cultural work. Organizational readiness now depends on the condition of the existing founding meeting place as well as finite operating support.

New institutions start at 50% organizational readiness but cannot provide institutional services until construction finishes. Each quarter a new meeting place may reserve up to 0.125 worker-months for construction, repair and operation, plus 0.5 abstract money for organizational upkeep. Operating support alone requires 0.025 worker-months. Work comes from the cultural allocation already reserved before production, leaving less for other cultural decisions. The institution pays its settlement for upkeep; the exact representable receipt is debited from its treasury and recorded in expenses. There is no new money source. The previous per-decision upkeep fee is retained only for legacy institutions without capacity state.

When several staffed institutions share a site, each has equal access to the initial local labor budget, capped at its maintenance requirement. Institutions with no locally present adult members reserve no work. Support is limited by the fractions of required work and money provided and locally present adult membership (two members provide full staffing). Each quarter readiness changes by `+0.12 * support - 0.08 * (1 - support) - 0.12 * disruption`, clamped to 0–1. Disruption uses existing waterlogging/cleanup exposure. Upkeep can consume effort without restoring readiness when money or staffing is missing.

Readiness below 25% blocks institutional lessons and merchant-house expedition sponsorship. Individual teaching, surviving manuscripts and alternative expedition sponsors retain their own existing requirements. An `institution_impaired` event records the threshold crossing; an `institution_recovered` event requires readiness to reach 60%, avoiding repeated notices near the operating threshold. Limited service can resume at 25% before full recovery is announced. Events identify the institution, measured support and disruption, and link to a previous institutional event where available.

Inactive institutions preserve their final capacity record but provide no services. Abandoned sites supply no upkeep work. Knowledge, property and institutional identity are not deleted when readiness falls.

## Compatibility and inspection

Capacity is an optional record on each institution. Old archives and explicit legacy fixtures default to `None` and retain their previous behavior; no historical neglect is fabricated. New state, expenditure and observation clocks persist through the existing archive header. Updates are idempotent within the same quarter. The explorer shows readiness, paid upkeep and maintenance work; evaluation summaries include each institution's capacity and operational status.

## Meeting places and finite repairs

New foundations reserve **2,000 kg of existing bricks** into their physical artifact, and require **four worker-months** of construction: 0.2 at founding and the remainder accumulated from the settlement's reserved cultural work. This represents a modest masonry meeting-room enclosure, not a complete temple, school campus or guild complex. These are explicit game construction parameters, not an architectural estimate. Roofs, interiors and detailed geometry are not modeled.

Founding requires the entire brick inventory up front. Unfinished construction is visible in the explorer and persists in saves; it provides no institutional services. Present members must supply work. With the full 0.125 quarterly allocation, completion takes 31 subsequent quarters; competing institutions and labor shortages delay it. Completion emits a provenance-linked event. Initial assembly does not consume a second material inventory or generate replacement waste.

Completed rooms lose 0.0025 condition plus 0.08 times local waterlogging/cleanup exposure per active quarter. Repair restores at most 0.1 condition per quarter, limited by bricks, treasury and reserved work. Replacement mass is **the actual embodied brick mass × repaired condition**, with four worker-months per full replacement of a new room. Thus a 1% repair takes 20 kg and 0.04 worker-months. Replacement bricks enter the structure while equal worn mass enters existing waste and C/N/P ledgers; the artifact's embodied mass stays constant. Repair work reduces organizational support. Institutions pay the site for repairs through the existing cash transfer path.

Legacy two-kg artifacts retain their old mass and labor basis; they are not upgraded into new buildings for free. Archives without meeting places keep their earlier behavior. The UI's numeric condition is an abstract usability score, and readiness remains a bounded game rule for staffing, support and disruption—not a measured organizational property. No new claim of scientific realism is intended.

Condition caps operational support; below 25% it blocks institutional services. Dilapidation is announced once below 25%, and repair recovery once condition reaches 70%. Events link to the foundation's provenance and identify both institution and artifact. Destruction immediately disables the meeting place; lost or inaccessible property is recognized at the next maintenance boundary and cannot receive repairs. Destroyed property is never rebuilt automatically.

Inactive institutions preserve their final record; this version does not weather abandoned ruins. It does not add housing, building geometry, roads or ports. The explorer shows condition, cumulative replacement materials and repair expenditure. These records use the existing monthly archive boundary.

## Validation

Controlled tests verify gradual decline/recovery, bounded flood effects, exact cash transfers, finite reserved work, repeated-update idempotence, single impairment/recovery episodes and legacy archive defaults. Existing culture and expedition tests cover continuation and sponsorship accounting.

### Historical evaluation (before construction scale correction)

Three 50-year runs at crop yield 0.33:

| Seed | Active institutions | Currently below operating threshold | Impairment events |
|---|---:|---:|---:|
| 17 | 74 | 0 | 0 |
| 81 | 67 | 0 | 0 |
| 256 | 70 | 0 | 0 |

Ordinary support met upkeep in these worlds; no natural impairment was observed. Controlled neglect and flood fixtures verify decline and recovery, but these baseline results do not establish a calibrated rate of institutional failure. Maximum relative conservation residual stayed below 1.54e-5. Reports: `output/institution-capacity.json`.

44 distinct targeted tests passed across library, culture and expeditions, including the additional upkeep fixture and checkpoint continuation. Formatting and whitespace checks passed.


### Meeting-place evaluation

The follow-up runs use the same seeds, 50-year duration and crop yield 0.33
(resolution 64, one terrain epoch, 16 founding civilizations, frozen environmental
history). Results are in `output/meeting-places.json`.

| Seed | Active meeting places | Dilapidation events | Replacement bricks (kg) |
|---|---:|---:|---:|
| 17 | 70 | 0 | 62.50 |
| 81 | 68 | 0 | 62.25 |
| 256 | 68 | 0 | 63.29 |

All remained operational, with minimum final condition above 99.99%.
Maximum relative conservation residual was 1.43e-5 or less. This is a normal
upkeep baseline, not evidence of realistic disaster failure rates; environmental
history was frozen. The controlled fixture applies ten unsupported flooded
quarters, then restores work, money and bricks, producing one dilapidation and
one repair-recovery event. It checks stock/waste/CNP replacement, constant
embodied material, cash conservation, inaccessible foundations, legacy records,
and repeated-boundary idempotence.


## Local staffing and displacement

Upkeep now uses the same presence query as practical teaching: living adult
household representatives at the institution's home site, excluding households
in transit or recorded as lost. In histories without households, the existing
local founding-leader proxy remains in use. Membership remains an affiliation;
relocation does not erase membership or create a branch at the destination.

Only staffed institutions enter the site's equal-share maintenance calculation.
An empty institution receives no upkeep work, spends no maintenance cash, cannot
repair its meeting place, and follows the existing readiness decay. Its accessible
meeting place still weathers on the usual active-institution schedule. Returning
members can restore support if work and money are available. Recovery remains
subject to existing thresholds and annual deactivation rules: this does not
reactivate institutions already closed by the annual cultural update.

The inspector distinguishes total membership from locally present adult members.
Impairment and recovery events include the local staffing count. No new fields,
archive conversion, population stock, funding source or GPU buffer are introduced.
Old saves remain readable, but absent members no longer sustain local upkeep.

### Attendance verification

A controlled fixture gives two schools the same home town: one with two local
members and one whose members reside elsewhere. With 0.025 worker-months, the
staffed school receives its full allowance; the empty school spends neither work
nor money. Moving the local representatives away causes one impairment episode
while retaining their membership. A traveler supplies no local staffing; one
remaining member yields half staffing support. Returning both members restores
readiness through finite expenditure, and serialized continuation matches.

Location and journey records in this fixture are declared attendance inputs, not
an end-to-end migration or population-conservation experiment. Existing relocation
regressions separately exercise the journey transfer accounting. This changes
institutional consequences of displacement; it is not a calibrated prediction of
how often historical institutions should fail.

```sh
mise exec rust@1.89.0 -- cargo test --lib institution_capacity::tests -- --include-ignored --test-threads=1
```

Attendance validation passed 51 ordinary all-target tests and 12 explicitly run
GPU-backed tests: two institutional fixtures, nine culture regressions, and one
relocation conservation fixture. The ordinary command left 120 hardware/long-run
tests ignored. Formatting and Clippy with warnings denied passed. Hardware was
the Quadro RTX 5000 Max-Q on Vulkan. Inspector text was compiled, not manually
exercised in a window. [Artifact retention policy](evidence/README.md)
preserve the commands and evidence. These are controlled verification results,
not a new multi-seed calibration of institutional survival.

## Leadership continuity

[Institutional mandates](institutional-succession.md) now gate formal services during
leadership vacancies. Staffed institutions can still maintain their premises and
readiness; succession spends the same finite cultural work budget.

### Construction-scale verification

The hardware-backed institution fixture covers legacy repairs and cash/material/work accounting, then checks a new 2,000 kg room: 30 quarters remain unfinished, the next quarter completes exactly once, construction creates no replacement waste, and continuation from a mid-construction serialized state agrees. A 1% repair withdraws 20 kg and adds equal waste. Other fixtures cover local staffing and recovery after neglect.

This change makes founding materially harder and takes years of the deliberately small cultural work allocation. Long-history institution frequency has not yet been recalibrated; the controlled checks establish construction behavior, not that every seed will build schools or orders at a desirable rate. Other town infrastructure still uses older game-scale costs and is not rescaled here.

Checks for this change: 37 ordinary library tests passed (46 hardware tests excluded from that run); all three institutional-capacity tests passed with hardware enabled, as did the controlled majority/minority founding fixture with explicitly funded construction stocks. The founding fixture also rejects insufficient bricks and verifies unfinished institutions are non-operational. Clippy and repository artifact checks passed.
