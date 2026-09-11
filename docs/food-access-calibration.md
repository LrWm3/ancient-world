# Food access versus physical food shortage

This experiment separates food available in a town from residents' ability to
consume it. It does not treat every shortage as an insufficient crop yield.

## Protocol and attribution

Eighteen twenty-year runs cross seeds 17/81/256, crop yield scales 0.33/0.15, and
common food shares 0.5/0.75/1.0. Individual nutrition stays enabled. Other settings
match the previous nutrition suite: five requested civilizations, terrain/ecology
32, one geological epoch followed by frozen history, society/politics/governance/
offices/shipping, individual demography, and ordinary household payroll/relief.
No food, money or nutrient inventory is added by the intervention.

For each town at its completed consumption boundary, in food-equivalent kg:

- `N`: pre-demographic food need.
- `A`: physically available food after ordinary spoilage and production, before consumption.
- `F`: funded entitlement, including common access and affordable purchases.
- `C`: actual consumed food, from the GPU ration allocation.
- Physical gap: `max(N - A, 0)`.
- Access gap: `max(min(N, A) - C, 0)`.

The gaps sum to `N - C` within floating-point tolerance, checked monthly. Compute
these per town before summing: distant surplus does not automatically feed another
town. Physical availability includes reserves, transport and spoilage effects as
well as harvests, so a physical gap is not uniquely attributable to crop yields.
These gaps describe aggregate unmet need; within-town household inequality can
persist independently.

Common-share interventions change the consumption cap immediately. They can also
change later cash receipts, population, demand and production. Those later effects
are part of the toy economy, not an isolated causal estimate of agricultural
productivity. The report includes produced food separately from stored availability.
Gap percentages divide cumulative gap by cumulative need, avoiding artificially
low averages from months with no population left.

## Reproduction

```sh
cargo run --example nutrition_evaluate -- --affordability --years 20 --output output/food-access.jsonl
python3 scripts/summarize_food_access.py output/food-access.jsonl
```

`--common-shares` changes the access scenarios; `--yields` changes the production
scenarios. Without `--affordability`, the previous nutrition on/off comparison is
preserved. Outputs belong under ignored `output/`; only this summary is committed.
The summarizer rejects incomplete suites. The analytical test covers physical-only,
access-only and combined shortages, full consumption, zero need, and separated towns:

```sh
cargo test --example nutrition_evaluate
```

## Opening evidence

All seeds and yield settings start with 9,120 kg-equivalent monthly need and
127,858.926 available. Changing the common share leaves that availability unchanged:

| Common share | Funded and consumed | Physical gap | Access gap |
|---|---:|---:|---:|
| 0.5 | 4,810 | 0 | 4,310 |
| 0.75 | 7,090 | 0 | 2,030 |
| 1.0 | 9,120 | 0 | 0 |

Thus the opening 47.3% shortage is produced by the entitlement/purchasing-power
rules despite ample food. Increasing yield cannot fix that immediate restriction.

## Twenty-year results

All eighteen runs completed (4,320 monthly boundaries) on Quadro RTX 5000 Max-Q/Vulkan.
No population residual was observed; maximum relative food residual was 4.601e-7.

| Seed | Yield | Population at 50% access | At 75% | At 100% | Physical gap at 50% | Access gap at 50% |
|---|---:|---:|---:|---:|---:|---:|
| 17 | 0.33 | 435 | 477 | 516 | 1.29% | 4.25% |
| 17 | 0.15 | 36 | 29 | 35 | 24.96% | 6.53% |
| 81 | 0.33 | 519 | 598 | 669 | 0.45% | 4.12% |
| 81 | 0.15 | 68 | 59 | 61 | 21.28% | 7.45% |
| 256 | 0.33 | 539 | 640 | 725 | 0.22% | 3.44% |
| 256 | 0.15 | 39 | 42 | 40 | 23.04% | 6.21% |

At yield 0.33, access restrictions account for more unmet need than local physical
shortages under the current 50% common-share policy. Raising common access improves
survival in all three seeds. At yield 0.15, physical gaps dominate and broader access
does not reliably improve final population. Removing the access cap increases
consumption earlier; later scarcity, changing population and economic feedback
still matter. A lower final population is not proof that withholding food is better.

With 100% common entitlement the measured access gap is zero within tolerance
throughout all six runs, while physical shortages remain. Physical gaps reach
28.8–32.2% of cumulative need in the low-yield, full-access worlds. Production is
recorded independently: it changes over time as population and economic activity
diverge, not because the entitlement setting multiplies harvests directly.

## Tuning decision

Keep yield and access as separate controls. No production or household-policy
defaults were changed in this pass.

1. For moderate-yield worlds, address food access first. A 75% common share is a
   useful intermediate scenario, but still causes 22.3% opening shortage despite
   ample founding provisions. Merely selecting it as a new default would leave
   that founding mismatch unresolved.
2. Review whether declared founding food should initially be available communally,
   with a deliberate later transition to household purchasing. That is a policy
   design question, not a reason to raise crop yield.
3. For harsher low-yield worlds, inspect seasonal reserves, growing conditions,
   land/labor constraints and transport. This experiment identifies local physical
   shortage but does not distinguish those supply mechanisms from each other.
4. Before changing defaults, repeat chosen policies with living ecology and longer
   histories. The present results are frozen, low-resolution, three-seed evidence.

The analytical decomposition test and example Clippy checks passed. The report
also checks every monthly decomposition, complete paired coverage, unchanged
opening supply across access controls, and negligible access gaps with full common
entitlement. No core simulation code changed; the addition is diagnostic tooling
and controlled evaluation.
