# Shared organizational decisions: states and households

Implemented 2026-10-05, after [state formation](STATE-FORMATION.md) and
[governance](STATE-GOVERNANCE.md). This completes a bounded organizational loop:

**founding mandate → observations → objectives → candidate programs → authorized
dated instructions → ordinary execution → observed outcomes → review.**

`agency::Controller` is attached to an existing agent ID. The same controller
chooses state legal policy, physical minting attempts, paid institutional work,
and household allocation policy. It creates no population, hours, inventory,
money, ownership or financial reporting scope. The existing person planners
remain responsible for individual acceptance and work.

The [combined 14-month control](INTEGRATED-AGENCY.md) now exercises both controllers
alongside person work, paid mint inputs, lending, a household forward and annual
land dues. CPU/reference books agree. Its baseline forward is late; an optional
household commitment-priority policy with personally signed surplus funding pays
on time without additional food shortages. This uses the same organizational
controller and bounded constitutional program menu.

`Controller::discovering` now derives program candidates from constitutional
options and resource-producing recipes instead of a supplied menu. It retains
and validates each decision's catalog and supports an initially idle mint. The
[self-starting control](ENDOGENOUS-DISCOVERY.md) installs these controllers after
citizenship and beneficial household formation; objectives and institutional
rules remain inputs.

## Responsibilities and reusable parts

| Part | Implemented behavior |
| --- | --- |
| Static operating mandate | Ordered objectives, bounded forecast horizon, review cadence, minimum program tenure, optional emergency threshold, supplied person preferences and a finite program catalog |
| Founding | An optional mandate is included in signed state founding terms and installed atomically with the state; subsequent edits invalidate those terms |
| Observation | Current organization/member/named-agent stocks, need fulfillment, deaths, failed processes, debt, commitment coverage and membership sufficiency |
| Decision | Compare retaining accepted plans with each permitted candidate, using separate reference-backend projections from one opening snapshot |
| Authority | State and household adapters identify the current person governor; only that office can issue its organization's instructions |
| Execution | Future legal/allocation policies, own-agent work targets, or a dated productive-process attempt; existing law, resource allocation and settlement remain authoritative |
| Elections | Eligible people with supplied preferences generate their own candidate choices for the next term; existing ballot acceptance and tally enforce authority and turnout |
| Continuity | Retain on equal forecasts, optional minimum tenure, explicit emergency override, immutable accepted program snapshots and once-per-Open application |
| Outcomes | Retained observations, forecasts, rejection reasons, selected program, author, effective month and ballots; financial effects use existing separate double-entry books |

Objectives are lexicographic losses: the first objective wins before the next is
considered. Food, warmth, coins and distinct denominations are never added into
one arbitrary score. Scopes select the organization, its members, or explicit
agent IDs. State membership observations currently include accepted roles;
election eligibility specifically requires citizenship. Household observations
use the dated roster. Forecasts freeze the opening beneficiaries for welfare
metrics, so dropping someone from a later roster cannot hide their outcome.

`NeedDeficit` uses accumulated reports for forecasts and preceding-month
fulfillment for current observations; month one has no completed need interval.
Deaths and failed processes are cumulative counts. Reserve targets measure
closing stock shortfalls. Debt measures outstanding loan balances in one resource.
`FundingGap` measures stock cover for the existing common accepted-claim readers,
including their land, wage, loan and process-input rules. It is a funding estimate,
not a new liability or a promise of future income. `MembershipShortfall` compares
living membership with a configured minimum.

The organization normally selects against its objectives. A governor with
supplied preferences selects against those preferences instead, within the same
permitted program menu. Preferences remain static scenario/founding inputs;
person self-directed policy changes remain deferred. This permits comparisons of
different governing priorities without silently changing individual personalities.

## Programs and monthly boundaries

Programs combine typed commands. `StatePolicy` and `HouseholdPolicy` use their
existing constitutional menus. `WorkTargets` replaces only the organization's
discretionary targets. `StartProcess` requests one future attempt, not guaranteed
production. The public-work fixture uses a zero-hour state participant and an
already accepted employment agreement with a real worker.

At Open, previously accepted work instructions become visible. Each active
controller then observes the common opening snapshot. A candidate is issued only
inside an isolated projection, run for at most 24 months. At most 16 named
programs are considered, plus retaining the current plan. Peer organization
controllers are frozen in these projections; person decisions and existing
economic settlement still run. No hypothetical peer work is published to live
state. Failed projections remain visible as rejected alternatives.

Only a selected instruction is published. Its policy/work effective date is the
following month. A newly announced mint target can inform ordinary funding orders
in the current month, as in the existing mint adapter. Acquire, reservation,
production, wages and Close accounting retain their existing timing. Actual work
rechecks available goods, lawful actions, rights, capacity and receiving storage.
The whole Open operation is staged: a failed commit rolls back its decisions,
ballots and economic effects together.

There is no automatic cancellation when retaining a plan or losing a governor.
Previously authorized work and contractual obligations survive vacancies. A
vacancy authorizes no new command. Programs and observations are retained in the
in-memory checkpoint; configuration is sealed at the first autonomous Open.
Explicitly initialized legacy institutions can attach a mandate before that
opening. States founded with a mandate additionally validate it against the
signed founding agreement.

Review cadence, minimum tenure and emergency thresholds are separate settings.
An emergency threshold names an observed objective and loss ceiling; exceeding
it allows early reconsideration, not bypassing authority or settlement. Election
observation runs before each scheduled term even when policy review is not due.
Supplied ballots are preserved. People without supplied preferences do not cast
automatic ballots. Candidates are ranked by the voter's projected losses under
the candidate's preferred feasible program, with stable ID ties. These are
current-condition platforms, not binding campaign promises or strategic politics.
During vacancy, unauthorized program projections reject; voters can still assess
the retaining-plan alternative and elect a successor.

## Public history and financial boundaries

Private economic forecasts sometimes omit other people's work and lifecycle
tables. `PublicHistory` preserves observed citizenship and death dates for state
authority, so such a projection cannot revive a dead governor or erase founding
citizenship. Live state remains authoritative; snapshots exist only in forecasts.
The affected forecast adapters also disable recursive organization decisions.
Published generated-mint target catalogs and accepted dated work survive forecast
sanitization; unpublished generic future fixture starts remain excluded.

Decision records contain no ledger postings. Actual purchases, production, wages,
taxes, loans and issuance still use the existing accounting adapters. Audited
stepping records the committed world's dated configuration, so replay sees the
same work targets that execution used. There is no automatic consolidation of
state, citizens or households.

With the settlement observer enabled, `organization_decision` logs each committed
Open's observations, alternatives/failures, chosen program, author, effective date
and ballots. Private rollouts do not emit live decision logs. Existing filters and
log limits apply; the retained controller history remains inspectable without logs.

## Demonstrated controls

| Scenario | Observation |
| --- | --- |
| Paused land admissions, two citizens with eight grain each | The eight-month forecast initially retains policy, then selects reopening in month two, effective month three. Both people independently accept land and complete harvests by the eight-month cutoff without deaths. The unchanged-policy control admits neither. |
| Insufficient state membership | A membership-shortfall objective selects reopening a posted citizenship offer. The decision itself creates no citizen; the second person subsequently accepts citizenship and land through ordinary discovery/settlement. |
| Physical minting with a 14-coin state reserve target | The first configured attempt remains public. The state chooses one additional attempt, purchases finite metal and hours, reaches 14 coins and stops adding attempts. Removing metal or mint permission prevents the target from being achieved. |
| Public firewood reserve | The state chooses work using two paid hours in each of months two and three, receives two firewood, and pays four coins. Without employment, its zero labor endowment cannot produce the reserve. |
| Vacant elected state office during public work | No new instruction is issued; previously authorized work completes and wages remain payable. |
| Household allocation | The same controller selects needs-first allocation and reduces four-month nutrition deficits against unchanged net-output policy. State and household office remain separate; simultaneous decisions preserve separate statements. The eight-month demonstration still closes with two units of member nutrition deficit; selection does not guarantee viability. |
| State founding plus household production | Signed operating terms install the controller, which subsequently generates citizen ballots while household production continues. Illegal programs reject before formation commits. |
| Continuation and instrumentation | CPU/reference mint outcomes, decision history, ledger and books agree under per-phase reconstruction. Observing execution does not alter decisions. Failed Open commits publish nothing. |

Run the four CPU demonstrations from `exp/economics`:

```sh
cargo +1.92.0 run --release --locked --example state_agency
```

Focused coverage is in `tests/agency.rs` and `tests/state_formation.rs`, alongside
the state-governance, forecast, mint, household and telemetry regressions. This is
post-v1 scoped evidence; the historical v1 full-suite counts are unchanged.

Verification: **193 scoped tests passed**, including 14 agency controls and the
related founding, governance, forecast, household, mint, composition, competition,
intermediary and telemetry suites. Counts include each test once across the runs.
The four demonstrations above ran on CubeCL CPU. Strict all-target Clippy,
formatting and the repository artifact check also passed. Reproduce the core run:

```sh
cargo +1.92.0 test --release --locked --test agency --test state_formation --test state_governance --test forecast_context --test mint_orders --test mint_finance --test mint_cycles --test households --test household_integration --test telemetry --test competition --test intermediary --test forward --test composition --test composition_market --test planner_persons
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
cargo +1.92.0 fmt --check
```

## Present limits

This fills the basic observe/choose/authorize/execute/review loop for states and
demonstrates reuse for households. It does not fill every possible state role.
In particular:

- Program menus, objectives, prices, voter preferences and offered employment
  terms are supplied. The controller does not invent laws, discover arbitrary
  program combinations, negotiate charters or autonomously sign new wage/loan
  contracts. Current lending, tax, collection and liquidation adapters still
  execute their configured or explicitly accepted terms.
- Opening full-information projections are deterministic hypotheses, not promises
  of counterparties' future behavior. There is no learned political/economic
  model, stochastic risk ensemble or large-population performance claim.
- Legal policy selects added prohibitions. Governed changes to tax rates,
  agreement recognition, inheritance rules and delegated jurisdictions require
  further typed policy adapters with their own transition rules.
- Recruitment here changes access to an existing offer; there is no negotiated
  membership benefit package, automatic founding consent, citizenship exit,
  emergency succession, state dissolution or general diplomatic system.
- State/household authority adapters are implemented. Another institution needs
  its own lawful authority/membership adapter and ordinary execution support;
  it does not need a different objective/comparison loop. Individuals can reuse
  objective observations without gaining an organizational governor.
- Retained history grows with the run. Durable checkpoint serialization, history
  compaction and cross-composition guarantees for every experimental driver are
  still separate work.
