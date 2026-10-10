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

## 2. Commitment-aware public purchases

Independent review identified asymmetric protection: public sales retained seller
claims but treated the buyer's entire opening coin balance as discretionary.
The same opt-in adapter now protects accepted claims and unpaid entry inputs in
its claim horizon before sizing affordable purchases. This is an explicit,
conservative commitment-first purchase policy; it is not a universal rule that
debt should always precede food. Unaccrued future loan interest is not covered by
`claims()`, which uses current loan dues.

`Plan.purchases` records requested lots, shared opening cash, protected cash,
affordable/submitted lots and matched lots. Actual boundary deals and settlement
remain separate evidence of payment. The matched control holds food, prices and
needs constant: three coins buy one food lot without a claim; a three-coin future
delivery prevents that purchase; six opening coins permit both. The accepted
claim actually settles in month four. Keeping only enough money for the claim
therefore produces three food deficits in months two through four, which the test
asserts rather than treating conservation as good economic balance.

A separate due-boundary case with six coins pays the three-coin claim and buys
food at that same Acquire boundary using remaining opening cash. Its receipt shows zero remaining
protected claim, guarding against subtracting a collected debt twice. Validation:
29 tests across public sales, orders and provisioning passed; the expanded
seven-test public-sale target passed again. Strict all-target Clippy, formatting
and artifact checks passed; independent review found no blocker.

## 3. Whole-lot demand for small needs

Independent review confirmed that floor division silently removed positive buying
needs smaller than a market lot. Fixed and public-sale buyers now use widened
ceiling division of the unmet target. Sellers still round down; no fractional
inventory or cash is invented. Quote ceilings, opening cash, claims and actual
shared storage admission still constrain fills. Buying one whole lot may exceed
the target; stored remainder reduces subsequent demand.

New controls cover a one-month nutrition target buying a three-grain lot and
actually consuming food on CPU; partial and sufficient holdings; multi-lot gaps;
zero authorization; insufficient cash/storage; and maximum integer target without
overflow. The prior cash-commitment comparison now uses a three-month target (one
exact lot), keeping that policy comparison independent of the rounding correction.

Validation: 34 discovery/order/provisioning regressions and all ten public-sale
controls passed. Earlier replenishment changed when the nine-coin buyer trades,
so a separate due-boundary fixture now isolates collection-plus-purchase behavior.
Strict all-target Clippy, formatting and artifact checks passed. Independent
review found no blocker; next is read-only observer coverage of these decisions.

## 4. Observe supply and purchase decisions

Independent review found that the new internal receipts were missing from useful
buyer-filtered logs. The existing read-only observer now emits `public_purchase`
for a selected buyer or issuer, showing requested, cash-protected, affordable,
submitted, matched and actually settled lots. Settlement counts come from accepted
boundary receipts, not a projection or a quote.

Planning observers export `discovered_supply` only for new decisions committed
by the observed Open. Selected detail includes the chosen quantity and baseline;
Alternatives adds attempted lots, conditional proceeds, losses and errors.
Large integer monetary/loss values are decimal strings. Capturing the pre-step
history length avoids exporting old decisions when attaching or resuming an
observer. No callbacks run inside private forecasts.

Controls compare observed and unobserved CPU worlds, state, ledgers and audited
books; verify buyer-only filtering, separate settlement reconciliation, continuation
without backfill, detail levels, month/log limits and absence of fabricated
supply records after a failed Open.

Validation: 34 public-sale, fixed-order and telemetry tests passed, including the
CPU accounting comparison. Strict all-target Clippy, formatting and repository
artifact checks passed. Independent production review found no blocker. Next:
combine discovered paid labor with later need-driven public food purchases in one
finite circulation control, instead of assuming the separate adapters compose.

## 5. Discovered wages-to-food circulation

Independent review selected the missing integration proof: separate paid-worker
and funded-food-buyer tests did not show that discovered wages could fund later
consumption. `discovery::scenario::circulation()` and the `discovered_economy
--circulation` CPU example now exercise that path. There is no new allocator or
scheduler. Initial operational agreements, counterparties, quotes, membership,
governor choices, process starts and dates remain absent.

The fourteen-month fixture has six public coins, sixteen public wheat, a four-wheat
reserve, and a worker with zero coins and three wheat. The supplier retains six
coins and ten metal. Only the worker can fill the two-hour mint labor lot: the other
two persons have one hour each. Their nutrition requirements remain active. Land,
household and finance discovery are disabled to isolate circulation; there is no
agricultural replenishment and no mortality rule. These are explicit experimental
endowments and constraints, not recommended balance or a whole-economy success case.

The supplier buys six wheat in month two. The worker earns four coins through a
settled labor sale in month three and buys three wheat for three coins in month
four. Purchased food is actually consumed. Its six total fulfilled nutrition units
reconcile with opening food, accepted purchases and ending inventory. Coins
reconcile with wage receipts and food payments, and the mint really completes.
CubeCL CPU and reference state, ledger, reports and audited books agree; rebuilding
at every phase includes the boundary between earning and buying.

| Fourteen-month case | Worker coins earned / spent on food | Completed mints | Ending public wheat | Nutrition deficits: state / supplier / grower / worker |
| --- | --- | --- | --- | --- |
| Public sales, 16 opening wheat | 4 / 3 | 1 | 7 | 0 / 8 / 14 / 8 |
| Legacy mint-funding sales | 8 / 0 | 2 | 16 | 0 / 14 / 14 / 11 |
| Public sales, 10 opening wheat | 4 / 0 | 1 | 4 | 0 / 8 / 14 / 11 |
| Public sales, reserve raised to 16 | 8 / 0 | 2 | 16 | 0 / 14 / 14 / 11 |

The first probe used ten wheat and failed to connect wages to food: supplier bids
bought all six saleable units before the worker earned anything. That result is
preserved as a scarcity control, with assertions showing a funded worker submits
bids that cannot match. The sixteen-wheat comparison changes only the physical
endowment. Raising the reserve preserves the initial wage opportunity but prevents
food sales. `public_sales=false` restores the legacy funding-sale policy; it is
not a universal ban on food sales, although this particular run settles none.

Even the positive case is temporary. The state meets its coin reserve objective
and stops requesting labor; the worker's remaining one coin cannot buy another
three-coin lot. Tests show remaining public wheat alongside unaffordable unmet
worker demand. Thus extra minting is not automatically helpful, and enabling sales
can reduce later wage opportunities under this state's supplied objective. State
needs are empty, so its zero deficit is not evidence of citizen welfare.

The observer coverage follow-up also exercises month/log limits, issuer-only
selection and planning-Off/settlement-On across an actual food-sale Acquire, beyond
the earlier failed-Open controls. All records still come from committed decisions
and accepted receipts.

Broader regressions exposed another bounded-forecast limitation after whole-lot
buying changed financial opportunities. The old fourteen-food paid-worker fixture
also permitted food forwards; it now sells one wheat in month five and another in
month eight, delivers in months seven and ten, then has food deficits in months
thirteen and fourteen despite twenty-one coins. The four-month proposal forecasts
do not see those late deficits. Independent review traced the stock reductions to
actual deliveries. The full-finance outcome is retained in a new regression; the
covered-labor control omits forward valuations while retaining loan discovery,
making its promised food cover explicit. No production policy was changed to hide
the shortfall.

Next review should distinguish recurring income, affordable food supply and public
objectives when restoring productive households to this circulation loop. It should
retain scarcity controls and every participant's outcomes rather than select only a
successful buyer. Anticipating mutually sustainable future work and food demand,
endogenous pricing, and broader self-starting institutions remain outstanding.

Validation: 69 targeted tests across discovered circulation/supply, discovery,
public sales, mint orders/provisioning and telemetry passed. The ten supply tests
passed again after narrowing the covered-labor isolation to forward valuations
only. These include CPU/reference and audited continuation comparisons; they are
not a new full-suite certification. Strict all-target Clippy, formatting, diff
and repository artifact checks passed. Independent review found no blocker.
