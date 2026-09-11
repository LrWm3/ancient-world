# Short-horizon petition causality checks

These tests follow the uncertain connections identified in
[the balance sweep](civic-petitions-balance.md). They check implemented game
mechanisms, not historical validity or a desirable rate of war.

Each test creates a small history for seeds 7 and 17, clones a completed
boundary and changes one mediator or prerequisite. The consumers are the
production code's institutional maintenance, annual taxation, monthly
governance, household faction selection and campaign mobilization. No second
implementation of their scoring rules is substituted into the tests.

## Interventions and observations

| Connection | Intervention | Expected observation | Control or ablation |
|---|---|---|---|
| Learning grant → maintained institution | Resolve a funded request; run four quarters of upkeep | Readiness remains operational; two currency units reach the town for upkeep | Return only the grant to its payer while retaining the response and its history; readiness falls below operational threshold |
| Resources × institutional staffing | Same grant, no work allowance or no present members | Money remains unspent and readiness follows the unfunded decline | Identical grant with staff and work improves readiness |
| Autonomy → fiscal burden | Resolve an autonomy petition, then run actual annual taxation | Town retains more cash and council receives correspondingly less | Restore only pre-response autonomy, keeping the same response and loyalty |
| Autonomy → administrative demand | Run monthly governance from the same branches | Less administrative payroll is required; unrest is no higher | Original autonomy with otherwise identical state |
| Petition memory → faction preference | Give artisans recent delivery credit in a near-tie household fixture | Eligible local households switch from growers to artisans | Remove only the petition memory; previous affiliation persists |
| Political credit locality | Same memory and material conditions | Other sites and households outside their decision schedule retain affiliation | Attribute the response to a different controller; local credit disappears |
| Grievance → campaign | Add a recent shortage event to a supplied, contested route fixture | One campaign mobilizes ten actual adults and carries the event as its cause | An honored petition without the grievance cannot trigger the campaign |
| Campaign prerequisites | Remove access, overlapping claim, tools, food, or grievance freshness separately | No war and no stock consumption | Full-prerequisite branch mobilizes while conserving people, provisions and equipment across town and army |

The fiscal test compares the observed transfer difference to the specified
tax rule, including the 0.75 autonomy coefficient. It also checks the reduced
payroll against the population-based administrative requirement. This checks
the consumer, not merely whether the autonomy field changed.

Grant tests count cash across town, council, institution and household
wallets. Returning a grant to its payer is a test-only ablation, not a new
player action. The political near-tie is intentionally controlled so that
removing the reputation connection fails the positive assertion. The war
fixture supplies a schematic route graph; it is not a geographical survey of
the sampled towns.

## Reproduction

```sh
CARGO_INCREMENTAL=0 cargo test --lib civic_petitions::causal_tests -- --ignored
```

Terrain/ecology resolutions: 64/16, five founding civilizations, two seeds per
test. GPU hardware is needed to initialize histories; the isolated social
consumers run on CPU. Generated test logs belong under ignored `output/`.

## Recorded result

All four targeted tests passed on the Quadro RTX 5000 Max-Q/Vulkan backend,
covering both seeds in each case. The ordinary library suite also passed
60 tests; its 62 hardware-dependent tests remained unselected in that
ordinary invocation. Formatting and the repository artifact check passed.

## What these tests do not establish

The previous ensemble's increase from 78 to 99 war declarations remains a
long-horizon association. These tests establish that public credit can alter
preferences, autonomy affects finances, and wars require material and spatial
prerequisites. They do not identify which of those pathways explains the
ensemble increase, nor demonstrate that any petition reduces famine or
improves learning outcomes. School readiness is not knowledge acquisition.

Longer matched branches with separate credit, grant and autonomy ablations
remain useful before adjusting political weights. No production coefficients
were changed in this test pass.


The subsequent [faction update](faction-interests.md#household-access-and-political-accountability-september-2026)
records which administration resolved each petition. Refusal/underfunding now
penalizes that administration rather than an opposition advocate. A matched
turnover fixture checks that responsibility remains attached to the original
responder after a government change; the existing delivery/contact fixture
continues to test the production allegiance path.
