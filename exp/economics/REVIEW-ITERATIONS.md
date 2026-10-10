# Independent review iterations

Follow-up to [discovered supply](DISCOVERED-SUPPLY.md), 2026-10-10. Each chunk is
selected by a read-only reviewer, implemented with focused controls, reviewed again
and committed separately. This record distinguishes bounded progress from the
longer ambitions in the goals and verification documents.

## 1. Public stock sales independent of mint funding

The reviewer found that an idle or funded issuer could withhold available food
because its sale order existed only to fund a future mint attempt. The opt-in
`discovery.public_sales` policy now derives buyers from eligible consumption needs
and a public reserve from the issuer's organizational reserve objectives. Ordinary
orders gain `StockSales { reserve, claim_months }` for reuse without discovery.

At Acquire, protect reserve **plus** accepted claims and unpaid process inputs,
using live balances after Due. Generate one public sale order capped by surplus
and funded demand. It replaces the legacy funding ask. Ordinary matching, storage,
shared opening budgets and journal settlement remain authoritative. Procurement
status (`Plan.reason`) and actual public sale deals can differ: idle minting need
not prevent a lawful food sale. Disabled/forbidden minting is independent of stock
trade permission. Prohibited consumption cannot generate discovered food demand.

The historical discovery control retains `public_sales = false`: spending public
food below a reserve to raise minting funds and protecting that reserve are
substantively different policies. The new controls enable it explicitly. This
option does not add a separately scripted set of buyers, dates or agreements.

`tests/public_sales.rs` covers an idle issuer selling two three-grain lots while
retaining four grain, recurring buyer consumption, CPU/reference books and phase
reconstruction; missing cash, trade/consumption permission or surplus; expired or
prohibited minting; delivery/input protection; competing buyers; and inability to
reuse current sale receipts or wages within Acquire. Existing discovery, supply,
fixed-order and provisioning controls remain regressions.

Limits: fixed reservation prices, finite starting food and stable buyer ordering
remain. This does not establish replenishment, wage-dependent supply planning or
long-run viability. It protects explicit reserves and claims rather than solving
an arbitrary seller objective tradeoff.

Validation: 49 tests across five targets passed; strict all-target Clippy,
formatting and repository artifact checks passed. Independent final review found
no blocker. Next: protect buyer commitments in the public-purchase budget.
