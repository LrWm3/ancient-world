# Collection across standalone land and prepaid deliveries

Fibonacci batch 21 extends the existing allocation window; monthly scheduling is
unchanged. `World.collection_policy` still defaults to `Stable`. `Proportional`
now also works without a loan-servicing driver.

## Land

Standalone annual dues and the later arrears retry inventory their own opening
requests. They reuse the same ranked native/accepted-tender allocator as loan/land
Due collection. Each agreement's grant settles its oldest dated bills first.
A retry sees its actual later resources; it cannot reuse a prior grant. Native
receipts, alternative currency transfers and issuance retain their existing units.

Collection receipts expose requested, allocated and paid quantities even when
payment is zero. The settlement observer reads these receipts from both standalone
and credit-driven batches. Modified grants fail exact settlement replay.

## Prepaid deliveries

`World.prepaid_admission` defaults to `SingleOutstanding`, retaining the previous
one-outstanding-delivery-per-seller rule. Opt-in `Concurrent` permits explicitly
consented contracts while reserving the buyers' actual opening prepayments and
prospective receiving storage. This does not underwrite future production or make
concurrent obligations safe to accept autonomously.

Mature deliveries share opening seller stock at Acquire, after credit transactions
and before new prepayments and spot trades. Stable order uses collection rank,
effective due date and contract identity. Proportional collection uses the common
ranked allocator. Essential protection and live receiving space bound execution.
The direct adapter stores collection attempts in `Batch.forward_collections`,
including unpaid attempts, and validates them with the acquisition batch. When composed with direct prepayments, tool-backed claims join that same pass and
attempt receipts. The standalone legacy tool-only driver still returns delivery
transactions without the attempt-receipt sidecar.

## Indivisible collection

A shared `RejectExchange` claim is one whole payment lot. A claim that cannot fit
its available budget or receiving storage is excluded from grants, allowing
other claims to use those resources. Divisible claims and accepted integer tender
lots retain proportional shares. This is a deterministic allocation policy, not
an optimizer for maximum completed packages. Multi-resource exchange packages
still require joint atomic acceptance through `Execution::exchange`.

## Controlled outcomes

`tests/standalone_collection.rs` compares identical opening conditions:

- Two four-unit land bills and five grain: Stable pays 4/1; Proportional pays 3/2.
- A higher-rank claim precedes sharing; a full creditor store releases capacity
  for the other feasible claim.
- Accepted two-coin conversion lots never extinguish fractional grain claims.
- Concurrent four-unit forwards produce the same 4/1 versus 3/2 distinction;
  insufficient prepayment or prospective storage rejects admission.
- Collection receipts retain unmet demand next month; forged grants cannot commit.

The tests compare CubeCL CPU and reference execution, reordered terms, checkpoints
and observers. `tests/finance.rs` also checks whole-claim feasibility and protected
stock. These tests establish allocation and conservation, not economic viability.
