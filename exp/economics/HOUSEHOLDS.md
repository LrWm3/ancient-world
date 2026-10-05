# Household agents

Status: implemented first adult-only pilot in `exp/economics`. Run
`households-32` for 32 people organized into eight households of four. This is
opt-in; the existing independent-person scenarios retain their previous rules.
Formation is explicitly configured, rather than simulated marriage or household
search. No children have been added. The [governance basics checklist](HOUSEHOLD-BASICS.md)
tracks integrated completion, reproducible examples and remaining extensions.

## Governance redesign: first implemented slice

The representative `households-32` scenario now uses the
[project goals](GOALS.md)'s 20% contributed-labor charter. The older spare-labor
behavior remains available through `Governance::legacy`. Founding templates,
static charter parameters, a named person holding policy-setting authority, and
swappable labor policies are implemented. Opt-in rotating terms and deterministic
succession and explicit-ballot elections now extend this foundation. Autonomous
voting preferences and general institutional markets remain future work. State law
now gates founding and bounds the accepted constitution and charter.

`Agreement.governance` separates:

| Component | Current role |
| --- | --- |
| Constitution | Permitted operational policies and an optional productive-activity subset. Selects fixed-founder, rotating or elected leadership and permitted allocation tie-breaks independently of operating objectives. |
| Static charter | Founding governor, term length, election turnout/tie rules, labor contribution percentage or legacy spare-labor mode, initial labor tie-break and initial policy. |
| Policy instructions | Governor-authorized objective and optional labor tie-break changes, effective in a specified future month. |
| Operational allocation | The household evaluates feasible member work; the governor does not name individual jobs or recipients. |

Constitution and charter are founding configuration. There is no amendment API.
`household_governance::schedule` atomically accepts a future objective instruction
from the living current governor and verifies the constitution. `schedule_allocation`
can bundle an allowed labor tie-break into that same instruction. Omitting a tie
retains the last effective choice. Both APIs preserve the founding documents. Accepted instructions
retain their issue month as well as their effective month and author. Authority
is validated at issue time, not against whoever governs when the policy becomes
effective. Two instructions issued in the same month for the same effective month
are rejected. A later-month authorized instruction can supersede that effective
month while preserving the earlier record. The effective policy uses the latest
effective date, then the latest issue date; row ordering cannot choose policy.
Already accepted instructions survive leadership changes and deaths. As with other
consented fixtures, callers supply instructions; no personality model autonomously
chooses policy changes.

## Rotating governance and succession

`Governance::contributed` retains a fixed founding governor by default.
`Governance::rotating(founder, term_months)` selects the rotating constitution;
for annual terms use `DEFAULT_TERM_MONTHS` (12). The charter's named founder is
the starting point, not a mutable record of the current office holder.

Terms start at founding and advance at Open after each configured interval.
Eligible adults follow sorted stable IDs, starting with the founder. Signature
order and the labor tie-break do not determine governance. A two-adult household
founded in month 1 with twelve-month terms changes governor at months 13 and 25.
The constitution and charter remain unchanged throughout.

The rotation uses the original adult roster. At each Open it skips members whose
terminal transition occurred in an earlier month, choosing the next surviving
member in that ring. A death does not reset the term calendar: a replacement can
therefore also hold the next scheduled term. There is no same-month retroactive
replacement authority. The fixed-founder variant instead leaves the office vacant
after the founder dies. With no survivors, neither variant selects a governor and
the household ceases operating under the existing rule.

A vacant office does not cancel previously accepted policy. Living members can
continue operating under it; nobody gains new instruction authority merely from
receiving household labor. The governor sets permitted policy, not individual jobs,
asset ownership or debt responsibility. Leadership rotation does not consolidate
member financial statements or transfer their property to the household.

`Boundary.governance` records the current governor (or vacancy), mechanism, term
start and effective policy at Open. Labor receipts also name the actual governor.
Replay reconstructs this evidence before publication, and the settlement observer
exports `household_governance` records, including when filtered by a member.
No new monthly phase or governance resource budget was introduced.

Rotation is a deterministic succession pilot. Hereditary succession, contested
authority and office resignation remain unimplemented. [Dated adult admission and
exit](HOUSEHOLD-MEMBERSHIP.md) now extend the fixed founding record. Outside member employment, private town sales and direct town lending now compose
in [bounded integration tests](INTEGRATION-PASSES.md). General institutional
founding, autonomous hiring discovery and longer-horizon collective planning remain
outstanding. Explicit budgeted hiring is now supported as described below.

## Elected governance

`Governance::elected(founder, term_months)` opts into scheduled plurality elections.
The founder serves the first term; later terms start on the same Open calendar
as rotation. The default election charter requires 50% turnout (rounded up) and
breaks equal vote totals by stable member ID. `ElectionTieBreak::Vacant` instead
leaves tied elections unresolved. These are static founding parameters, separate
from the labor allocation tie-break.

`household_governance::elections::cast` accepts one immutable ballot per member
per future regular term. The caller supplies a consenting member's candidate or
explicit abstention. Voters and candidates must be living current members when
accepted. Unknown members, duplicate ballots, off-calendar elections, late ballots
and non-elected constitutions reject atomically. This is a supplied-ballot pilot:
there is no autonomous voting preference, campaigning, secret ballot protocol,
ballot revision or election labor charge.

At a term's Open, only adults alive at that boundary comprise the electorate.
Ballots from people who died in an earlier month do not count. A living voter's
abstention, or vote for a candidate who has since died, counts toward turnout but
not toward a candidate. No quorum, no candidate votes or an unresolved tie leaves
the office vacant; the incumbent does not automatically continue. Previously
accepted operational policy still applies.

An elected governor's later death leaves the office vacant until the next regular
election. There is no by-election or immediate runner-up succession. Historical
election results use eligibility at the original term boundary, so a later death
does not rewrite the winner or invalidate instructions issued while alive.
As with other governance authority, a Close-month death affects eligibility at the
following Open; dead agents cannot submit new instructions or ballots.

Open receipts retain eligible voters, turnout, required turnout, candidate totals
and the elected winner, separately from the current living governor. Replay
reconstructs these fields and rejects altered tallies. Settlement observer records
include the election evidence and honor household/member filters. There is no
new scheduler phase. Leadership changes do not transfer ownership, consolidate
books or choose individual work assignments.

## Needs-first contributed labor

The later [combined agency control](INTEGRATED-AGENCY.md#optional-commitment-preparation)
adds the opt-in `NeedsThenCommitments { months }` policy: current needs, accepted
collective claim cover, then net output. It adds direct productive demand and uses
signed surplus funding at Acquire, without changing the default or appropriating
private holdings. Its dated receipts, shortage controls and accounting checks are
recorded there.

`Policy::NeedsFirst` is an opt-in operational policy permitted by the contributed
labor constitution. The default remains `PreserveCommittedWork`. A founding
charter may select it, or the current governor may schedule it through the existing
dated policy interface. Electing someone does not select this policy implicitly.

At the existing Productive reservation boundary, the household compares its
unmodified plan with each feasible recipient of the bounded contribution pool.
It keeps the existing mandate and committed-work checks: a proposal cannot abandon
a process that the baseline would continue successfully. It then ranks candidates
by projected **current-month unmet needs**, followed by the existing net-output
value proxy. A reduction in need deficit can justify producing less valuable output.
When the needs comparison is equal, additional net output wins; otherwise the
charter's labor tie-break retains its existing role.

Each forecast settles the candidate production on a private reference-backend copy,
collects actual household shares, clears arrears where scheduled, allocates common
stocks and executes consumption through the existing mechanisms. It stops before
Close. Missing inputs, whole consumption lots, storage, earmarked goods and available
consumption recipes therefore constrain the forecast. A future crop output is not
food available this month. The forecast creates no published transactions or logs;
actual execution still uses the dated reservation and normal settlement path.

Deficits combine members only within a common `(need priority, fulfillment resource)`
key. Smaller priority numbers compare first, with resource ID breaking equal ranks.
The comparison is lexicographic: food and warmth units are never summed into money
or a single utility score. This is an explicit bounded objective, not an assertion
that those rankings are universally appropriate. Protection of existing committed
work remains a hard constraint even if overriding it could improve immediate needs.

`LaborDecision.baseline_needs` and `projected_needs` record these forecasts, and the
settlement observer exports both. Other policies leave them absent. Replay checks
the entire decision receipt, including the forecast. These are conditional plans,
not guaranteed outcomes: later changes to shared opportunities can invalidate a
forecast. The policy still considers one recipient per household boundary and uses
members' existing candidate plans; it does not search arbitrary new collective work
combinations, forecast long-term deprivation, or automatically choose voting or
policy preferences. It also does not change household financial reporting scope.

## Agreement and continued existence

A household is an ordinary agent ID with its own inventory and an accepted
formation agreement. The agreement records its ID, household agent, formation
month, and consenting adults in signature order. `households::form` atomically
validates formation at the current boundary. One to four distinct, living adults
may sign. Signatories are declared adults by scenario configuration; age-driven
eligibility belongs to the later demographic model. Each person belongs to at most one household; nested households are
not supported. People retain their identity, needs, expertise, equipment,
processes, use rights, and individual debts.

A household is active while at least one member remains a living adult. Terminal
members stop reserving goods, contributing income, receiving dwelling services,
and supplying labor. With no surviving adult, the household ceases operation;
its historical agent/agreement and estate balances remain for accounting. There
is no inheritance or estate liquidation policy yet.

The demographic extension has a distinct allowance: **four additional adults
who grew up in the household may remain upon adulthood**, separate from the four
founding/admitted adult places. `GROWN_CHILD_ADULT_SLOTS` records that policy;
there is no child or maturation implementation yet. Future children have no
numeric household limit. Their actual material requirements must be supported,
and a child-only household cannot remain active without an adult.

[Adult accession and voluntary exit](HOUSEHOLD-MEMBERSHIP.md) now operate at Open.
They retain the founding record, property and debts, and recheck shared storage.
Recruitment offers remain an extension. [Solvent last-member dissolution](HOUSEHOLD-DISSOLUTION.md)
now provides opt-in wind-down and stock distribution under static founding terms.

## Half of receipts, with separate ownership

Actual productive stock receipts, spot purchases, sale proceeds, and tool-sale
proceeds contribute half to the household inventory. The other half remains
personal. These are actual receipts, not forecast output. Costs of the same
resource in the same transaction are netted before calculating the receipt.
Returned seed is a stock receipt and can subsequently be reserved for planting.

Small indivisible ticks carry their fractional contribution forward per member
and resource: two receipts of one tick contribute one tick in total. The carry
is part of checkpointed state and replay validation. Fractional claims do not
mint stock or exceed the physical receipt.

Opening endowments stay personally owned. Household distributions are not
income and are not pooled again. Borrowing is not income: advances still go
straight to the tool provider, whose earned payment is pooled if that provider
is a member. Fulfillment and perishable service tickets are not pooled as stored
wealth. Durable tools retain individual ownership; a dwelling can provide a
shared service as described below.

## Requests, reservations, and internal priority

Members submit requests for pooled goods with quantity, minimum useful grant,
individual benefit, collective benefit, purpose, and reservation sequence.
Allocation requires positive individual and collective benefit, and then sorts
by **descending collective benefit, followed by reservation order**. Competing
requests share one finite opening pool. Repeated claims for the same member and
resource do not add duplicate entitlement; the requested total is capped against
what that member has already reserved. A grant below its useful minimum is not
reserved. Receipts retain both the request and the actual allocation.

This first policy estimates benefit rather than solving joint utility:

- Immediate need requests use current deficits and deprivation, weighted by the
  existing need priority. Alternative foods use the existing recipe catalog.
- Missing inputs for members' active stages or requested activities are next.
- Support for due obligations follows these internal requests.

Goods move to the member before the relevant ordinary resolver runs. Consumption,
productive feasibility, money, equipment, rights, and storage are still checked
there. A resource grant does not guarantee that a multi-input process completes.
Requests do not yet evaluate complete alternative household plans over a year.

Current need inputs are protected from outside spot sales and debt settlement
for household members, including under the otherwise debt-first policy. Internal
support can therefore precede external claims. This does not cancel those claims:
unpaid amounts remain recorded. Internal pooled distributions require no coins;
private inventories can still participate in the existing market.

## Storage and a shared dwelling

Each member keeps exclusive access to half their storage capacity. The other
half enters a common capacity pool. Household-owned inventory and members'
usage above their private half compete for that same common space. For capacities
`C_i` and physical storage use `U_i`, the shared constraint is:

```text
household inventory space
+ sum(max(0, member usage - member private capacity))
<= sum(member contributed capacity)
```

Personal inventories may use otherwise free common space; unused private space
is not silently donated. Capacity is never duplicated. Existing resource weights
still apply, and coins require no space. Legacy unconstrained stores remain
unconstrained. Production and exchanges reserve room for the mandatory income
contribution before proceeding, with a conservative rounding bound for tiny
indivisible receipts.

The agreement can name the catalog's dwelling-occupancy process. When a living
member actually completes occupancy using an accessible, functioning dwelling,
its non-rival shelter service is available to every living member that month.
The dwelling is used/worn by the single actual process, not once per resident.
Without a completed occupancy service there is no household shelter credit.
Ordinary expiry, consumption, and deprivation still apply. Sharing does not
confer the owner's plot rights on every resident. Existing personal home-building
orders are not yet coordinated into a collective construction target, so this
pilot allows one shared dwelling but does not prevent redundant construction.

## Supporting taxes and forwards

The household may distribute the native commodity or eligible coins to fund a
member's dated tax, and may supply goods for a due forward. The existing
commitment/forward resolver records the payment against the **original debtor**.
It retains creditor storage bounds, native-tax currency issuance rules, protected
stocks, partial payment, and overdue balances. Paying in coins does not mint new
currency. A household transfer itself does not discharge debt; only actual
settlement does. There is no automatic joint liability or debt reassignment.

## Contributed labor and operational policy

At the existing **Productive reservation boundary**, after pooled input allocation
and before ordinary productive work, each living member reserves
`floor(current available capacity * charter percent / 100)`. The representative
percentage is 20. This is the actual remaining capacity at that dated boundary,
not nominal healthy capacity or future regenerated hours. Earlier phases retain
their existing priority. Fractional labor ticks are not carried into another month.
Each person still has at most one household, preventing overlapping pledges here.

Reservations remove those hours from private budgets in candidate evaluation.
Only compatible capacity resources can be combined. The household tests directing
the pool to each member's already-requested or active work using the existing
feasibility resolver. Rights, equipment, inputs and permissions still belong to
the executing member. A constitutional activity subset limits the recipient's
whole funded plan conservatively; it does not grant extra permissions.

The recipient receives only the useful amount. Its own reserved hours are applied
first, then other contributions in the charter's order. All unused reservations
return to their contributors **before** execution. The final plan is probed again
after these returns, and the funded work must remain feasible. Atomic effects
publish only net transfers between members; the reservation receipts represent
the household's authority over hours, without creating a second spendable labor
account. Ordinary process receipts record actual completion and consumption.

The original three operational choices use a bounded, current-month net-output progress
score (quoted spot values, otherwise par; duration-adjusted):

- `PreserveCommittedWork` additionally rejects reallocations that displace a
  member's already-active process progress achievable without delegation. This
  is the representative default.
- `NetOutput` permits that displacement if the aggregate score improves. Normal
  missed-work consequences still execute; the household does not suppress them.
- `NeedsFirst` retains committed-work protection and compares projected current
  need deficits before the output score, using settlement previews described above.

Opt-in [`NeedsThenIncome`](HOUSEHOLD-INCOME.md) preserves current needs and
commitments, then compares expected collective cash through the next town book.
It requires explicit constitutional permission and contributed labor. Forecasts
use real market budgets and create no spendable receipts. The 12-month reciprocal
fixture funds food purchases; a longer run exposes the remaining conflict between
private stock targets and collective income demand.

None optimizes six-month wealth. Each requires a strict improvement under its
configured ordering over the ordinary unreserved baseline, or returns all hours.
At most one recipient is chosen per household per month. This is a bounded search,
not a complete household process planner or a guarantee of economic sustainability.
Donated hours remain fungible capacity in the existing executor: the recipient
performs the work and gains practice. Contributor-specific skills and multi-worker
execution are not modeled.

Equal-score recipient and donor ties use an explicit static charter choice:
`MemberId`, `Rotating` (sorted living IDs, rotated monthly from founding), or
`SignatoryOrder`. Rotating allocation is independent of the constitution’s
governor rotation; either can be selected without the other.
Pooled-goods allocation still uses its existing benefit/reservation-order rule.
The [town adapter](HOUSEHOLD-MARKET.md) also uses the effective objective to
authorize collective consumption purchases.

Dated labor receipts retain governor, policy, tie-break, baseline/projected score,
recipient and granted hours, plus each contributor's available, reserved, directed
and returned quantities. Directed includes a recipient's own contribution, while
net reassigned labor counts only transfers from other members. Neither measure
should be substituted for actual work completion. Settlement telemetry exports
`household_labor` records, including when filtered by a contributing member.

## Legacy spare-labor option

At Productive, the household first previews the members' ordinary work requests.
Labor those requests would use remains protected. It tests lending the remaining
compatible labor to each member, using the ordinary feasibility resolver. This
lets the household support activities a member already requests or operates,
without granting new skills, tools, sites, or technologies.

The bounded objective compares expected net-output progress at current spot
values (par value for unquoted resources), divided by process duration. Durable
creation/repair has a minimal positive proxy value. Among feasible improvements,
the highest household score wins; signature order breaks ties. A candidate may
not displace the other members' previously feasible work. Only the additional
labor actually needed is granted. At most one recipient per household is selected
in this monthly pilot; unused labor remains unused and expires normally.

The decision receipt records baseline score, projected score, recipient, and
labor granted. With no beneficial feasible activity, the household waits. This
is a transparent heuristic, not a global optimum, a wage contract, or a forecast
that output will definitely sell. Practice remains with the process operator;
contributing labor does not transfer expertise.

## Monthly integration and replay

No scheduler phase was added. Each existing batch has an optional household
boundary record:

1. Reserve pooled goods and charter-selected labor against the observed boundary.
2. Run the existing Due, Acquire, Productive, ClearArrears, or Consumption work
   against those granted resources.
3. Collect half of realized receipts and publish any shared dwelling service.
4. Publish the complete boundary atomically after validation.

Open still regenerates capacities and expires services. Close still evaluates
conditions and terminal states. Fresh production can support ClearArrears or
consumption later that month, but cannot retroactively fund the completed Acquire
market. Incoming pooled goods are available to the next reservation boundary.

Household before/after effects, fractional carries, reservation receipts, and
labor decisions are canonical ledger data. Replay rebuilds them and rejects
alteration, duplicate batches, overspending, and effect-buffer overflow without
partially publishing state. CubeCL CPU performs the balance gathers. Forecasts
and request matching remain host-side, as in the preceding pilots. Household
coordination currently requires a fixed individual work priority (the fixture
uses `NeedFirst`), rather than composing with the older `ConsequenceAware`
planner and its pending speculative batches. Unsupported combinations fail
validation at formation.

Individual tool/plot underwriting remains a local individual rollout; it excludes
future household support and is not a joint household credit assessment. Pooled
external sales, valued bilateral/town barter, bounded joint work and explicit
wind-down property transfers now have adapters. General migration, inheritance,
children and optimal multi-period household plans remain extensions. The
[integration matrix](INTEGRATION-STATUS.md) records the current combination limits,
including general household prerequisite search and cooperative purchasing.
Explicit prerequisite/process acceptance is covered by [v1 verification](V1-ACCEPTANCE.md).

## Running and verification

```sh
cd exp/economics
cargo +1.92.0 run --locked -- households-32
cargo +1.92.0 run --locked --config 'profile.dev.package.economics-compute-smoke.opt-level=2' --example household_audit -- 24 cpu
cargo +1.92.0 run --locked --config 'profile.dev.package.economics-compute-smoke.opt-level=2' --example household_audit -- 24 reference control
cargo +1.92.0 run --locked --config 'profile.dev.package.economics-compute-smoke.opt-level=2' --example household_audit -- 72
cargo +1.92.0 test --locked --test households
```

The focused tests cover membership limits and duplicate membership, benefit/order
contention, minimum grants and duplicate requests, shared storage conservation,
pooled consumption, fractional receipts, protected personal work, idle decisions,
actual shared shelter and wear, native/coin taxes, household-funded forwards,
last-adult death, atomic failed replay, CPU parity, monthly stepping, and in-memory
checkpoint continuation. Generated audit logs are kept under ignored
`output/economics/`.

### Rotating-governance validation

The focused run passes **41 checks**: 27 household, four household-accounting and
ten telemetry checks. The existing long household accounting stress test remains
ignored. Strict all-target Clippy passes. New controls cover annual term boundaries,
signature-order independence, historical authorization, future-policy supersession,
post-death succession, vacant offices, and forged Open authority receipts. A
four-month two-adult comparison using two-month terms matches CPU/reference,
monthly versus batched stepping and cloned continuation. Holding policy constant,
fixed and rotating governors produce identical economic state; changing the office
holder alone does not choose a different job or transfer property. Member-filtered
logs identify both governors across a handoff without changing execution.

This verifies bounded governance mechanics, not elections, autonomous policy
selection or better household economic outcomes. No new long 32-person calibration
was performed. Generated logs remain under ignored `output/economics/`.

### Governance validation (2026-09-24)

All **23 household tests and 10 telemetry tests passed**. New controls cover a
bounded 20% contribution, rotating ties after reversing signature order,
unused/out-of-mandate returns, future governor-authorized policy changes,
forbidden policies, indivisible-hour rounding and current-capacity shortfalls,
CPU/reference equality, continuation, tampered reservation rejection and
member-filtered observational equivalence. A controlled comparison uses identical
opening requests and resources: `NetOutput` completes a higher-value job while a
donor's active process aborts; `PreserveCommittedWork` declines the reallocation
and preserves that process. Allocation preference does not change execution order.

The representative eight-household/32-person scenario was run for **three months**
with `cargo +1.92.0 run --locked --example household_audit -- 3 cpu`. State, every
ledger batch and reports matched exactly between CPU and reference. It recorded
320 reserved labor ticks, 176 directed (including recipients' own shares), 144
returned and 96 net reassigned between members. There were no terminal people,
tax arrears, nutrition or warmth deficits; 64 startup shelter deficit units
remained. Three tools were purchased and no forwards issued. This short run does
not validate annual taxes, long-term finance, sustainable specialization or a
need-first collective objective. The longer legacy results below are not evidence
for the new default. Raw logs remain under ignored `output/economics/`.

Formatting, strict all-target Clippy and repository artifact checks passed. The
full economics crate suite was not rerun.

### Earlier spare-labor results

These results describe the legacy policy, not the new contributed-labor default.

The matched 24-month comparison uses identical opening resources, state prices,
three tool providers, and 32 adults. The treatment accepts eight household
agreements; the control leaves the same people independent.

| Measure after 24 months | Independent people | Eight households |
| --- | ---: | ---: |
| Terminal people | 0 | 0 |
| Unpaid tax ticks | 0 | 0 |
| Nutrition / warmth deficits | 0 / 0 | 0 / 0 |
| Shelter deficit units across person-months | 64 | 64 |
| Tools purchased | 20 | 16 |
| Forwards issued | 12 | 8 |
| Overdue forward commodity ticks | 0 | 0 |
| Spare labor reassigned, quarter-unit ticks | 0 | 226 |

The shelter deficits occur during startup. Household state, every ledger batch,
contribution carry, and monthly report match exactly between CubeCL CPU and the
reference backend. The 226 reassigned ticks are 56.5 labor units over the full
24-month run. Zero overdue forwards does not mean every issued forward has
matured by this boundary.

Pooling did not improve every measured outcome: fewer tools were purchased in
the household treatment. This is an observation, not a causal attribution to one
mechanism. Shared money changes private balances, and the first household planner
does not yet reserve coin budgets for new tool purchases. Personal production
buffers also do not fully account for common surpluses, so households can keep
producing goods they already collectively hold in abundance. The pilot establishes
working agreements and finite coordination; it does not establish efficient
specialization or long-run household equilibrium.

The extended 72-month reference run also ends with no terminal people, tax
arrears, nutrition/warmth deficits, or overdue forward deliveries. The same 64
startup shelter deficits remain; no additional shelter deficits accumulate.
It records 42 tool purchases, 22 forwards, and 2,658 reassigned labor ticks
(664.5 labor units). This longer run was checked on the reference backend;
the exact CPU/reference comparison covers 24 months. Large common inventories
remain, consistent with the production-buffer limitation above.

Validation passes all 133 tests, including 16 household tests, plus formatting,
strict Clippy, and the repository artifact check.

### Election validation

The election slice passes 46 focused checks: 32 household tests, four household
accounting tests and ten telemetry tests. The existing slow household accounting
calibration remains ignored. Strict all-target Clippy and formatting pass.

New controls cover plurality, rounded turnout thresholds, explicit abstention,
two tie rules on identical ballots, roster/ballot ordering, atomic rejection of
invalid votes, and deaths before and after election. Historical policy authority
survives the elected governor's subsequent death. A four-month, two-person case
matches CPU/reference execution, monthly/batched stepping and cloned continuation;
changing only the leadership rule leaves economic state unchanged. Forged tallies
reject without publication. Member-filtered observer output reports the tally and
does not affect execution.

These checks establish election mechanics and accounting regression safety. They
do not show that elected leadership improves welfare: voting and policy choices
are supplied, and no new long 32-person calibration was run.

### Needs-first validation

The controlled two-adult fixture gives each member five labor units and reserves
20%. Either a repair-material job or a food job needs six units; only the food
producer holds the required seed. Both members need one nutrition unit and begin
without grain. With identical requests and budgets, `NetOutput` funds repair
material and leaves two nutrition units unmet. `NeedsFirst` funds two grain units;
one is pooled, both members eat, and unmet nutrition is zero. The selected food
plan has a lower output-value proxy. These are abstract fixture units, not a
calibrated welfare estimate.

Controls cover sufficient opening food, unavailable seed, continuing-work
protection, and food-versus-warmth priority reversal. Receipt forecasts match actual
consumption. CPU/reference state and ledgers match through three months with
batched, monthly and cloned continuation; forged deficit receipts reject atomically.
The observer exposes only committed choices and does not alter results.

Validation: **52 checks passed** (38 household, four household accounting and ten
telemetry). The existing slow household accounting calibration remains ignored.
Strict all-target Clippy, formatting and repository artifact checks pass. The full
crate suite and a new long-run 32-person calibration were not run. The opt-in policy
adds private reference settlements per candidate; larger-scale performance has not
been established, and current-month need satisfaction does not establish sustainable
future production or debt repayment.


### Integrated governance completion

See [Household governance basics](HOUSEHOLD-BASICS.md) for lawful admission,
authorized objective/tie changes, the executable six-month election scenario and
current verification. Founding laws now coexist with the household driver;
recognized households do not acquire their members' process rights automatically.
Historical calibration sections above retain their original scope and dates.

## Solvent dissolution extension

[Household dissolution](HOUSEHOLD-DISSOLUTION.md) adds an opt-in constitutional
permission and static charter recipient. The last member can stop collective
operations, settle residual stock through Open receipts after obligations clear,
and then release membership/storage. Separate books and historical authority survive.
General household lending/recovery, asset dispositions and death estates remain
uncomposed; this path does not silently write off or transfer their claims.


## Initial economic loop and voluntary surplus

The [completed loop](PERSON-HOUSEHOLD-LOOP.md) connects personal needs, household
purchases, contributed work, market income and separate accounting for 120 months
on CPU. `Agreement.support` holds dated member-authored surplus mandates, independent
of governor authority. Accepted transfers protect personal reserves and supported
claims, require a useful need/income improvement by default, and settle before contributed
labor at Productive. Members may withdraw future support; the ordinary half-output
rule remains unchanged. The new receipts are replayed and observed alongside labor
receipts. See the linked document for the original failure control, recovery test,
composition limits and subsequent work.

## Further household/shared consolidation

The [second five-pass batch](INTEGRATION-PASSES-2.md) adds useful partial support,
physical barter pooling and two static charter parameters: `purchasing` selects
collective or member consumption bids, and `fund_committed_inputs` permits a
collective needs-first buyer to cover active member-process inputs. Current
membership controls bid eligibility. These options reuse existing governance,
allocation, settlement and separate accounting; they do not let leaders rewrite
personal policy or claim private money.


## Opt-in member loan assistance

Static charter `support_member_loans` allows current, active household members to
request collective stock for current collectible loan principal and interest at
Due. Existing native dues and loan amounts share one account requirement; the
member's stock offsets it. Assistance retains the household's own current loan
payment, and existing needs/input precedence is unchanged. It transfers resources,
not the debt, and does not pledge future income. Exit stops assistance.

`debt_support` defaults to `ReservationOrder`. `ClaimPriority` reorders only loan
support slots within each household/resource pool by the lowest covered creditor
rank, then stable member ID. Receipts retain the selected policy and rank. A
combined native request containing several loans or other dues gets that minimum
loan rank; this is coarse household assistance, not direct creditor-specific
escrow or a replacement for the collection waterfall. Separate creditor collection
still resolves actual payments. A request that cannot fit storage retains a zero
grant under the existing allocator. Future refinements can split claim-specific
support and add other policies without moving monthly phases.

`fund_due_loans` separately permits collective market purchases for these claims
under a needs-first objective. Current loans alone qualify; estate claims and
future installments require different adapters. See [passes 11–15](INTEGRATION-PASSES-3.md).

## Purchased labor and payroll

Static charter `hiring_budget: Option<Amount>` opts a percentage-contribution
household into preaccepted outside employment in one wage denomination. The budget
is a monthly maximum, constrained further by opening funds less earned arrears;
it does not change the 20% contribution or imply autonomous contract acceptance.

At Productive, paid household capacity can supplement member contributions under
the same objective, mandate, feasibility and personal-right checks. The household
pays; the selected member operates the process. Normal output pooling still applies,
so the household does not automatically own all output. `LaborDecision.purchased`
records available, directed and unused quantities by capacity resource. Paid basis
follows the hours into the member's production; idle hours and basis expire.

Existing earned wages reserve their stock before discretionary member allocations.
`fund_earned_wages: bool` (default false) enables collective needs-first acquisition
of payment stock for these claims. It is independent of current-loan funding and
member-loan-support settings. No hypothetical market fills or projected outputs
count as current funding. Earned-only wage demand is the default; the optional
[payroll outlook](PAYROLL-OUTLOOK.md) adds current-month funding estimates without
creating debt or spendable funds. See [employment](EMPLOYMENT.md) and the
[fourth combined verification batch](INTEGRATION-PASSES-4.md).

## Private member employers

Members may hire outside labor through preaccepted employment. Buying hours does
not enlarge their percentage contribution: own availability adds back this month's
outgoing employment delivery and subtracts acquired hours. The shared paid-capacity
adapter retains historical basis through member allocation, production and pooling.
Internal household employment remains excluded, including common past/future
membership; a household cannot itself deliver worker services.

Static `support_member_wages` defaults false. When enabled and governance is active,
current members request their earned native wage shortfall before Close payroll.
`debt_support` orders these wage slots independently of its Due loan slots, with
reservation-order or lowest covered claim rank then member ID. Collective own
payroll and current loan dues retain their stock before member wage assistance.
Transfers support payment; they never assume or discharge the personal liability.

`fund_earned_wages` can also buy missing stock for these supported claims under
collective needs-first purchasing. Individual holdings offset combined requirements
once. New wages are not forecast demand unless the charter opts into
`payroll_outlook: CurrentDelivery`. Assistance itself remains earned-only.
A member leaving loses subsequent
support while keeping their private contracts, assets and debts. See
[the fifth integration batch](INTEGRATION-PASSES-5.md) for controls and funding lag.

## Voluntary support for collective payments

Static `accept_payment_support` defaults false. Under needs-first operating
policies, existing personally signed surplus mandates can fund the household's own
earned-wage/current-loan/current-land-bill shortages when ordinary consumption/income evaluation
rejects the offer. A second candidate is capped at the shortage and rechecked for
protected needs. Donor reserves and personal claims remain protected; later donors
see the funded collective balance. The flag does not authorize a mandate.

Transfers run at the existing Productive support boundary and record transfer
expense/income. Land retains its after-Productive ClearArrears pass, wages collect
at Close, and loan money waits for Due. There is no
new member claim, liability transfer or payment guarantee. See
[combined cases, receipts and limits](HOUSEHOLD-PAYMENT-SUPPORT.md).
The [land extension](HOUSEHOLD-LAND-SUPPORT.md) shares collection claim rules and
tests limited donations and creditor storage without advancing future bills.
