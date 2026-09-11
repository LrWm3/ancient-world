# Research and cultural work requests

Research and culture now estimate work from opening-boundary opportunities rather
than reserving an unconditional quarterly cultural allowance. Existing sea-cargo
commitments retain priority. Requests consume the remaining shared labor ceiling;
research retains its two-worker-month cap and culture its half-worker-month cap.

## Research

A read-only forecast walks the same specimen kinds as execution. It includes:

- Annual method learning from an accessible workshop that already knows the method,
  requiring 0.05 kg of writing material and 0.25 worker-months.
- Study or processing of actual local specimens, limited by application needs,
  remaining tools, fuel and worker availability.

Forecast supplies are subtracted locally between actions, preventing both specimen
kinds from each requesting work against all the same tools/fuel. Forecasting a
learned method does not create a knowledge event. Execution checks the current
teacher, route and stock again. Method-only learning now works without specimens,
tools or fuel; previously the reservation gate required local specimens even though
the learning action did not. Tiny processing batches no longer encounter a separate
hardcoded minimum tool/fuel gate.

## Cultural demand

`Culture::work_requests` lists action categories and their requested worker-months.
They include locally staffed institutional upkeep/elections, accessible documents,
knowledge successors, recoverable objects, pilgrimage opportunities, specimen
curation, viable institutional founding, material-backed craftsmanship, campaigns,
charity, ownership disputes and due heritage study. Resident actors, knowledge gaps,
material stocks, dates, routes and existing deterministic actor-choice tests inform
the requests. A town with no eligible people or activities does not automatically
reserve 0.5 worker-months.

`History::cultural_work_requests()` exposes a read-only JSON view of the current
requests and capped total. This is a current-state query, not an archived statement
of earlier decisions or completed work. No archive schema changes were necessary.

Personal decisions previously counted the entire available grant as completed work
before checking whether any action succeeded. They now charge explicit successful
study/teaching/founding/crafting/charity/dispute work. Recovery and curation cost
0.1 worker-months; pilgrimage charges actual route-derived travel work. Unused
personal grants no longer inflate the completed-work counter or increment personal
action/skill counters. Institution, heritage and petition handlers retain their
existing explicit charges.

## Limits

These are demand estimates, not a universal action-proposal/commit engine. They do
not lock goods or people before production. Earlier consumers can exhaust supplies,
actors can leave, and routes can close before execution. Execution can therefore
complete less work than reserved. Candidate activities can also become possible
later in the month, and still compete within the site's grant.

Institutional petition hearings remain a bounded candidate request; their existing
political scoring decides whether a petition is ultimately justified. Curation
rechecks catalog chemistry at execution. Several individually feasible actions can
compete for the same materials; execution's existing inventory transfers prevent
duplication. There is no claim that every forecast worker is subsequently productive.

Maintenance and special-action priority inside culture remain as before. A small
combined grant can fall below an action's minimum work and go unused. Further
per-action arbitration would be needed to eliminate those cases rather than merely
avoiding blanket allowances. Resource holds and persistent action identities are
outside this increment.

## Tests

The cultural fixture removes available actions and confirms zero demand, then
introduces a teachable knowledge gap and obtains a 0.1 request and a real teaching
event. It verifies an unused 0.5 grant does not count as completed work and that
adding/removing crafting material creates/removes the matching request.

The research fixture contrasts closed/open contact and absent/present writing
material. A method-only request completes with zero local samples/tools/fuel and
uses 0.25 worker-months. Fuel-free processing requests zero; scarce tools permit
only the corresponding small batch. These setups modify fixture state explicitly
and do not represent long-run world calibration.

The prior scarcity and vessel fixtures now allow actual cultural demand below the
old flat ceiling. Historical comparisons in earlier reports describe their stated
commits, not the current dynamic-request implementation.

Routine institutional administration is also an explicit request: successful dues
and knowledge-administration work charges 0.05 worker-months per institution and
stops when the grant is exhausted. This preserves a feasible funding activity that
previously depended on leftover blanket work, without making that administration
free. It does not alter the existing dues transfer formula.

The first new fixture exposed newly created household heads without synchronized
cultural agents. Monthly reservation now synchronizes those records before the
forecast; read-only queries return no requests for an unsynchronized site rather
than indexing missing agents. The fixture explicitly synchronizes its baseline
before altering actor knowledge. This is initialization handling, not invented
knowledge transmission.

Final verification on the Quadro backend: 71 ordinary library tests passed (68
hardware tests ignored by that command). Targeted GPU runs passed the research
fixture, 13 culture fixtures including the new request test, two institution-upkeep
fixtures, the crew fixture and three enterprise fixtures including the 144-case
scarcity comparison. All three history-environment fixtures passed in 25.66 seconds
excluding compilation; seed 17/81/256 batch and checkpoint continuations remained
exact. Strict Clippy, formatting and repository artifact checks passed. This is
verification of requests, work accounting and continuation, not long-run balance
calibration. Generated logs stay in ignored `output/`.
