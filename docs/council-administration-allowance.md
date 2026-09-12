# Basic administration alongside household assistance

The opt-in `HouseholdEconomy.council_allocation = ProtectAdministration` policy
limits household relief to cash remaining after the council's **opening monthly
administrative cost forecast**. `Existing` preserves the previous policy and is
the default for new histories and old archives. The evaluation runner exposes
`--protect-administration` and records the switch in report metadata.

For each council, let T be its current treasury, A its administrative forecast,
R its total eligible household request, and s its existing relief spending share.
The existing ceiling is min(T × s, R). The pilot grants the smaller of that ceiling
and max(T − A, 0). Household grants remain proportional to eligible requests.
Unused household demand cannot increase the administrative request.

The forecast uses the same cost function as actual governance payment: resident
population × (0.01 domestic / 0.04 foreign) × (1 − 0.5 × autonomy). Abandoned towns
request zero; absent governance requests nothing. Every controlled town contributes
to its council's forecast before any household grants are allocated.

This is a scoped spending constraint, not escrow or guaranteed administrative
funding. Household relief still executes in Reserve; administration still pays in
Respond after population and other systems have changed. Other spending channels
can consume remaining cash. Annual tax collection is unchanged. No transfer is
made by the forecast itself, and the later administration payment cannot spend it
as though it were a new stock.

Latest council receipts retain month, policy, opening treasury, administrative
forecast, relief request, ordinary ceiling, grant and actual household transfer.
Actual administrative requests and payments remain in `CouncilFunding`; a forecast
is not reported as completed public service. Receipts persist and are validated.

The key comparison is household food access **and** actual administrative funding.
A larger council balance is not the objective. A successful analytical allocation
does not by itself justify enabling the policy by default.

## Tax evidence

`CouncilFunding.taxes` retains each site's latest actual annual collection:
month, controller, opening town cash, tax rate, autonomy, delivered office capacity
and payment. This records the existing transfer without adding revenue or changing
its timing. Archives without tax observations start with an empty baseline.
Payment follows opening cash × rate × (1 − 0.75 × autonomy) × office capacity.

The evaluation runner observes council relief receipts every month and accumulates
requests, ordinary ceilings, grants, actual payments and reductions caused by the
allowance. Decadal snapshots alone cannot measure monthly withheld relief.

## Verification and balance

The pure scarcity fixture covers insufficient cash, absent administrative demand,
absent household demand, and demand below its budget. A GPU-founded household case
checks that the policy reduces real household relief and increases real subsequent
administrative payments, conserves money, survives serialized continuation between
those operations, and is inert without governance. Both tests pass. The ordinary
library suite passes 136 tests; 116 hardware/long tests are ignored in that command.
The full frozen scheduler test passes seeds 17, 81 and 256 through monthly, batched and saved continuation; 17/256 enable the allowance and 81 retains existing allocation. All-target Clippy passes with warnings denied. The allowance-only century outcomes are reported below. Council inspection now displays the dated allocation and latest tax collection; no interactive visual test is claimed.

Reproduce both arms with `cargo build --example cultural_work_calibrate`, then:

```sh
target/debug/examples/cultural_work_calibrate --seeds 256,409 --years 100 \
  --resolution 32 --crop-yield-scale 0.5 --individual-demography \
  --workshop-refinement --agriculture-refinement --extraction-refinement \
  --construction-refinement --compare-resolution --household-diagnostics \
  --family-support --output output/council-allowance-baseline.json
```

Repeat with `--protect-administration` and output
`output/council-allowance-protected.json`. Compare completed reports:

```sh
python3 scripts/compare_food_access.py output/council-allowance-baseline.json \
  output/council-allowance-protected.json --allow-difference protect_administration
python3 scripts/compare_council_funding.py output/council-allowance-baseline.json \
  output/council-allowance-protected.json --allow-difference protect_administration
```

These runs overlap for wall-clock efficiency; their durations are not performance
benchmarks. Raw reports and logs remain ignored under `output/`.


## Separate annual town-support correction

Inspection of the first baseline's year-100 tax boundary found that site 2 paid
73.58 currency in taxes from 4,830.05 town operating cash, with office capacity
0.75, rate 0.025 and autonomy 0.25. Its council nevertheless ended empty. The
annual support rule triggers on hunger and requests ten currency per resident
**without subtracting money already held by the town**. This can return tax
receipts immediately to an already cash-rich town. Such a transfer is distinct
from helping households buy the food that town holds.

`Society.town_support_policy = CashGap` is a separate opt-in correction. It retains
the hunger eligibility threshold and existing ten-per-resident cash target, but
requests only max(target − current town operating cash, 0), evaluated after tax
collection. Payment remains bounded by the live council treasury and is rounded
down to avoid spending beyond its f64 budget when transferring to f32 town cash.
This is a working-capital target, not a costed inventory of essential services.
It does not protect a year's administration, alter taxes, or give households cash.

Use `--cash-gap-town-support` in the same runner. Compare it independently and
combined with `--protect-administration`; do not attribute both changes to the
relief allowance. Annual tax observations now also retain support requested and
paid, so the suspected immediate refund can be measured directly.

## Completed allowance-only century comparison

Both arms complete seeds 256/409 at terrain 32, ecology 16, one epoch, sixteen
founders and yield 0.5. Political distribution, family support and individual
production refinements remain enabled in both. Only the allowance differs.
All previously published baseline sample fields (including funding totals) exactly
match the earlier council-funding political runs for both seeds.

| Seed | Population existing → allowance | Cumulative food access gap | Unpaid share of administrative demand | Longest ending unpaid streak | Relief withheld over century |
|---|---:|---:|---:|---:|---:|
| 256 | 2,511 → 2,558 | 2.0209 → 2.0164% | 6.929 → 5.214% | 84 → 96 months | 26.506 |
| 409 | 2,647 → 2,630 | 2.0056 → 2.0061% | 7.776 → 7.845% | 1 → 2 months | 25.363 |

Population remains growing in the final decade, but final-decade access gaps
worsen from 2.0639 to 2.3567% and 2.1442 to 2.3316%. The longest unpaid streak
moves to a different town in seed 256; it is not the original town's streak
increasing. Physical food shortages remain zero, population residuals zero and
maximum monthly food residual below 1.7e-6. Sixteen sites remain active in all arms.

Decision: **do not enable the allowance by default**. It changes allocations as
intended, but it does not reliably improve service coverage or food access. Its
scope is too narrow to resolve the annual tax/support connection. The separate
cash-gap support comparison is pending; these results do not demonstrate its
outcome. Raw evidence remains local under `output/council-allowance-*`.

The cash-gap correction passes four focused tests (two analytical and two
GPU-founded transfer/continuation fixtures), the ordinary library suite (137 pass,
117 hardware/long fixtures skipped), all eight market integration tests including
hardware cases, and the three-seed frozen scheduler comparison. Seeds 17/256 use
both council pilots in that scheduler test; seed 81 retains both legacy policies.
All-target Clippy passes with warnings denied. The existing explicit market
fixture needed the new default policy field; its economic assumptions are unchanged.
The matched cash-gap century comparisons remain pending at this implementation
commit. Neither policy is enabled by default on this evidence alone.
