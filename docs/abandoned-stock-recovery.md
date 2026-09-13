# Paid recovery of abandoned bulk stocks

Status: experimental implementation; one matched 50-year screen is complete,
with limited recovery and no measured workshop improvement.
Enable with `--abandoned-stock-recovery` (or explicitly disable with `=false`).
Omission preserves `Society.stock_recovery`; new and older societies default to
false until calibration. `History::recover_abandoned_stock` also exposes a bounded
explicit request.

At the end of quarterly market decisions, after ordinary purchases have reserved
cash and freight, an enabled town may buy retained goods from an empty abandoned
site on a passable direct land route on the same inner continent. Different
administrations may trade when both sites retain trade permission and no active
war connects their controllers. Payment remains with the estate; this is a
standing purchase permission, not a simulated negotiation with absent residents. Stock, buyer demand, incoming shipments, cash, dry storage and shared
freight all cap the amount. Goods rotate priority monthly; town iteration remains
stable. Recovery does not reopen earlier production or labor reservations.

Actual goods leave the source's available inventory once, becoming bonded recovery
cargo. The cargo represents both goods awaiting collection and the return load;
it is not a claim that loading happened instantly. Buyer stocks increase only at
the due boundary, after twice the ordinary land journey duration. One-way distance
is limited to six months. Buyer-provided aggregate freight is occupied throughout;
an abandoned endpoint supplies no imaginary workforce. This uses the game's
existing carrying-capacity abstraction, not named crews or a separate wage contract.
Road surface limits throughput. Dispatch requires access; subsequent closure or
flooding delays delivery under existing cargo spoilage/loss accounting.

Payment goes from the buyer's real operating cash to the retained source estate's
operating account at its stored quote. It is a purchase, not new currency or free
confiscation. Household wallets, ownership shares, institutions, objects, buried
resources, buildings and geological reserves are not directly transferred. The
estate retains money and existing claims. Consequently, this can return materials
to use without solving retained-cash circulation; inheritance and legal recovery
remain separate mechanisms.

Recovery cargo persists in the ordinary cargo archive with a default-false marker.
It uses the same material/nutrient and money ledgers and eventual delivery path.
Recovery departures retain round-trip route geometry, while arrivals are explicit
recovery events. Collection from an empty place does not generate lexical or social
trade contact with nonexistent inhabitants. Older cargo remains ordinary trade.

This version does not cover sea salvage, individual recovery parties, excavation,
foreign seizures, extraction from unworked deposits, or recovery of ruined building
materials. Nor does it guarantee profitable recovery: acquired inputs must still
meet real production and demand constraints after delivery.

## Initial verification

The hardware-founded controlled fixture passes (1.06 s). It checks actual stock
and cash transfers, unchanged household accounts, round-trip delivery timing,
source exhaustion, insufficient cash/demand, occupied sources, a closed
road, automatic policy opt-in, shared freight exhaustion, and replay of the same
in-transit serialized history through a closure and reopening. Missing policy and
cargo marker fields import with their disabled/ordinary defaults. This is history
serialization replay, not a full GPU checkpoint/batch comparison.

The existing ordinary land-freight capacity/supplier-selection test passes (1.07 s),
and strict all-target Clippy passes. Subsequent production and balance checks are
recorded below; full checkpoint continuation remains pending. No claim of
improved population, workshop profitability or general circulation follows yet.
The native build also passes, and `--help` exposes the explicit true/false option.
Repository artifact and whitespace checks pass. Generated logs remain ignored.

## Cross-administration and production checks

A seed-1024 endpoint revealed an empty site with approximately 1,012 kg of bulk
stock and an open 831 km road to an inhabited site on the same inner continent.
The previous same-administration restriction excluded that buyer despite both
standing trade policies permitting purchases and no active war. Eligibility now
uses those permissions rather than requiring common administration.

The recovery fixture passes with foreign purchases, source trade closure,
different inner continents, active war in either direction, and ended-war controls
(0.94 s). Rejected requests leave the serialized state unchanged.

A separate hardware production fixture passes (2.34 s): recovered metal enables
tool output only after actual cargo arrival. No recovery, undelivered cargo and
recovered timber do not enable that recipe. Without prepaid operator attendance,
public production can still use delivered metal, but private operator work remains
zero. Input use matches output in this one-to-one fixture and private work never
exceeds prepaid attendance. This isolates real cargo delivery followed by the GPU
production dispatch; it is not a full monthly-history or checkpoint comparison.

The experiment runner can now hold `--abandoned-stock-recovery` on across all
monetary arms. The 12 existing runner unit tests pass. Long-run results follow
separately; these controls alone do not establish profitable recovery.

## Matched 50-year screen

Eight runs completed: seed 1024, recovery off/on, each with baseline, credit,
issuance and combined monetary settings. Frozen founding checkpoint:
`output/monetary-estates-heldout-founding/1024.world` (32-cell terrain and ecology,
frozen environment, five founding civilizations). Common options were funded
service procurement at 0.25, contract/demand workshop staffing, local estate
inheritance and named office service. Cash reclamation remained off. Reproduce
with `scripts/monetary_experiment.py --years 50` and those common flags, adding
`--abandoned-stock-recovery` only to the treatment. Local artifacts are under
`output/recovery-border-screen/{off,on}`; binaries and raw exports are ignored.

| Arm | Population off → on | Recovery dispatches / arrivals | Recovered kg, approximately | Estate payment, approximately | Operator completed work off → on |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | 163.153 → 163.044 | 14 / 14 | 31.05 | 122.95 | 4.450 → 4.450 |
| Credit | 163.153 → 163.044 | 14 / 14 | 31.05 | 122.95 | 4.450 → 4.450 |
| Issuance | 162.169 → 162.117 | 5 / 5 | 7.91 | 25.89 | 4.456 → 4.456 |
| Combined | 162.169 → 162.117 | 5 / 5 | 7.91 | 25.89 | 4.456 → 4.456 |

Recovered quantities/payments in the table sum rounded event descriptions;
canonical inventories and money accounting use the underlying values. Baseline
bulk stock at the abandoned site falls from 1,012.213 to 994.429 kg and its food
stock also supplies recovery. Its operating cash rises from 62.061 to 185.021.
The maximum absolute relative cash residual across all eight endpoints is
9.35e-8. All dispatched recoveries arrive by the endpoint in these runs.

Operator margins are unchanged (+25.76 baseline/credit, +25.89 issuance/combined)
and all operators are closed at year 50 in this seed. Thus the feature releases
some finite stocks but does not resolve workshop viability or improve population
in this screen. Payment also accumulates at the estate: material recovery is not
cash reclamation. A full explanation of downstream demographic divergence needs
more than endpoint comparisons.

At the disabled endpoint, the accessible buyer already exceeds timber and brick
targets. It has zero purchase targets for the estate's malachite ore, copper tools
and copper scrap. Do not infer universal lack of use from one endpoint, or force
purchases merely to empty ruins. Review material substitution and input demand
alongside actual funded workshop work next.

The disabled baseline exactly matches the preceding saved history after accounting
for the new default-false `stock_recovery` field. The native build, strict
all-target Clippy, both hardware fixtures and 12 experiment-runner unit tests pass.
The experimental runs overlap a compiler check during part of the control arm;
these are behavioral comparisons, not isolated performance benchmarks.


## Service-equivalent tool recovery

An additional recovery ceiling now recognizes bronze/copper tools as substitutes
for an unmet basic tool reserve, even when the current local recipe plan names
generic tools. It requires alloy use and ordered production to be enabled. Held
generic/bronze tools count at full service and copper at the shared 0.6 factor.
Incoming cargo of every tool variant and planned contracted deliveries count
against the same requirement. The ceiling is the existing per-person tool reserve,
not a new desired quantity per material.

An alternative qualifies only when its estate quote per service is no higher than
the buyer's generic-tool quote. This is a local heuristic, not a comparison against
all possible suppliers or a lifetime-cost optimizer. Normal procurement retains
first access. Price, payment, round-trip freight, stock, access and warehouse caps
remain unchanged. The tool ceiling permits substitution; it neither creates goods
nor makes them usable before arrival. Once delivered, the normal planner retains
useful alternatives through their real reserve targets.

An explicit target for a specific material continues to express its own demand;
the quote rule governs additional substitute demand. The system does not repurpose
unprocessed ore as tools or promise that recovered tools create private employment.


The hardware-founded recovery fixture passes (1.17 s) with additional controls:
a five-unit service deficit buys only 5/0.6 kg of copper tools, incoming copper
blocks another copper or bronze purchase, delivered tools continue to cover the
need, and money is conserved. Sufficient stock, disabled alloy use and an inferior
service quote reject the substitute purchase without mutation. The fixture uses
the real alloy catalog; its first attempt correctly failed because the base
catalog leaves those goods reserved. Earlier route, war, exhaustion, serialization
and freight controls remain in the same passing test.


### Substitute recovery screen

Four further 50-year seed-1024 runs completed, holding the preceding tool-retention
change and every common monetary-screen setting fixed. Outputs:
`output/recovery-substitutes-screen`; comparator: `output/tool-retention-screen`.

| Arm | Population before → after | Copper tools left in ruin before → after (kg) | Operator work before → after | Recovery arrivals before → after |
| --- | ---: | ---: | ---: | ---: |
| Baseline | 163.451 → 164.955 | 38.031 → 0 | 6.411 → 6.411 | 8 → 6 |
| Credit | 163.451 → 164.955 | 38.031 → 0 | 6.411 → 6.411 | 8 → 6 |
| Issuance | 162.906 → 164.290 | 39.198 → 0 | 6.251 → 6.251 | 7 → 8 |
| Combined | 162.906 → 164.290 | 39.198 → 0 | 6.251 → 6.251 | 7 → 8 |

In baseline, month 414 reserves 38.03 kg of copper tools for approximately 202.71
currency (rounded event values), returning at month 426. At year 50 the buyer
holds 6.551 kg, records 31.480 kg of cumulative tool use and retains 28.332 kg of
copper scrap. Thus the recovered goods are not merely sitting at the new endpoint:
they enter the existing wear/recycling model. The estate is paid; none of these
transfers create currency. The maximum absolute relative money residual in the
four new runs is 1.28e-7.

Workshop operating margins are unchanged at +41.68 baseline/credit and +39.88
issuance/combined; all operators remain closed at year 50. A modest population
increase is encouraging in this seed but does not establish general balance or
attribute all downstream changes solely to productivity. Arrival counts can fall
because shared cash/freight and subsequent demand change; count alone is not the
amount or utility of recovered resources.

Native build and strict all-target Clippy pass. The baseline partially overlaps
compiler work, so no isolated timing conclusion is drawn. Held-out seed screens,
general supplier-aware substitution and a full-history checkpoint comparison for
substitute recovery remain outstanding. The larger circulation and sustained
workshop-customer problems are not resolved by these results.
