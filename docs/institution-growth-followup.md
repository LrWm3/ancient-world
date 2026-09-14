# Institutions in growing purchasing worlds

This follows [municipal purchasing assistance](municipal-food-relief.md).
Population and settlement expansion no longer imply that institutions work:
only one institution per municipal-relief world met the operational predicate at
year 200. The present task compares existing mechanisms before inventing another
funding source or changing the service scheduler.

## Implementation

Two existing Culture fields now have ordinary system-registry controls:

- `institution-working-core`: basic operation uses the space demand of at most
  two local members. Full membership still determines expansion demand. It does
  not conjure staff, construction materials, usable buildings, money or work.
- `institution-operating-funding`: a quarter's planning captures a year of upkeep
  and component wear replacement, minus current institutional cash. Requests share
  0.5% of town cash. Conditional ceilings still require actual administrative work
  and live cash to collect. It does not create a public subsidy or protect spending
  ahead of earlier claims.

Both are opt-in, query actual archived Culture state, and support explicit disable.
Existing archives retain their current values when loaded. These policies were
previously available through the cultural calibration example and library fields;
they are now available to ordinary native runs. No underlying space, funding,
work-priority, or scheduling equation changes accompany registration.

## Experiment

Fresh worlds: seeds 1024/409, terrain/ecology 32/32, one geological epoch, five
initial civilizations/600 people, 200 living years, aggregate demographic authority.
Initial-town nutrient retention 0.95, global finite geological phosphorus release
5e-7/month. Ordinary household purchasing, municipal relief, household solidarity,
estate inheritance/reclamation, wealth tax, council welfare and named office
service are enabled, as in the previous municipal comparison. No harvest,
provisioning, mortality or founding threshold changes.

Run the municipal comparison command and add `--enable-system institution-working-core`
for the first arm. Add `--enable-system institution-operating-funding` for the
second. Neither arm enables named institutional administration or essential-first
work. Thus these results cannot evaluate those separate policies.

Inspect operating institutions using the existing predicate (active identity,
leadership, readiness and usable building), not merely retained names. Compare
population, food gaps, service readiness, treasury and actual institutional dues.
Dues include initial founding transfers; they are not exclusively operating grants.
Keep native exports and logs under ignored output/growth-rounds.

## Results and interpretation

At year 200 (operational / active institutions):

| Seed | Policy | Population | Active towns | Institutions | Final-decade births / deaths |
| --- | --- | ---: | ---: | ---: | ---: |
| 1024 | Municipal baseline | 1,185 | 7 | 1 / 16 | 272 / 238 |
| 409 | Municipal baseline | 1,219 | 10 | 1 / 20 | 280 / 243 |
| 1024 | Working core (corrected rerun) | 1,197 | 10 | 5 / 24 | 280 / 300 |
| 409 | Working core | 1,116 | 10 | 4 / 20 | 260 / 252 |
| 1024 | Core + operating funding | 1,156 | 9 | 13 / 20 | 272 / 298 |
| 409 | Core + operating funding | 1,184 | 10 | 18 / 20 | 277 / 275 |

Working space alone helps some institutions, but the joint space/funding policy
has the much larger endpoint effect. Seed 409's cumulative dues rose from 9,703
to 12,938 currency units; seed 1024's rose from 9,553 to 10,609. Ending institutional
cash was only 46 and 28 respectively, so small institutional balances need not
mean failed service: recurring transfers can maintain operations.

This is not a population-growth improvement. Seed 1024's funded arm ended with a
negative final-decade demographic balance. Its whole-run physical food gap rose
from 0.141% to 0.470%, while the purchasing access gap fell from 1.176% to 0.882%.
Seed 409's corresponding gaps changed from 0.232% / 1.090% to 0.229% / 1.124%.
These are demand-weighted shortfall measures, not counts of starving people.
Long nonlinear trajectories do not identify which institutional service caused
any population change. Keep these policies opt-in; do not describe service
recovery as evidence that food supply or survival is solved.

## Failure found during the comparison

The working-core seed 1024 initially stopped at month 2052. New household 99 in
site 8 selected person 236 as its head because that person remained the historical
civilization leader and no existing household claimed them. That person had died
at month 1296. Household initialization checked prior assignment but not life.

Initialization now requires a living leader before reusing that identity. Otherwise
it follows the existing new-representative path. It neither resurrects the old
person nor silently changes political succession. Household validation now reports
the offending record and month instead of an undifferentiated lineage error.
The focused GPU regression covers an unused deceased leader and verifies that
household heads are living while the original death and political identity remain.

This is a sparse-head initialization correction, not a complete roster or political
succession redesign. The funded arms and working-core seed 409 completed without
this failure before the correction; the failed arm was rerun with the correction and completed all 2,400 months
in 92.6 seconds. It ended with 1,197 people, ten active towns and five operational
institutions. Its final-decade demographic balance was still negative.

## Verification

- Regular library suite: 216 passed, 158 hardware/long-running tests ignored.
- Explicit hardware founding regression: passed on Quadro RTX 5000 Max-Q / Vulkan.
  The fixture opens a daughter site with an unassigned deceased historical leader
  and checks initialization directly, independently of political validation.
- System registry hardware apply/disable/archive/resume fixture: passed with both
  new flags; ordinary registry tests also pass.
- Growth-summary Python tests: five passed.
- Experiments use one GPU/backend and two seeds at diagnostic resolution. They
  establish neither cross-hardware equivalence nor default-resolution balance.

- Controlled hardware space × funding × working-core fixture: passed. A small
  funded building recovered with core space accounting; unsupported institutions
  did not recover merely because they had a building.
- Clippy (library/native binary, warnings denied), diff whitespace check and
  repository artifact policy: passed. Raw runs remain ignored.

## Next investigation

Keep funding and space available as separate experiment controls. Before adding
another cash source, inspect the late physical food shortages and measured service
outputs in the funded seed 1024 trajectory: requests, actual work, production and
food entitlement. Endpoint readiness is useful but does not establish that schools,
guilds or religious services delivered enough useful work to change those outcomes.
A funding-only arm and a larger seed ensemble would further distinguish the joint
policy's benefits and costs. No default changes are justified by these two seeds.
