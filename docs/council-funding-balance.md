# Council funding under political household distribution

Follow-up to [the century comparison](distribution-policy-century.md): does
increased household assistance crowd out services that actually request council
money? A lower treasury balance alone does not answer that question.

## Measurement

`Society.council_funding` records cumulative requested and paid currency, positive
shortfall, positive requests and requests funded below 90%. Receipts are recorded
where transfers actually execute; they do not influence allocations, policies,
randomness or scheduling. They survive serialization. Old histories default to
zero totals, so importing one begins a new diagnostic baseline, not a reconstructed
spending history.

- **Administration:** monthly required payment and actual transfer to each town.
  The below-90% threshold matches the existing unpaid-month condition. Cumulative
  counts retain earlier failures even if the current unpaid streak resets.
- **Roads:** annual requests after passability, materials, construction capacity
  and the road-size cap, but before treasury availability. Shortfall measures
  feasible spending left unfulfilled, including small f32 withdrawal rounding;
  it does not count unavailable bricks as a shortage of money. Ineligible roads
  do not request funding.
- **Emergency town support:** the existing annual hunger-triggered request of
  ten abstract currency units per resident, compared with the actual transfer.
  This is a game-policy request ceiling, not a measured minimum rescue cost.

The runner additionally exports administrative state, road stock/passability and
individual petition outcomes with recorded failure reasons. Petitions distinguish
political opposition, unavailable institutions, changed control, unavailable
delivery channels and insufficient council funds. A pending petition is not a
failure. The stored reason uses the resolver's precedence: a politically rejected
petition does not establish that funds would have been sufficient.

Household aid still executes before monthly administration and petition resolution.
Annual tax collection, emergency town support and road spending follow those
monthly operations. These observations preserve that timing, including its
existing competition for cash. Institutional treasuries and town operating cash
are separate pools and are not silently counted as council reserves.

## Reproduction

Build `cargo build --example cultural_work_calibrate`, then repeat the two commands
in [the century setup](distribution-policy-century.md#setup), using
`output/council-funding-fixed.json` and `output/council-funding-political.json`.
The only configuration difference is `--fixed-distribution`. Compare food and
population with:

```sh
python3 scripts/compare_food_access.py output/council-funding-fixed.json \
  output/council-funding-political.json --allow-difference political_distribution
```

Each decadal sample contains `council_funding`, `administrations`, `road_state` and
`civic_petitions`, alongside the previous food, population and fiscal observations.
Subtract cumulative receipts between samples for interval results. The replay
also checks every previously recorded result against the uninstrumented runs,
excluding wall time and the newly added fields. Generated reports remain ignored
under `output/`.
