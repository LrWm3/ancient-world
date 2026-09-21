# Citizenship as an enabling agreement

Status: implemented in the one-person `opportunity-farming` CPU scenario.

The person keeps its `Person` agent type. The state posts one reusable citizenship
offer, eligible to Person agents. Acceptance creates a durable membership agreement:

```text
member: Person 88
organization: State 0
role: Citizen
source offer: 1
accepted month: 1
```

This accepted agreement is the authoritative relationship record, keyed by
(member, organization, role). There is no separate membership copy that can drift
away from the agreement. Each eligible person may accept the posted offer once;
acceptance does not consume the offer globally.

The state policy now grants **land-agreement access and cultivation through Citizen
membership**, rather than directly through the Person type. Person baseline
permissions allow membership acceptance, firewood collection, eating and fuel use.
Membership grants are scoped to the issuing organization. This experiment has one
state jurisdiction; spatial jurisdiction resolution is not implemented.

Citizenship has no payment, upkeep, expiry, breach conditions or automatic exit.
It changes no stock balance and creates no tax obligation. Annual grain taxes still
belong to the separately accepted land agreement. Unpaid land taxes block new
cultivation under the land rules but do not revoke citizenship. Permission remains
distinct from actual access and feasibility: citizenship supplies no plot, seed,
food or labor.

## Discover, forecast, accept

Noncitizens can see cultivation and land offers whose permissions can be obtained
through an eligible membership offer. Discovery therefore includes locked but
obtainable opportunities; `permits` reports actual current authorization. Preparation
and settlement enforce current membership, even if an offer is visible.

The planner follows:

```text
food need -> cultivation -> land access + cultivation permission
          -> citizenship offer -> land agreement -> cultivation
```

At Acquire, it compares doing nothing, accepting membership alone, accepting an
already-permitted land agreement, or accepting membership and land together. Each
candidate's forecast first creates the relationship, then checks the land offer,
then executes the existing monthly work/consumption schedule. Unpaid forecast
obligations exclude new land agreements, as in the preceding experiment.

Membership and land can settle in the same acquisition batch. Their order is
explicit: validate citizenship, stage membership, validate land with those staged
permissions, then publish together. If either acceptance fails, neither agreement
is published. The selected dated production plan executes at the next existing
barrier. This is an atomic prerequisite bundle, not a scheduler change.

There is still a bounded policy portfolio, not a general contract solver. A batch
supports at most one new membership and one land agreement. Indirect chains of
multiple memberships, concurrent state jurisdictions, membership withdrawal and
law changes are outside this slice. Accepted agreements retain their source offer
in the catalog for checkpoint validation. ZIP pricing remains unimplemented.

## CPU result and controls

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked -- opportunity-farming
cargo +1.92.0 test --locked --test membership -- --nocapture
```

The 36-month run accepts citizenship and land access in month 1, retains exactly
one membership, completes five harvests and has a sixth crop active at the end.
Both annual two-grain payments are met, with zero nutrition or warmth deficits.
CPU and reference ledgers agree, and a checkpoint immediately after acquisition
replays identically under monthly versus batched continuation.

Controls verify visible-but-unauthorized offers, agent-type eligibility, missing
membership offers, denied membership acceptance, missing seed, unaffordable land,
organization scope, duplicate acceptance, and atomic rollback of a failed land leg.
A separate arrears control retains citizenship despite unpaid land debt. The
preceding direct-type permission fixture remains available through
`opportunities::scenario()` for regression comparisons.

Validation: all eight membership tests pass, alongside the existing opportunity,
access, commitment, planning, condition and basic economics test groups. Clippy
with warnings denied, formatting and repository artifact checks pass. A reusable
offer control admits two distinct people and creates no membership dues across
an annual boundary.
