# State governance: bounded legal policy

Implemented 2026-10-05 as an opt-in extension beyond the verified v1 release.
State agents now have person governors and a constitutional policy menu. Their
decisions constrain the ordinary person and household admission paths. They do
not create a second population, labor supply, treasury or accounting system.

## Model and authority

`World.state_governance` identifies the existing state agent and its founding
month. The state must be the transaction policy's authority and have state type.
Its founding governor must be a person with that state's accepted citizenship.
Older fixtures initialize this institution explicitly. The subsequent
[state-founding adapter](STATE-FORMATION.md) creates a fresh state, initial law,
governance and founder citizenships atomically from a signed bootstrap template.

| Record | Responsibility |
| --- | --- |
| Constitution | Fixed leadership method and menu of named legal policies |
| Charter | Static founder, term length, election parameters and initial policy |
| Citizenship agreement | Eligibility for state office and voting; independent of household membership |
| Accepted ballot | A citizen's supplied preference for a future regular term |
| Accepted policy instruction | Who authorized a permitted policy, when issued, and its future effective month |
| Authority observation | Current governor, term/election, effective policy and instruction provenance |

Leadership supports a fixed founder, scheduled rotation and elections. The
founder holds the first term. Elections use the same ballot records, turnout
calculation and tally implementation as households, preserving the existing
household API. A charter selects stable-ID or vacant-office election ties;
abstentions count toward turnout without voting for a candidate. Citizen/voter
ordering is deterministic.

Regular terms admit citizens accepted before the opening month. Joining during
that month cannot change an already opened election or rotation. Founding citizens
are supplied at initialization or granted by the founding agreement. A governor dying during a term vacates the office;
there is no emergency succession yet. Historical eligibility uses dated deaths,
so a later death cannot erase a previously valid vote or policy instruction.
Vacancy leaves the effective law in force but authorizes no new instruction.

State office does not confer household office, or the reverse. The same person
may hold both, but each instruction must pass the receiving institution's own
authority check. Holding either office confers no ownership of institutional assets.

## Legal policy and timing

A policy contains additional prohibitions by action and optional agent type.
Its menu is fixed in the constitution. Governors may select an option, but cannot
edit the menu, charter, base permissions, fixed legal constraints, agreement
recognition or contractual terms through this API. An empty prohibition set only
removes this policy layer's restrictions; it grants no missing permission or right.
Policies compose with the existing `laws::evaluate` path, returning a
`StatePolicy { policy }` reason when they deny an action.

The fixture's two options are **open admissions** and **pause new land and
household agreements**. Accepted land rights, ongoing crops, household operation
and existing payment obligations survive this admission pause. This is a property
of these policy terms, not blanket immunity from every future law: a configured
process-execution prohibition uses the existing checks on ongoing work as well.
There is no contract cancellation or debt forgiveness implied by changing policy.

Instructions must be authorized by the current living governor and take effect
strictly after the current month. There is at most one instruction issued per
effective date in a given month. A later authorized instruction may supersede a
pending date; the earlier record remains in history. The latest effective date,
then latest issuance month, determines policy independently of vector ordering.

This follows the existing monthly boundary rather than adding a scheduler:

1. At the dated month's Open, policy lookup reflects accepted instructions.
2. Discovery, feasibility and acceptance read that month's policy. Known pending
   instructions remain visible in ordinary forecasts.
3. Existing allocation resolves actual resource contention and settlement rechecks
   authorization. A governor does not award a plot merely by opening admissions.
4. Completed production and consumption are unchanged by a later instruction.

Policy activation moves no balances. It is a dated read, so it cannot run twice
under batching or disappear across an in-memory checkpoint. Constitutions,
charters, ballots and instructions travel with `World`; economic state travels
with `State`, matching the existing household checkpoint convention. Durable
serialization of the complete simulation remains separate work.

The external settlement observer emits `state_governance` once per observed,
committed Open, with governor, leadership method, election and policy provenance.
It does not observe private forecast executions or insert telemetry into planners.

## Demonstrated outcomes

`state_governance::scenario::pair()` supplies two citizens, two land offers,
two-month electoral terms and ballots electing the second person in month three.
Political preferences are supplied inputs, not a claim of autonomous political
reasoning. Tests exercise these matched controls:

| Control | Result |
| --- | --- |
| Admit both farmers in month one, pause new admissions from month three | Over 25 completed months: two accepted leases, eight completed harvests, eight grain units of dues paid, neither person dead. State, economic ledger and reports match the unchanged-law control. |
| Start paused, incoming governor reopens in month four; add three opening grain per person to cover the pause | No lease before month four; land accepted by the six-month cutoff. CPU and reference results, ledgers and reports match across monthly continuation and checkpoint validation. |
| Same reopening with the original five opening grain per person | No lease by month six. Independent search rejects farming plans for forecast production failure, and deferred alternatives for unpaid future land dues. Legal access alone does not repair missing subsistence cover. |
| State pause/reopening plus an existing elected household's needs-first instruction | Existing household work continues. The six-month CPU/reference comparison agrees, food production completes, and separate person/household/state statements reconcile after every boundary. |
| New household formation during the pause, then after reopening | Paused formation rejects without mutation; month-four formation succeeds with a dated legal admission receipt. |
| Different people hold state and household office | Cross-institution instructions reject; each actual governor can set their own institution's permitted policy. |

Authority controls additionally cover missing citizenship, non-person candidates,
dead participants, invalid terms/policies, duplicate ballots/instructions, ties,
turnout failure, later entrants, historical death handling and a successor replacing
a pending instruction. Reversing instruction order preserves effective precedence.

Verification: 15 state-governance tests plus 103 existing household, household
integration, citizenship, laws, agreement-law and telemetry tests passed. Strict
all-target Clippy, formatting and repository artifact checks also passed. This is
scoped verification, not a replacement for the historical full v1 suite.

From `exp/economics`:

```sh
cargo +1.92.0 test --release --locked --test state_governance --test households --test household_integration --test laws --test membership --test agreement_laws --test telemetry
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
cargo +1.92.0 fmt --check
```

## Remaining work

The next bounded step is a **state decision policy**: compare allowed options
against observed citizen/household need coverage and finite state resources,
submit the chosen instruction through this authority interface, then compare it
with an unchanged-policy control. The admission-pause failure provides a useful
case for testing whether a governor considers transition costs.

Still outside this slice:

- Autonomous voter preferences, state objectives, policy search and spending;
  minting and issuance continue using their configured policies.
- Founding under another sovereign, citizenship exit, councils, emergency succession and
  constitutional or charter amendment.
- Multiple jurisdictions, delegated lawmaking, general changes to recognition,
  household constitutional limits, tax terms and liquidation law.
- Universal composition with every market/finance/private-forecast adapter.
  Some individual projections trim peer lifecycle records; preserving public
  political history through those projections needs a separate integration pass,
  especially around deaths and later elections. The mixed controls here cover
  competitive farming and the fixed individual work/household governance fixture.
- Automatic estates, children, person self-directed policy changes and broad
  political balance claims. The two-month terms are a small test setting.
