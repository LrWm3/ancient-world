# Bounded town procurement of workshop services

`--service-order-procurement[=true|false]` independently enables towns to reserve
next-month workshop fees. It defaults off and is independent of commercial credit,
service-order lending and issuance. Omitting it preserves a loaded policy. Direct
callers can configure `Enterprises.procurement`; the explorer's enterprise summary
includes its current policy and latest claims.

## Demand, budget and commitment

At the end of a completed month, active firms without a pending service order
receive a proposal from the town's last production plan. Recipe worker-months are
grouped by workshop family, allocated to the firm's fraction of installed capacity,
and capped by its installed worker-month capacity. Closed firms, abandoned towns,
missing equipment and zero planned work produce no proposal. Quotes use the
existing service rate, without a procurement markup.

For each town, reserve the larger of its configured cash floor (initially 100) and
the existing forecast cost of missing production inputs. At most the configured
share of the remaining cash (initially 25%) is available for service escrow.
Collect all firm requests before spending, then divide that envelope in proportion
to requested fees, capped at each request. Firms are committed in stable
site/family/ID order; that order does not give the first firm the full envelope.

Retained f32 town cash is rounded upward so each actual escrow withdrawal is no
larger than its allowance. Subprecision grants are skipped. Receipts retain
requested fee, allowance, actual funding and the created order ID. They supplement
the durable order record, which retains its full fee/payment/refund history.

This is forward procurement based on a lagged plan, not a promise that next month's
workers, inputs or customers will match it. It can tie up cash in an unsuccessful
order. Existing production, operator payroll, service settlement and refunds remain
authoritative. There is no advance operator income, free labor or loan guarantee.

## Monthly boundary and persistence

Procurement runs in Close, after Respond and before the final economy upload. It
creates orders due next month; the next Reserve can underwrite their service
receipts before wages are funded. The explicit zero-step initialization path does
not procure. A saved last-decision month prevents repeated boundary calls from
posting duplicate orders. Existing pending orders also exclude their firm.

Disabling procurement stops new commitments but does not cancel existing orders,
fees or debts. Old enterprise records initialize with procurement disabled. Policy,
latest claims and durable escrow orders persist; validation checks claim bounds
and their links to the matching order, firm and posting month.

## Verification and remaining evaluation

The hardware fixture exercises disabled procurement, an unaffordable reserve,
absent production demand, actual bounded escrow, same-month replay, corrupted
receipts and multi-month GPU checkpoint/batch continuation. The CLI fixture checks
explicit true/false, omitted policy and independence from service-credit switches.

These checks do not demonstrate a balance benefit. The next experiment should
compare funded procurement with ordinary invoicing, then compare credit off/on
with procurement held constant. Measure escrow occupancy/refunds, completed work,
operator income and failure, buyer input/food affordability and loan outcomes.
Keep the existing fee and cost policies during that first comparison. If procurement
mostly locks up town cash, record that result rather than automatically raising
fees or reducing lender reserves. Longer matched seed comparisons and Stage 2's
benefit gate remain outstanding.

The experiment runner accepts `--service-order-procurement`. With that option,
procurement stays on in **all four** monetary arms, while service lending follows
the existing credit switch. Without it, the runner explicitly disables both new
service switches, preserving an ordinary-invoicing comparison even when loading a
checkpoint that had opted in. Consequently, "baseline" within a procurement run
means procurement without credit/issuance; it is not an ordinary-invoicing control.
Run the same checkpoints and interval without the option for that separate control.

Verification passed: the procurement GPU boundary/continuation fixture (one test),
the CLI override fixture (one), nineteen active market tests (two ignored), ten
Python reporting tests, strict all-target Clippy and the native build. These are
implementation checks. The [eight-arm 50-year screen](service-procurement-screening.md)
now shows mixed balance effects: more work and less terminal hunger without
issuance, but less work and more terminal hunger with issuance. No loans issued.
Procurement remains opt-in; this does not pass the Stage 2 gate.
