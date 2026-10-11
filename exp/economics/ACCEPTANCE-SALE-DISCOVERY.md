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
