# Repairing demonstrated workshop capacity

This is a planning correction with mixed game-balance results, not a solution to
industrial viability or household food access.

## Boundary and mechanism

The GPU executes non-food recipes using installed equipment before household
craft fallback. The planner previously subtracted all hypothetical household
capacity from demonstrated industrial work before calculating specialized asset
targets. Consequently a working kiln could receive a shrinking repair target even
when it was actually used: hypothetical replacement by home work was treated as
if that replacement had happened.

In Reserve, specialized workshop planning now preserves a target for the utilized
portion of installed equipment:

```
utilized_units = min(installed_units, completed_family_work / work_per_unit)
target = max(previous_expansion_target, utilized_units)
smoothed_target = 0.9 * previous_target + 0.1 * target
```

The observation is the previous completed production interval. Installed equipment
is consumed first in that dispatch, so the minimum excludes household overflow.
Absent demand or observed work still allows target decay. Zero installed capacity
is not magically recreated from household production. Expansion retains its
existing demonstrated-work/workforce limits. This changes specialized planning;
the legacy unspecialized path is unchanged.

Execution still spends finite timber, bricks, tools and construction attendance.
No money is issued or paid by the forecast, and no material or asset is installed
in Reserve. No scheduler phase moves. Forecast smoothing and construction limits
mean even a utilized asset can decay if repairs cannot actually be completed.

## Matched screen

Baseline: selective food connections from commit 3bd7c8f. Candidate changes only
this repair target. Use the same 32/32 terrain/ecology founding archives and
settings as `selective-food-connections.md`: delivery-paid exports, funded service
procurement at .25, demand/contract staffing, estate inheritance, named office
service and abandoned stock recovery; credit, issuance and estate reclamation off.
Each of seeds 1024, 256 and 409 advances 600 months.

| Seed | Arm | Population | Ending hunger | Operator work | Operating margin | Active operators |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | Baseline | 154.171 | .04330 | 12.306 | 48.34 | 0 |
| 1024 | Repair | 154.263 | .12297 | 12.680 | 50.79 | 0 |
| 256 | Baseline | 350.482 | .02812 | 4.696 | 19.97 | 0 |
| 256 | Repair | 345.715 | .03746 | 4.693 | 19.66 | 0 |
| 409 | Baseline | 342.345 | .03661 | 187.326 | 951.31 | 3 |
| 409 | Repair | 352.545 | .01885 | 183.934 | 952.25 | 4 |

Hunger is weighted by current household food need, not a lifetime welfare measure.
Work and margins are cumulative; margins exclude capital, dividends and financing.
These are tuning seeds, not held-out validation. More firms alone is not success:
seed 409 initially has slightly less completed work despite more operators.

Extend seed 409 in both arms to 1,200 months:

| Arm | Population | Ending hunger | Operator work | Operating margin | Active operators | Operator cash |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Baseline | 244.652 | .01784 | 340.496 | 1854.73 | 2 | 1124.19 |
| Repair | 242.021 | .01830 | 381.455 | 2111.57 | 3 | 1858.87 |

This supports useful repeat business in the favorable seed, not general population
improvement. Seeds 1024 and 256 remain without operators at year 50. The maximum
absolute relative cash residual across the five new runs is 2.54e-7. Raw exports,
logs, commands and frozen candidate executable stay ignored under
`output/workshop-utilization-screen/`. Runs overlapped compilation or GPU tests;
no performance comparison is claimed.

## Remaining circulation work

The correction is retained because maintenance should follow actual equipment
use. It does not force every town to retain an uneconomic industry, lower entry
requirements, supply missing metals, or make household savings available to town
procurement. Scarce generic tools can still block repair below the construction
reserve. The next comparison should distinguish missing material inputs, viable
funded orders, equipment repair and the minimum operating scale before relaxing
any one constraint. Unworked abandoned deposits also still lack an extraction
expedition path; stored-goods recovery is separate.

## Verification

- Analytical fixture: a quarter worker-month of completed equipment work supports
  1/16 unit of maintenance even below household fallback. Idle work requests none;
  household overflow cannot enlarge the utilized installed stock.
- Hardware-backed planning fixture: used versus idle equipment produces different
  repair targets from otherwise identical opening snapshots. Planning changes no
  goods or physical assets. Serialized continuation reproduces the target.
- Four existing GPU recipe-allocation fixtures pass, covering prepaid attendance,
  blocked recipes, recovered inputs and finite ore-to-tools processing.
- Ordinary library suite: 196 passed, 152 hardware/long tests ignored. Native
  candidate build and strict library Clippy pass.

The first added fixture had an invalid redundant economy-upgrade call; founding
already enables it. Corrected before the passing run. An interrupted incremental
build also produced unresolved cached linker symbols; the fixture was rebuilt
with `CARGO_INCREMENTAL=0`. Neither failure was counted as a successful check.
