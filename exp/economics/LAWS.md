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
catalog itself validate deposits or founding charters. Term ceilings are a separate
layer, described below.

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

## Legal ceilings on agreement terms

`Policy.agreement_limits` now supplies two optional, authority-wide ceilings:

```rust
laws::AgreementLimits {
    max_lease_months: Some(120),
    max_monthly_interest_bps: Some(100),
}
```

Interest uses the loan's existing **monthly basis points**, so 100 means 1% per
month, not an annual rate. Both ceilings are inclusive. `None` imposes no ceiling;
zero is a real limit (only zero-interest loans pass a zero interest ceiling).

Lease duration is the remaining tenure at new acceptance, including the expiry
month: `right.through - acceptance_month + 1`. Discovery before an offer activates
uses its activation month as the earliest possible start. Date arithmetic is
checked. Existing acceptance still checks availability and expiry independently.
Waiting can make a previously excessive remaining duration fit the cap; neither
the offer's expiration nor its annual payment is changed.

`laws::evaluate_terms` combines form recognition, action permission and the supplied
terms. `TermLimit` reports the term, offered value and maximum. The narrower
`evaluate_agreement` API checks only form recognition and permission, so domain
acceptance uses the term-aware API. The terms are derived from the actual catalog
offer at discovery and acceptance, not supplied as a claim by the applicant.

Lease discovery checks the terms separately from prospective citizenship permission,
so a lawful prerequisite remains visible but citizenship cannot make an excessive
lease legal. Financed-purchase discovery and origination check the actual monthly
loan rate. Forbidden terms are rejected; there is no automatic repricing, truncation
or counteroffer. The existing credit receipt classifies such refusal as
`Ineligible`; the legal evaluator supplies the detailed reason.

Ceilings apply to **new entry only**. Tightening a cap does not revise an accepted
loan rate, shorten an existing right or change rent. Controlled continuation tests
tighten both ceilings to zero after acceptance, resume on CPU at checkpoints, and
match unchanged reference state and ledger through repayment or annual rent
settlement. As with recognition withdrawal, this is a test of static policy
replacement, not an implemented legislation or amendment system.

Five term-specific tests cover values below/at/above ceilings, zero-interest
acceptance, remaining lease duration, citizenship discovery, atomic rejection of
prepared/forged batches, and continuation of previously accepted terms. These
extend the six agreement-recognition controls. The caps do not yet cover total
borrowing cost, fees, rate changes, deposits, rents, loan maturity, organizational
charters or different rules by actor type.

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
dates, organizational recognition templates, charter limits, courts or
illegal-action enforcement. Term bounds currently cover only lease duration
and monthly loan interest. Structured decisions are callable
inspection results; existing generic denial receipts are not yet a complete legal
observer stream.

A useful next extension is to expose legal denial reasons through the existing
planning/settlement observers, so a scenario can distinguish a missing permission,
unrecognized form and excessive term without separate inspection. Negotiating
legal alternatives and organizational founding templates can follow, keeping
recognition, permission, terms and economic feasibility distinct.

Validation after term ceilings: all 62 focused tests passed (11 agreement-form
and term controls, six laws, eight acquisition, six borrowing, nine credit, eight
membership, seven negotiation and seven opportunity tests). Strict all-target
Clippy and formatting passed. The full crate suite was not run.
