# Collateral settlement from actual resale proceeds

Implemented as an opt-in agreement policy in `src/credit.rs` and `src/resale.rs`.
`CollateralSettlement` now selects either `FixedValue { value }` or
`ResaleProceeds { minimum_price }`. Accepted loans copy their settlement terms;
the new policy does not change existing fixed-value controls.

This is a controlled listing with one prospective buyer, not a general auction,
ZIP market or integration into the earlier grain marketplace agent.

## Repossession, custody and settlement

At the existing Due boundary, eligible default under the resale rule:

1. Transfers the plot and its active crop to the creditor using the same attachment
   receipts as fixed-value repossession. Crop progress and future work stay intact.
2. Records `PendingSale` and the listing month.
3. Leaves principal and accrued interest unchanged. No asking price, bid or
   appraisal is credited against debt.

Interest stops accruing after repossession. Automatic borrower collection also
pauses while the sale is pending; redemption and borrower cure during custody
are outside this pilot. These are explicit scenario rules, not claims about
real-world mortgage law.

The first sale attempt is at Acquire in the following month. This preserves a
work window for the lender before sale. The lender's existing remaining-value
policy decides whether to maintain the crop or allocate its capacity to wood.
Later Acquire boundaries retry an unsold listing; there is no automatic price
reduction or timeout.

An accepted buyer pays actual coins. The portion covering debt goes directly to
the creditor, with interest settled before principal. Any surplus goes directly
to the original borrower. Splitting the payment avoids requiring the lender to
spend newly received money within the same boundary. Cash, debt, title, asset
carrying value and crop control commit together. A remaining deficiency is still
collectible, with no further interest, under the existing enforced-debt rule.

If the buyer is absent, inactive, already operating another active process, or
offers less than the contractual minimum, the listing remains pending. An absent
eligible bidder or rejected bid produces a receipt and changes no cash, debt,
title or crop. Normal work later in the month can still change the crop.

## Balance-sheet treatment while waiting

Legal control and financial realization are separate here. The lender controls
the asset for sale, but counting its full carrying value alongside an unchanged
loan receivable would inflate lender equity.

While pending, the original borrower retains the restricted economic asset in
`assets`, identified by the `assets_awaiting_sale` subtotal. The lender reports
`collateral_in_custody` as a memo excluded from equity. Matching principal and
interest receivables/payables remain outstanding. This is a carrying-value view,
not a current market appraisal.

At sale, the restricted asset and custody memo disappear, the buyer records the
asset at the actual purchase price, and debt/surplus settlement realizes the
difference. A bid alone changes none of those balances. As before, these financial
views do not value every inventory stock in the broader simulation.

## Buyer valuation

The prospective buyer has a configured bare-land value, a horizon and stock
valuations expressed in loan-denomination ticks. It reuses the remaining-value
evaluator in two read-only forecasts:

```text
without asset: best available alternative work
with asset:   best work after acquiring the plot and attached crop

crop premium = max(0, with-asset value − without-asset value)
willingness  = configured land value + crop premium
bid          = min(willingness, opening available coins)
```

The crop premium is its marginal benefit over alternative work, not gross harvest
value. Remaining inputs, work, rights and storage constrain both projections.
Lack of productive capacity can reduce the premium to zero. Land's longer-term
value remains an explicit assumption rather than a forecast of indefinite harvests.

The seller accepts the bid amount if it meets the minimum. There is no bargaining,
second-price rule or competition between buyers. Buyer cash limits bind before
settlement, and each receipt records both forecasts, the premium, willingness,
available cash and actual bid.

Work forecasts exclude future sale attempts and unseen fixture shocks. In this
fixture, the lender remains the agent using the monthly remaining-value work
policy. The buyer uses that evaluator for bidding and the ordinary continuing-work
policy to finish an acquired crop. The alternative-work projections used to price
its opportunity cost are not automatically installed as its future monthly plans.

## CPU results

The plot was purchased for 100 coins with 20 down. Default and repossession occur
in month three at 81.60 debt. The lender maintains the crop that month. At month
four's sale attempt, harvest requires two labor units and produces eight grain
and one seed. The buyer values those stocks at one coin each. It compares 21 coins
of projected output value with the asset against 16 without it, over four months.
The five-coin premium plus 90-coin land value gives 95 willingness to pay. The
contractual minimum is 75 coins.

| Six-month control | Buyer initial coins | Sale price | Borrower surplus | Ending debt | State |
| --- | ---: | ---: | ---: | ---: | --- |
| Funded buyer | 100 | 95 | 13.40 | 0 | Repaid; buyer owns plot |
| Cash-limited but acceptable bid | 80 | 80 | 0 | 1.60 | Deficiency; buyer owns plot |
| Bid below minimum | 70 | No sale | 0 | 81.60 | Pending; lender controls plot |
| No buyer | — | No sale | 0 | 81.60 | Pending; lender controls plot |

Sold crops complete for the buyer: eight grain and one seed. Unsold crops complete
for the lender under the selected work policy. A later-funding control retries in
month five, after that harvest, and sells the bare plot for 90, returning 8.40 to
the borrower. Already harvested goods stay with the lender; they are not part of
the later asset sale.

That last behavior is an explicit limitation of the current proceeds rule:
maintenance costs are borne by the controller and interim outputs belong to it;
neither is included in loan recovery. Applying interim income or reimbursing costs
would need separate terms. Sale fees, multiple liens, write-offs, custody expiry,
redemption, reserve-price adaptation and an auction remain unimplemented.

## Verification

Run from this directory:

```sh
cargo +1.92.0 run --locked --example resale
cargo +1.92.0 test --locked --test resale
```

Seven resale tests cover delayed realization, custody accounting, surplus and
deficiency, cash and valuation limits, missing buyers, retries, unchanged fixed
settlement, crop transfers, forged receipts and repeated settlement. CPU and
reference results agree across monthly, batched, reordered and checkpoint-resumed
execution. Unseen future capacity shocks do not change a current bid. Cash is
conserved through completed sales.

Resale plus work-choice, collateral-crop, credit, finance, economics and conditions
regressions passed **51 tests**. All-target Clippy passed with warnings denied.
Generated results and logs remain under ignored `output/economics/`.
