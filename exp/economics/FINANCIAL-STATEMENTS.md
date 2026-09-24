# Double-entry financial statements

Implemented reporting foundation, September 24, 2026. The complete statement set
is available for supported financial scenarios; accounting coverage of the entire
economy remains unfinished. This adds reporting alongside the existing simulation
ledger and loan book, rather than replacing their execution rules.

## Book and statements

`accounting::Book` records integer reporting ticks in one named resource denomination.
Each journal entry must balance **within each agent**. A borrower's debit cannot
balance a lender's credit across two otherwise unbalanced books. Every entry has
an ID, month, description and optional source batch ID. Publication rejects duplicate
IDs, backdating, arithmetic overflow, negative assets, debit-positive liabilities
and unclassified cash movements atomically.

The book produces:

- A debit/credit trial balance through the reporting boundary.
- A balance sheet: recognized assets, liabilities and equity.
- An income statement using movements within the selected period.
- Changes in equity: opening equity + contributions − distributions + monetary issuance + net income.
- Direct cash flows: operating, investing and financing movements, reconciled to
  opening and closing owned cash. Internal restricted-cash transfers net to zero.

Every report verifies trial-balance equality, the accounting equation, the equity
rollforward and the cash rollforward. Income/expense accounts remain cumulative
in the journal; period results use dated movements. Previous results become part
of opening equity for subsequent reporting periods without fabricated closing
transactions. Negative net worth is allowed.

`Book::open` explicitly recognizes opening assets/liabilities and their net equity.
`open_at` dates a snapshot and rejects reports reaching behind that opening. Opening
equity is not income and cannot subsequently be posted as a balancing adjustment.
The generic chart also supports explicitly costed inventory, cost of sales,
depreciation and capital movements. Spot inventory exchanges now have a validated
adapter as described below. Manual postings require their own domain
validation; the journal cannot determine whether an economically plausible entry
really occurred.

## Simulation adapter and boundaries

`financial_reporting::Audit` is the strict adapter. Open it at a month's Open
boundary. Its `step` executes a cloned simulation, verifies the committed batch,
prepares the accounting entry and reconciles recognized closing positions against
the authoritative state. It publishes both only after validation. Accounting failure
therefore leaves both simulation and journal unchanged. Existing Open/Due/Acquire
and other monthly execution phases stay in place; no reservations or payments move.

`record` supports an external observer receiving exact before/batch/after snapshots.
It checks the saved opening boundary and independently replays commit validation.
A replay, skipped boundary, edited state, unexplained cash movement or unsupported
financial source fails. Entries reference committed batch IDs; retain the simulation
ledger for individual events and receipts behind each aggregated boundary entry.

`book().statements(agent, from, through)` reports the journal **as recorded**. The
caller should complete Close for `through` before labelling it a completed monthly
report. A cloned Audit and Simulation preserve continuation history. Opening a new
Audit at a later month recognizes a new opening snapshot, not reconstructed history.
These remain in-memory checkpoints. The standalone journal now has validated JSON
persistence and finalization (below); restoring the complete Audit and Simulation
from disk remains outstanding.

## Recognition rules and supported arrangements

| Arrangement | Recognition and cash treatment |
| --- | --- |
| Cash loan advance | Lender exchanges cash for principal receivable; borrower records cash and principal payable. No income. Lender investing outflow, borrower financing inflow. |
| Monthly accrual | Lender interest receivable/income and borrower interest payable/expense. No cash flow until payment. |
| Installment | Principal reduces both parties' claims; only interest affects income through its earlier accrual. Interest cash is operating; principal is lender investing/borrower financing. |
| Configured endowment | Explicit capital contribution and financing inflow. This fixture initialization is not a monetary-authority issuance accounting policy. |
| Configured cash transfer | Recipient transfer income, payer transfer expense, operating cash. Different economic purposes require a different adapter. |
| Financed asset purchase | Buyer records the full purchase basis and loan payable. Seller derecognizes carrying cost and records disposal gain/loss. Only actual cash enters cash flows; funds paid directly to the seller are not fictional borrower cash receipts. Seller and lender may differ. |
| Fixed-value repossession | Asset transfers at the contractual value, reducing principal/interest; debtor recognizes disposal gain/loss against carrying cost. Debt extinguishment is noncash. Actual funded surplus alone moves cash. Remaining deficiency remains payable. |
| Repossession awaiting resale | Borrower retains the economic asset while the creditor holds title for realization. No second lender asset or premature profit. Actual sale updates the buyer's basis, pays creditor/surplus and recognizes the debtor's disposal result. |
| Spot stock exchange | Seller records actual proceeds as revenue and releases opening carrying cost as cost of sales. Buyer capitalizes actual payment into inventory. Both cash legs are operating. Posted stock bids and bilateral negotiation/ZIP use the same cost subledger. |
| Called guarantee | Creditor receives actual principal/interest cash; guarantor exchanges cash for the recourse receivable. Borrower substitutes creditor, rather than receiving debt-relief income. |
| Estate sweep | Debtor cash becomes restricted cash. Estate recognizes equal custody cash and custody payable; custody is not estate wealth or income. |
| Estate asset sale | Buyer records acquired asset; debtor derecognizes it, recognizes gain/loss and restricted sale proceeds; estate records matching custody positions. |
| Estate distribution | Restricted cash settles loan principal/interest or eligible native-coin/accepted-coin dues. Custodian equity stays unchanged. |
| Explicit loan discharge | Creditor records credit loss; debtor records debt-relief income. No cash movement. |

Tangible assets require an explicit opening valuation for **every** asset through
`Audit::with_assets`. Accepted purchases, contractual fixed-value enforcement and
actual resales establish subsequent carrying values. Market quotes alone do not
revalue holdings. Pledged land is counted once, on the economic owner's books.
This is a model-specific cost/contract-value convention, not a claim of compliance
with a real-world accounting standard.

The adapter recognizes the selected coin at one reporting tick per stored tick.
It rejects nonzero stocks without explicit opening cost, unsupported stock movements,
mixed loan or estate denominations, unvalued equipment barter, unconfigured royalties and unconfigured land dues. Town-market trades use the costed stock adapter. Minting/issuance requires its explicit policy below. Process transactions
require the explicit material-cost opt-in described below. Unsupported activity is an error,
not zero value or an unexplained income/equity adjustment. Unused capacities and
need satisfaction are not financial assets. Uncalled guarantees remain contingent
agreements in the contract inspection view; they are not recognized liabilities
or a complete accounting disclosure schedule here.

## Costed inventory and spot exchange

`Audit::with_inventory` accepts **total opening carrying cost** per `(agent, resource)`
plus the existing tangible-asset valuations. Explicit zero cost is valid; absent
cost for a nonzero stock is an error. An `inventory_accounting::Inventory` subledger
tracks quantities and reporting-tick costs separately from simulation resource
quantities. Every committed reporting boundary reconciles both quantities and
monetary positions. Its candidate changes publish atomically with the journal.

The costing policy is **opening-boundary weighted average**, not current market
price or agent willingness to pay. For opening quantity Q and cost C, total sales
of q release `floor(C * q / Q)` ticks. All sales from the same holding in the same
boundary are pooled before rounding; remaining rounding ticks stay in inventory
until it is depleted. Purchases add their actual paid cost after opening-stock
sales have been costed. Incoming stock cannot fund a same-boundary sale, matching
the settlement reservation rule. Transaction order and splitting identical total
sales into smaller lots therefore do not change total cost of sales.

Only verified one-good/one-coin spot transactions are admitted: existing posted
stock bids and bilateral negotiated trades, including ZIP quotes. A quote or a
failed/unfunded match creates no revenue. The adapter does not alter prices,
allocation, permissions, storage checks or the monthly schedule. It does not infer
an exchange merely from coincident stock/cash changes.

The controlled CPU example sells two grain units carried at 14 ticks for 40 ticks:
the seller recognizes 40 revenue, 14 cost of sales and 26 profit; the buyer records
40 inventory and zero profit. A six-month ZIP control checks realized revenue
against actual proceeds while preserving cost independently of quote changes.
Split-lot and reversed-order controls cover rounding, full depletion and simultaneous
sales/purchases; checkpoint, forged-batch and missing-cost controls preserve atomicity.

Run the one-month CPU report:

```sh
cargo +1.92.0 run --locked --example inventory_statements > ../../output/economics/inventory-statements.md
```

Inventory extension validation: **41 distinct tests passed** across `accounting`
(11), `inventory_accounting` (6), `finance` (3), `negotiation` (7), `storage_currency`
(7) and `zip` (7). The first focused run contained five inventory tests; the final
six-test inventory run added the ZIP accounting control. CPU export, formatting,
strict all-target Clippy and repository artifact checks passed. No full-crate
rerun was performed.

Town-market trades share this stock-cost adapter. Posted commodity-for-commodity
bids now use explicitly valued payment goods, as described below. Expiration,
dues and production/consumption have their own recognition rules while sharing
opening-stock cost allocation. An inventory cost alone does not authorize an unknown stock movement.

## Production and consumption costs

`Audit::with_processes` opts into material-cost accounting with the same opening
inventory costs and tangible valuations. It additionally accepts explicit relative
cost shares per output resource for definitions producing joint products. Provided
shares must be positive and cover the definition's stock outputs. A joint-output
completion without shares rejects atomically; no grain/seed values are guessed.

| Event | Monetary recognition |
| --- | --- |
| Input committed to productive work | Reduce input inventory at opening carrying cost; capitalize into `WorkInProgress(process_id)`. |
| Work continues | Retain accumulated input cost across months; add any further consumed stock cost. No automatic income or depreciation. |
| Process completes | Move accumulated work cost into actual stock outputs. Single output receives all cost; joint products receive configured shares. |
| Consumption completes | Reduce consumed inventory and recognize `ConsumptionExpense`. Nutrition/warmth satisfaction is nonfinancial, not inventory or revenue. |
| Process aborts | Derecognize its work-in-progress cost into `ProductionLoss`. Do not expense the seed again or pretend the lost harvest was produced. |
| Productive work with no financial output | Recognize material cost as `ProductionExpense` instead of inventing an asset. |

Unpaid labor, regenerated capacities and environmental services have no monetary
cost in this policy. Consequently labor-only output has a known zero material
basis, not a guessed market price. Equipment wear is included as described below;
paid labor and overhead capitalization are not covered. These are recognition rules, not planner valuations.

All input releases use the opening-boundary average cost. When several processes
consume the same stock, cumulative rounding allocates released cost by stable
process ID. Output shares use cumulative rounding in resource-ID order. These
conventions allocate every tick and do not change which process receives physical
resources. New output is not available to other work in the same batch; inventory
cost becomes visible with the existing committed Productive boundary and is then
available to the later Consumption boundary.

The default process policy requires the operator and output beneficiary to agree.
Explicit policies below support shared-pool inputs, earned-only royalties, and
completed output transfers to a distinct beneficiary at cost. Validated
title-following attachment transfers move WIP carrying cost between operators.
Non-pool third-party inputs and priced contract production still need adapters;
combined trade/production batches remain subject to the existing scheduler. Cloning the audit preserves work
costs for continuation. A new book over active work requires explicit complete
historical carrying costs through Opening; ordinary constructors still reject it.

Storage currently constrains acceptance/completion; it does not emit spoilage or
stored-goods discard transactions. A blocked harvest aborts under existing process
rules, so only capitalized work is lost. No new spoilage rate, storage-loss event
or monthly execution stage was introduced. A real stored-goods loss adapter needs
an explicit validated event before it can release inventory cost into an expense.

CPU examples (from `exp/economics`):

```sh
cargo +1.92.0 run --locked --example process_statements -- harvest > ../../output/economics/process-harvest.md
cargo +1.92.0 run --locked --example process_statements -- missed-work > ../../output/economics/process-missed-work.md
cargo +1.92.0 run --locked --example process_statements -- storage-blocked > ../../output/economics/process-storage-blocked.md
```

These controls start with grain cost 10 and seed cost 12, explicitly value land
at zero, and run six months. Successful harvest leaves 11 inventory and 11
consumption expense. Both failure controls finish with 12 production loss and 10
consumption expense. Cash and revenue remain zero. An additional grain/seed
control allocates 12 material-cost ticks as 9 grain / 3 seed; the 18-month repeated
harvest test reconciles remaining assets plus cumulative expenses to the original
22 ticks.

Validation: **38 tests passed** across `accounting` (11), `inventory_accounting`
(6), `process_accounting` (5), `process_offers` (6) and `repeated_and_warmth` (10).
Controls include CPU/reference agreement, in-progress checkpoint continuation,
missing joint-product shares with atomic rejection, missed work, storage blockage,
and preventing a fresh book from forgetting active work costs. All three CPU
exports, formatting, strict all-target Clippy and artifact checks passed. This was
a focused run, not a full-crate rerun.

## Dated land dues and alternative tender

`Audit::with_dues` adds the ordinary land-agreement adapter. Supply fixed reporting
ticks per **native payment unit**, keyed by agreement ID; native reporting-coin
dues always use one tick. Non-coin dues require an explicit positive valuation.
The claim valuation remains fixed across periods: price quotes and the debtor's
inventory cost do not silently revalue arrears.

When the existing Due resolver first creates a bill, the debtor recognizes
`DuesExpense` and `DuesPayable(agreement, due_month)`; the creditor recognizes
`DuesIncome` and the matching receivable. This policy recognizes the bill at its
due boundary, **not monthly accrual of the preceding year's use**. Earlier future
commitments remain contract information. Existing arrears in an opening snapshot
become opening claims/equity, not fresh period expense. Partial payment leaves
symmetric residual claims; subsequent months do not charge them again.

Native payment in goods releases the debtor's opening inventory cost, settles the
claim at its fixed value, and records the difference as disposal gain/loss. The
creditor receives inventory at the extinguished claim value. Accepted coin tender
records only actual operating cash; a difference from claim value becomes opposite
`SettlementGain`/`SettlementLoss` entries for the parties. No extra revenue is
recognized merely for receiving payment on a previously recognized claim.

For example, two grain units owed at three reporting ticks each create a six-tick
bill. Paying one grain carried at two ticks produces a one-tick disposal gain.
Paying the other unit with the accepted two-coin tender produces a one-tick debtor
settlement gain and creditor settlement loss. The debtor's result is −4; the
creditor's is +5, with three-tick grain inventory and two actual coins received.
These figures depend on the supplied valuation convention, not inferred fair value.

The adapter reconciles authoritative changes in owed/paid/native-paid quantities
against the exact validated commitment transactions, without parsing cause strings.
Native input costs use the opening average, with stable agreement/due ordering
for rounding across bills. Incoming grain cannot finance another bill in that
boundary. Loan and dues receipts can share a report without counting loan principal
as income. `with_process_policy(world, shares)` composes production costing with a
newly opened dues book; it must be selected before recording the first batch.

Limits remain explicit: arbitrary dues relief and combined same-boundary
trade/production/dues cost
allocation require further adapters. Ordinary Due and Productive/Consumption work
compose on their existing separate boundaries. Prepaid-forward tool purchases,
physical delivery and accepted relief now have the recognition adapter below.

```sh
cargo +1.92.0 run --locked --example dues_statements > ../../output/economics/dues-statements.md
```

Validation: **57 distinct tests passed** across `accounting` (11), `agreements` (3),
`dues_accounting` (6), `inventory_accounting` (6), `process_accounting` (5) and
`recovery` (26). The initial focused run had five dues tests; the final six-test
run added combined loan/dues reporting. Controls cover CPU/reference equality,
partial arrears, the next annual bill, checkpoint continuation, opening arrears,
native cash, grain plus alternative coins, and atomic rejection of missing
valuation or unaccounted issuance. A 13-month CPU farming/consumption/dues control
also passes. The CPU export, formatting, strict all-target Clippy and artifact
checks pass; no full-crate rerun was performed.

## Run and inspect

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --example financial_statements -- default > ../../output/economics/financial-default.md
cargo +1.92.0 run --locked --example financial_statements -- repaid > ../../output/economics/financial-repaid.md
```

The CPU example exports all five reports for borrower and state, plus the journal
showing cash classifications and noncash account movements. It uses five months,
10,000 opening land ticks, the existing 2,000 downpayment, 8,000 financed principal
and 1% monthly loan interest. `surplus` and `downpayment` exercise the existing
surplus-repossession and rejected-purchase controls. Generated exports stay under
ignored `output/`.

## Validation and remaining work

Focused tests cover direct lending CPU/reference equality, principal versus income,
interest cash classification, guarantees/recourse, cash and asset estates, write-off,
all four mortgage controls, third-party lending, resale/no-buyer controls, explicitly
costed inventory sale, capital distributions, period reporting and cloned checkpoint
continuation. Invalid entity cross-netting, missing valuation, unjournaled changes,
unclassified cash, duplicate entries, overflow and invalid opening periods reject.

Initial financial adapter validation: **79 tests passed** across `accounting` (11), `credit` (9),
`finance` (3), `lending` (18), `loan_views` (5), `recovery` (26) and `resale` (7).
Formatting, strict all-target Clippy, diff whitespace and repository artifact checks
passed. This is a focused regression run, not a full-crate rerun.

Both five-month CPU exports completed. In the default control the borrower has
2,160 remaining debt, 160 interest expense, 4,000 disposal loss and −2,160 closing
equity; repossession did not create cash. In the repayment control the borrower
has 200 cash, 10,000 land, no debt, 200 interest expense and 10,200 equity after
8,400 configured transfer income and the 2,000 opening endowment. These are
fixture outcomes, not evidence of sustainable autonomous income.

Extend adapters next, preserving these reconciliation gates:

1. Extend material costing to capitalized paid labor and third-party production.
   Historical WIP opening, title-following WIP transfers, configured expiration,
   town-market trades and explicitly valued equipment barter are implemented.
   Posted barter and shared opening-stock cost allocation are also implemented.
   Royalty consideration and other exchange forms still need explicit adapters.
2. Extend the explicit non-redeemable issuance convention to redeemable issuer liabilities,
   retirement/burning and broader monetary instruments; do not infer promises from tokens.
3. Extend ordinary and estate-paid dues to explicit relief.
   Prepaid-forward origination, delivery, extensions and write-offs now have an
   adapter; broader repricing/refunding and impairment policies remain open.
4. Add household/institution contributions and
   distributions, then ownership and consolidation eliminations. Summing entity
   statements is not a consolidated economy statement: custody and intra-economy
   claims need elimination.
5. Add valuation/FX policies, impairment allowances, contingent-claim and noncash
   financing disclosure schedules and full Audit/Simulation persistence. Journal-only
   persistence and completed-period finalization are implemented below.

The adapter deliberately replays validation and clones small scenarios. It has not
been optimized or benchmarked for a large population. Balanced books establish
accounting consistency, not economic calibration, solvency or recoverability.

## Coin equipment acquisition and use-based cost

Every opening durable asset now requires an explicit carrying cost in the same
asset-value map as land. Zero cost is allowed explicitly; an exhausted tool must
have zero carrying cost. An accepted posted equipment offer denominated in the
reporting coin transfers the asset at its actual purchase price. The seller removes
its previous basis and recognizes disposal gain/loss; both sides classify actual
cash as investing. Asking prices alone do not revalue unsold equipment.

At the existing Productive boundary, validated technique use releases
`floor(opening carrying cost × uses spent / opening remaining uses)`.
Rounding stays in the tool; its final use releases all remaining cost. Idle tools
do not depreciate under this policy. This is a remaining-use cost convention, not
a calendar-life estimate.

Wear is a production input cost: it joins seed/material cost in work in progress,
passes into actual outputs at completion, or becomes production loss if work
subsequently fails. It is not also charged as a separate depreciation expense.
The process-cost opt-in remains required. No planner, reservation or monthly
execution timing changes were made.

Verification: `equipment_accounting` compares CPU/reference purchase and harvest,
then follows six uses through month 60 with checkpoint continuation. A price of
3 against seller basis 2 records gain 1 and buyer cost 3; full exhaustion releases
all three ticks. Combined entity assets less liabilities and cumulative income
retain opening equity at every boundary. Missing opening costs and missing barter
valuations are rejected; failed accounting publishes neither simulation nor book.

This covers posted coin purchases and owner-operated wear; subsequent extensions
add explicitly valued equipment barter and title-following WIP transfers. Royalty
tool delivery still requires an adapter. A financial statement is not yet available
for every tool scenario.

The focused equipment-accounting regression run passed **38 tests**: accounting
11, dues 6, equipment simulation 8, equipment accounting 2, inventory 6 and process
accounting 5. Strict all-target Clippy and formatting passed. The legacy equipment
tests validate simulation behavior; they do not imply financial-report support
for barter or every legacy arrangement.

## Prepaid forwards, delivery and accepted relief

The existing bundled tool-purchase flow now joins the same reporting book.
Recognition uses the original coin advance, not the projected harvest's market
value. This is a prepaid goods contract, not an interest-bearing cash loan.

| Committed event | Recognition |
| --- | --- |
| Advance pays tool provider directly | Creditor debits `ForwardPrepayment(contract)` and credits actual cash; producer credits `DeferredRevenue(contract)` and capitalizes the full purchased tool. Any producer contribution alone reduces its cash. Provider removes equipment basis and recognizes disposal gain/loss. |
| Physical commodity delivery | Producer releases deferred revenue into sales and expenses inventory carrying cost. Creditor moves the released prepayment into received inventory. No new cash movement. |
| Missed/partial delivery | Keep the remaining prepaid asset and delivery liability; neither a missed deadline nor a revised forecast invents income, loss or delivered goods. |
| Accepted maturity extension | Preserve carrying amounts. Only the contractual date changes. |
| Accepted quantity write-off | Creditor recognizes `CreditLoss`; producer recognizes `DebtRelief`. Reduce both carrying amounts without changing actual-delivery or inventory counters. |

Creditor prepayment cash is operating (a future goods purchase); provider equipment
sale proceeds and the producer's own equipment payment are investing. The financed
equipment portion is noncash for the producer. No fictitious producer cash inflow
or financing cash outflow is inserted to balance the statements. A comprehensive
noncash-disclosure schedule remains future work.

Released value is cumulative:
`floor(original advance × (delivered + written-off units) / contracted units)`.
Each action receives the increase from its prior settled quantity; rounding stays
with the residual claim, and final settlement releases every reporting tick.
Physical deliveries and relief retain separate native counters. Spot quotes and
unaccepted projections never revalue the prepayment.

For example, an advance of 2 coins against 4 grain retains value 1 after delivery
of 3 grain. The producer recognizes sales 1, plus cost of those 3 grain; the creditor
holds received inventory 1 and residual prepayment 1. Forgiving the final grain
then produces loss 1 and debt relief 1, with actual delivery still 3. Delivery of
only the first grain can release zero reporting ticks because of integer precision;
its physical quantity and inventory cost are still accounted for.

Forward deliveries and spot sales in the same Acquire batch share opening
inventory-cost allocation. All outgoing quantities are pooled before rounding,
and incoming goods cannot finance another same-boundary sale. Process production
and dues still require their separate existing boundaries for cost allocation.

Verification in `tests/forward_accounting.rs` covers actual CPU tool purchases
with full prepayment, part own cash and entirely own cash; CPU/reference partial
and complete delivery; checkpoint continuation; combined spot and forward delivery;
accepted extension then residual write-off through an estate; and forged delivery
rejection without publication. Opening accepted forwards reconstruct their remaining
historical advance cost from validated terms and performance history.

Limits: no fair-value marks, discount accretion, expected-loss allowance, cash
refund/repricing adapter. Royalty deals now have the separate earned-only policy below;
this does not establish reporting coverage for every unrestricted specialist scenario. Supporting forward reports does not remove simulation-driver
composition restrictions or establish recoverability of unpaid goods.

The forward-accounting regression run passed **68 tests** across accounting (11),
dues accounting (6), equipment accounting (2), forward simulation (7), forward
accounting (5), inventory accounting (6), process accounting (5) and recovery (26).
Strict all-target Clippy and formatting passed. Simulation regression coverage
does not imply financial-adapter support for every configuration in those suites.


## Physical minting and collection-linked issuance

Reporting now offers an explicit initial currency convention:

```rust
audit.with_issuance_policy(
    issuance_accounting::Policy::NonRedeemableEquity,
)?
```

Select it at book opening. Without it, minting and collection-linked issuance
remain rejected. This is a model convention for coins with no redemption promise:
authorized newly created face value credits `MonetaryIssuance` equity and debits
issuer cash. It is neither sales income nor a loan from an invented counterparty.
It does not claim compliance with sovereign or central-bank accounting standards.

Statements expose `issuance_change` separately from `capital_change`. The cash
reconciliation shows `Flow::Issuance` separately from operating, investing and
financing flows: self-created currency is not an external cash receipt. Existing
coin holdings at reporting opening stay in opening equity; past issuance is not
recognized again.

Physical minting preserves the existing timing:

1. Open regenerates period capacities without creating a financial asset.
2. Acquire validates funded market packages. Stock purchases/sales use the common
   inventory-cost adapter. A paid capacity transfer recognizes seller
   `ServiceIncome`, buyer `ServiceExpense` and operating cash transfers.
3. Productive consumes actual mint materials. With the policy enabled, the
   reporting projection treats authorized coin output as issuance instead of
   inventory; material cost becomes `ProductionExpense`. The original committed
   process and ledger remain unchanged.
4. Close statements reconcile owned cash, inventories, results and issuer equity.

Purchased monthly capacity is a delivered period service and is expensed when
made available. This initial policy does **not** capitalize wages into work in
progress or promise that purchased hours result in completed output. Unused paid
hours therefore remain an expense; unspent materials remain inventory. Unpaid
own labor remains unpriced. General paid labor capitalization, future service
contracts and refunds require additional policies.

In the normal two-month CPU fixture, the issuer sells wheat for 6 (opening cost
6), pays 2 for metal and 4 for labor, and creates 10 coins. Its net income is
**−6**, monetary issuance is **+10**, and closing cash/equity is **10** against
opening equity 6. The worker earns service income 4. Coin creation is visible
without disguising the cost of issuing it as profit.

Collection-linked issuance uses the same equity convention. Authorization comes
from the increase in `floor(in_kind_paid / collected_per_token)` for each dated
obligation. The reporting adapter verifies and separates the actual one-sided
coin-creation effects from ordinary dues transfers, without parsing cause text.
Native goods and dues income retain their existing accounting; token creation
does not add a second dues income entry. Accepted coin payment of dues never
counts as native-goods collection. Unchanged receipts cannot issue again.

Verification covers CPU/reference and checkpoint consistency; normal minting;
treasury, metal and labor shortfall packages; paid but unused hours; repeated
minting with finite ore, missed labor and low yields; rejection without an explicit
convention; forged output rejection; and native versus alternate-tender dues
issuance. The CPU export is reproducible with:

```sh
cargo +1.92.0 run --locked --example minting_statements > ../../output/economics/minting-statements.md
```

Limits: no redeemable currency liability, issuer reserve requirement, coin
retirement, FX or consolidation treatment. The physical minting driver remains
isolated from collection-linked issuance and other acquisition drivers in the
simulation. Reporting each separately does not remove those composition limits.
The provisioning/leisure variants now reconcile configured service-ticket expiration
through InventoryLoss. Estate-paid native/accepted-coin
dues now have the adapter below; arbitrary non-loan discharge remains unsupported.

The issuance-accounting regression run passed **72 tests across 11 suites**:
accounting 11, dues accounting 7, equipment accounting 2, forward accounting 5,
inventory accounting 6, issuance accounting 5, mint cycles 6, mint orders 10,
minting 8, process accounting 5, and storage/currency 7. Strict all-target Clippy,
formatting and the CPU statement export passed. Legacy simulation regressions
do not establish accounting support for every scenario they cover.


## Estate-paid land dues

Verified `LandDistributed` receipts now compose with the existing dues and estate
accounting. The receipt's actual tender, not requested or allocated amounts,
determines the payment. The adapter cross-checks those receipts against physical
estate-to-creditor transfers, projects the debtor as the economic payer for dues
recognition, and classifies its payment against `RestrictedCash(proceeding)`.
The authoritative transaction still names the actual custodian.

The debtor reduces the existing dated payable; the creditor reduces the matching
receivable. Native-coin dues extinguish at face value. Accepted alternative coin
payments retain the configured native-unit reporting value and recognize any
settlement gain/loss against actual tender. Paying an opening arrear creates no
new dues expense or income.

The estate's custody cash and matching custody payable decline together. Neither
custody nor payout produces custodian revenue, equity or owned-cash flows. Earlier
cash sweeps retain their internal classification; dues payouts are operating,
while loan-principal payouts retain their financing/investing classifications.
Partial payments leave the unpaid dues on both parties' books and keep the
existing closure restrictions. Goods dues without accepted coin tender are not
converted into a cash claim or fictional goods delivery.

A controlled proportional case opens with 12 coins against two loans of 10 each
and 2 grain of dues. The estate pays 5 to each loan creditor and 2 coins for 1 grain
of dues. At a reporting value of 3 per grain, debtor/creditor recognize settlement
gain/loss 1; the remaining dues balance is 3. Custodian equity remains zero and
the unresolved claim keeps the estate active.

With land dues ranked first against the same 12 coins, 4 coins settle both grain
units and 8 repay loans. The configured discharge writes off the remaining 12
loan principal and the estate closes. Dues settlement gain 2 is separate from
loan debt relief 12. Paying in coins does not count as native grain collection and
does not trigger collection-linked issuance.

Verification covers CPU/reference equality, continuation from an open-estate
checkpoint, partial and ranked full payment, native coin dues, no-alternative
controls, closure plus loan discharge, and forged-receipt rejection without
publication. The focused run passed **58 tests** across accounting (11), dues
accounting (7), estate dues accounting (4), forward accounting (5), issuance
accounting (5), and recovery (26), plus strict all-target Clippy and formatting.

This changes reporting only: no scheduler, admission, priority, whole-unit
allocation, storage, issuance or legal-discharge rule changed. Direct goods
liquidation into land claims, new non-loan relief terms, mixed denominations and
general death/dissolution estates still need explicit domain rules and adapters.

## Equipment manufacture, repair and decay

The opt-in process adapter now capitalizes owner-operated durable manufacture.
Consumed material cost and productive equipment wear remain in WIP while work
is active, then transfer into the completed durable's historical basis. Aborted
work remains production loss. No imputed labor income or market-price uplift is
recognized. Execution and reporting share the produced asset identity function.

Repair restores physical service capacity but leaves existing carrying cost
unchanged. Repair materials and helper-tool wear are `ProductionExpense`; later
wear allocates the remaining basis over the restored remaining uses. This is an
expense policy, not capitalized improvements or replacement-component accounting.

Required equipment bindings now release carrying cost into the process alongside
optional techniques. Reporting reconstructs the actual sequential binding boundary.
Configured monthly idle decay releases basis to `Depreciation` at Open; productive
wear goes into output/WIP rather than also being expensed as depreciation. Integer
rounding retains fractional costs until later uses and releases all cost on
exhaustion. The house catalog still has zero idle decay; the decay test explicitly
configures one use per month. No simulation timing or catalog balance was changed.

`manufacture_accounting` checks CPU/reference two-month construction and checkpoint
continuation (eight material-cost ticks become house basis eight), full configured
decay, helper wear expensed on repair without revaluation, required-tool wear in
manufacture, missing-material nonproduction, and atomic rejection of mixed stock
and durable outputs without an allocation policy. The typed policy described below
now supports such outputs; stock cost is never silently assumed to be zero. General third-party work, capitalized paid labor,
royalties and unrestricted specialist reports remain
outside this increment.

Validation: **42 tests passed** across accounting (11), activities (10), equipment
accounting (2), forward accounting (5), issuance accounting (5), manufacture
accounting (4), and process accounting (5). Strict all-target Clippy, formatting,
diff checks and the repository artifact policy passed. Generated test logs remain
under ignored `output/economics/`.


## Journal persistence and completed periods

The version-1 JSON journal archive stores denomination, opening month, entries and
an optional finalized-through month. Use `book.to_json()` and
`Book::from_json(&text)`; callers own file I/O and should place generated archives
under ignored `output/`. Loading validates the schema version and opening entry,
then replays ordinary posting checks to reconstruct balances and duplicate-entry
protection. Cached balances are not persisted or trusted. Integer amounts retain
full i128 precision in Rust; consumers must not parse them as floating-point values.
Archives validate accounting consistency, not authenticity or agreement with a
simulation ledger. They are not an alternative way to attach an arbitrary book to
an Audit.

`Audit::finalize_through(month)` rejects the current/incomplete simulation month.
It locks postings through a completed month without creating synthetic closing
entries or resetting income accounts. `book.finalized_statements(agent, from,
through)` requires the requested period to be locked. The existing `statements`
method remains available for provisional inspection. Later valid entries do not
change earlier finalized reports. Locks may advance or repeat but cannot retreat;
there is no reopening API. Corrections must be explicit later-period entries.
Standalone `Book::finalize_through` can verify journal bounds only: its caller is
responsible for knowing that all events for the month have been posted.

This is journal persistence, **not a durable simulation restart**. Full restart must
also preserve authoritative World/State, pending work, ledger provenance, reporting
inventory and equipment basis, WIP, and configured dues/issuance policies. In-memory
Audit/Simulation clones retain these today. Historical WIP import is now supported through explicit Opening costs; it does not restore historical income.

Validation: all 14 accounting tests pass, including amounts above u64, round-trip
balance and lock equality, malformed/unbalanced/duplicate/version rejection,
atomic failed finalization, frozen report stability and CPU continuation after
finalization. The archive is human-readable but is generated output, not a source
artifact to commit.

The broader focused run passed **34 tests**: accounting 14, manufacture accounting
4, forward accounting 5, dues accounting 7 and estate dues accounting 4. Strict
all-target Clippy, formatting, diff and repository artifact checks passed.

## Joint stock and durable output shares

`Audit::with_output_cost_policy(world, weights)` selects a complete process-cost
policy at reporting opening. Keys are typed `process_accounting::Output::Stock`
(resource ID) or `Output::Durable` (durable kind ID), so equal numeric IDs do not
conflate a resource with equipment. The existing stock-only constructor remains
available and translates its resource shares into the same allocation engine.
Calling the new builder replaces the complete output-share map, not just one
process's shares; include all joint processes that require a policy.

Every joint product must have a positive share, with no extra outputs or missing
kinds. Shares divide the total historical material and productive-wear cost of the
completed process, including accumulated WIP. They apply to each output lot, not
each unit of stock, and are not sale prices. A single output receives all cost
without configuration. Configured shares must still match actual outputs at
completion; a changed catalog cannot silently redirect costs to a surviving output.

Allocation uses cumulative integer rounding in a fixed order: stock resource IDs,
then durable kind IDs. Thus all carrying cost is assigned exactly once, including
small amounts that give an individual product zero carrying cost. An eight-tick
construction with equal grain, fuel and house shares assigns 2/3/3. Stock portions
join inventory cost and the house portion becomes tangible asset basis. There is
no production profit or duplicated expense at completion. Active work still keeps
its cost in WIP; aborted work still expenses that cost as production loss.

The new tests compare CPU/reference and checkpoint continuation through joint
construction, verify exact WIP release and no income creation, reject missing,
extra, zero and wrong-kind shares, reject policy changes after opening, and verify
atomic failure if actual outputs differ (including removal of the stock output).
Existing unconfigured joint-output rejection remains covered. This only covers the
current one-durable-per-process execution model, with owner-operated costs;
cross-agent production and capitalized paid labor remain outstanding.

Validation: **33 tests passed** across accounting (14), equipment accounting (2),
issuance accounting (5), manufacture accounting (7) and process accounting (5).
Strict all-target Clippy, formatting, diff and repository artifact checks passed.
Logs remain under ignored `output/economics/`.


## Single accounting implementation

The legacy `credit::BalanceSheet` and `credit::balance_sheet` snapshot calculation
have been removed. Financial statements now come only from `accounting::Book`
and the validated `financial_reporting::Audit` adapters. The credit example opens
explicit plot cost, records all boundaries and finalizes its reporting period;
it no longer values assets using a live offer-price fallback.

The credit contract book, physical account balances, transaction effects, forecasts
and operational telemetry remain: these drive or inspect execution and are not
alternative financial statements. Credit-stress output now includes finalized journal
reports alongside cash, debt and title diagnostics. Unsupported events still reject;
no silent snapshot fallback is retained.

Simulation tests retain contract, cash, collateral-title and custody assertions.
Financial regression tests use journal balances and independently specified
expected equity, including repaid, default, surplus and rejected-downpayment
mortgages. Existing journal tests cover restricted borrower assets pending resale
and neutral estate custody. The old calculation is no longer a financial test oracle.

Removal validation: **84 focused tests passed** across accounting (14), credit (9),
credit stress (5), lending (18), loan views (5), recovery (26) and resale (7).
The final accounting assertions also passed after adding explicit journal checks
that pending-sale collateral is excluded from lender assets while both sides retain
their principal/interest positions. Strict all-target Clippy and the CPU credit
example passed. Source search finds no remaining legacy balance-sheet API or calls.
Generated summaries and logs stay under ignored `output/economics/`.

## Coverage expansion and remaining adapters

The next coverage pass closes these concrete gaps:

| Situation | Accounting treatment and evidence |
| --- | --- |
| Town-market clearing | Validated market transactions join the existing stock cost pool with spot and forward deliveries. CPU/reference and three-month continuation agree. |
| Repossession/resale of an attached crop | Validated attachment receipts move historical WIP basis to the new operator. Noncash TransferExpense/TransferIncome balance the owners; collateral valuation and loan recovery do not change. Maintained crops release cost into harvest, neglected crops recognize ProductionLoss. |
| Historical active work at reporting opening | `Audit::with_opening` accepts an `Opening` containing a Costs subledger with every active process, including zero-cost work. Missing, extra, negative or mismatched ownership fails. This creates an opening snapshot, not reconstructed prior income. Setting output policies preserves imported WIP. |
| Configured expiring stock | At Open, domain-authorized expiration removes the complete holding and expenses its remaining basis once as InventoryLoss. This adds no new physical spoilage rule. Four provisioning variants reconcile over 12 months. |
| Equipment paid for in goods | `Opening.exchange_values` gives positive reporting ticks per noncash stock unit. The tool receives consideration value, the provider recognizes disposal results and received inventory, and the buyer recognizes goods sales/cost of sales. No cash legs are invented. Missing values remain errors. |
| Observed financial execution | `Observer::step_audited` uses Audit's atomic execution before observing committed results. Accounting failure publishes neither simulation nor journal; telemetry I/O failure retains its existing post-commit semantics. |

WIP transfers use carrying cost as a noncash transfer convention. They do not
appraise the crop, capitalize anticipated profits, settle a loan twice, or price a
separate crop sale. Economic changes from maintaining/neglecting the crop remain
in the simulation. The configured barter values are recognition values, not marks
to all existing stock or changes to market negotiation.

The credit-stress example now runs normal, temporary and persistent harvest-loss
cases with finalized financial reports and JSONL telemetry. Coverage tests check
CPU/reference agreement, checkpoint continuation, unchanged deficiency, WIP
transfer gains/losses, expiration once, historical-opening parity, and failed
accounting through the observer. Equipment barter tests check full recognition,
zero cash flows and continuation.

The expansion is **not universal coverage yet**. These valid economic situations
still need accounting adapters or policy definitions:

- Household dissolution/estate distributions and consolidated reporting beyond the supported pooling agreement.
- Estimated/capitalized contingent consideration beyond the earned-only royalty policy below; noncash exchanges outside supported posted, negotiated and town-market payment terms.
- Paid labor capitalization, priced contract production, non-pool resource ownership, and combining distinct beneficiaries with royalties.
- Multiple loan/estate denominations and FX valuation; redeemable currency and retirement.
- Explicit dues discharge, forward refund/repricing and impairment policies.
- Full durable Audit/Simulation restart, consolidation and contingent/noncash disclosures.

Ordinary validation failures (forged batches, insufficient physical stock,
missing historical values, overflow or postings into finalized periods) must stay
errors. Removing them would not expand legitimate economic coverage.

Validation: the final accounting regression run passed **57 tests** across accounting
(14), dues (7), equipment (3), estate dues (4), forwards (5), issuance (6), manufacture
(7), processes (5) and coverage (6). The earlier broader run also passed credit
stress (5), telemetry (10) and town-market (17) suites. Strict all-target Clippy and
the CPU credit-stress example passed. Test logs, reports and telemetry remain under
ignored `output/economics/`; no generated artifacts are committed.


## Household pooling and shared resource inputs

Households now have individual financial statements alongside their members.
Accounting reuses the authoritative household settlement's three ordered views:
allocation to members, core execution, and collection of contributions. These
views are candidates until the complete batch and journal both validate. There
is still one journal entry per batch and no monthly scheduler change.

Stock grants and contributions transfer carrying cost, recognizing TransferExpense
for the sender and TransferIncome for the recipient. These ownerless households
do not issue equity claims to members. Cash support is an operating transfer;
member dues and forwards retain their original debtors and separate settlement
entries. Unpaid delegated labor and nonfinancial fulfillment receive no
invented monetary value. Extra nonrival dwelling-service tickets enter at zero
incremental cost: the actual occupancy process already bears the dwelling wear.
This avoids depreciating one shared house once per member. Household collection after production uses the output's
actual assigned cost; allocation before execution makes that cost available to
consumption or work in progress. Each transfer sub-boundary uses opening holdings,
with cumulative rounding that preserves every reporting tick.

Authorized shared-pool inputs also carry their cost into the operator's production.
The pool owner recognizes a transfer expense and the operator a transfer income.
Only declared process/pool input permissions qualify; unrelated third-party
inputs still fail. Distinct operator/beneficiary arrangements now have the separate
carrying-cost policy below. Verified Open
regeneration adds quantity at zero new acquisition cost and preserves the pool's
existing total basis. This is a historical-cost convention, not fair-value
biological growth revenue. A coin-generating pool is not monetary issuance.

Focused tests exercise repeated farming and household consumption across 12 months,
CPU/reference and checkpoint parity, household-funded native/coin dues, forged
receipt rejection, shared dwelling consumption without duplicated wear, shared-pool
cost transfer, zero-cost regeneration, and rounding
without spending same-boundary incoming stock. These checks establish journal
recognition and conservation, not household welfare or economic balance.

The larger specialist-household integration test is deliberately opt-in because
its repeated planner verification is substantially slower than the focused tests.
It uses 32 people, eight households, the default contributed-labor governance,
explicit unit opening costs and equal total joint-output cost shares. Its target
is 13 months, including the first annual-dues boundary. Run it from `exp/economics`:

```sh
cargo +1.92.0 test --locked --test household_accounting specialist_households -- --ignored --nocapture
```

The fixture keeps the real specialist catalog, finite shared inputs, house/tool
manufacture, household sharing, forwards and collection-linked currency rules.
Equal accounting cost shares are a test convention, not calibrated market values.

Validation result: **92 focused/regression checks passed**, plus the explicit
32-person CPU run completed all 13 months and finalized balanced statements for
every agent. The slow run took about nine minutes on this machine. Strict
all-target Clippy, formatting and repository artifact checks passed. The result
establishes supported accounting through this scenario; it does not validate all
royalty, insolvency, FX or dissolution arrangements. Logs remain under ignored
`output/economics/`.


## Shared opening-cost allocation and posted barter

Trade, forward, equipment-barter, process and dues adapters now use a single
`CostAllocation` for each core accounting boundary. It retains opening quantities
and basis while the working inventory accumulates receipts and outputs. All
outgoing claims share the original quantity limit: a receipt can neither fund
another same-boundary disposal nor change its unit cost.

Cost recognition visits aggregated sales first, processes by stable process ID,
and dues by dated agreement key. Cumulative proportional rounding allocates each
reporting tick once and releases the final remainder on full depletion. This is
an accounting rounding convention, not a physical allocation priority. Exact
quotient/remainder arithmetic also avoids rejecting representable costs merely
because an intermediate multiplication would overflow.

**The scheduler and settlement admission rules are unchanged.** Trades and
production still run at their existing phases. The three-adapter composition is
tested directly as costing machinery; it does not authorize an otherwise invalid
mixed simulation batch. Existing CPU scenario regressions verify integration at
the current phase boundaries.

For an accepted posted bid with two noncash commodities, `Opening.exchange_values`
now prices the bid's **payment** commodity in reporting ticks. Actual payment
quantity times that explicit unit value establishes both deliveries' consideration.
Each party recognizes sales, releases its surrendered stock's historical cost,
and receives inventory at that consideration value. Neither records a cash flow.
This valuation does not mark existing holdings to market or alter physical prices.

Example: two grain units with carrying cost 14 exchange for four wood units with
carrying cost 8. With wood valued at three reporting ticks per unit, each party
records sales of 12. The grain seller records cost of sales 14 and wood inventory
12; the wood seller records cost of sales 8 and grain inventory 12. Missing quotes
or an unrepresentable consideration value reject accounting without publishing it.
Equipment barter retains its existing convention. Negotiated and town-market
stock barter now use their own accepted payment terms, as described below;
unsupported noncash exchanges still fail rather than guessing a payment resource.

Validation: **74 checks passed**: 71 accounting integration/regression tests and
three cost-allocation unit tests. Posted barter checks compare CPU/reference,
checkpoint continuation, finalized periods, combined coin/barter sales, reversed
transaction order, and missing/overflowing valuation rejection. The cost-only
composition balances a journal across trades, WIP and dues, retains newly received
stock, and checks exhausted/failed allocations and large representable costs.
The slow 32-person integration was not rerun for this change; the four focused
household accounting tests passed. Logs are under ignored `output/economics/`.


## Earned-only tool royalties

`Opening.processes.earned_royalty_values = Some(prices)` opts into a specific
contingent-payment convention. Prices are positive reporting ticks per unit of
noncash stock delivered as royalty. `None` preserves strict rejection. Missing
prices at delivery of output, arithmetic overflow and forged receipts fail without
publishing either the simulation or its financial report. Changing joint-output
cost weights at opening preserves the royalty policy.

This convention recognizes no estimated future royalties, guaranteed principal,
loan, receivable or payable at tool delivery. The supplier releases the tool's
remaining historical basis to `CostOfSales`; the recipient acquires a zero-basis
tool. Physical ownership, useful life, wear, production benefits and the existing
output-share agreement still operate normally. Future consideration is contingent
on completed production, so unsuccessful or idle work earns no royalty.

For completed work, first allocate material and productive wear costs over **all**
actual joint output using the existing output weights. Then allocate each product's
cost proportionally between the operator's retained units and the supplier's share.
Rounding remains in the retained output. Only this process's new output pays the
royalty; it cannot draw on the operator's opening inventory or another process.
Recycled seed remains exempt according to the authoritative production receipt.

The operator records the delivered share as a noncash sale at the configured
reporting value, releases its allocated `CostOfSales`, and recognizes the same
value as `ServiceExpense`. The supplier recognizes `ServiceIncome` and inventory
at that value. These postings do not create cash, finance or minting entries.
The operator's retained inventory keeps its allocated production basis; the
royalty expense is not capitalized into it. The supplier's received goods can
subsequently be sold, pooled, consumed or expire using ordinary inventory adapters.

For example, production consuming 100 ticks of hay and one tick of herd wear has
101 total cost. Equal milk/wool cost weights assign 50 and 51 ticks. A 25% share
of 200 milk and 100 wool sends 50 milk and 25 wool to the tool supplier. At three
reporting ticks per unit, this is 225 of supplier service income and inventory.
The operator recognizes 225 noncash sales, 225 service expense and 24 cost of sales;
its retained products carry 38 and 39 ticks. A tool previously costing 24 is
expensed once by the supplier at delivery, not again as the recipient uses it.

This is an explicit experimental reporting convention, not estimated fair-value
accounting or a claim of accounting-standards compliance. Capitalized contingent
purchase prices, royalty contract assets, minimum guarantees, changing valuation
quotes, royalty impairment and general third-party production still need policies
and adapters. No matching, physical allocation, royalty rate or monthly phase was
changed. A zero-percent agreement is also treated as delivery for zero consideration.

Verification in `tests/royalty_accounting.rs` covers 0%, 25% and 100% output shares,
exact cost allocation, tool delivery, idle periods, CPU/reference equality,
checkpoint continuation, finalized periods, failed work and exhaustion followed by
successful manual production. Missing policy, missing product price and overflow
are checked for atomic rejection. The fixture uses the existing specialist catalog
with other work and needs disabled to isolate accounting; it does not establish
economic sustainability of the full specialist economy.

Validation: **68 checks passed** across library (10), accounting (14), posted barter
(3), equipment (3), forwards (5), household accounting (4), inventory (6),
manufacture (7), process accounting (6), reporting coverage (6), and royalties (4).
The existing slow full-household stress test remains ignored in this run. Strict
all-target Clippy, formatting, diff checks and repository artifact checks passed.
Generated logs are under ignored `output/economics/royalty-*.log`.


## Completed output for a distinct beneficiary

Set `Opening.processes.beneficiary_policy` to
`Some(BeneficiaryPolicy::TransferAtCost)` when an existing production right grants
output to someone other than the operator. The simulation already determines this
recipient through `ProcessInstance.beneficiary`; reporting does not choose a new
recipient or change allocation, planning, rights or monthly execution. Absence of
the policy still rejects distinct-beneficiary work before publication.

The accounting boundaries are:

- Inputs and productive equipment wear enter the operator's WIP when committed work
  uses them. Unfinished costs belong to the operator even when another agent is
  entitled to future output. A historical opening must identify that operator and
  the exact WIP cost; an expected harvest is not an asset of the beneficiary.
- At successful completion, existing cost shares allocate the WIP basis across
  stock and durable outputs. Each completed financial output goes to its actual
  beneficiary at carrying cost. The operator recognizes `TransferExpense`, the
  beneficiary `TransferIncome` and inventory or a tangible asset. No cash, sale
  price, wage, debt, profit margin or ownership stake is inferred.
- Missed work or blocked completion leaves `ProductionLoss` with the operator.
  Nothing transfers to the beneficiary when the process produces no output.
  Consumption and repair costs remain expenses of the agent carrying those costs;
  nonfinancial fulfillment does not itself create a financial asset or transfer.
- Later inventory consumption, sale or expiration, and durable depreciation, use
  the recipient's ordinary accounting. Transfers occur once at completion, not
  when the output right is granted or at each monthly progress update.

For a person supplying seed costing 12 to grow grain for another agent, the person
initially holds WIP of 12. A completed harvest transfers that cost to the recipient's
inventory and recognizes a transfer expense/income of 12. With grain/seed output
weights of 3:1, those inventories carry 9 and 3. A failed harvest instead recognizes
12 of production loss for the person and no recipient income.

This composes with authorized shared-pool inputs: donor-to-operator input cost and
operator-to-beneficiary completed output are separate transfers at their actual
boundaries. When the beneficiary also supplies the seed, the two transfers conserve
its original cost through the operator's WIP. They do not invent production profit.
A constructed asset likewise belongs to the beneficiary and depreciates there.

This is a carrying-cost contribution convention, **not paid contract production**.
It does not choose who should bear contract risk, bill a customer, recognize wages,
or capitalize purchased labor. Distinct-beneficiary output combined with supplier
royalties still rejects atomically: a further policy must specify which party owes
the royalty and how that consideration relates to the output transfer. The existing
owner-operated royalty policy remains unchanged. Arbitrary third-party inputs,
intermediate output allocation and organizational consolidation remain outside this
increment.

`tests/beneficiary_accounting.rs` checks CPU/reference equality, continuation from
unfinished work, historical WIP opening, exact joint-stock cost allocation, a joint
house/material output followed by beneficiary depreciation, missed work,
recipient-storage failure and beneficiary-supplied pool inputs. Missing policy and
wrong historical WIP owner are rejected. `tests/royalty_accounting.rs` also checks
that unsupported three-party royalty/output arrangements do not publish either
state or journal. These are controlled accounting fixtures; they do not establish
that an autonomous operator would choose an unpaid production obligation.

Validation: **74 checks passed** across library (10), accounting (14), posted barter
(3), beneficiary accounting (5), equipment (3), forwards (5), household accounting
(4), inventory (6), manufacture (7), process accounting (6), reporting coverage (6),
and royalties (5). The existing slow full-household stress test remains ignored.
Strict all-target Clippy, formatting, diff checks and repository artifact checks
passed. Generated logs remain under ignored `output/economics/beneficiary-*.log`.


## Negotiated and town-market barter

The stock adapter now handles bilateral negotiated barter and town-market matches,
including ZIP-priced exchanges, using the same opening-inventory costing as posted
barter. `Opening.exchange_values` supplies a positive reporting value per unit of
the actual payment commodity. The negotiation session identifies that commodity;
a town market uses its catalog's common payment resource across listings.

Only a replay-verified, settled exchange creates revenue. The actual four transfer
legs determine both parties, goods quantity and payment quantity. The adapter checks
two matching stock transfers, applies the payment commodity's reporting value, and
recognizes two reciprocal noncash sales. Each party releases its own opening cost
and receives inventory at the agreed consideration. Cash flow remains zero. Quotes,
unsuccessful negotiations, unmet orders and the town's last posted price are not
used as evidence of delivery or as inventory revaluation.

For example, if a completed match exchanges two grain for ten wood and wood is
valued at three reporting ticks per unit, both transfers carry consideration 30.
The grain supplier receives wood costing 30 and releases its own grain basis; the
wood supplier receives grain costing 30 and releases its own wood basis. Different
historical costs create different disposal margins, while physical quantities and
the canonical journal continue to reconcile.

The shared adapter also reads actual effects for posted barter, retaining support
for seller-specific payment terms. It requires two noncash stock resources and an
explicit supported source of payment terms. Missing values, overflow, mismatched
legs or unauthenticated transactions still reject without publishing simulation or
journal changes. This does not add capacity barter, FX remeasurement, multilateral
netting, or new negotiation/clearing behavior.

`tests/barter_accounting.rs` checks negotiated and town exchanges with ordinary and
ZIP quote policies on CPU and reference backends, exact revenue and released costs,
continuation, completed-period finalization, and zero cash flows. Missing prices and
overflow reject atomically for both venues. A no-agreement control needs no barter
valuation and recognizes no revenue. Existing posted barter, concurrent coin sales,
and opening-cost allocation tests remain in the regression suite.

Validation: **77 checks passed** across library (10), accounting (14), barter (6),
beneficiary accounting (5), equipment (3), forwards (5), household accounting (4),
inventory (6), manufacture (7), process accounting (6), reporting coverage (6), and
royalties (5). The existing slow full-household stress test remains ignored. Strict
all-target Clippy, formatting, diff checks and repository artifact checks passed.
Generated logs remain under ignored `output/economics/venue-barter-*.log`.
