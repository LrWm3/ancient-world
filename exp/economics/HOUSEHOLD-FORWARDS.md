# Direct prepaid deliveries and household funding

Implemented integration pass 30, September 2026. Households can now be either party
to an explicitly consented prepaid commodity delivery, without pretending to be a
person buying a tool. Direct terms, the original tool-financing adapter, household
funding and financial reporting use the **same forward book and delivery executor**.

## Admission and consequences

`World.prepaid_deliveries` contains dated `forward::direct::Terms`: identity, seller,
buyer, acceptance month, delivery month, goods and prepayment. These records supply
both parties' consent. They are not discovered offers or a forecast of feasible
production. The buyer prepays the seller; the seller then owes the goods.

At Acquire, admission checks:

- Distinct known agents, positive quantities, a later delivery date, distinct stock
  and payment resources, and a payment resource requiring no physical storage.
- Both parties' `StockTrade` permission and recognition of `PrepaidDelivery`.
  Household parties must be active and cannot be winding down or dissolved.
- Actual remaining opening funds. Incoming loan proceeds, other prepayments or spot
  receipts cannot finance another action at this same boundary.
- Prospective receiver storage, including outstanding promised receipts and shared
  household storage. This conservative admission check does not escrow capacity or
  guarantee storage will still exist on the delivery date.
- No other outstanding forward delivery by this seller. This deliberately retains
  the pilot's one-outstanding-advance restriction rather than adding underwriting.

Rejections retain a reason and create neither payment nor claim. Accepted terms
create the existing `forward::Contract`, observable through the common agreement
view. Missing stock or receiver space leaves an outstanding delivery and its
prepayment/deferred-revenue positions intact. It blocks a new advance; it does not
become a cash loan, fictitious delivery, automatic refund or automatic write-off.
Withdrawing permission or legal recognition does not cancel accepted performance.

## Funding and the monthly boundary

`Charter.fund_forward_deliveries` defaults to false. When enabled, collective
need-generated orders can buy missing goods for the household's **own current**
forward deliveries. Existing signed member surplus mandates can also fund these
shortfalls under `accept_payment_support`. Consent, member reserves and claims,
needs, storage and the existing collective purchasing policy still apply.

The common current-forward reader subtracts actual deliveries and accepted relief
and respects the effective due date. Future promises remain in commitment
projections but do not create current funding demand. Forward, rent, wage and loan
requirements add within each resource; actual holdings offset the combined gap once.
No new debt book, substitute denomination or member liability assumption is added.

The shared Acquire allocation order is explicit:

1. Existing credit acquisition reservations.
2. Direct forward delivery, then dated prepayment admissions in stable contract-ID
   order; delivery retains existing claim rank, due date and ID ordering.
3. Town or bilateral spot exchange against the remaining opening budget.

Later order generation sees the updated delivery/prepayment book and projected
holdings, while spendable funds exclude incoming transfers. This distinction avoids
both duplicate funding of a settled bill and same-batch cash reuse.

Forward performance still occurs at Acquire, before that month's market purchase
and before Productive support/work. Goods acquired afterward are therefore delivered
at next month's Acquire. Land still retries at ClearArrears, wages at Close and
loans at Due. Plain scenarios with direct terms now enter the existing Acquire
boundary; no new phase or reordered production is introduced.

## Separate financial statements

Prepayment records operating cash flow for both parties, a buyer prepayment asset
and seller deferred revenue. It is not immediate sales revenue or pooled member
income. Actual delivery releases historical prepayment cost proportionally, records
sales/cost of sales and transfers inventory through the existing accounting adapter.
Actual direct deliveries to members now contribute half of received stock to their
household, with exact fractional carry and storage reserved before payment. The
prepayment itself remains unpooled. Partial delivery keeps the residual asset and
liability. Voluntary member support
remains a separate transfer expense/income until the household delivers the goods.
Reporting currently requires the advance in the journal's reporting currency, as
with the existing forward adapter; no FX conversion or synthetic valuation is added.

## Combined evidence

`tests/household_forwards.rs` demonstrates:

| Control | Observed result |
| --- | --- |
| Four coins prepaid for four grain due in month 3; household funding enabled | No speculative purchase in months 1–2; buys four grain for two coins in month 3; delivers in month 4 and retains two coins |
| Same terms, funding disabled | Four-grain claim and four-coin deferred revenue remain; no invented fulfillment |
| Signed member grain support | Donates four only when due, protects private reserves and makes no second donation after settlement |
| Four rent plus four forward grain due together | An explicitly configured eight-grain market lot funds both; ClearArrears pays rent, next Acquire delivers the remaining four |
| Receiver storage shrinks from sufficient room to two | Delivers two; retains two goods and residual claims without another purchase; restored room permits completion |
| A three-coin loan competes with a four-coin prepayment from a four-coin buyer | Credit reserves three; forward admission rejects the remaining one-coin budget |
| Incoming prepayment could finance another contract | Second acceptance rejects rather than spending incoming coins |
| Household buyer with insufficient shared storage | Admission rejects even without an explicit raw household capacity; sufficient shared room permits acceptance |
| Permission withdrawn after acceptance | Existing delivery completes, later admission rejects |
| Altered receipt, malformed terms or altered historical advance | Rejected without publishing an unauthorized contract or transfer |

Eight execution tests compare CPU/reference outcomes; a ninth checks malformed
configuration and history. They exercise reconciled statements, ledger replay,
checkpoint continuation, reversed
member/trader/term tables, law controls and finite storage/money. The rent comparison
changes only its fixture lot size: the town book still permits one whole-lot order
per registered participant per month.

The selected regression and final focused runs passed **153 distinct integration
tests across 17 suites**, including the nine new cases. Strict all-target Clippy,
formatting and the repository artifact check passed. The full crate suite was not
run.

Run from this directory: `cargo +1.92.0 test --locked --test household_forwards`.
Generated logs stay under ignored `output/economics/forward-funding-*.log`.

## Remaining scope

Direct admission composes with plain, town and bilateral acquisition, direct loans,
accepted land bills and the existing household execution loop. Tool-underwritten
and direct admission configurations cannot yet coexist in one world, despite
sharing records, inspection, performance and accounting. Mortgage configuration,
recovery proceedings, minting, search acquisition and joint production remain
explicitly excluded from this new admission path. The old forward recovery and
relief scenarios retain their existing support.

Discovery, negotiated forward pricing, production underwriting, multiple concurrent
seller advances and default/refund negotiation remain extensions. Household funding covers its own promises; no new member-forward
assistance policy is introduced. Bounded [labor-offer acceptance](HOUSEHOLD-HIRING-OFFERS.md)
is now implemented; longer-horizon employer viability remains later work. Person
self-directed policy changes remain deferred.

## Direct-forward recovery follow-up (Fibonacci batch 1)

Direct prepaid deliveries now use the existing authorized recovery proceeding,
including household wind-down, native delivery claims, deadline extensions and
explicit write-offs. Partial performance remains real delivery; waived quantities
recognize creditor loss/debtor relief without creating goods or cash. Neither
party may enter a new prepayment while in an active proceeding. Dedicated estate
custody agents cannot be configured as forward counterparties.

Tests cover a two-of-four delivery, extension, residual write-off, preserved
balance sheets, blocked dissolution until resolution, CPU/reference agreement,
checkpoint continuation and rejection of new seller/buyer admissions during
recovery. The subsequent two-item batch also covers town recovery: a case opened
at Due prevents ordinary spot orders at Acquire despite an earlier opening
admission. Existing native deliveries remain serviceable.

## Delivery contribution integration

Direct forward collection now shares contribution reservations with prior estate
inventory purchases and subsequent bilateral/town/mint matching. Full pooled
storage can reduce or block actual delivery without deleting the residual claim.
Prepayments and credit receipts retain separate, unpooled reservation paths.
Stable and proportional collection still keep requested, allocated and delivered
quantities distinct; a pooled-space limit can reduce completion below a prior
proportional grant. Guarantor-delivered goods now use the same contribution rules at their existing
Due boundary. Actual guarantee performance creates equally sized native recourse;
space-blocked quantities retain their original claim.

The mixed estate/direct-delivery checks compare full, partial and zero delivery,
fractional carry, retained stock and claims, CPU/reference execution, checkpoints
and separate statements. Two affected selections passed 36 and 60 tests, with
strict all-target Clippy. These counts overlap earlier gates.

Guaranteed delivery and integer-cost follow-up passed 85 tests across five targets
and strict all-target Clippy. Small partial deliveries can release zero historical
prepayment value; their quantities and recourse still change, but the accounting
adapter now omits zero-valued journal lines.
