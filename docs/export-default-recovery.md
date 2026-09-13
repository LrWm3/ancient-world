# Recovery from late export proceeds

This opt-in policy connects actual delivery-paid export cash to previously
defaulted loans backed by the same contract and expected payment date.

Configure `history.credit.export_recovery.policy.enabled` through the reusable
history API, or use `--headless --export-default-recovery` when creating or
resuming history. `--export-default-recovery=false` disables future recovery;
omitting the flag preserves an archived policy. The default is disabled; this is a separate decision from enabling
new commercial loans. The policy defaults to at most 25% of newly observed seller
proceeds and a 100-unit operating cash floor.

## Monthly boundary and allocation

Open first settles arrivals and actual export payments, prepares the economy,
and services live credit. Export recovery follows that servicing pass, before
the second estate settlement and new Reserve work requests.

The policy observes cumulative cash actually paid to sellers. Escrow, cargo
value, expected proceeds and buyer refunds cannot fund it. A loan must already
have defaulted before this month, and its original borrower and export source
must match the new payment. It does not immediately undo a default recorded by
the preceding servicing pass.

Matching losses share the source's proceeds allowance proportionally. The
borrower's remaining cash further caps their combined allowances after:

- The larger of the policy floor, explicit debt-service reserve and input-cost
  estimate from the prior production plan at current opening stocks/prices.
- All remaining live principal and interest, including unmatured obligations.

These are conservative guards, not a reservation of future production. Current
month's production plan does not exist yet at this Open boundary. All proposals
use one cash snapshot; a creditor's incoming recovery cannot fund another
proposal in the same pass merely because it sorts later.

Actual transfers use the [default recovery ledger](credit-default-recovery.md).
They preserve the original write-off, default status and borrowing exclusion.
Unavailable creditors leave their shares unspent. Abandoned borrowers are
excluded from automatic recovery; explicit retained-account settlement remains
available separately.

## Observation and persistence

The first observation establishes a baseline. Older archives do not turn all
historical sale revenue into collectible proceeds. Disabled months still update
observed payments, so toggling the policy does not replay money received while
it was disabled. Each month is observed once.

Unused allowance stays with the borrower and is not carried forward as an
automatic claim on unrelated income. A later actual installment can be observed
separately. Partial exact-transfer shortfalls also remain cash, not invented
repayments. Automatic requests use the reserved high-bit ID namespace; interrupted
partial batches surface an error rather than replaying committed transfers.

General estate-surplus allocation and legal assignment of creditor claims remain
unfinished. This policy does not promise better population or food outcomes;
it needs balance evaluation after the integration fixtures pass.


## Verification

All thirteen CPU export-contract tests passed, including a delayed cargo fixture
that defaults two claims before delivery. It verifies no recovery from escrow,
proportional allowances, operating reserves and live-debt priority, disabled and
legacy baselines, invalid policy rejection, same-month replay protection, cash
conservation and serialized continuation. Original default histories remain intact.
Strict all-target Clippy passed. These tests establish the transfer and timing
contract; they do not establish a beneficial long-run balance effect.

The headless flag parser test also passed for omitted, explicitly enabled and
explicitly disabled settings. Omitting the flag does not override archive state.
