# Institutional succession and service continuity

See [leadership recovery and representation](leadership-recovery-and-representation.md)
for the later funded local recruitment path and political weighting policies.

Schools, guilds, merchant houses and religious orders now retain a local leadership
mandate alongside their operating capacity. Previously an annual update replaced
a dead leader with the first member, including members living elsewhere. Institutions
with capacity records now recognize leadership loss at a monthly cultural boundary
and resolve succession through their own eligible local members.

A mandate records its holder, start and vacancy dates, observation month, last
ballot support, contested state, cumulative assembly work and causal event IDs.
`Institution.leader` remains the last appointed person for compatibility; the
mandate's optional holder is authoritative during a vacancy. Neither political
conquest nor a new town administrator appoints an institutional leader.

## Selection and actual consequences

Eligible voters and candidates are living adult household representatives who are
present at the home town and already members. Travelers and lost households do
not count. Religious orders additionally require the representative's existing
faith affiliation to match the institution. Membership survives departure, but
local authority does not. A returning former leader must be selected again.

Each representative casts one vote, using this bounded decision score:

```
0.6 * relevant competence + 0.3 * personal relationship
    + 0.1 * ambition (only when voting for oneself)
```

Competence uses piety for religious orders, curiosity and known-topic fraction for
schools, practiced craftsmanship for guilds, and navigation skill and loyalty for
merchant houses. Relationships range from -1 to 1. Exact ties favor the lower
persistent person ID. These are explicit game rules, not empirically fitted
historical voting models; representatives are not equal-sized population ballots.

Succession can convene quarterly. Each ballot spends 0.05 worker-months from the
existing cultural allocation; insufficient work postpones voting. A first ballot
without a majority records a contest. A subsequent quarter resolves by plurality,
with the same stable tie rule, so a two-member split cannot paralyze an institution
forever. Both ballots are paid. Institutions process in persistent ID order and
share the existing finite town budget with maintenance and other cultural work.
No money, population, goods or knowledge are created by succession.

An empty mandate blocks the existing `operational()` services: institutional
instruction, merchant expedition sponsorship and religious relief sponsorship.
Maintenance remains possible, consuming its usual money, materials and work.
Individual teachers, documents, already committed voyages and relief shipments
retain their existing behavior. Selection restores eligibility for services only
if building condition and organizational readiness also permit them. This is an
aggregate authorization rule; differentiated deputy powers remain future work.

The monthly cultural update checks mandates before quarterly upkeep and personal
cultural decisions. Earlier stages of that month have already executed; the
observation affects subsequent service decisions and months. Annual recruitment
still replenishes membership through its existing rules. Abandonment can still
close an institution. The initial succession increment added no branches or relocation. Funded relocation
now exists separately; independent officer elections and automatic reopening remain
outside this mandate model.

## Persistence, evidence and compatibility

Existing capacity archives default to no mandate. Their next monthly cultural
observation establishes a baseline from the actual eligible incumbent, without
inventing an earlier election. Records without institutional capacity retain the
legacy annual succession rule. New institutions receive their mandate baseline
at the next monthly cultural observation.

Vacancy, contest and selection events name the institution and involved person;
later events link to the previous succession event, or to an earlier institutional
event when available. The explorer distinguishes a missing local constituency from a vacancy with eligible
candidates, and shows ballot support, assembly work
and links to those events. The existing cultural evaluation summary serializes
mandates with capacity state. Archive validation checks IDs, clocks, support,
work and ordered institutional event references.

Controlled checks cover stable voting, trait/relationship sensitivity, displacement
without deletion of membership, withheld work, split-ballot delay, finite assembly
expenditure, restoration of service eligibility, causal links, repeated-boundary
idempotence and serialized continuation during a contest. The attendance fixture
sets completed relocation as an input; it does not substitute for the separate
migration-conservation suite.

Reproduce the focused checks with:

```sh
mise exec rust@1.89.0 -- cargo test --lib institution_succession -- --include-ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --test culture -- --ignored --test-threads=1
```

## Fifty-year comparisons and follow-through

Seeds 17 and 81 were compared with the retained pre-mandate employer runs, using
identical settings: terrain/ecology resolution 64, one terrain epoch, 16 founding
civilizations, crop yield 0.33, living environmental history, discoveries, local
offices, household operators and occupation-linked payroll. The evaluator build
manifest retains exact source and executable hashes. Implementation: `5837c2a`.
Hardware: Quadro RTX 5000 Max-Q, Vulkan. These are game-balance comparisons, not
empirical calibration, isolated timing benchmarks, or cross-hardware validation.

| Seed | Institutions active | Vacancies recorded | Contested ballots | Successors selected | Vacant at year 50 | Without eligible local members | Assembly work (worker-months) |
|---|---:|---:|---:|---:|---:|---:|---:|
| 17 | 91 | 157 | 101 | 140 | 19 | 18 | 12.05 |
| 81 | 107 | 167 | 88 | 155 | 12 | 12 | 12.15 |

All thirty vacancies without a local constituency were religious institutions.
They retain identity, knowledge, property and existing affiliations; this pass
does not dissolve an order simply because its remaining members no longer live
locally or share its faith. The inspector and evaluator now distinguish this
condition from a vacancy with eligible local candidates. Counts of selection and
vacancy events do not alone reconcile outstanding vacancies: observation baselines
and closed institutions also matter.

One seed-17 institution had an eligible representative at month 600 despite an
older vacancy. A deterministic rerun extended to month 612 showed appointment at
month 603, with exactly one funded assembly. Its readiness was still only 8% at
month 612, so appointment did not bypass accumulated organizational decline.
The report's age of a currently represented vacancy is a snapshot statistic;
it is **not** evidence that the same members were present throughout that vacancy.

| Seed | Population before / mandates | Shortage site-years before / mandates | Operational institutions before / mandates |
|---|---:|---:|---:|
| 17 | 1759.55 / 1759.71 | 150 / 150 | 86 / 72 |
| 81 | 1560.55 / 1559.68 | 183 / 184 | 102 / 95 |

Maximum reported relative conservation residual across the three final trajectories
was `1.31844e-5`. Population differences are small and do not establish a welfare
benefit. The meaningful immediate changes are restricted authorization, paid
succession work and traceable recovery; downstream history also includes nonlinear
feedback from other systems. Old runs did not record mandates, so their zero
succession-event counts do not imply that leaders never changed.

Verification passed **60 ordinary tests** (134 hardware/long tests left ignored),
plus **17 explicitly executed GPU-backed tests**: three institutional fixtures,
nine culture tests and five expedition tests. Three additional ordinary tests were
repeated in the focused institutional command. Culture and expedition checks
include full archive continuation and batch/single-month equivalence. Formatting,
whitespace checks and all-target Clippy with warnings denied passed. Inspector
changes were compiled, not manually exercised in a window.

The initial rescue fixture failed because its second voyage no longer had an
affordable sponsor. It already transferred existing tools to isolate rescue
accounting; it now also transfers existing town cash into the sponsoring council.
The refined fixture passes without weakening the actual sponsorship requirements
or creating money. The failed and passing logs are retained.

[Artifact retention policy](evidence/README.md)
and [Artifact retention policy](evidence/README.md)
make the evidence reviewable. To regenerate the comparison directly from the
compressed checked-in trajectories:

```sh
python3 scripts/summarize_institution_succession.py \
  --baseline docs/evidence/workshop-operators/final \
  --current docs/evidence/institutional-succession/final \
  --output /tmp/succession-comparison.json
```

The comparison reader checks raw-content hashes and matching evaluator settings.
A deliberately changed trajectory was rejected. To run new worlds with the same
settings, use fresh output directories:

```sh
python3 scripts/build_history_evaluator.py --output output/succession-build
python3 scripts/evaluate_enterprises.py \
  --binary output/succession-build/evaluator.bin \
  --build-manifest output/succession-build/manifest.json \
  --output output/succession-evaluation \
  --seeds 17,81 --years 50 --modes operators-linked
```

The existing evaluator harness retains its employer-oriented filename and general
economic summaries; `summarize_institution_succession.py` extracts the institutional
results. The follow-through repeats seed 17 with `--years 51`. Broader seed and
hardship ensembles, branch formation and differentiated officer powers remain
future work.

A subsequent [funded relocation mechanism](institution-relocation.md) can preserve
a displaced institution at a destination with existing members. Its identity and
portable property survive transit. Branch services and autonomous destination
selection remain future work. See [the continuity checks](continuity-expansion.md)
for the current source-only reproduction procedure; the historical compressed
evidence directories referenced above are no longer stored in Git.
