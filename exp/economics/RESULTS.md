# Process simulation: CPU results

[Specialized activities](ACTIVITIES.md) adds mining/refining, produced and repaired
tools, livestock, plot-attached housing, work targets and coin payment alternatives.

[Four people](FOUR-PEOPLE.md) extends the storage/currency fixture with three additional
people, shared-budget tests and a 60-month capacity comparison.

Latest extension: [storage and currency](STORAGE-CURRENCY.md) records fixed storage,
two-grain annual taxes, treasury issuance and grain sales, with a 60-month CPU comparison.

Implemented and verified on Linux x86_64 with Rust 1.92.0 and CubeCL 0.10.0 CPU.
These are correctness results for abstract deterministic fixtures, not calibrated
economics or a performance comparison. The ten original nine-month controls and
ten new 60-month scenarios have CPU/reference coverage. The long scenarios were
also run through the CLI, with local traces under ignored `output/economics/`.

## Implementation exercised

Stable-ID tables describe agents, generic participant components, stocks, capacities, requirements,
assets, rights, process definitions and process instances. A passive state agent
owns the plot. Its right grants a person exclusive use subject to occupancy and
specifies who receives output. No farmer class or farming update is involved.

The planner searches a bounded two-link chain: fulfillment process, then a process
producing its stock input. It forecasts shortages by month over a six-month horizon,
includes dated output from active instances, prefers feasible deficit-reducing
candidates and retains rejection reasons for infeasible wishes. It selects at most
one new productive process per need per person per month, deduplicating a producer
selected for multiple needs. Need-tagged requests share the same finite budgets. Future labor is forecast, not
escrowed; only plot occupancy is committed across the process's full duration.

Four boundaries execute each month: Open, productive process planning/resolution/
execution, consumption, and Close. Existing and new work share opening budgets.
Minimum useful work is indivisible; output cannot fund another same-round action.
Execution emits typed process changes and account effects. Failed stages abort,
retain sunk inputs and release occupancy at commit. Nutrition and warmth are
separate fulfillment accounts for the current person/month and reset next month;
unused labor also expires.

Host Rust code plans, reserves, validates and groups effects. The CubeCL CPU kernel
performs segmented integer gather into candidate balances. Host validation then
publishes balances and structural changes together. Every integer prefix is checked
before launch. Failed or duplicate batches cannot partially publish. Transaction
identity is `(batch ID, transaction ordinal)`, with effect ordinals within each
transaction; IDs do not depend on thread append order.

## Baseline outcome

The person begins with 5 grain, 1 seed and 2 labor per month; nutrition requires
1 grain per month. The plot right covers months 1–9. Planting consumes the seed
and 2 labor in month 1; growth uses 1 labor in each of months 2–5; harvest uses
2 labor and produces 8 grain in month 6 before consumption.

| Month | Grain at Close | Unused labor | Nutrition fulfilled | Deficit |
| --- | --- | --- | --- | --- |
| 1 | 4 | 0 | 1 | 0 |
| 2 | 3 | 1 | 1 | 0 |
| 3 | 2 | 1 | 1 | 0 |
| 4 | 1 | 1 | 1 | 0 |
| 5 | 0 | 1 | 1 | 0 |
| 6 | 7 | 0 | 1 | 0 |
| 7 | 6 | 2 | 1 | 0 |
| 8 | 5 | 2 | 1 | 0 |
| 9 | 4 | 2 | 1 | 0 |

Exactly one productive instance completes. Grain reconciles as `5 + 8 - 9 = 4`;
seed ends at zero and completed productive labor totals 8. There are 36 committed
batches. Later labor does not accumulate, and the exhausted seed stock prevents
another start. This is a single harvest experiment, not a self-sustaining farm.

## Controls

All results below cover nine months with the same process and timing rules except
for the named control. Each matches the Rust reference state, ledger and reports.

| Scenario | Final grain | Total unmet nutrition | Observed explanation |
| --- | --- | --- | --- |
| `baseline` | 4 | 0 | Harvest in month 6 |
| `short-food` | 4 | 3 | Two initial grain; shortages in months 3–5 remain despite later harvest |
| `no-seed` | 0 | 4 | Missing stock prevents planting |
| `no-right` | 0 | 4 | Missing use authority prevents planting |
| `short-right` | 0 | 4 | Right ends before harvest; no start |
| `missed-work` | 0 | 4 | Zero labor in month 3 aborts after 3 units of sunk work; seed is not refunded |
| `no-need` | 5 | 0 | No productive start or consumption; seed remains |
| `disabled` | 0 | 4 | Known producing process disabled |
| `continuing-first` | 4 | 0 | Harvest receives both labor units; competing one-unit process is refused |
| `new-first` | 0 | 4 | Competing process completes; harvest fails; unusable remaining labor stays available |

The last two controls submit identical month-6 requests against identical opening
resources. Only allocation priority changes, not execution phases. The competing
process is an explicit fixture intent, not evidence of a multi-need planner.

## Repeated harvests: 60 months

`repeated-harvests` keeps the original 5 grain, **1 initial seed** and 2 labor per
month. Planting consumes 1 seed. A successful harvest now emits **8 grain and
1 seed** together. The land right lasts through month 240, so the run cutoff does
not prevent a process being planned beyond month 60. Other timing and yield rules
are unchanged. The original `baseline` remains a frozen single-harvest control.

The person plants in months **1, 9, 17, 25, 33, 41, 49 and 57**. Harvests complete
in months **6, 14, 22, 30, 38, 46 and 54**. All 60 nutrition needs are fulfilled.
At Close 60, grain is `5 + 7 × 8 - 60 = 1`; the eighth crop is active and due in
month 62. Seed stock is zero because its one seed has been planted, not lost.
At every committed boundary, stored seed plus active crop count equals one.
Productive labor used is 61 units, including work on that eighth crop.

This demonstrates repeated production under the deterministic fixture, not an
indefinite ecological sustainability result. With initial seed removed, no crop
can start and 55 months have unmet nutrition. A three-month planning horizon also
fails: the six-month process never reduces a deficit inside that horizon. Seed
availability and looking far enough ahead both matter.

## Warmth competing for the same labor

The person now has two essential monthly needs: 1 nutrition and 1 warmth. The
same generic planner and process resolver handle both. Warmth adds:

- Initial stocks of 1 fuel and 40 raw wood units owned by the person.
- A one-month preparation process: 1 raw wood plus 1 labor produces 2 fuel.
- A consumption process: 1 fuel produces 1 current-month warmth fulfillment.

Raw wood is finite, non-regenerating stock; it is sufficient for these runs and
is not a modeled forest. The initial fuel unit covers a first month spent planting.
The six-month forecast applies to both needs. Thus fuel production can be requested
to cover future months even when current warmth is already affordable.

`NeedFirst` allocates work by each need's explicit rank, then continuing work and
stable IDs within a rank. Consumption uses the same need ranking if stocks compete.
`ContinuingFirst` instead protects active instances before ranking new requests.
These are allocation policies at the same boundary, not scheduler changes.

| 60-month scenario | Labor/month | Completed harvests | Food-shortage months | Warmth-shortage months | Aborted crops |
| --- | --- | --- | --- | --- | --- |
| `warmth-food-first` | 2 | 7 | 0 | 0 | 0 |
| `warmth-first` | 2 | 0 | 55 | 0 | 1 |
| `warmth-protect-active` | 2 | 7 | 5 | 0 | 0 |
| `warmth-abundant-food-first` | 3 | 7 | 0 | 0 | 0 |
| `warmth-abundant-warmth-first` | 3 | 7 | 0 | 0 | 0 |
| `warmth-inactive` | 2 | 7 | 0 | 0 (zero demand) | 0 |
| `warmth-no-wood` | 2 | 7 | 0 | 59 | 0 |

Food-first completes the same crop cycles as the repeated-harvest run. Fuel is
prepared 32 times: wood ends at `40 - 32 = 8`, and fuel at `1 + 64 - 60 = 5`.
Combined work is `61 + 32 = 93` out of 120 available units. Stored fuel bridges
planting and harvest months, when cropping consumes both labor units. No resource
or capacity is duplicated between needs.

The warmth-first failure is **avoidable starvation caused by fixed priorities**:

1. Fuel-buffer requests win in months 1–5, leaving only one labor unit for a
   planting operation that requires two. The unused partial allowance cannot plant.
2. Planting finally happens in month 6; initial food is already exhausted.
3. At Open month 11, there are **5 fuel units** and 2 labor available. Fuel
   preparation requests 1 labor to top up the six-month buffer; harvest requests 2.
4. Fuel wins. Harvest gets no useful grant and aborts under the current strict
   failure rule. Its seed is sunk, so there is no next crop and no external rescue.

A paired boundary test holds that exact month-11 opening state and the same two
requests fixed. Changing only need priority completes the harvest and returns the
seed. This separates policy failure from an impossible resource budget. With three
labor units, both priority orders succeed. Protecting active work also preserves
the crop, but planting is still delayed: its first harvest is month 11, leaving
food deficits in months 6–10. Subsequent harvests occur every eight months.

The inactive-demand control consumes no fuel or wood; the missing-wood control
uses its initial fuel once, then records warmth shortages without taking crop work.
In these original controls, deficits are observations only. The separate
condition-enabled runs below add consequences without changing their allocation policies.

The practical next planner improvement is to distinguish **immediate essential
shortfalls and deadline-critical work from discretionary buffer replenishment**.
These results do not establish food-first as a universal policy or justify hiding
the failure by changing phase order, adding resources, or refunding failed seed.

## Generic requirement consequences: 60-month CPU runs

The seven new scenarios use the same `Participant`, `Requirement`, provision and
condition machinery for people and an institution. Original scenarios remain
consequence-free controls. Policy and phase order are unchanged; the new evaluator
is not consulted by the planner.

Illustrative rules: two deprivation points per missing provision unit, one point
recovered per fully supplied month, half capacity from four points, and a terminal
transition at twelve. These are deliberately abstract values, not calibrated
survival times. Conditions take effect on capacity at the following Open. Separate
rules can use different parameters, affected capacities and terminal names.

| Scenario | Observed result |
| --- | --- |
| `conditions-warmth-food-first` | Both needs met for all 60 months; zero deprivation; seven harvests and the eighth underway |
| `conditions-warmth-first` | Food shortages in months 6–11; labor falls from two to one starting month 8; death at Close 11 |
| `conditions-warmth-no-wood` | Warmth shortages in months 2–7; labor falls to one starting month 4; harvest fails at month 6; death at Close 7 |
| `conditions-repeated-no-seed` | Food shortages in months 6–11; death at Close 11; no harvest |
| `institution-upkeep` | Two opening supplies cover months 1–2; upkeep shortages in months 3–8; administrative capacity falls from four to two starting month 5; dissolution at Close 8 |
| `institution-recovery` | Same opening upkeep supplies; a pre-existing five-month process releases 60 finite reserve units in month 5; shortages only in months 3–4; full capacity returns in month 6 and deprivation reaches zero at Close 8 |
| `institution-supplied` | Sixty opening supplies cover every month; no impairment or dissolution |

The institution recovery fixture conserves supplies: two initial units plus sixty
released units minus 58 completed upkeep provisions leaves four. Unmet upkeep in
two months consumes nothing. Its abstract administrative capacity is an exogenous
fixture allowance, not a claim to create human labor or a staffed organization.
There are no person-specific branches in the evaluator or lifecycle handling.

The new feedback matters: missing warmth now reduces the labor needed to complete
the food harvest. The warmth-first planner still makes the earlier buffer mistake;
consequences expose its cost rather than improve the decision rule. The healthy
food-first control remains feasible with exactly the earlier resources.

Terminal agents cease autonomous planning and consumption, and regenerate no
capacity from the next Open. Unfinished processes abort at the next Productive
barrier with no refunds or new outputs. Stocks remain on the inactive identity;
there is no estate or succession system. Requirements cease after the terminal
month. Therefore the six recorded food-shortage months before death must not be
interpreted as an improvement over 55 shortage months in the consequence-free
control. Reports preserve lifecycle state and its date as well as deficits.

Close records completed-process provision provenance, requested versus supplied
quantities, before/after condition, per-condition capacity modifiers, and terminal
transitions. Multiple modifiers use the minimum, not their product; this remains a
scoped experimental policy. Partial recovery persists across months, while excess
provision cannot bank satisfaction or accelerate recovery.

Ten focused tests cover severity/duration, recovery and inactive demand; condition
configuration rejection and arithmetic bounds; modifier composition and rounding;
actual provision rather than stored inputs; atomic rejection of altered/missing
condition settlements; terminal behavior; all seven CubeCL CPU/reference and ledger
replay comparisons; monthly versus batched execution; resumption at every barrier
in the first twelve months; and reordered catalogs, requirements and condition rules.
See [conditions.rs](tests/conditions.rs). Raw traces are ignored local artifacts.

## Consequence-aware allocation: matched CPU comparisons

The optional policy compares a small portfolio of six-month forecasts, prioritizing
terminal outcomes, impairment and deprivation before inventory buffers and work.
Every forecast uses the existing resolver and condition evaluator, so shared labor,
finite stocks and process failure are enforced. Only its current productive batch
is committed; subsequent months are replanned. Controls use identical opening
states, catalogs, resources and condition rules within each pair.

| Comparison | Fixed policy | Consequence-aware policy |
| --- | --- | --- |
| Warmth-first repeated harvest fixture | Death at month 11; no harvest | Alive after 60 months, no food/warmth shortfall; harvests at 6, 14, 22, 30, 38, 46, 54; eighth crop active |
| Severe cold with ample grain at month 6 | Food-first harvests, then cold causes death at Close 6 | Fuel-first prevents cold death; crop aborts and loses the only seed; grain eventually runs out and starvation causes death at Close 31 |
| Institution resupply | Two upkeep shortages, then recovery | Same actual state and reports; retains one existing delivery process and recovers without duplicate interventions |
| Exhausted grain, seed, fuel and wood | Death at Close 6 | Same outcome; every initial candidate predicts terminal harm |

The healthy forecast run completes 31 fuel preparations, leaving nine wood and
three fuel; it spends 92 of 120 available labor units on crop work and fuel. Seed
stock plus active crop count stays exactly one at every committed boundary.
The original consequence-free food-first control prepares 32 fuel lots; this small
difference is an observed scheduling outcome, not evidence of global optimality.

The cold comparison starts from a constructed month-six Open with the existing
crop ready for harvest, twenty grain, no fuel, no spare seed and ten warmth
deprivation points. Base capacity four is reduced to two by impairment. The
harvest requires both units; making fuel needs one. Both actions cannot complete.
Both policies run 60 months from this same boundary (months 6–65). Choosing heat
is justified by immediate survival, but it is not a permanent solution. Neither
policy has a mechanism to acquire replacement seed. All condition parameters remain
illustrative and monthly; these are not empirical human survival estimates.

Each `Decision` retains alternatives, scores, first-work receipts and predicted
monthly reports. Selected current-month forecasts match actual reports in all
152 decisions across the four forecasting fixtures. Full observed report-row
matches across their forecast windows are:

| Fixture | Exact matching predicted/observed rows |
| --- | --- |
| Repeated harvest | 122 / 345 |
| Cold | 156 / 156 |
| Resupply | 345 / 345 |
| Exhausted resources | 36 / 36 |

These rows compare balances, conditions, need fulfillment and lifecycle, not just
survival. Forecasts assume the named strategy continues; later replanning frequently
changes inventory trajectories in the harvest case. They are conditional scenarios,
not calibrated predictions of the receding-horizon policy itself. End-of-run windows
are compared only where observations exist. A separate test introduces an unobserved
future capacity loss and scheduled intent: neither changes the earlier forecast,
while the actual later outcome diverges as expected.

Seven focused tests in [planning.rs](tests/planning.rs) cover both priority choices,
matched opening inputs, seed continuity, recovery, unavoidable scarcity, observation
boundaries, bounded scope and failure atomicity, prediction/actual comparisons,
CPU/reference parity, ledger replay, batching, checkpoint continuation and reordered
catalogs. All six new named scenarios run on CubeCL CPU; forecast simulations use
the Rust reference backend. The search is limited to one participant and at most
four requirements, and does not explore every possible action sequence.

## Durable-tool barter on CPU

The state posts one tool for three grain. It saves one labor unit per harvest
(two becomes one), preserves eight grain plus one seed output, and lasts six
completed harvests. All six fixtures ran on CubeCL CPU for sixty months.

| Fixture | Observed result |
| --- | --- |
| `tool-beneficial` | Purchases in month 6; meets food and warmth every month through 65; eight harvests; ends with five grain |
| `tool-no-offer` | Same four-grain/cold opening, no purchase available; aborts harvest, loses seed, dies from nutrition deprivation in month 15 |
| `tool-food-risk` | Keeps food; never buys; meets both needs throughout months 1–60 |
| `tool-unavailable` | State tool is not offered; seven manual harvests and both needs met |
| `tool-exhausted` | Posted tool has zero uses; never buys; seven manual harvests and both needs met |
| `tool-lifetime` | Already owns a one-use tool; assisted harvest in month 6, six later manual harvests; both needs met |

In the beneficial pair, month 6 starts with cold impairment, two available labor
units and a due harvest. Purchase reduces food from four to one and transfers
three grain to the state. Assisted harvest and fuel preparation each reserve one
labor unit; the harvest then adds eight grain and one seed before consumption.
Without the tool, avoiding immediate cold death consumes labor needed for the
harvest. The aborted crop cannot return its sunk seed.

Assisted harvests complete in months **6, 15, 23, 31, 39 and 47**. Manual harvests
follow in **55 and 63**. The sixth use succeeds before exhaustion. The seller ends
with exactly three grain, the offer fills once, and the tool remains owned by the
buyer with zero uses. Idle, rejected and aborted work do not reduce its lifetime.
The unavailable/exhausted/one-use controls have warmer openings; they isolate
fallback behavior and are not matched survival comparisons with the cold pair.

The food-risk offer is affordable, but forcing its acceptance causes a nutrition
shortage before the first harvest. The chosen no-purchase path has none. Thus the
planner distinguishes payment feasibility from the consequences of spending food.

**Horizon limitation:** restore twenty opening grain in the beneficial fixture
and the six-month planner declines the tool: the food consequences of losing seed
lie beyond its forecast. Extending that same opening forecast to thirty months
selects purchase. The experiment demonstrates a bounded investment decision,
not reliable long-term valuation.

All six fixtures match the Rust reference in final state, reports and ledger.
Selected first-month predictions include ownership and remaining uses and match
actual outcomes. Tests cover replay, monthly batching, checkpoint continuation at
all five first-month barriers, altered dated plans, forged trade/equipment effects,
duplicate buyers, joint overspending and exclusive tool use across competing
processes. Failures leave the original state unchanged. Raw traces remain ignored
under `output/economics/tool-*.md`; source assertions are in
[equipment tests](tests/equipment.rs).

## Experience on CPU

Three additional fixtures ran for sixty months on CubeCL CPU. Practice awards
one cultivation point per completed harvest. Four points unlock a manual
one-labor harvest; the original manual technique needs two. Tool-assisted
harvests award the same practice. These are illustrative, uncalibrated units.

| Fixture | Harvests and practice | Outcome |
| --- | --- | --- |
| `experience-manual` | Seven harvests, seven points | Fourth harvest at month 30 still costs two labor; months 38, 46 and 54 cost one |
| `experience-tool` | Eight harvests, eight points | Tool used in months 6, 15, 23 and 31; experienced manual work in 39, 47, 55 and 63; two tool uses preserved |
| `experience-no-seed` | No harvests or practice | No seed to start production; nutrition deprivation becomes terminal in month 11 |

Both productive fixtures meet nutrition and warmth throughout. Their harvest
dates match the respective pre-learning controls: in these scenarios the gain
reduces labor requirements and preserves equipment rather than increasing yield
or harvest frequency. The tool fixture runs months 6–65; the others run 1–60.
They have different opening conditions and are not a paired tool-versus-manual
survival comparison.

The fourth harvest does not receive its own experience discount. Forecasts
predict threshold crossings using actual simulated practice awards, and selected
current-month reports match completed outcomes including competency balances.
This establishes learning-aware evaluation of existing candidate work, not a
separate motivation to train or an optimal long-term learning policy.

Tests cover multi-month stages (award only at completion), idle/failed work,
missing inputs, forged technique eligibility, practice overflow with atomic
rollback, all three CPU/reference comparisons, ledger replay, monthly batching,
checkpoint continuation at the threshold and reversed technique storage order.
The ledger's validated stage completion and the fixed catalog supply award
provenance. No transfer, decay or failure-based learning is implemented.

Source assertions: [experience tests](tests/experience.rs). Local CPU traces are
ignored artifacts under `output/economics/experience-*.md`.

## Annual access commitments on CPU

Historical baseline before the candidate change; current outcomes are in
[dated production candidates](DATED-CANDIDATES.md).

The accepted agreement grants plot access starting in month 1 for one grain per
year, first due in month 13. Three fixtures ran sixty months on CubeCL CPU.

| Fixture | Payments | Nutrition / warmth shortfall | Outcome |
| --- | --- | --- | --- |
| `annual-free` | None | 0 / 0 | Survives, seven harvests |
| `annual-access` | Four grain, due 13, 25, 37, 49; all paid | 4 / 0 | Survives, same seven harvest dates as free access |
| `annual-arrears` | Five grain, due 13, 25, 37, 49, 61; all paid by run end | 2 / 0 | Survives, eight harvests; first arrears cleared by month-13 harvest |

Free and paid access have identical initial stocks, rights and productive policy.
The rent reduces resources available for food; successful full collection does
not demonstrate a balanced contract. The current mandatory oldest-first payment
policy runs before consumption, without a subsistence reserve. The planner sees
upcoming payments within its horizon, but does not choose whether to accept the
agreement or refuse payment.

The arrears fixture starts at month 13, with no grain and an existing crop due
to finish. Due records one grain unpaid. Production is allowed to finish that
crop, yielding eight grain. ClearArrears transfers one to the state, leaving seven
before consumption and six afterward. New planting is blocked at the earlier
productive boundary and cannot be backdated after debt clearance. This fixture
has different opening conditions from the free/paid pair.

Obligations retain gross amounts owed, paid and remaining debt by agreement and
anniversary. Tests establish no billing before twelve elapsed months, billing
without any harvest, partial payment, oldest-first arrears, no double charge,
atomic rejection of missing/altered settlements, expiry retaining old debt,
forecast visibility, CPU/reference equality, ledger replay, monthly batching and
checkpoints at every month-13 phase. Continuing processes retain access through
arrears; ordinary expiry/resource constraints still apply.

These fixtures initialize acceptance. There is no negotiation, dynamic agreement
termination, interest, subsistence exemption or estate collection. Source tests:
[commitments](tests/commitments.rs). Raw runs remain ignored under
`output/economics/annual-*.md`.

## Access-offer decisions on CPU

Historical baseline before the candidate change; current outcomes are in
[dated production candidates](DATED-CANDIDATES.md).

Four offer fixtures ran sixty months on CubeCL CPU. Plot rights begin dormant;
the planner compares acceptance with no acquisition. Acceptance enables the right
and fixes the first bill twelve months later. The state posts terms but does not
negotiate them or autonomously optimize its offer.

| Fixture | First decision | Observed sixty-month outcome |
| --- | --- | --- |
| `offer-useful` | Accept one-grain annual access in month 1 | Survives; seven harvests; four installments paid; four food provisions missed |
| `offer-no-seed` | Decline; access cannot produce without seed | Never accepts or owes rent; no harvests; nutrition death in month 11 |
| `offer-short` | Six-month forecast selects eight-grain offer | Two harvests; first installment paid; nutrition death in month 20 |
| `offer-long` | Eighteen-month forecast selects one-grain alternative | Survives; seven harvests; four installments paid; four food provisions missed |

The horizon pair has the same opening state, two offered rights on the same plot,
prices, candidate-production horizon (six months) and inventory-buffer target.
Only decision rollout length changes. The two offers score identically over six
months because neither payment is visible; the lower-ID expensive offer wins the
stable tie. Eighteen months include the first bill at month 13 and next harvest
at month 14. The cheaper offer then scores strictly better. This is evidence of
a horizon blind spot, not a preference for paying more. Reversing table storage
preserves both decisions; renumbering tied offers need not.

The expensive agreement takes the crop proceeds needed for food. Eight grain is
eventually collected against the first bill; later annual installments remain
unpaid after death. The cheaper agreement still causes food shortfalls under
mandatory payment priority. Longer foresight improves this choice without making
the contract free of harm or proving that all future payments are sustainable.

Tests also delay acceptance until month 2 and verify the first bill occurs in
month 14, with no pre-acceptance access or month-13 charge. Duplicate, incompatible,
unknown and out-of-phase acceptance fail without partial changes. All four runs
match CPU/reference state, reports and ledger; replay, monthly batching, reversed
offer/right storage and continuation after acceptance preserve results. Selected
first-month forecast rows match actual agreements and debt as well as stocks.

The portfolio chooses at most one offer per month, assumes no future acquisitions
within rollouts, and cannot cancel or renegotiate an accepted agreement. Tests:
[access decisions](tests/access.rs). Raw traces are ignored under
`output/economics/offer-*.md`.

## Payment-policy comparison on CPU

Historical baseline before the candidate change; current outcomes are in
[dated production candidates](DATED-CANDIDATES.md).

Four sixty-month CPU runs compare DebtFirst with ProtectEssentials. Each pair has
identical opening state, one-grain annual rent, production policy and six-month
forecast. Only the payment policy changes. DebtFirst remains the default.

| Fixture | Food deficits | Rent collected | Active closing months in arrears | Blocked new-work requests | Harvests / survival |
| --- | ---: | ---: | ---: | ---: | --- |
| `payment-debt` | 4 | 4 | 0 | 0 | Seven harvests; survives months 1–60 |
| `payment-protected` | 4 | 4 | 2 | 0 | Same seven harvests; survives months 1–60 |
| `payment-trap-debt` | 6 | 1 | 0 | 0 | No harvests; dies in month 18 |
| `payment-trap-protected` | 6 | 0 | 6 | 7 | No harvests; dies in month 19 |

Warmth shortfall is zero in all four. Active arrears counts only reports without
a terminal state; requests can still be blocked in the month whose Close becomes
terminal. The scarce pair runs months 13–72 and starts with one grain, one seed,
no active crop and a payment immediately due. Debt after death is retained, not
collected: total closing months with arrears are 48 and 60 in the scarce pair,
which should not be mistaken for those agents spending that long alive in debt.

In the normal protected run, month 13's last grain is kept for consumption.
The existing crop finishes in month 14 and clears rent from its output; the same
one-month delay occurs at 37–38. Both installments eventually get paid, alongside
the other two due dates. No new planting request is blocked during those delays.
But protection does not add food: both policies still miss four provisions over
the full run and have the same harvest dates. It shifts shortages rather than
eliminating them.

In the scarce protected run, the first grain feeds the agent instead of paying
rent. Unpaid access then blocks planting. With no crop already underway there is
no harvest to clear the debt, and protection extends survival by only one month.
The debt-first forecast also fails to find a viable recovery and completes no
crop. Neither policy establishes sustainable production from that opening.

Protection is generic: it uses consequence-bearing requirements and their enabled
consumption recipes, including shared-stock demands, existing fulfillment and
integer lots. It limits payments at Due and ClearArrears without changing phase
order or making a physical escrow. It cannot protect grain against unrelated
later spending or supply missing seed/capacity.

All four CPU runs match reference state, reports and ledger. Tests compare
identical opening obligations, abundant and inactive-demand controls, missing
recipes, shared stocks and rounded lots; verify policy/protected-stock tampering
rolls back; and check post-harvest cure, forecast agreement, replay, batching,
catalog-order invariance and continuation at both payment boundaries.
Source: [payment-policy tests](tests/payment_policy.rs). Raw CPU traces are ignored
under `output/economics/payment-*.md`.

## Combined production bottleneck audit

Historical baseline before the candidate change; current outcomes are in
[dated production candidates](DATED-CANDIDATES.md).

The [production audit](PRODUCTION-AUDIT.md) compares eight rent/tool/experience
variants with eighteen-month decision forecasts, plus four diagnostic controls.
Experience saves labor but does not increase harvest frequency; offered tools
are never purchased. An already-owned tool also leaves harvest timing unchanged.

Idle-plot traces identify the six-month, consumption-only candidate gate:
available seed, access and labor do not produce a planting intent while six
months of food are in stock. The longer evaluator sees rent but cannot choose
work excluded by candidate generation. An explicit legal crop/fuel calendar
completes ten harvests, meets both needs and pays rent, ending with 21 grain.
That establishes physical feasibility; autonomous policies complete seven or
eight harvests. This audit changes no core production rules.

## Alternative food and foraging

[Foraging results](FORAGING.md) compare six nine-month CPU scenarios. Abundant
food suppresses unnecessary foraging; spare labor lets it bridge a shortage while
preserving the crop. Under normal capacity, six-month decisions abandon the crop
for immediate food and later run short of warmth. Eighteen-month decisions preserve
the harvest, accepting one temporary food deficit. An urgently lethal need reverses
that immediate choice, extending survival without establishing recovery.

Consumption, candidate coverage, payment protection and buffer scoring now support
alternative food stocks. Shared wild supply is finite, replenishes at Open, and
uses existing joint stock/labor reservations. See the linked document for controls,
boundaries, limitations and exact measurements.

## Validation

`cargo +1.92.0 test --locked` passed 76 tests: the original CubeCL add-one smoke
test, 12 initial integration tests, 10 repeated-harvest/warmth tests and ten
generic-condition tests, plus eight forecasting tests and eight equipment/barter tests, plus four experience tests and five commitment tests, plus four access-offer tests and four payment-policy tests, plus three dated-claim unit tests and seven substitution/foraging tests. None failed or was ignored. `cargo +1.92.0 fmt --check` and strict all-target Clippy passed.

Checks include analytical balances and harvest timing; all 59 CPU/reference
comparisons; raw-effect accounting independently of grouping; committed-ledger
replay; monthly versus nine-month batching; in-memory continuation from every
barrier; duplicate batches; bounded effect buffers; gross overspending despite
positive net receipts; integer overflow; invalid process progress/output; shared
plot contention despite overlapping rights; reordered table storage; changed
process IDs/names/duration; output entitlement distinct from land ownership; and
missing or too-distant process chains. The latter checks show that a short planning
horizon can miss useful long-duration production, rather than silently treating
future output as current food.

Additional checks cover seed accounting at every boundary over repeated cycles,
finite wood/fuel accounting, period-specific fulfillment of both needs, per-month
shared labor limits, identical-opening-request priority comparisons, abundant and
inactive-demand controls, missing inputs, shared consumption stock, duplicate-need
rejection, and continuation at planting/harvest/cycle boundaries. Reversing need
and catalog storage order preserves results. A full fuel buffer is correctly
reported as no deficit, not credited to an unrelated active crop.

Raw CLI traces are local ignored artifacts under `output/economics/`. The editable
fixtures live in [scenario.rs](src/scenario.rs); assertions are in
[the original integration tests](tests/economics.rs) and
[the repeated-harvest/warmth tests](tests/repeated_and_warmth.rs). Run commands are in [README](README.md).

## Limits and next evidence

Conditions and output are deterministic; no random seed is needed. No weather,
general market matching, finance, tenure changes or cancellation commands are
implemented. Generic terminal transitions are implemented, including an illustrative
death state, but there is no calibrated physiology, mortality hazard or population
demography. Participants currently have one capacity and a list of ranked requirements. Each instance has
one operator/input owner, one output beneficiary and at most one persistent
exclusive asset plus one optional stage tool.
Only abort-on-missed-stage behavior exists. Consumption definitions accept one
stock input and produce one fulfillment kind. Definitions are Rust catalog data,
not an external DSL.

The planner is a two-link heuristic, not a general graph optimizer. It does not
secure future labor, jointly optimize need chains, value risk or replan inside a matching
round. Rights and configuration are fixed for a run. Checkpoints are in-memory
clones of the simulation; there is no on-disk serialization or crash durability.
Replay accepts trusted internal records, not arbitrary external instructions.

Ledger history and completed instances currently remain in memory. The kernel
receives host-grouped buffers each settlement and has not been tuned for scale.
No speedup, CUDA compatibility or general CPU/GPU determinism claim follows from
these small exact-integer runs. Multi-need planning remains a bounded per-need
heuristic with shared settlement budgets; it is not a global optimizer. Static
priority and the shared forecast horizon can favor unnecessary buffer work over
critical production. The optional forecasting portfolio improves the tested
harvest case, but horizon limits, omitted recovery options and fixed rollout
strategies remain material constraints.
