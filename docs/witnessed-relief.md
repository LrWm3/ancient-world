# Witnessed appeals and household choices

New societies require an arriving help-seeking household before a town can organize outside food relief. The old annual rule that inspected every town's shortage and automatically selected a donor is disabled under this policy. Neighbor-charity decisions also require a recent appeal; ordinary commerce and local assistance continue.

The History window's **Household relocation** section has **Require arriving families to request outside relief**, with recent reports and response status. `History::set_witnessed_relief(bool)` exposes the same control. Older archives retain their previous behavior until enabled. Pending appeals and already funded shipments continue if the policy is changed.

## Decisions and information

The existing sustained-hardship trigger now considers each eligible household's ambition, loyalty, caution and persistent seeded attachment. Some households remain, others become candidates to leave; leadership obligations still prevent a ruler abandoning their post. Annual notices record when no eligible household chooses departure. The origin still sends at most one household a year and retains a resident household. These are sparse household decisions over population cohorts, not an individual evacuation census.

Two of three household IDs have a stable help-seeking preference; the others seek permanent resettlement without requesting relief. Help-seekers require the route's travel provisions plus one reserve month, versus three reserve months for other movers. All provisions come from existing town food, leaving a month's food for residents staying behind. No departure is a free rescue. Families still need an admissible nearby town with spare productive capacity and land, and actual open roads. Distance and destination relationships affect which host is preferred.

Only successful arrival creates an appeal. The record contains the originating household, origin and host, route, departure date, population remaining when they departed, arrival date, causal event and eventual response. The host does not inspect the origin's current hunger to discover the emergency. Reports become stale after 18 months. A new appeal from the same origin within two years cannot authorize a duplicate response at another host.

The host answers no earlier than the following month. Shared political control favors assistance. For other controllers, diplomatic trust, recorded trade contacts and household lineage ties affect willingness; greater distance requires stronger ties. Active war, closed routes, host hardship and stale testimony prevent funding. The host retains twelve months of food, and aid is capped by the reported population and a distance-dependent shipment allowance. This is a bounded use of the existing abstract relief transport system, not a new crew/vehicle simulation.

Food is withdrawn immediately and enters the existing shipment inventory. Closure delays delivery and causes spoilage; six blocked months terminate a shipment. An abandoned destination cannot consume a relief delivery: the shipment is recorded as lost. Arrival, loss and response events reference the appeal chain. A successful delivery replenishes the origin's actual food inventory, which can support staying residents or provision later independently chosen departures. It does not force families to evacuate or promise rescue capacity.

## Limits

There are no dedicated refugee camps, guaranteed return rights, new maritime journeys, independently controlled family provision pledges, or organized convoy pickup rosters in this increment. Arriving families settle through the existing relocation system; return trips remain the existing failed-destination behavior. The new distinction is witnessed information, differentiated household departure choices, and affordable aid transported back to the origin.

## Validation and reproduction

The controlled GPU fixture covers no appeal before arrival, deduplication, stale testimony, distance/weak ties, insufficient host stocks, finite funded shipments, single response, and conservation. Existing relocation fixtures cover starvation, failed-destination returns, identity and in-transit serialization. Integrated evaluation compares seeds 17, 81 and 256 for 100 years at yield 0.33, with living ecology and discoveries enabled, against the legacy policy.

```sh
mise exec rust@1.89.0 -- cargo build --release --example history_evaluate
# Add --legacy-relief for the paired control.
target/release/examples/history_evaluate --seeds 17,81,256 --years 100 \
  --crop-yield-scale 0.33 --discoveries --living-world --save-worlds \
  --label witnessed --output output/witnessed-relief/enabled
```

Reports are in `output/witnessed-relief/`. These small diagnostic worlds use 64 cells per face for both terrain and ecology. Execution times from concurrent GPU runs are not performance benchmarks.

## 100-year results

| Seed | Population, legacy → witnessed | Departures | Appeals / funded / refused | Food crises, legacy → witnessed | Abandonments |
|---|---:|---:|---:|---:|---:|
| 17 | 2,174 → 2,209 | 4 | 1 / 1 / 0 | 54 → 56 | 1 |
| 81 | 2,220 → 2,236 | 1 | 1 / 1 / 0 | 48 → 47 | 1 |
| 256 | 4,219 → 4,219 | 2 | 1 / 1 / 0 | 15 → 15 | 0 |

These compare the whole new policy, including household choices and the reduced help-seeking reserve requirement, against legacy behavior; they do not isolate the benefit of delivered food. Aid is uncommon under the conservative admission rules. Refusal and stale-report behavior are tested with controlled fixtures instead of forcing every seed to exhibit them. The largest normalized ledger residual was 2.64e-05. Final additions to stay/refusal event explanations were validated separately; these seed runs measured the same underlying choice and aid rules. `comparison.json` records traced appeal chains.

Validation completed: 102 tests passed including hardware-GPU tests, followed by the expanded household stay/leave fixture. Clippy with warnings denied and formatting checks passed. The final build continued seed 17 for 24 months identically across batched execution versus monthly execution with a checkpoint at month 12; history, terrain and ecology matched. The three-world lifecycle audit found no post-abandonment local activity notices.
