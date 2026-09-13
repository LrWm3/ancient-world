# Council credit and administrative payment

The institutional reserve experiment exposed a second constraint: even when a
lender has surplus cash, the borrower needs net future receipts. The council
forecast currently subtracts three groups from its conservative tax estimate:

- Requested annual town support from the last tax boundary.
- Requested annual road upkeep from that boundary.
- Twelve times current administration plus recently requested household relief.

`src/credit/taxes.rs` caps receipts by both the last actual collection and current
collectible town cash, using current ownership, tax policy and administrative
capacity. `src/credit/councils.rs` adds annualized monthly needs. Support and road
requests remain costs even when lack of money prevented their payment. This
prevents an insolvent council appearing solvent simply because it stopped paying.

There are potential forecast errors in both directions. A temporary relief spike
is treated as persistent for a year; lagged support and road requests can miss new
needs or retain old ones. Current cash can understate tax receipts after future
sales, while prior collection cannot guarantee those sales. These are reasons to
measure forecast components and realized collection, not to drop operating costs
until borrowing succeeds. No forecast rule is changed by this verification work.

The controlled bridge fixture tests the narrower causal path from automatic
underwriting through existing administrative payroll to unpaid-month and loyalty
responses. It transfers existing cash to make a lender liquid, supplies explicit
prior-tax evidence and a schematic open contact route, and compares disabled credit with sufficient and insufficient
net receipts. The same test retains live current-tax limits, accounting checks and
serialized continuation at the disbursement boundary.

Administrative payroll is a council-to-town transfer. It does not by itself prove
that a named administrator completed more work, that future taxes will materialize,
or that a full monthly history benefits. Those remain separate integration and
balance requirements. This test should not be reported as completed labor evidence.

## Reproduction and scope

```sh
cargo test --lib automatic_bridge_funds_administration_without_forgiving_operating_costs -- --ignored
```

The fixture supplies a funded lender, a schematic contact, and declared prior-tax
receipts. It verifies automatic source-limited borrowing, a disabled-credit
control, an insolvent operating-budget control, and a closed-contact control.
Governance then makes its normal payment. The funded branch must pay more, retain
fewer unpaid months and show a stronger loyalty response than the disabled branch.
The insolvent branch must match disabled administrative payments. Loans remain
outstanding after payroll; paying wages does not settle principal. The money
residual must remain within 1e-6 of its opening value, and restoring immediately
after disbursement must match the original outcome without another loan.

This is a hand-constructed liquidity case, not evidence of spontaneous viable
borrowing in the 200-year ensemble. It validates the payment pathway and its
existing governance consequences, while the full completed-work requirement
remains open.

Verification: the hardware fixture passed (one test), including the closed-contact
control. Strict all-target Clippy passed. No underwriting coefficients changed.
