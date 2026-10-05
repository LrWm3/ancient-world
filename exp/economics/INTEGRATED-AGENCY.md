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

The forward exposes a remaining planning gap. Feeding members and holding enough
stock for a future collective promise are different objectives. The current
household policy meets consumption first and does not reliably prepare that future
delivery. Shared settlement correctly retains the arrears and blocks another
advance; eventual repayment must not be reported as timely performance.

The next bounded extension should let a household compare **dated production and
stock cover for accepted obligations** alongside member needs, using the shared
claim readers. It needs an explicit policy for their priority, feasible member work,
and tests showing both timely performance and unavoidable shortfalls. It should not
silently make every external creditor outrank member consumption. General joint
planning, autonomous underwriting and sustainable price discovery remain separate.

## Reproduce

From `exp/economics`:

```sh
cargo +1.92.0 run --release --locked --example integrated_agency
cargo +1.92.0 test --release --locked --test integrated_agency
```

The example prints monthly food deficits, overdue deliveries, treasury and debt,
chosen programs, actual delivery dates and separate financial statements. The six
focused controls compare CPU and reference state, ledger, reports and books; rebuild
the simulation at every phase; resume a law-interruption checkpoint; reverse input
agent/participant tables without changing explicit priorities; and check committed
observer events. They exercise the common loop without removing unsupported planner
combination guards. This does not replace the historical full-v1 baseline.

Verification: **130 tests passed** across `integrated_agency`, `agency`,
`state_governance`, `state_formation`, `mint_finance`, `households`,
`forecast_context` and `telemetry` (including six new combined controls).
The CPU example, strict all-target Clippy, formatting and repository artifact
checks also passed. The full crate suite was not rerun.

Sources: `src/agency/integration.rs`, `tests/integrated_agency.rs` and
`examples/integrated_agency.rs`. Generated logs remain under ignored `output/`.
