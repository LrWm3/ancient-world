# Planner comparison manifest

Frozen before implementing the composition searches. Historical control: `f592fbd`.
This is P0 of [the experiment plan](PLANNER-EXPERIMENTS.md), not a new v1 gate.

## Controls

Use `membership::scenario()` unchanged except a requested forecast horizon of 12
and a 24-month observed run. B2 removes either all access offers or opening seed;
the scarcity control removes both and the environmental wood supply/regeneration.
Keep condition consequences, annual two-grain dues and existing production terms.
B3 uses two plots, one person, two seeds, three monthly hours, and a second crop
whose final month needs three hours: two six-month crops require less than 18
hours in total but cannot both harvest in the same month. All other terms match B1.

B5 reuses the two-member/two-plot fixture in `tests/household_offers.rs`: five
hours/member, contributed governance, household-held two seeds and ten grain,
40 storage/member. The shared seed control reduces that stock to one. Existing
household charter and consent remain authoritative. The legacy consequence
search is unsupported here; the current household driver is the control.

B6 uses `competition::scenario(1, seed)`. Plan for each person independently,
collect applications against the same opening, then use the existing allocator.
Compare StablePriority and PriorityLottery separately; lottery seeds 0–9 are
controls and 10–19 are held out. No private rival plans enter either search.

B4 uses `production_market::reciprocal_scenario(true)` and its existing driver;
the no-buyer control removes market access for agents 91/92 at opening. This is
a four-person finite-counterparty diagnostic, an explicit exception to the
proposed single-person fixture. The current common-offer adapter does **not**
expose town bids/asks as executable requests. Record that as unsupported for the
composition searches unless an adapter is actually implemented and tested; do
not report the existing driver's decisions as new-search results. B2's wood/buy
alternative has the same dependency on this missing adapter.

## Search and comparison

Both new strategies compose existing membership, land and productive process
requests, derived from backward resource relevance. One offered process lot per
actor/definition/boundary; one acceptance per actor/offer; no integer quantity
enumeration. All prerequisites precede work. The maximum package has six requests.
The current month's package is followed by the same ordinary ContinuingFirst
rollout. No recursive search occurs within a forecast. Requests go through
`offers::prepare`; hypothetical resources never become live resources.

Beam width is eight. Best-first ranks partial packages by relevant outputs,
prerequisites, dated capacity pressure and time to output. These are optimistic
ranking hints, not permissions or welfare scores. Both evaluate complete feasible
packages with the existing consequence score. Waiting/continuing is always a
candidate. Existing obligations and process failures remain in the rollout.
Do not reject a lawful fallback merely because needs cannot all be satisfied.

Budgets: 64/8, 256/32, 1024/128 attempted child expansions/full forecasts. Count
failed expansions too. Forecast horizons: 12 initially, 6 and 24 sensitivity.
Use identical horizons for the two new strategies; report legacy evaluator's
minimum contract horizon separately. Replan at Acquire only, preserving the
existing monthly scheduler. Runtime is measured, never a decision input.

The legacy search is an economic/control-policy comparison, **not** an identical
action-set oracle: it chooses continuation priorities and can generate automatic
work, whereas the new searches accept explicit lots. Report this distinction.
Use tiny exhaustive package enumeration as the within-action-set reference in
tests. It is an offline reference, never a runtime allocation optimizer.

Record food/warmth deficits, completed/failed crops, dues arrears, ending stocks,
selected requests, candidate/forecast counts, forecast-months, peak frontier,
exhaustion, rejections and elapsed time. Compare CPU/reference, independent input
reordering, stale rejection, resource scarcity, and continuation across acceptance.
For household runs reconcile the existing separate financial statements.

Commands and actual results will be published in `PLANNER-COMPARISON.md`. Raw
stdout belongs under ignored `output/economics/`. Negative and unsupported results
are part of the comparison. Keep the production default unchanged.
