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
The full frozen scheduler test passes seeds 17, 81 and 256 through monthly, batched and saved continuation; 17/256 enable the allowance and 81 retains existing allocation. All-target Clippy passes with warnings denied. Matched century outcomes are pending. Council inspection now displays the dated allocation and latest tax collection; no interactive visual test is claimed.

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

