# Credit implementation record

This tracks delivery of the [credit and currencies design](credit-and-currencies-design.md).
The monetary experiment is not yet enabled in history. Neither issuance nor
multiple-currency exchange is implemented.

## Account and timing audit

Inspected against the current source before adding the contract ledger:

| Source | Existing authority | Integration consequence |
| --- | --- | --- |
| `society.rs::social_year` | Annual taxes debit current town market cash and credit the controlling council; autonomy and office capacity affect collection | Underwrite future collectible cash, not agricultural output. Record prior actual collection; payment due after annual Respond can first settle at the next Open |
| `export_contracts.rs` and economy dispatch | Buyer-funded escrow; producer payment at dispatch | Delivery loss is not automatically borrower revenue loss. A delivery-paid contract needs explicit different settlement terms |
| `enterprises.rs` | Operators receive proportional affordable payment for completed services from town funds | Firms cannot pledge the town's export goods or proceeds. Financing must extend the enterprise cash ledger without entering operating revenue or profit |
| `culture.rs::Institution` | Treasury follows stable institution identity | Relocation need not change debtor identity. Dissolution needs an explicit settlement/default path |
| `economy.rs::economy_residuals` | Counts town, firm, institution, household, council, expedition, relocation and escrow money | Claims must not be added to money supply; eventual issuance enters as a declared external monetary flow |
| `household_economy.rs::withdraw/deposit` | f32 town pools interact with f64 wallets using actual representable transfers | Credit adapters need paired transfer tests at large and small balances; nominal requested principal is not proof of cash delivered |

Remaining audit at integration: every closure, succession and payment path used by
an eligible account; source IDs for funded orders; treatment of simultaneous
ordinary spending and debt service. This table does not prove those adapters exist.

## First component: contract ledger

`src/credit.rs` introduces currency and account identities, repayment-source
references, loan terms and dated contract entries. It deliberately contains no
second store of cash. A caller must transfer existing account money and record the
actual amount. This is a component API, not authorization to create a historical
loan without the future account/policy adapter.

Implemented contract behavior:

- Fixed annual simple interest accrued monthly on remaining principal, without
  compounding unpaid interest. The disbursement month accrues no interest.
- Interest-first payment quotes and partial principal repayment; full early
  repayment closes the contract without a penalty.
- Maturity produces arrears, followed by a configured grace period.
- One term extension, without capitalizing interest or forgiving balances; consent
  and revised repayment evidence remain responsibilities of the pending policy.
- Default writes off claims and liabilities; it does not touch money.
- Serializable entries and balance reconciliation; batched accrual preserves the
  monthly addition order and repeated observation of a boundary does not accrue twice.

The first component uses a single maturity payment rather than a general
installment schedule. Installment allocation, consent, underwriting, account
transfers, history persistence integration, and automatic default/closure remain
pending. Monthly accrual receipts are suitable for initial fixtures; retention
and summarized reporting need review before long ensembles.

## Next implementation boundary

1. Add existing-account adapters and atomic transfer quotes; extend firm financing
   counters independently of revenue. Test insufficient balances and rounding.
2. Persist the ledger in `History`, default old histories to no loans, and validate
   counterparties and record IDs. Do not silently forgive dangling accounts.
3. Add source-aware requests, lender reserves and explicit joint allocation, then
   integrate the monthly payment and disbursement windows.
4. Deliver the council tax bridge and commercial payment pilots, with failures and
   exact cash-ledger tests, before any issuance experiment.

No balance improvement is claimed from the standalone component.

## Initial verification

Four focused contract tests passed: analytical simple interest and partial payment,
serialized/monthly/batched continuation, bounded restructuring/default, and invalid
payment rejection without mutation. The complete ordinary library suite passed
155 tests (130 extended/GPU cases remain explicitly ignored). Strict all-target
Clippy passed. These checks establish a tested component, not integrated monetary
conservation or a favorable balance result. Raw logs remain in ignored `output/`.
