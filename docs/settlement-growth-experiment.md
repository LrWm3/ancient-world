# Settlement growth ladder

`examples/growth_ladder.rs` prepares matched experiments to ask whether modestly
more favorable conditions reach the settlement limit. This is a game-balance
experiment, not a prediction of historical population growth. Long runs are
explicitly separate from the short operational checks below.

## Treatments

All arms of a seed load the same founded world, candidate survey, initial people,
and supplies. Default structural systems and living ecology are enabled. Complete
individual demographic resolution is used unless `--aggregate` is supplied.
Experimental policies outside this table retain their registered defaults.

| Mode | Change from baseline |
| --- | --- |
| baseline | Existing settings |
| health | Background mortality ×0.85; hunger, disease and combat losses are not scaled |
| access | Municipal food relief and needs-first municipal welfare reserves enabled; requires actual cash and available food |
| yield | Crop yield scale ×1.15, after the common candidate survey |
| expansion | Daughter-founding population thresholds ×0.85; provision requirement and transferred supplies reduced from 12 to 9 months |
| combined | All four changes |

No change raises starting food or storage, creates free relief, or changes ruin
resettlement admission. The expansion arm trades a lower departure barrier for a
smaller safety reserve; it need not improve survival. Health leaves births alone.
Food access and food production are separate arms so redistribution does not get
mistaken for a larger harvest.

The runner records the first failed annual daughter-founding gate for each origin:
population, provisions, reachability, occupied candidates, current terrain safety,
settlement cap, or whole-household admission. These are sequential admission
outcomes, not an exhaustive list of every simultaneous constraint. Settlement
records include abandoned sites; active towns and resettlement counts are reported
separately. Population includes travelers so departures do not look like deaths.

## Stage gates

Default checkpoints are at **100, 200, 500 and 1,000 years**. At each gate:

1. Save a complete restartable world.
2. Examine annual population observations over the trailing 20 years.
3. Stop that seed/mode if its least-squares slope is negative **and** its endpoint
   population loss exceeds 1%. A flat or noisy trajectory continues. Extinction
   also stops continuation.
4. Continue other arms independently. Simulation errors are reported as errors,
   never as a biological decline.

The rule detects recent decline, including a run that never grew. It does not
claim a later recovery is impossible. `--continue-on-decline` overrides decline
stops, including when resuming a previously stopped arm. `--trend-years` and
`--reversal-fraction` make the rule explicit and adjustable before a suite starts.
Only gate checkpoints are restart points: interruption between gates replays from
the preceding gate and discards uncheckpointed annual observations.

## Long-run commands

From the repository root, build once:

```sh
CARGO_INCREMENTAL=0 cargo build --release --example growth_ladder
```

Run the default 18-arm suite (three seeds × six modes):

```sh
target/release/examples/growth_ladder --output output/growth-long
```

Defaults are seeds `17,81,256`, terrain/ecology resolution `256`, one geological
epoch, 16 initial civilizations, and settlement capacity 2,560. The shared initial
world is generated once per seed. This is a potentially expensive experiment;
there is no completion-time promise.

Restart with precisely the same settings:

```sh
target/release/examples/growth_ladder --output output/growth-long --resume
```

Continue stopped declines intentionally:

```sh
target/release/examples/growth_ladder --output output/growth-long --resume --continue-on-decline
```

To begin with one seed and the combined arm plus its control:

```sh
target/release/examples/growth_ladder --seeds 17 --modes baseline,combined --output output/growth-pilot
```

Use a new output directory for different settings. `--resume` checks the suite
settings, checkpoint month and uninterrupted annual evidence. Use the same binary
and catalogs for continuation; changing the model mid-run is not a matched
experiment. Complete checkpoints contain configuration and catalogs.

## Reading the results and testing caps

Each seed/mode directory contains `annual.jsonl`, `progress.json`, `summary.json`,
`effective-settings.json`, and gate `.world` archives. A failed arm also has
`error.json`. Root metadata records settings, source revision and GPU. Annual data
includes births/deaths, total living population, active/abandoned records,
remaining survey candidates, food production/consumption, household food gaps,
latest-month municipal relief, council cash, event/person counts and conservation
residuals. Latest-month relief is a snapshot, not an annual transfer total.

Summaries record first passage through 25%, 50%, 75%, 90% and 100% of the configured
settlement cap. Check admission reasons alongside these milestones: population
stagnation is not evidence of a settlement cap unless plausible new founding is
actually blocked. Candidate exhaustion, distance and household eligibility can
bind before the record limit.

`--settlement-cap` is an experimental **soft cap**, between the initial civilization
count and 2,560. It does not change the GPU allocation or candidate survey ceiling.
For a controlled lower-cap comparison, run identical suites with, for example,
`--settlement-cap 128` and `--settlement-cap 2560` in different directories. Existing
ruins can still be reoccupied at the cap. Raising the real allocation above 2,560
is a separate implementation if these experiments demonstrate it is necessary.
The 500,000 named-person record ceiling and one-million-event ceiling can also
interrupt a growing long run. An error at those limits is computational capacity
pressure, not a demographic stall; retain its preceding observations.

After screening at 256, repeat promising arms with `--resolution 1024
--ecology-resolution 256` in a new directory. This changes the candidate geography,
so compare trajectories and blockers rather than requiring identical histories.

Outputs can be large, especially full-resolution gate archives. They belong under
ignored `output/`; commit only Markdown summaries. Keep all failed arms in the
analysis, and inspect conservation failures before interpreting balance results.

## Short verification

The shortened equivalent exercises all six modes and all four gates without
starting the century runs:

```sh
CARGO_INCREMENTAL=0 cargo build --example growth_ladder
target/debug/examples/growth_ladder --seeds 17 --resolution 32 --ecology-resolution 16 --civilizations 5 --gates 1,2,3,4 --trend-years 2 --continue-on-decline --verify-resume --output output/growth-smoke
```

`--verify-resume` compares exact serialized history after one month of both
uninterrupted and checkpoint-loaded continuation at each arm's first gate, then
reloads the gate before proceeding. The test month does not count twice.

### Results, 2026-09-14

Quadro RTX 5000 with Max-Q Design, development profile, seed 17, terrain 32 /
ecology 16, five founding civilizations, one geological epoch. All arms started
with 600 people. Every arm completed all four shortened gates and passed exact
serialized-history continuation after a checkpoint and one additional month.

| Mode | Year-four population | Births / deaths | Cumulative food production, kg | Final-month household food gap, kg | Final-month municipal relief paid |
| --- | ---: | ---: | ---: | ---: | ---: |
| baseline | 646 | 74 / 28 | 196,490 | 235.07 | 0 |
| health | 650 | 75 / 25 | 199,430 | 234.85 | 0 |
| access | 648 | 74 / 26 | 197,346 | 0 | 323.43 |
| yield | 646 | 74 / 28 | 196,739 | 235.02 | 0 |
| expansion | 646 | 74 / 28 | 196,490 | 235.07 | 0 |
| combined | 653 | 75 / 22 | 200,908 | 0 | 328.15 |

All arms retained five sites and reported population as the founding blocker.
Expansion therefore matching baseline is expected, not evidence of a broken
switch. A separate GPU fixture demonstrated provision and population threshold
changes admitting an otherwise blocked founding, and a soft cap refusing it while
preserving the candidate count.

Maximum absolute normalized population residual was zero; food residual was
7.16e-8; economy residual was 7.86e-7 across annual observations. Additional
aggregate-demography baseline/health checks completed two years with exact
checkpoint continuation: populations 622.23 / 624.05, respectively. The maximum
normalized economy residual in that pair was 6.93e-7.

Regular verification: 221 library tests, 19 market tests, three runner tests and
one explicitly enabled GPU admission fixture passed. Gate fixtures cover recent
versus whole-run trends, noise, stalls, decline stops, the continuation override,
and extinction. Completed-suite resume also succeeded without adding annual rows. Resetting the
baseline progress marker to the real year-two checkpoint and replaying years three
and four reproduced the complete annual file byte-for-byte, despite the existing
uncheckpointed tail. All test and example targets passed `cargo check`.

Four-year, coarse-grid runs validate operation and mode separation; they cannot
establish century viability or settlement-cap pressure. The small increase in food
from the yield treatment shows that a 15% potential increase does not imply 15%
more realized food under the other production limits. Relief altered actual food
access without directly increasing the yield parameter. Biological and economic
feedbacks can subsequently change total production in the access and health arms.
No century or millennium run was launched as part of this preparation.

## Default long-run results, 2026-09-14

The documented release command was run unchanged on a Quadro RTX 5000 with
Max-Q Design. It used three seeds, six modes, 256 terrain/ecology resolution,
16 founding civilizations, and individual demographic resolution. Wall time was
about 94 minutes; completed gate stages accounted for 4,936 seconds. The output
directory is approximately 9.1 GB. The process exited nonzero because two arms
hit the same explicit individual-roster consistency error; the runner continued
all other arms before reporting the failures.

Every valid arm reached the first gate and stopped for recent population decline.
No arm reached years 200, 500, or 1,000. Population and recent slope at the first
gate were:

| Seed | Baseline | Health | Access | Yield | Expansion | Combined |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 17 | 988 (−8.29/y) | 1,076 (−5.71/y) | 1,437 (−4.13/y) | 843 (−9.21/y) | 988 (−8.29/y) | 1,548 (−2.58/y) |
| 81 | error, year 92 | 795 (−6.75/y) | 964 (−1.09/y) | 738 (−9.61/y) | error, year 92 | 1,006 (−0.92/y) |
| 256 | 831 (−6.56/y) | 913 (−4.23/y) | 1,033 (−0.62/y) | 833 (−5.96/y) | 831 (−6.56/y) | 1,041 (−0.65/y) |

The access treatment consistently retained more population and had the shallowest
decline. Combined also improved retention, but still reversed by the first gate.
Yield did not reliably improve the final population. Expansion produced the same
trajectory as baseline because the populations never reached the daughter-founding
threshold; its relaxed settings were not the active constraint. Settlement records
remained at 15 or 16, while candidate surveys retained many unused locations, so
these runs were demographic tests rather than settlement-cap tests.

The two failed arms were `seed-81/baseline` and `seed-81/expansion`, both at month
1,110 (year 92). Site 1 had aggregate age cohorts `[4, 11, 3]`, but its prior-age
resident roster required `[4, 14, 3]`. The error occurred after repeated
`household_represented` and `inheritance` events. This is the known unfinished
individual-demography boundary: named residents, household membership and
fractional/aggregate aging have diverged. It is an implementation blocker for
trusting long individual-resolution runs, not evidence of population decline.

The largest economy residual reported by completed arms was about `2.96e-5`, with
food and population residuals remaining finite. Before using the 200-year gates,
repair resident-roster reconciliation at this boundary, then rerun the same suite
in a new output directory. The existing archives and `error.json` files preserve
the failure evidence; generated outputs remain under ignored `output/`.
