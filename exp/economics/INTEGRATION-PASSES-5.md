# Fifth household and consolidation batch

Five further passes connect individual member employers to the shared employment,
household allocation, market and accounting paths. Employment terms remain
preaccepted. Person self-directed policy changes remain deferred.

Subsequent [pass 26](PAYROLL-OUTLOOK.md) adds an optional current-month payroll
outlook and compares it with this batch's unchanged earned-only default.

| Pass | Household work | Shared consolidation | Status |
| --- | --- | --- | --- |
| 21 | Members hire outside labor | Own-capacity contribution and paid-hour accounting | Complete |
| 22 | Physical payroll from member inventory | Storage, wage pooling and carrying costs | Complete |
| 23 | Optional assistance for member wage debts | Claim-driven Close allocations with explicit priority | Complete |
| 24 | Buy denomination stock for supported member wages | Shared funded requirements and finite market budgets | Complete |
| 25 | Combine member employment, collective support and changing membership | Continuation, replay and separate books | Complete |

## Pass 21

Individual household members may employ outside workers under the ordinary
preaccepted employment contract. Shared-household employment (including past/future
common membership) and households acting as workers remain excluded. The member
owes the wage; a household hiring budget does not govern a private agreement.

Own-labor contributions add back delivered outgoing hours and subtract acquired
hours from the observed capacity. Buying labor therefore does not enlarge the
membership contribution. Homogeneous capacity retains average historical cost:
a transfer may carry some already paid basis, never an imputed own-labor wage.
The existing cost adapter follows it through another member's output and pooling.

CPU/reference controls combine five own and five bought hours, direct one own
contribution into another member's production, and retain/expense all ten paid
cost units exactly once. Buying hours with zero own capacity contributes zero.
Internal employment rejects. Ledger replay and checkpoint accounting agree; all
ten previous household hiring tests also pass.

## Pass 22

Physical member-employer payroll now has cross-household composition coverage.
The same bounded wage settlement reserves raw receipt space and the worker's
mandatory pooled share, including fractional carry. No negative payment becomes
income for the employer's household; the worker's household receives only its
share of actual settlement. Storage comments now describe this wider valid scope.

Six opening space/carry controls produce zero, partial or full affordable payments
without discarding arrears. A supplied storage increase permits later collection
from the expired contract. CPU/reference runs reconcile native wage claims,
carrying costs and separate employer/member/household books; the employer's
household never acquires the personal payable. First-month replay and resumed
collection agree. All ten existing household finance-loop tests also pass.

## Pass 23

Static `support_member_wages` enables before-Close transfers for current living
members' earned wage shortfalls. Private denomination holdings offset requests.
The existing household sub-boundary supplies payment stock before ordinary Close
payroll; no monthly phase moves, no future wage is funded and the liability stays
personal. Collective earned payroll and current own-loan claims retain their stock.

The existing `debt_support` policy also orders wage-assistance slots: reservation
order or lowest covered employment rank then member ID. This does not create a
creditor priority between loans and wages. Combined per-member/resource requests
are not claim-specific escrow.

CPU/reference tests exercise disabled assistance, partial support, competing
household payroll and two member employers with different claim ranks. Reversing
signatories under claim priority preserves recipients and financial outcomes;
receipts still retain actual submission sequences. Forged allocations reject,
replay and checkpoint continuation agree, and household books contain transfer
expense rather than a member wage payable. Existing loan/hiring tests pass.

## Pass 24

`fund_earned_wages` now covers member wage shortfalls only when
`support_member_wages` also authorizes assistance. The shared requirement builder
adds enabled obligations, offsets each member's private stock once, and passes
missing quantities through the existing collective needs-first order generator.
Future delivery is still absent from demand; admission, quotes and actual payment
stock constrain fills.

An imported six-coin wage claim with two private coins generates a four-coin
collective purchase paid with two grain. Close transfers and settles the missing
four, leaving no household liability. Disabled support, disabled funding and no
barter stock controls leave four coins owed after the private two are paid. No
new work is performed before arrears clear. CPU/reference, checkpoint and replay
agree; existing need-order and household market tests pass.

## Pass 25

The six-month live scenario starts with four collective coins and twelve external
counterparty coins. A member hires two hours for four coins, produces four grain
and pools half the output. The household can buy four coins with two pooled grain,
and optionally assist actual wage debts. Contracts, work target and market quotes
are supplied; there is no new autonomous employment discovery.

The household pays month-one wages from its initial coins. Month two creates an
unfunded wage claim; month-three market purchases clear it, but delivery had already
suspended at Acquire. Work resumes in month four, followed by another arrears/funding
cycle. This deliberately records the consequence of buying for earned claims only.
Four coins remain owed after month six; sixteen total coins are conserved. It is
not a demonstration of financially sustainable or optimal hiring.

In the exit control, the employer leaves at Open three. The rotating governor
changes to the remaining adult, so the household is still operational. It stops
funding the departed member's wages; the old four-coin personal claim survives and
employment stays suspended. No private property or debt is appropriated on exit.

CPU/reference runs with reversed participant/resource/employment tables agree.
The same dated exit reproduces full ledger replay, and a checkpoint before exit
reproduces state, ledger and Audit. Separate financial statements balance. The
settlement observer now exposes all household allocations, including requested,
minimum and allocated wage support plus rank/policy and original submission order.

A native assistance control transfers six grain to the member but settles only two
when worker storage is limited. Four remain in private inventory at carrying cost,
and eight remain owed. With adequate worker space all six settle. Funding and
payment stay separate, and neither case duplicates grain, cost or wage income.

## Remaining limits

Employment is still preaccepted. Internal household hires (including common
past/future members), household-as-worker delivery, wage insolvency, wage guarantees,
autonomous recruitment and general long-horizon financing remain unsupported.
Wage assistance supplies stock; it is not creditor-specific escrow, co-borrowing or
a promise that the member can successfully pay a storage-constrained worker.
Household support priority does not establish statutory ranking across wage and
loan creditors. Person self-directed policy changes remain deferred.

## Verification — 2026-09-29

**360 tests passed across 30 selected suites**, including all eight new member
employment checks and the existing ten-year CPU/reference household income loop.
Coverage includes household formation, governance, allocation, membership, disposal,
employment, wage arrears, need orders, town markets, laws, loans/recovery, inventory,
process/service costing and telemetry. The existing slow annual 32-person accounting
test remained ignored; the full crate suite was not run.

Formatting, strict all-target Clippy, whitespace checks and the repository artifact
policy passed. Focused runs overlap this final selection and are not counted twice.
Generated logs remain under ignored `output/economics/household-batch5-*.log`;
commits contain source, tests and Markdown only. These checks establish the listed
execution/accounting combinations, not autonomous hiring viability or universal
contract composition.
