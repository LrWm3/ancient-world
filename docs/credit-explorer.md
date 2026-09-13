# Credit and issuance inspection

Open **Civilizations and history → Towns and economy → Credit and issuance
(experimental)**. This is a read-only panel; it does not originate loans, change
policies, accrue interest or transfer money.

The panel shows the four policy switches, total authorized shared-currency
issuance, recent contracts, underwriting outcomes and issuance receipts. Contracts
retain account kind/ID, currency, repayment evidence, opening/maturity month and
simple annual interest rate. Their amount categories are deliberately separate:

- Original principal is financing received, not production or sales.
- Principal and interest due are live claims, not spendable cash.
- Repayments report principal and interest separately.
- Original default losses remain visible after later recovery.
- Precision write-offs are not defaults.
- Post-default recoveries are actual separate transfers, not reopened debt.

Each recent-record list is capped at 32. Older contracts remain available through
`Credit::loan_reports()` and the full history archive/export. Underwriting rounds
show requested, eligible and granted amounts plus recorded capacity snapshots;
these describe decisions at that time, not current lending capacity. The panel
uses stable account identifiers rather than assuming that an institution or
operator is the same legal owner as its home town.

The reusable report derives all amounts from existing records and adds no
persistent state. It retains denomination per loan instead of summing unlike
currencies. Shared issuance remains the only implemented issuance denomination.

## Verification and remaining work

The accounting fixture checks a 100-unit loan, 12 interest accrued, a 22-unit
payment, 90 principal default loss and a later 5-unit recovery. It verifies that
reporting preserves the default, does not mutate credit state and survives
serialization. The fixture and strict all-target Clippy passed. The panel was
compiled, but this change has not had an interactive visual inspection.

This closes the basic explorer-inspection gap, not all monetary reporting:
click-through account navigation, dedicated monetary chronicle events, household
cash-distribution plots and per-project causal funding reports remain separate
work. Claims, cash and historical losses must stay distinguishable in those views.
