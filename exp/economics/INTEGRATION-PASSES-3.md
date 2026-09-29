# Third household and consolidation batch

Five further household/shared passes. Personal self-directed policy changes and
household hiring were outside this batch; existing monthly boundaries are preserved.
[Passes 16–20](INTEGRATION-PASSES-4.md) subsequently add bounded household hiring.

| Pass | Household work | Shared consolidation | Status |
| --- | --- | --- | --- |
| 11 | Physical wage receipts | Bounded payroll plus exact contribution storage | Complete |
| 12 | Costed physical wage income | Native wage claims and noncash settlement reporting | Complete |
| 13 | Opt-in support of member loan dues | Shared current loan-claim calculation | Complete |
| 14 | Acquire resources for due obligations | Claim-driven collective market demand | Complete |
| 15 | Prioritize scarce debt support | Explicit allocation policy and combined controls | Complete |

## Pass 11

Close payroll reserves raw receipt storage and the household contribution together.
It uses the same contribution classifier/carry tracker as town trades and a bounded
shared claim-payment method. Requested wages remain the original earned claim;
partial settlement preserves arrears and the existing suspension consequence.

CPU/reference tests check zero, one and two units of available collective storage,
odd contributions, conservation, replay and subsequent delivery suspension.
No scheduler phase moved and no household hiring restriction was lifted.

## Pass 12

Physical wage claims use an explicit positive, fixed reporting value per native
stock unit. Earning recognizes service income/expense; payment exchanges inventory
for the earned claim through the existing costed noncash settlement adapter. It
creates no cash flow. Household contributions carry the received inventory cost.

CPU/reference tests combine partial physical payroll, contribution storage,
subsequent arrears clearance after capacity returns, balanced statements and
ledger replay. Missing values fail opening; valuation overflow cannot publish
simulation or reporting state. Employment and barter-accounting regressions pass.

## Pass 13

Static `support_member_loans` opts current members into Due-boundary assistance.
The shared current-claim preview includes this month's interest once, excludes
stayed/discharged/noncollectible loans, and also serves collection allocation and
market stock protection. Support adds to existing native dues; it does not assume
the member's liability. The household retains its own current loan payment before
funding member debt assistance. Need allocations retain their existing precedence.

CPU/reference tests cover enabled/disabled assistance, accrued interest, scarce
cash competing with a household loan, separate balanced accounts and replay.
A membership-exit control receives no assistance. Lending and need-order tests
exercise the shared claim preview against existing collection/protection paths.

## Pass 14

Static `fund_due_loans` extends collective needs-first bids to missing denomination
stock for current collectible loans. Member claims qualify only when member loan
support is also enabled. Private stocks offset combined enabled process/loan
requirements once. Existing prices, admission, protection, funding and storage
still decide whether a useful order can settle.

CPU/reference audited tests trade collective grain for coins, preserve the unpaid
loan after that month's Due, and use actual coins for next month's support and
collection. Disabled purchase/support and unfunded-bid controls leave debt unpaid;
collective-own debt can generate demand without member support. Checkpoint/replay
and separate balanced statements agree. Future installments are not demand and
this does not introduce refinancing or household hiring.

## Pass 15

Static `debt_support` selects reservation order (default) or lowest covered loan
claim rank followed by member ID. It only reorders loan-assistance slots within a
household/resource pool. Receipts retain rank and policy; needs, other allocation
slots, shared budgets, creditor collection and the scheduler retain their roles.
A combined native request uses its lowest covered loan rank; claim-specific escrow
and independently ranked pieces of that request remain extensions.

An identical-opening scarcity test has five coins and two five-coin requests.
Reservation order pays the first member; claim priority pays the higher-ranked
claim. Both pay exactly five. Claim priority preserves outcomes when signatories,
participants and lending configuration are reordered. Relabelled reservation
receipts fail replay atomically.

A further audited CPU/reference scenario delivers one member's actual paid labor,
pools the Close wages, and uses that cash for partial repayment of another member's
loan at the following Due. The first unpaid installment persists; the household
does not acquire the liability. Checkpoint continuation and full replay agree.

## Scope and remaining work

These passes connect existing payroll, household pooling, town barter, allocation,
loan collection and accounting. No scheduler was added and no person self-directed
policy change was introduced. Charter flags are static, false by default.

At this batch's boundary, household/member employers and onward paid-capacity
delegation still needed cost allocation and hiring budgets. Current due loans qualify for funding, not future
installments, arbitrary estate claims, refinancing or speculative work. Fixed
native-wage values do not establish FX or changing valuations. Town recovery and
mortgage configuration retain their driver exclusions. Wage insolvency, autonomous
recruitment and longer-horizon organizational planning remain open.

## Verification

Across 28 relevant integration suites: **342 passed, 1 existing ignored test**.
This combines the 27-suite final regression selection, the latest 10-test
`household_finance_loop` run (including receipt tampering), and the 24-test income
suite, without double-counting overlapping runs. The income suite includes its
120-month CPU/reference continuation and separate-book check. Additional focused
runs were used while each pass was developed.

Coverage includes household composition/integration, resource allocation, payroll,
physical inventory and barter accounting, current-loan support, priority controls,
market admission/protection, lending/credit/recovery, laws and telemetry. Formatting,
strict all-target Clippy, whitespace and repository artifact checks pass. Commands
use Rust 1.92.0 from `exp/economics`; generated logs remain under ignored
`output/economics/`. These controlled cases verify the listed combinations; they
do not demonstrate autonomous hiring, universal composition or economic balance.
