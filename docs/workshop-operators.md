# Household-owned workshop operators

This is a first implementation of independent employers, operating accounts and
business closure. Operators are toll manufacturers: a household owns the firm,
and the firm leases a subset of an existing communal workshop. It employs local
households to process the settlement's materials. Finished goods remain with the
settlement and enter its existing consumption, contract and trade paths.

No parallel goods inventory is introduced. A lease is a claim on canonical
installed equipment, not a second equipment stock or a free transfer of physical
ownership. This distinction is deliberate: private factories buying their own
inputs and selling their own goods, loans, bankruptcy courts and multi-owner
shares remain extensions beyond these operators.

## Accounts and ownership

Each firm has a stable ID, owner household, home site, workshop family, leased
units, founding/closure dates, cash, capital, revenues, wages, rent, dividends,
liquidation and written-off service fees. The owner is a persistent household,
so head succession does not erase ownership. A household's departure ends its
local operating lease and returns remaining cash to that household wallet;
ordinary journey/lost-estate accounting then applies.

Every firm's account satisfies:

```
cash = invested capital + paid service fees - wages - rent - dividends - liquidation
```

Global money totals include firm cash once, alongside settlement operating cash,
household wallets, councils, institutions and cargo escrow. Household wallets
include explicit capital invested/returned in their existing reconciliation.
The cash-buffer social indicator continues to measure spendable household cash,
not inaccessible company capital.

Quarterly founding opportunities require a resident household with actual cash,
installed specialized equipment and recent family work. The highest-cash eligible
household can invest 25% of its wallet, leasing half the family's existing units,
up to one unit. The founder compares fees expected from the locally observed work
and leased fraction against the planned shift's wages and rent; a nonprofitable
forecast blocks entry. Capital must cover three such shifts plus rent. This is a
backward-looking forecast, not knowledge of future orders, inputs or payment.
New operators do not receive a subsidy or create equipment.
Only one operator per site/family can hold an active lease; failed leases have a
six-month interval before another founding opportunity.

## Monthly transfers and production

1. End leases whose owner has left, town is abandoned, or specialized operation
   has been disabled. Return remaining cash; never remove physical equipment.
2. Pay rent to town operating cash: `leased units × 0.08 × wage rate` each month,
   bounded by company cash. The wage rate is the existing `18 × food price` per
   worker-month, in the simulation's abstract money units.
3. Plan a shift from the previous family's work multiplied by the leased fraction
   and 1.1, with a 0.05 worker-month minimum. Cap it by equipment capacity, the
   remaining town craft allocation and company cash. Pay resident households
   before retail demand is calculated. Rounding never authorizes unpaid GPU work.
4. Remove these funded worker-months from the municipal payroll's craft work
   basis. The existing town-cash cap still applies to remaining municipal pay.
   This does not create a second workforce: all recipe execution uses the same
   GPU craft-labor pool, including its cultural and infrastructure reservations.
5. On GPU, partition installed family capacity into communal and leased subsets.
   Only the prepaid portion of leased capacity operates. Prepaid shifts run first;
   communal equipment and the existing shared household-craft allowance supply
   remaining work. Inputs, output targets, recipe knowledge and labor still limit
   execution. Record actual operator work separately from paid/possibly idle work.
6. Bill actual completed work at `1.25 × wage rate`. Gather all town invoices,
   then allocate at most 20% of available town operating cash proportionally.
   Unpaid fees are written off; they are not spendable receivables or new money.
7. Distribute at most 5% of cash above three last-shift payrolls as owner dividends,
   additionally capped by accumulated revenue minus wages, rent and prior dividends.
   Capital investment is not profit; accumulated losses must be recovered first.
   Retain the remaining working capital for subsequent shifts.

Materials, climate, trade demand, knowledge and damaged workshops thus affect
operator utilization and solvency. In the reverse direction, insufficient cash
reduces available leased production capacity, wages and household food access.
Communal capacity remains available, so closure need not shut down an entire
industry or make every town fail. Lease capacity contracts with the surviving
canonical equipment; wear is recorded only by the existing GPU asset ledger.

## Distress, closure and persistence

Three months funding less than one quarter of the planned shift trigger closure
for exhausted working capital. Six months using less than 15% of the prepaid shift
trigger closure for insufficient orders or inputs. Idle wage and rent payments
are real losses, not retrospective cancellation. Closure returns any remaining
cash to the owner and releases the lease for subsequent communal production.
Failure events record revenue, wages, rent, liquidation and the measured cause.

Old archives omit the optional employer subsystem and retain communal operation.
`Generator::set_enterprises(bool)` establishes or switches the policy at a
completed boundary; funded accounts settle before closure on the next monthly
step. New worlds enable operators, but no firm exists until its founding costs
and opportunities are met. Packed GPU lease/allowance/work vectors and sparse
firm records are archived with the ordinary monthly state. Inspectors show firm
owner, lease, cash, utilization and cumulative transfers.

The coefficients above are explicit game-balance assumptions. They are not fitted
historical business-survival, wage or rental data. Paired experiments and retained
fixtures below establish implementation and causal coupling separately from
credible parameter ranges.

## Reproduction

```
python3 scripts/build_history_evaluator.py --output output/operators-build
python3 scripts/evaluate_enterprises.py \
  --binary output/operators-build/evaluator.bin \
  --build-manifest output/operators-build/manifest.json \
  --output output/operators-comparison --seeds 17,81,409 --years 50
```

The four modes are communal/equal wages, communal/industry-linked wages,
operators/equal wages and operators/industry-linked wages. Every mode uses the
same executable, seed, geography and settings. The default experiment uses yield
0.33, living ecology, discoveries, offices and waterworks recovery priority.
Annual trajectories, complete firm accounts and closure causes, manifest hashes,
utilization and population/shortage summaries are retained. The runner rejects
partial output, broken accounts and failed conservation tolerances. Five-year
survival excludes firms too young to have reached five years at the final date.

## Verification and integration claims

| Claim | Evidence |
|---|---|
| Startup transfers existing wealth | Controlled history fixture debits settlement cash into a declared owner wallet, then exercises normal wallet-to-firm investment; money residual remains unchanged |
| Leases cannot duplicate workshops | Equipment stocks are identical before and after startup/closure; GPU partitions leased and communal capacity from the same installed units |
| Cash affects production | A funded lease completes GPU work; returning its cash to the owner while retaining the lease stops its contribution and reduces fixture output |
| Wages and fees are separate transfers | Prepaid work limits, actual completed work, wage expenditure, received fees and written-off fees are recorded independently |
| Failure has consequences | Idle operations spend wages/rent, close, return only surviving cash, and release equipment; corrupt accounts are rejected |
| Investment is not dividend income | Analytical cases reject dividends from initial capital or unrecovered losses and enforce both profit and reserve limits |
| Entry responds to opportunity | Analytical cases distinguish recent work that covers the wage/rent quote from work that cannot cover it |
| Ownership links reconcile | Per-household invested/returned capital must match capital and liquidation across that household's firms |
| Persistence preserves economic effects | GPU fixture compares saved/reloaded six-month continuation against batched advancement, then disables firms without discarding cash |
| Existing physical limits still apply | Full GPU economy regression suite exercises overstock, finite inputs, industrial specialization, mining limits, recycling and archived equipment |

The controlled fixtures use explicit test-only imported equipment and metal to
isolate industrial capacity. Production initialization does not import those
stocks in ordinary worlds. Annual seed comparisons address emergent balance;
they do not substitute for the immediate-mediator checks above. Tests of different
GPUs/backends, empirical fitting, and claims about fully simulated labor contracts
or private inventory ownership are outside this increment's evidence.

The [calibration and verification report](workshop-operator-calibration.md) retains
all development and final seed comparisons, numerical tolerances and test scope.
