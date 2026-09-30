# Durable equipment in estate liquidation

Authorized proceedings can now list portable, usable equipment in addition to
catalog property. Discovery, supplied bid acceptance, funded priority, custody
and creditor distribution use the existing recovery path. There is no separate
equipment auction engine or debt ledger.

The debtor retains ownership until an actual funded sale at Acquire. Payment and
ownership publish atomically; the tool retains its kind, remaining uses and last
use marker. A tool attached to a site, exhausted, used exclusively this month, or
bound to an outstanding output-share delivery cannot use this adapter. Execution
rechecks eligibility after discovery. Estate custody does not operate equipment.

The purchaser records actual cash paid as its new historical cost. The seller
recognizes gain or loss against its remaining carrying amount. Later depreciation
uses the buyer's cost and the tool's existing remaining life. Sale proceeds enter
estate custody and become distributable at the following Due; appraisals and
unfunded bids never create cash.

A completed sale releases that asset from the original proceeding's disposal
restriction. The historical listing and sold receipt remain valid when its buyer
later retires the equipment. This allows a household to buy an estate tool, use
up its life, retire it and complete solvent dissolution without the old estate
listing acting as a permanent lien.

## Verification

`tests/equipment_liquidation.rs` exercises two ten-coin loan claims, a controlled
pre-book opening loss, a tool with eight months of life and an eight-coin buyer.
At month three the tool has five uses left. Sale realizes a three-coin gain over
its five-coin carrying value. Creditors subsequently receive four coins each;
explicit deficiency discharge closes the estate. The buyer depreciates its eight
coins over the remaining life. The household-buyer variant retires the exhausted
tool and dissolves with separate balanced statements.

CPU/reference, checkpoint continuation, common offer preparation, committed
replay and rejection of altered ownership/condition agree. Unfunded bids retain
the asset; newly exhausted or currently used tools fail execution without payment.
The recovery, household dissolution/disposal and common financial offer suites
also pass. Strict all-target Clippy passes. Logs stay under ignored `output/`.

## Boundaries

Inventories, receivable assignment, attached equipment packages and autonomous
estate listing/valuation remain extensions. Exhausted tools need an explicit
retirement/salvage disposition; this adapter does not invent a scrap price. It
also does not novate output-share contracts, add competing equipment liens or
make every asset of the debtor automatically available for liquidation.
