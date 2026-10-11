# Sale-linked production acceptance

This ten-iteration follow-up tests whether signed sale opportunities can replace
inventory objectives as the reason to discover productive work. It builds on
[bounded peer expectations](ACCEPTANCE-PEER-EXPECTATIONS.md). Initial identities,
consent, prices, employment and physical endowments remain supplied.

## Iteration record

1. **Signed issuer-input sales.** The existing `stock_markets` authority and
   `PeerStockQuotes` program now also support stock sales into issuer-input markets.
   They route to the existing mint counterparty book; independent stocks retain
   the peer book. Only signed issuer-input slots are replaced, including explicit
   stops; unrelated legacy quotes and other agents remain unchanged. As before,
   the actor's complete peer portfolio is replaced. Capacity sales and input-market
   buys are excluded. Two atomic validation/routing tests and nine existing
   authority, discovery and execution tests pass. This adapter still requires an
   enabled peer-stock book; it is not universal market authority.

2. **Dated execution and access.** A signed combined ore-purchase/metal-sale
   program is selected through ordinary forecasts, becomes effective at the next
   Open, and produces real sales on both reference and CPU. A matched legal-access
   revocation suppresses sales while retaining accepted history, installed quotes
   and employment obligations. In-memory reconstruction preserves the boundary.

3. **Sale-linked productive discovery.** Signed stock-sale options whose payment
   could meet an organizational cash-reserve or funding-gap objective now root
   productive alternatives. A target is the signed floor plus the authorized lot
   count times listing quantity, using checked arithmetic. Only signed productive
   recipes can supply it. Explicit inventory objectives retain their original
   targets; otherwise recurring work replaces the one-shot candidate and combines
   with existing quote portfolios. Catalogs depend on static options, never live
   balances or quotes. Two new tests and five discovery/execution regressions pass.
   A target is a persistent stock buffer, not a promise of demand or a rule that
   automatically stops work when the cash goal is reached.

4. **Cash-only cold-start control.** A new fixture removes workshop/town
   inventory objectives and the initial workshop metal ask, while retaining
   signed price options and finite endowments. Its 48-month reference run fails:
   zero mints and productive batches, 67 food deficits and ending wage arrears 2.
   All first workshop cash projections end at zero, so the normal retain-on-tie
   rule selects no program. Sale-linked candidates exist, but observed-recurrence
   expectations have no successful demand history to repeat. This is an explicit
   negative control, not a passing acceptance claim.

5. **Bounded catalog and compatibility controls.** Unsigned productive recipes
   cannot become sale candidates. Oversized signed production catalogs reject
   initial installation atomically instead of silently truncating alternatives.
   Explicit inventory objectives retain their original target rather than taking
   the derived sale buffer. Four sale-discovery and two joint-authority tests pass,
   including arithmetic overflow and historical catalog stability.

6. **Finite startup comparison.** Monthly review and an initial metal ask alone
   yield one mint and no recurring work. Four opening metal units yield four
   mints and two workshop batches, but no town production: the chain still stops.
   Supplying four opening ore units and an initial signed town ask as well yields
   44 mints, 43 workshop batches and 40 town batches over 48 months, with no food
   deficits or wage arrears. The unchanged governed readiness check passes all
   three post-startup years. These are finite opening inventories and posted
   quotes, not supplied work targets or future purchase decisions. The cash-only
   cold fixture retains annual reviews; all posted-startup comparisons use monthly
   reviews, with workshop/town forecast horizons still 12/4 months.

7. **Sale provenance in decision observations.** Discovered decisions record the
   signed market/recipe/lot terms behind productive alternatives. Each private
   projection records only accepted market sales, with month, buyer, resource,
   quantity, payment and value. Existing organization observers serialize these
   records without injecting telemetry into economic execution. A filtered month-5
   observer pins the two-metal/four-coin sale tuple, hypothetical peer work/ballots,
   forecast dates and identical live world/state/ledger/accounting. That test and
   eleven existing expectation/telemetry controls pass. These are private forecast
   receipts from the current mint/peer market batches, not promised live income or
   coverage of every financial arrangement.
