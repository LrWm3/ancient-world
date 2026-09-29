# Second household and consolidation batch

Five further alternating household/shared passes, continuing the first
[integration batch](INTEGRATION-PASSES.md). Person self-directed policy changes
remain excluded. Monthly boundaries are unchanged.

| Pass | Household work | Shared consolidation | Status |
| --- | --- | --- | --- |
| 6 | Useful partial voluntary support | Storage-bounded conserved transfers | Complete |
| 7 | Physical barter proceeds | Pooling-aware town storage reservations | Complete |
| 8 | Charter delegates consumption buying | Exclusive collective/member purchase routing | Complete |
| 9 | Purchase eligibility follows membership | Runtime admission rather than historical exclusion | Complete |
| 10 | Collective purchases for accepted process inputs | Shared commitment demand and integrated tests | Complete |

## Pass 6

A support mandate's offered quantity can exceed what shared storage admits.
The resolver now tests the greatest fitting whole-unit transfer, then retains
existing donor protection and collective benefit checks. Offered and accepted
quantities remain distinct. A full store rejects the donation without aborting
execution. CPU/reference audited tests compare partial food support with this
blocked control and replay the resulting ledger.

Physical wage pooling and household hiring remain separate follow-ups.

## Passes 7–8

Town settlement reserves both the raw transfer and its mandatory income share.
The reservation uses the same income classification as collection, maintains
relationship-specific fractional carry across trades, and excludes loan advances.
Private sales can therefore accept physical payment. Raw and pooled storage must
both fit; proceeds still cannot finance another trade in the same book.

The static charter selects `Purchasing::Collective` (default) or `Members`.
Delegation disables collective consumption bids and permits registered private
buyers. Their need projection counts only the portion retained after pooling;
redistribution later in the month remains a possible benefit, not promised supply.

Evidence: CPU/reference audited controls include storage rejection, odd payment
carry, conserved barter stocks, ledger replay, and a private purchase feeding both
members through subsequent household allocation without spending collective cash.
Existing household integration/market and barter accounting controls pass.

## Pass 9

Registration no longer permanently excludes buyers who have ever belonged to a
household. Acquire checks current membership and the static purchase route. An
outside former member can buy privately; re-entry restores the collective gate.
Adaptive members can still offer protected surplus when their buy side is blocked.

Evidence: an audited three-month CPU/reference scenario exercises collective
buying, explicit member exit with private purchases and no pooling, then accession
with private buying blocked again. Continuation from each monthly boundary agrees.
Registration, legal permissions and locality remain explicit prerequisites; this
is not autonomous registration or recruitment.

## Pass 10

The opt-in `fund_committed_inputs` charter parameter adds missing stock inputs of
active member processes to collective order deficits. Input protection and demand
share one calculation of unpaid stage entry requirements. Private holdings offset
demand; paid stage inputs are not requested again. Collective routing and an
eligible needs-first objective still gate buying. Speculative work orders do not
qualify, and whole-lot matching does not promise an entire future production plan.

Evidence: a two-month audited CPU/reference scenario buys two seed units for 40
coins, transfers them to the committed worker through Productive allocation,
completes the process and pools two of four output grain. The transferred grain
carries 12 reporting ticks from the worker's average stock basis (three existing
units at basis 3 plus four produced at basis 40), rather than an invented sale
value. Replay and checkpoint continuation agree. Disabled funding, an unstarted
work opportunity and already-held private inputs all suppress unnecessary orders.

A cross-listing barter control also reserves one common storage budget: two
three-unit physical payments pool one unit on the first fill and two on the second,
using the accumulated odd-unit carry. Two free storage units permit only the first
fill; three permit both. CPU/reference books agree.

## Scope and remaining work

All five pairs are implemented. The fixtures supply static charter parameters,
registration, prices, membership decisions and accepted processes. They establish
composition and conservation under controlled conditions, not calibrated economic
sustainability for arbitrary terms. Private purchase projections conservatively
ignore future redistribution of the pooled portion; separate member bids may
therefore retain extra collective stock. Input funding does not forecast new
businesses, output demand, investment returns, or all future labor availability.

Next distinct gaps include household hiring and purchased-labor cost allocation,
physical wage pooling, autonomous recruitment/registration, speculative input and
investment planning, town recovery and broader institutional formation. Personal
self-directed policy changes remain explicitly excluded.

## Verification

Final run: **301 tests passed across 25 suites**, including all seven new
`household_composition` tests, the prior combined integration controls and the
120-month household income CPU/reference test. One existing slow annual 32-person
accounting test remained ignored. The full crate suite was not run. Strict
all-target Clippy, formatting, whitespace and repository artifact checks passed.

From `exp/economics`:

```sh
cargo +1.92.0 test --locked --test household_composition \
  --test household_integration --test household_income --test households \
  --test household_accounting --test household_credit --test household_dissolution \
  --test household_equipment_disposal --test household_property_package \
  --test household_market --test town_market --test need_orders \
  --test agreement_laws --test laws --test telemetry --test inventory_accounting \
  --test process_accounting --test employment --test acquisition --test lending \
  --test credit --test finance --test service_accounting --test reporting_coverage \
  --test barter_accounting
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
cargo +1.92.0 fmt -- --check
```

Generated logs remain under ignored `output/economics/household-batch2-*.log`.
Only source, tests and Markdown documentation are committed.
