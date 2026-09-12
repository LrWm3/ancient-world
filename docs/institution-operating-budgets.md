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
