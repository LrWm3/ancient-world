# Startup forward proposals

Follow-up: [startup stocks and liquidity preferences](STARTUP-LIQUIDITY-COMPARISON.md)
tests the retained workshop stocks, adds monthly liquidity scoring, and extends
the continuation/replay comparison below from 16 to 48 months. This document
records the original experiment and its verification window.

This opt-in experiment asks whether two organizations can replace missing town
ore inventory with a voluntarily accepted delivery and productive operating plans.
It builds on [stocked offer publication](ACCEPTANCE-OFFER-PUBLICATION.md). A loan
alone does not solve the observed problem: the town already has cash and labor,
but no observed reason to produce an initial stock of ore.

## Scope and timing

An organization's signed operating agreement can authorize one startup forward
per resource. Spot quote authority is insufficient. Both parties must sign the
same exact prepayment and delivery delay for a listed stock lot; this is not price
negotiation. Candidate buyers have a gap below a signed purchase target. Sellers
have a signed productive recipe; opening stock is unnecessary. Discovery uses
existing productive work/quote bundles and rejects oversized catalogs.

Each current governor evaluates its own objectives and preferences, with its own
forecast horizon and frozen peer assumptions, against the same opening state.
Assessment extends a short horizon through delivery. Both must be no worse and
at least one strictly better; both projections must admit and deliver the specific
contract's full goods on time. Cash substitution does not count as delivery.
These comparisons are not a solvency or whole-system readiness guarantee. Cash
reserve loss can saturate when both alternatives run out of cash.

At Open month M, publication seals both mandates, records their exact proposed
programs and governors, and posts ordinary direct-forward terms for Acquire M+1.
No stock or cash changes during publication. If admission succeeds, the accepted
work/quote programs activate at Open M+2; delivery is due no earlier than Acquire
M+3. Failed admission leaves these programs inactive. Production cannot use the
prepayment in another reservation at the same Acquire boundary. Existing work
and obligations still execute normally while these new programs wait.

These are recurring discretionary operating programs, not a guarantee to produce
exactly one lot. Production, wages, inputs and spot sales remain ordinary work;
subsequent policy review may change the programs. The delivery obligation remains
in the existing forward/claims/default/accounting machinery. Private projections
freeze future autonomous reviews and therefore cannot guarantee actual delivery.

## Iteration record

1. Separate signed forward authority, stock denomination and duration validation.
2. Stable bounded candidates from intermediate stock gaps and productive sellers.
3. Pure bilateral forecasts, actual admission and on-time delivery observations.
4. Fixed-program mandates excluded from unauthorized discovered alternatives.
5. Atomic publication, admission-contingent activation, sealed mandates and tenure.

6. Opt-in Open review and the matched missing-town-ore fixture. Stable agent,
   market and program ordering select the first mutually acceptable proposal;
   only one publishes globally per Open. Ordinary accepted decisions have priority.
   Pending agreements suspend further ordinary review, including emergency review,
   until activation or rejection; proposal review itself has no emergency override.
7. Failure and reconstruction controls: law, funds, date, production, preference,
   duplicate authority use, exact program snapshots and impossible activation.
8. Sixteen-month CPU/reference comparison with reconstruction at every phase,
   dated ledger replay, accounting equality and independent catalog reordering.
9. A CPU command reports the matched authority-off/on comparison and receipt dates.
10. This report, README and integration status record measured scope and next gaps.

## Measured comparison

Both variants retain workshop ore 4 and metal 4, town ore 0, their original cash,
employment, rights, memberships, recipes, pricing bounds and private/state offers.
There are no supplied workshop/town asks, work targets or accepted ore forwards.
Only separate signed startup-forward authority differs. Workshop forecast horizon
is 12 months; town horizon is 4. Existing observed peer-process/state-ballot
expectations remain in use, with full-information observations.

| 48-month result | No forward authority | Signed startup forward authority |
| --- | ---: | ---: |
| Completed mint batches | 4 | 42 |
| Workshop batches | 2 | 41 |
| Town batches | 0 | 40 |
| Person food deficit | 37 | 0 |
| Ending wage arrears | 2 | 0 |
| New ore forwards | 0 | 1 |
| Economic/governance readiness | Fail | Pass |

The governors accept in month 7; a two-coin prepayment for two ore admits in month
8, the signed operating programs activate in month 9, and delivery is due in month
10. Actual delivery completes. Subsequent replenishment uses ordinary spot orders.
Each of the three full post-startup years has 12 workshop and 12 town batches,
24 ore delivered, and 12 paid employment months for each organization. Their annual
incomes are 12 and 13; ending cash is 50 and 43. All nine financial statements
reconcile; the earlier household delivery and state loan also perform.

Readiness now reads authorized program snapshots from both ordinary decisions and
activated startup agreements. Pending/rejected proposals do not count. Production,
food, wages, repayment, delivery, operating-income and accounting gates are unchanged.

## Reproduce

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example acceptance -- --startup-forward-comparison
cargo +1.92.0 test --locked --release --test organization_forwards --test forward_assessment --test forward_controls --test acceptance_financing --test forward_continuation
```

The comparison command reports failures as data; its successful exit alone does
not assert readiness. Generated run logs belong under ignored `output/`.

## Limits and next steps

This removes one supplied town working inventory, not all startup endowments or
institutional setup. The precise contract price, whole-lot quantity, delivery delay,
legal types, constitutions, employment and governance preferences remain supplied.
There is no ZIP pricing for forwards, general autonomous underwriting, fair rationing
among competing counterparties, or automatic renegotiation. One startup agreement
per party/resource is a conservative bound, not a general credit limit.

The pure zero-workshop-ore diagnostic exposes cash-loss saturation: a twelve-month
cash objective can regard two exhausted-cash outcomes as equal while the seller
benefits. Shorter cash projections can refuse the same deal. Mutual forecast consent
is therefore distinct from sustained solvency; do not use it alone as acceptance.
The successful 48-month fixture retains workshop ore and is separately verified.

Production remains discretionary after activation, while the delivery claim remains
binding. Circumstances and future governance can invalidate forecasts; ordinary
shortfall/default handling still applies. Future work should test conflicting
accepted obligations, stronger liquidity-sensitive preferences and withdrawal or
renegotiation of pending proposals without bypassing existing claims. Removing the
remaining workshop working stocks is a separate matched experiment.

Verification: 155 distinct tests pass: 140 existing tests across 22 selected
regression suites, 13 forward authority/assessment/failure controls, one matched
48-month reference comparison, and one 16-month CPU/reference continuation test.
The separate 48-month CPU command reproduces both rows above. Reconstruction,
ledger replay and catalog-reordering coverage is 16 months, not 48. Formatting,
all-target release Clippy with warnings denied, and the repository artifact check
pass. An independent reviewer checked authority, activation, evidence reporting
and the scope of these claims. This is a targeted regression run, not the entire
experiment test suite.
