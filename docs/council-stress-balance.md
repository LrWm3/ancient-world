# Council policies under broader environmental pressure

This follow-up extends the small-grid council comparison to held-out seeds and a
longer horizon. It tests a toy budget policy, not historical calibration.

The evaluation runner's `--weather-stress` option sets drought probability 0.5,
drought severity 0.9, storm probability 0.5 and storm multiplier 4. These are the
same forcing values used by the existing gathered/full-readback scheduler fixture.
The complete weather parameters are recorded in report metadata. No food, cash,
people, equipment or institutions are added to help the scenarios survive.

Both arms use the same weather stress and political distribution. Only annual
town support differs: existing support versus a request for the actual gap to
the town's working-cash target. The monthly administrative allowance stays off.
This separates the two budget policies after the allowance-only comparison's
mixed outcome.

## Protocol

Build `cargo build --example cultural_work_calibrate`, then:

```sh
target/debug/examples/cultural_work_calibrate --seeds 17,81,1024 --years 200 \
  --resolution 32 --crop-yield-scale 0.5 --individual-demography \
  --workshop-refinement --agriculture-refinement --extraction-refinement \
  --construction-refinement --compare-resolution --household-diagnostics \
  --family-support --weather-stress --output output/council-stress-existing.json
```

Repeat with `--cash-gap-town-support` and output
`output/council-stress-cash-gap.json`. Compare with both
`scripts/compare_food_access.py` and `scripts/compare_council_funding.py`, allowing
only `cash_gap_town_support` to differ. Raw results stay ignored under `output/`.

The runner already records actual expedition voyages and crew identities, military
service and deaths, and event counts. An enabled system with zero voyages or
service does **not** count as an exercised fiscal pressure. These runs impose
weather stress; they do not force wars or finance expeditions. Any unexercised
conflict/expedition path remains an explicit evidence gap requiring a separately
funded scenario fixture. Grid-resolution and hardware comparisons also remain
separate work.

## Status

The one-year seed-17 smoke test completes with 1,955 residents and records all four
forcing values correctly. The original ensemble fails as recorded below; the
isolated corrected ensemble completes all six histories below. Failed or incomplete histories
are separate from completed 200-year results.

## Funded campaign boundary check

`cargo test --test governance council_allocation_campaigns_conserve_and_resume -- --ignored`
passes on Vulkan. A real affordable campaign is launched before saving in each of
three variants: existing support, cash-gap support, and cash-gap plus the monthly
administration allowance. Thirty-six months in one batch match thirty-six single
steps after reload. The war ends; complete histories match, tax observations are
present, and economic residuals remain below 0.001 in their native units.

This exercises resource-funded war under the policies. It does not establish
long-run wartime fiscal balance or prove that one policy improves war outcomes.

## Detected failure: actual farm-work over-allocation

The existing-support seed 1024 stops at month 1688 (140 years plus eight months).
Its requested farming allowance is 106.666664 worker-months, but the stored
individual grants sum to 106.66677019000053. The float32 display sum is lower,
106.66662; this is actual accumulated over-allocation, not merely a display/audit
sum problem. Seeds 17 and 81 complete 200 years in that arm. The failed seed's
140-year sample is not treated as a 200-year result.

A 399-worker fixture reproduces the fault with an identical 160/1.5 allowance and
0.8 capacity per worker. Repeated float32 subtraction can leave positive apparent
capacity after actual commitments exceed the total. The correction retains the
remaining allowance in double precision, rounds each individual grant downward,
and similarly bounds the GPU allowance by the double-precision sum of actual
commitments. It preserves the validation tolerance rather than accepting excess.
Both the new true-overgrant fixture and the previous false-audit-sum fixture pass.

For the corrected stress comparison, use isolated source `b9ed7bf` plus the
`src/agriculture_participation.rs` allocator patch. Do not include the intervening
husbandry, continuing-study or captured-route changes when interpreting the council
policy comparison. The completed corrected results are reported below.

The cash-gap seed 1024 also fails, at month 2200, with stored grants
106.66678148508072 against the same 106.666664 request. Both policies therefore
expose the allocator fault. The corrected three GPU agriculture/extraction/
construction fixtures pass, including wage attribution and saved continuation.

## Interpreting funding observations

The administrative record can retain unpaid-month counters after a site becomes
inactive. The comparison script labels its maximum as a **stored** streak; it is
not automatically the current service gap of an occupied town. Use cumulative
actual requested, paid and shortfall totals to assess coverage, and report active
settlement counts separately. Differences in survival also change service demand.
An ending treasury alone cannot establish either adequate revenue or unnecessary
reserves.

The allocator correction is not trajectory-neutral. In seed 17, both original and
corrected policy arms match population and completed cultural work at year 10,
but later discrete outcomes diverge. The existing-support endpoint changes from
1,144 to 1,073 residents; cash-gap support changes from 1,501 to 1,165. These are
original-versus-corrected code comparisons, not treatment effects. The new policy
comparison must use corrected source for **both** arms. The bounded-grant proof
and fixtures justify fixing the overdraw; they do not establish insensitivity to
small allocation changes or identify every subsequent causal branch.

## Completed corrected ensemble

All six histories exit successfully at 200 years. The isolated executable uses
`b9ed7bf` plus the allocator change committed as `00f1fa9`, without later husbandry,
freight, continuing-study or heritage changes. Both arms run seeds **1024, 17, 81**
in that order to verify the formerly failing seed first. All other protocol
settings above match. Strict comparison scripts accept only the declared
`cash_gap_town_support` difference and verify complete monthly-observed endpoints.

Food gaps are cumulative shares of required food: physical supply shortage and
unmet purchasing access are separate. Administrative shortfall is cumulative
unpaid/requested cost. Population change is the final observed decade, years
190–200. Active sites are endpoint counts, not a survival fraction of a fixed
initial roster.

| Seed | Support | Population | Change 190–200 | Active towns | Physical food gap | Access gap | Admin unpaid |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 17 | Existing | 1,073 | −25 | 9 | 0.8020% | 2.3460% | 9.972% |
| 17 | Cash gap | 1,165 | −4 | 8 | 0.8276% | 2.3129% | 4.629% |
| 81 | Existing | 1,489 | −84 | 12 | 0.7769% | 2.0759% | 10.334% |
| 81 | Cash gap | 1,470 | +69 | 10 | 0.9421% | 1.9978% | 4.765% |
| 1024 | Existing | 1,465 | −37 | 11 | 1.0469% | 2.0315% | 15.407% |
| 1024 | Cash gap | 1,424 | +85 | 9 | 0.9670% | 2.0676% | 5.377% |

**Interpretation:** cash-gap requests improve administrative payment coverage in
all three seeds and improve the final decade's population change. They do not
uniformly improve food supply, purchasing access or ending population. Every
treatment has fewer active towns. Retain the opt-in policy; improved fiscal
coverage alone does not justify selecting it as the universally better setting.
Two positive final decades are not proof of a stable long-run population.

The histories record 131–175 regional drought events and 4–8 abandonment events
per run. Road-flood closures occur in the seed-1024 pair. There are **zero actual
expedition voyages and zero people serving in military cohorts** in every run:
those enabled systems were not exercised by this balance ensemble. The separately
funded campaign fixture above covers accounting/continuation, not long wartime
balance. Heritage effects are checked in their own finite-recovery fixture.

Maximum monthly population residual is **zero** in every run. Maximum reported
food residual is **1.584e-6**; the largest absolute ending economic residual across
the reported native-unit ledgers is **2.395e-5**. These small residuals support the
accounting checks, not a claim that the policy or resulting histories are ideal.
No unresolved simulation error is accepted to complete the corrected ensemble.

These processes overlap other validation and are briefly paused during the lake
microbenchmark; their wall times are not comparative performance evidence. Raw
JSON/logs remain local as `output/council-corrected-{existing,cash-gap}.*`.
Reproduction uses the protocol above with the corrected source and seed order,
then runs both comparison scripts with `--allow-difference cash_gap_town_support`.
