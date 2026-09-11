# Crew demand and monthly commitments

The scarcity comparison at commit `b92efbf` showed that merely moving every crew
request earlier substitutes workshop starvation for shipping starvation. Ports
previously requested full staffing for all backed hulls even with no cargo.

## Implemented change

The monthly reservation boundary now:

1. Clears last month's service/enterprise reservations, fleet funded-work counters
   and employer-income markers once.
2. Funds crew requests for cargo already occupying a sea lane.
3. Reserves research, cultural and enterprise work using the remaining service
   allowance.
4. Tops up crews for outstanding commitments and at most 100 kg of additional
   standby capacity per port, if labor and treasury remain available.

A cargo's `sea_lane` determines its two port endpoints, matching `sea_capacity`.
Each endpoint requests one worker-month per 1,000 kg of in-transit cargo. Inland
cargo creates no sea request. Cargo removed by opening-month arrival no longer
reserves capacity. Closed lanes still retain their cargo reservation; closed/flooded
ports cannot hire. No new route or terrain readback is required.

A port may fund at most its backed hulls, 0.25 worker-months per vessel, its resident
household slots, its treasury and the existing health-adjusted service ceiling.
Work and exact payroll accumulate across the two passes. The second pass requests
only the difference between its target and already-funded work. Partial payment
cannot authorize more work than reserved. Enterprise preparation no longer clears
the employer-income record of crews already paid earlier in the same month.

Standby is a game-design allowance: it leaves room for new trade without requiring
existing cargo to bootstrap the first shipment. It is 100 kg beyond commitments,
not a second independent fleet. There is no promised demand forecast or optimal
staffing claim. Full idle-fleet staffing is removed.

## Scope and limitations

This prioritizes **existing commitments**, not hypothetical trade profitability.
It does not infer urgency from a commodity name or automatically give speculative
food orders priority. Existing voyages can still be underfunded if workers, cash or
hulls are unavailable; they cannot mint crew labor. The existing cargo travel and
arrival rules remain unchanged: this patch does not newly delay cargo for crew
shortfalls or reconstruct vessel-by-vessel voyages.

Culture and research still request aggregate allowances; enterprise demand still
uses observed workshop activity. A common action-request allocator and completed
versus paid work calibration remain separate follow-ups. Relief journeys that do
not use a sea-lane cargo record are not counted as maritime commitments.

## Verification

The endpoint fixture tests overlapping sea lanes, inland exclusion and release of
commitments on cargo removal. The integrated vessel fixture checks that an idle
port hires at most 0.1 worker-months, 800 kg of existing cargo obtains 0.8 crew
worker-months before quarterly culture, total service work stays within the
8-adult ceiling, payroll markers survive enterprise preparation, and repeating the
late pass does not double-hire. Synthetic cargo isolates reservation behavior; it
is not a cargo-dispatch or archaeological inventory fixture.

The earlier scarcity experiment is rerun against the new bounded standby requests.
Its historical report describes `b92efbf`; the test command now runs the current
implementation. It still varies placement of standby within discretionary work,
with empty cargo in those arms. The separate committed-cargo fixture exercises the
new early pass.

## Comparison results

The 144 empty-cargo scarcity cases (seeds 17, 81, 256) passed. At eight healthy
adults without quarterly culture, all tested orders now fund 0.88 workshop and
0.10 crew worker-months. Previously crew-first funded 1.0 crew and only 0.28
workshop worker-months. At twenty healthy adults, idle crew work falls from 1.0 to
0.1 without reducing the 0.88 workshop or 0.5 quarterly cultural requests.

This does not eliminate discretionary conflict. At eight healthy adults with
quarterly culture, the current late-standby order still funds 0.50 culture,
0.78 enterprise and zero standby work. The new early pass protects existing cargo,
not unused fleet capacity. A subsequent action-request allocation is still needed
to choose among useful discretionary activities during extreme scarcity.

An initial integrated assertion expected culture to receive no more than 0.480001
worker-months after an 0.8 crew request. The measured crew grant was 0.7999946
because the existing f32 treasury debit rounds payroll; culture correctly received
0.4800055. The individual expectation now allows 1e-5 precision, while the test
retains its total reservation ceiling and money-conservation checks. No production
formula was changed to satisfy this expectation.

The ordinary library suite passed (71 tests; 67 ignored), the three enterprise GPU
fixtures passed including the 144-case comparison, and all three history-environment
fixtures passed (34.99 seconds excluding compilation, with concurrent compilation
on the host). Seed 17/81/256 batch/checkpoint comparisons remain exact on this
backend. These are regression and immediate allocation checks, not a long-run
shipping/economic balance study.

The corrected integrated vessel fixture passed in 2.31 seconds excluding
compilation. Strict Clippy, formatting and repository artifact checks passed.
Generated logs remain in ignored `output/`; only source and Markdown summaries
are committed.
