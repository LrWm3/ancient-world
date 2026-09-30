# Frozen economics v1 verification scenarios

This is the executable manifest for V1-02 through V1-05 in the
[bounded release checklist](V1-RELEASE.md). The test constructors and catalogs at
the tested revision are the exact fixtures; the descriptions below identify their
actors, resources, policies, consent and controls. No scenario balance changes
were made for this freeze. S1 is new; S2–S6 retain their existing parameters.

Run from any directory in a clean checkout:

```sh
python3 exp/economics/scripts/check_v1.py
```

The runner itself resolves paths, uses Rust 1.92.0 with locked dependencies and
runs S1–S6, the full crate suite, explicit ignored population test, all-target
Clippy, formatting, artifact and diff checks. `--list` prints every exact command.
`--scenarios-only` is a diagnostic subset, not release certification. Zero selected
tests, a failed command or a changed checkout fails the run. It does not tag or
publish anything. Raw logs, revision/settings JSON and a generated Markdown report
live under ignored `output/economics/v1-<revision>-<UTC>/`.

## S1 — enter and perform

Fixture: [household_offers](tests/household_offers.rs), building on
`competition::scenario(plots, 7)`. Two adult persons, a household and state; two
plots in the baseline and one in the contention control. First month is 1; the
baseline finishes month 24. There is no randomness except the explicit seed-7
`PriorityLottery` for the one-plot allocation. Five hours/person/month make the
20% household labor contribution nonzero. Each member contributes half of 40
storage units. Grain, seed and fuel use space; coins do not. The household starts
with two seed, ten grain and four coins; persons retain the base five grain and
one fuel each. The state holds the replenishing wood pool.

Supplied consent: lawful household founding, citizenship and land terms, and the
explicit ordered candidate bundles. Fixed `ContinuingFirst` person priorities;
static contributed household governance. Ordinary work decisions follow acceptance.
The planting/growth/harvest schedule uses 2/1/2 labor, lasts six months and returns
8 grain plus 1 seed. Land costs 2 grain annually starting the following year.

Assert at least two crops, seed reuse, exact once-only pooling, separate books,
CPU/reference equality and resume immediately after acceptance. The baseline
currently completes four crops; all 48 person-month food and warmth needs are met.
Permission, seed, labor, storage and forged/stale controls reject atomically.
Both adults unavailable in month 2 abort both crops without seed refund.
The one-plot loser has no accepted land or citizenship package and may collect wood.
Existing `household_support_funds_member_dues_in_native_goods_or_coins_once` is the
native/coin pooled-payment control (month 13, 1 grain or 3 coins; private debt stays
private). See [acceptance evidence](V1-ACCEPTANCE.md).

## S2 — continuing household economy

Fixture: `households::income::scenario::{scenario, coordinated}` and
[household_income](tests/household_income.rs). Four persons: two household members
and two outside grain producers; town and marketplace agents. Baseline months
1–120. No random draws. Members have five hours each, producers one. Storage is
64 units/person; productive inputs and opening cash are those of the inherited
household-market fixture. Fuel work requires six hours for two fuel, so household
direction matters; producers consume and return one seed and make four grain.
Supplied local venue, crossed 40-coin grain/fuel quotes and work targets (24 private
fuel, 8 grain). The fuel buyer starts with 100 coins. Household governance is
`NeedsThenIncome`, with an exclusive fuel-production mandate.

The coordinated variant supplies prospective member consent to contribute at
most one surplus fuel/month, protects two private fuel, and targets one collective
fuel. It does not alter the private target or productive technology. Assert all
240 household food requirements, 119 fuel traded, final household cash 60, finite
coin conservation and separate books. Resume with Audit after month 60 and compare
remaining ledger, state and statements. The two producers' startup deficits are
included in the evidence, not hidden by the household success claim.

No-support control runs 27 months: private fuel reaches 24, trade stops, household
cash is 20 and the two members miss food in month 27. Interruption control runs
25 months, shuts the fuel matcher in months 11–13, asserts an actual household
food deficit, restores the matcher and asserts members fed from month 15 onward.
Both controls now run through Audit. The support-observer test establishes that
its explanation records do not alter state or work.

## S3 — production and commitments

Fixture: `fixture(household, sales)` in
[household_farm_finance](tests/household_farm_finance.rs), derived from
`stock_sale::scenario("funded")`. Person, state and optional one-member household;
one financed plot and one independent leased plot. Months 1–24, no random draws.
Existing endowments include 6,500 bridge coins; buyer allowance is 12,000 coins,
maximum two grain lots/month at the existing 1,200-coin quote. Crop yield remains
20 grain plus returned seed. Configured 12-month mortgage, 2-grain annual lease,
and two supplied 600-coin prepayments for 2 grain, admitted in months 7/15 and due
in 8/16. Consent and counterparties are supplied, not autonomously underwritten.

Six-month sale forecasts bound cumulative nutrition deficit to zero. The supported
baseline must feed the person and perform mortgage, rent and both deliveries.
No-buyer control sets purchase allowance to zero and must retain repossession.
Fixed-buffer control uses six months of stock without forecast and must retain
its missed meal despite paid financial claims. All variants use actual accounting;
main variants resume before month 8. The household does not assume the private
mortgage. See [farm-finance interpretation](PRODUCTION-FUNDED-CREDIT.md#continuing-personhousehold-farm-finance).

## S4 — physical issuance

Main fixture: `persons_household_land_credit_forwards_and_need_orders_run_in_one_mint_economy`
in [mint_finance](tests/mint_finance.rs), from `minting::provision_scenario("adequate")`.
Two persons, household, state issuer and marketplace, months 12–17. No randomness.
Opening issuer granary 18 wheat and treasury 12 coins; persons have 3 coins each;
supplier has 2 metal and the worker additionally has 1 wheat. The worker contributes
20% of hours. Stock storage is finite (32 units before household sharing).
Minting requires 2 metal and 2 paid hours per 10 coins. Existing metal and labor
quotes, three-month provision horizon and dated issuance opportunities remain.

Supplied 3-coin supplier loan, 1-wheat forward bought for 3 coins and due month 13,
and 2-coin annual land bill. Incremental provision must repay loan/delivery/rent
and issue coins; full-buffer control must retain zero issuance and 2 rent unpaid.
Resume after month 13, compare state/ledger/Audit and reverse participant ordering.
The entire `mint_finance` target additionally bounds shared cash, incoming-fund
visibility, metal, labor, pooled storage, funded estate/property purchases and
forged packages. These are short controls (up to month 13 except the main case).
They do not imply a unified marketplace engine or endogenous state policy.

## S5 — loss, custody and exit

These are distinct deterministic controls, not one large synthetic economy.
The runner names exact suites/tests; supplied consent includes proceedings,
listings, bids, guarantees and explicit discharge terms. No random draws.

| Fixture / selected target | Actors, resources and fixed limits | Required evidence |
| --- | --- | --- |
| All `mortgage_receivables` | Person borrower, household lender/member, investor, state guarantor and one/two custodians; 8-coin plot, 2 down, 6 principal; 4-coin property sale; claim price 3/7 or face 6; guarantee 2 beginning month 4/5; through month 7 | Funded/unfunded claim purchase, crop control, shared custody, guarantee timing, retained/discharged deficiency, discount/premium cost, continuation and journal reconstruction |
| All `native_receivables` | Household/person creditor, borrower, investor, guarantor and custodian; native grain principal with fixed reporting quotes, coin bids, finite storage; through month 8 | Real native repayment, insufficient receiving storage, unfunded buyer, partial relief, purchase cost and member exit without erased claims |
| `mortgage_recovery::financed_land_and_attached_crop_use_the_authorized_estate_lifecycle` | Existing mortgage/crop fixture, authorized property estate, funded/unfunded buyer, buyer labor 2/0; through month 7 | Maintained crop completes; neglected crop aborts only at work, not transfer; actual sale proceeds and losses reconcile and resume |
| `estate_receivables::receivable_and_inventory_lots_compete_for_one_opening_cash_budget` | Household/person estate inventory and loan claim, competing supplied bids; inspect month-3 Acquire | Cash cannot buy both independently affordable lots; no same-window receipt financing |
| All `household_dissolution` | Existing one/two-adult households, static dissolution mandate, explicit exit consent, dues/assets/private claims; short fixed fixtures through month 15 | Last-member wind-down, live claims, title/residual custody and supported financial reporting; unavailable resources do not silently close claims |
| `recovery::recovery_observers_report_actual_guarantees_distributions_and_writeoffs` | Bounded existing coin proceeding with guarantee/discharge; through month 5 | Actual recovered versus forgiven amounts remain distinct and observer output is read-only |

S5 continuation is supplied by the mortgage and native suites; journal round-trip
by the priced-mortgage suite. The controls do not certify new denomination/security
combinations or automatic death estates. Financial outcomes, including actual
recovery, remaining claim face, purchase basis, settlement gains and losses, are
asserted in the tests rather than inferred from balanced journals.

## S6 — population regression

Exact ignored test:
`household_accounting::specialist_households_reconcile_production_trading_and_annual_dues`.
Fixture `households::scenario()`: 32 distinct adult specialists, eight households,
state, configured tools/forwards/land obligations and finite storage. Months 1–13,
CubeCL CPU, deterministic catalogs; no random draws. Each household uses static
contributed governance. Existing opening allocations, specialist work catalogs and
posted financial terms are supplied. Cost shares are equal across joint outputs;
issuance uses explicit nonredeemable-equity reporting. All agent statements must
reconcile through annual dues. Record need deficits, unused capacity, work receipts,
completed processes and runtime; zero deficits and speed are not acceptance gates.

## Explanations and reporting conventions

The runner extracts test-only `V1_EVIDENCE` rows from committed state, work receipts
and financial statements. It introduces no telemetry calls into production logic.
Resource IDs are fixture-local; work receipts count their native request units,
not universal hours. Unused capacity is period expiration plus final unused stock
of capacity. A completed-process count includes consumption where defined; farming
is definition 1 in S1/S3. Grain/fuel markets in S2 are 1/2; S4 uses its own mint book.
Empty town volume means no town-book driver, not absence of all financial transfers.

Absent/prohibited opportunities are exercised by S1 and the full suite's existing
planning/permission observers. S1 bounds stock/labor/storage and rejects unfulfilled
packages; missed work has a real sunk-input consequence. S2 support receipts explain
protected reserves, policy rejection and unavailable demand. S3 sale decisions
separate future production from current funds; S4 provisioning records explain
full-buffer versus incremental choice. S5 recovery observers distinguish payment,
write-off, custody and unresolved claims. Existing telemetry observer-equivalence
tests remain in the full suite. No claim of universal planner introspection follows.

All books remain separate-agent statements. Fixed report quotes are valuation
conventions, not FX, and journals do not eliminate household/member claims. See
[V1-RELEASE](V1-RELEASE.md) for deliberate exclusions and the stopping rule.
