# Creditor allocation at the monthly Due boundary

Implemented as a policy of the existing contract execution window. This does not
introduce an insolvency proceeding or another debt ledger.

## Configuration and boundary

`World.collection_policy` defaults to `finance::CollectionPolicy::Stable`, retaining
rank, contract type and ID order. `Proportional` is opt-in. Lower collection ranks
still precede higher ranks. Collateral lien priority remains a separate concept;
this policy does not define a liquidation waterfall.

At Due, the proportional adapter inventories outstanding loan installments after
this month's accrual and native annual land dues. Accrual is previewed on copies;
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

## Limits

- This adapter currently requires credit servicing and native-denomination land
  dues. Opting in without credit configuration or with alternative coin payment
  terms fails validation. Default stable-policy scenarios retain their support.
- Forward collection remains at Acquire; land arrears retries remain after
  production. Neither participates retroactively in the Due pool.
- Shared receiving storage across different debtor/resource pools resolves in
  stable pool order. This is not a simultaneous multicommodity clearing solver.
  Actual contract execution retains stable contract order and can pay less than
  a planned ceiling if intervening storage changes constrain receipt. Inspect
  paid amounts, not only grants. There is no cross-resource netting or implied
  exchange rate.
- No new minimum-useful threshold, indivisible allocation, automatic debt
  discharge, guarantee, or general insolvency/liquidation lifecycle is introduced.
  Existing arrears and agreed collateral consequences still apply.

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
Unsupported alternative-denomination configuration is explicitly rejected.

Validation: nine focused suites passed (66 tests), followed by the final lending
suite with 18 passing tests, including two added controls. These counts overlap.
`cargo +1.92.0 fmt --check`, all-target Clippy with warnings denied, and the
repository artifact check passed. Results and limitations here are source
documentation; raw local test output remains under ignored `output/economics/`.

Next: finish the remaining acceptance/execution adapters, then use the shared
claims and explicit allocation boundary for authorized insolvency, capped
contingent guarantees with recourse, and funded liquidation sales. Those require
separate legal triggers and lifecycle states; a collection shortage alone does
not imply any of them.
