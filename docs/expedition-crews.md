# Expedition crew competence and attrition

Crew roles now affect field work, sample collection, hazard resistance and repairs.
Previously all surviving parties used the charter's fixed shared skill, and
casualties always removed the last living roster entry. That made porters absorb
the initial casualties and left specialist losses largely cosmetic.

## Preparation, work and losses

New charters retain the existing eight withdrawn adults and roles. Each crew member
receives a bounded competence derived from the existing charter skill, a dedicated
seeded variation stream and practical knowledge held by local living representatives:

| Role | Relevant local practice |
|---|---|
| Captain, navigator | Lake navigation |
| Naturalist | Medicine |
| Engineer | Metalworking |
| Guard | Weather signs |
| Porter | Preservation |

With the practice present, initial competence is `base + variation`; without it,
it is `0.85 * base + variation`. Variation is in `[-0.05, 0.05)`, and the result is
clamped to `[0.05, 1]`. These are fictional preparation proxies, not historical
measurements or newly simulated training courses. The source is existing local
knowledge; remote manuscripts and absent experts do not directly supply it.

For a task, effective competence is half general capacity and half relevant
specialist capacity. Each is the sum of living members' competence divided by its
original roster-slot count. Dead slots stay in the denominator. This prevents
attrition from raising output just because only skilled survivors remain.
Without any roster specialist, generalists supply half their general capacity
for the specialist component. All results are bounded by one.

Ecology uses the naturalist, geology the engineer, charting the navigator, and
other objectives the captain. Observation growth and finite geological/ecological
sample collection use this effective research competence. Tools and source stocks
retain their existing limits; higher competence does not duplicate material.

Field danger uses the average of navigator and guard capacity in the existing
hazard equation. Repair probability is `0.35 * (0.5 + 0.75 * engineering capacity)`;
the existing deadline, ten kg timber and three kg tool requirements still apply.
Loss of the engineer makes repair less likely, without forbidding improvised work
by survivors. A living field worker gains `0.002 * (1 - competence)` each camp
month. This is bounded experience, not additional labor or biomass.

Casualties select a living roster slot through a separate seeded random channel.
There is no longer a guaranteed porter-first sequence. The existing death and
resource ledgers remain authoritative. A completely dead party does no further
field work. Rescue copies only living crew with their actual competence; the
original expedition becomes inactive, so those people are not active twice.

## Inspection and compatibility

Crew competence and effective research competence appear in the expedition
inspector. The optional competence field is archived with each existing named crew
record. Old records without individual competence retain their shared research
skill; no earlier training is fabricated. New charter rosters use the new rules.
Casualty selection and the revised repair rule apply to continuing histories, so
this is not a promise of identical futures across software versions.

Crews still represent withdrawn aggregate adults rather than genealogy-linked
household members. There is no reserve of recruitable named veterans, and this
increment does not add sailing storms or inland tactical routes. Permanent
colonization is outside the island civilizations' intended scope; separate
ancient-continent civilizations remain undecided (see [civilization scope](civilizations.md)).
Experience returns through the existing delivered
findings/expedition-preparation feedback, while individual crew records preserve
what happened to the expedition.

## Verification

Analytical fixtures check an intact eight-person party at competence 0.8, then
loss of its naturalist: ecological competence drops to 0.35 while engineering
competence remains higher. Other checks cover zero survivors, correct versus
irrelevant local knowledge, legacy records, and serialized competence.

The full expedition fixtures now additionally require field experience to grow
and rescued survivors to retain exactly their pre-rescue competence. Existing
fixtures check finite stores, death accounting, rescue/recall travel, heritage
findings, rejection of invalid archives and exact same-backend checkpoint and
single-month/batched continuation. Discovery regressions exercise finite sample
collection, application and conservation.

```sh
mise exec rust@1.89.0 -- cargo test --lib crew_tests
mise exec rust@1.89.0 -- cargo test --test expeditions -- --ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --test discoveries -- --ignored --test-threads=1
```

## Matched fifty-year evaluation

Seeds 17 and 81 used the same settings and retained baseline as the institutional
succession evaluation: resolution 64, one terrain epoch, 16 starting civilizations,
yield 0.33, living environment, discoveries, offices and household employers.
The only simulation changes in this comparison are crew competence, casualty
selection and their expedition consumers. Later economic differences include
feedback and are not isolated estimates of the value of expedition experience.

| Seed | Launches before / after | Returns before / after | Casualties before / after | Delivered knowledge before / after |
|---|---:|---:|---:|---:|
| 17 | 9 / 9 | 8 / 8 | 3 / 5 | 22.48 / 19.41 |
| 81 | 10 / 10 | 10 / 10 | 1 / 1 | 34.14 / 32.02 |

Seed 17 retained one active voyage at year 50. Neither world lost an entire
expedition. Seed 17 had one stranding and one successful repair in both versions;
seed 81 had neither. Neither sampled world performed a rescue, so rescue claims
come from the controlled tests, not these ensembles. Research fell by roughly
6–14%; this is consistent with incomplete preparation and attrition now limiting
field work, but the analytical fixtures establish the immediate mechanism more
clearly than the diverging long runs.

Maximum reported relative conservation residual in the new runs was
`1.31843e-5`. These are small game-balance samples on a Quadro RTX 5000 Max-Q using
Vulkan, not empirical calibration, cross-GPU validation or evidence for a specific
historical casualty rate. No numerical tuning was made to force failures or
successful rescues. Larger hazard/seed ensembles remain useful future evaluation.

Verification passed 62 ordinary tests (134 hardware/long tests remained ignored),
five explicitly run GPU expedition regressions and four GPU discovery regressions.
Archive validation rejects out-of-range individual competence. All-target Clippy
with warnings denied, formatting and whitespace checks passed. The inspector was
compiled, not manually exercised in a window.

One discovery-rescue fixture initially lacked a sponsor able to afford its second
voyage. It already supplied existing tools to isolate specimen custody; it now
also transfers existing town cash to the council and verifies that total money
is unchanged. The simulation's sponsorship gate was not relaxed. Both the failed
fixture log and the corrected passing run are retained.

[Artifact retention policy](evidence/README.md)
and [Artifact retention policy](evidence/README.md) preserve the results.
The scripts reject altered raw results and mismatched evaluator settings.

```sh
python3 scripts/summarize_expedition_crews.py \
  --baseline docs/evidence/institutional-succession/final \
  --current docs/evidence/expedition-crews/final \
  --output /tmp/expedition-crew-comparison.json
```

To generate fresh runs with the same settings and an immutable executable:

```sh
python3 scripts/build_history_evaluator.py --output output/crew-build
python3 scripts/evaluate_enterprises.py \
  --binary output/crew-build/evaluator.bin \
  --build-manifest output/crew-build/manifest.json \
  --output output/crew-evaluation \
  --seeds 17,81 --years 50 --modes operators-linked
```

The shared evaluation harness retains its employer-oriented filename; the crew
summary extracts expedition outcomes from those verified trajectories.
