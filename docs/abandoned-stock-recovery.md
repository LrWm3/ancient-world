# Paid recovery of abandoned bulk stocks

Status: experimental implementation; long-run balance evaluation is pending.
Enable with `--abandoned-stock-recovery` (or explicitly disable with `=false`).
Omission preserves `Society.stock_recovery`; new and older societies default to
false until calibration. `History::recover_abandoned_stock` also exposes a bounded
explicit request.

At the end of quarterly market decisions, after ordinary purchases have reserved
cash and freight, an enabled town may buy retained goods from an empty abandoned
site on a passable direct land route. Current administration and island access
are checked. Stock, buyer demand, incoming shipments, cash, dry storage and shared
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
source exhaustion, insufficient cash/demand, occupied or foreign sources, a closed
road, automatic policy opt-in, shared freight exhaustion, and replay of the same
in-transit serialized history through a closure and reopening. Missing policy and
cargo marker fields import with their disabled/ordinary defaults. This is history
serialization replay, not a full GPU checkpoint/batch comparison.

The existing ordinary land-freight capacity/supplier-selection test passes (1.07 s),
and strict all-target Clippy passes. Full-world balance runs, production use of
returned inputs, and full checkpoint continuation remain pending. No claim of
improved population, workshop profitability or general circulation follows yet.
The native build also passes, and `--help` exposes the explicit true/false option.
Repository artifact and whitespace checks pass. Generated logs remain ignored.
