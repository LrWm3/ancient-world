# Funded workshop service orders

This is the payment foundation for operator working-capital credit. Existing
operators sell workshop services to their town, not the town's finished goods.
A future service fee must have an actual payer and reserved cash before it can
be described as a funded receivable.

`History::fund_workshop_order(firm, work, due_month)` is an explicit opt-in call at
a completed history boundary. It posts one pending order per operator, with a
fixed service quote, a shared-currency identity and a due month within twelve
months. The town moves existing cash into escrow. Insufficient cash funds a smaller
work quantity; the record distinguishes requested work from actually funded work.
Cash uses the actual f32 withdrawal, so small representation differences remain
visible in the funded quantity.

The order reserves neither labor nor materials. The normal planner still needs
workers, wages, tools and inputs. At the contracted month's enterprise settlement:

- Completed work earns the fixed fee, bounded by funded work and escrow.
- That work is excluded from ordinary service invoices, preventing double payment.
- Remaining work uses the existing quoted-fee/affordability settlement.
- Unused escrow returns to the original town, including its retained account after
  abandonment. Refunds smaller than the town account can represent remain in
  escrow and are retried, rather than being silently discarded.

A closed operator or abandoned site cancels unpaid work. A skipped due month
expires the order; later work cannot fulfill it retroactively. Revenue is recorded
only for actual completion. Posting an order is not operator income, output, or
repayment. Funds still in escrow count once in the existing monetary ledger.

Orders and their paid/refunded/remaining balances persist in history. Older
archives default to no orders. With no orders, the existing service-invoice path
remains in use. This initial interface has no automatic town procurement policy,
and no explorer controls. [Optional service credit](workshop-service-credit.md)
now constructs loan requests from existing orders; automatic procurement and
active-order production comparisons remain unfinished.

The focused fixture uses the existing GPU-backed workshop setup but supplies
explicit completion amounts to isolate settlement. It compares zero, partial and
full service; validates money and operator ledgers; checks restored continuation;
rejects overlapping orders; and verifies missed deadlines refund rather than pay.
It does not establish additional real production or a credit benefit. A later
full-production comparison must show wages funded, workers assigned, inputs used
and goods produced before claiming that financing unlocked work.

Verification passed: the hardware-backed order settlement fixture (one test),
the existing hardware posted-wage/service-quote fixture (one test), five active
enterprise tests and strict all-target Clippy. Closure/abandonment refunds and
invalid requests are included. Fifteen other enterprise tests remain ignored in
the ordinary filtered run; this is not a claim that the whole GPU suite ran.

## No-order continuation comparison

A seed-1024 founding checkpoint was run for 200 years at terrain/ecology edge 32
on the Quadro RTX 5000 with Max-Q Design. Delivery-paid exports were enabled;
commercial/council credit, issuance and institutional lending were explicitly off.
No service orders were posted. The copied executable was built from `8ece891`,
before the subsequent staffing-constant extraction. Outputs stayed under ignored
`output/service-order-baseline`.

After removing only the new empty `enterprises.orders` field, the entire exported
history exactly matched `output/monetary-institution-reserves/1024-baseline.json`.
Ending population was 98.3713. This checks inactive-path continuation, including
previously committed constants changes; it does not evaluate active orders or
credit-backed production. The concurrent shader fixture is separate verification.


[Bounded automatic procurement](workshop-service-procurement.md) is now available
as an independent opt-in policy. It reserves next-month fees from capped town
surplus using the existing order and refund path. Explicit API posting remains
available. Ordinary invoicing is still the default.
