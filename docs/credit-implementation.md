# Credit implementation record

This tracks delivery of the [credit and currencies design](credit-and-currencies-design.md).
The council credit pilot is opt-in; it is not enabled by default. Neither issuance nor
multiple-currency exchange is implemented.

## Account and timing audit

Inspected against the current source before adding the contract ledger:

| Source | Existing authority | Integration consequence |
| --- | --- | --- |
| `society.rs::social_year` | Annual taxes debit current town market cash and credit the controlling council; autonomy and office capacity affect collection | Underwrite future collectible cash, not agricultural output. Record prior actual collection; payment due after annual Respond can first settle at the next Open |
| `export_contracts.rs` and economy dispatch | Buyer-funded escrow; producer payment at dispatch | Delivery loss is not automatically borrower revenue loss. A delivery-paid contract needs explicit different settlement terms |
| `enterprises.rs` | Operators receive proportional affordable payment for completed services from town funds | Firms cannot pledge the town's export goods or proceeds. Financing must extend the enterprise cash ledger without entering operating revenue or profit |
| `culture.rs::Institution` | Treasury follows stable institution identity | Relocation need not change debtor identity. Dissolution needs an explicit settlement/default path |
| `economy.rs::economy_residuals` | Counts town, firm, institution, household, council, expedition, relocation and escrow money | Claims must not be added to money supply; eventual issuance enters as a declared external monetary flow |
| `household_economy.rs::withdraw/deposit` | f32 town pools interact with f64 wallets using actual representable transfers | Credit adapters need paired transfer tests at large and small balances; nominal requested principal is not proof of cash delivered |

Remaining audit at integration: every closure, succession and payment path used by
an eligible account; source IDs for funded orders; treatment of simultaneous
ordinary spending and debt service. This table does not prove those adapters exist.

## First component: contract ledger

`src/credit.rs` introduces currency and account identities, repayment-source
references, loan terms and dated contract entries. It deliberately contains no
second store of cash. A caller must transfer existing account money and record the
actual amount. This is a component API, not authorization to create a historical
loan without the future account/policy adapter.

Implemented contract behavior:

- Fixed annual simple interest accrued monthly on remaining principal, without
  compounding unpaid interest. The disbursement month accrues no interest.
- Interest-first payment quotes and partial principal repayment; full early
  repayment closes the contract without a penalty.
- Maturity produces arrears, followed by a configured grace period.
- One term extension, without capitalizing interest or forgiving balances; consent
  and revised repayment evidence remain responsibilities of the pending policy.
- Default writes off claims and liabilities; it does not touch money.
- Serializable entries and balance reconciliation; batched accrual preserves the
  monthly addition order and repeated observation of a boundary does not accrue twice.

The first component uses a single maturity payment rather than a general
installment schedule. Installment allocation, consent, underwriting, account
transfers, history persistence integration, and automatic default/closure remain
pending. Monthly accrual receipts are suitable for initial fixtures; retention
and summarized reporting need review before long ensembles.

## Next implementation boundary

1. Add existing-account adapters and atomic transfer quotes; extend firm financing
   counters independently of revenue. Test insufficient balances and rounding.
2. Persist the ledger in `History`, default old histories to no loans, and validate
   counterparties and record IDs. Do not silently forgive dangling accounts.
3. Add source-aware requests, lender reserves and explicit joint allocation, then
   integrate the monthly payment and disbursement windows.
4. Deliver the council tax bridge and commercial payment pilots, with failures and
   exact cash-ledger tests, before any issuance experiment.

No balance improvement is claimed from the standalone component.

## Initial verification

Four focused contract tests passed: analytical simple interest and partial payment,
serialized/monthly/batched continuation, bounded restructuring/default, and invalid
payment rejection without mutation. The complete ordinary library suite passed
155 tests (130 extended/GPU cases remain explicitly ignored). Strict all-target
Clippy passed. These checks establish a tested component, not integrated monetary
conservation or a favorable balance result. Raw logs remain in ignored `output/`.

## Existing-account settlement adapter

`src/credit/accounts.rs` now resolves council, institution, town and operator IDs
into their existing balances. Unsupported currencies, missing/inactive accounts,
self transfers and invalid amounts are rejected before mutation. This component
still requires the pending underwriting and loan-commit layer; calling a cash
adapter alone is not evidence of an authorized loan.

A quote caps payment to actual funds and reduces the candidate until both account
representations admit the same debit and credit. If precision prevents a useful
exact transfer, it returns zero without modifying either account. It does not
mint a rounding balance, overdraw the lender or claim the requested amount was
paid. Both endpoints and cumulative operator counters are checked before commit.

Operators persist separate principal-received/paid and interest-received/paid
counters. Their cash reconciliation includes these flows. Only net interest
enters the earned-profit calculation; principal does not become service revenue
or earned dividends. Old operator records default all financing counters to zero.
No automatic loan origination, collection, issuance or policy is enabled yet.

Adapter verification: seven focused credit tests passed, including 500 combinations
of account precision, balance scale and requested amount. The hardware-enabled
operator/town fixture passed, verifying equal cash changes, unchanged service
revenue, operator reconciliation, rejection without mutation and serialized
continuation. The ordinary library suite passed 158 tests (131 extended cases
ignored), and strict all-target Clippy passed. Initial tests exposed a precision
case that a common-step-size quote did not handle; the final quote checks actual
endpoint changes and declines incompatible transfers. These are accounting tests,
not the planned multi-seed credit/issuance balance experiments.

## Persisted loan/cash commit boundary

`History.credit` now archives loans and their cash receipts, defaulting missing
old-history fields to an empty state. `commit_credit_loan` consumes existing
account funds and records the amount actually delivered as principal.
`pay_credit_loan` accrues through the current month, respects a caller-provided
allowance and available cash, then records the corresponding debt reduction.
History validation compares cash disbursements and repayments with loan entries.

These are explicit commit APIs for already authorized transactions. Automatic
underwriting, consent, repayment-source verification/reservation, proportional
collection and scheduler invocation remain pending. The test's repayment-source
reference is a fixture, not evidence of a funded commercial contract.

Commit-boundary verification: the generated-world loan test passed, checking
actual transferred principal, a bounded repayment that leaves arrears, old-field
defaults, matching serialized continuation and missing-receipt rejection. A
CPU-only `markets` integration test also passed: 25 units move from a 100-unit
lender, an underfunded repayment leaves principal outstanding, and subsequent
write-off changes debt without changing the 100-unit money supply. All 158 active
library tests passed (132 extended cases ignored). Strict all-target Clippy passed
before the additional CPU fixture; that fixture compiled and passed separately.

## Snapshot underwriting and funding rounds

`credit::underwriting` now accepts dated lender offers, repayment evidence and
requests. The resolver protects offered operating reserves, caps borrower/lender
principal exposure, discounts net receipts, subtracts existing pledges, and checks
beneficiary, timing, evidence age, loss risk and the offered return. New borrowing
and lending roles cannot overlap in the same batch or active portfolio. Recent
defaults exclude the borrower temporarily.

Approved requests share three finite budgets: lender cash, borrower capacity and
repayment-source capacity. Each receives the smallest of the three proportional
scales. Stable request IDs fix summation order; reversing input order does not
create first-applicant priority. This conservative pass can leave capacity unused.
Initial policy defaults are explicit game-experiment parameters, not calibrated
financial estimates.

`History::fund_credit_requests` caps offered cash to the actual existing account,
resolves a round, then commits approved loans through the cash/debt adapter. It
retains offers, evidence, requests, grants and funded loan IDs. Replaying a request
ID in the same month is rejected before transfer. Individual transfers remain
atomic; a later settlement error retains an incomplete round, which closing
validation rejects rather than treating it as a completed or unexecuted batch.

The source producer is still pending: tests supply explicit evidence fixtures.
Actual tax-history and commercial-payment observations must generate those inputs
before automatic history lending or ensemble balance conclusions are justified.
Consent currently means an explicit offer/request, not an autonomous political
or merchant decision. Scheduler collection, closures and default policy likewise
remain pending.

Underwriting verification: three CPU resolver tests passed, covering competing
claims on one receipt, reversed input order, lender reserves, existing pledges,
stale evidence, wrong beneficiaries, loss risk and absent repayment capacity.
Both CPU credit integration tests passed; the funding case caps an overstated
1,000-unit offer to 100 actual units, preserves 20 in operating reserve and funds
two 40-unit loans. Replayed requests leave history unchanged. All 161 active
library tests passed (132 extended cases ignored). Strict all-target Clippy passed
before a declaration-order-only move of the underwriting test module. Long-run
credit and issuance experiments have not started.

## Council receipt evidence

Annual Respond now records each council's actual collected tax and requested town
support immediately after `social_year`. Observations are dated, archived and
idempotent. Older archives start without tax evidence. Zero collections remain
visible, including councils that have lost all their sites.

A council forecast uses the smaller of its previous collection and the current
collectible levy on active controlled towns. Collection and forecasting share the
same cash × tax rate × autonomy × office-capacity expression. Production and
council treasury balances do not count as future receipts. Requested support is
an operating deduction even when it was unaffordable. Evidence expires at the next
annual collection; its source names that future collection, not the previous one.
This is a conservative game underwriting ceiling, not a calibrated default model.
Other essential operating costs still need to enter the automatic lending policy.

Focused tests cover cash/policy/administration changes, reduced and lost tax base,
source dates, expiration and serialized observation continuation. Automatic
council lending and scheduled collections remain pending.

Verification for this increment: both focused tax tests passed; all 163 active
library tests passed (132 extended/GPU tests ignored); strict all-target Clippy and
repository artifact checks passed. These checks do not establish long-run lending
benefits or replace the planned four-arm monetary experiments.

## Monthly servicing integration

Open now services existing loans after arrivals, appeals and economy preparation,
before new work reservations. Principal is due at maturity; simple interest is
accrued monthly. Same-borrower debts share one opening budget proportionally,
with an explicit protected cash reserve and a default 25% available-cash share.
These are experimental payment policies, not measured historical behavior.
New lending can later be disabled without erasing existing obligations.

The schedule records due amounts, opening cash, protected reserves, allocated
payments, actual transfers, unavailable accounts and default. At the end of the
contract's grace period it attempts the affordable payment first, then writes off
remaining claims/liabilities without changing cash. Automatic restructuring and
closure recovery are not implemented by this step.

Verification: the scheduled two-lender fixture passed, including pre-maturity
nonpayment, protected cash, proportional payments, grace-period default, exact
money residual, same-month idempotence and serialized continuation. All nine
active market integration tests passed (two extended cases ignored), and strict
all-target Clippy passed. The first checkpoint attempt exposed JSON enum-map key
incompatibility; protected reserves now use validated account–amount entries.
No long-run credit/issuance benefit is claimed by these boundary tests.

## Automatic council pilot

`History.credit.council_policy.enabled` opts into the first automatic borrowing
path. Reserve compares current administration requirements and the preceding
month's observed household relief demand with actual council cash. A positive gap
can request a tax-backed loan; no collection history means no request. Underwriting
subtracts annualized operating demand as well as recorded annual town-support needs
from expected receipts. This deliberately conservative ceiling needs calibration.

Other councils offer a configured fraction of surplus after a three-month operating
reserve and a fixed cash floor. Only direct open-route contacts can lend; active
war excludes contact. Requests split across eligible offers before the common
exposure/source/cash resolver. Loans transfer real cash into the existing treasury,
so ordinary relief and administration rules still determine spending and work.
No special loan-funded output or bypass of distribution policy is introduced.

Defaults: disabled experiment, reserve floor 100 currency units, reserve duration
three months, offered surplus fraction 25%, annual simple rate 6%. They are editable
game policies. Existing loans continue servicing if new lending is disabled.
Automatic merchant lending, delivery-paid contracts, consensual restructuring,
closure recovery, UI/CLI controls and long-run comparisons remain unfinished.

Headless runs can set `--council-credit` or `--council-credit=false` before history
advancement. Omitting the flag preserves an archived policy. A history must already
exist or be founded with the normal civilization options. The switch changes new
council lending only; it does not cancel obligations or enable minting.

Pilot verification: a controlled borrower receives 20 currency units from two
connected councils, with total council cash unchanged. Missing tax history,
disabled lending, closed routes and an empty current tax base each prevent funding.
Repeated calls and serialized continuation preserve the funded result. The initial
fixture caught a wrapper returning the allocation-round index instead of its
funded-loan count; this was corrected without changing the allocation API.
All ten active market integration tests passed (two extended cases ignored), both
CLI tests passed, and strict all-target Clippy passed. These establish the request →
underwriting → treasury transfer path; they do not yet establish improved food
access, completed work or viable repayment in a long history.

## First live comparison and missing annual commitments

The [seed-17 smoke report](credit-council-smoke.md) found ten loans and ten defaults,
despite gross tax receipts arriving. Annual road spending consumed the same cash
before the next collection window. Council-specific road request/payment receipts
now enter the tax forecast's operating deduction. Prior archives without this
observation cannot supply tax evidence until a new collection. The controlled
pilot fixture rejects lending when annual roads absorb its projected receipts.

Viewer constants cleanup continued while the original two arms ran. Full shader
and remaining viewer semantic review is still outstanding; an outdated grain
storage display also remains to be corrected separately.

The corrected repeat rejected all 22 requests and produced no loans. Sites, events,
people, cargo and councils exactly matched the no-credit baseline; all ten active
market tests, both tax tests, strict Clippy and the live 30-year validation passed.
This is evidence of preventing unsupported loans, not a successful repayment or
productivity demonstration. The experiment gates remain open work.

## Commercial source identity preparation

Export evidence previously used only a mutable vector position and was removed
after inactivity. A persistent identity registry now records its buyer, actual
seller/payee, good and assignment date, separately from live procurement records.
Retired identities remain archived so an old loan cannot accidentally refer to a
later supplier that occupies the same vector slot. This is source provenance,
not another inventory or account.

New observed relationships receive identities on creation. Monthly Open assigns
missing IDs in older archives before evidence expiry; these are explicitly marked
legacy identity baselines, not invented historical contract dates. Quantities and
escrow are preserved. Delivery-paid settlement and commercial credit underwriting
are still the next integration steps; dispatch-paid behavior is unchanged.

Identity verification: all seven export-contract tests passed, including legacy
assignment, expiration, non-reused IDs, retirement rejection, checkpoint continuation
and unchanged dispatch escrow settlement. All ten active market tests passed (two
extended cases ignored); the hardware-enabled history checkpoint/export test and
strict all-target Clippy passed. These checks establish stable source references,
not delivery-paid accounting or automatic commercial borrowing.

## Delivery-paid export experiment

A funded order can capture `History.export_payment_timing = Delivery`; the default
and old archives retain dispatch payment. At dispatch, the buyer's payment moves
into a separate, persistent delivery escrow instead of seller cash. The whole
shipment follows the captured payment condition, including any quantity bought
with additional buyer cash beyond its reserved order quantity.

Arrival resolves the seller's entitlement from delivered/original mass. Lost mass
returns its share to the buyer. Full loss refunds the entire payment. Order expiry
refunds only unused procurement escrow; it cannot cancel money accompanying cargo.
Monthly Open settles resolved payments before debt servicing. Tiny amounts that
cannot transfer exactly into existing f32 accounts remain owned escrow for retry.
The existing monetary inventory includes these balances once. A town's persistent
account can receive its proceeds even after abandonment; this does not reactivate
the town or invent a new commercial operator.

The payment ledger preserves original value, delivery condition, paid proceeds,
refunds and remaining escrow. It is a foundation for commercial repayment evidence;
it does not yet issue commercial loans or treat expected proceeds as spendable cash.

Headless runs can set `--delivery-paid-exports` or
`--delivery-paid-exports=false`. Omitting the flag preserves the archived setting.
Changing it affects newly funded orders, not payment terms already captured on an
order or traveling shipment.

Verification for this increment: eight export-contract fixtures, three CLI tests,
and ten active market tests passed (two extended market cases ignored). Strict
all-target Clippy and the source-artifact check passed. The new fixture covers
full/half/zero delivery, order expiry while cargo travels, malformed payment
references, monetary/material residuals and JSON continuation without double payout.
It injects the lost quantity directly; it does not establish calibrated cargo-loss
frequency or cover every weather path.

A matched seed-17 smoke comparison used the existing 32/32-resolution founding
checkpoint, five civilizations and 30 history years on the Quadro RTX 5000 Max-Q.
Dispatch payment ended with 551.852862 people; delivery payment with 550.785824.
The latter created 108 payment records, held 5,011.40194 total buyer cash, paid
4,823.51099 to sellers and retained 187.89095 in escrow; no refunds occurred.
Its relative monetary residual was -2.01e-7. Both runs completed validation.
Single-run wall times were 10.31/9.42 seconds and are not a performance comparison.
This establishes live integration and a small negative population difference in
one world, not a credit benefit, a long-run balance result, or permission to skip
the planned multi-seed and shock experiments.

## Commercial receipt evidence

`export_credit_evidence(loss_assumption)` reads unresolved, buyer-funded delivery
payments and produces dated evidence for their actual town payees. It aggregates
payments sharing a contract and original due month, applies known remaining cargo
mass and an explicit loss haircut, and never credits cash. Dispatch-paid sales,
settled claims, abandoned payees and known-delayed/overdue shipments cannot back
new requests. A delay does not manufacture a new source date that escapes an
existing pledge. Borrower policy must still deduct operating commitments before
requesting working capital; this adapter alone does not initiate loans.

The export suite now has ten passing fixtures. New coverage checks forecasts
against funded payments, half-loss, delay, due-date expiry and completed payment;
it also funds competing requests against one receipt, verifies their combined
principal stays within the unpledged allowance, and repays through the existing
monthly servicing path after delivery. The repayment fixture uses zero interest
and sufficient borrower cash to isolate transfer/pledge accounting; it is not a
working-capital benefit or a risk-pricing calibration.

At month 360 of the seed-17 delivery smoke run, the three pending payments all
belonged to town 4: approximately 57.32, 67.69 and 62.86 due in months 361, 364 and
367. That town already held approximately 5,196.98 cash. Automatic borrowing must
therefore test an actual funding shortfall, not merely the existence of receivables.

## Automatic commercial working-cash pilot

`--commercial-credit` enables a separate opt-in policy; false stops new requests
without cancelling debt. Omission preserves the archived setting. The pilot runs
in Reserve after production planning and before enterprise funding. It quotes
missing non-food recipe inputs, subtracting current inventory and planned local
outputs, then compares that amount with actual town cash. This quote is neither
an input reservation nor a guarantee that speculative local outputs will finish.

Only a buyer already funding one of the town's traveling delivery payments can
lend in this initial commercial path. Its own input quote and a cash floor remain
protected; it offers a bounded fraction of surplus. Underwriting still applies
shared exposure and source limits and rejects simultaneous borrowing/lending.
The borrower's upcoming input commitment is deducted from repayment evidence.
One funding gap is divided across available source requests, rather than requested
in full against every invoice. Requests capture the current month and cannot be
replayed after checkpoint reload.

The rate quote accounts for the loss assumption and loan duration: expected
principal-and-interest recovery must cover the lender's required annual return.
Quotes outside the existing annual-rate bound are declined. Ordinary markets,
production and household policies spend the borrowed town cash; it is not earmarked
or automatically paid to an operator. This preserves existing ownership but means
other spending can consume the bridge before the intended procurement succeeds.
That is an explicit outcome to measure, not evidence of completed financed work.

Verification: eleven export fixtures (including automatic borrowing and negative
controls), four CLI tests and ten active market tests passed; two extended market
cases remain ignored. Strict all-target Clippy passed. A matched 30-year seed-17
run at 32/32 resolution enabled delivery payment in both arms and changed only
commercial credit. Neither arm made a loan or request. Both ended with 550.785824
people; serialized sites, events, people, cargo and delivery payments matched.
This is a null observation consistent with the measured cash-rich exporter, not
proof that the policy helps strained exporters. Broader seed and scarcity cases
remain required before advancing the monetary experiment gate.

## Bounded shared-currency issuance pilot

`--shared-issuance` creates a dated experiment authorization on first activation;
omitting the flag preserves archive settings, while false stops future creation.
The initial authorization names the currently existing councils, begins next month
and expires after 120 months. It requests 25 shared-currency units every 12 months,
with caps of 25 per issue, 100 in a rolling 12-month window and 250 over the issuer's
lifetime, plus a 12-month cooldown. These are configurable toy experiment limits,
not measured monetary parameters or recommendations for a balanced world.

Monthly Open credits the existing council treasury before ordinary obligations.
All issuers are preflighted before any treasury changes. Receipts retain the
issuer, known leader reference, currency, authorization dates through the schedule,
requested/permitted/actual amounts, prior cap usage and treasury balances. Positive
issues produce historical events. Existing payroll, relief and procurement spend
the cash; neither goods nor income are created by the act of issuance itself.

The monetary residual now compares owned balances against original money plus
recorded issuance. Debt claims remain separate. Exact representable treasury
increments count as issued money; requested but uncredited fractions do not.
Caps depend on recorded amounts and fixed limits, not prices, nominal production
or enlarged treasury balances. Re-enabling does not renew the original window or
bank missed grants. Leadership changes do not reset issuer totals, and newly
created civilizations are not silently added to this authorization.

This is shared currency: another civilization can receive the resulting cash
through ordinary transfers. Receipts identify issuance and account balances show
where cash accumulates; they do not tag the ancestry of individual coins. Distinct
currencies and foreign exchange remain subsequent work, subject to the design's
experiment gates.

Issuance verification: eleven active market tests and five CLI tests passed, with
two extended market cases ignored; strict all-target Clippy passed. The new
boundary fixture verifies per-issue/rolling annual/lifetime/cooldown limits,
repeated calls, disabled intervals, expired authorization, continuation, malformed
receipts and authorities, and atomic rejection of aggregate supply overflow.
Issued cash reconciles through the existing monetary residual. These are accounting
and scheduling checks. The [first four-arm comparison](shared-issuance-smoke.md)
found mixed issuance outcomes and no realized credit effect across two 30-year
worlds; it does not establish the gate for distinct currencies.

## Return and loss use the same horizon

Underwriting now shares commercial credit's term-aware required-rate calculation.
For a term of `t` years, required annual simple return `r`, and expected fraction
`L` lost from maturity proceeds, the offered annual rate must meet
`(r + L / t) / (1 - L)`. This makes expected proceeds cover the original principal
plus the lender's required return over that term. Complete loss cannot be financed.
The loss assumption is still a toy risk estimate, not a fitted default model.

Previously the generic resolver compared the offered annual rate with `r + L`,
which mixed an annual rate with a whole-loan loss. The analytical fixture lends
100 for three months with 10% expected loss and a 6% required annual return;
expected repayment at the minimum acceptable rate is 101.5. It rejects the former
16% quote, accepts the term-aware quote, and checks zero loss and longer terms.
Commercial proposals already used this calculation; they now call the shared
function rather than maintaining a separate formula. Existing contracts retain
their agreed rates. The four-arm executable from `0aa337c` predates this fix, so
its results must retain that revision rather than being labeled as a new-code run.

Verification for the horizon correction: 13 credit unit tests and 11 active
market integration tests passed (two extended market cases remain ignored).
Strict all-target Clippy passed. The correction does not refit the risk estimate
or establish that credit improves a full world.

## Restructuring archive integrity

Loan validation now reconstructs the original and revised maturity from the ledger,
validates original terms at disbursement, permits at most one dated extension and
requires the stored flag and final maturity to agree with that history. Extension
entries must move no principal or interest, occur after the original maturity,
retain outstanding principal and stay within the permitted revised term. This
closes an archive path that previously checked amounts while ignoring extension
semantics. Fourteen credit unit tests and eleven active market tests passed,
including corrupt flags, duplicate entries, altered dates and invalid rates.

This is a prerequisite for the negotiated policy, not its implementation. Consent,
updated repayment evidence and the next-boundary application still need a dated
proposal/decision record. The original repayment source must remain auditable;
rolling a delayed cargo into a new source ID would let it be pledged twice.

The [two-century four-arm comparison](shared-issuance-century.md) records longer
issuance consequences, rejected credit requests and remaining experiment gates.

## Consensual extension decision API

[Restructuring decisions](credit-restructuring.md) now have a dated proposal,
explicit lender/borrower consent, original-source continuity, competing-claim
coverage, accepted/declined receipts and a History commit operation. It runs after
current debt servicing and leaves all cash unchanged. The next integration step
is automatic policy and observation adapters; callers currently supply their
consent and forecast explicitly. This does not complete the restructuring/closure
checklist or enable refinancing in ordinary generated histories.

The [delayed export adapter](credit-restructuring.md#delayed-export-policy) now
constructs evidence from funded cargo and negotiates in Reserve under the opt-in
commercial policy. Council negotiation is still outstanding. The longer experiment
also found [numerical-residue defaults](credit-precision-residuals.md), which need
a distinct bounded settlement treatment before default counts are useful balance
evidence.

Monthly servicing now distinguishes bounded [precision settlements](credit-precision-residuals.md)
from insolvency. Forgiven principal/interest remain explicit debt-ledger entries;
no payment or income is fabricated. The experiment runner reports their count
and written-off amount separately from default counts. Existing recorded defaults
are not relabeled when loading older archives.

## Affordable claims blocked by transfer precision

The first residue fix exposed a still-smaller loan above the relative forgiveness
cap. Monthly service now retains an affordable but untransferable claim with a
`precision_blocked` receipt instead of treating it as insolvency. It remains due,
continues contractual interest and is retried; actual later budget/cash shortfalls
still use ordinary default rules. Forgiveness caps and monetary inventories are
unchanged. See [the precision follow-up](credit-precision-residuals.md) for the
observed loan, controls and unresolved long-run evaluation.

## Credit-to-production execution fixture

`enterprises::tests::credit_funds_gpu_work_but_cannot_replace_materials` exercises
one real GPU production month at terrain resolution 32 / ecology resolution 16.
It first moves an existing operator's capital to a council, then compares that
cash-starved workshop with a loan-funded copy. Both retain the same finite lease,
workforce and materials. A third, funded copy moves its metal stock to another
settlement before execution, preserving world material inventory.

The hardware fixture passed: credited workshop work was positive; unfunded and
missing-metal workshop work were zero. Disbursement did not increase operator
revenue, monetary residual changed by less than `1e-9` at disbursement, and the
monthly combined economy residuals remained below `0.02` in the fixture's ledger
units. A saved/reloaded funded run produced identical serialized history after
that month. This establishes an execution mediator, not merely a later population
difference.

The service-order source is explicitly supplied by the fixture. This is **not**
a test of automatic operator underwriting, order profitability or loan repayment;
those remain separate requirements. The fixture also does not establish labor
scarcity behavior, household food-access improvement, or long-run balance. It
neither changes production rates nor enables credit by default.

Reproduce on a hardware GPU:

```sh
cargo test --lib credit_funds_gpu_work_but_cannot_replace_materials -- --ignored --nocapture
```

## Retained abandoned-town treasuries

Abandonment no longer makes a town's existing debts uncollectible solely because
its operating account is inactive. Monthly Open servicing and direct repayment
can debit or credit the retained town treasury. Money remains attached to the
same site; collection does not revive its population, reoccupy it or assign its
assets to another civilization. Ordinary collection shares and protected balances
still apply. An empty treasury is not replenished by this rule.

Origination now explicitly preflights operating eligibility before transferring
cash. Both an abandoned lender and an abandoned borrower are rejected without
mutating balances or contracts; underwriting continues to use the active-account
query. Missing accounts still fail. This separates settlement access from new
credit eligibility without changing the monthly schedule or loan terms.

The CPU market suite passed (15 tests; two unrelated hardware tests excluded).
The new fixture checks both abandoned-party positions, exact repayment and money
balance, unchanged abandonment, rejected new loans, idempotent monthly service
and serialized continuation. Existing protected-cash, default, restructuring and
precision-residue controls also passed.

This is only the town-account part of closure handling. Closed operators still
require debt-aware liquidation and receivable succession; inactive institutions
need equivalent treatment of their distributed treasuries. Their account filters
are deliberately unchanged. Neither automatic legal succession nor general
estate administration is implemented by this change.


## Operator credit estates

[Closed operator accounts](operator-credit-estates.md) now keep borrowing claims
ahead of liquidation returns. Closure retains cash for live debts; proportional
early repayment uses actual existing money and the original contracts. Closed
creditors can receive later payments and return residual cash to their existing
household owner. No account reopens and no household inherits personal borrowing
liability. Plans use opening estate cash rather than spending another estate's
same-pass incoming payment according to iteration order.

Open settlement and the existing Reserve/Execute closure windows explicitly call
the estate pass. Insolvent unpaid claims retain their normal maturity and default
rules, rather than becoming fabricated repayment at closure. Inactive institution
shutdown, post-default recovery, general legal assignment and broader balance
evaluation remain unfinished. This does not enable automatic operator lending.

## Institutional shutdown and traveling accounts

[Institutional estates](institution-credit-estates.md) now share the operator
estate settlement policy. Shutdown retains cash for live claims; debt-free
residuals go to the existing home town through an exact transfer. Inactive
creditors can receive later repayments without acquiring new-credit eligibility.
Untransferable town-precision remainders remain in their original treasury.

An unfinished institutional relocation excludes its temporarily inactive account
from liquidation. The funded relocation fixture includes a live loan and verifies
that estate processing leaves the traveling institution, treasury and claims
unchanged before arrival. This preserves relocation as continuity rather than
mistaking it for dissolution. Formal legal assignments and post-default recovery
remain unfinished.

The [held-out estate-build comparison](monetary-estates-heldout.md) has completed
on seeds 256, 409 and 1024 for 200 years per arm. Credit-only produced no
realized loans; combined produced one export-backed loan in seed 409. Issuance
outcomes were mixed across seeds and metrics. The broader gate remains unmet;
this is not a reason to enable the pilots by default.

## Underwriting capacity evidence

The [capacity audit](credit-capacity-diagnostics.md) explains the held-out
submitted rejections and documents new per-grant capacity/demand snapshots.
The resolver retains its existing funding arithmetic; these records expose
simultaneous constraints and distinguish policy grants from actual cash.

## Shared commercial operating costs

[Operating-cost allocation](commercial-credit-shared-costs.md) now protects one
town-level forecast across eligible receipts instead of subtracting the whole
forecast from each. Single-source behavior and all underwriting margins remain
unchanged. This fixes duplicated costs, not the broader planned-versus-committed
expense distinction.

## Explicit default recoveries

[Default recovery](credit-default-recovery.md) records voluntary/authorized
post-default returns separately from ordinary repayments. Original loan losses
and default exclusion remain intact. The API uses original counterparties,
bounded actual cash and replay-safe requests. The opt-in
[late-export recovery policy](export-default-recovery.md) now connects newly
received matching proceeds to old defaults after live obligations and operating
reserves. Proportional [estate cash recovery](estate-default-recovery.md) now also exists; bounded operator/institution
claim succession is described below.

## Read-only explorer and reusable contract reports

The [credit inspector](credit-explorer.md) now connects existing loan, underwriting,
issuance and recovery records to the Towns and economy page. It shows original
losses and later recoveries separately. `Credit::loan_reports()` exposes all
contracts without mutating accounting; the UI bounds recent-record lists.
[Committed loan milestones](credit-chronicle.md) now connect funding, arrears,
settlement, default, restructuring and recovery in the existing chronicle.
Broader per-project causal reporting remains pending.

## Closed-estate creditor succession

Eligible closed operators now assign receivables to their existing owner household,
and inactive institutions to their home town, through the dated ownership ledger.
The estate pass includes accounts with zero cash and skips pending assignments,
in-transit institutions, missing/lost household successors, debtor-as-successor
cases and estates with borrowing claims. The beneficiary receives subsequent
payments only from the following month; original terms and past receipts remain
unchanged. The assignment records an estate basis rather than inventing consent.

Review exposed a priority mismatch: unrecovered defaults previously blocked claim
distribution but not residual cash distribution. Both now retain estate assets,
including later incoming cash, until recovery clears the loss. Voluntary gifts use
the same priority guard. A subsequent [estate recovery increment](estate-default-recovery.md)
shares opening cash across live and defaulted claims. Broader bankruptcy policies
and balance evaluation remain unfinished.

## Actual tax-base loss boundary

The [matched tax-base fixture](credit-tax-base-shock.md) now verifies a contracted
council bridge against real annual collection. Retained taxable settlements repay;
abandoning those sites after borrowing yields zero collection and eventual default.
Every tested boundary preserves the money residual, and serialized continuation
matches in each arm. The expenditure is explicit fixture setup, so useful service
completion and automatic financing still need their own causal comparisons.

## Feasible commercial request shares

The [request-allocation correction](commercial-request-allocation.md) moves the
funding-gap split after request construction checks. Unsupported-duration sources
and skipped counterparties no longer dilute an otherwise feasible request.
Underwriting still caps receipts, cash and exposure independently. A matched
funded-cargo fixture verifies the change; no ensemble benefit is inferred.

## Proposal-stage experiment diagnostics

[Recorded-demand analysis](credit-request-evaluation.md) now separates requests,
eligibility, grants and actual transfers in the monetary runner. Reanalysis of
20 saved exports distinguishes zero recorded requests from seed-1024 capacity
rejections. All its rejected requests have nonpositive archived net receipts;
older detailed capacity records are unavailable and explicitly marked as such.
This is evidence about those prior inputs, not a rerun of current behavior.

Council request construction now has [dated opportunity reviews](council-credit-reviews.md), including skipped cash gaps/evidence/contact cases and separate annual cost components. These diagnostics preserve the existing funding rules.

The [completed seed-1024 review comparison](council-credit-review-evaluation.md) covers four 200-year arms: unchanged pre-existing results, no loans, and 12,000 council-month reviews per enabled arm. Most cash gaps never reach underwriting; the report separates those construction outcomes from rejected grants.

[Local institutional offers](institution-credit-offers.md) are now an independently
switchable council-credit extension. They reuse the institution's annual operating
quote, require local leadership, and retain the existing underwriting and settlement
path. Controlled tests pass; the [six-arm seed comparison](institution-credit-evaluation.md)
completed with no loans or effects. The reserve-scaling comparison is reported
below.

[Institutional reserve policies](institution-credit-reserves.md) now distinguish
the inherited council cash floor from an annual operating-cost reserve. The old
policy remains the default. Controlled tests pass; the eight-arm reserve comparison
completed, with institutional upkeep/repair and cash metrics added to reporting.

## Institutional operating-reserve results

The [eight-arm seed-1024 comparison](institution-credit-reserve-evaluation.md)
completed for 200 years. Institutional operating reserves increased credit-only
requests from 6 to 35, but all were rejected for zero net repayment-source
capacity. No loans occurred. History outside credit records matched controls
exactly. Stage 2 remains gated; the next evidence gap is an automatic bridge
funding useful work, with realistic retained operating costs.

## Automatic bridge to administrative payroll

A [controlled council service fixture](council-credit-service-bridge.md) now traces
automatic lending through actual administrative payments and subsequent unpaid-
month/loyalty responses. It preserves repayment-source operating costs, compares
disabled credit, insolvent receipts and absent contact, and checks disbursement
replay and monetary conservation. Administrative payroll is a payment to the town;
this is not yet proof of additional named work or a positive long-run balance gate.

## Workshop service-order payment foundation

[Funded service orders](workshop-service-orders.md) now give an explicit caller
a real dated town-to-operator contract. Escrow joins the canonical monetary
inventory; completed work earns its fixed fee, ordinary invoices exclude covered
work, and unfinished/cancelled fees refund to the payer. Automatic procurement,
loan requests against these orders, and full production comparisons remain open.
