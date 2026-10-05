# First-state founding agreement

Implemented 2026-10-05, building on [state governance](STATE-GOVERNANCE.md).
Living persons can explicitly found a new state, becoming its first citizens
under agreed constitution, charter and initial law. The existing person governor
then holds office according to those terms. Founding does not make that person an
owner of the state or merge citizens' resources into a treasury.

## Admission and terms

`World.state_founding` is an optional, scenario-supplied bootstrap template:

| Field | Meaning |
| --- | --- |
| Agent | Fresh state identity and name |
| Eligible founders / minimum founders | Who can sign, and how many distinct signatures are required |
| Constitution | Fixed leadership method and permitted legal-policy menu |
| Charter | Static founding governor, term length, election rules and initial policy |
| Law | Initial typed roster, action permissions, recognized agreement forms, legal limits and membership grants |
| Citizenship offer | The state's ordinary person-citizenship offer, also identifying the founders' grants |

The typed roster and eligibility are explicit scenario inputs: before the first
state there is no existing sovereign to license its founders. A labor component
does not imply personhood. The configured founding governor must sign, and every
signer must be a known, living person in the eligible set. The new law, including
the initial constitutional policy, must permit their membership action.

The caller supplies consent. Passing a signer list means those persons have
agreed to these exact terms; no agent political planner chooses this list yet.
It is a unanimous agreement among the listed signatories, not a majority vote
that binds everyone eligible to sign. Signature order grants no priority.

```rust,ignore
use economics_compute_smoke::state_governance::formation;

// Scenario has supplied world.state_founding and existing person agents.
let proposal = formation::propose(&world, &state, &signatories)?;
formation::accept(&mut world, &mut state, proposal)?;
```

Proposal is read-only and performs full staged feasibility. Acceptance rechecks
the live boundary, exact offered terms and living founders. It publishes the new
agent, initial law, governance, accepted founding receipt and founding citizenships
together, or publishes nothing. Duplicate acceptance, existing authority, reused
identity, changed offer, stale month and non-Open acceptance reject.

Founding is an explicit **Open-boundary** operation, before monthly plans and
reservations. Its law is visible to that month's normal execution. It does not
advance a phase or add another scheduler. Later governor instructions retain the
existing future-month rule. There is no permission to found another state merely
because a person can join this one.

The optional `Template.agency` field supplies a [shared organizational operating
mandate](STATE-AGENCY.md). Founders sign its objective order, review parameters,
static preferences and finite program catalog along with the other terms.
Acceptance validates the program against the new constitution/law and installs
its controller atomically. Later mandate edits invalidate the accepted formation.
This does not imply autonomous founding consent or capitalization.

## Property, citizenship and continuing governance

The state starts without resources, assets, obligations or a participant labor
endowment. Existing holdings and ownership remain unchanged, including privately
owned land. Dangling accounts/assets cannot be legitimized by assigning their
previously unknown owner the new state's identity. Initial capitalization,
land contribution and taxation are not implicit founding effects.

Founders receive ordinary citizenship records, with the founding month and posted
citizenship offer as provenance. Later citizens use normal discovery and Acquire
settlement; joining does not retroactively make them founding signatories.
Regular electoral eligibility still uses citizenship predating the term opening.
Governors have the existing fixed, rotating or elected terms and dated policy API.

The accepted founding receipt snapshots the constitution, charter and initial law.
Validation rejects edits that disagree with those terms. New persons and household
agents may add classifications without changing founding classifications or law.
Selecting a dated constitutional policy is independent of that immutable base.
Later mortality does not erase valid founding consent or citizenship history.

`agreements::for_agent` includes `Identity::StateFormation(state_id)` for founders
and the state. Its view exposes founding terms and current authority. It has no
distinguished creditor/debtor and generates no payment claim. Existing citizenship,
household and financial agreements remain separate. This is a domain admission
adapter, like household founding, rather than an ordinary single-person market
order or a new financial contract executor.

## Verification and bounds

`tests/state_formation.rs` covers read-only proposal, deterministic signature
ordering, atomic rejection, changed terms, insufficient/duplicate/ineligible
signatures, death before acceptance, stale month, wrong phase, existing authority,
identity collision, invalid citizenship law, immutable accepted terms and later
death. A private-asset control checks that founding does not seize property.

The mixed control founds the state, forms a lawful household and elects the second
person as state governor. Six months of actual household/person production agree
on CPU and reference backends. A reconstructed checkpoint after two months gives
the same final world and state under monthly continuation. Accounting starts
before founding and reconciles through subsequent execution. Another control
pauses household admission, rejects formation atomically, and admits a later
citizen through the ordinary offer interface while preserving the prohibition.

Verification: nine founding tests and 118 existing governance, household,
integration, citizenship, law and telemetry tests passed. Strict all-target Clippy,
formatting and repository artifact checks passed. This is scoped evidence, not a
rerun or expansion of the historical v1 release:

```sh
cd exp/economics
cargo +1.92.0 test --release --locked --test state_formation --test state_governance --test households --test household_integration --test laws --test membership --test agreement_laws --test telemetry
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
cargo +1.92.0 fmt --check
```

Still outside this slice: autonomous founding/recruitment, negotiation of terms,
capital contributions, ownership issuance, land transfers, secession, multiple
sovereigns, territorial jurisdiction, recognition by other states, citizenship
exit, state dissolution and constitutional amendment. Older explicitly initialized
state fixtures remain supported without invented founding receipts. The subsequent agency integration preserves observed citizenship and deaths in
private economic forecasts; see its scoped controls and remaining limits.

When an audit already exists, use `Audit::accept_state_founding` in place of
the unobserved `formation::accept` call. It requires the exact audited opening
state and runs the same atomic admission before retaining the new citizenship
boundary. Its book and opening entries are unchanged: formation has no financial
legs. This prevents an arbitrary checkpoint reset from concealing changed balances.

The subsequent agency tests add founding with a signed operating mandate, automatic
ballots alongside household production, and rejection of an unconstitutional
program. The nine-test count above records the original founding slice.
