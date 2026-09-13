# Circulation and workshop recovery

Status: diagnosis and reproduction tooling; behavioral fixes and balance evaluation
remain outstanding. The target is usable circulation, economically viable workshop
operation where real demand permits it, and lawful recovery of abandoned wealth.
No default change or successful balance claim follows from this audit.

## Reproduce the opening diagnosis

Run against an exported history, not a binary checkpoint:

```sh
python3 scripts/audit_circulation.py output/contract-staffing-comparison/on/1024-credit.json > output/contract-staffing-comparison/on/1024-credit-circulation.json
python3 -m unittest discover -s scripts -p test_audit_circulation.py
```

The audit counts the cash compartments used by `History::economy_residuals`,
including both export escrows, service escrow, expedition purses and relocation
purses. It separates cash stocks from cumulative flows and flags retained wallets
without declaring them ownerless. It does not infer inaccessible currency merely
from a positive balance. Diagnostic household subsets overlap and must not be
added to the money total.

The existing frozen seed-1024 comparison used 32/32 terrain/ecology resolution,
five founding civilizations, 600 months, delivery-paid exports, automatic service
procurement with a 0.25 surplus share, and contract-aware staffing. See
[the staffing comparison](contract-staffing-comparison.md). These are observations
of those existing outputs, not new simulation runs.

| Endpoint / cumulative measure | Credit | Credit + issuance |
| --- | ---: | ---: |
| Total cash | 50,000.009 | 51,250.000 |
| Household cash | 42,129.23 | 43,813.48 |
| Town operating cash | 7,567.71 | 7,082.41 |
| Council cash | 297.78 | 346.94 |
| Institutional cash | 5.28 | 7.18 |
| Operator cash | 0 | 0 |
| Cash in vacant households or households registered at abandoned sites, union | 17,271.94 | 18,013.69 |
| Of that, cash without known living members | 16,972.49 | 17,556.07 |
| Operator revenue minus wages and rent, cumulative | -587.78 | -425.18 |
| Operators with negative operating margin | 9 / 10 | 11 / 11 |
| Relative money discrepancy | -1.783e-7 | -4.186e-9 |

Other cash compartments contain the rounding-sized remainder. The baseline and
credit arm had identical non-credit histories, so these observations do not imply
that lending caused these outcomes. No loans were committed.

### Household interpretation

These runs have a named-demography record but `individual = false`. They use
aggregate population authority with sparse named people. That distinction is
critical: an absent named adult is not proof that no dependents or anonymous
beneficiaries remain. The retail allocator explicitly distributes unrepresented
age-band food need across household accounts. A vacant household can therefore
still have positive food need. It must not be confiscated simply because a query
finds no named residents.

The union contains 33 retained household records, one with a known living member.
The 32 vacant records contain no known living members in either arm. A household
registered at an abandoned town is also not automatically extinct: relocation,
known heirs, existing ownership and outstanding obligations need review.

Earlier discussion described these as potentially stranded balances. That is a
hypothesis requiring a succession and access audit, not evidence that all 17,272
units are immediately recoverable. The existing lost-relocation estate return
handles a narrower case; vacancy retains the wallet and property deliberately.

### Workshop interpretation

The no-issuance arm's largest operator paid for 14.47 worker-months but completed
8.99. It received 444.55 in service revenue, paid 593.24 in wages and 12.92 in rent,
and closed for exhausted working capital. The other nine closures cite six months
without sufficient orders or inputs. Across all firms, nine lose money before
financing. Aggregate conservation does not establish business viability.

Current shift demand has a minimum 0.05 worker-month floor and uses lagged completed
work with headroom. A due contract can raise demand. Neither a past completion nor
a funded service fee guarantees current materials or useful output demand. Wages
are paid for funded attendance; service income depends on completed work. Removing
wages after unsuccessful work would hide the problem and change the employment
contract. Inflating service prices would shift losses to towns.

## Behavioral work to complete

1. **Household continuity and reclamation.** Define claimant eligibility using
   living members (including children and travelers), recorded kinship, migration,
   population authority and legal claims. Resolve valid inheritance first. Add a
   dated waiting period and explicit local authority for unclaimed estates. Preserve
   source wallets, transferred amounts, beneficiary and causal records. A vacant
   representative is insufficient evidence. Retained ownership shares and later
   income need settlement as well as current cash, or the sink simply refills.
2. **Abandoned settlement recovery.** Give recovery a physical access/travel and
   carrying-cost boundary. Reuse actual return/reoccupation or recovery journeys;
   do not teleport abandoned town money into a distant council. Keep existing
   household, institutional and artifact claims separate from municipal property.
3. **Workshop request feasibility.** Observe current production orders, installed
   capacity and available inputs before committing labor. Keep unconstrained
   requests, feasible work, funded attendance and completed work separately visible.
   Test whether the forecast meaningfully reduces paid but unusable shifts without
   preventing a temporarily idle firm from restarting when demand returns.
4. **Circulating household savings.** Evaluate inheritance/recovery and existing
   family support first. Then examine savings investment or lender participation
   against explicit voluntary terms, food reserves and credible repayment sources.
   Wealth outside current lender categories is not evidence for forced lending.
5. **Underwriting.** Audit cost time windows and loan-funded cost treatment using a
   profitable cash-short operation with exact receipts. Compare rejection evidence
   before loosening thresholds. Distinguish liquidity gaps from persistent losses.
6. **Balance gates.** Matched controls, multiple seeds and longer runs must measure
   food access, production, completed/paid work, business margins, council services,
   cash ownership, estate recovery, debt and conservation. Keep failed firms and
   legitimately retained estates in results. Check boundary replay and checkpoint
   continuation for transfers. Survival of every business is not the target.

## Verification of the audit

Three CPU fixtures pass: all ten cash compartments counted once with issuance;
positive living-traveler evidence retained despite vacancy; abandoned non-vacant
wallets included without counting dead members as living. Both existing histories
parse successfully. No new GPU run was needed for this read-only diagnosis.

### Follow-up claimant inspection

In the same no-issuance endpoint, traversing recorded parent/child links from
vacant household heads finds two estates with a known living descendant: households
52 and 58 both lead to person 192, resident in household 46 at site 3. Their wallets
hold 238.28 and 1,010.58 respectively. Existing succession requires an unoccupied
head; this is a concrete case for allowing a resident household to inherit another
estate's economic interest without installing its head in two households. It does
not establish that no other relatives have claims. No transfer has been applied.

[Demand-aware workshop staffing](demand-aware-workshop-staffing.md) is the first
behavioral pilot following this audit. It caps shifts by current recipe demand;
input feasibility and long-run economic viability still require evidence.

[Local household estate inheritance](household-estate-inheritance.md) now provides
an opt-in same-site sole-descendant transfer of cash and ownership, preserving
identity and excluding live members, travelers and indebted estates. The first
screen exposed repeated residual transfers, so the revised policy protects three
months of recorded food need before transferring cash. This is an implemented
continuity pilot, not completion of unclaimed-estate or abandoned-site recovery.


### Local cash review follow-up

The [unclaimed-estate pilot](unclaimed-estate-reclamation.md) now exercises funded
local administrative review while protecting known kin and a food reserve. Its
matched 50-year comparison reclaims substantial cash but does not improve the
population/food outcomes or rescue workshop operators. It remains opt-in. This
closes a local transfer mechanism, not the broader circulation objective or
physical recovery from abandoned settlements.


### Input-aware workshop follow-up

[Current-stock staffing](workshop-input-feasibility.md) now separates eligible
industrial orders from household food processing and bounds private requests
against shared stock, knowledge, storage and residue capacity. Controlled tests
and three 50-year seed comparisons demonstrate the connection. Results vary:
substantial improvement in seed 409, lower losses but less work in seed 1024,
and slightly worse margins in seed 256. Unused GPU recipe labor and the mismatch
between household cash in price signals and actual household purchasing are the
next concrete investigations. Circulation and abandoned-site recovery remain open.

### Distinguish money stocks from repeated spending

The audit now exposes lifetime household wage, dividend, relief and food-spending
counters independently of balances. Missing counters in older inputs are unknown,
not zero. Site summaries group accounts by their **current** home; they are not a
reconstruction of where historical payments occurred. The counters can exceed the
entire money supply because the same currency can be spent repeatedly.

For the input-aware staffing seed-1024 baseline (50 years, `e401393` executable),
households have 43,091.81 cash at the endpoint, but their cumulative wages are
1,434,172.55, dividends 28,796.13, relief 7,261.48 and food spending 1,427,021.44.
These flows are not additional assets or a complete cash-flow reconciliation;
capital, inheritance and other recorded transfers also exist. They contradict an
interpretation that currency simply stopped moving throughout the whole run.

Distribution and access are more specific problems. Seven of the twenty accounts
currently registered at Litugie hold less than one currency unit, despite total
local household cash of 12,228.59 and a town food stock of 12,876.12 food equivalents.
Across all twenty accounts, unmet monthly need totals 53.25 food equivalents.
Elsewhere, Lethor na Refojea has no food stock and about 9,989.35 household cash.
The food-stock observation is an endpoint; zero terminal stock does not establish
zero harvest throughout that month or year. The next interventions should separate
household purchasing distribution from physical import and production constraints.

Five audit fixtures now pass, including large lifetime flows that do not change
cash totals and missing-counter handling. The twelve monetary runner fixtures also
pass. Raw runs and audit outputs remain under ignored `output/`.

The [paid bulk-stock recovery pilot](abandoned-stock-recovery.md) now connects
retained source inventories to occupied buyers through bounded round-trip land
cargo. It preserves estate cash and claims rather than confiscating them. Initial
scope is direct routes under one administration; broader access and balance tests
remain outstanding. This is resource circulation, not a solution to retained money.


### Cross-border recovery screen

[Abandoned stock recovery](abandoned-stock-recovery.md) now permits paid purchases
between different administrations under standing trade permissions and absent
active war. A controlled GPU fixture demonstrates delivered matching material
actually enabling production. Eight matched seed-1024 runs recover small amounts
of food/goods but leave operator work unchanged and slightly reduce endpoint
population. The access restriction was real; relaxing it alone is not an economic
solution. Review downstream input targets and material substitution next, while
retaining the separate household purchasing-power and firm viability workstreams.


### Useful tool reserves

[Tool substitution retention](tool-substitution-retention.md) fixes an inconsistency:
held alloy tools covered service demand without receiving reserve targets, leaving
them unprotected by the existing export reserve fraction. Six planner tests and
the GPU finite-tool/checkpoint fixture pass. Four seed-1024 runs show more completed
operator work and a modest margin increase, but all firms still close by year 50.
Imported substitute selection, sustained firm demand and household purchasing-power
circulation remain unfinished; this correction is not a workshop viability claim.


### Recovered tools actually enter use

[Service-equivalent recovery](abandoned-stock-recovery.md#service-equivalent-tool-recovery)
now buys cheaper usable alloy tools against the shared basic tool-service deficit,
counting held, incoming and contracted variants. Four seed-1024 comparisons empty
the abandoned copper-tool stock and show actual wear/scrap at the buyer; private
operator work remains unchanged. Recovery now has an observed stock → paid cargo
→ tool service → wear/recycling path. This does not establish general recovery
coverage, profitable workshops or restored cash circulation. Sustained paying
customers and household savings returning to useful production need further work.


### Due contracts and execution shortfalls

[Funded workshop share](workshop-contracted-share.md) now lets a due commitment
claim more than the ordinary lease-proportional demand share, while retaining
real order/input/equipment/work/cash limits. Three paired 50-year seeds have mixed
outcomes; this is a contractual allocation correction, not a balance victory.
The new audit points to paid-but-unproductive work in seeds 256/409, with virtually
no unpaid completed-service invoices. Inspect shared GPU craft labor and competing
recipes before interpreting these firms as merely short of customer money.
Household material purchasing still needs its own finite ownership/consumption
path; it has not been implemented by this change.


### Confirmed craft-labor reservation leak

[Prepaid craft isolation](prepaid-craft-labor.md) reproduces and fixes unrelated
household recipes spending prepaid industrial attendance. The red fixture used
only five of eight reserved metalworking months despite sufficient input/order;
the corrected fixture uses eight within the same ten total months. Three-seed
results are mixed, with markedly smaller losses in 256 but greater payroll and
losses in 409. This is a verified cross-system labor correction, not proof that
all firms are viable. Review earlier asset/service labor and input timing next;
do not assume larger budgets alone fix paid-but-unproductive attendance.

### Construction reservation follow-up

Aggregate construction now honors prepaid workshop attendance just as named
construction already did. A red/green hardware fixture demonstrates the specific
leak and its bounded correction. Three matched 50-year runs remain mixed; two
have smaller operator losses but neither becomes profitable. See the construction
follow-up in [prepaid craft labor](prepaid-craft-labor.md) for settings, measured
work/pay, food access, cash residuals and remaining service-priority questions.

### Workshop hiring versus earlier public services

Workshop hiring now subtracts current public commitments from its prior craft
ceiling and allows for installed water-service operation before committing wages.
The paired actual-payroll fixture and checkpoint tests pass. Across three matched
50-year runs, paid attendance completion exceeds 99.5% and cumulative operating
margins become positive in all three; total output and welfare remain mixed.
[Service-aware hiring](workshop-service-hiring.md) records the full before/after
results and the remaining circulation/throughput questions. This does not complete
the broader economic recovery objective.

### Quote demand follows account ownership

Adaptive quotes no longer distribute household savings across municipal material
orders. Household support is capped to a monthly food budget; town cash retains
its own order basket. Focused tests and continuation checks pass. The matched
three-seed comparison produces lower material quotes and still-positive operator
margins, but lower populations and mixed hunger. See
[ownership-aware quote demand](market-demand-ownership.md). This is an accounting
of plausible buyers, not evidence that cheap nominal quotes solve circulation.

### Household earnings follow adult cohort shares

Aggregate resident payroll now weights represented adults as well as occupations,
while named-sector earnings retain their actual-work overrides. It redistributes
existing payroll instead of raising it. A long-run failure exposed tiny negative
dividend remainders; individual dividends are now bounded by the remaining budget.
Four focused GPU checks and 14 CPU household tests pass. All three corrected
50-year runs complete: two show materially higher population and lower hunger,
while the third is slightly worse. Workshop margins remain positive. See
[household adult payroll](household-adult-payroll.md) for the initial failure,
correction, comparison and outstanding supply/access questions.


### Harbor shortages enter production planning

Existing surveyed ports now request their finite build/repair materials through
ordinary production targets, protecting the working reserves used by installation.
Seven CPU production tests and two focused GPU construction/procurement fixtures
pass. Three matched 50-year runs have lower ending hunger, mixed industrial work,
and still-positive operator margins, but no additional commissioned ports. One
port opens a year earlier. Inspection now isolates annual leftover construction
work as a further bottleneck even where materials are stocked. See
[harbor production demand](harbor-production-demand.md) for settings, outcomes and
limits; circulation and transport recovery remain unfinished.


### Annual harbor work: three experiments rolled back

Tested protected leftovers, explicit staffing demand, and a bounded annual public
work allowance. All nine 50-year runs completed, but each arm lowered population
in every seed. The public-work variant commissioned five additional ports and
increased workshop activity, yet food access worsened in two seeds and crew costs
rose. None is retained in current source/defaults. See
[harbor labor experiments](harbor-work-allocation.md) for the complete comparison.
The next concrete blocker is equipment allocation: some important ports have full
timber/masonry and 22–23 kg generic tools in town, but cannot install tools because
the 0.5 kg/resident working-reserve floor exceeds their stock. They have no spare
copper/bronze either. Test desired versus indispensable tool reserves and finite
sharing first; then alternatives/recovered metal where actually available, alongside
route use and recurring crew costs.

### Scarce-tool investment and source diagnosis

Two more three-seed, 50-year screens tested investing up to 10% of scarce tool
stock, alone and alongside bounded public harbor labor. The combination opened
fleets at every surveyed port and substantially increased industrial work in one
seed, but food/population outcomes remained mixed, including sharply worse hunger
in seed 1024. Both experiments were rolled back. See the follow-up in
[harbor work experiments](harbor-work-allocation.md).

The circulation audit now separates monthly ore allowance buffers from canonical
source stock. Settlement clears those buffers every month: zero never proved
exhaustion. Seed 256's registered deposits have no supported metal output, while
seed 1024 retains metal deposits at declining/abandoned sites. Six audit tests
pass, including this distinction. Investigate access to finite sources and useful
transport next; completed infrastructure alone did not solve circulation.

### Stored ore becomes an explicit tool-input offer

The next bounded connection works already extracted ore rather than opening a
new deposit. Tool planning can use a supported imported-metal chain even when the
town's own mineral points to another tool type. Recovery can evaluate that chain
against existing fuel, stocks and pending deliveries, retaining ordinary payment,
route, inventory and arrival rules. GPU controls require arrival, fuel and workers
before malachite produces copper/tools; the recovery fixture checks duplicate
purchases, estate payments and continuation.

The first three-seed screen exposed redundant buying: ore and finished tools were
reserved for the same deficit, then ore sat unused. Automatic recovery now visits
finished goods across sources before raw tool inputs. See
[stored ore and tool planning](recovered-ore-tool-planning.md) for the comparison
and limits. This is a causal capability, not proof that workshop activity or
household food access has improved across the ensemble. Unworked deposits and
the gap between household savings and public import funding remain open.

The final processing-work ceiling reduced seed 1024's idle ore purchase to 1.70 kg
(4.25 currency), but it remained unused at year 50. Across twelve runs in four
iterations, the last version left population, hunger and operator work effectively
unchanged from baseline. The controlled capability is retained under optional stock
recovery, with its limitations documented; it is not an economic recovery success.
Next prioritize a funded processing customer or household-backed import demand,
rather than assuming that additional raw inventory creates either.

### Food imports: larger orders and lower seller reserves

Nine further fifty-year runs tested a quarterly food ceiling of 54 rather than
five kg per resident, then paired twelve- and six-month seller reserves. Larger
orders worsened seed 1024's ending hunger and population; lower reserves increased
food sales and operator work but did not recover baseline population. Seeds 256
and 409 recorded no ordinary food sales and unchanged reported outcomes.
Catalog-loading controls matched the larger-order results. All runs completed
with small monetary residuals, but neither change demonstrated a circulation fix.
The candidate was rolled back; see [food import screening](food-import-circulation-screen.md).
Next gather request-level failure reasons before connecting household funding to
imports, so inaccessible supply is not mistaken for a shortage of cash.

### Request-level evidence changes the next priority

[Food request diagnostics](food-request-diagnostics.md) now persist bounded,
non-causal per-site counters. Controlled tests distinguish supply, access,
freight, cash and batch limits. Three fifty-year exports exactly match baseline
apart from the added observations. Seeds 256/409 have 651/621 requests blocked by
inaccessible eligible surplus; neither dispatches ordinary food. Money is never
the tightest eligible food-order limit in these three runs. Defer household-funded
imports and investigate selective, economically useful route completion first.
This is evidence about ordinary town food purchasing, not proof that household
food affordability or workshop finance is adequate.

### Selective food connections

A [bounded annual connection rule](selective-food-connections.md) now targets one
prospective food-supply pair plus ports carrying committed cargo. Extra work stays
inside public-service limits; selected scarce-tool investment is capped at 5%.
The economy catalog can disable the rule for comparisons. Six fifty-year runs
and four century runs verify real new food dispatches in previously disconnected
worlds, with finite money/materials/work, but mixed population outcomes. The 10%
variant was rejected. The retained 5% mechanism is a functional connection, not
a declaration that economic recovery is solved: century runs still show very
little late operator activity, and seed 256 loses its early population advantage.

### Actual equipment use and repair targets

[Workshop utilization repair](workshop-utilization-repair.md) corrects the
specialized planner's assumption that hypothetical household craft capacity has
already replaced completed work in installed equipment. Targets now retain the
utilized portion of existing assets; execution still requires materials and work.
Three matched fifty-year runs and a paired century extension are mixed: seed 409
has sustained additional operator business at year 100, while seeds 1024 and 256
still have no operators at year 50 and worse ending hunger. This is a maintenance
correction, not a declaration that circulation or industrial recovery is solved.

### Recovery across inner continents

[Buyer-funded sea stock recovery](sea-stock-recovery.md) extends paid collection
beyond direct land routes. It uses existing surveyed lanes, surviving harbors and
the buyer's funded fleet, recording both physical endpoints while charging crew
service only to the buyer. Eleven completed comparison runs and an integration
failure/fix are documented. The final century runs return tools/bricks in seed
1024 and flax in seed 409, but do not improve broad economic outcomes. Unmined
abandoned deposits and inactive estate money remain separate unfinished problems.


### Shared inheritance and idle cash review

[Shared-descendant inheritance](household-estate-inheritance.md#shared-descendants-and-local-review-screen-september-2026)
allows multiple eligible local heirs without duplicating the estate. Three
fifty-year runs and a paired century find little additional cash released. Enabling
existing unclaimed-estate review releases 537–3,407 currency units in each of three
fifty-year worlds, but scarcely changes workshop business and yields mixed welfare.
The recipient is the town operating account. Trace actual procurement and input
bottlenecks next; neither mechanism establishes that circulation is solved.


### Rejected general recipe-work reservation

[A three-seed staffing screen](recipe-work-reservation-screen.md) tested a
2%-of-workforce, feasibility-capped recipe allowance beyond tool maintenance.
Operator work falls in all three worlds; welfare improves in two but worsens in
409, which also loses active operators. The shader change was reverted. Household
material purchases and durable possession remain the next missing demand path;
reassigning municipal labor does not substitute for actual customers.
