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

## Existing-account settlement adapter

`src/credit/accounts.rs` now resolves council, institution, town and operator IDs
into their existing balances. Unsupported currencies, missing/inactive accounts,
self transfers and invalid amounts are rejected before mutation. This component
still requires the pending underwriting and loan-commit layer; calling a cash
adapter alone is not evidence of an authorized loan.

A quote caps payment to actual funds and reduces the candidate until both account
representations admit the same debit and credit. If precision prevents a useful
exact transfer, it returns zero without modifying either account. It does not
mint a rounding balance, overdraw the lender or claim the requested amount was
paid. Both endpoints and cumulative operator counters are checked before commit.

Operators persist separate principal-received/paid and interest-received/paid
counters. Their cash reconciliation includes these flows. Only net interest
enters the earned-profit calculation; principal does not become service revenue
or earned dividends. Old operator records default all financing counters to zero.
No automatic loan origination, collection, issuance or policy is enabled yet.

Adapter verification: seven focused credit tests passed, including 500 combinations
of account precision, balance scale and requested amount. The hardware-enabled
operator/town fixture passed, verifying equal cash changes, unchanged service
revenue, operator reconciliation, rejection without mutation and serialized
continuation. The ordinary library suite passed 158 tests (131 extended cases
ignored), and strict all-target Clippy passed. Initial tests exposed a precision
case that a common-step-size quote did not handle; the final quote checks actual
endpoint changes and declines incompatible transfers. These are accounting tests,
not the planned multi-seed credit/issuance balance experiments.

## Persisted loan/cash commit boundary

`History.credit` now archives loans and their cash receipts, defaulting missing
old-history fields to an empty state. `commit_credit_loan` consumes existing
account funds and records the amount actually delivered as principal.
`pay_credit_loan` accrues through the current month, respects a caller-provided
allowance and available cash, then records the corresponding debt reduction.
History validation compares cash disbursements and repayments with loan entries.

These are explicit commit APIs for already authorized transactions. Automatic
underwriting, consent, repayment-source verification/reservation, proportional
collection and scheduler invocation remain pending. The test's repayment-source
reference is a fixture, not evidence of a funded commercial contract.

Commit-boundary verification: the generated-world loan test passed, checking
actual transferred principal, a bounded repayment that leaves arrears, old-field
defaults, matching serialized continuation and missing-receipt rejection. A
CPU-only `markets` integration test also passed: 25 units move from a 100-unit
lender, an underfunded repayment leaves principal outstanding, and subsequent
write-off changes debt without changing the 100-unit money supply. All 158 active
library tests passed (132 extended cases ignored). Strict all-target Clippy passed
before the additional CPU fixture; that fixture compiled and passed separately.

## Snapshot underwriting and funding rounds

`credit::underwriting` now accepts dated lender offers, repayment evidence and
requests. The resolver protects offered operating reserves, caps borrower/lender
principal exposure, discounts net receipts, subtracts existing pledges, and checks
beneficiary, timing, evidence age, loss risk and the offered return. New borrowing
and lending roles cannot overlap in the same batch or active portfolio. Recent
defaults exclude the borrower temporarily.

Approved requests share three finite budgets: lender cash, borrower capacity and
repayment-source capacity. Each receives the smallest of the three proportional
scales. Stable request IDs fix summation order; reversing input order does not
create first-applicant priority. This conservative pass can leave capacity unused.
Initial policy defaults are explicit game-experiment parameters, not calibrated
financial estimates.

`History::fund_credit_requests` caps offered cash to the actual existing account,
resolves a round, then commits approved loans through the cash/debt adapter. It
retains offers, evidence, requests, grants and funded loan IDs. Replaying a request
ID in the same month is rejected before transfer. Individual transfers remain
atomic; a later settlement error retains an incomplete round, which closing
validation rejects rather than treating it as a completed or unexecuted batch.

The source producer is still pending: tests supply explicit evidence fixtures.
Actual tax-history and commercial-payment observations must generate those inputs
before automatic history lending or ensemble balance conclusions are justified.
Consent currently means an explicit offer/request, not an autonomous political
or merchant decision. Scheduler collection, closures and default policy likewise
remain pending.

Underwriting verification: three CPU resolver tests passed, covering competing
claims on one receipt, reversed input order, lender reserves, existing pledges,
stale evidence, wrong beneficiaries, loss risk and absent repayment capacity.
Both CPU credit integration tests passed; the funding case caps an overstated
1,000-unit offer to 100 actual units, preserves 20 in operating reserve and funds
two 40-unit loans. Replayed requests leave history unchanged. All 161 active
library tests passed (132 extended cases ignored). Strict all-target Clippy passed
before a declaration-order-only move of the underwriting test module. Long-run
credit and issuance experiments have not started.

## Council receipt evidence

Annual Respond now records each council's actual collected tax and requested town
support immediately after `social_year`. Observations are dated, archived and
idempotent. Older archives start without tax evidence. Zero collections remain
visible, including councils that have lost all their sites.

A council forecast uses the smaller of its previous collection and the current
collectible levy on active controlled towns. Collection and forecasting share the
same cash × tax rate × autonomy × office-capacity expression. Production and
council treasury balances do not count as future receipts. Requested support is
an operating deduction even when it was unaffordable. Evidence expires at the next
annual collection; its source names that future collection, not the previous one.
This is a conservative game underwriting ceiling, not a calibrated default model.
Other essential operating costs still need to enter the automatic lending policy.

Focused tests cover cash/policy/administration changes, reduced and lost tax base,
source dates, expiration and serialized observation continuation. Automatic
council lending and scheduled collections remain pending.

Verification for this increment: both focused tax tests passed; all 163 active
library tests passed (132 extended/GPU tests ignored); strict all-target Clippy and
repository artifact checks passed. These checks do not establish long-run lending
benefits or replace the planned four-arm monetary experiments.
