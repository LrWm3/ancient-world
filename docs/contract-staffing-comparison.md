# Contract-aware staffing comparison

Status: all eight arms completed successfully; results are mixed. Compare the staffing override off/on using one frozen executable
from `81e4a02`. Each series contains baseline, credit, issuance and combined arms.
Use the same debt-free seed-1024 32/32 checkpoint, five civilizations and 50-year
interval as the previous procurement screen. Procurement stays enabled at 25%,
with delivery-paid exports and unchanged fees, wage rules, cash floors and credit
underwriting. Only the shift-request policy differs between the two series.

Measure completed operator work, funded labor versus completion at observed
contract boundaries, fees/refunds, crop harvest separately from household hunger,
actual loans and monetary residuals. Increasing requests, wages or contracts alone
is not a benefit. This is a single-seed integration screen; retain the opt-in
policy unless broader comparisons support changing the default.

Raw outputs remain under ignored `output/contract-staffing-comparison/`. Reproduce
with the monetary runner and `--service-order-procurement
--service-procurement-share 0.25`, once without and once with
`--contract-workshop-staffing`, using distinct output directories. The runner
explicitly sets staffing false in the first series. Constants verification may
share the GPU, so elapsed times are not isolated performance benchmarks.


## Results

The off series exactly replays the previous observation screen after removing
only the newly defaulted `contract_staffing` policy field. In both series, credit
issues no loans and changes no history outside its own records. The four distinct
outcomes are:

| Contract staffing | Issuance | Completed operator work | Paid operator work | Terminal need-weighted hunger | Population | Contract fees earned | Contract refunds |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| off | Off | 19.5190 | 32.0434 | 0.05885 | 160.775 | 682.9594 | 899.0669 |
| off | On | 17.4370 | 26.3810 | 0.06844 | 157.701 | 576.9221 | 571.4520 |
| on | Off | 21.6516 | 38.5557 | 0.06117 | 156.000 | 811.9413 | 644.2477 |
| on | On | 22.0134 | 35.4823 | 0.07178 | 156.676 | 825.1417 | 472.7068 |

Work is measured in worker-months. Without issuance, the new policy raises
completed work from 19.5190 to 21.6516, but paid work rises from 32.0434 to 38.5557.
With issuance, completion rises from 17.4370 to 22.0134 while paid work rises from
26.3810 to 35.4823. Completion per paid worker-month therefore falls in both cases:
this is not an unqualified productivity improvement. Unearned fees still refund
rather than becoming operator income.

Terminal hunger rises in both enabled comparisons and population is lower.
These endpoint outcomes do not identify the immediate household cause. They must
not be attributed solely to new wage spending without tracing food prices,
purchases, production and competing work. More completed workshop work is real,
but it does not establish a net benefit for the settlement economy.

The policy produces 342 versus 381 orders without issuance and 280 versus 268
with issuance. Zero-completion counts are 51 versus 58 and 38 versus 42 respectively;
changing order counts makes raw failure counts insufficient for judging reliability.
All orders have due-month observations in this screen.

The largest absolute endpoint relative money residual is 1.79e-07. These are
managed-ledger money values (index 4), not monthly maxima. All native arms pass
their checks. The positive request, bounded money, CLI policy independence and GPU
checkpoint/batch fixtures are documented with the implementation.

## Decision and remaining work

Retain contract-aware staffing as an opt-in experiment. It repairs a real demand
connection and increases completed work, but also increases paid idle work and
worsens terminal food access in this seed. Do not enable it by default or claim
that credit now works: no loans issued. Stage 2 remains gated.

Next distinguish inputs, output-demand limits and installed productive capacity
within funded-but-incomplete shifts. Use an opening-boundary comparison to see
whether input-backed requests improve completion without crowding out food or
merely suppressing useful work. Keep the original request visible alongside any
feasibility limit; do not relabel a reduced request as full satisfaction. Then
validate the chosen behavior on additional seeds and longer intervals.

Executable SHA-256: `a7f4e348e52c6d820f6aa7dcc5301fa114f6c3266e8726945357b67ee5b6e938`.
Both series use the same binary; the subsequent ecology header relocation is
excluded. Generated results are not committed.
