# Fourth household and consolidation batch

Another five household/shared passes, preserving the monthly scheduler and
excluding person self-directed policy changes. Hiring terms are preaccepted.
[Passes 21–25](INTEGRATION-PASSES-5.md) subsequently extend member-employer and
member wage-support composition beyond this batch's limits.

| Pass | Household work | Shared consolidation | Status |
| --- | --- | --- | --- |
| 16 | Budgeted external hiring | Bounded employment delivery and earned claims | Complete |
| 17 | Direct purchased hours to member work | Transfer paid-capacity basis into production | Complete |
| 18 | Preserve payroll funding | Shared earned-wage protection across allocations | Complete |
| 19 | Buy resources for wage arrears | Collective claim-driven market demand | Complete |
| 20 | Repeated hiring, work, sales and payroll | Combined interruption response and continuation controls | Complete |

## Pass 16

A static optional `hiring_budget` permits external employment in its denomination.
Acquire caps new earned wages by that month's budget and remaining opening funds,
less older earned arrears. Contracts share the allowance in existing rank/ID order.
Delivery, earning and payment stay distinct; this is an affordability gate, not
escrow or a prediction that later production will pay for itself. Missing governor,
inactive parties and legal permissions still prevent new delivery.

CPU/reference tests cover cash/charter caps, whole hours, zero budgets, competing
contracts, permissions, explicit opt-in and excluded internal workers. Idle hours
and their cost expire next Open; wages remain real earned obligations. Separate
statements and full replay agree. Member employers and household-as-worker remain
excluded pending their own delegation/contract semantics.

## Pass 17

Purchased household hours join the existing Productive candidate pool. Member
contributions are used first; paid hours fill only the useful remaining request.
The same mandate, personal rights, existing-work preservation and objective choose
the recipient. Receipts distinguish purchased availability, direction and unused
hours from member contributions. Unused purchased hours stay with the household
until period expiration.

A shared capacity allocation adapter moves proportional historical cost with
verified paired hour transfers before production. The recipient carries that cost
through existing WIP/output/loss accounting; ordinary output pooling retains its
basis. The household records transfer expense and the member transfer income, not
a second wage. Separate identities and books remain intact.

CPU/reference tests hire three hours, direct two into real production, pool output
and expire the remaining hour. All six cost units are retained or expensed exactly
once. Constitution and legal-permission controls pay earned wages but produce no
unauthorized output. Accounting, full replay and checkpoint continuation agree.

## Pass 18

The shared earned-wage claim map now serves hiring affordability, market stock
protection and household resource allocation. Discretionary member transfers cannot
consume the household's earned payroll stock. This reserve does not set statutory
priority over loan creditors or guarantee payroll against other enforced claims.
New delivery also deducts older wage arrears from available funding, including when
the contract permits continued work on credit.

A physical-payroll test keeps six grain available for earned wages despite a member
requesting two as production inputs; the request is recorded but not funded. A
second control opens with eight coins and six of old wages: only one two-coin hour
is hired, then old and new wages clear once. Existing loan-assistance and market
protection tests continue to pass. No projected output is spendable funding.

## Pass 19

Static `fund_earned_wages` allows collective needs-first market bids for stock
needed to settle the household's earned wage claims. It uses the shared native
claim map; future contractual hours do not generate a payable or a funded bid.
Loan-funding and member-loan-support choices remain separate.

An imported arrears opening trades two grain for six coins. Acquire records a real
fill but does not reuse its incoming coins for new hiring at that boundary. Close
then clears the old wage claim. Disabled funding and missing-payment-stock controls
leave arrears outstanding. Audited CPU/reference, checkpoint and full replay agree;
existing household loan funding and market controls also pass.

## Pass 20

An eight-month scenario connects two hired hours, four grain of member output,
half pooled output, a two-grain/two-coin town sale and two-coin payroll. The household
starts with four coins. The worker needs two nutrition units per month; the first
month has no grain or wage cash available at Acquire, and records an unmet need.
From month two the uninterrupted case meets that need and hires every month.
Member-retained output stays private, with its carrying cost.

A supplied month-three market closure changes the outcome. The household cannot
fund hiring in month four. Freed worker hours allow self-production, reducing later
purchase demand. It hires again in month five but not months six through eight;
all earned wages are paid and four total coins are conserved. Worker needs recover,
but reopening the market does not restore sustained household employment. This is
an observed response, not a forced recovery or evidence of optimal planning. Prices,
wage terms, work target, initial funds and the interruption are test configuration.

CPU/reference execution agrees with reversed participant/resource/contract tables.
Full ledger replay and a checkpoint before the funding shortfall reproduce the
same state, receipts and separate reconciled books. Forged purchased-hour receipts
or transfers reject without publication; valid transfers replay once. Settlement
logs expose purchased availability, direction and unused capacity alongside wages.

## Limits

Hiring remains preaccepted; the charter budget is static and supplies no recruitment,
wage discovery or forecast of business viability. No speculative wage bids or
same-boundary incoming-fund reuse were introduced. Payroll retention only protects
specified discretionary allocations; it does not escrow cash against other claims
or establish insolvency priority. Member employers, internal hires, households as
workers, wage estate claims and longer-horizon joint planning remain future work.
Person self-directed policy changes were not added.

## Verification — 2026-09-29

**352 tests passed across 29 selected suites**, including ten new household-hiring
checks and the existing 120-month household income comparison. Coverage includes
household formation/allocation/membership/dissolution, physical and coin payroll,
markets and need orders, loans/recovery, laws, inventory/process/service accounting,
and telemetry. The existing slow annual 32-person accounting test remained ignored;
the full crate suite was not run.

A final focused rerun of hiring and employment passed all 19 checks after Clippy's
style fixes; these overlap the 352 above. Formatting, strict all-target Clippy,
`git diff --check` and the repository artifact policy passed. Raw runs are under
ignored `output/economics/household-batch4-*.log`; only this summary is committed.
CPU/reference agreement, conservation and balanced statements verify the tested
execution boundaries, not economic optimality or arbitrary composition.
