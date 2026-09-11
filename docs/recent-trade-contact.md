# Recent trade contact

Completed land and sea cargo deliveries now contribute kilograms to a common
rolling twelve-month observation, grouped by directed settlement pair and month.
The window is pruned in monthly Open. Delayed cargo counts only on delivery;
spoiled or lost cargo does not contribute its lost mass. Orders, route existence,
and promised payments are not observations. Old archives start with an empty
window rather than inventing past transactions.

This is derived information, not another goods or money inventory. It cannot
consume stock, pay anyone or alter delivered quantities. The actual market arrival
path records it once when removing the delivered cargo from transit. Multiple
consignments on the same pair/month combine, and checkpoints preserve the window.

Two consumers use it:

- Annual political interest updates use local imports plus exports over the recent
  window. Exposure is `kg / (kg + residents * 18 * 12)`: a bounded commercial
  activity proxy normalized against a year's staple mass. This replaces lifetime
  sales divided by initial finance. Old trading success no longer guarantees
  permanent merchant appeal. It is not household merchant profit or real wages;
  price and volume should remain distinct if that next connection is added.
- Heritage news uses current-month pairs delivering at least one kilogram as
  contact opportunities, in either direction. The completed witness snapshot
  prevents same-month multi-hop propagation. Roads without traffic no longer
  spread fame; sea traffic can. This does not simulate dispatch-time messenger
  knowledge, individual conversation or probability proportional to cargo mass.

Boundary tests cover expiry, split deliveries, unrelated sites, serialization,
no-traffic isolation and one-hop transmission. Long-run political consequences
need the integrated balance evaluation, not only these mechanism checks.
