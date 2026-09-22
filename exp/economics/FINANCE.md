# Shared financial primitives

Implemented in `src/finance.rs`. These primitives serve the existing economics
experiment; they do not introduce banks, interest-bearing credit or new markets.

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
| Land payments | Dated claim, outstanding amount, bounded payment, transfer legs, new-use restriction | Annual bill creation, rights duration, oldest-due ordering, essential reserves, coin conversion and collection-linked issuance |
| Prepaid harvest delivery | Dated claim, outstanding amount, bounded payment, transfer legs, new-advance restriction | Forecast underwriting, prices, treasury funding, protected stock and delivery receipts |
| Stock exchange | Full payment legs on acceptance | Posted prices, both parties' opening stock, joint storage check |
| Bilateral negotiation pilot | Both full transfer legs on acceptance | Reservation limits, quote policies, dated price receipt, permissions, joint storage check |
| Equipment purchase | Full payment leg on acceptance | Ownership, remaining life, single-fill validation and atomic asset transfer |

Cash financing of specialist tools remains a multi-party transaction: the buyer
and state jointly pay the provider. Its underwriting and combined accounting legs
remain specialized. Household contributions, royalties, issuance and production
are not all migrated by this change.

## Timing and limits

No scheduler changes. Annual claims are created in Due and evaluated again at the
existing arrears boundary. Due forwards settle in Acquire before new purchases,
after taxes. Each resolver supplies opening spendable stock and current creditor
storage room to the shared payment calculation. Incoming payments do not become
spendable again within that boundary. Existing commit validation publishes the
whole batch or nothing, including receipts and ownership changes.

Land arrears prevent starting another crop on the affected right, not completion
of a crop already underway. An outstanding forward prevents another advance even
before maturity, preserving the existing underwriting rule; its delivery only
becomes payable at maturity. Neither consequence forgives unpaid amounts.

This is a small settlement foundation, not a universal contract interpreter.
A scoped [negotiated-pricing pilot](NEGOTIATED-PRICING.md) now reuses its exchange
legs. Arbitrary event triggers, interest, collateral, guarantees, ZIP pricing,
priority across all claims, insolvency and double-entry financial statements
remain future work. Future contract types should supply their terms and receipts
through this shared view before introducing a second settlement mechanism.

## Validation

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
