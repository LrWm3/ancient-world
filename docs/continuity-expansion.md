# Six continuity additions: implementation and checks

These are bounded game mechanisms over existing records and ledgers. Each addition
has its own commit and module; no new scheduler or parallel population authority
was introduced.

| Addition | Implemented connection | Remaining scope |
| --- | --- | --- |
| [Institution relocation](institution-relocation.md) | Funded journey moves accessible portable property to existing destination members, preserving institutional identity and treasury | Autonomous relocation planning and multi-site branch services |
| [Artifact petitions](artifact-petitions.md) | Later completed office service hears a consent-based title return or finite institutional compensation | Contested evidence/guilt, personal wallets separate from households, and case-specific court staffing |
| [Route warnings](traveling-route-warnings.md) | Surviving relocating households deliver observed closures; local aging evidence changes destination preference | Merchant warning channels, occupation and outbreak reports |
| [Contagion](contagious-illness.md) | Conserved SEIR partitions, relocating populations and dated cargo contacts feed existing illness consequences | Military/expedition partitions, multiple diseases and empirical calibration |
| [Peace](negotiated-peace.md) | Bilateral acceptance funds first payment, orders traveling withdrawal, and creates enforceable monthly dues | Autonomous bargaining, territorial exchanges and third-party enforcement |
| [Sieges](supply-limited-sieges.md) | Actual construction grants create defenses; rationed armies delay battle, reserve shared freight and withdraw under shortage | Autonomous investment/resupply planning, naval blockades and detailed carriers |

New records persist with serde defaults for earlier archives. Existing worlds do
not acquire invented historical moves, lawsuits, peace terms or fortifications.
New-world contagion is enabled; old archives retain their prior health behavior
until explicitly enabled. Scenario APIs request relocation, petitions, peace,
construction and military freight. These actions are not automatically selected
by a new strategic AI. Monthly progression then executes their consequences.

## Reproduction

Run `python3 scripts/check_continuity.py` with a working native GPU backend. It
runs ordinary library tests, then the focused hardware fixtures serially. Raw logs
and checkpoint files stay under ignored `output/`. Also run
`cargo clippy --all-targets -- -D warnings` and
`python3 scripts/check_repository_artifacts.py`.

The controlled fixtures use small generated worlds (64 terrain / 16 ecology,
seed 17 for the shared fixture). They test immediate mediators, actual resources,
negative controls, identity/custody, and relevant checkpoint continuation. The
siege fixture declares imported bricks, campaign tools and food in the existing
ledgers so campaign eligibility does not depend on incidental household spending.
It then uses ordinary construction, mobilization, ration and return operations.

An initial peace fixture incorrectly assumed council reserves were already
funded; it was corrected to transfer existing municipal cash before acceptance.
An initial siege fixture lacked campaign provisions after its construction period;
it now declares those starting inventories explicitly. Neither failure justified
loosening normal affordability checks.

These checks establish the tested mechanisms and boundaries, not long-run balance.
No multi-seed war/disease incidence calibration or universal population-survival
claim is made. Food, population and economic conservation use the repository's
existing relative residual tolerance of 0.001 in integrated GPU runs; local
compartment and money-transfer fixtures use tighter numerical checks.

## Recorded result

The final combined check passed: 146 ordinary library tests (123 hardware or
scenario tests remain ignored by the ordinary invocation), plus all nine focused
checks across the seven filters. Seven of those focused checks are GPU-backed;
two are unit checks. The integrated siege/peace fixture also passed whole-history
validation and exact 18-month batch/checkpoint comparison. Earlier contagion
verification includes 12-month batch/checkpoint equality and a 1,200-month closed
compartment conservation case. This is intentionally narrower than running every
ignored integration test in the repository.

## First refinement pass

Two integration gaps were addressed after the initial six commits:

- Institutional compensation now reaches the claimant's actual existing household
  wallet when applicable. Dedicated compensation receipts participate in household
  cash reconciliation. Missing wallets and insufficient funds leave claims intact;
  buying out one claim leaves unrelated claims attached to the object.
- Land encirclement now checks the cargo's captured intermediate stops and road
  approaches, including inland portions of sea journeys. Direct port-to-port sea
  access remains open. Relief checks both land endpoints.

The first household fixture failed the existing cash reconciliation check after
wallet balances changed without an income receipt. Adding the explicit legal
compensation ledger fixed that accounting gap; no tolerance was loosened.

The repeated `check_continuity.py` run passed 146 ordinary library tests and all
11 focused checks (nine hardware-backed, two unit checks). The two new hardware
fixtures cover household payment/remaining claims and intermediate-route access,
respectively. The latter also checks once-only delivery and serialized continuation.
These remain controlled mechanism tests, not a long-run balance study.
