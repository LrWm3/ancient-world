# Household agents

Status: implemented first adult-only pilot in `exp/economics`. Run
`households-32` for 32 people organized into eight households of four. This is
opt-in; the existing independent-person scenarios retain their previous rules.
Formation is explicitly configured, rather than simulated marriage or household
search. No children have been added.

## Governance redesign: first implemented slice

The representative `households-32` scenario now uses the
[project goals](GOALS.md)'s 20% contributed-labor charter. The older spare-labor
behavior remains available through `Governance::legacy`. Founding templates,
static charter parameters, a named person holding policy-setting authority, and
swappable labor policies are implemented. Opt-in rotating terms and deterministic
succession now extend this foundation. Elections, household law recognition and
general institutional markets remain future work.

`Agreement.governance` separates:

| Component | Current role |
| --- | --- |
| Constitution | Permitted operational policies and an optional productive-activity subset. Selects fixed-founder or rotating leadership independently of operational policies. |
| Static charter | Founding governor, term length for rotation, labor contribution percentage or legacy spare-labor mode, labor tie-break and initial policy. |
| Policy instructions | Governor-authorized changes to an allowed policy, effective in a specified future month. |
| Operational allocation | The household evaluates feasible member work; the governor does not name individual jobs or recipients. |

Constitution and charter are founding configuration. There is no amendment API.
`household_governance::schedule` atomically accepts a future policy instruction
from the living current governor and verifies the constitution. Accepted instructions
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

This is a deterministic rotation/succession pilot, not elections, hereditary
succession, contested authority or resignation. Membership is still the fixed
adult founding roster. General lawful founding rules, household employment and
credit integration, and need-aware collective planning remain outstanding.

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

Two operational choices currently share a bounded, current-month net-output
progress score (quoted spot values, otherwise par; duration-adjusted):

- `PreserveCommittedWork` additionally rejects reallocations that displace a
  member's already-active process progress achievable without delegation. This
  is the representative default.
- `NetOutput` permits that displacement if the aggregate score improves. Normal
  missed-work consequences still execute; the household does not suppress them.

Neither policy yet optimizes all needs or six-month wealth. Both require a strict
improvement over the ordinary unreserved baseline and otherwise return all hours.
At most one recipient is chosen per household per month. This is a bounded search,
not a complete household process planner or a guarantee of economic sustainability.
Donated hours remain fungible capacity in the existing executor: the recipient
performs the work and gains practice. Contributor-specific skills and multi-worker
execution are not modeled.

Equal-score recipient and donor ties use an explicit static charter choice:
`MemberId`, `Rotating` (sorted living IDs, rotated monthly from founding), or
`SignatoryOrder`. Rotating allocation is independent of the constitution’s
governor rotation; either can be selected without the other.
Pooled-goods allocation still uses its existing benefit/reservation-order rule;
these new policies currently govern labor only.

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
future household support and is not a joint household credit assessment. Shared
production targets, pooled external sales, general bilateral barter, migration,
voluntary exit, inheritance, children, and optimal multi-period household plans
remain outside this first version.

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
