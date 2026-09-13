# Funded work and proportional workshop shares

Inspection of the seed-1024 recovery/substitution baseline found six private
operators. Each earned a positive cumulative operating margin, but all closed for
six months without sufficient orders or inputs. Founder capital was approximately
188–215 currency per operator. This is not evidence that another loan would fix
those closures. Some actual funded services also went uncompleted and refunded.

The demand-staffing path applied the operator's fraction of installed equipment to
both current recipe work and stock-feasible work. That remained a ceiling even
when a customer had already funded a larger due commitment. The earlier
contract-staffing path lifted the historical-work floor but could not lift this
later proportional ceiling.

For each of the existing ordered/stock-feasible pools, entitlement now becomes:

`min(total, max(total * lease_share, due_contracted_work))`

The final request remains bounded by both pools, leased equipment, the preexisting
desired shift, labor capacity and cash. Individual matching and GPU execution
still determine actual work. A contract neither creates physical demand nor makes
missing materials available. With no due contract, the existing proportional
allocation is unchanged. This uses the existing per-site/family operator allocation
and does not introduce an auction or change monthly scheduling.

Procurement still transfers town operating cash into real escrow. Household
savings are not treated as committed customer money. A separate household consumer
mechanism would need actual goods/services, ownership, maintenance/consumption and
food-reserve protection; simply adding household balances to service funding would
spend money without household consent or a delivered benefit.

## Verification

The focused entitlement fixture covers ordinary share, larger/smaller due
commitments, no feasible work and independently restricted ordered/input pools.
Existing hardware procurement and checkpoint validation are run separately.


Both feasibility unit tests pass. The existing hardware procurement fixture passes
(2.01 s), including bounded escrow, missing orders/inputs, excluded household food
processing, due-contract staffing, unchanged money and three-month batch versus
checkpoint continuation. Native build and strict all-target Clippy pass.

The paired screen uses frozen executables from before/after the change and seeds
1024, 256 and 409, loading `output/monetary-estates-heldout-founding/SEED.world`.
Each runs 50 years with frozen 32/32 environment and delivery-paid exports,
service-order procurement (share 0.25), contract/demand workshop staffing, local
estate inheritance, named office service and abandoned stock recovery enabled.
Commercial/council/service-order credit, shared issuance and estate reclamation
are explicitly disabled. Artifacts and exact invocation arrays are local under
`output/contract-share-screen`. Compilation overlaps the first control, so this is
not an isolated timing benchmark.


## Paired results

All six runs completed. The repeated seed-1024 control exactly matches its prior
export. Values below are cumulative operator work/margins and year-50 population;
margin excludes capital, dividends, financing and liquidation.

| Seed | Population before → after | Operator work before → after | Operating margin before → after | Contract work before → after | Refunds before → after |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 164.955 → 164.485 | 6.411 → 6.770 | 41.682 → 44.040 | 3.688 → 4.047 | 108.315 → 89.765 |
| 256 | 306.248 → 307.153 | 5.845 → 6.197 | -233.130 → -242.988 | 3.083 → 3.435 | 353.000 → 341.639 |
| 409 | 330.988 → 328.579 | 104.418 → 102.481 | -2573.411 → -2578.427 | 97.620 → 95.761 | 7314.223 → 6865.270 |

No firms remain open in seeds 1024/256 in either version. Seed 409 declines from
two surviving operators to one. Maximum absolute relative cash residual is
1.80e-7. These results support the bounded contract-entitlement correction, not a
claim of generally improved firm survival or household welfare. Higher granted
work can still fund attendance that does not become output.

The after-run audit identifies the next distinction: seed 256 pays for 16.859
worker-months but completes 6.197; seed 409 pays 193.960 and completes 102.481.
Their unpaid-invoice writeoffs are respectively zero and approximately 0.001.
Thus these losses primarily accompany paid-but-unproductive attendance, rather
than completed service customers refusing to pay. This does not establish the
exact execution shortfall cause. Inspect input timing, competing recipes and the
GPU's shared craft-labor pool against firm-specific prepaid capacity before
increasing customer budgets. Seed 1024, by comparison, pays 6.896 and completes
6.770, with zero invoice writeoff.
