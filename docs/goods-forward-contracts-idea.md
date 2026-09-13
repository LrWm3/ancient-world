# Goods forward contracts: experiment idea

Status: idea to try; unimplemented and opt-in if developed. This is an ordinary
CPU simulation extension using existing production kernels. It does not depend
on the emergency GPU architecture proposal.

## Question

Can agreeing on future deliveries at a fixed price improve production planning
and supply reliability when output and demand occur in different months?

The motivating example is a crop purchase agreed before harvest. The mechanism
should work for **any tradable catalog good**, including food, individual crop
goods, wood, ore, tools, pottery and other manufactured output. Seasonality belongs
in the supply forecast, not in a crop-specific contract type. Eligibility still
requires a meaningful unit, supported inventory/transport accounting and evidence
that the seller can supply the good.

For example, a town agrees now to receive a specified quantity of grain after the
next harvest. The same terms can describe tools expected from several months of
workshop production or ore expected from a finite source. These are promises of
future delivery, not additions to current inventory.

## Existing foundation and proposed difference

Sources inspected: [export contracts](../src/export_contracts.rs),
[persistent contract identities](../src/export_contracts/identities.rs),
[delivery payments](../src/export_contracts/payments.rs),
[markets](../src/economy.rs) and [production](../src/production.rs).

The existing [repeat export contracts](export-contracts.md) use observed deliveries,
buyer escrow and supplier production orders. Their current admission excludes food.
Procurement is tied to unmet buyer targets and eligible available stock; it does
not express a separate future arrival window. The current implementation also
supports explicit dispatch/delivery payment timing, described in the later
[credit implementation record](credit-implementation.md).

The forward-contract experiment adds a dated obligation: specified counterparties
agree on an exact good, quantity, price and arrival window. Once accepted, the
buyer cannot cancel simply because its target or the spot price changes. A seller
cannot reprice the remaining quantity after a price increase. Explicit cancellation,
shortfall and closure rules handle failures.

Reuse identity, escrow, cash-transfer and cargo primitives where their semantics
fit. Keep a distinct forward agreement record and feature flag initially. Existing
repeat procurement and old archives retain their behavior. A delivery-paid export
by itself is not this new future-delivery mechanism.

## First experiment

- Town trading accounts are buyer and seller. Existing workshop operators sell
  services to towns and must not be treated as owners of town export goods.
- A contract names one exact catalog good and a quantity in its catalog unit
  (currently market quantities are kilograms). No automatic substitute goods.
- Price and a future arrival window are fixed at acceptance. Allow partial
  deliveries within that window and record the unfulfilled remainder.
- The buyer fully funds the accepted amount into escrow. Pay the seller only for
  accepted delivered quantity. There is no producer advance in the first arm.
- Formation uses bounded buyer requests and independently acceptable seller offers.
  No contract is required to form when terms or capacity do not match.
- Physical production, source stocks, local food protections, storage, routes and
  freight remain binding. A promised harvest does not guarantee a harvest.
- Start with direct bilateral commitments. Resale, netting, speculative positions,
  financial settlement of price differences and a general exchange are later ideas.

This first arm tests planning and price commitment. It does not test whether
prepayment finances a cash-poor producer. A later producer-advance or credit arm
must make that additional transfer and default exposure explicit.

## Proposed agreement and receipts

| Field group | Required information |
| --- | --- |
| Identity | Stable agreement ID; buyer/seller town and account IDs; exact good ID; currency |
| Terms | Accepted month; quantity; fixed unit price; earliest/latest arrival; dispatch cutoff; explicit delivery/grace and cancellation rules |
| Evidence | Forecast month; output/storage/route assumptions; observed fulfillment; policy settings used for acceptance |
| Funding | Requested and actually funded amount; unassigned escrow; amounts assigned to dispatched cargo; seller payments; buyer refunds; representational residual |
| Quantity state | Accepted quantity; unshipped outstanding; in transit; accepted delivered; failed/cancelled quantity |
| Scheduling | Monthly production-demand additions; dated physical stock/freight grants; linked cargo IDs |
| Outcome | Active, settling or closed; shortfall/cancellation reasons; late-delivery and loss observations; causal event IDs |

Keep proposal, accepted agreement, monthly forecast, reservation and settlement
receipts distinct. A forecast revision does not revise accepted terms. Changes to
terms require an explicit bilateral amendment; omit amendments from the first
implementation if they are unnecessary for the controlled experiment.

## Supply, demand and admission

At formation, the buyer forecasts demand at the delivery window using current
stocks, expected consumption, already-dispatched cargo and existing commitments.
Expected receipts are not current food access. Maintain separate in-transit and
unshipped supply projections so dispatch does not count the same agreement twice.
A known shortfall restores forecast unmet demand and can trigger later spot buying.

The seller offers conservative deliverable surplus over the same horizon:

```text
opening usable stock
+ forecast feasible output before the dispatch cutoff
- expected local use and protected reserves
- already accepted delivery obligations
- expected spoilage / other modeled losses
```

Also bound the offer by storage and plausible transport before the arrival
deadline. This is evidence for acceptance, not a reservation of future physical
goods, workers or freight. Actual monthly reservations remain bounded by live
availability. Store forecasts so later failure can be explained against the
opening expectation.

| Supply kind | Forecast evidence |
| --- | --- |
| Crops and food | Actual crop calendar, current seasonal state, land, expected inputs, recent yields, household/town consumption and storage losses |
| Manufactured goods | Recipe throughput, equipment, labor, affordable inputs, competing output claims and observed completed work |
| Wood, clay and ore | Accessible remaining sources, extraction allowances, tools, labor and competing claims |
| Other tradable catalog goods | Supported source/recipe or existing stock with an explicit forecast adapter |

Shared recipe inputs, workers and source stocks must bound the seller's combined
offers across goods. Co-products can be forecast together, but their shared inputs
are charged once. Missing forecast support means no future-production offer for
that good yet; it does not mean unlimited supply. A stock-backed offer can still
be possible. Food and crop goods must use their actual inventory and nutrition
adapters, preserving the existing distinction between aggregate food stocks and
individual goods.

Use named experiment settings for forecast conservatism, term limits, cash-at-risk
ceilings, minimum useful shipments and observed reliability. Start with bounded
terms and requests. Calibrate values rather than treating arbitrary percentages
as historical facts.

## Prices and competing claims

Calculate seller asks from supported cost/output forecasts, storage, expected
delivery losses and the selected risk allowance. Buyers have independent price
ceilings. Unsupported quotes are declined. The first pilot can accept the seller's
fixed ask when it fits the buyer's ceiling; negotiation is not required.

Define the quoted unit price as the seller payment per accepted delivered unit.
For the first implementation, retain the existing freight-cost mechanism and
record its payer separately; do not create an unmodeled carrier account or silently
include the same cost twice. Subsequent spot prices do not reset an agreement.

Collect feasible offers and requests before allocating the buyer's formation cash
or the seller's forecast capacity. Proposed initial priority is earliest delivery
deadline, followed by acceptance order for existing obligations; new equal-ranked
requests share divisible capacity proportionally. Apply minimum shipment thresholds
before commitment and explicitly release/refill unusable allocations. Record the
policy rather than making vector iteration order the priority.

Within monthly dispatch, existing protected local use comes first. The proposed
experiment then serves due forward obligations before discretionary spot exports
from the same eligible surplus. This is an explicit allocation-policy change,
not a reason to reorder monthly phases. Existing funded repeat procurement must
have a stated place in the combined window. For the first comparative fixture,
disable new repeat-procurement formation equally in all arms and resolve existing
commitments under their original rules before accepting overlapping forwards.

## Monthly integration

Follow the [monthly schedule](monthly-schedule.md), the coordinator in
[civilization.rs](../src/civilization.rs), and the
[explicit allocation pattern](service-allocation.md).

| Phase | Proposed work and visibility |
| --- | --- |
| Open | Process due cargo under existing delay/loss rules; accept eligible deliveries; resolve cargo-linked payment/refund amounts once. Current arrivals can support current consumption. Record opening delivery evidence. |
| Reserve | Turn previously accepted agreements into dated production-demand additions; combine overlapping demands once; reserve this month's actual labor/material/source allowances through existing systems. Do not reserve future labor by spending this month's time. |
| Execute/settle | Existing kernels produce actual output. Settle actual inputs/output and shortfalls. Contract demand gives no yield, staffing or source-stock bonus. |
| Respond | After this month's production, allocate due dispatches against actual surplus and freight. In the existing market decision window, periodically collect and accept new forward agreements, with effects on production starting next month. Record shipment and acceptance events immediately. |
| Close | Reconcile escrow, cargo, obligations and completed receipts; record outcomes and validation. No second payment or delivery pass. |

Choose quarterly new-agreement formation for the first experiment to match the
existing procurement opportunity frequency; check and dispatch outstanding
agreements monthly. The earliest new arrival must leave time for at least a later
monthly arrival pass and the feasible route journey. A seasonal contract's cutoff
must allow harvest, dispatch and travel; arrival month is not harvest month.

Demand additions target the appropriate future production interval. The initial
crop adapter must honestly describe how existing production responds: contractual
demand does not automatically add planting, acreage switching or anticipatory
investment to crop equations that do not already support those decisions. Measure
the resulting production response before extending those mechanisms.

## Delivery, losses and cancellation

The contract quantity is accepted arrival quantity. The initial dispatch policy
sends at most the unshipped outstanding amount, avoids intentional early arrival
and stops new dispatches after its recorded cutoff. In-transit delay follows a
fixed, bounded grace rule recorded in the terms. Report late delivery separately
even when accepted during grace. During grace only already-dispatched cargo is
eligible; new shipments cannot perpetually extend maturity.

At final cargo resolution, pay the agreed unit price for the eligible amount that
arrived and refund the funded share corresponding to failed quantity. In the
first arm there is no automatic replacement of spoiled or lost cargo; the amount
is recorded as unfulfilled. A future replacement policy would need its own
deadline and obligation accounting.

At the dispatch cutoff, refund unassigned escrow for the quantity that will not
be shipped. Do not refund escrow assigned to unresolved cargo at the same time.
An agreement closes only after every linked cargo and payment residual is settled.
Remaining late cargo cannot be credited to a closed obligation and also sold as
new stock. Define return or ordinary late-sale disposition before implementation;
it must conserve goods and have a real payer if sold.

Food emergency, drought, source depletion, unavailable inputs, blocked routes and
lost cargo can cause shortfall. Keep food protections explicit and record why a
promise failed. No automatic money creation, forced local starvation, indemnity
or new unsecured debt fills the gap. Observed fulfillment/loss can affect future
offers under a named reputation policy, with disaster and voluntary cancellation
recorded separately.

Changed demand or adverse prices alone do not cancel a funded agreement. Abandoned
counterparties, permanent closure or disabling new formation need explicit runoff
rules: stop forming new agreements, settle existing cargo, and return only the
escrow still owned by the buyer. Resolve inactive-account claims through an
explicit successor/unclaimed-balance path rather than deleting funds.

## Conservation and interaction with credit

At every boundary, cumulative funding equals unassigned agreement escrow plus
unresolved cargo escrow plus seller payments plus buyer refunds, including any
owned precision residual. Moving funds from an agreement to a cargo payment is
a transfer between escrow buckets, not a second buyer debit. Payment is recorded
once and uses the actual representable amount at both endpoints.

Accepted quantity equals unshipped outstanding plus in-transit quantity plus
accepted delivered plus finalized failed/cancelled quantity. Loss moves between
these categories; it does not decrement two of them for one failure. Whole-world
goods accounting additionally includes production, consumption, storage loss,
transit and returned goods under existing units and ledgers.

Buyer escrow is not spendable seller cash. If a later credit arm lends against
expected proceeds, use the stable agreement/payment identity and actual seller
beneficiary. Underwriting must subtract other pledged proceeds and assess net
receipts after costs and delivery risk. Credit default does not make the buyer pay
twice or release escrow before the delivery condition. Producer advances need a
separate unearned-advance/refund ledger; they must not be labeled earned sales at
acceptance.

## Evaluation plan

First build controlled fixtures with the same opening requests, stocks, money,
production capacity and route conditions:

- A seasonal crop delivery and a non-crop manufactured delivery use the same
  contract state machine. Include a finite extracted good and aggregate food.
- Fulfillment, partial production, zero output, missing inputs and an exhausted
  source; compare grants, actual output, dispatched and accepted quantities.
- Multiple buyers share one seller; multiple goods share recipe inputs or freight;
  scarce buyer cash and a minimum shipment that cannot be usefully filled.
- Prices rise/fall after acceptance and buyer targets change. Accepted terms stay
  fixed, while spot transactions retain their normal prices.
- Spoilage, blocked routes, grace expiry, inactive accounts and feature-disable
  runoff. Unshipped refunds and cargo refunds cannot overlap.
- Arrival nutrition timing, annual boundaries, monthly/batched advancement and
  checkpoint continuation. Pending agreements and escrow survive reload.

Then compare three arms across matched seeds and horizons:

| Arm | Purpose |
| --- | --- |
| Existing market control | Current mechanism with the chosen common procurement/credit settings |
| Generic forwards with fully funded buyer escrow | Isolate future quantity/price commitment and dated planning |
| Later: same forwards plus a bounded producer-finance mechanism | Isolate financing from planning; only after escrow-only results are understood |

Report food access and hunger, production/stockouts, delivered contract share,
late/failed quantity, spot availability, storage losses, working cash tied up,
price dispersion, actual seller receipts, local reserve violations and runtime.
Break results down by good, household/town exposure and scarcity, not only total
population or number of contracts. Fewer spot trades are not proof of better access.

The idea is worth retaining if measured reliability/planning improves without
unacceptable food access, liquidity or failure effects in the tested settings.
Record mixed or negative results; promises and conserved ledgers alone do not
establish a useful market mechanism. Keep generated outputs under ignored
`output/` and summarize settings, results and limitations in Markdown.

## Implementation worklist, if the experiment is selected

1. Specify price/cost treatment, units, deadlines, late-cargo disposition and
   inactive-account runoff; add an opt-in configuration and archive defaults.
2. Implement the generic agreement ledger and reuse cash/cargo settlement with
   explicit links, conservation checks and runoff behavior.
3. Add good-specific forecast adapters and joint admission for buyer cash and
   seller capacity. Food eligibility must not inherit the industrial-only filter.
4. Integrate dated demand and due dispatch under the existing monthly phases;
   implement the fixed-priority/shortfall receipts and controlled fixtures.
5. Run the escrow-only comparisons before adding producer advances, credit,
   contract resale or more elaborate enforcement.
