# Economics v1: bounded release scope

Status: release checklist established; **v1 is not complete or released**.
Baseline: implementation `30c74e6`, documentation cleanup `54cf8fa`.
This is v1 of the stand-alone `exp/economics` experiment, not an Ancient World
release, a published Rust library, or a promise of a stable public API.

## Release promise and stopping rule

V1 demonstrates a small CPU economy in which distinct persons and a household
plan useful work, acquire permitted opportunities, share finite resources, trade,
service commitments and experience recorded consequences. The state supplies
configured law and offers, including physical issuance. Supported financial
arrangements use the existing authoritative books and settlement boundaries;
separate double-entry statements explain actual outcomes.

**V1 is done when V1-01 through V1-06 below are complete, the six named scenario
families pass their specified gates on one release candidate, and no known defect
violates those gates.** Then record the verified revision and close this checklist.
Do not continue adding features to fill a Fibonacci batch or complete the broader
finance roadmap. Batch sizes are work organization, not release requirements.

Only this document defines v1 blockers. [Goals](GOALS.md), the
[consolidation backlog](CONTRACT-CONSOLIDATION.md) and the
[financial stress-test program](VERIFICATION-STRESS-TEST.md) continue beyond v1.
The [integration matrix](INTEGRATION-STATUS.md) reports what currently works; a
listed future gap does not automatically become a release blocker.

## In scope

| Area | V1 commitment | Existing foundation / work to finish |
| --- | --- | --- |
| Execution | CubeCL CPU and reference execution; existing monthly phases, explicit allocation and atomic commit | Implemented; verify the release scenarios together |
| Persons | Adults with individual needs, deprivation consequences, stocks, capacities, rights and dated commitments; bounded work/search policies | Implemented policies retained; no new general optimizer |
| Households | Lawful formation, static constitution/charter, existing governor/term/election rules, contributed labor, resource/storage pooling, member support, external hiring, joining/exit and explicit wind-down | Existing behavior retained; finish household-aware prerequisite/process acceptance |
| State and law | Configured citizenship, permissions, recognized agreement forms/term limits, land offers and issuance; accepted obligations survive later permission withdrawal | Existing laws and state roles; no autonomous state government required |
| Production | Farming with seed return, wood/warmth, finite storage and competing labor; existing catalogs remain regression fixtures | Use existing processes and consequences; no new occupations or physical simulation |
| Markets | Existing local town books, need-generated orders, bilateral fixed/concession/ZIP pricing, finite funding and storage | Preserve supported drivers and observations; no requirement to merge all matchers into one session |
| Finance | Existing coin/native loans, mortgages, land dues, direct/tool forwards, employment/arrears, collection priorities, accepted relief and configured guarantees | Exercise named mixed cases; rates, consent, counterparties and permitted tender terms may be supplied |
| Recovery | Authorized proceedings, explicit custody, actual funded liquidation, crop-control transfer, supported liens/recourse, priced whole-claim assignment and separate losses | Existing supported terms; no new asset class, denomination or security mode required |
| Reporting | Sole double-entry reporting system, explicit separate-agent scope, supported cost policies, journal persistence and in-memory continuation | Reconcile every release scenario; no fallback to guessed equity or unreported movements |
| Observability | Existing metrics/logs and planning/settlement receipts explain decisions, rejected requests, deficits, arrears and actual performance | Package a repeatable release report using these observers |

The household is an agent with delegated resources, not an aggregate population.
Membership, governance, ownership and reporting scope remain distinct. Private
loans and household/member claims are not automatically pooled or eliminated.

Supplied opening endowments, catalogs, lawful offers, quotes/limits, support consent,
financial terms, governance instructions and recovery authorization are acceptable
scaffolding. Each scenario must identify them. At least the continuing household
case must generate operational work, need orders and allocation decisions from
state; a fully scripted transaction sequence alone cannot satisfy the v1 promise.

## Work remaining

Each item needs a linked test/runner and a human-readable result record before its
checkbox is marked complete. Existing passing tests may satisfy criteria; do not
reimplement supported systems or add redundant tests to increase a count.

- [x] **V1-01 — Household-aware common acceptance.** Carry the ordinary household
  resource/allocation boundaries through ordered prerequisite acceptance and a
  dated process start. Cover citizenship/land access and cultivation for members,
  preserving the acting person, household mandate, ownership and future claims.
  Scope this to existing fixed individual priorities and explicit candidate
  bundles. A general consequence-aware household search is not required.
  Acceptance must be read-only during preparation, revalidate stale plans, and
  publish all or none of the requested package. Test shared seed/cash/storage,
  competing member labor, one scarce plot, missing permission and a forged or
  stale receipt. Starting work must not double-reserve labor or pool output twice.
  Execution and subsequent consequences must use the ordinary scheduler.
  Implemented and checked in [household acceptance](V1-ACCEPTANCE.md).
- [x] **V1-02 — Freeze the six release scenario families.** Map S1–S6 below to exact
  fixtures, tests and commands; reuse the existing cases where identified. Record
  actors, opening resources, horizon, policies, seed or “no randomness,” supplied
  consent and expected outcomes. Finish the specified mixed coverage, especially
  S1; do not require the Cartesian product of every subsystem or policy. Existing
  scenario parameters may be calibrated before this baseline is frozen, with the
  reason and before/after outcomes recorded. Do not weaken assertions after a
  failed candidate merely to pass it.
  Frozen fixtures and exact commands: [scenario manifest](V1-SCENARIOS.md).
- [ ] **V1-03 — Verify economic outcomes and explain failures.** For each named
  baseline/control, assert the outcomes in the matrix as well as conservation.
  Identify when an opportunity was absent, prohibited, unaffordable, storage- or
  labor-blocked, rejected by policy, unmatched or accepted but unfulfilled. Use
  existing observers; add only missing evidence needed by these cases. Preserve
  requested, reserved and completed work, actual trade and due/paid/unpaid claims.
  Failed controls must retain real shortages or debts rather than create rescue
  resources. Fix defects that violate these expectations.
- [ ] **V1-04 — Close accounting and continuation coverage for the release set.**
  Audit all six families through completed boundaries with separate statements.
  Verify matched debtor/creditor exposures, household/private separation, real
  custody balances, explicit issuance/transformation and loss provenance. Reuse
  CPU/reference, forged-batch, journal round-trip and in-memory continuation
  controls. In S1–S5, resume across an acceptance, annual bill, delivery or recovery
  boundary relevant to that case and compare final state, ledger and statements.
  S6 is the existing CPU population stress control; do not require a new large
  reference benchmark. Unsupported combinations must reject clearly before
  publication. No new accounting convention is required outside this scope.
- [ ] **V1-05 — Provide one release-check entry point and result summary.** Add a
  small documented command/script that runs the named existing tests/examples
  and S1 additions, exits nonzero on failure and records the tested revision and
  settings. It may orchestrate current runners; it must not become another
  simulation engine. Write raw output under ignored `output/economics/`. Publish
  a Markdown summary with need deficits, completed work/harvests, trade volumes,
  payments/arrears, issuance, recovery/losses and reconciliation results applicable
  to each case. Include reproduction instructions and current exclusions.
- [ ] **V1-06 — Validate and close the release.** On the same frozen code/catalog
  revision, run the full crate suite, release entry point, the explicitly ignored
  S6 stress test, strict all-target Clippy, formatting and artifact checks. Resolve
  every failing required check and every known violation of the release criteria.
  Record results and limitations, update README/integration status, and identify
  the completed v1 revision. A docs-only result commit may follow; any subsequent
  code/catalog change invalidates the candidate evidence until reverified.
  Creating a public release/tag or publishing a package is separate from this
  checklist and is not performed by writing this document.

Work order: V1-01, then freeze V1-02; close V1-03/V1-04 together per scenario;
finish V1-05 and V1-06. Checks may be reused across items, but each item must point
to evidence. Completing a batch or reaching a test-count target is insufficient.

## Fixed release scenario matrix

These are six bounded families, not one enormous scenario. They all execute
through the existing simulation, authoritative books and settlement machinery.
Focused variants below are required; additional policy combinations are later work.

| ID | Configuration and horizon | Required result and controls |
| --- | --- | --- |
| **S1: enter and perform** | Two adult members, one household and a state offering citizenship/land; farming and wood/warmth; 24 months. New common-acceptance integration from V1-01. | Funded/permitted bundle acquires its prerequisites and completes at least two crops with returned seed and ordinary pooling. One-plot contention uses the existing explicit allocation policy; the losing request receives no phantom right. Missing permission, insufficient joint resources and stale/forged acceptance leave no partial package. A deliberately missed work requirement produces the existing agreed failure and sunk-input consequence. |
| **S2: continuing household economy** | Existing four-person town fixture: two household members and two outside grain producers; coordinated 120-month run. Reuse [person–household loop](PERSON-HOUSEHOLD-LOOP.md). | All 240 household member-month food requirements met in the baseline, finite coin supply conserved and separate books reconciled. Retain the no-support failure control and the three-month market interruption: real shortage occurs, then household feeding resumes by month 15 under the existing calibration. No changed private target or implicit appropriation may conceal the control failure. Startup deficits outside the household remain explicitly reported. |
| **S3: production and commitments** | Existing 24-month [farm-finance](PRODUCTION-FUNDED-CREDIT.md#continuing-personhousehold-farm-finance) case with household membership, financed land, independent lease, two prepaid deliveries and finite stock buyers. | Bounded-forecast baseline meets nutrition and performs the configured mortgage, rent and deliveries. No-buyer control retains the existing repossession outcome; fixed-buffer control preserves the observed food shortfall even when financial claims perform. Future harvests and same-window receipts cannot fund current outgoing commitments. |
| **S4: physical issuance** | Existing six-month combined mint-finance case, months 12–17, with two persons, household, state, food/metal/labor orders, loan, forward and annual dues. Reuse [mint finance](MINT-FINANCE.md). | Incremental-provision baseline produces coins only after real inputs/labor complete and settles the documented claims. Full-buffer control retains zero issuance and unpaid rent. Existing cash, metal, hour and pooled-storage contention controls must reject or bound commitments without duplicate resources. This does not require a new autonomous issuer policy. |
| **S5: loss, custody and exit** | Existing short estate/mortgage/native-receivable controls through their final collection/discharge and household-exit boundaries. At most 24 simulated months per variant; freeze exact existing limits in V1-02. | Funded/unfunded liquidation, maintained/neglected transferred crop, guarantee timing, retained/discharged deficiency, and discounted/premium claim purchase reconcile actual proceeds and losses. Shared custody cannot cross-spend estate funds; native repayment can be storage-blocked. Household exit cannot erase member/investor claims. Existing separate suites can supply these controls; no all-in-one combination is required. |
| **S6: population regression** | Existing 32-person/eight-household specialist accounting test for 13 months, including annual dues. | Completes on CPU with reconciled separate statements, bounded resources and valid ordinary execution. Report deficits, idle work and runtime; zero deficits, optimal specialization and a performance threshold are not release requirements. Explicitly run the ignored test on the candidate. |

S1 financial contracts and S2 market behavior need not use every financing product.
S3 uses its existing posted stock-bid driver; it does not require a new town-market
mortgage driver. S4 uses its mint matcher, not a merged universal order book. Existing
ZIP and governance/election regressions remain required in the full suite, without
multiplying every release scenario by every pricing or governance policy.

For S5, start from `mortgage_receivables`, `native_receivables`, `estate_receivables`,
`recovery` and `household_dissolution`; V1-02 must name the actual selected tests
and their fixed limits. For S6, the current explicit command is:

```sh
# Run from exp/economics.
cargo +1.92.0 test --locked --test household_accounting \
  specialist_households_reconcile_production_trading_and_annual_dues -- --ignored --nocapture
```

## Out of scope for v1

These are deliberate deferrals, not release defects merely because they remain
unimplemented. Existing pilots stay available and retain their regression tests.

| Deferred area | Boundary |
| --- | --- |
| General autonomous finance | New loan/guarantee underwriting, endogenous credit terms, autonomous liquidation listing/valuation, relief negotiation and arbitrary business discovery |
| Broader household planning | Conditional cooperative credit forecasts, collective purchasing under `Cooperate`/`Agreement`, a collective horizon optimizer and general consequence-aware household search |
| Personal self-government | Self-directed policy changes, personality-driven policy selection and charter revision; explicitly deferred by request |
| General institutions and law | New firm/cooperative/state-coalition types, nested jurisdictions, autonomous state governance, constitutional amendments, market recruitment and autonomous voting |
| General labor and membership markets | Negotiated/ZIP wages, internal household employment, membership counteroffers and new service-contract forms |
| Demography and physical expansion | Children, reproduction, automatic death/inheritance estates, migration, new terrain/weather systems and additional industries |
| Wider finance/accounting | New financial instruments/institution classes from the stress-test program, multiple custody currencies/FX, redeemable money, consolidation eliminations, effective-yield accounting, partial/onward claim sales and purchased unpaid-interest allocation |
| General allocation/security | Joint multi-resource minimum-grant solver, arbitrary substitute tenders, cross-currency liens and unsupported security inheritance |
| Platform/product work | CUDA/other GPU certification, fully device-resident planning, large-population performance guarantees, GUI, main-game integration, durable full Simulation/Audit restart and public API stability |
| Universal combinations | Every current pilot composing with every other pilot; replacement of typed adapters by a universal contract interpreter or scheduler rewrite |

## Release evidence and scope control

The existing 1,023-test full-suite checkpoint is evidence for `30870e5`; later
focused gates cover subsequent changes. **Neither certifies this release list.**
V1-01 has focused implementation evidence and V1-02 freezes the scenario manifest;
the remaining items await the complete candidate run.
The complete release still requires the frozen candidate checks below.

Each completion record must include item/scenario IDs, exact command and revision,
expected and observed outcomes, passing/failing checks and limitations. A scenario
with intended default can pass; a scenario that silently loses debt cannot. A
balanced journal alone cannot satisfy the healthy-baseline economic criteria.

New findings are classified as either a defect against a named v1 criterion or a
post-v1 enhancement. Do not silently extend the release with an extra scenario,
agent type, financial product or generalization. An intentional scope change must
edit this checklist with the reason and changed acceptance criteria before work
begins. There is no obligation to finish the rest of a Fibonacci sequence after
the release gate passes.
