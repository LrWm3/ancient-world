# Evidence-driven religious change

The former annual 8% household conversion roll, global hardship/dissent schism
threshold and twenty-year 20% practice-copy roll have been removed. They were
compact game heuristics, not historically grounded explanations. This replacement
is also a **game model**, with explicit causal inputs and controlled tests. Its
coefficients are not fitted historical constants.

## Observations and boundaries

An annual cultural update reads one snapshot of resident household affiliations.
New converts cannot transmit their new faith or join a reform in that boundary.
Departed households, dead representatives and abandoned sites do not participate.
Local minority affiliations count; a neighboring town's plurality does not stand
in for everyone who lives there. Political conquest does not convert inhabitants.

Inter-site influence requires a currently passable route or a cargo arrival
witnessed during the preceding twelve months. Local co-residence also provides
contact. The old tradition-pair contact counter remains for historical/knowledge
contact bookkeeping, but cannot authorize a religious conversion or syncretism.
A route is a potential-contact proxy, not an individually simulated conversation.

## Household adoption

For each accessible alternative tradition, the household observes:

- Reachable adherents relative to supporters of its present affiliation.
- Parent/daughter household and sibling-household connections from existing ancestry.
- Positive or negative personal relationships recorded for its representative.
- Locally operational religious institutions and their maintained readiness.
- Recently **delivered** religious relief at its site, attributed to the funding
  institution; promises, losses and remote charity do not count as delivered aid.
- Overlap among doctrinal themes and whether those values address its preferences
  and local hunger, disease, disruption and inequality. A household's own observed
  food shortage also matters when its food account describes its current site.
- Loyalty and piety as resistance to changing affiliation.

Contacts receive weight 1 locally, or `0.35 / (1 + km/500)` across a route;
kinship and relationship strength modify this weight. A completed delivery without
a current direct route uses a conservative 1,500 km contact-distance proxy.
Household counts are representative constituencies, **not population or property
mass**. Plurality and the social faith overlay now also count households; ownership
shares previously gave wealth an unintended vote in religious prevalence.

Net advantage is a bounded weighted score, not a probability:

| Term | Coefficient |
| --- | ---: |
| Contact share minus current-faith contact share | 0.55 |
| Accessible kin | 0.25 |
| Strongest positive relationship | 0.25 |
| Local institutional readiness advantage | 0.25 |
| Difference in doctrinal fit | 0.30 |
| Recent delivered relief | 0.18 |
| Doctrinal distance penalty | −0.15 |
| Loyalty penalty | −0.15 |

Conviction follows `old × 0.75 + max(advantage, 0) × 0.35`, capped at one
and at a 0.25 annual increase. Disconnected memories decay but cannot cause an
adoption. A household adopts its strongest supported alternative after conviction
exceeds `0.28 + 0.12 × piety + seeded_tolerance`, where tolerance is fixed in
0–0.10 for that household. This introduces stable individual variation, not a
new annual coin toss. A single annual observation cannot trigger conversion.

Events report conviction, contact share, kinship, relationships, service advantage,
doctrine overlap, previous faith and an actual contact representative. Relief-based
adoptions also reference the delivered mission's outcome event. No goods, money,
food or population are produced or transferred by affiliation changes.

## Supported reform and shared practice

Each local congregation ranks absent themes against its least-supported existing
theme. Preferences connect named values to human dispositions and observed needs:
for example hunger strengthens hospitality/stewardship, inequality reciprocity,
and disease inquiry/restraint. These are explicit fictional cultural assumptions,
not universal claims about what hungry or sick people believe.

A proposal needs at least three supporting households and 40% of the local
congregation. Each supporter must prefer the proposed value by more than 0.35.
The same proposal must qualify for five consecutive annual observations; losing
support or changing the proposal resets that run. Candidate leaders are ranked
by piety, ambition, curiosity and actual positive relationships with supporters.

Two outcomes are distinguished:

- **Syncretism:** a currently contacted tradition already practices the proposed
  theme, shares at least half the existing themes, and an operational local order
  has a supporting leader to interpret it. Support must represent at least half
  of the receiving tradition's resident households. One supported theme changes;
  identity, patron ancestry and household affiliations persist.
- **Schism:** unresolved local hardship and weak institutional support sustain a
  grievance. The parent tradition must be at least ten years old, the chosen
  leader must be religiously engaged, and maintained institutional readiness can
  prevent the split. Only the supporting households join the child tradition;
  others retain their faith. The child inherits the patron and parent identity,
  changes the specifically supported theme, and records its human author.

At most one doctrinal change per parent tradition commits in a year. Competing
congregations are evaluated by unmet local pressure, with stable IDs breaking ties. The 256
tradition cap remains. `Tradition.dissent` is now a diagnostic of current unmet
local grievance; directly editing it does not manufacture a constituency.

## Persistence and limitations

Conviction, reform support runs and the last annual observation are serialized in
`Culture.religious_dynamics`. Old archives default to an empty evidence baseline;
we do not fabricate past persuasion. Repeated processing of the same boundary
cannot accumulate evidence twice. Household enumeration is sorted by stable ID
before reductions, so changing vector order does not change these decisions.

Not modeled here: explicit intermarriage, state conversion policies/coercion,
missionary travel budgets, exact household religious censuses, measured prestige
networks, or empirically estimated rates. Source readiness is local: a powerful
remote institution is not omniscient prestige. Relief remains unconditional;
receiving it does not compel conversion. Representative households, fixed themes
and annual snapshots still limit realism.

## Verification

Hardware fixtures run with `mise exec rust@1.89.0 -- cargo test --lib
culture::dynamics -- --ignored --nocapture`:

- Seeds 17, 81, 256: isolated households retain faith; reachable, receptive
  households adopt. Kinship/relationships increase the immediate conviction
  mediator. Serialized continuation and reversed household enumeration agree.
- Sustained supported reform creates a child with its parent's patron, a named
  human author and the intended theme; non-supporters remain. Four observations
  are insufficient. Operational institutional support prevents the matched split.
- Current compatible contact plus a supported operational interpreter permits
  syncretism; stale century-long contact and inactive institutions do not.
- Legacy cultural state defaults cleanly and invalid memory references are rejected.

Pure tests cover evidence accumulation/decay and doctrine/preferences. The cultural
GPU integration suite also covers conquest without conversion, source inventories,
artifact accounting and full archive/monthly-versus-batched continuation. A legacy
fixture now verifies that high scalar dissent and ambitious people alone cannot
create a schism. These tests establish implementation and coupling, not empirical
calibration of conversion rates or religious diversity.

## 100-year integration samples (2026-09-10)

Final implementation, Vulkan on Quadro RTX 5000 Max-Q; terrain 64, ecology 32,
one geological epoch with one ecological year; eight founding civilizations;
households, politics, governance and shipping enabled. Planetary fields stayed
fixed during these social-history runs (`--living-world` was not enabled).

| Seed | Household adoptions | Syncretisms | Schisms | Registered traditions | Final population |
| --- | ---: | ---: | ---: | ---: | ---: |
| 17 | 23 | 3 | 0 | 8 | 1,210 |
| 81 | 7 | 2 | 0 | 8 | 1,242 |
| 256 | 29 | 2 | 0 | 8 | 807 |

These worlds retained their eight founding traditions and produced supported
practice exchange without obligatory splintering. Controlled hardship fixtures
establish that splits are possible, not that their frequency is calibrated.
Adoption counts are events, not unique people or percentages of the population.
These small, fixed-environment samples are integration smoke tests, not a
historical or climate-coupled validation ensemble.

Reproduce each seed with:

```sh
mise exec rust@1.89.0 -- cargo run --bin ancient-world -- \
  --headless --resolution 64 --ecology-resolution 32 --ecology-years 1 \
  --epochs 1 --seed 17 --civilizations 8 --society --politics --governance \
  --shipping --history-years 100 --history-export output/religion-17.json
```

Raw exports remain local and ignored. Final checks: 34 ordinary library tests,
three new GPU integration fixtures, nine cultural archive/accounting fixtures,
nine cultural-practice fixtures and the social-observation fixture passed. Clippy
with warnings denied, formatting and the repository artifact check passed.
