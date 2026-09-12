# Institutional operating-budget pilot

`Culture.institution_funding = Operating` enables dated requests for operating
funds. Legacy remains the default during calibration; the balance runner exposes
`--operating-institutions`. This is a game allocation rule, not an estimate of
historical charitable finance.

At Reserve, active institutions with local representatives quote a year of
administrative fees (2 abstract currency units) and component replacement at
current prices, wear and disruption. Existing cash reduces the request. Lost,
destroyed or inaccessible buildings receive no repair allowance. Expansion and
accumulated repair backlog are not funded by this quote. Legacy brick structures
quote their own embodied brick and wear rule.

Requests share a ceiling of 0.5% of opening town cash in proportion to unmet
operating need. Zero demand gets no share; allocations never exceed demand. These
are **conditional ceilings, not escrowed cash**: earlier spending retains its
existing priority. The existing administration action must still receive and use
work in Respond. It transfers at most its captured ceiling, current unmet need
and live town cash. Changed prices do not enlarge a captured request. A depleted
town can pay less; unused ceilings expire rather than moving cash retroactively
ahead of earlier claims.

Payments use the actual representable town withdrawal as the institution's credit.
Each operating request settles at most once, belongs to its captured month and
institution, and persists through checkpoints. Inactive, moved or canceled targets
cannot collect. Legacy plans lacking a funding quote use the existing small-donation
schedule; changing the policy after planning does not change captured requests.

This funding policy does not add a second source of money. Without the optional
named-administration pilot below, fundraising still competes within the generic
cultural action bundle. Realized collection can retain execution-order effects
when live cash or administration work is scarce. The next review must distinguish requested
funding, work actually provided, money collected, and service delivered. More
money alone cannot fix insufficient usable space; see the controlled comparison
in [institution allocation balance](institution-allocation-balance.md).

## Verification

The operating-budget GPU fixture checks read-only forecasting, missing work, a
changed policy after planning, depleted cash, filled reserves, stale and duplicate
collection, inactive/moved institutions, legacy-plan import, inaccessible buildings
and fractional transfer accounting. A CPU fixture checks proportional sharing,
zero demand, ample/empty budgets and reversed input order.

All six institutional hardware tests and 116 regular library tests pass (107
hardware tests are excluded from the ordinary library run). Full frozen scheduler
monthly/batched/checkpoint equivalence passes on seeds 17, 81 and 256; seed 256
enables operating funding alongside rotating institutional work priority. These
checks establish timing and accounting, not balanced institutional survival.

The funding report accumulates requested amounts, conditional ceilings and actual
payments every month in abstract currency units. Each quarterly request appears
once. Legacy runs have no operating requests, so their zero funding-report totals
do not mean they received no old-style donations; use institutional dues for that
comparison. Requests for a repeated reserve shortfall are counted again next
quarter and must not be interpreted as distinct annual expenses.

The representable-withdrawal correction also applies to legacy donations. The old
formula and work schedule remain, but exact old-version trajectories are not
promised. Matched policy comparisons must use the same new executable.

## First matched 30-year comparison

Implementation `4404756`, seeds 17 and 81, terrain 32/ecology 16, one geological
epoch, sixteen founders, living history, crop yield 0.5, common-food share **0.65
as a test intervention**, individual demography and workshop/agriculture/
extraction/construction refinement, with resolution comparison enabled. Both arms
use this executable; only `--operating-institutions` differs. Stable institutional
work priority remains in use, and named office service is not enabled.

| Seed / policy | Year-30 population | Institutional dues including founding cash | Administration fees paid | Repairs paid | Operational / active institutions |
|---|---:|---:|---:|---:|---|
| 17 / legacy | 1,956 | 1,065.32 | 411.32 | 651.49 | 0 / 32 |
| 17 / operating | 1,939 | 1,348.28 | 372.62 | 975.65 | 0 / 32 |
| 81 / legacy | 1,996 | 1,061.05 | 397.71 | 663.34 | 0 / 32 |
| 81 / operating | 1,986 | 1,394.36 | 365.22 | 1,029.14 | 0 / 32 |

The operating arms request 16,253.52 / 17,066.20 cumulatively, receive conditional
ceilings of 14,908.68 / 14,961.48, and actually collect 548.28 / 594.36 (3.68% /
3.97% of those ceilings). Those sums repeat unmet reserve requests across quarters;
they are not distinct annual bills. The 32 founding cash transfers add 800 to
the dues column.

More collection did not improve administrative continuity: fee payments fell,
repair expenditure rose, and all 32 institutions still had readiness below 0.25
in both seeds and both arms. Poor building condition affected 22 → 23 institutions
in seed 17 and 21 → 21 in seed 81. Treasuries were again nearly empty at year 30.
The pilot does not establish healthier institutions or justify changing defaults.

Only a small fraction of quoted funding was executed. The next integration should
measure which requests actually received administration work and why others did
not, then give those duties explicit participant assignments where appropriate.
The present report cannot separate missing work, canceled bundles, changing reserve
need and later cash shortages as causes of every uncollected ceiling. Repair
spending versus future fee reserves and usable-space service scaling also need
review; simply raising the ceiling is not supported by these observations.

All sixteen sites remain active. Monthly food access gaps rise slightly: 1.9453% →
1.9537% and 1.8991% → 1.9459%. Maximum population residual is zero; maximum monthly
food partition residual is 2.34e-7, and the largest absolute terminal relative
economic residual is 2.60e-6. Both ensembles finish successfully in about 59–63
seconds per seed on the Quadro RTX 5000. This is short game-balance evidence, not
a stability claim.

Reproduce the control with:

```sh
cargo build --example cultural_work_calibrate
target/debug/examples/cultural_work_calibrate --seeds 17,81 --years 30 \
  --common-share 0.65 --individual-demography --workshop-refinement \
  --agriculture-refinement --extraction-refinement --construction-refinement \
  --compare-resolution --output output/institution-funding-control-30.json
```

Add `--operating-institutions` and change the output path to
`output/institution-funding-operating-30.json` for the treatment. The comparison
checker verifies complete reports and matched metadata:

```sh
python3 scripts/compare_food_access.py output/institution-funding-control-30.json \
  output/institution-funding-operating-30.json --allow-difference operating_institutions
```

Raw outputs remain ignored; this document retains the settings, results and limits.

## Named administration pilot

`Culture.named_administration = true` with individual participation enabled assigns
an eligible local institution member to quarterly administration. The balance
runner exposes `--named-institution-administration`. It remains opt-in; aggregate
histories and older archives retain generic administration.

Reserve captures separate dated requests of 0.05 worker-months per institution.
Election convening precedes upkeep, which precedes administration; the existing
stable/rotating policy orders requests within each class. Grants still share the
existing site allowance and each person's available time. An assignment needs its
full minimum grant before reserving; it cannot silently borrow another person's
work. Generic cultural work receives the remaining allowance.

In Respond, upkeep and generic cultural actions execute first. Named administration
then rechecks the assigned member's presence and membership and collects only the
captured funding ceiling and available cash. A canceled generic action bundle does
not cancel this separately assigned duty. The assigned member supplies knowledge;
it becomes visible after generic actions so it cannot invalidate their captured
opening identities. Completed administration consumes its grant once; absence,
stale plans and membership loss leave it unused. Close settles the commitment and
checks that settled operating requests have matching completed administration.

Reports distinguish positive-ceiling funding requests from executed collections,
and report requested, granted and used institution work separately for elections,
upkeep and administration. Executed collection can pay zero if cash or need has
changed. This does not protect next quarter's fees from repairs, increase usable
space, or establish balanced institutional survival.

The named-administration fixture passes seven controlled cases: normal service,
insufficient minimum grant, no allowance, member fully committed elsewhere,
revoked membership, death after reservation and stale month. It checks exact cash
transfers, participant knowledge, no duplicate execution, completed work, invalid
payment receipts and serialized continuation. All seven institutional GPU tests
and 116 regular library tests pass (108 hardware tests remain ignored in an
ordinary library run). The full frozen scheduler monthly/batch/checkpoint test
passes on seeds 17, 81 and 256, with seed 256 enabling named administration and
operating funding together. These are implementation checks; natural balance
comparisons remain a separate requirement.

A follow-up inspection found a specific reserve mismatch: `Facility::repair_order`
quotes materials after withholding 0.5 currency, but `Facility::advance` can spend
the whole treasury when the town already has additional materials. The existing
repair fixture supplies exactly the quote and therefore does not exercise surplus
stock. A surplus-stock counterfactual and matching execution budget are the next
bounded repair task; changing this during the administration comparison would
confound its result.

## Named administration: matched 30-year results

Implementation `576ab70`, seeds 17 and 81, with the same settings as the preceding
comparison. Both arms enable operating funding; only
`--named-institution-administration` differs. Both processes completed, and the
comparison checker accepted complete reports with only that metadata difference.

| Seed / administration | Year-30 population | Positive-ceiling requests / executions | Funding collected | Administration fees paid | Repairs paid | Operational institutions |
|---|---:|---:|---:|---:|---:|---:|
| 17 / generic | 1,939 | 2,912 / 147 | 548.28 | 372.62 | 975.65 | 0 / 32 |
| 17 / named | 1,948 | 2,907 / 232 | 839.22 | 408.18 | 1,231.03 | 0 / 32 |
| 81 / generic | 1,986 | 2,733 / 139 | 594.36 | 365.22 | 1,029.14 | 0 / 32 |
| 81 / named | 2,018 | 2,732 / 196 | 815.86 | 388.50 | 1,227.36 | 0 / 32 |

Named administration completes all its granted work, but grants only 19.95 of
171.15 requested worker-months in seed 17 and 17.10 of 160.65 in seed 81. There
are 1,475 / 1,361 site-quarter administration shortfalls; election and upkeep
requests have none. The narrow service window still prioritizes their work.
Collection includes requests whose conditional ceiling is zero, so completed
administration assignments need not equal positive-ceiling execution counts.

Funding collection rises by about 53% / 37%, and fee payments improve rather than
fall. This establishes a useful participation connection, not viable institutions:
all 32 still lack operational readiness at year 30. Named assignments collect only
5.68% / 5.52% of cumulative funding ceilings. Repair reserve enforcement, useful
space and competing service allocations remain separate constraints. Keep the
pilot opt-in and retain generic administration as the comparison arm.

All sixteen sites remain active. Food access gaps change from 1.9537% to 1.9817%
and from 1.9459% to 1.8662%; this is not a consistent food-access improvement.
Maximum population residual is zero, maximum food partition residual is 2.46e-7,
and maximum absolute terminal relative economic residual is 2.60e-6. Runs take
about 63–66 seconds per seed on the Quadro RTX 5000; these are diagnostic timings,
not an optimization benchmark or long-run stability claim.

Reproduce with the preceding control command, adding `--operating-institutions`
and writing `output/institution-named-control-30.json`. Add
`--named-institution-administration` for
`output/institution-named-treatment-30.json`, then run:

```sh
python3 scripts/compare_food_access.py output/institution-named-control-30.json \
  output/institution-named-treatment-30.json --allow-difference named_institution_administration
```

## Repair reserve enforcement

Repairs now leave up to 0.5 currency in the institution's existing treasury for
its next administration fee. This is a spending limit, not a second account or a
guaranteed future donation. The currently due fee still executes first; a treasury
below that fee can still be exhausted by administration. Room components and
older brick foundations use the same remaining-cash rule.

Procurement runs before the due fee, so it subtracts that expected fee before
quoting against the shared repair budget. Repair execution reads actual remaining
cash and available materials. Extra timber or bricks already in the town cannot
expand the repair budget. Limited reserves can therefore trade slower building
repair for one additional funded quarter. This does not solve low service-space
coverage or recurring income shortages.

The new surplus-stock test failed on the prior implementation (a 0.25 reserve was
fully consumed), then passed with the cap. It covers opening repair balances 0,
0.25, 0.5, 0.75 and 5, checks cash and embodied-material accounting, and exercises
payment of the subsequent fee. A quarterly institution fixture also tests the
older brick path, two paid quarters followed by depletion, and serialized
continuation. The previous quote-only fixture remains as a supply-limited control.

Verification for this increment: 117 regular library tests, seven institutional
hardware tests, and the full frozen monthly/batched/checkpoint test (seeds 17, 81,
256) pass. The surplus-material boundary test is included in the regular suite.

The next allocation review should distinguish basic upkeep from repair work:
`upkeep_work_limit` reserves up to 0.125 worker-months per facility, although its
administrative component is 0.025. Those combined requests precede 0.05-member
fundraising assignments. A scarce window can therefore grant repair capacity while
leaving collection unassigned. Compare an explicit essential-service allocation
against the same requests, total allowance and member availability before changing
priority; do not silently increase labor or infer that equal grants will improve
all institutions. Space-limited readiness needs a separate service-capacity review.

## Repair reserve: matched 30-year comparison

The pre-fix named-administration results above (`576ab70`) are the baseline;
`4727d02` supplies the corrected execution and procurement rule. Seeds 17/81 and
all runner settings match: operating funding and named administration enabled,
common share 0.65 as a test intervention, individual demography and four production
refinements, terrain 32/ecology 16, one geological epoch, sixteen founders and
living history. The report checker verifies both completed reports and matching
metadata. This is a before/after code comparison, not a policy toggle within one
executable.

| Seed / repair rule | Year-30 population | Funding collected | Administration fees paid | Repairs paid | Operational institutions |
|---|---:|---:|---:|---:|---:|
| 17 / before | 1,948 | 839.22 | 408.18 | 1,231.03 | 0 / 32 |
| 17 / reserve enforced | 1,940 | 716.53 | 468.99 | 1,047.53 | 0 / 32 |
| 81 / before | 2,018 | 815.86 | 388.50 | 1,227.36 | 0 / 32 |
| 81 / reserve enforced | 2,011 | 792.76 | 456.52 | 1,135.24 | 0 / 32 |

Fee payments increase by 15% / 18%; repair spending falls. The controlled fixture
establishes the direct reserve mechanism; divergent later collection and population
are integrated outcomes, not proof of a single causal path. All institutions remain
nonoperational at year 30. Administration completes 18.95 / 17.20 worker-months
against unchanged demands of 171.15 / 160.65, so the earlier allocation bottleneck
remains. The fix is retained for consistent spending limits, not presented as a
successful institutional balance solution.

All sixteen sites stay active. Food-access gaps increase slightly, 1.9817% →
1.9967% and 1.8662% → 1.8860%. Maximum population residual remains zero, food
partition residual is at most 2.67e-7, and absolute terminal relative economic
residual is at most 2.35e-6 across both versions. New runs finish in about 62–66
seconds per seed. No funds, materials or work were injected during these runs.

Reproduce using the named-administration command above with output
`output/institution-repair-reserve-30.json`, then compare:

```sh
python3 scripts/compare_food_access.py output/institution-named-treatment-30.json \
  output/institution-repair-reserve-30.json
```

## Essential-service allocation pilot

`Culture.institution_work_policy = EssentialFirst` with named administration enabled
changes allocation within the existing institutional service window. The runner
exposes `--essential-institution-work`, requiring
`--named-institution-administration`. `FullUpkeepFirst` remains the default,
including older archives. Each dated work plan captures its policy.

Both policies receive the same election, upkeep and administration requests and
share the same finite site allowance and member availability. Essential-first
reserves election work, up to 0.025 basic upkeep per institution, then 0.05
administration assignments. Remaining time extends the same upkeep member's grant
for repairs up to the original request. The extension cannot reassign the member,
change the month, reuse completed work, or exceed personal capacity. Unfundable
minimum administration grants leave time available for repairs. Stable/rotating
order still applies within each request class.

Only reservation priorities change. Actual upkeep, collection, financial transfers
and Close settlement retain their existing order and live guards. This policy may
improve collection while slowing repairs; it cannot fix inadequate buildings or
missing qualified members. Keep it opt-in pending controlled and natural balance
comparisons rather than declaring essential-first universally preferable.

Verification: seven allowance levels compare the policies against identical
requests and one shared participant. With 0.10 worker-months, full-upkeep-first
grants 0.10 upkeep and no administration; essential-first grants 0.05 upkeep and
0.05 administration. With ample time both fully grant the same 0.175 request.
Below-minimum floating-point allowances remain unallocated rather than rounding up
an indivisible task. Participant commitments sum to the same bounded site grant,
serialized settlement matches, and settled grants cannot be enlarged.

All 118 regular library tests and seven institutional GPU tests pass. The full
frozen monthly/batched/checkpoint comparison passes on seeds 17, 81 and 256;
seed 256 enables essential-first, named administration, operating funding and
rotating within-class priority together. These checks establish implementation and
continuation behavior; natural calibration is separate.

## Essential-first: matched 30-year comparison

Implementation `21b617e`, seeds 17/81, both arms using that executable and the same
settings as the reserve comparison above. Only `--essential-institution-work`
differs. Both processes complete and the metadata/completion checker passes. The
refreshed full-upkeep-first runs reproduce all earlier per-seed fields exactly
apart from elapsed time, verifying the default allocation refactor preserved those
observed histories.

| Seed / work policy | Population | Operational / active institutions | Funding collected | Fees paid | Repairs paid |
|---|---:|---:|---:|---:|---:|
| 17 / full upkeep first | 1,940 | 0 / 32 | 716.53 | 468.99 | 1,047.53 |
| 17 / essential first | 1,918 | 6 / 31 | 2,432.18 | 1,266.50 | 1,710.26 |
| 81 / full upkeep first | 2,011 | 0 / 32 | 792.76 | 456.52 | 1,135.24 |
| 81 / essential first | 2,002 | 9 / 32 | 2,588.12 | 1,247.50 | 1,916.15 |

Essential-first executes 1,834/1,835 and 1,793/1,794 positive-ceiling collections,
versus 221/2,915 and 206/2,744 in the controls. It collects over 99.9% of cumulative
ceilings and largely eliminates repeated unmet operating requests. Completed
administration increases from 18.95/17.20 to 126.70/124.90 worker-months. Completed
upkeep falls from 427.50/401.125 to 206.725/203.425, while both fee payments and
repair expenditure increase. Actual institutional formation and membership also
diverge, so lower cumulative work is not a fixed-organization efficiency estimate.

This is the first tested allocation change in this series to leave operational
institutions at year 30. It supports the narrow diagnosis that reserving repair
work before collection can undermine the funding needed to use that work. It does
not establish universal superiority: population declines slightly relative to the
controls, food-access gaps worsen from 1.9967% → 2.0226% and 1.8860% → 1.9411%,
and most institutions remain nonoperational. Keep the policy opt-in while reviewing
service-space limits and evaluating longer held-out histories.

All sixteen sites stay active. Maximum population residual is zero; maximum food
partition residual is 2.67e-7; absolute terminal relative economy residual is at
most 2.46e-6. Both arms ran on the Quadro RTX 5000, with portions overlapping;
elapsed times are not a performance comparison.

Reproduce the prior named-administration command with output
`output/institution-essential-control-30.json`. Add `--essential-institution-work`
for `output/institution-essential-treatment-30.json`, then compare:

```sh
python3 scripts/compare_food_access.py output/institution-essential-control-30.json \
  output/institution-essential-treatment-30.json --allow-difference essential_institution_work
```
