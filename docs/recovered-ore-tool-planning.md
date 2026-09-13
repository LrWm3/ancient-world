# Stored ore recovery and tool planning

Retained as a bounded recovery capability; no ensemble economic improvement
established. This extends
recovery of already extracted goods; it does not open abandoned deposits.

## Connection being tested

A town previously chose its new working-tool type from its local extraction
mineral. Held finished copper/bronze tools were retained, but imported copper ore
could remain irrelevant to a non-copper town's tool plan. Recovery in turn bought
only materials already demanded by that plan, so useful stored ores could be
ignored even with a passable route.

The candidate planner first retains held/expected finished tool service. It then
replays candidate recipe orders against finite held/expected inputs and this
month's extraction allowances, consuming forecast inputs within each replay. It
tries supported tool chains before leaving the remaining deficit to the ordinary
local preference. Recipe knowledge still applies. Catalog-order passes allow an
upstream recipe to precede its dependent work in the forecast even if the catalog
lists it later. This is a material feasibility forecast, not a workforce grant.

For recovery offers, the forecast includes the source's finite stored input but
no unworked extraction allowance. It requires the complementary processing inputs
to be held or expected and does not use the speculative fallback tool chain.
Finished tools, pending inputs and funded deliveries reduce repeat demand.
Additional input demand is also bounded by a three-month processing-work forecast
from completed metalworking or prepaid operator attendance. The existing
transaction still enforces buyer cash, storage, road capacity,
round-trip range, source permission and war/siege restrictions. The source quote
must not exceed the buyer's local quote for this additional input demand.

Actual goods remain at the source until reserved into cargo and are unavailable
to production before delivery. Mining, smelting, fuel, residue, tool production and
labor accounting remain in their existing execution paths. No invented metal,
extra labor allowance, smaller harbor or treasury transfer is introduced.

## Limits to evaluate

- Cheap ore does not guarantee profitable tools: the local input quote comparison
  is not a complete forecast of fuel, wages or future tool prices.
- Expected cargo may be delayed; forecasts do not promise completion this month.
- The work ceiling is per offer, not a forward labor reservation across all offers;
  only actual execution settles shared labor. Cold starts without demonstrated or
  funded processing activity need an explicit restart path.
- Source/settlement iteration and existing quarterly trade priority remain. Within
  automatic recovery, finished goods are considered across all estates before
  raw tool inputs; each class retains rotating material order.
- There is no new mine crew, excavation or access to neighboring deposits.
- More recovered ore is useful only if delivery produces useful outputs without
  unacceptable losses in food access or other services.

## Focused verification

- All 194 ordinary library tests pass (150 hardware tests remain ignored in that
  command). Ten production tests include finite ore-to-tool forecasts, retained
  substitutes, complementary fuel requirements and pending-input demand reduction.
- A GPU fixture uses the actual malachite smelting and copper-tool recipes. It
  compares cargo not yet arrived, missing fuel, absent workforce and useful
  delivery. Only the final arm makes tools; consumed ore, copper yield and copper
  embodied in tools agree within 0.001 kg in the small fixture.
- The stock-recovery fixture verifies paid source→cargo→buyer transfers, finite
  source inventory, no duplicate ore purchase and serialized arrival continuation,
  alongside existing route, war, permission, capacity and estate protections.

These checks do not establish economic benefit. The forecast does not reserve
labor or residue capacity; execution continues to enforce both.

## Initial three-seed screen and correction

Matched seeds 1024/256/409 for 600 months against dc93a9d (runtime behavior acd49bf),
using the same founding archives and controls as the harbor demand comparison.
All three completed validation. Seeds 256 and 409 matched the baseline population,
hunger and operator work; seed 1024 differed only at rounding scale on those
measures. Maximum absolute relative cash residual was 1.29e-7.

However, seed 1024 bought 48.13 kg malachite for 120.32 currency at month 402,
then also bought finished tools. Both arrived at month 414; the ore remained
unused at month 600. This was not successful recovery into productive use.

The follow-up makes automatic recovery consider finished goods across all sources
before raw tool inputs. Raw offers therefore see the earlier committed finished
supply. A fixture intentionally starts the material rotation at malachite and
checks that useful finished tools ship while the unnecessary ore stays at source.
This is a bounded procurement priority, not a new monthly scheduler phase.
Initial ignored results are under `output/recovered-ore-screen/`. The priority-only
comparison under `output/recovered-ore-priority-screen/` reduced the seed-1024
ore purchase to 15.11 kg for 37.77 currency, but it still remained unused. Priority
alone was insufficient. The final candidate first allocates already supported
inputs to tool service, then evaluates the offered input against only the remaining
deficit. A negative control gives the buyer enough existing copper to meet the need
and verifies that cheap offered malachite produces zero additional demand.
The incremental-input comparison under `output/recovered-ore-incremental-screen/`
still bought about 15.11 kg in seed 1024, with no meaningful use. Its metalworking
activity was nearly zero: material feasibility did not imply a credible processing
customer. The final candidate bounds the extra input demand by three months of
the larger of last completed metalworking effort and funded operator work, using
the catalog recipe work needed by the additional chain. This is a procurement
forecast ceiling, not reserved future time; execution must still obtain workers.
Dormant workshops need a funded restart or demonstrated activity before this extra
recovery path stocks them. Ordinary explicit material targets remain separate.
A no-activity control verifies zero additional ore demand. Final comparison is
recorded under `output/recovered-ore-work-screen/`.

## Final screen and retained scope

All three final runs completed validation. Seeds 256 and 409 matched the baseline
population, hunger and completed operator work exactly in the reported exports.
Seed 1024 changed from 159.870712→159.870622 people and
0.036954269→0.036954306 ending need-weighted hunger; completed operator work
remained 11.218812 worker-months. Maximum absolute relative money residual was
1.29e-7. These are negligible outcome differences, not a welfare improvement.

In seed 1024, 1.70 kg of malachite was purchased at month 408 for 4.25 currency,
arrived at month 420 and remained unused at month 600. The successive refinements
reduced speculative buying from 48.13→15.11→1.70 kg, but did not establish a
productive long-run use. Existing finished-tool recovery continued. The feature
is retained for the demonstrated controlled path, under the existing optional
abandoned-stock recovery system; there is no new default switch or archive field.

Across four iterations, twelve 50-year runs completed. Their settings used
terrain/ecology 32/32 founding archives, zero additional geological epochs,
delivery-paid exports, service procurement share 0.25, contract/demand workshop
staffing, household inheritance, named office service and abandoned-stock recovery.
Commercial/service/council credit, shared issuance and estate reclamation were
disabled. Native development build on Quadro RTX 5000 with Max-Q Design; some
compilation/checks overlapped frozen-binary runs, so these are behavioral screens,
not isolated performance benchmarks. Strict library Clippy and native builds pass.
The final ordinary suite reports 194 passed and 150 ignored; targeted GPU recovery
and ore-processing controls were run explicitly and passed.

The next economic question is who funds useful continuing production and imports.
Stored material access alone did not create that customer. Keep workshop restart,
private purchasing-power-to-import funding, and unworked deposit access open;
do not add more speculative stock on the assumption that it will create demand.
