# State, household and persons in one economy

Implemented 2026-10-05. This is a bounded composition check for the
[organization controller](STATE-AGENCY.md), not another simulation engine.
One initialized state, three individual citizens, a legally formed two-person
household and a marketplace execute **14 months through the existing scheduler**.
Both organizational controllers run at the same opening boundary. Market exchange,
production, pooling, loan servicing, forwards and annual land dues use ordinary
settlement and separate double-entry books.

## The connected scenario

- The state starts with wheat, a plot and no coins or labor. One person lends it
  six existing coins. It sells wheat and buys metal and two actual worker hours
  per mint batch. Each completed batch produces ten coins. Its controller compares
  another dated mint attempt with retaining its plan, targeting a 14-coin reserve.
  One initial mint attempt is supplied for month 2; the controller selects the
  additional attempts for months 3–5.
- The metal supplier and a grower form a household. Each has five monthly hours
  and contributes 20%. A six-hour job therefore needs collective allocation. Plot
  work consumes one seed and produces four wheat and two seed; a competing wood
  job produces ten wood. These are deliberately small, one-month test recipes,
  not a historical farming calibration. Metal comes from finite opening stock.
- Household members each need one nutrition unit monthly. The household begins
  with `NetOutput`; its governor can choose `NeedsFirst`. Half of eligible member
  receipts and output are pooled, with the existing shared-storage and reservation
  rules. Personal identities, labor and financial books remain distinct.
- The state prepays the household two coins in month 2 for two wheat due in
  month 4. The grower's preaccepted plot agreement owes one coin in month 13,
  which collective resources can support. This is a lease, not a mortgage.
- Both offices have two-month elected terms. Ballots are generated from supplied
  static preferences. The state preferences target treasury cover; household
  preferences target member nutrition. This does not model political negotiation.

The outside worker offers hours for coins but has no recurring consumption need
in this control. State identity/citizenship and the plot agreement are initialized;
first-state founding is verified separately in `state_formation`. Household founding
uses the normal formation adapter. Financial terms, prices, initial stocks and the
finite program menus are supplied. Neither organization fabricates consent or
automatically discovers new contracts.

This uses the mint market's generated orders with supplied limit prices, ordinary
person process planning and spot paid capacity. It does not combine every market
driver, ZIP, composition search, mortgages, wage arrears and insolvency in one run.

## Timing and observed results

At Open, organizational decisions read the same boundary and issue next-month
instructions. Private projections freeze other organizational controllers; they
do not assume those organizations will select convenient new programs. Accepted
instructions, individual planning and ordinary contract execution remain visible.
Acquire uses shared opening budgets; the state cannot recycle a new loan or
same-book sales proceeds into another outgoing purchase at that boundary.
Household productive allocation uses its real contributed hours. The existing
production, consumption, pooling and financial boundaries are unchanged.

| Check | Observed outcome |
| --- | --- |
| Normal 14-month run | Four mint batches, 40 newly issued coins; state closes with 18 coins |
| Currency reconciliation | Closing total supply minus opening supply equals the state's recorded issuance; lending and prepayment create no money |
| Household policy | Selects `NeedsFirst` in month 1, effective month 2; two unmet food units in month 1, none in months 2–14 |
| Same resources, retain `NetOutput` | Eight unmet food units across the run; elections still operate |
| State loan | All six coins repaid by month 7; transfers retain separate lender, state and household books |
| Annual lease bill | One coin owed and paid in month 13 |
| Household forward | **Late:** both units remain outstanding at month 4; one delivered in month 10 and one in month 11 |
| No forward agreement | State chooses one fewer mint batch: three rather than four |
| No worker capacity | No completed mint batch and no issuance; accepted financial claims remain recorded |
| No household trade permission | No forward acceptance or prepayment |
| Second advance during late delivery | Rejected under the existing single-outstanding-delivery rule |
| Minting prohibited in months 4–6, reopened in month 7 | No new mint authorization while prohibited; later minting resumes, loan and land dues settle |

The month-one shortages are real: the initial allocation policy applies before the
new policy takes effect. They are reported for both people, not inferred from a
single participant's final report. This fixture has no mortality/deprivation rule.

## Optional commitment preparation

The baseline above remains unchanged. The opt-in `commitment_scenario()` uses
`NeedsThenCommitments { months: 4 }`, explicitly permitted by the household's
constitution. Its controller can select that policy, with member nutrition first
and collective funding cover second. Both members also sign finite wheat-surplus
mandates; a governor cannot supply those signatures.

The work allocator compares, in order:

1. Current member need deficits, using the existing need priority and resource units.
2. Accepted collective claim shortfalls after actual same-month production, pooling,
   arrears and consumption. Each stock resource is compared separately; resource ID
   is the stable tie priority between denominations, not a currency conversion.
3. The existing net-output proxy, then the configured member ordering for ties.

The horizon includes the current month and excludes `month + months`; arrears count
now. Funding and personal protection share `need_orders::accepted_claims`: accepted
forward deliveries and land bills within that horizon, including current accepted
land-tender alternatives, plus currently collectible loan dues and earned wages.
Future loan installments, hypothetical contracts, unearned wages and expected sales
are not counted. Process entry inputs remain in the separate personal protection
reader. A claim forecast is neither an asset nor a reservation.

Unsupplied collective claims create requests for directly productive processes
available to a member. Input provisioning, allocation previews and actual execution
use the same requests. The ordinary resolver still checks legality, rights, seeds,
finite hours and storage. The agreed contribution limit and constitutional work
scope remain binding. Hiring comparisons use the same need/funding priority and
include the newly earned wage claim; ordinary finite hiring budgets still apply.

Timing is a separate choice: under this new policy, signed surplus funding runs
**once at Acquire**, after current household/member reservations and before ordinary
collection and exchange. Current member consumption must already have stock cover; an unfilled food
reservation blocks this early payment-support path even if future work might feed
the member. The donor's accepted obligations, process inputs, signed
consumption horizon and private floor are protected. Only already available private
surplus, within the signed quantity/stock limits and actual shared storage, transfers.
This policy uses the support mandate for claim funding only, gated by
`accept_payment_support`; it does not run the legacy productive need/income-support
choice a second time. Existing policies retain their Productive support window.

That distinction matters in month 4. The household opens with two wheat and the
grower with two private wheat. One shared wheat is reserved for the other member's
food; the grower keeps one for their own food and voluntarily supplies one surplus
wheat before collection. Both delivery units are then available at the original
deadline. Production later in the month is never backdated to fund this payment.

| Matched control | Result |
| --- | --- |
| Original `NeedsFirst` economy | One wheat delivered in month 10, another in month 11 |
| Commitment priority + signed surplus funding | Both wheat delivered in month 4 |
| Same policy without signed surplus, or charter acceptance disabled | Timely full delivery fails; policy cannot appropriate private stock |
| One-month claim horizon | Full delivery still outstanding at the month-4 boundary |
| Grower's consent withdrawn before month 4 | Only one unit delivered by month 4 |
| Unfilled member food reservation despite another member’s surplus | No early payment donation; future production is not treated as present food cover |
| No opening seed | No crop output; both accepted units remain owed, food shortages persist |
| Constitutional work scope excludes cultivation | No collective hours directed to the grower and no completed crop |
| No accepted forward | No wheat delivery target inferred from possible future offers |

With the successful variant, total unmet food remains two units in month 1 and zero
thereafter. Issuance remains 40 coins, the state closes with 18, its loan clears by
month 7 and the annual land bill clears in month 13. Separate entity books reconcile.
Baseline/projected funding records and dated support receipts are exposed through
existing observers; settlement rejects altered receipts atomically.

This is bounded preparation, not a general joint planner or a guarantee. It searches
direct producers and assigns one productive recipient per household boundary;
long process chains, multi-period feasibility, loan amortization forecasts and
endogenous contract negotiation remain separate work. Funding cover is measured
after this month's consumption; it is not a promise that later member needs or
other uses will leave it intact. Donated goods are not creditor escrow. No default
policy, creditor priority, price, crop yield or delivery deadline was changed.

## Reproduce

From `exp/economics`:

```sh
cargo +1.92.0 run --release --locked --example integrated_agency
cargo +1.92.0 run --release --locked --example integrated_agency -- --commitments
cargo +1.92.0 test --release --locked --test integrated_agency --test commitment_agency
```

The example prints monthly food deficits, overdue deliveries, treasury and debt,
chosen programs, actual delivery dates and separate financial statements. The six
focused controls compare CPU and reference state, ledger, reports and books; rebuild
the simulation at every phase; resume a law-interruption checkpoint; reverse input
agent/participant tables without changing explicit priorities; and check committed
observer events. They exercise the common loop without removing unsupported planner
combination guards. This does not replace the historical full-v1 baseline.

Original integration verification: **130 tests passed** across `integrated_agency`, `agency`,
`state_governance`, `state_formation`, `mint_finance`, `households`,
`forecast_context` and `telemetry` (including six new combined controls).
The CPU example, strict all-target Clippy, formatting and repository artifact
checks also passed. The full crate suite was not rerun.

Commitment-preparation verification: ten additional controls cover timely
performance, CPU/reference replay with every-phase reconstruction, signed funding
limits and revocation, short horizons, missing seed, work-scope limits, unaccepted
offers, catalog permutations, unfilled food reservations and forged receipts.
**207 distinct tests passed** across `commitment_agency`, `integrated_agency`,
`agency`, `households`, `household_payment_support`, `household_hiring`,
`household_income`, `household_market`, `household_land_funding`,
`household_forwards`, `need_orders`, `forecast_context`, `telemetry` and
`mint_finance`. Strict all-target Clippy, formatting, the CPU example and repository
artifact checks passed. This is a selected regression set, not the full crate suite.

Sources: `src/households/funding.rs`, `tests/commitment_agency.rs`, `src/agency/integration.rs`, `tests/integrated_agency.rs` and
`examples/integrated_agency.rs`. Generated logs remain under ignored `output/`.
