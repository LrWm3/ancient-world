# Funded repeat export contracts

Repeat customers can now commission production beyond a supplier's local reserves. This is a sparse CPU procurement system connected to the existing GPU orders, adaptive staffing and workshop capital. It does not grant output bonuses or bypass recipes, source reserves, storage or transport.

A buyer/good relationship needs two successful deliveries from the same supplier before it qualifies. One-off sales, dispatched cargo that never arrives, and food shipments do not establish industrial contracts. At most one relationship is tracked per buyer/good, and stale evidence expires after two years. Supplier selection can restart after that expiry.

At a quarterly market, a reachable supplier can receive a six-month commitment. The quantity is bounded by the buyer's current unmet target, 125% of its observed shipment size, two kg per resident, and available funding. Across goods, new commitments can reserve at most 20% of the buyer's currently liquid cash that quarter. A buyer without six months of food reserves does not commission new exports.

Funding transfers actual cash into archived escrow. The producer receives no cash until dispatch. Expiry, abandonment, market closure or disabling contracts refunds unused escrow to the buyer once. A broken route prevents dispatch; reserved funds return on expiry if the route does not recover. These are limited procurement commitments, not credit or loans.

Funded quantities become supplier work orders. Buyers treat pending procurement as expected supply when planning local recipes, reducing duplicate work; it is never entered as physical stock. Export additions are tracked separately from the town's local reserve target. A supplier therefore neither buys its promised finished exports from other towns to satisfy its own reserves nor withholds that entire export quantity as a protected local reserve. Ordinary ingredients may still be imported.

The market prefers the contracted supplier when it has eligible stock. Contract pricing starts from the observed delivery price; the subsequent [supplier quote increment](supplier-quotes.md) adds replacement-cost estimates, negotiation and rejection for new worlds. Dispatch consumes actual goods, checks current routes and freight capacity, respects storage and buyer demand, and pays the seller from escrow before using additional liquid buyer cash. If a different supplier satisfies the need first, the unused commitment can expire; no forced surplus purchase occurs. Standard cargo then handles delivery, delay and loss. Arrival never pays the seller a second time.

## Controls and inspection

New worlds enable `production.export_contracts`. Missing settings and relationship fields in older archives default to disabled/empty. The explorer offers **Fund repeat export contracts** and lists funded relationships in town production inspection, including remaining quantity, counterparties, escrow and expiry. Existing `configure_economy` and archived monthly checkpoints carry the same state.

`History::export_contracts` is the read-only inspection data source for clients. `production_summary` includes active funded count, escrow, and dispatched quantities for retained relationships. The latter is not a lifetime total because stale relationships are removed. Funding/refund events identify the towns; physical shipments remain ordinary market events.

The evaluator's `--no-export-contracts` control retains workshops, demand orders and adaptive staffing. Annual settlement roles still derive from actual activity; contracts do not directly assign a town a manufacturing role.

## Validation

Focused fixtures require repeated completed delivery evidence, reject inaccessible suppliers, prevent double funding, test expiry and disabling refunds, check payment and physical cargo accounting, and keep export targets out of local import demand. The full suite covers ecological conservation, history and checkpoint continuation.

```sh
cargo test --tests --no-fail-fast -- --include-ignored --test-threads=1
cargo run --release --example history_evaluate -- --seeds 17,81,256 --years 100 --discoveries --living-world --save-worlds --output output/export-contracts
cargo run --release --example history_evaluate -- --seeds 17,81,256 --years 100 --discoveries --living-world --save-worlds --no-export-contracts --output output/export-control
```

This increment introduces repeat export customers. Specialized workshop types, private firms, wage competition, credit, paid carriers, and long-distance enforceable contracts remain outside it. Industrial specialization must be assessed from actual deliveries and investment, rather than assumed from the existence of a contract record.

## Three paired century runs

Quadro RTX 5000 Max-Q, terrain/ecology edge 64, seeds 17/81/256, living environment and discoveries, scarce-inner-continent defaults. The control disables only export contracts; workshop capital and adaptive staffing remain enabled.

| Seed | Population control → contracts | Shortage site-years control → contracts | Tool sufficiency control → contracts | Dry kg/person | Installed workshop units |
|---|---:|---:|---:|---:|---:|
| 17 | 5334 → 5210 | 5 → 9 | 97.7% → 99.3% | 10.54 | 2.82 |
| 81 | 5729 → 5692 | 5 → 5 | 98.8% → 99.0% | 10.96 | 11.97 |
| 256 | 5643 → 5600 | 7 → 9 | 98.6% → 99.1% | 10.37 | 2.09 |

| Seed | Funding episodes | Refund episodes | Retained relationships: dispatched tonnes | Year-100 funded agreements | Year-100 escrow |
|---|---:|---:|---:|---:|---:|
| 17 | 9215 | 860 | 36.3 | 6 | 16.2 |
| 81 | 13570 | 2040 | 42.7 | 14 | 945.9 |
| 256 | 10725 | 1079 | 28.1 | 5 | 19.8 |

Funding episodes include renewals, not unique firms. Dispatched tonnes cover retained relationships and exclude expired history; they are not lifetime export totals. Most funded quantities are small replenishment purchases. Many agreements dispatch immediately from existing stocks, while shortages leave orders for later production.

The feature supports real procurement and slightly higher tool sufficiency in these comparisons, but populations fell slightly and shortages worsened in two seeds. Workshop investment changed little from the controls (2.84, 12.37 and 2.15 units). This is not evidence of strong industrial specialization. Specialized facilities and supplier economics remain necessary before claiming that outcome; no abundance or productivity multiplier was introduced to force it.

Maximum observed relative accounting residual stayed below 2.20e-5. Dry inventories per resident remained close to their year-50 values. Enabled histories took 66–71 seconds each with concurrent workloads; these are observations, not isolated performance benchmarks. Social processing/validation was the largest recorded history stage (22–25 seconds per run). Larger resolutions and multi-century contract performance were not calibrated here.

Artifacts: `output/export-contracts.json`, `output/export-control.json`, `output/export-contracts-analysis.md` and their saved worlds. These are whole-history comparisons; production and event changes can alter later political and demographic trajectories.

Mature replay passed from the enabled seed-17 year-100 archive: 24 months in one batch exactly matched monthly advancement with a checkpoint at month 12. History, terrain and ecology matched; report: `output/export-contracts-replay.json`.

Final verification: all 96 tests passed, including hardware GPU tests; the strengthened supplier/buyer order fixture also passed on rerun. Clippy with warnings denied and formatting checks passed.
