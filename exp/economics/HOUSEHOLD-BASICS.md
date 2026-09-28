# Household governance basics

Status: core checklist complete and verified on 2026-09-28.

This completion checklist covers the adult-only household governance loop. It does
not claim that every institutional or financial extension in GOALS.md is complete.
The working scope is lawful founding, static constitutional limits and charter
parameters, human governors, authorized operating policy, bounded member labor,
needs and commitment-aware allocation, outcomes, and separate financial reporting.

| Requirement | Status / evidence |
| --- | --- |
| One to four adult signatories; at least one living member to operate | Implemented; formation and dissolution controls in household tests |
| Static constitution and charter; no leadership/ownership conflation | Implemented; ownership/reporting remain separate |
| Fixed, rotating or elected leadership; terms and vacancies | Implemented; dated authority and election receipts |
| State recognition of household founding and constitutional limits | Implemented in laws::households; historical admission retained |
| Governor changes objective and labor tie-break within constitutional limits | Implemented; atomic dated objective/tie instructions within constitutional limits |
| Reserve bounded contributions, return unused hours and respect worker rights | Implemented; 20% default and explicit receipts |
| Current needs and continuing commitments considered in allocation | Implemented; opt-in NeedsFirst, separate NetOutput and PreserveCommittedWork |
| Household resources, storage, shelter and member payment support | Implemented pilot; half resource/storage sharing and native/coin tax/forward support |
| One integrated lawful founding → governance → policy → allocation → accounting test | Passed; six-month CPU/reference and resumed audit |
| Multi-household accounting across the first annual dues boundary | Passed; 32 people/eight households, 13 months on CubeCL CPU |

Ballots and policy instructions may be supplied by the scenario. Autonomous political
preferences, personality-driven policy revisions, six-month collective optimization,
children/adulthood, market recruitment, exit settlements, nested institutions, household hiring
and borrowing, and full household estates are extensions beyond this governance
checklist. Unsupported finance combinations must continue to reject explicitly.

## Legal founding slice

The existing single-state law policy now recognizes `AgreementForm::Household`
and grants `Action::FoundHousehold` to eligible founders. Every adult must pass the
same admission checks, including membership requirements and prohibitions. Form
recognition alone grants no action permission. Absent transaction policy retains
legacy permissive admission, which is recorded explicitly.

Optional household founding rules bound adult count, recognized leadership modes,
term length, constitutional operating policy choices, labor contribution ceiling
and productive mandate, including permitted labor tie-break choices. This allows annual elected/rotating households to be
recognized while another constitution is rejected. An unrestricted mandate is not
accepted under an explicit activity ceiling. Legacy spare-labor delegation requires
a ceiling permitting all available labor, since its actual percentage is variable.

Formation atomically records the authority, month, each founder's legal decision
and the accepted rule snapshot. It registers the household's agent type without
granting new process rights. Later changes to recognition do not silently annul
accepted households; their founding limits remain binding. Current process law
continues to constrain the individual performing delegated work. This is admission
and operational permission, not a legal dissolution or constitutional amendment API.

Validation: 62 checks passed (41 household, four household accounting, 11 agreement
law and six action-law tests); the existing slow accounting calibration was ignored.
Tests cover atomic rejection, alternative constitutions, legal term limits,
non-retroactive recognition, no automatic organizational rights, prohibited member
work and CPU/reference monthly/batched equivalence. Strict all-target Clippy passed.


## Authorized operating policy and integrated example

`household_governance::schedule_allocation` accepts an objective plus an optional
labor tie-break as one future-dated instruction. Both must be permitted by the
constitution and issued by the current living governor. The charter supplies the
initial defaults; it is not rewritten. Objective-only instructions preserve the
last effective tie-break. Existing rules for future activation, successor authority,
supersession and historical validity also apply to tie changes. Open and labor
receipts report the effective choice. A controlled equal-output test changes who
receives labor while retaining the same total grant and completed output value.

Run the reproducible two-person example from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example household_governance
```

The state recognizes elected households with two-month terms. Both adults sign and
supply ballots electing the second member for terms starting in months three and
five. The incoming governor issues a NeedsFirst/MemberId instruction in month three,
effective in month four. No founder, elected official or owner gains permission to
change the static founding documents. The household contributes 20% of each adult's
five labor units; either useful job needs six, so allocation changes what completes.

| Completed months | Governor | Operating policy / labor tie | Unmet nutrition per month |
| --- | --- | --- | ---: |
| 1–2 | Founding member | NetOutput / Rotating | 2 |
| 3 | Elected second member | NetOutput / Rotating | 2 |
| 4–6 | Elected second member | NeedsFirst / MemberId | 0 |

CPU and reference states, complete ledgers and reports match at every month. The
accounting integration test also verifies batched/monthly and cloned continuation,
finalizes statements and checks the accounting equation independently for each
person and household. Political choices in this fixture are explicit supplied
instructions; the experiment demonstrates authority and execution, not endogenous
voting or evidence that all elected governments choose beneficial policies.


## Validation scope

The final focused regression set covers 75 checks: 43 household tests, five
household-accounting tests, 11 agreement-law tests, six action-law tests and ten
telemetry tests. It includes the new integrated lawful election/policy/accounting
case and old controls for dues, forwards, storage, private ownership, vacancy,
replay and observers. The six-month example separately matches state, ledger and
reports on CPU and reference after each completed month. Strict all-target Clippy,
formatting and repository artifact checks pass.

The full economics crate suite was not rerun. The 32-person annual accounting
integration is recorded separately below because it is slower and normally ignored.
A successful reconciliation is evidence of financial consistency; it does not
establish efficient specialization, long-run welfare or performance at large scale.
The lawful elected pair and the legacy larger economy deliberately exercise
different combinations: the legacy exchange driver still cannot be combined with
state transaction law, and the larger run retains its default committed-work policy.


### Annual accounting result (2026-09-28)

The normally ignored `specialist_households_reconcile_production_trading_and_annual_dues`
test was explicitly run through 13 months on CubeCL CPU. All eight households and
32 specialist people completed production, trading, pooled transfers, issuance and
annual dues through the audited settlement adapter. Finalized separate statements
for every agent satisfy assets = liabilities + equity. The test passed in about
nine minutes in this debug build.

Together with the 75 focused checks, this gives **76 passing checks**, plus the
executable CPU/reference example and strict Clippy. The annual run does not compare
32-person CPU/reference ledgers; that equality is checked in the small integrated
fixture, and this larger run checks accounting consistency under the existing
default policy. It does not validate new-law/legacy-market interoperability or
32-person NeedsFirst performance. Those limits remain explicit above.


## Subsequent extension: adult membership

[Household membership](HOUSEHOLD-MEMBERSHIP.md) adds explicit unanimous accession
and member-requested exit at Open, dated rosters, retained governance history,
updated storage sharing and household-specific fractional collection. Property and
debts stay separate. Market recruitment remains outside this slice. A subsequent
[solvent dissolution extension](HOUSEHOLD-DISSOLUTION.md) handles last-member
wind-down and residual stock; asset and insolvency estates remain outstanding. The original validation counts above describe the earlier governance
completion; membership verification is recorded in the extension document.
