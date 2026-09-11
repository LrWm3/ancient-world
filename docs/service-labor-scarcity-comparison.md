# Service labor scarcity: priority counterfactuals

## Question and experiment

Do inherited monthly priorities allocate scarce workers sensibly? The new common
ceiling prevents overbooking, but does not answer who should receive that ceiling.
This experiment tests the immediate reservation boundary, not demographic or
long-run economic balance.

Run the hardware fixture:

```sh
mkdir -p output
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib \
  service_scarcity_priority_comparison -- --ignored --nocapture \
  > output/service-scarcity.log 2>&1
```

Three seeded starting histories (17, 81, 256), four adult workforces (2, 8, 20,
100), two illness levels (0, 0.5), cultural work off/on, and three schedules give
144 matched reservation cases. Each case starts from a fresh clone. The options
are frozen history with society enabled, month 3, a financed workshop operator
requesting 0.88 worker-months, and a commissioned four-vessel port capable of
employing one worker-month. Quarterly culture requests up to 0.5 worker-months.
Research has no active sample request in this fixture.

Orders:

- **0, current:** culture → enterprise → crews.
- **1, shipping earlier:** culture → crews → enterprise.
- **2, economic work earlier:** crews → enterprise → culture.

The test explicitly imports installed workshop/port capital and starting treasury
for a synthetic port at the workshop town. It changes adult counts as an
intervention without reconstructing households. It tests existing employer, culture
and vessel functions, not a replacement numerical allocator. These are controlled
fixtures rather than naturally occurring port populations. Route accessibility,
cargo demand and population conservation across these setup edits are not tested.
After setup, wage/rent transfers must conserve combined household, firm and town
money. Reservations must stay within the adjusted ceiling.

## Measured outcomes

All three seeds produced the following allocations (rounded). Work is in effective
worker-months; freight is **staffed capacity**, not cargo actually shipped.

| Adults | Illness | Culture enabled | Order | Culture work | Enterprise work | Crew work | Freight capacity kg |
|---:|---:|:---:|---:|---:|---:|---:|---:|
| 2 | 0 | no | 0 | 0 | 0.32 | 0 | 0 |
| 2 | 0 | no | 1 | 0 | 0 | 0.32 | 320 |
| 2 | 0 | yes | 0 or 1 | 0.32 | 0 | 0 | 0 |
| 2 | 0 | yes | 2 | 0 | 0 | 0.32 | 320 |
| 8 | 0 | no | 0 | 0 | 0.88 | 0.40 | 400 |
| 8 | 0 | no | 1 | 0 | 0.28 | 1.00 | 1,000 |
| 8 | 0 | yes | 0 | 0.50 | 0.78 | 0 | 0 |
| 8 | 0 | yes | 1 | 0.50 | 0 | 0.78 | 780 |
| 8 | 0 | yes | 2 | 0 | 0.28 | 1.00 | 1,000 |
| 8 | 0.5 | no | 0 | 0 | 0.88 | 0.08 | 80 |
| 8 | 0.5 | yes | 0 | 0.50 | 0.46 | 0 | 0 |
| 8 | 0.5 | yes | 1 | 0.50 | 0 | 0.46 | 460 |
| 8 | 0.5 | yes | 2 | 0 | 0 | 0.96 | 960 |
| 20 | 0.5 | yes | all | 0.50 | 0.88 | 1.00 | 1,000 |
| 100 | 0 or 0.5 | yes | all | 0.50 | 0.88 | 1.00 | 1,000 |

Illness reduces total effective service capacity by 25%, but in the eight-adult
current-order case it reduces shipping from 400 to 80 kg (80%). This amplification
is a consequence of giving crews the residual after higher-priority claims.
Quarterly culture removes that remaining shipping capacity entirely.

Abundant-workforce cases are negative controls: all three schedules agree once the
requests fit. Culture-disabled orders 1 and 2 also agree exactly. Tests assert these
controls as well as scarcity effects and budget bounds. Small freight deviations
below the rounded figures arise from existing wage-transfer rounding.

## Interpretation and decision

The inherited order is not neutral. It protects cultural activity and manufacturing
at the expense of shipping during scarcity. Reversing it does not solve scarcity:
crew-first can eliminate workshop and cultural work instead. In the smallest town,
priority alone selects the sole funded activity.

There is not yet enough information here to justify shipping always winning.
`prepare_vessels` hires against installed hulls, resident households, available
workers and money; it does not require pending cargo. Thus order 2 can fund idle
shipping while suppressing useful manufacturing or teaching. Conversely,
`reserve_cultural_work` reserves a quarterly allowance before individual cultural
actions are chosen, and firm requests use previously observed workshop activity.
None is a reliable common measure of this month's urgent useful work.

**Keep the production schedule unchanged in this experiment.** The next scheduling
change should distinguish committed/urgent demand from discretionary opportunities:

1. Derive crew requests from existing voyages, reserved cargo and a bounded standby
   allowance; distinguish capacity needed for relief/food from speculative service.
2. Gather affordable workshop and cultural/research action requests before granting
   their shared discretionary allowance. Preserve named project and wage provenance.
3. Compare bounded sharing or rotating discretionary priority against the current
   order. Ensure unpaid/unfunded requests cannot crowd out feasible work.
4. Then measure completed versus paid work, actual deliveries, food shortfalls and
   institutional activity over multiple months, including recovery after scarcity.

A flat change to the 20% service ceiling would conceal competition by taking more
workers from other livelihoods. It is not supported by these results.

## Limits and verification

This test does not execute production, food consumption or cultural action completion,
so it cannot establish improved harvests, less famine, institutional survival or
useful work per wage. The printed `idle` column is unused **service allowance**, not
measured unemployed population. Research competition, living-history recovery,
material shortages and treasury shortages need additional controlled arms.

An initial fixture expectation assumed eight healthy adults would leave no crew
capacity under enterprise-first; the actual result was 0.40 worker-months. That
expectation was corrected rather than changing the model to satisfy it. The small
workforce cases do demonstrate complete exclusion.

Verification on the Quadro backend: the 144-case GPU fixture passed in 2.18 seconds
excluding compilation. Printed allocations agreed across all three seeds at the
reported precision. The ordinary library suite passed (70 tests, 67 hardware tests
ignored); strict Clippy, formatting and the repository artifact policy passed.
Only source, tests and this human-readable summary are committed. Generated output
stays under ignored `output/`. No new full-history balance claim is made.
