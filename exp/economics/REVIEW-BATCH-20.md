# Twenty further independent review iterations

Starting revision: `78202fa`. Scope: post-v1 discovery, useful economic outcomes,
and composition of persons, households and the state. Each numbered iteration is
independently reviewed, tested and committed. This does not reopen the completed
bounded v1 release or promise completion of the broader ambitions. Individual
self-directed policy changes remain deferred. Generated runs stay under ignored
`output/`; this record retains the settings, findings and limits.

## 1. Separate forward assessment from market stocking

Added optional `FinanceRule.forward_horizon`, capped at 24 months. The assessment
of both counterparties covers the maximum of this setting, the existing discovery
horizon, and delivery duration plus the existing buffer. `None` preserves the
historical policy; a short override cannot truncate the delivery forecast. Market
holding targets continue to use the discovery horizon. Assessment diagnostics
record the inclusive interval and number of projected suppliers, including zero.

The fourteen-food worker with competing one-hour persons uses a fourteen-month
assessment while retaining a four-month market horizon. Its finite food buffer
is no longer a projected surplus to sell; no food forward is admitted and its
nutrition remains covered. CPU/reference state, ledger and audited books agree
through phase-by-phase reconstruction. An eight-month assessment still accepts
and fulfills delivery of unneeded stock from a funded bilateral surplus control; invalid horizons
are rejected. The original short-horizon late-hunger control remains supported.
This is a supplied static financial decision policy, not a self-modifying agent.

Validation: 25 discovery/supply tests passed, including the unchanged productive
baseline; strict all-target Clippy, formatting, diff and artifact checks passed.
Independent review found no blocker. Longer forecasts remain conditional and may
reject deals that a short forecast accepted; they are not a universal welfare rule.

## 2. Exact-lot forward discovery

The supplier filter now admits exactly one projected surplus unit for the existing
one-unit forward. Ordinary delivery and independent benefit checks still decide
acceptance. CPU/audited cases deliver one or two opening stock units' single sale;
zero stock, private nutrition, an existing claim and denied trade permission do
not create a new harmful worker contract. Collected goods may subsequently be
sold by their new owner; the committed-stock control tests the original debtor,
not an unjustified permanent ban on circulation of those goods.

The 25 discovery/supply controls remain unchanged and passing. Two new financial
controls cover actual delivery and exclusions. Independent review found no blocker;
strict Clippy, formatting and artifact checks passed.

## 3. Finance whole procurement lots

Loan sizing now uses the same ceiling-to-lots helper as actual mint procurement
and need-generated purchases. Holding one of the two required metal units still
requires buying a whole two-unit lot; the labor purchase is unchanged. Audited
CPU/reference ten-month controls borrow 6, 6 and 4 coins for opening metal 0, 1
and 2, complete minting and repay, retaining the expected spare metal.

The control uses zero interest and an eight-month term. With a four-month term,
the first principal installment is collected before next-month procurement and
leaves too little working cash; that proposal is rejected. This is a documented
liquidity constraint, not an interest-only effect or a change to repayment rules.
Missing lender funds, metal and storage likewise prevent an unsupported loan.
Four finance and 25 procurement/public-sales controls pass, plus strict Clippy,
formatting and repository artifact checks.

## 4. Reject mixed-currency mint underwriting

Mint procurement prices and loan principal must share a denomination. Discovery
now rejects a differing financial denomination when mint procurement is enabled;
it does not invent a conversion rate. Forward-only configurations retain their
stock denomination support. The grain-as-coins negative control and all five
financial discovery tests pass. Independent review found no blocker.

## 5. Protect active process inputs during land admission

Land admission now shares the unpaid-process-input claim reader with need orders.
Seed committed to a current or future unpaid stage cannot also justify an optional
new lease. Consumed entry inputs are not reserved twice. Existing monthly service
reservation and conservative handling of same-boundary incoming purchases remain.
A two-case integration control compares CPU/reference admission at Acquire, then
checks existing work completes on Reference. Independent review found no blocker.

## 6. One published forward per buyer and boundary

Reserve objectives are collapsed by buyer/resource using the maximum target.
Current or future published forward terms now block another proposal for that
buyer, just as an outstanding accepted delivery already did. Different resources
use stable resource-ID order; this is an explicit bounded search order, not a
claim of optimal procurement. Duplicate and distinct-resource controls select
one offer even when objective order is reversed. All six finance tests pass.

## 7. Widen demand rounding before arithmetic

Stocking demand now rounds consumption batches in widened units before checking
that the final stock target fits. Equal near-maximum need and recipe output still
require one batch; a remainder requires a second. An unrepresentable target fails
Open with a specific error and leaves world, state and ledger unchanged. The
boundary arithmetic and public rollback controls pass. Input sums already bounded
by catalog validation were left unchanged. Independent review corrected an empty
worker-needs fixture before the final passing run.

## 8. Structured forward assessments

Financial discovery now retains typed assessments with candidate counts, bounded
horizons, parties and units. Attempts distinguish projection failure, performance
shortfall, no mutual gain and publication. Candidate IDs may repeat after rejection;
only published terms can be correlated with a later contract. These records are
diagnostics, not another execution model. Existing text receipts remain available.

No-supply, successful-delivery and legally forbidden-trade controls distinguish
empty assessments, published proposals and performance shortfalls. Seven finance
tests and strict all-target Clippy pass. Actual admission and delivery remain in
the ordinary ledger and forward book.

## 9. Loan assessment and publication boundaries

Loans use the same diagnostic assessment model. Candidate counts cover funded,
Lend-permitted people, while actual admission still checks the full agreement.
Controls distinguish no funded lenders, repayment without productive benefit,
projection failure, and publication. At the publishing Open, the loan terms exist
but the accepted loan book is still empty; Acquire remains the admission boundary.
Eight finance tests pass and independent review found no blocker. An invalid-rate
control exposed a missing discovery configuration bound, queued for correction.

## 10. Observe financial discovery without decision hooks

The external planning observer exports new financial assessments at committed
Open. Requester/counterparty filtering, selected outcome detail and optional
comparison detail use the existing observer configuration. A continuation emits
only new assessments. Read-only controls compare complete world, state and ledger
with an unobserved run; no forecast branch produces external logs. See TELEMETRY.md
for the distinction between proposal publication and actual financial settlement.

## 11. Validate discovery interest terms up front

Discovery now shares the executed credit model's rate bound. Zero and 10,000 basis
points are valid configurations; higher rates fail construction instead of being
repeatedly proposed and rejected in private rollouts. The temporary invalid-rate
projection fixture was replaced by endpoint/extreme configuration controls.
Candidate-specific projection failures remain observable. Nine financial controls
pass; independent review found no blocker.

## 12. Compose discovered loans with configured purchase offers

Loan identity allocation now includes unused financed-purchase offer IDs, matching
the credit adapter's shared identity rules. Previously offer ID 1 blocked a useful
mint loan with a duplicate-ID projection error; changing only the offer ID to 99
made it succeed. Controls now discover IDs 2 and 100 respectively, complete minting
and repay while the declined property purchase leaves ownership unchanged.
CPU/reference state and ledger agree under financial audit. Ten finance tests and
strict all-target Clippy pass; the latter also corrected the preceding test's
`err().expect()` style. Independent review found no blocker.

## 13. Configurable land allocation, with completion checks

Discovered land admission now uses an explicit policy and seed. Stable priority
remains the default. Identical opening requests under stable and seeded lottery
policies compete for the same plots and shared household labor; only one joint
reservation is feasible. Different seeds select both applicants, and the selected
applicant actually completes farming. Seedless applicants cannot win. All current
claim priorities are equal, so priority lottery has no extra distinction here.
Fifteen discovery tests pass, including CPU/reference and reordered-participant
state/ledger equality. This demonstrates bounded execution, not improved welfare.

## 14. Protect financial claims during land admission

Land admission now protects accepted claims within its configured horizon as well
as unpaid process inputs. Its preview stages current credit and forward changes
before subtracting claims, so a delivery is not reserved twice and a newly signed
forward is protected immediately. Incoming purchases remain unavailable to this
conservative opening-stock admission check. This covers bounded deliveries/dues
and currently collectible loan/wage amounts, not every future loan installment.

Sixteen discovery tests pass. The new CPU/reference controls cover future seed
delivery, delivery at this boundary, a new forward, and the exact horizon cutoff.
Granted farming completes and protected deliveries settle. Historical contracts
in these continuation fixtures are opening claims, not observed prepayments.
Independent review found no production blocker.

## 15. Bounded search over loan durations

Financial discovery can now try explicit alternative loan durations. It tries the
primary term first, then unique alternatives in ascending order, and stops at the
first published loan. Empty alternatives preserve the original fixed-term policy;
valid durations cap search at 22 terms and assessment at 24 months. Each candidate
uses the existing repayment, legal admission and bilateral comparison machinery.
This is first-acceptable search, not negotiated or globally optimal credit terms.

The zero-treasury control rejects a four-month loan: its first installment leaves
too little money for whole-lot mint procurement. An optional eight-month term
finances actual inputs, completes minting and repays. Primary-eight preference is
preserved even when four is offered. Audited CPU/reference runs agree. Diagnostics
and observer records distinguish contract duration from forecast horizon and pin
safe ID reuse for rejected proposals. Independent review found no blocker.

## 16. Preserve reserved guarantee recourse identities

The reviewer reproduced a valid guarantee reserving loan ID 100 for potential
recourse. With a declined purchase offer 99, discovery chose 100 and its forecast
failed validation. Loan discovery now includes configured recourse reservations
when allocating IDs. The combined control instead issues 101, completes minting
and repayment, and leaves both the purchase and guarantee unexercised. Audited
CPU/reference state and ledger agree; all eleven financial discovery tests pass.
This does not change guarantee admission, priority or payment semantics.

## 17. Explain projected contract performance

Financial diagnostics now retain terminal forecast evidence as well as a decision:
nonadmission, loan status/outstanding debt, or delivered/outstanding goods. Failed
projections have no performance witness. The external observer exposes these in
both selected and alternative detail without logging inside decision kernels.

Fourteen finance/observer tests pass. Controls distinguish a legally inadmissible
forward, a fulfilled delivery, an admitted high-interest loan with unpaid debt,
and a fully repaid short loan that still produces no mutual gain. Forecast records
remain distinct from actual admission and settlement; independent review found
no blocker. No pricing, repayment or selection behavior changes in this iteration.

## 18. Resume duration search under real scarcity

The optional duration search now has a joint scarcity/continuation campaign.
Identical primary-four/alternative-eight rules run with adequate funding, one tick
less than required lender cash, absent ore, insufficient issuer storage, and law
denying lending. Only the funded control publishes a loan and completes minting.
Longer terms cannot manufacture input stocks, storage, money or permission.

Every CPU phase is reconstructed from its retained world/state, ledger and reports;
an uninterrupted reference run agrees on all of those and independently maintained
financial books through month ten. All thirteen financial discovery tests pass.
This is additional integration evidence, with no new execution or policy default.

## 19. Finance, household farming and personal food in one control

Added the reusable [financed circulation scenario](FINANCED-CIRCULATION.md) and a
CPU example flag. Fourteen-month controls retain endogenous household/land/state
formation while comparing whole labor lots, missing ore and missing lender money.
The calibrated worker earns eight coins and purchases six wheat after actual wages;
its food deficit falls from eleven to five units, while both household members
retain their one-unit startup deficits. Ten crops and paid annual dues coexist
with the repaid eight-month loan. No ore still permits repayment but no minting;
no lender money prevents finance and minting without preventing household farming.

All three circulation tests pass. Added controls compare uninterrupted reference
execution against CPU reconstruction at every phase with independent accounting,
actual traded quantities, cash/food conservation and paid obligations. These are
finite calibration results, not proof of sustainable income or an allocation-policy
improvement. Five-hour technology, worker capacity and market lots change together.
