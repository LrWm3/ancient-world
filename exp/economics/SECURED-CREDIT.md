# Secured asset financing on CPU

Implemented as a secured-financing component in `src/credit.rs`. A person
purchases a plot with initial coins and a secured loan. Purchase, monthly interest,
repayment and default settle through the existing transaction and CPU commit path.
Application terms and repayment cashflows remain supplied controls. An opt-in
[borrowing comparison](BORROWING-DECISIONS.md) now chooses whether to accept the
configured offer using needs, productive work and projected coin payments. A second control connects an active crop to ownership
and follows it through repossession, maintenance or neglect.

## Generic pieces

### Common offer adapter

Financed purchases now use `offers::discover`, `prepare`, `feasible` and `accept`
with `Id::FinancedPurchase(offer_id)`. Discovered terms include the sale/loan offer
and the configured application, making its buyer, date and downpayment visible.
The monthly driver's scripted application also passes through this dispatcher.
Credit still owns funding checks, collateral availability and settlement receipts.

Discovery excludes inactive counterparties, already-used loan IDs, unavailable
title, pledged collateral and buyers lacking `FinancedPurchase` permission when
a state policy exists. It does not promise affordability. The common
adapter exposes only the configured application to its buyer, through its stated
month; acceptance requires that month's Acquire boundary. It does not yet permit
arbitrary buyer applications, negotiate a downpayment or select borrowing through
need-directed search.

For example, at the configured Acquire boundary:

```rust,ignore
let request = offers::Request::new(offers::Id::FinancedPurchase(1), buyer);
offers::feasible(&sim, &[request.clone()])?; // read-only
offers::accept(&mut sim, &[request])?;     // ordinary atomic settlement
```

Opening coins fund the downpayment and advance. Prepared batches revalidate
against current terms, title, debt and dated state before publication; they are
not binding price reservations. ID-only requests use current terms when prepared.
Purchase receipts make cash, title and loan changes visible together before
Productive. Existing Due interest/collection and later collateral resale timing
are unchanged. Failed explicit acceptance changes neither state nor ledger; the
monthly driver retains its ordinary credit rejection receipt and proceeds.

This first adapter requires one standalone financed-purchase request. Duplicate
requests and bundles with production or other acquisitions are rejected rather
than implying joint reservation support. Existing credit incompatibility guards
remain except that a configured bilateral negotiation can share the acquisition
batch through the [shared resolver](INTEGRATION-STATUS.md). That session is a
separate driver, not an extra financed-purchase request. Resale bids still use
their separate pilot resolver.

### Domain records

The common-offer adapter passed four focused tests covering discovery, read-only
feasibility, explicit/automatic receipt equality, both funding failures, changed
terms and ownership, duplicate acceptance, unsupported bundles, an independent
lender and checkpoint continuation. Together with process-offer, credit, resale,
collateral-crop and loan-view regressions, **36 tests passed**. CPU/reference
comparisons and all-target Clippy passed. No new simulation parameters or monthly
phases were introduced.

Run `cargo +1.92.0 test --locked --test credit_offers` from this directory.

- `Sale`: seller, asset and price.
- `LoanOffer`: creditor, denomination, maximum principal, monthly rate, term and
  arrears grace period.
- `Collateral`: asset, priority and an agreement-selected settlement rule;
  `FixedValue { value }` and the opt-in [resale-proceeds policy](COLLATERAL-RESALE.md)
  are implemented variants of `CollateralSettlement`.
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
not complete financial statements for every existing subsystem. The optional
[journal-backed reporting adapter](FINANCIAL-STATEMENTS.md) now adds full statements
for valued mortgage and recovery scenarios, including disposal results and actual
cash flows. Owner-operated attachments can opt into material costing; transfer
of active crop costs still requires an accounting adapter.

`World.assets` describes opening ownership. `credit::owner` applies the book's
subsequent title changes. Rights explicitly listed in `Config.attached_rights`
follow that ownership for both cultivation and future output entitlement. Legacy
access agreements remain separate and cannot be combined with this fixture.

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

## Standing crops follow ownership; settlement value stays fixed

The baseline rule is **option 1: no crop-value adjustment**. The lender receives
the standing crop opportunity and its remaining work requirements at the agreed
fixed settlement value. A nearly mature, newly planted or failing crop does not
change the amount credited against debt. The crop has economic consequences but
is not separately appraised in the collateral settlement or loan balance sheets.

At purchase or repossession, active processes bound to an ownership-following
right transfer atomically with title. The receipt records their before/after
state. Operator and output beneficiary become the new owner; the previous
operator's personal planning goal is cleared. Definition, stage, elapsed work,
start date, occupancy end date and active status remain unchanged. No seed is
refunded and no output is produced by the transfer. Completed/aborted process
records, harvested goods and the former owner's labor balances stay where they
were. Already incurred debt also remains with the borrower.

Open regenerates capacities once using the existing opening implementation. Due
can then transfer the crop before Productive allocates that month's work. The
new controller must supply its own available services and remaining stage inputs;
ownership does not create labor or transfer the previous owner's future labor.
The crop's occupancy end date is not an escrow of somebody's labor. There is no
pending production plan across this boundary in the supported credit fixture.

The current process engine combines controller and worker in `operator`. Hiring
someone else to supply work remains a later integration. The separate
[resale pilot](COLLATERAL-RESALE.md) now transfers the plot and crop to a cash buyer.
In the
control, the state receives explicitly configured capacity to represent available
services, not an automatic capacity gain from repossession. With capacity, the
existing continuing-work policy maintains the crop. Without it, the existing
missed-work consequence aborts the crop at Productive, not during transfer.

A subsequent [remaining-value policy](REMAINING-VALUE.md) now lets the state
compare continuation against wood collection and waiting, using the same
ownership transfer and fixed settlement terms. The controls below retain their
original continuing-work policy as a comparison.

Two six-month CPU controls plant with one seed in month one and repossess during
growth in month three. Each uses the same 60-coin settlement value:

| New owner's available labor per month | Crop at month six | State outputs | Remaining debt |
| --- | --- | --- | ---: |
| 2 units | Completed | 8 grain, 1 seed | 21.60 coins |
| 0 units | Aborted | None | 21.60 coins |

Both preserve crop progress at the ownership boundary. The original person
receives no future harvest. A separate completed-crop control retains its already
harvested goods with the person. If surplus payment cannot be funded, enforcement
and the crop transfer both wait. Five attachment tests cover these outcomes,
tampered/missing transfer receipts and reference/CPU, monthly/batched and
checkpoint-resumed equivalence. Run `cargo +1.92.0 test --locked --test collateral_crop`;
the `credit` example also prints the maintained/neglected comparison.

After the attachment change, `cargo +1.92.0 test --locked --tests` passed all
234 tests. Clippy with warnings denied and formatting checks also passed. CPU
example output and raw verification logs remain in ignored `output/economics/`.

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

Common offer discovery, bounded accept/decline borrowing, ownership-following
production and production-funded coin repayment are implemented in their linked
pilots. Credit and bilateral marketplace exchange now share acquisition budgets;
existing citizenship can authorize financed purchases. See the
[integration matrix](INTEGRATION-STATUS.md) for exact supported combinations.
Households, legacy access and joint production plans with negotiated exchange
still require additional integration.

**Implemented opt-in extension: option 3, settlement from actual resale
proceeds.** The [resale pilot](COLLATERAL-RESALE.md) adds pending custody, a
cash-limited prospective buyer, frozen interest after repossession and atomic
debt/surplus settlement from actual payment. Missing buyers and insufficient bids
leave the asset pending. Fixed-value settlement remains available; both variants
use the same crop-transfer machinery.

Subsequent [contract consolidation](CONTRACT-CONSOLIDATION.md) adds unsecured
direct advances and shared competing-claim allocation. [Contract recovery](CONTRACT-RECOVERY.md)
adds capped guarantees, configured funded liquidation, explicit write-offs and
single-denomination direct-loan estates. Pending-resale guarantees and admission
of this older mortgage driver to estate proceedings remain rejected.
Negotiated loan terms, refinancing, autonomous competitive liquidation and general
insolvency remain outstanding. These components do not establish a universal
contract interpreter or a sustainable farming-and-mortgage economy.

## Scoped stock-sale extension

The optional [production-funded credit pilot](PRODUCTION-FUNDED-CREDIT.md) permits
one posted stock bid with explicit food/input reserves, quantity and funding caps
inside Acquire. Forecasts use the same settlement path. It can now share the
[acquisition boundary](INTEGRATION-STATUS.md) with a bilateral negotiation; the
posted bid retains its own pricing policy and finite spending allowance.
