# Household agents

Status: implemented first adult-only pilot in `exp/economics`. Run
`households-32` for 32 people organized into eight households of four. This is
opt-in; the existing independent-person scenarios retain their previous rules.
Formation is explicitly configured, rather than simulated marriage or household
search. No children have been added.

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

## Household decisions about spare labor

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

1. Reserve pooled goods and any spare labor against the observed boundary.
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

### Observed results

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
