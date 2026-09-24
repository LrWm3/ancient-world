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
- Changes in equity: opening equity + contributions − distributions + net income.
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
These are in-memory checkpoints; durable accounting serialization is not implemented.

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
| Estate distribution | Restricted cash settles loan principal/interest. Custodian equity stays unchanged. |
| Explicit loan discharge | Creditor records credit loss; debtor records debt-relief income. No cash movement. |

Tangible assets require an explicit opening valuation for **every** asset through
`Audit::with_assets`. Accepted purchases, contractual fixed-value enforcement and
actual resales establish subsequent carrying values. Market quotes alone do not
revalue holdings. Pledged land is counted once, on the economic owner's books.
This is a model-specific cost/contract-value convention, not a claim of compliance
with a real-world accounting standard.

The adapter recognizes the selected coin at one reporting tick per stored tick.
It rejects nonzero stocks without explicit opening cost, unsupported stock movements,
mixed loan or estate denominations, equipment, forwards, unconfigured land dues,
households, town-market clearing and minting transactions. Process transactions
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

This spot-trade adapter does not recognize spoilage, commodity obligations, barter
or town-market accounting. Production/consumption use the separate opt-in adapter
below. An inventory cost alone does not authorize an unknown stock movement.

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
basis, not a guessed market price. Paid labor, equipment depreciation and overhead
capitalization are not covered. These are recognition rules, not planner valuations.

All input releases use the opening-boundary average cost. When several processes
consume the same stock, cumulative rounding allocates released cost by stable
process ID. Output shares use cumulative rounding in resource-ID order. These
conventions allocate every tick and do not change which process receives physical
resources. New output is not available to other work in the same batch; inventory
cost becomes visible with the existing committed Productive boundary and is then
available to the later Consumption boundary.

This adapter supports owner-operated work: input owner, operator and beneficiary
must agree. Work-in-progress title/beneficiary transfers, third-party inputs,
royalties and combined trade/production batches reject pending explicit adapters.
Cloning the audit preserves work costs for continuation. Opening a new audit over
already active work rejects, including when no seed remains in physical inventory;
otherwise a restart could silently discard capitalized cost. Supplying historical
work costs to a new book remains outstanding.

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

Limits remain explicit: collection-linked currency issuance, estate distributions
for dues, arbitrary relief, and combined same-boundary trade/production/dues cost
allocation require further adapters. Ordinary Due and Productive/Consumption work
compose on their existing separate boundaries. Forwards remain unsupported:
their current origination bundles a prepaid delivery agreement with tool purchase,
so equipment acquisition/cost accounting is a prerequisite to that integration.

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

1. Extend material costing to paid labor, equipment and cross-agent work; supply
   historical work costs on reporting restart. Add validated stored-goods loss events,
   then town-market and barter accounting adapters.
2. Define minting input cost, issuance and issuer equity/liabilities explicitly.
3. Recognize forward advances and delivery obligations, restructuring and delivery
   write-off without erasing native performance. Ordinary dues/alternative tender
   now have an adapter; extend it to estate payments and explicit relief.
4. Add durable equipment depreciation, household/institution contributions and
   distributions, then ownership and consolidation eliminations. Summing entity
   statements is not a consolidated economy statement: custody and intra-economy
   claims need elimination.
5. Add valuation/FX policies, impairment allowances, contingent-claim and noncash
   financing disclosure schedules, statement-period finalization and persistence.

The adapter deliberately replays validation and clones small scenarios. It has not
been optimized or benchmarked for a large population. Balanced books establish
accounting consistency, not economic calibration, solvency or recoverability.
