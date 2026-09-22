# Secured asset financing on CPU

Implemented as an isolated financial experiment in `src/credit.rs`. A person
purchases a plot with initial coins and a secured loan. Purchase, monthly interest,
repayment and default settle through the existing transaction and CPU commit path.
The application and repayment cashflows are supplied controls, not yet decisions
made by the farming planner.

## Generic pieces

- `Sale`: seller, asset and price.
- `LoanOffer`: creditor, denomination, maximum principal, monthly rate, term and
  arrears grace period.
- `Collateral`: asset, priority and agreed enforcement value.
- `Offer`: sale and financing terms, with minimum downpayment.
- `Application`: buyer, offer, acceptance month and downpayment.
- `Loan`: accepted terms, parties, outstanding principal, accrued interest,
  dated accrual state and collateral status.

These use agent, asset and resource IDs rather than a mortgage-specific person
type. Seller and lender can differ; a test covers a third-party lender. One dated
application is supported in this pilot. Offer discovery checks eligibility and
ownership; acceptance also checks funding. No cash, title or debt changes if
acceptance fails. The sale, advance, title transfer and pledge commit together.

The authoritative loan book supplies both parties' balance sheets. Principal and
interest are liabilities of the borrower and matching receivables of the lender.
The borrower owns the purchased asset while it is pledged; the lender does not
also count the collateral as an owned asset. Equity is owned assets plus coins
and receivables, less payables. These are scoped views of listed assets and loans,
not complete financial statements for every existing subsystem.

`World.assets` describes opening ownership. `credit::owner` applies the book's
subsequent title changes. Existing farming rights have not yet been connected to
this ownership view; validation prevents mixing this fixture with those systems.

## Monthly boundaries and policy

The fixture uses the existing scheduler, with no additional monthly loop:

1. **Open:** record initial endowments in month one and scheduled cash transfers
   in subsequent months. Committed transfers are visible to Due.
2. **Due:** accrue interest once, collect due payments, record arrears, then
   enforce eligible collateral claims. Outcomes are committed before Acquire.
3. **Acquire:** accept the scheduled financed purchase against current ownership
   and opening cash budgets. Interest first becomes due the following month.

Amounts are integer ticks, with 100 ticks per coin. The example sells a plot for
100 coins, requires 20 down and lends 80 at 1% monthly for four equal principal
instalments. Payments cover interest before principal. Interest uses opening
outstanding principal, excludes unpaid interest, and carries sub-tick remainders
between months. A residual fraction below one tick is waived when debt is cleared.
Principal due uses cumulative scheduled instalments, so missed payments remain due.

Initial endowments explicitly create starting coins; lending itself creates none.
With separate seller and lender, the advance goes directly to the seller. When
seller and lender are the state, the financed portion becomes its receivable
without a self-transfer; it receives the downpayment in coins. The pilot still
requires the lender to have the principal available at acceptance, a conservative
funding rule even for seller financing. Incoming funds within the same boundary
cannot fund another outgoing transfer in that boundary.

## Missed-payment consequences

One full month in arrears permits enforcement. A payment missed in month two can
be cured at month three's Due boundary before enforcement. Otherwise the lender
takes title and credits the configured collateral value against debt, interest
first. A valuation shortfall remains a borrower liability and lender receivable;
it is not forgiven. A valuation surplus must be paid to the borrower. If the
lender cannot fund that surplus, enforcement is deferred without transferring
title or reducing debt.

Enforced deficiency debt remains collectible but stops accruing interest in this
pilot. Paying it later does not return the asset. Full repayment releases the
pledge. Collateral value is an explicit scenario assumption, not an auction price;
the borrower bears a loss if it is below the asset's purchase carrying value.
Deficiency receivables remain valued at face, without expected-loss provisions.
Priority is recorded, but multiple liens on an asset are rejected.

## CPU controls

Run from this directory:

```sh
cargo +1.92.0 run --locked --example credit
cargo +1.92.0 test --locked --test credit --test finance --test economics --test conditions --test zip
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Six-month CubeCL CPU results, with amounts below expressed in **coins**:

| Control | Ending principal | Cash interest paid | Plot owner | Buyer coins |
| --- | ---: | ---: | --- | ---: |
| Regular repayment | 0 | 2.00 | Person | 2.00 |
| Downpayment one tick short | No loan | 0 | State | 19.99 |
| Missed payment, then recovery | 0 | 2.20 | Person | 1.80 |
| Default, collateral worth 60 | 21.60 | 0 | State | 0 |
| Default, collateral worth 100 | 0 | 0 | State | 18.40 |

The repayment controls supply treasury-to-person transfers: 21 coins in months
two through five, or 42 in month three followed by 21 in months four and five.
These are recorded test cashflows, not farming income or evidence of affordability.
Default cases accrue 1.60 interest, covered through collateral rather than cash;
this explains the zero cash-interest column. Insufficient lender funding also
rejects the purchase without changing title or creating debt.

All 41 tests across the five suites above passed, including nine credit tests;
Clippy passed with warnings denied. Controls cover fractional interest, partial
payment, surplus funding failure, later deficiency collection, separate lenders,
forged receipts and atomic rejection. Monthly, batched and checkpoint-resumed
execution agree, as do reference and CubeCL CPU settlement. Opening table order
does not change final state. Generated logs remain under ignored `output/economics/`.

## Next integration boundary

Expose this offer through common opportunity discovery, let planning compare
future instalments against production and essential needs, and make ownership
grant cultivation access. Repossession must then remove future access without
retroactively undoing completed work. This experiment deliberately rejects mixed
production, household, marketplace and legacy access configurations until their
accounting and rights boundaries are connected.

Negotiated loan terms, refinancing, unsecured lending, multiple competing claims,
guarantors, liquidation markets, write-offs and general insolvency remain future
work. This establishes a secured financing component, not a universal contract
interpreter or a sustainable farming-and-mortgage economy.
