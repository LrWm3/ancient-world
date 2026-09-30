# Funded inventory liquidation

Authorized recovery proceedings can now list indivisible stock lots alongside
land and portable equipment. This extends the existing estate, credit and
transaction books; it introduces no separate liquidation runtime.

`World.recovery.inventory_listings` names an authorized proceeding, commodity,
quantity and minimum price. Dated `inventory_bids` identify buyer, lot and actual
coin price. At Acquire, physical asset bids run first, then inventory bids ordered
by lot, descending price and stable bid ID. Both consume the same opening funding
budget, and subsequent commodity advances retain their receiving-space reservations. This is an explicit allocation order; it does not move monthly work.

A sale requires an active proceeding, an unsold lot, permitted stock trading,
actual unreserved inventory, receiving space and fully funded payment. Explicit
current-essential exemptions also apply. An unfunded high bid leaves the lot
available to the next bidder. Multiple lots cannot spend the same grain, coins or
storage twice. Payment goes directly to estate custody; the debtor cannot spend
it and creditors receive it at the subsequent Due boundary. An unsold configured
lot blocks closure. The first adapter neither splits lots nor automatically
withdraws unavailable listings.

`recovery::inventory::discover` exposes available unsold lots. The common offer
interface adds `InventoryLiquidationBid` for supplied dated consent. Discovery is
not underwriting or a promise that a competing bid will clear. Preparing a named
bid previews the ordinary shared boundary; failed explicit acceptance publishes
nothing. Normal scheduled execution retains its independent successful bids.

## Household and accounting integration

Household agents can purchase inventory with their own funds and storage. Their
books remain separate from member books. Member purchases in credit-only, bilateral-market, town-market, physical-mint and legacy stock-market
Acquire drivers reserve their exact household contribution, including fractional
carry across lots, and pool it once after settlement. Subsequent commodity loans
also respect that reserved space; loan proceeds remain unpooled. Bilateral negotiation and town matching inherit those reservations and fractional carry. Direct forward performance now inherits the same contribution budget, as do later
bilateral, town, physical-mint and legacy stock trades. Legacy tool replacement
purchases during estate configurations remain unsupported; servicing existing
forwards and ordinary trading by other agents remain available.

The buyer records actual purchase cost. The debtor records sales revenue,
released inventory basis and restricted proceeds. Estate custody and later loan
payments use the existing reporting adapters. Tests inspect inventory, sales,
cost of sales and balanced statements; appraisals never create cash. Estate
proceeds currently must use the reporting denomination. Perishable lots and lots
in the custody currency are excluded.

## Verification

`tests/inventory_liquidation.rs` covers funded and unfunded bids, insufficient
space, competing lots sharing inventory/cash/storage, reordered catalogs,
essential exemptions, household purchases, exact financial statements, common
acceptance, duplicate sale prevention, tampered batches, CPU/reference agreement
and checkpoint continuation.

The affected seven-target gate passed 63 tests before the final exemption and
explicit accounting assertions; all five final inventory tests passed, as did strict all-target
Clippy. These gates are overlapping
checks, not a sum of unique tests. Autonomous listings, appraisal, member pooling across later markets,
receivable assignment and multiple custody denominations remain separate work.

The household contribution follow-up passed 35 tests across five affected targets,
then all eight final inventory checks and strict all-target Clippy. One-unit lots
exercise fractional carry and full pooled storage; a later household commodity
loan is rejected if only the raw, unpooled holdings would fit.

The bilateral-market adapter passed 22 tests across five affected targets and
strict all-target Clippy. An estate lot followed by a negotiated lot shares cash,
raw storage and exact household pooling reservations. Configured estate bids are
supplied consent; the negotiated buyer still follows the household purchasing
policy. These tests do not establish autonomous estate demand.

Household seller integration now compares retained debt, explicit discharge and
unfunded bids. Unsold lots block estate closure; a closed estate with residual debt
still blocks household dissolution. Only after claims clear can the existing
wind-down policy distribute remaining goods. Member cash remains private. The
three affected targets passed 35 tests, including CPU/reference, checkpoint and
separate financial statements; strict all-target Clippy passed.

The town-market follow-up passed 41 tests across four targets and strict all-target
Clippy. Need-generated demand observes already purchased inventory; settlement
respects the accumulated household share, preserving a submitted-but-storage-blocked
order when shared capacity is full.

Direct-delivery follow-up: actual deliveries pool after acceptance, using the same
fractional carry as prior estate purchases; full space leaves a recorded forward
shortfall. Both original-seller and guarantor deliveries now pool actual goods, with native
recourse only for completed guarantee payments. See [household forwards](HOUSEHOLD-FORWARDS.md).

The physical-mint integration in `tests/mint_finance.rs` checks actual issuance
before estate recovery, then two one-unit purchases with a single fractional
household contribution. Full shared storage rejects the later wheat trade while
preserving the earlier funded estate purchase and its custody proceeds.

Legacy stock-market integration retains fractional contribution reservations
across an estate purchase and multiple later lots. Ordinary buyer/seller trading
is stayed during active recovery; authorized liquidation remains a separate
route into custody. Custody agents cannot place ordinary bids or offer stock.
The affected six-target gate passed 93 tests and strict all-target Clippy.

A supplied seed bid now joins prerequisite and cultivation requests through common
acceptance. The purchaser plants from delivered seed, while custody proceeds
reduce the seller's debt only at the next recovery window. Funded/unfunded, altered
receipt, CPU and reconstructed-checkpoint controls pass with separate statements.
See `tests/recovery_search.rs`. The five-target gate passed 26 tests and strict
all-target Clippy.

The specialized mortgage stock-sale path now carries the same contribution
reservations after estate purchases and general advances. Previously a fractional
estate purchase followed by several stock lots could overfill collective storage
and reject the entire household commit. Identical opening shared stocks now allow
three, two or zero later lots in both legacy and specialized drivers. The scoped
specialized fixture declines its mortgage offer to isolate stock settlement.

Its bounded sale forecast caps candidates by actual contribution space before
rollout, preserving the seller's explicit nutrition constraint. The stock receipt
and settlement log expose `contribution_limit` separately from raw storage room.
It includes preceding eligibility, money, lot and raw-storage limits; it is not an
independent physical capacity. CPU/checkpoint execution, pooled quantities, actual
custody and separate books agree. No new receipt funds another outgoing action.

Earlier accepted native advances can also release physical storage for a later
stock purchase. Both specialized quoting and final stock-trade validation observe
that transfer prefix. Their spending budgets still use opening resources: newly
borrowed goods cannot be resold in the same boundary. Full-storage, outgoing-loan
and incoming-only controls retain household contribution carry and native debt,
with reference/CPU and reconstructed continuation agreement.
