# Inventory-driven replacement purchases

This opt-in follow-up removes the recurring ore-forward schedule from the
[48-month acceptance scenario](ACCEPTANCE-READINESS.md). The original forward-based
baseline remains unchanged. Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example acceptance -- --spot
cargo +1.92.0 run --locked --release --example acceptance -- --spot --continuation-matrix
cargo +1.92.0 test --locked --release --test peer_stock_orders --test acceptance_continuation
```

The workshop bids for enough whole lots to restore four ore. The town asks two
coins per two-ore lot from available inventory, with a zero sale floor. Its
production target remains four ore: confusing that target with a sale reserve
would prevent sustained sales. Quotes and targets are supplied policies, but dated
ore orders and trading counterparties are no longer supplied contracts. Orders
stop when the buyer is stocked and return as production consumes ore. Only
accepted marketplace fills count as replacement deliveries.

## Shared execution and protection

`minting::orders::Policy.peer_stocks` adds stock quotes to the existing venue and
Acquire resource pool. It uses ordinary market eligibility, transactions, storage,
settlement and accounting. The town now needs both legal stock-trade permission
and venue admission. The state is not a counterparty to these ore trades.

Each Acquire observes current holdings after earlier financial admissions, uses
remaining opening resources for reservations, then settles accepted exchanges.
Incoming sale proceeds do not fund another purchase in the same boundary.
Production sees settled inputs later in the month; newly produced ore becomes
available for sale at the next Acquire. The scheduler is unchanged.

Buyers protect accepted claims; household and personal protection also uses the
existing needs adapter. Sellers protect the greater of their stock floor and
existing protected stock. Earned wage arrears are claims; undelivered future work
does not automatically become a protected liability. Cash protection is shared
across all of a buyer's bids: clearing preserves the maximum applicable reserve,
even when food and ore policies have different horizons. It does not subtract the
same reserve once per order. Individual affordable/submitted quantities remain
distinct from shared-budget matches and actual settled quantities.

Standalone stock sales clear in stable market/ask/agent order before the issuer's
conditional input package. A failed mint input package retains successful peer
trades. Peer orders continue when no mint work is scheduled. This is explicit
priority, not a claim of fair allocation under scarcity. Purchases and observer
records identify their actual market rather than assuming every buyer buys food.

The extension currently accepts only nonissuer participants on additional stock
markets, with distinct resource listings separate from the funding and mint-input
markets. It rejects capacity goods, duplicate agent/market policies, invalid lots,
prices and horizons, and combination with the alternative provisioning driver.
This bounded adapter is not a replacement universal exchange or ZIP implementation.

## Acceptance and disruption

Measured CPU results on 2026-10-11:

| Spot case | Ready | Food deficits | Final wage arrears | Final-year workshop / town income |
| --- | --- | ---: | ---: | ---: |
| Baseline | Yes | 0 | 0 | 12 / 13 |
| No ore production | No | 43 | 2 | 0 / 0 |
| No replacement bids | No | 43 | 2 | 0 / 0 |
| Workshop excluded from venue | No | 67 | 2 | 0 / 0 |
| Metal lot price reduced to two | No | 2 | 2 | 0 / 0 |
| Startup payroll one coin short | No | 0 | 0 | 12 / 13 |

The baseline retains 46 mint batches issuing 460 coins, 47 jobs per organization,
48 wage coins per employer and nine reconciled books. Each of its three post-startup
years earns twelve workshop and thirteen municipal income units. Closing workshop
and town cash is 52 and 57, versus 50 and 59 under forwards. The two-coin difference
is consistent with eliminating the final outstanding ore prepayment; it is not a
claim of greater profitability. Eleven grower storage rejections remain without
nutrition deficits.

The spot baseline must pass exactly the existing recurring acceptance gates. Tests
compare reference and CubeCL CPU, reconstruct Simulation after every phase, replay
with dated World contexts, and reverse independent catalogs including peer quotes.
The original forward and new spot variants retain their failure controls for supply,
missing purchases, venue access, margin and historical late wages.

A separate disruption disables town ore extraction in months 13–15 and restores it
in month 16. The spot economy resumes replenishment without an injected grant,
replacement contract schedule or changed quote. Its final two operating years each
return to twelve jobs per organization, twelve metal sales and 24 delivered ore.
Across 48 months all people remain fed, wages are paid, and all nine books reconcile.
There are 45 mint batches, 45 workshop jobs and 44 extraction jobs. The interrupted
year records ten workshop jobs, nine extraction jobs and 18 delivered ore; it
correctly fails the uninterrupted acceptance gate despite subsequent recovery.

This is recovery from a temporary supplied process-availability change. It does
not prove adaptive pricing, discovery of a substitute supplier, governance-selected
employment cancellation, or survival after an arbitrary permanent disruption.
Employment still pays for delivered hours when productive work is unavailable.

## Remaining boundaries

Initial agents, consent, employment terms, stock targets, fixed prices and the
issuer reserve objective remain supplied. Ore deposits are not finite. Membership,
rights, employment and financial contracts are not all traded through this book.
The household's original forward remains in force; only recurring ore forwards are
removed in this variant. No personal self-directed policy changes are introduced.

## Verification record

Seven peer-order tests cover inventory targets, eligibility, accepted payroll
claims, cross-market cash protection, storage, no reuse of sale proceeds, failed
mint packages, invalid policy combinations, altered receipts and market-specific
observer output. Six recurring-acceptance tests cover both variants and their
controls, phase reconstruction, dated replay, catalog reordering and recovery.
The ten related release regression targets pass 101 tests: `mint_orders`, `minting`,
`mint_cycles`, `mint_provision`, `public_sales`, `discovered_private_sales`,
`discovered_circulation`, `acceptance`, `telemetry` and `mint_finance`.

This batch totals **114 passing tests**. Strict all-target release Clippy, formatting,
diff and repository artifact checks pass. The full crate suite was not rerun for
this batch; the earlier full-suite result remains recorded with the forward baseline.

Generated results and logs remain under ignored `output/`.
