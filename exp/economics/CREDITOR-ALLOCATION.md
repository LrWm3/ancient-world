# Creditor allocation at the monthly Due boundary

Implemented as a policy of the existing contract execution window. This does not
introduce an insolvency proceeding or another debt ledger.

## Configuration and boundary

`World.collection_policy` defaults to `finance::CollectionPolicy::Stable`, retaining
rank, contract type and ID order. `Proportional` is opt-in. Lower collection ranks
still precede higher ranks. Collateral lien priority remains a separate concept;
this policy does not define a liquidation waterfall.

At Due, the proportional adapter inventories outstanding loan installments after
this month's accrual and annual land dues. Accrual is previewed on copies;
only the committed loan book advances its accrual date. Land dues aggregate by
agreement for allocation and settle oldest first within that agreement.

Each debtor/resource account is a distinct pool. Essential-stock protection is
subtracted once from its opening budget. Incoming receipts do not fund outgoing
payments in the same boundary. The shared allocator weights equal-rank claims by
their outstanding quantities, distributes integer remainder units by largest
fractional remainder then stable contract identity, and reassigns storage-blocked
shares while another claim can receive them. Zero demand and future claims receive
nothing. Atomic exchange claims are rejected rather than partially funded.

Grants are ceilings passed to the existing claim adapters. Actual execution still
checks live storage and resources. Fixed-value repossession cannot spend funds
reserved for later collections to pay its surplus; it defers when those funds are
needed. Collateral title and attached crop obligations keep their existing rules.
Settlement observers expose `requested`, `allocated`, and `paid`; stable-policy
receipts have a null allocation because they use sequential reservation.

## Alternative tender

Within each rank, collection plans native non-currency obligations first. It then
allocates the currency pool jointly between native coin loans and the remaining
land dues that permit that tender. Higher ranks finish both passes before lower
ranks. This is an explicit native-first rule, not a best-exchange-rate search.

`finance::Execution::pay_tender` is the shared conversion adapter. The accepted
`coins_per_unit` rate determines payment lots: a two-coin tender can extinguish
one claim unit, never half of one. Unusable fractional allocations are released.
Actual transfers contain coin quantities; obligations and collection receipts
remain in original claim units. Alternative payments do not increment native
collection totals or trigger grain-linked issuance. Protected coin balances are
excluded from planning and execution. Currency chains are rejected; the adapter
does not solve cyclic conversions.

## Limits

- The proportional Due adapter currently requires credit servicing. Standalone
  land collection retains its existing sequential policy. Alternative tender
  must be an explicitly accepted storage-free stock resource.
- Forward collection remains at Acquire; land arrears retries remain after
  production. Neither participates retroactively in the Due pool.
- Shared receiving storage across different debtor/resource pools resolves in
  stable pool order. This is not a simultaneous multicommodity clearing solver.
  Actual contract execution retains stable contract order and can pay less than
  a planned ceiling if intervening storage changes constrain receipt. Inspect
  paid amounts, not only grants. There is no cross-resource netting or implied
  exchange rate.
- Payment lots support integer conversion. General indivisible multi-resource
  exchanges still require atomic acceptance, not partial collection.
- Collection alone never starts insolvency or discharges debt. The separately
  authorized [recovery lifecycle](CONTRACT-RECOVERY.md) reuses these primitives.

## Verification

The lending tests compare the same opening shortages under stable/ranked and
proportional collection. Controlled outcomes include:

| Opening situation | Proportional result |
| --- | --- |
| Two equal coin installments, five coins available | Three and two coins, with deterministic remainder handling |
| Ten grain loan due and ten grain land due, six grain available | Three grain paid to each claim |
| Twelve grain, three protected, two creditors sharing storage for only two grain | Two grain total to constrained creditors; seven to the other eligible same-rank creditor; zero to the lower rank |
| Future installment | Zero allocation |
| Indivisible exchange submitted for proportional collection | Explicit rejection |

The CPU loan case compares batched and resumed execution and reverses configured
advance order. Both final state and committed ledger agree. Regression checks
cover existing loan servicing, annual agreements, collateral/crop handling,
acquisition composition, payment protection and observer output. A separate
regression verifies that a five-coin surplus cannot consume five coins reserved
for another loan: repossession defers and the reserved installment is paid.
The mixed CPU case owes four grain plus four coins, has two grain and five coins,
and accepts two coins per grain unit. It pays two grain in kind, two coins toward
one remaining land unit, and three coins toward the loan. One grain and one coin
remain owed; only the two actual grain units count as native collection.

Earlier native-allocation validation: nine focused suites passed (66 tests), followed by the final lending
suite with 18 passing tests, including two added controls. These counts overlap.
`cargo +1.92.0 fmt --check`, all-target Clippy with warnings denied, and the
repository artifact check passed. Results and limitations here are source
documentation; raw local test output remains under ignored `output/economics/`.

Current alternative-tender and recovery validation is recorded in
[Contract recovery](CONTRACT-RECOVERY.md).

Further coverage: standalone and forward allocation adapters, additional tender
routes, and broader claim admission to recovery. A collection shortage alone
does not imply a legal proceeding.
