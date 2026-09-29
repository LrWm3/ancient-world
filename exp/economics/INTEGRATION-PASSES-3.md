# Third household and consolidation batch

Five further household/shared passes. Personal self-directed policy changes and
household hiring remain out of scope; existing monthly boundaries are preserved.

| Pass | Household work | Shared consolidation | Status |
| --- | --- | --- | --- |
| 11 | Physical wage receipts | Bounded payroll plus exact contribution storage | Complete |
| 12 | Costed physical wage income | Native wage claims and noncash settlement reporting | Complete |
| 13 | Opt-in support of member loan dues | Shared current loan-claim calculation | Complete |
| 14 | Acquire resources for due obligations | Claim-driven collective market demand | Pending |
| 15 | Prioritize scarce debt support | Explicit allocation policy and combined controls | Pending |

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
