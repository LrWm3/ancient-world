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
mixed loan or estate denominations, equipment barter/royalties, unconfigured land dues,
households and town-market clearing. Minting/issuance requires its explicit policy below. Process transactions
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
basis, not a guessed market price. Equipment wear is included as described below;
paid labor and overhead capitalization are not covered. These are recognition rules, not planner valuations.

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

1. Extend material costing to capitalized paid labor, equipment repair/manufacture and cross-agent work; supply
   historical work costs on reporting restart. Add validated stored-goods loss events,
   then town-market and barter accounting adapters.
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
   financing disclosure schedules, statement-period finalization and persistence.

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
retain opening equity at every boundary. Missing opening costs and unsupported
barter are rejected; failed accounting publishes neither simulation nor book.

This covers posted coin purchases and owner-operated wear. Barter, royalty tool
delivery, tool manufacture/repair, and transferred
production costs still require adapters. A financial statement is not yet available
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
refund/repricing adapter, or recognition of royalty deals. Tool manufacture/repair
and transferred work costs still prevent reporting an entire unrestricted
specialist scenario. Supporting forward reports does not remove simulation-driver
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
The provisioning/leisure variants and their perishable service-ticket inventory
are not yet covered by this reporting adapter. Estate-paid native/accepted-coin
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
