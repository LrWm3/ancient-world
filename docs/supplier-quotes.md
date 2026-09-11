# Cost-aware export procurement

Repeat suppliers now estimate replacement cost before accepting a new funded order. A quote includes recipe inputs at local prices, labor valued at 18 kg of prepared food per worker-month, and workshop upkeep. Raw extraction includes labor and a reserve allowance. These are opportunity-cost estimates, not wages, booked expenses, or new cash transfers.

The supplier asks for the greater of its observed delivery price and estimated cost plus the configured margin (10% by default). The buyer rejects prices above its current local willingness to pay, with the same base-price floor used by replacement estimates. Rejected quotes leave cash untouched. Accepted orders use the existing limited escrow, six-month expiry, physical production, freight and storage rules. Previously funded commitments retain their agreed price.

Recipe estimates respect local knowledge. A scrap recipe qualifies only when current scrap covers the proposed quantity, preventing a tiny scrap stock from pricing an entire virgin-material order. Unsold byproducts receive no speculative revenue credit. Existing finished stocks without a production route can be quoted for resale at local replacement value. Inputs remain subject to actual GPU consumption limits; the estimate does not reserve them against competing work.

`production.supplier_profitability` enables this rule in bundled new-world settings. `production.contract_margin` accepts 0–1. Older archived catalogs missing these settings retain the previous price behavior. The explorer's **Require profitable supplier quotes** control changes future negotiations; contract inspection shows estimated and agreed unit prices. `History::supplier_unit_cost` exposes a one-kg estimate.

`production_summary` reports supplier declines and quoted export surplus for retained relationships. The latter accumulates dispatched quantity times agreed price minus the accepted cost estimate: it is not realized profit, an asset, or additional money. Estimates can become stale during a commitment, and labor has no actual wage market yet. Stale relationships are removed, so these diagnostics are not lifetime totals.

## Verification

Fixtures exercise rejected and accepted quotes, margin and cash limits, expiry refunds, recipe knowledge, scarce scrap, changing input costs, old archive defaults, and invalid margin settings. Paired history controls use `--no-supplier-profitability`, keeping export contracts, workshops and adaptive staffing enabled.

[Specialized workshop types](specialized-workshops.md) and [household wages](household-economy.md) are now implemented. Private firms and realized firm profit-and-loss accounts remain future work. This increment prevents automatic acceptance of poorly priced procurement; it does not establish those systems or guarantee that every contract proves profitable.

## Three paired century runs

Seeds 17, 81 and 256 used terrain/ecology edge 64, living-world evolution, discoveries and scarce-inner-continent defaults on the Quadro RTX 5000 Max-Q. The control disables only supplier profitability. Arrows show control → cost-aware quotes.

| Seed | Population | Shortage site-years | Tool sufficiency | Funding episodes | Dry kg/person, quotes |
|---|---:|---:|---:|---:|---:|
| 17 | 5210 → 5295 | 9 → 6 | 99.3% → 98.9% | 9215 → 6566 | 10.43 |
| 81 | 5692 → 5718 | 5 → 5 | 99.0% → 98.8% | 13570 → 6331 | 11.07 |
| 256 | 5600 → 5607 | 9 → 7 | 99.1% → 98.1% | 10725 → 7181 | 10.46 |

Retained relationships recorded 921/3353/897 declined quotes and 31.2/13.5/19.2 tonnes dispatched respectively. These are retained-record diagnostics, whereas funding episodes count recorded events including renewals. Year-100 installed workshop units were 2.80/12.23/2.04, still close to the control. Dry goods per person changed from 10.43/10.98/10.36 at year 50 to the table's year-100 values, with no renewed runaway dry stock growth in these runs.

Quotes reduced funding churn substantially. Populations rose slightly and shortages improved or stayed unchanged, while tool sufficiency declined modestly. This supports retaining the procurement constraint; it does not demonstrate stronger specialization or universally better outcomes. Whole-history trajectories can diverge, so demographic differences are not isolated causal measurements.

Maximum observed relative accounting residual remained below 2.20e-5. Enabled runs took 53–69 seconds with concurrent workloads; these are not isolated benchmarks. Social processing and validation remained the largest recorded history stage at 18–23 seconds. Larger worlds and multi-century profitability behavior were not calibrated here.

All 99 tests passed, including GPU tests. All six contract fixtures passed again after strengthening the quantity-aware scrap assertion. Clippy with warnings denied and formatting checks passed. A mature seed-17 checkpoint continued for 24 months identically in one batch and in monthly steps with a save/reload at month 12; history, terrain and ecology matched exactly.

Artifacts: `output/supplier-quotes.json`, `output/supplier-quote-control.json`, `output/supplier-quotes-analysis.md`, saved worlds, and `output/supplier-quotes-replay.json`.
