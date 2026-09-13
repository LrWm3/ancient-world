# Committed credit milestones in history

Credit now emits sparse chronicle events at the existing commit boundaries:

- `loan_issued` after actual principal reaches the borrower.
- `loan_arrears` when current servicing leaves a loan in arrears.
- `loan_repaid` or `loan_precision_settled` on settlement, keeping those outcomes distinct.
- `loan_defaulted` after the servicing pass writes off the claim.
- `loan_restructured` after an accepted extension is committed.
- `loan_recovery` after a positive post-default cash transfer.

Ordinary interest accrual and partial monthly payments stay in the detailed ledger;
they do not each produce a chronicle entry. Zero recoveries and rejected
restructurings do not announce a successful transaction. Shared-currency issuance
already emitted `currency_issued`; that existing behavior remains.

Each loan event links to the previous event for the same loan. Its site anchors
identify the borrower's and lender's known sites at the time. Council anchors use
an active governed site; they are institutional associations, not proof that a
physical transaction occurred there. Stable loan subjects support event selection
within the existing u32 subject range. The credit last-event map retains full u64
loan IDs and persists across saves.

Old archives initialize an empty event-link map. There is no retrospective
origination story: the first new milestone for an old loan can have no previous
cause. Raw `Loan` accounting primitives remain independent of `History`; callers
using those primitives to construct fixtures do not receive automatic history
events. Normal history origination, payment, servicing, restructuring and recovery
paths do.

## Timing and verification

Origination occurs at its caller's funding boundary. Servicing and automatic
late-export recovery emit at Open; an explicit settlement API emits at its actual
call boundary. No Close-phase sweep infers events from already-mutated summaries.
Same-month servicing and recovery replay protections also protect event emission.

All sixteen CPU market tests and thirteen export tests passed, as did strict
all-target Clippy. The two GPU market tests were not rerun for this sparse event
change. The scheduled two-lender fixture checks exact
origination → arrears → default chains, dates, borrower anchors, repeated servicing
and serialized continuation. Existing market and export tests exercise payment,
restructuring and recovery paths.

Adding chronicle events changes event IDs. Previously completed balance runs
predate this addition; they are not evidence of identical downstream histories
where unrelated decisions depend on event IDs. No monetary rate or allocation
policy changes in this increment.

## Archive link validation

Validation now follows each persisted loan-event chain rather than checking only
that its last event has a `loan_` prefix. Every link must use a known monetary
milestone, identify the same loan within the subject index range, precede its
child by ID and have a non-later date. Missing events and branching causes are
rejected. This prevents another loan's valid event from becoming an apparently
valid causal parent. The focused fixture injects cross-loan pointers, a self-cycle
and a future-dated cause. The fixture passed, including ordinary replay and
serialized continuation, and strict all-target Clippy passed.
