# Cross-scale coupling and counterfactuals

Repository audit, September 2026. This document combines an architectural roadmap with implemented slices.
See [model evidence](model-evidence.md) for current executable coverage and limitations.

The first implemented slice is documented in [shared resources](shared-resources.md):
shared accessible extraction, access interventions and checkpoint-branch experiments.
The broader proposals below remain a roadmap.

## What already connects

- Living history advances ecology before each social month, then commits managed
  withdrawals before the next environmental step. It preserves clock offsets,
  rejects incomplete checkpoints, and has continuation/batching fixtures
  (`src/civilization.rs`, `advance_history`).
- Plot claims transfer soil, detritus, vegetation nutrients, phosphorus reserves
  and water out of ecology. Reserved fractions suppress wild production on managed
  land (`shaders/economy.wgsl`, `claim_plots`).
- Fisheries withdraw accessible aquatic biomass; crop and livestock production
  use finite nutrient, water, feed and labor budgets.
- Markets transfer goods and money; household purchasing power constrains actual
  consumption. Migration transports population and possessions, while housing,
  route availability and destination conditions constrain acceptance.
- Material structures and artifacts hold real economic inputs. Wear, loss and
  destruction have ledger consequences. Provenance is strongest for unique objects.

## Where feedback stops

| Boundary | Existing behavior | Consequence |
|---|---|---|
| Geology to extraction | Optional shared sources with access policies and mineral-specific processing | Canonical finite withdrawals now implemented; secondary deposits and regional partitions remain absent |
| Planet to region | Derived regional snapshot with inherited geology and parent IDs | No canonical local edits, overlapping extraction rights or stitched upstream river inflows |
| Managed land to water | Optional bounded nutrient/runoff transfer to the actual fine river network | Domestic outflow and other water uses still need distinct destinations; leaching needs longer calibration |
| Abandonment to ecology | Optional finite land return, reservation release and reoccupation withdrawal | Recovery remains a coarse regional fraction, without a resolved town footprint |
| Expeditions to environment | Separate finite accessible coastal specimen sources | Repeated collection exhausts the source ledger but not corresponding planetary biomass/minerals |
| Historical production to archaeology | Optional finite processing residue with first-deposition event, site location and survey views | Aggregate deposits persist; recovery, detailed strata and other production waste remain incomplete |
| Population to named people | Cohort health and household conditions plus sparse genealogy/agents | Individual biographies cannot safely claim exact exposure or possessions from averages alone |
| Scenario to living history | Boundary-validated interventions record both clocks and a history-event reference; [fishery controls demonstrate the immediate food pathway](living-scenarios.md) | Future scheduling, long-run natural-stock trajectories and wider mediator experiments remain |

These are explicit abstraction boundaries, not all conservation bugs. A closed
subsystem budget can still hide missing feedback if an internal world transfer
is represented as an external import/export.

## Proposed coupling contract

Every physical stock has one authority. Other representations are views,
reservations or subdivisions of that stock. Ownership claims and knowledge about
a resource are separate from physical custody and remaining quantity.

Each interaction specifies:

- Source/destination identity, material/population identity, units and spatial support.
- Who owns the source stock, who proposes the transfer, and who commits it.
- Requested and accepted quantities, prerequisites and capacity limits.
- Departure, travel and arrival timing; in-transit custody where necessary.
- Conversion products and declared losses, with physical receiving compartments.
- Which observations actors can use, separately from the simulator's actual state.
- Diagnostic counters and an intervention/cause reference when applicable.

Use typed transfer families rather than a universal per-cell event object. Dense
GPU transfers remain compact arrays; sparse trade, ownership and historical
records remain CPU structures. Aggregate routine traffic; retain detailed traces
for counterfactual diagnostics, significant batches and historical events.

A domain boundary follows propose -> arbitrate -> commit -> observe. Requests
sharing a source are bounded against one completed snapshot and committed once.
Arbitration follows explicit access rights, capacity and shortage rules, not
incidental settlement iteration order. Money, labor and route capacity must also
be reserved where required. A failed commit must not be partly checkpointable.

## Representation across scales

Start deposits at the existing canonical planetary grid. Give each finite source
an identity and explicitly initialized material inventory. A prospect is evidence
about the source, not a second inventory. Distinguish total source material,
concentration, accessible material and recoverable output; mining also requires
labor, tools and access. Geological renewal, if modeled, consumes a geological
source rather than resetting extraction stocks.

Later regional refinement partitions an existing source budget over canonical
subcells/chunks. Overlapping surveys reference the same children. An unrefined
remainder is retained; refining cannot create ore, water, plants or residents.
Physical local edits update the authoritative partition and its parent summary.
Survey-only elevation noise remains a view until a physical refinement contract
exists. Reopening a depleted region cannot regenerate it from its original seed.

Restriction to coarser grids must distinguish extensive quantities (sum mass,
volume, people) from intensive quantities (area-weighted concentrations, density,
temperature where appropriate). Preserve land/water fractions and receiving-basin
identity; do not spread river nutrients across unrelated waters in one coarse cell.
Store fractions/distributions for narrow hotspots and disrupted corridors when a
mean would erase the process. Equal totals do not guarantee equal dynamics under
refinement, so conservation tests and response-convergence tests are separate.

## Feedback with physical delays

Local effects reach higher scales through actual quantities and constraints:

- Extraction -> remaining accessible ore -> mine labor/yield -> tool supply and
  prices -> household purchasing power -> migration and political revenue.
- Land clearance -> vegetation/soil retention -> runoff and nutrient export ->
  receiving water -> aquatic production/oxygen -> fishery yield and food prices.
- Abandonment -> transfer remaining managed pools back to ecological custody ->
  succession, with surviving assets and degraded soil retained.
- Workshop production -> actual slag/waste -> dated site evidence -> later
  recovery, with recoverable mass withdrawn once from its physical compartment.

Do not add a global prosperity or pollution modifier simply because an event
occurred. Effects should be geographically bounded and delayed by transport,
production, ecological response or institutional decisions. Not every town action
should alter the entire planet. Fixed atmospheric/geological boundaries can remain
explicit approximations where a responsive global model is not justified.

Keep different physical clocks. Monthly social coupling does not imply monthly
tectonics. Fast transport can substep; slower land-cover/erosion responses can
accumulate measured fluxes for later updates. Record which month's snapshot each
consumer reads. Keep a documented lag rather than repeatedly iterating politics,
markets and ecology to artificial instantaneous equilibrium.

## Counterfactual execution

Branch a validated monthly checkpoint into a baseline and treatment, preserving
catalogs, stocks, clocks and prior history. Apply a typed, archived intervention
at a stated boundary; do not rerun founding independently. Distinguish changes to
policy/access from destructive shocks and parameter experiments.

Examples: close a mine without deleting its ore; restrict extraction labor; close
a shipping route while preserving in-transit cargo; remove a guild by transferring
its biomass into detritus; install waterworks by paying for actual construction.
Injected aid is an explicit external intervention with a declared inventory.

The current scenario API validates the shared boundary, pending returns and both
clocks, then records a linked history event. It applies immediately at that boundary;
a future scheduling queue still needs to preserve those same invariants. See
[living scenarios](living-scenarios.md) for implementation and controlled evidence.

Use common exogenous forcing and random keys based on stable entity identity,
month, subsystem and decision kind. Existing keyed cultural randomness is a useful
start. Audit append-order IDs and shared random streams: an intervention that adds
one event or birth must not accidentally reshuffle unrelated weather or decisions.
Entities created only in one branch need not have artificial counterparts.

Report trajectories and mechanisms, not only end population: extraction, stocks,
prices, fulfilled demand, household food access, migration, ecological state and
institutional outcomes. Separate direct debit/credit effects from subsequent model
responses. Event cause links aid explanation but do not establish causal attribution
for every downstream divergence. Use several checkpoint dates/seeds and controlled
fixtures; counterfactuals establish behavior of this model, not empirical validity.

## Delivery status and remaining acceptance gates

The list below preserves the original dependency order. Steps 1–3 have implemented paths: [shared sources](shared-resources.md), [checkpoint experiments](model-evidence.md), and [environmental returns](environmental-returns.md). Their linked evidence states what was actually checked. Steps 4–5 remain incomplete: mineral-specific processing is not complete product provenance, and regional surveys are not authoritative editable sites.


1. Shared finite deposits and extraction claims. Two towns compete for one deposit;
   aggregate withdrawal never exceeds it. Survey/refinement cannot add stock;
   depletion survives save/load, overlapping views and reoccupation. Existing town
   reserves migrate as identified legacy sources, not invented historical deposits.
2. Coupled monthly intervention queue and checkpoint-branch evaluator. No-op branches
   match exactly on the same backend, batching preserves results, unavailable-stock
   requests fail safely, and ecology interventions preserve both clocks.
3. Close one environmental return loop: abandoned-land release, followed by managed
   runoff to actual receiving waters. Transfers balance across grids, seams and
   coastlines; downstream response appears only after transport.
4. Material provenance through one industry and historical waste deposits. Smelting,
   sale, wear, burial and recovery preserve mass and custody. Mixing retains bounded
   source information with explicit unknown provenance where detail is discarded.
5. Regional editing contracts and situated person views. Refined actions use the
   same source authority; individual evidence is distinguished from household or
   settlement exposure. Sampled NPCs reference existing population rather than
   adding new people or additional household property.

For each gate test direction, locality, lag, saturation and recovery as well as
budget residuals. A shut mine should leave ore in place, shortages should depend
on reserves/import alternatives, and reopening should recover only if labor,
access, tools and demand still permit it. Avoid requiring a particular political
outcome from every economic shock.
