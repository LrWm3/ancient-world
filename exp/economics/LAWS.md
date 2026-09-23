# Static legal constraints

The first legal layer builds on the existing transaction policy. A policy identifies
one authority, agent types, direct permissions, membership offers and permissions
granted by accepted membership. Its new `laws` catalog constrains those grants.
An empty catalog preserves existing classified-agent behavior.

Each named rule has a stable ID, an action, an optional agent type and one of:

- `Prohibited`: that action is forbidden even if a type or membership grants it.
- `Membership(role)`: the actor must hold that role in the policy authority,
  in addition to having a permission grant.

For example, a law may require citizenship for land access even when a person-type
permission grants access generally. Another law may prohibit land access altogether.
All matching requirements apply; there is no last-rule-wins override. A prohibition
cannot be lifted by accepting citizenship. Rules can target any existing action:
membership, land access, a named process, equipment trade, stock trade or financed
purchase. This does not enable unsupported legacy drivers.

## Evaluation and integration

`laws::evaluate(world, state, agent, action)` returns the authority, an allowed flag
and structured reasons: unrestricted legacy scenario, granted, unclassified actor,
missing grant, prohibition rule ID or missing membership with rule ID and role.
Multiple failed rules are returned in ID order. Rule names live in the catalog.
The existing `opportunities::permits` interface delegates to this evaluator, so
supported discovery, agreement acceptance, process settlement, marketplace and
credit permission checks share the same interpretation.

Agents under a configured policy must be classified, including when using a
membership grant. Scenarios without a transaction policy retain their explicitly
unrestricted legacy behavior. Laws never create a right, feasible plan, market
admission, labor or settlement funds.

Discovery distinguishes a lawful prerequisite from current authorization. It may
preview one obtainable membership and expose the opportunities that would become
permitted, using the same evaluator against the preview. The preview creates no
live membership or rights. If membership itself is prohibited, or the downstream
action remains forbidden, discovery does not expose that route. This matches the
existing single-membership acquisition bundle; it is not a multi-membership planner.

Existing boundaries remain intact. No scheduler or monthly law-update phase is
added. Acceptance and settlement recheck permission; a prepared productive batch
cannot bypass a prohibition. Aborting work remains possible under existing process
rules. Due debt and enforcement retain their existing handling independently of
permission to originate a new loan. This is not a new cancellation, restitution or
grandfathering policy for existing contracts.

## Recognized agreement forms

The authority can now configure `Policy.agreement_forms` with an explicit set:

| Form | Current meaning |
| --- | --- |
| `LandUseLease` | Accept the existing land-use agreement: dated use right in exchange for annual payment, with ownership retained |
| `FinancedAssetPurchase` | Accept an asset sale bundled with a collateralized coin loan; the current control purchases a plot |

`None` preserves recognition of both forms for earlier scenarios. An explicit
empty set recognizes neither. A lease-only state can retain land ownership;
a purchase-only state can refuse new leases while permitting financed purchase.
The latter form applies to financed assets generally, not only plots. This catalog
does not govern cash purchases, collateral resale, inheritance or other agreement
forms not yet connected to recognition.

`laws::evaluate_agreement` combines recognition with the existing action check.
A refused form produces `UnrecognizedForm` identifying that form and the policy
authority. Recognition does not supply an action permission, citizenship, a right,
money or collateral. Type/membership prohibitions still apply. Current recognition
is authority-wide; actor-specific constraints remain in the action rules.

Unrecognized leases are removed from opportunity discovery and rejected by the
land-acceptance resolver. Unrecognized financed purchases are hidden from credit
discovery and rejected by origination, including scripted applications. The
existing credit rejection receipt remains `Ineligible`; the legal evaluation API
provides the more specific reason. Commit-time re-evaluation rejects a purchase
prepared before recognition was withdrawn, and forged lease acceptance is atomic.

### Existing agreements

Recognition applies only to entering new agreements. Withdrawing a form leaves
accepted land-use rights, annual payments, outstanding debt, collateral and due
servicing in place. It does not independently authorize new process execution:
the existing action laws and rights still govern use. Preconfigured agreements
are treated as existing agreements, rather than as applications for new recognition.

Tests manually replace the static catalog between committed boundaries to exercise
this distinction. There is no enacted-law event, amendment schedule, historical law
archive, retroactive invalidation or compensation mechanism yet. Nor does the
catalog validate interest limits, deposits, tenure durations or founding charters.

Six additional tests compare catalog alternatives on identical openings within
the existing lease and mortgage fixtures. Both allow-list and empty-list cases run
on CPU. Existing-agreement controls accept first, withdraw recognition, then compare
continued CPU execution with unchanged reference execution: the mortgage finishes
repaying and the lease continues harvesting and paying its annual obligation.
A default control also verifies that withdrawing recognition cannot prevent
repossession of collateral on an existing loan. Denied-form controls compare
six-month reference batches with monthly CPU execution
and checkpoints. These are two separate fixtures, not a new integrated planner
choosing between leasing and buying the same plot.

## Verification

The six focused law tests check:

- Membership requirements constrain direct grants; accepted citizenship satisfies
  the requirement but cannot override a prohibition.
- Prohibited membership cannot unlock downstream discovery.
- Forged prohibited land acceptance fails atomically.
- A prohibition is rechecked against an already prepared productive batch.
- Duplicate IDs, empty rule names and unknown process references are rejected by
  validation; legacy no-policy interpretation stays explicit.
- Identical openings under permissive and prohibitive land rules produce different
  lawful behavior. Both run six months on CPU, matching reference batches and
  checkpoint continuation with reversed rule order.

The comparison uses the existing citizenship scenario. Under a citizenship
requirement, the person obtains membership, accepts land access and starts farming.
Under an additional land prohibition, it accepts neither citizenship nor land,
and records one unit of unmet food over six months (versus zero without the
prohibition); warmth remains satisfied in both. This is evidence
of a bounded legal constraint, not evidence that the prohibited economy is viable.

Run from `exp/economics`:

```sh
cargo +1.92.0 test --locked --test laws --test agreement_laws -- --nocapture
cargo +1.92.0 test --locked --test membership --test opportunities --test acquisition --test negotiation
```

## Scope and next steps

These are static, single-authority rules in world configuration. There are no
territorial jurisdictions, delegated rulemakers, autonomous legislation, amendment
dates, organizational recognition templates, charter limits, legal contract-term
bounds, courts or illegal-action enforcement. Structured decisions are callable
inspection results; existing generic denial receipts are not yet a complete legal
observer stream.

A useful next extension is legal bounds on the terms of a recognized form:
for example, a maximum monthly interest rate or a permitted lease duration,
with the same distinction between new entry and existing obligations.
Organizational founding templates and constitution/charter constraints can later
build on this separation between recognition, permission and feasibility.

Validation: all 36 focused tests passed (six laws, eight acquisition, eight
membership, seven negotiation and seven opportunity tests), along with strict
all-target Clippy and formatting. The full crate suite was not run.

Agreement-form validation: all 50 focused tests passed (six recognition controls
plus 44 acquisition, borrowing, credit, laws, membership and opportunity tests).
Strict all-target Clippy and formatting passed; the full crate suite was not run.
