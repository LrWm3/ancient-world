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
