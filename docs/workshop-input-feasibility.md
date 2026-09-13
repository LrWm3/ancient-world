# Workshop staffing against usable orders

This extends the opt-in demand-staffing pilot; it does not change aggregate
production authority or the monthly schedule. The physical recipe solver remains
on GPU. The sparse CPU staffing planner forecasts how much private attendance
could use the current town stock before paying wages.

## Correcting the work boundary

Both service procurement and demand staffing now exclude any recipe with an
edible output. The production shader executes those recipes as household work and
does not increment operator `enterprise_used` for them. Sharing an industry ID
is therefore insufficient evidence that an operator can earn a fee. The same
catalog predicate is shared by the two request paths. Household food production
continues normally; this change does not privatize it.

This mismatch did not explain the latest seed-1024 closures: its firms were in
metal and ceramic families, while the bundled food recipes are family zero.

## Stocked-work forecast

At Reserve, after production orders and before workshop attendance/payment:

1. Copy current goods into scratch stock; do not add pending cargo or new extraction.
2. Visit recipes in the month's ordinary rotating order, checking required knowledge.
3. Bound batches by orders, available inputs, target output room, dry storage and
   processing-residue capacity. Count shared inputs once; a later recipe may use
   a forecast intermediate from an earlier recipe.
4. Sum non-food work by family. Cap an operator's original request by its leased
   share of that work and by the raw eligible-order ceiling.
5. Apply existing cash, actual participant and capacity limits, then pay only the
   resulting attendance. Production still reports completed work and customer fees.

The latest staffing observation records raw orders, optional stocked-work ceiling,
original request and final pre-cash request. Older observations have no feasibility
field. Validation bounds requests against every ceiling that is present.

This is deliberately a forecast, not an escrow or a duplicate production run.
No scratch goods, output or nutrient totals enter physical ledgers. It can be
optimistic about intermediates whose upstream labor is later unavailable, and
about inputs consumed by other activities after Reserve. It can be pessimistic
about same-month extraction. This may postpone private hiring until a later month;
upstream town orders remain intact. Tool-priority waves, equipment wear, personal
skill and subsequent consumption remain execution constraints. No guarantee of
full utilization or profitability is implied.

## Verification and evaluation

The analytical fixture checks absent inputs, shared-input competition, intermediate
chains, monthly recipe ordering, target capacity, full storage, waste capacity,
knowledge requirements and scratch-stock immutability. It passes. The hardware-GPU
procurement fixture checks that food-only orders cannot fund private services,
missing inputs cannot pay speculative attendance, valid industrial work can
restart, and batched history matches save/reload continuation. It passes (2.03 s).
Strict all-target Clippy and the native build pass.

Keep demand staffing opt-in until broader workshop and household outcomes justify
a default change. The following comparison holds all policy switches constant
and compares the previous executable with the input-aware executable.


### Seed 1024, fifty years

Frozen 32/32 founding checkpoint, five civilizations, 600 monthly steps, named
administration, local inheritance, delivery-paid exports, funded procurement with
0.25 surplus share, and contract/demand staffing enabled. Estate reclamation is
**off** on both sides. Previous outputs are the control side of the estate review
screen; new outputs are under ignored `output/workshop-input-screen/on/`.

| Measure | Previous, no issuance | Stocked, no issuance | Previous, issuance | Stocked, issuance |
| --- | ---: | ---: | ---: | ---: |
| Operator paid worker-months | 31.794 | 20.683 | 29.312 | 23.381 |
| Operator completed worker-months | 23.455 | 17.262 | 23.048 | 18.673 |
| Revenue minus wages and rent | -225.32 | -17.17 | -80.16 | -46.65 |
| Profitable operators / records | 5 / 12 | 5 / 9 | 4 / 7 | 4 / 12 |
| Operators still open | 0 | 0 | 0 | 0 |
| Population | 154.824 | 153.044 | 155.181 | 158.208 |
| Need-weighted household hunger | 0.06166 | 0.07630 | 0.05348 | 0.05935 |

All four new arms exit successfully. No loans issue. Baseline equals credit and
issuance equals combined after removing only the credit record. Maximum absolute
relative cash-audit residual is 1.32e-7. These paired arms are not independent
replications. Runtime is 16.62–16.99 seconds per small diagnostic world; this is
not a default-resolution benchmark.

The filter cuts wasted wages, but also completed work. It does not establish an
economy-wide improvement: hunger rises in both issuance conditions, and population
moves in opposite directions. Several operators are profitable before closing;
closure alone is not proof of a loss-making business. Conversely, positive margins
for a subset do not establish a healthy industrial economy. Source availability,
actual customer demand, operating continuity and household purchasing remain
necessary follow-up questions. Held-out seeds 256 and 409 were compared with
the same settings and frozen before/after binaries.


### Held-out seeds 256 and 409

All sixteen before/after executions (four monetary arms per seed per executable)
completed successfully. Baseline and issuance arms below isolate the staffing
change without a credit intervention. No parameters were fitted to these seeds.
Each pair uses exactly the same founding checkpoint and settings as above.

| Seed / arm | Completed work, before → after | Operating margin, before → after | Population, before → after | Hunger, before → after |
| --- | ---: | ---: | ---: | ---: |
| 256 / baseline | 17.564 → 16.522 | -372.25 → -378.74 | 291.851 → 291.849 | 0.05083 → 0.05083 |
| 256 / issuance | 18.084 → 16.993 | -372.68 → -379.17 | 294.280 → 305.501 | 0.06216 → 0.05061 |
| 409 / baseline | 135.051 → 189.043 | -3,042.61 → -1,135.09 | 291.409 → 322.869 | 0.06433 → 0.06667 |
| 409 / issuance | 150.593 → 186.691 | -3,150.96 → -1,285.57 | 315.006 → 315.904 | 0.06943 → 0.07219 |

Seed 256 still closes all five operators, three of which have positive operating
margins. Seed 409 has 23 operators in the new runs, nine profitable and three
still open, compared with 25/5/3 (baseline) or 29/5/2 (issuance). Its new baseline
pays 254.772 worker-months versus 234.200 previously: the improvement is not simply
less hiring. Completed/paid work rises from about 58% to 74%. Seed 256 remains
below 50% utilization despite the input check, warranting a separate labor-allocation
investigation. These are cumulative operator totals, not per-firm causal matches.

Credit arms cannot all be collapsed into their no-credit counterparts. Before the
change, the combined arm issues one loan in each held-out seed; seed 409 defaults.
Afterward, seed 409's credit-only arm issues and defaults on one loan, while the
other new held-out arms issue none. Its completed work and population round to the
baseline values in the table, but operating margin is -1,135.23 rather than
-1,135.09. No claim of complete-history equality is made for those differing arms.
Maximum absolute relative cash-audit residual across the held-out runs is 1.76e-7.
Raw outputs remain ignored in `output/workshop-input-screen/heldout-{before,after}`.

### Decision and remaining connections

Keep the improved demand-staffing pilot, still disabled by default. It fixes a
request/execution mismatch and substantially improves one held-out industrial
trajectory, but is not a general solution: seed 256 margins worsen slightly,
seed 1024 loses completed work, and household food access is not consistently
better. Do not calibrate by aggregate operator profit alone.

The next specific checks are:

- **Unused recipe labor:** the GPU divides an ordinary recipe's allowance by a
  capped count of remaining recipe slots, not just feasible jobs. Test whether
  blocked jobs leave usable paid time idle, then explicitly redistribute unused
  capacity without exceeding any material or work budget.
- **Spending-backed prices:** `market_decisions` distributes town plus household
  cash across all quote deficits, while ordinary household retail buys food.
  Inspect that mismatch before treating retained wallets as demand for industrial
  goods. Household investment and institutional transfers are separate channels.
- **Physical access and recovery:** retained wealth needs an actual usable town,
  claim resolution and transport. Cash transfer alone did not improve the estate
  comparison. Remote abandoned-settlement recovery remains unfinished.

The broader circulation objective remains open.

## Follow-up: reclaiming unused recipe allowances

The hardware fixture reproduced a separate execution defect: with identical metal,
orders, ten available craft worker-months and three prepaid operator worker-months,
adding five recipes blocked by absent charcoal reduced operator completion from
3.0 to 2.0. The first ordinary recipe received a fraction of labor based on remaining
catalog slots, and later blocked slots left their allowance idle.

Ordered economies now run one bounded spare-capacity sweep after the tool-priority
and ordinary sharing sweeps. It preserves the first ordinary sweep's sharing and
monthly rotation. The extra sweep retains remaining labor, typed capacity, prepaid
attendance, inputs, storage, waste room and completed-order counters. It can use
leftover work without another wage payment or a fresh production order. Experienced
operator output still debits actual attendance at its bounded productivity rate.
Unmanaged legacy economies keep their original two sweeps.

This is not a converged production-chain solver. Rotation still affects which
feasible job receives spare capacity first; a chain can remain blocked, and no
new customers or raw materials are supplied by the extra sweep. Whole-world balance
comparisons remain necessary before claiming improved workshop survival or food access.

Verification: the expanded hardware fixture passes (2.54 s), including absent
and limited inputs, a one-batch order, shared finite prepaid attendance and
experienced-worker controls. The existing prepaid-capacity/checkpoint fixture
passes (2.31 s), and the experienced-work finite-input fixture passes (1.70 s).
These checks establish bounded execution and continuation for their fixtures;
they do not substitute for the pending multi-seed economic comparison.
Strict all-target Clippy also passes after using a direct initializer in the new
fixture. Repository artifact and whitespace checks pass.
