# Shared financial primitives

Implemented in `src/finance.rs`. These primitives serve the existing economics
experiment. General advances and [secured purchases](SECURED-CREDIT.md) now
share the loan book and servicing. The active [contract consolidation](CONTRACT-CONSOLIDATION.md)
record describes implemented integration and remaining adapters.
[Creditor allocation](CREDITOR-ALLOCATION.md) covers ranked/proportional loan and
land collection with accepted coin alternatives. [Contract recovery](CONTRACT-RECOVERY.md)
adds configured guarantees, loan-estate proceedings and funded liquidation.

- `Execution`: shared opening-resource and storage reservations for partial
  claims and atomic exchange packages. Used by loan, land and forward collection.
- `Execution::pay_tender`: accepted alternative payments transfer actual tender
  units while extinguishing whole claim units at the agreed rate.
- `proportional_grants` / internal `proportional_lots`: demand-capped allocation
  from opening budgets, with explicit ranks, storage limits and whole tender lots.
- `Transfer`: a positive commodity amount between distinct agents, producing
  equal debit and credit effects. World validation and batch settlement still
  check account eligibility, balances, storage and atomic publication.
- `Obligation`: a transfer claim, settled quantity, activation condition and
  failure rule. Outstanding and settlement quantities use the claim's units.
- `Condition`: accepted exchange or accepted agreement reaching a dated month.
- `FailureRule`: reject an incomplete exchange, carry arrears, block new use,
  or block a new advance. Restrictions have their existing domain scope.

Obligations are views derived from authoritative agreement receipts, not another
mutable debt ledger. Land receipts retain native commodity collections separately
from total settled units; forwards retain delivered quantities. This prevents
alternative coin payments from creating fictitious grain collections or issuance.

## Migrated behavior

| Path | Shared behavior | Domain responsibility retained |
| --- | --- | --- |
| Loans and financed purchases | Optional collateral, conserved advances, dated claims, shared collection budgets and authoritative borrower/creditor positions | Accepted rates, amortization, interest carry, collateral consequence |
| Land payments | Dated claim, outstanding amount, bounded payment, transfer legs, new-use restriction | Annual bill creation, rights duration, oldest-due ordering, essential reserves, accepted conversion terms and collection-linked issuance |
| Prepaid harvest delivery | Dated claim, outstanding amount, bounded payment, transfer legs, new-advance restriction | Forecast underwriting, prices, treasury funding, protected stock and delivery receipts |
| Stock exchange | Full payment legs on acceptance | Posted prices, both parties' opening stock, joint storage check |
| Bilateral negotiation pilot | Both full transfer legs on acceptance | Reservation limits, quote policies, dated price receipt, permissions, joint storage check |
| Equipment purchase | Full payment leg on acceptance | Ownership, remaining life, single-fill validation and atomic asset transfer |
| Guarantee calls | Partial claim execution, original-debt reduction and same-book recourse | Configured consent, cap, expiry, trigger and priority; no lien subrogation |
| Collateral resale and estate asset sale | Shared atomic funded asset-transfer helper, title and attached-process transfer | Buyer selection, accepted price, sale timing and recipient/waterfall rules |
| Loan-estate distributions | Same executor and ranked/proportional allocation, actual payments and balance-sheet custody treatment | Authorization, stay, frozen interest, lien proceeds, closure and explicit write-off terms |

Cash financing of specialist tools remains a multi-party transaction: the buyer
and state jointly pay the provider. Its underwriting and combined accounting legs
remain specialized. Household contributions, royalties, issuance and production
are not all migrated by this change.

## Timing and limits

No scheduler changes. Annual claims are created in Due and evaluated again at the
existing arrears boundary. Due forwards settle in Acquire before new purchases,
after taxes. The shared executor retains opening spendable stock and current creditor
storage room. Loans and land claims share an explicitly ranked Due window;
forward claims retain their separate Acquire collection window. Incoming payments do not become
spendable again within that boundary. Existing commit validation publishes the
whole batch or nothing, including receipts and ownership changes.

Land arrears prevent starting another crop on the affected right, not completion
of a crop already underway. An outstanding forward prevents another advance even
before maturity, preserving the existing underwriting rule; its delivery only
becomes payable at maturity. Neither consequence forgives unpaid amounts.

This is a small settlement foundation, not a universal contract interpreter.
A scoped [negotiated-pricing pilot](NEGOTIATED-PRICING.md) now reuses its exchange
legs, including an opt-in [ZIP quoting policy](ZIP.md). The secured-credit pilot
adds scoped interest, collateral and symmetric loan balance-sheet views.
Capped original-loan guarantees, scoped proportional collection and authorized
single-denomination cash estates are implemented.
[Land/forward admission](LAND-FORWARD-ADMISSION.md) now includes native performance
claims without inventing conversion or discharge. Arbitrary event triggers,
general non-loan discharge, multicurrency recovery, competing liens, general
death/dissolution administration remain future work.
[Double-entry financial statements](FINANCIAL-STATEMENTS.md) now cover the full
report set for cash loans, valued mortgages and loan recovery. Integration with
production/inventory, minting/markets, forwards/dues and households remains open. Future contract types should supply their terms and receipts
through this shared view before introducing a second settlement mechanism.

## Validation

The counts and snapshot comparisons below describe the earlier primitive migration.
See [contract recovery](CONTRACT-RECOVERY.md#earlier-loan-estate-validation) for the earlier
451-test full run and 61-test final focused run (overlapping), and
[contract consolidation](CONTRACT-CONSOLIDATION.md) for integration gates; they are not evidence that all arrangements now compose.


- Shared primitive tests cover acceptance, maturity, partial settlement, protected
  budgets represented as spendable amounts, receiving limits, scoped restrictions,
  invalid quantities and rejection of incomplete exchange payments.
- Existing land, forward, equipment, access and storage/currency tests pass,
  including CPU/reference equality, ledger replay, checkpoint continuation,
  partial payments, alternate coin settlement and atomic rejection.
- Four 18-month reference comparisons (`opportunity-farming`, `tool-beneficial`,
  `offer-long`, `storage-exchange`) retain identical state, reports and committed
  batches against the pre-refactor snapshots, excluding planner diagnostics.
  These are regression controls, not evidence of economic balance at scale.
- Full crate suite: 156 tests passed. Formatting, Clippy with warnings denied,
  and repository artifact checks passed. Raw outputs remain under ignored `output/`.

Accepted [forward relief](DELIVERY-RELIEF.md) now stages dated extensions and
quantity write-offs through the credit boundary. It retains original acceptance
and actual deliveries, with separate release history; it does not add cash damages
or land-bill discharge.
