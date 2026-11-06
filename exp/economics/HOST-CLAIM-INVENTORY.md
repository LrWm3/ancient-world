# Source claims before liquid-asset import

`StockBindings::inspect_claims` now inventories the main-world claims associated
with the town and household accounts selected for
[liquid-asset import](HOST-STOCK-IMPORT.md). The bootstrap calls this inspection
and refuses to start if it would omit a live debt, receivable or default history.
This tightens the previous partial-import API, which explicitly excluded those
positions but could still start from the associated cash.

The inspection reads the completed source history. It does not accrue interest,
assign a creditor, collect money or install experimental loan terms. Household
beneficial shares are retained as source evidence without duplicating town stock.

## Why admission precedes loan migration

The main engine accrues simple interest monthly and starts ordinary collection
at maturity. Its collection policy shares bounded opening cash among claims;
grace/default and later recovery have their own records. The experiment supports
both monthly installments and opt-in [maturity collection](LOAN-REPAYMENT-SCHEDULES.md).
Mapping a principal balance into the default installment schedule would still
change when money becomes due. Calendar rebasing, rate precision,
collection policy, default recovery and assignment timing all need explicit
support before these contracts can execute faithfully in the experimental engine.

This iteration makes the unsupported boundary inspectable and enforceable. It
does not introduce a second debt implementation or change either scheduler.

## Scope and receipts

The source scope consists of `Town(site)` and every household currently at that
site. Bindings must cover those wallets exactly once, with distinct target agents.
The native wallet and credit validators check account receipts, disbursements,
repayments, recovery and creditor-ownership chains. Invalid source credit is an
error even when its parties are outside the selected site. Full archive validation
remains the caller's responsibility.

The adapter additionally requires finite, nonnegative derived report totals and
consistent default/write-off evidence. Native balance reconciliation alone can
accept individually finite receipts whose cumulative reported repayment overflows,
or a written-off zero balance mislabeled as repaid. Neither can be published as a
valid claim inventory or admitted into the liquid bootstrap.

Each relevant contract appears once. Its receipt retains:

- Original lender, borrower, current creditor and next-boundary creditor from
  assignments already committed at the completed boundary. External parties
  retain their main-world identity and have no invented experimental wallet.
- Contract ID, currency, repayment source, opening/accrual/maturity dates, grace
  period and annual simple rate. Amounts remain in original source money units;
  an amount smaller than one experimental tick is still a claim.
- Outstanding principal/interest, repaid amounts, write-offs, precision write-offs,
  recoveries and remaining default losses as separate quantities. Remaining losses
  use sequential subtraction in native recovery order.
- Assignment request IDs, dates and counterparties. These are provenance pointers,
  not a replacement for original consent/estate-authority evidence in the archive.

A contract is relevant if the selected accounts include its borrower, its current
creditor or its creditor at the next opening. A pending assignment into a household
therefore blocks import before becoming effective. An outgoing assignment still
blocks at the current boundary; after it takes effect, an otherwise external
contract no longer belongs to the original lender's selected scope.

Nonzero outstanding claims block. Every relevant default also blocks, even after
full recovery, because native credit policy retains default history. Repaid loans
and precision-settled loans do not block solely because they are historical;
their evidence remains in the receipt. Unrelated contracts do not block, provided
the source credit ledger validates. This is a scoped credit check, not proof of
complete migration or readiness to replace the main engine.

Household rows preserve the original share, head identity and vacancy date. The
share sum is reported without normalization or interpretation as a second stock
pool. This does not establish private/public stock partitions, reproduce property
law or implement member ownership, inheritance or distributions in the experiment.

The successful bootstrap's `StockReceipt.claims` retains the inspection. Its
existing balances and financial book contain only the liquid assets actually
imported. No receivable is credited as cash, no loan is reissued, and no opening
income is created from claim evidence.

## Inspect an export

From `exp/economics`, using the stock binding shape documented in the bootstrap
API and a normal main-engine `--history-export` file:

```sh
cargo +1.92.0 run --locked --release -j8 --features ancient-world-host \
  --example host_claim_inventory -- \
  ../../output/economics/stock-bindings.json \
  ../../output/economics/completed-history.json \
  > ../../output/economics/claim-inventory.json
```

The JSON includes `blocking_loans`, `credit_check_passed` and the inventory. A
valid but blocked inventory prints successfully so callers can inspect it;
malformed inputs fail. A passed credit check does not validate stock resource
bindings, quantity scales, the target `World`, or future replay inputs.

## Validation

Focused controls use the main engine's actual origination, repayment, assignment
and recovery methods, with native loan accrual/write-off for the default fixture.
They cover both borrower and creditor scopes, pending household assignments,
assignment away from the original lender, partial/full recovery, unrelated and
repaid controls, fractional claims, malformed source ledgers, source immutability
and JSON history continuation. The existing stock/labor tests continue to cover
Reference/CubeCL execution and depletion after bootstrap.

Sub-agent review reproduced an aggregate interest overflow that JSON rendered as
`null`; explicit derived-total validation and a regression now reject it. A second
regression demonstrates rejection of a native-validator-accepted default relabeled
as repaid. The default fixture also checks interest-first recovery: a 4-unit
principal with 0.125 accrued interest loses 4.125 on default; recovering 1 pays the
interest and 0.875 principal, leaving 3.125. Full recovery still retains default
history. The CLI's initial hand-built smoke fixture omitted required Economy
serialization fields; correcting the fixture made it a complete input for the
native deserializer without changing the adapter's input requirements.

Verification on 2026-11-06: all 22 focused host tests passed. Strict release Clippy
passed for all targets with the host feature and warnings denied; release checking
without the feature, formatting and diff checks passed. Controlled CLI fixtures
reported no blockers for a credit-free source, loan 0 as the sole blocker for a
4-unit council-to-town loan, and an error with no JSON publication when the source
disbursement receipt was missing. Generated inputs, reports and build logs remain
under ignored `output/economics/claim-inventory/` and `output/economics/`.

```sh
cargo +1.92.0 test --locked --release -j8 --features ancient-world-host \
  --test host_ancient_world
cargo +1.92.0 clippy --locked --release -j8 --features ancient-world-host \
  --all-targets -- -D warnings
cargo +1.92.0 check --locked --release -j8 --no-default-features --all-targets
cargo +1.92.0 fmt --check
```

These are controlled fixtures, not a generated-world rollout or a balance
comparison. Live contract execution,
material property migration, due arrivals and exclusive live Reserve integration
remain outstanding.
