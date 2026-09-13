# Inactive institutional credit accounts

Institutions retain their treasury at shutdown until existing borrowing claims
have been considered. The shared `credit::estates` pass observes both operator
and institutional estate cash together, accrues live loans, and reserves
proportional early repayments before any beneficiary receives the remainder.
Loan principal does not become dues, income or a material production expense.

An inactive institution can receive repayment on an existing loan without
reopening services or becoming eligible for new borrowing/lending. When it has no
live borrowing claims or unrecovered default losses, residual treasury cash goes to its existing home town,
including an abandoned town's retained treasury. This keeps the prior beneficiary
rule. Eligible surviving receivables now follow
[dated estate succession](credit-claim-succession.md) to the same town next month;
this does not transfer personal liability.

Residual distribution uses the exact money-transfer adapter. Institutional cash
is `f64` and town cash is `f32`; any untransferable remainder stays in the
institution rather than being rounded away. Only the amount actually delivered
increments the institution's existing expense counter. Principal/interest debt
payments retain their separate credit receipts.

## Closure versus relocation

An institution is also marked inactive while physically relocating. An unfinished
relocation therefore excludes it from the estate set. Its treasury, portable
property and existing loans remain attached to the same ID. Existing payments can
reach that account while it travels, but it cannot initiate new credit until
active again. Arrival restores its existing operating state rather than creating
an estate or a new institution.

## Timing and limits

The annual cultural shutdown marks inactivity and retains the treasury. A dated
Respond settlement window immediately after `culture_month` resolves claims and
returns residual cash. This replaces the former immediate treasury dump inside
the cultural loop: later cultural actions in that loop cannot spend cash whose
credit claims have not yet been settled.

The existing Open and operator closure windows use the same estate pass. Each
pass uses opening cash; same-pass inter-estate payments cannot be respent merely
because one type or account sorts later. Unpaid debt retains its original maturity
and default rules. A separate [explicit recovery API](credit-default-recovery.md)
can return authorized cash after default. Bounded claim reassignment now exists; automatic general
recovery allocation and differentiated bankruptcy priorities remain absent.
It does not accelerate a loss
or reopen a defaulted contract merely because the institution has dissolved.

## Fixtures

The institutional fixture compares ample cash with insufficient cash, checks
proportional payments and unchanged dues, rejects new loans while inactive,
preserves active relocation identity, receives a later creditor payment, verifies
actual residual distribution and conservation, and compares serialized
continuation. Its service-order contracts are supplied fixtures, not automatic
underwriting evidence.

The existing funded relocation fixture now includes a live loan and explicitly
runs the estate pass while the institution travels. Treasury, claims and portable
property must survive unchanged before normal arrival and checkpoint continuation.

The direct annual-shutdown fixture forces an abandoned home site, runs
`culture_month`, and requires borrowed treasury cash to remain available until the
explicit estate window repays it. This distinguishes the new closure behavior
from merely testing an already inactive account.

Verification passed: institutional estate/shutdown fixture, funded relocation
with a live loan, operator estate/checkpoint fixture, all 15 CPU market tests,
and strict all-target Clippy. The full culture suite passed 12 of 13 cases,
including all hardware cases; its one failure was a stale human-guide appearance
assertion. Updating that expectation to the existing catalog text, “An ape-like
biped.”, made the affected test pass. No patron appearance data changed.
