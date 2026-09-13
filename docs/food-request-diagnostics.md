# Food request constraints

Quarterly ordinary-market food requests now leave bounded, cumulative observations
in `History.trade_contact.food_requests`, one record per buyer site. These are
observations only: they change neither purchasing decisions nor money, food,
shipping, historical events or information available to residents. Old archives
start with empty counters; no earlier requests are reconstructed.

An enabled, occupied buyer with at least one kilogram of unmet food target records
one outcome. Pending cargo is deducted by the existing market rules before that
point. There is no observation on non-purchasing months or when stock and incoming
cargo already meet the target.

With no selected supplier, the diagnostic tests existing eligible surplus first,
then its accessibility. The three gates are:

1. No active trading seller has food above its protected reserve.
2. Surplus exists, but none has an eligible route within the market distance limit.
3. Reachable surplus exists, but no supplier passes current freight availability.

The route predicate is shared with actual supplier selection, including legacy
versus network distance semantics. Sea routes require usable vessel capacity, so
an uncommissioned harbor, unavailable ship or closed connection can appear under
`no_usable_route`; this is not a claim that the geography is impassable. The final
freight gate also includes missing freight-service paths.

With a supplier selected, record the first tightest quantity bound in this order:
need, storage, freight, seller surplus, per-purchase ceiling, available money.
Record the requested and dispatched quantities and number of actual dispatches.
A quantity below the minimum shipment still records its limiting bound, with zero
dispatched. The current food path does not enforce durable-goods storage; its
storage bound is infinite. Keeping that slot reflects the common amount expression
without claiming it is an active food constraint.

Constraints are sequential evidence, not mutually exclusive real-world causes.
For example, a buyer lacking a route might also lack money. Repeated requests can
count the same unmet need across quarters. Dispatched food is not delivered or
eaten food. Counters persist across checkpoints and site abandonment and are never
included in material or cash ledgers.

Use `python3 scripts/audit_circulation.py output/history.json` for labeled counts.
The audit returns `null` for old exports without observations, rather than treating
missing evidence as zero failed demand.

## Verification

The focused GPU-founded market fixture covers six cases: per-purchase ceiling,
cash, partial freight capacity, no free freight, no surplus and no cross-island
route. It checks one recorded constraint for the buyer, equality between diagnostic
dispatch quantity and actual outgoing cargo, no premature food at the buyer,
validation and serialization continuation. A non-quarter boundary records nothing.
Seven Python audit fixtures pass, including absent old observations and ensuring
new counters do not enter the cash or stock summaries. The ordinary library suite
passes 194 tests with 151 hardware tests ignored; the new hardware fixture was
separately executed.

## Three-seed fifty-year evidence

Same founding archives and settings as [the import sizing screen](food-import-circulation-screen.md),
with the rejected sizing change absent. Baseline runtime is `bdedba7`; `af1a6f9`
adds documentation only. Each new run used a frozen executable and completed 600
months at terrain/ecology 32/32 on the Quadro RTX 5000 with Max-Q Design.

| Seed | No surplus | No usable route | Freight is tightest | Batch ceiling is tightest | Money is tightest | Dispatches |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | 641 | 94 | 68 | 26 | 0 | 94 |
| 256 | 23 | 651 | 0 | 0 | 0 | 0 |
| 409 | 64 | 621 | 0 | 0 | 0 | 0 |

All other constraint slots were zero. Seed 1024 dispatched 8,844.71186 kg by the
unrounded counter. Sites 3 and 4 received those orders; sites 1 and 2 each had 47
inaccessible-surplus requests and no food dispatches. Site 4's last request was
month 399, before its eventual abandonment. Seeds 256 and 409 made no ordinary
food dispatches even though most unmet requests had eligible surplus elsewhere.

After removing only `trade_contact.food_requests`, the complete parsed JSON history
matches the baseline exactly for all three seeds: this comparison includes events,
people, inventories and accounts, not just final population. Native build and
strict library Clippy passed. Outputs remain ignored under
`output/food-request-screen/`; this Markdown is the published summary.

### Consequence for the next circulation experiment

These results do **not** show that residents always have enough money. Cash was
never the first tightest bound on an eligible food order, while earlier supply
and access gates often prevented an order from reaching that comparison. The
household-funding proposal should therefore wait: it would not repair the measured
route failures. Investigate specific surplus-to-shortage pairs and which missing
harbor, vessel, distance or route service prevents delivery. Then test funded,
selective completion of useful connections, rather than the previous blanket
harbor staffing change. Preserve enough working tools and labor for production,
and measure actual delivered cargo as well as commissioned infrastructure.

Seed 1024 separately needs freight and food-reserve/production investigation.
Larger batches and lower reserves already failed to improve its overall outcome;
these counters do not overturn that result. Workshop viability and extraction
from abandoned deposits remain unfinished parts of the circulation objective.
