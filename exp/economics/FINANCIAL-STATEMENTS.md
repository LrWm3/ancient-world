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
depreciation and capital movements. Manual postings require their own domain
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
It rejects other nonzero stocks or stock movements without valuation, mixed loan
or estate denominations, equipment, forwards, land-dues obligations, households,
production and market/minting transactions. Unsupported activity is an error,
not zero value or an unexplained income/equity adjustment. Unused capacities and
need satisfaction are not financial assets. Uncalled guarantees remain contingent
agreements in the contract inspection view; they are not recognized liabilities
or a complete accounting disclosure schedule here.

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

Completed validation: **79 tests passed** across `accounting` (11), `credit` (9),
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

1. Choose inventory costing and recognition rules for production, consumption,
   storage losses and market trades; distinguish price from carrying cost.
2. Define minting input cost, issuance and issuer equity/liabilities explicitly.
3. Recognize forward advances and delivery obligations, land dues, alternative
   tender, restructuring and delivery write-off in monetary statements without
   erasing their native performance requirements.
4. Add durable equipment depreciation, household/institution contributions and
   distributions, then ownership and consolidation eliminations. Summing entity
   statements is not a consolidated economy statement: custody and intra-economy
   claims need elimination.
5. Add valuation/FX policies, impairment allowances, contingent-claim and noncash
   financing disclosure schedules, statement-period finalization and persistence.

The adapter deliberately replays validation and clones small scenarios. It has not
been optimized or benchmarked for a large population. Balanced books establish
accounting consistency, not economic calibration, solvency or recoverability.
