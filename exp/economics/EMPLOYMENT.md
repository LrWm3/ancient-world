# Employment agreements and wage arrears

Implemented pilot, September 2026. Employment uses the existing monthly scheduler,
shared `finance::Obligation` and `finance::Execution`, atomic gather/commit,
accepted-agreement inspection and double-entry reporting. It is not a second
balance-sheet system or a worker-specific production engine.

## Terms and acceptance

`World.employment` contains explicitly preaccepted `employment::Terms`: stable ID,
employer, worker, inclusive start/end months, capacity resource and monthly quantity,
wage denomination and integer rate per delivered unit, priority rank and arrears
policy. Household employers require the explicit charter described below; household
workers and member employers remain excluded. Actual delivery requires both parties
to be active and permitted to perform `CapacityTrade` under the current laws.

The fixture supplies consent. Agents do not yet discover, negotiate or sign these
terms through market orders. Salaries, piece rates, dismissal damages, minimum
shifts, overtime and holiday pay are not inferred from the hourly contract.
Use finer resource/coin units when fractional economic quantities are needed.

## Monthly boundary

1. **Open:** regenerate capacity using existing rules. Outstanding wages persist.
2. **Acquire:** existing acquisitions reserve first. Dated production plans also
   protect their already-granted capacity. Employment requests then reserve the
   remaining opening capacity by `(rank, agreement ID)`. Divisible delivery may
   be less than requested. Hours transfer from worker to employer, becoming usable
   at the next execution barrier. Incoming hours cannot be resold in this boundary.
3. **Earning:** only delivered hours earn wages. The receipt records requested and
   delivered hours and the earned amount. A dated wage obligation is created even
   when an ordinary employer has no money. Household hiring additionally requires
   opening affordability. Buying available hours earns the wage regardless
   of whether the employer subsequently uses those hours successfully.
4. **Productive/Consumption:** existing processes use the employer's actual
   capacity; production does not pay wages a second time.
5. **Close:** collect outstanding wages from opening denomination stock, ordered by
   `(rank, earning month, agreement ID)`. Partial payment is allowed. Incoming
   payment cannot fund another payment in the same Close. The due date is the
   following month boundary: Close evaluates that boundary before publishing the
   next Open. Thus a newly earned claim is not already in breach during Acquire.

`SuspendDelivery` withholds subsequent monthly deliveries while earlier wages
remain unpaid. `Continue` permits additional work on credit. Neither cancels
past earnings, reverses production nor confiscates output. Clearing arrears at Close
allows delivery at the next Acquire, with no backdated work. Expiring a contract
ends new deliveries but leaves claims collectible and inspectable.

This priority is scoped to employment. It does not reorder loan/dues collection,
establish statutory wage preference or add wage claims to insolvency estates.
Preexisting phases retain their timing and may have consumed cash before payroll.
The planner does not yet forecast a hiring opportunity or automatically reserve
cash for future wages. These are execution terms, not evidence that an employer's
plan is financially sustainable.

## Accounting

Earning four coins creates worker `WagesReceivable(agreement, month)` +4 and
`ServiceIncome` -4, and employer `WagesPayable(agreement, month)` -4 with either
`ServiceExpense` +4 or, when `Opening.services` is enabled, purchased-capacity cost +4.
Actual use then allocates that cost through the existing WIP/output/loss path.
Unused cost expires at the following Open without extinguishing the wage claim.

Paying three reduces the payable and receivable by three and transfers three coins
as operating cash flow. One remains owed on both balance sheets. Payment never
creates a second expense or income entry. Own labor has no imputed wage.

The operational contract accepts a stock denomination. Reporting uses the book's
coin or an explicit positive fixed `Opening.exchange_values` value for the wage
stock. Physical payment settles the claim through the existing costed inventory
exchange adapter: the employer recognizes goods sales and released historical
cost, and the worker receives inventory at the settled claim value. It creates no
cash flow and does not recognize service income twice. Outside employers may hire
household members for cash or physical wages. Acquire protects the household's percentage of own
labor, and Close pools half the actual payment, with fractional carry. Receivables
are not pooled cash. Contribution entitlement includes hours already delivered to
external employment, preventing the percentage from shrinking after that delivery.
Earned outstanding wages are protected by need-generated market budgets.

Household employers can opt into budgeted external hiring and costed delegation
to members. Member employers, household workers and internal household hires remain
rejected; terms involving future/past members retain the conservative exclusion. Physical payroll now reserves
both private receipt space and exact collective contribution space, including
fractional carry. Unstorable payment remains arrears. See the
[physical-wage verification](INTEGRATION-PASSES-3.md).
Future-period labor prepayments, refunds, wage guarantees, write-offs and estate
collection need separate accepted terms and adapters.

## Provenance and observation

`State.employment.earned` stores delivered quantity and the authoritative dated
obligation, including settled amount. `Batch.employment` stores deterministic
receipts, transfers and the candidate book. Settlement reconstructs and verifies
this adapter before publishing any effects or claims. The sidecar keeps other
acquisition validators' exact receipts intact while sharing their finite budget.
`Batch::all_transactions()` includes these transfers for ledger consumers.

The settlement observer emits `employment` records with parties, agreement,
earning month, requested/delivered/earned/paid amounts, dated and total agreement
outstanding amounts, and reason.
Requested units are capacity at Acquire and wage denomination at Close; the phase
and `Delivered`/`Collection` reason distinguish them. Standard resource-flow metrics
and transaction logs include the transfers. Full simulation/Audit disk persistence
is still separate from journal JSON round trips and in-memory continuation.

## Verification

`tests/employment.rs` checks CPU/reference equality and continuation, partial wages,
matching receivables/payables, capitalization into output, no duplicate recognition,
suspension and resumption, continuing work on credit, idle hours, expiry with arrears,
competing contracts, deterministic priority under catalog reversal, monthly capacity
and permissions, existing minting acquisition composition, observer receipts,
malformed terms, unsupported denomination, forged receipts and duplicate batches.

The repayment control demonstrates the settlement boundary: an employer receives
coins at Close but cannot reuse them until the next boundary. Its worker receives
the old wage at a later Close and resumes delivery the following month.

Validation: **61 distinct checks passed** across employment (9), accounting (14),
acquisition (8), agreements (3), shared finance (3), issuance accounting (6),
service accounting (8) and telemetry (10). Strict all-target Clippy passed.
Generated logs remain under ignored `output/economics/`. This is focused execution
and accounting verification, not evidence of labor-market calibration or employer
solvency under autonomous hiring.

## Budgeted household hiring

[Passes 16–20](INTEGRATION-PASSES-4.md) add static charter `hiring_budget`, an
optional monthly stock-denominated ceiling shared across preaccepted contracts.
It requires percentage-contribution governance and the same wage denomination.
Acquire caps delivered whole hours by that ceiling and remaining opening funds,
less earlier earned wages. Rank/ID resolves competing hires. Inactive households,
vacant governance and missing capacity-trade permission block new delivery.

This is an affordability gate, not escrow: later enforced claims may still consume
funds before payroll. Neither future output nor incoming same-boundary market
proceeds fund new hours. `Continue` does not bypass household affordability.

Purchased hours join Productive's household labor pool after member contributions.
Only useful, permitted, constitution-allowed member work receives them. Receipts
separate purchased availability, direction and unused hours; idle hours expire next
Open. With service capitalization enabled, allocation moves historical basis and
records household transfer expense/member transfer income. Production, pooling and
expiration then use the existing accounting adapters. No second wage is earned and
member accounts are not consolidated.

Earned payroll stock is protected from discretionary household member transfers.
Static `fund_earned_wages` additionally permits collective needs-first orders to buy
missing payment stock for already earned claims, not future employment. Tests cover
coin and physical wages, scarce budgets, prohibited work, real arrears purchases,
forged allocation receipts, replay and checkpoint continuation on CPU/reference.
Autonomous recruitment, price discovery for labor and wage insolvency remain open.
