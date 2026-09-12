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

## Verification

- The shared-treasury GPU fixture checks requested payments, actual payments,
  cumulative shortfall and the count of underfunded administrations. Its
  save/reload continuation compares complete histories, including the new totals.
- The road GPU fixture gives a repair 100 kg of bricks, sufficient construction
  capacity and 40 currency. It requests 200, pays 40, records 160 shortfall and
  adds exactly 20 kg to the road. Removing the bricks instead produces no funding
  request, preventing material scarcity from being diagnosed as a cash shortage.
- All 134 ordinary library tests pass (114 hardware/long tests remain ignored in
  that command). Both focused GPU fixtures above pass explicitly. The full frozen
  scheduler fixture passes monthly/batched/checkpoint comparisons on seeds 17, 81
  and 256. Clippy passes across all targets.

## Completed century replay

Instrumentation source: `f56b79e`. Both reports are complete. Seeds 256 and 409
run for 100 years at terrain/ecology 32/16, yield scale 0.5, sixteen founders,
individual demography and all four production refinements, with family support.
The strict comparison acknowledges only political distribution as a configuration
difference. Every existing per-seed field and decadal observation matches the
previous report exactly, except wall time; the four new diagnostic fields are
additional evidence. Population outcomes therefore remain 1,338 → 2,511 and
1,322 → 2,647, with zero physical food gap and improved purchasing access.

### Administration

Currency shortfall is `(requested - paid) / requested`, summed at each actual
payment. It is not the fraction of people unpaid. Each arm observes 19,200
administrative site-months. Requests are recurring service bills, not accumulated
arrears that the simulation promises to repay later.

| Seed | Century requested, fixed → political | Century paid | Century funding gap | Years 90–100 funding gap | Site-months below 90% funding |
| --- | ---: | ---: | ---: | ---: | ---: |
| 256 | 17,169 → 23,093 | 15,723 → 21,493 | 8.42 → 6.93% | 7.09 → 15.41% | 1,536 → 1,372 |
| 409 | 17,371 → 23,845 | 15,159 → 21,991 | 12.74 → 7.78% | 5.39 → 2.59% | 2,454 → 1,775 |

The century-wide funding fraction improves in both seeds despite smaller ending
council balances and increased administrative demand. The late result is mixed:
seed 256 has 262 underfunded site-months in the last decade versus 120 in its
control; seed 409 improves from 120 to 74. Seed 256's treatment ends with an
84-month underpayment streak in site 2, whose administrative loyalty is zero and
unrest is one. The longest control streak is 24 months. In seed 409 the longest
ending streak is only one month under treatment and zero under control.

These are real local constraints, not proof of general fiscal collapse. Nor does
this experiment isolate relief's immediate crowding-out effect: population,
administrative demand, politics and revenue collection also diverge. Tax income
depends on office capacity and local cash, so a zero treasury need not mean that
household relief alone consumed an otherwise adequate budget.

### Roads and emergency support

| Seed | Road requested currency, fixed → political | Road paid | Remaining road bricks, kg | Emergency town-support requests | Emergency support paid |
| --- | ---: | ---: | ---: | ---: | ---: |
| 256 | 6,648 → 7,936 | 973 → 1,724 | 54.9 → 249.8 | 104 → 39 | 38,324 → 4,172 |
| 409 | 16,882 → 18,901 | 5,844 → 5,061 | 451.0 → 569.9 | 129 → 11 | 30,963 → 679 |

Road funding remains imperfect in both arms. Political distribution increases
lifetime road payments in seed 256 and reduces them in seed 409; remaining road
material increases in both. Timing and weathering matter, so lifetime expenditure
alone does not establish current road quality. All 18 and 16 respective routes
remain passable in both arms; passability does not imply a well-maintained road.

The last decade's road shortfall is approximately 54.63 currency in political
seed 256. The other three arms each have less than 0.002 currency shortfall over
that decade. Tiny positive material requests can round to zero withdrawal from
f32 inventories: consequently raw below-90% road-request counts can look alarming
without meaningful unmet expenditure. Use amounts and available road stock, not
those counts alone. Recurring repair opportunities are not distinct promised
projects or an accumulated debt.

Emergency town-support triggers decline from 104 to 39 and from 129 to 11.
Political councils fund only about 6.1% and 5.2% of these request ceilings, versus
36.6% and 27.1% in the controls. This channel therefore has substantially less
coverage, even though fewer requests occur. Because its requested amount is a
policy ceiling rather than a costed rescue plan, neither the decline in spending
nor the unpaid ceiling proves an essential service was withheld. Seed 256 still
has late hunger-triggered requests; improved global affordability does not remove
all local hardship.

### Petitions

No petition in these runs closes with insufficient council funds as its recorded
reason. Seed 256 has two unavailable-institution closures under fixed policies
and three under political policies. Seed 409 has five unavailable-institution
closures, four political rejections and one delivered petition in the control;
all five treatment closures concern unavailable institutions. There are no pending
petitions at the final samples. This does not establish funding adequacy for
politically rejected or otherwise ineligible requests.

## Decision

Keep the political distribution defaults. Smaller reserves did not erase the
population gains or produce uniformly worse public funding. They also did not
solve local administrative shortfalls, and one treatment has a worse late funding
trajectory. Do not tune toward a treasury balance for its own sake.

The next bounded comparison should investigate the long unpaid streaks, including
tax collection and office capacity, and test an explicit basic-administration
allowance alongside household assistance against the same available council cash.
This should preserve scarce-budget accounting and measure both household food
access and completed public payments. A blanket reserve floor could simply move
hardship back to households. No funding priority or policy was changed in this
measurement pass. War, major expedition expenditure, larger grids and additional
seeds remain untested by this particular comparison.
