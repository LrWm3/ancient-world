# Funded inventory liquidation

Authorized recovery proceedings can now list indivisible stock lots alongside
land and portable equipment. This extends the existing estate, credit and
transaction books; it introduces no separate liquidation runtime.

`World.recovery.inventory_listings` names an authorized proceeding, commodity,
quantity and minimum price. Dated `inventory_bids` identify buyer, lot and actual
coin price. At Acquire, physical asset bids run first, then inventory bids ordered
by lot, descending price and stable bid ID. Both consume the same opening funding
budget. This is an explicit allocation order; it does not move monthly work.

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
books remain separate from member books. Active member purchases are currently
rejected because this adapter has not yet reserved and collected their mandatory
household contribution across later financing and market actions. This is a
bounded compatibility restriction, not an exemption from household pooling.

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
checks, not a sum of unique tests. Autonomous listings, appraisal, member pooling,
receivable assignment and multiple custody denominations remain separate work.
