# Joint production and sale planning

The opt-in `stock_sale::joint_scenario` now compares current work and sales with
multi-period production and selling continuations. This closes the missing
connection in the earlier sale-only experiment: ordinary production is no longer
the only work alternative, and future sales are represented as actual bounded
transactions in each candidate rollout.

## Bounded candidate set

At Acquire, the financial driver first selects the purchase/decline branch and
establishes current stocks and spending budgets. Joint planning crosses:

- Every current sale quantity from zero to the existing feasible lot cap.
- Ordinary work; defer new work this month then resume ordinary work; or repeatedly
  attempt a selected known productive process whenever no instance of it is active.
- A small configured set of future stock-reserve policies. This fixture compares
  zero-month and six-month food reserves.

Existing processes continue under every work policy. Explicit starts use the
ordinary request resolver, so seed, land permission, labor, storage and occupancy
can prevent them. A proposed start is not a guaranteed crop. The candidate receipt
records actual projected starts and sales, not requested but unfulfilled work.
The producer set is sorted by definition ID and uses no farming-specific branch.

Future sales use the existing monthly lot cap, finite state purchase allowance,
state coins, food/input reserve rule and storage limits. There is no speculative
cash credit. Loan payments remain before Acquire; production and consumption
remain after it. Forecasts use the existing sanitized observation context and
cannot see unpublished future gifts or capacity shocks.

The pilot bounds scope to one participant, four enabled productive definitions,
two current lots and two continuation reserve settings. That is at most 36
candidates; this fixture has at most 24. Horizons must span at least twice the
longest enabled production duration and cannot exceed 24 months. The fixture uses
18 months. Other recursive work/search policies cannot be combined with it.

## Selection and commitment

A qualifying plan must stay active, meet every configured cumulative need-deficit
cap, avoid missed payments and avoid new aborted processes. Among qualifying
plans, compare need deficits in priority order, then failures/payment outcomes,
end-of-horizon food/provision buffer gap, and net debt minus coins. Stable candidate
order breaks ties. The buffer measure reuses the existing capped coverage score;
stock security therefore takes precedence over maximizing sale income.

If no plan qualifies, choose among zero-current-sale alternatives using the same
outcome ordering and report `feasible=false`. This does not claim that waiting
solves scarcity. Nor does a feasible plan establish indefinite sustainability or
repayment of obligations beyond its horizon.

Only this month's sale and work are committed. Acquire carries the selected work
batch into the existing `pending_production` mechanism; Productive executes that
exact dated batch. The credit boundary recomputes and validates both the joint
receipt and production plan. Altered, missing, stale or replayed plans fail without
partial publication. Subsequent months replan from actual state; future production
and sale dates in the receipt are projections, not reservations or promises.

This preserves the scheduler. It extends the credit pilot's previous prohibition
on pending production to accept a correctly dated next Productive batch, while
retaining the other acquisition-system isolation guards. Common financed-purchase
acceptance also carries the selected plan, so that route and ordinary stepping
share the same commitment mechanism.

## CPU comparison

All cases retain the existing plot price, loan terms, initial savings, seed,
production yield and state bid budgets. Only decision policies change.

| 18-month case | Sale-only, six-month forecast | Joint, eighteen-month forecast |
| --- | --- | --- |
| Funded | Borrows; repays; 0 unmet nutrition; 55.80 closing coins | Borrows; repays; 0 unmet nutrition; 7.80 closing coins |
| Limited state purchase allowance | Declines loan; sells opening food; 8 unmet nutrition | Declines loan; no sales; 7 unmet nutrition |
| Low grain price | Declines loan; sells opening food; 8 unmet nutrition | Declines loan; no sales; 7 unmet nutrition |

The funded joint run sells two grain in each of months 7 and 8 and repays in month
13. It retains more food rather than maximizing short-run cash. The two unfundable
cases can see that their livelihood does not support the proposed depletion, so
the fallback retains opening food. Seven unmet units remain: slow gathering alone
is insufficient. The planner neither manufactures land nor removes that shortage.

Over 60 months the funded joint policy completes harvests in months 6, 14, 26,
38 and 50, with zero unmet nutrition. It spends only 48 coins of the state
purchase allowance and closes with 7.80 coins and one grain; seed remains held
or invested in an active crop.

The selected multi-period policy can change as the horizon moves: an early receipt
may project planting immediately after harvest, while a later receipt chooses to
wait. Such revisions are intentional receding-horizon behavior, but projected
future schedules should not be reported as completed or committed work.

## Verification and remaining limits

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --example joint_plan
cargo +1.92.0 test --locked --test joint_plan --test credit --test credit_offers --test sale_plan --test borrowing --test stock_sale
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

37 focused and regression tests passed, along with all-target Clippy.
Tests cover joint work/sale continuations, execution of the chosen dated batch,
no-land scarcity fallback, hidden shocks, horizon validation, forged receipts and
plans, replay rejection, CPU/reference and monthly/batched/checkpoint continuation
with reordered catalogs. An additional 60-month CPU control checks repeated
harvests, seed accounting, zero nutrition deficits and finite state funding.
Tests isolate the work/sale search using scripted origination; the three-case CPU
example additionally exercises the full borrowing comparison. Raw output stays
under ignored `output/economics/`.

This is a finite policy search, not arbitrary calendar optimization. It compares
repeated-process policies rather than every planting date or process portfolio.
Future sales still use simple reserve policies inside candidates. It is more
expensive than the sale-only search, especially when nested inside borrowing
forecasts, and remains opt-in. There is no uncertainty margin, competitor model,
price negotiation, discounting or labor-cost preference in its score. Longer
horizons and terminal stock coverage reduce the demonstrated boundary problem;
they do not prove that no end-of-horizon artifact can occur.
