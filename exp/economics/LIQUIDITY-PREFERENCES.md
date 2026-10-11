# Independent liquidity preferences

This follow-up to [startup stocks and liquidity](STARTUP-LIQUIDITY-COMPARISON.md)
separates workshop and town preferences. It changes the initial coin-reserve
objective from terminal shortage to accumulated monthly shortage independently
for each organization. Targets, priority order, candidate catalogs, prices, rights,
labor terms, money and stocks otherwise remain identical. These are founding
variants, not autonomous amendments of a sealed constitution. Terminal scoring
remains the default.

## This ten-iteration batch

1. Trim historical diagnostics from private forecast copies.
2. Verify behavioral equivalence with pending commitments and accepted future work.
3. Measure isolated diagnostic-copy cost from aged worlds.
4. Separate workshop and town founding liquidity preferences.
5. Verify configuration isolation, personal preferences and candidate equivalence.
6. Expose existing decision/forward receipts in a reproducible diagnostic command.
7. Extend CPU, reconstruction, replay and catalog-order controls to exposure and mixed preferences.
8. Compare the four metal-only preference combinations over 48 months.
9. Compare the four both-stock combinations and annual readiness windows.
10. Record measured results, limitations and remaining questions in the documentation.

Independent review checked the implementation, controls and interpretation as the
batch progressed. No scheduler, allocation priority or settlement rule changed.

## Comparison boundary

Each case runs 48 months through ordinary Reference execution and accounting.
Town opening ore is zero. Workshop opening metal is four; workshop opening ore is
four in the both-stock case and zero in the metal-only case. The two by two
comparison includes both symmetric preferences and both mixed preferences for
each stock configuration. Readiness still requires three complete operating
years after startup; signing and delivering one initial forward is insufficient.

The initial-state controls also cover both forward-authority settings and all
four stock ablations. They check exact conversion counts, unchanged whole World
and State after reversing just the intended objective change, and unchanged
work/quote catalogs and sale opportunities. Explicit personal-governor preferences
have a separate control because these acceptance fixtures use organizational
fallback goals. That control also preserves non-coin reserves and checks
idempotence.

## Measured outcomes

`T` means terminal reserve; `E` means monthly reserve exposure. Production counts
are completed batches; food deficit is cumulative, wage arrears are outstanding
at the ending boundary. All rows reconcile the accounting records.

| Workshop stocks | Workshop / town preference | Mints | Workshop | Town | Food deficit | Wage arrears | Signed / activated | Ready |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Metal only | T / T | 1 | 0 | 4 | 61 | 2 | 2 / 4 | No |
| Metal only | T / E | 1 | 0 | 4 | 61 | 2 | 2 / 4 | No |
| Metal only | E / T | 41 | 40 | 42 | 0 | 0 | 5 / 7 | Yes |
| Metal only | E / E | 41 | 40 | 42 | 0 | 0 | 5 / 7 | Yes |
| Ore and metal | T / T | 42 | 41 | 40 | 0 | 0 | 7 / 9 | Yes |
| Ore and metal | T / E | 42 | 41 | 40 | 0 | 0 | 7 / 9 | Yes |
| Ore and metal | E / T | 41 | 40 | 40 | 0 | 0 | 7 / 9 | No |
| Ore and metal | E / E | 41 | 40 | 40 | 0 | 0 | 7 / 9 | No |

Changing only the town's preference changes neither these outcomes nor the three
annual production summaries. Changing only the workshop reproduces both the
metal-only improvement and the both-stock regression. The general default stays
Terminal: monthly cash preservation is not uniformly beneficial, even in these
two closely related economies.

## Where the metal-only plans diverge

Holding town preferences Terminal, both workshop variants publish the same
sale-only program in month 1 and retain policy in month 2. Their first action
divergence is forward consent in month 2. Both predict admission and two units
delivered; their projected coin paths for this candidate are identical.

| Workshop preference | Workshop baseline → proposed loss | Town baseline → proposed loss | Consent |
| --- | --- | --- | --- |
| Terminal | 1000 → 1000 | 995 → 993 | Accept |
| Exposure | 11949 → 11973 | 995 → 993 | Refuse |

Smaller loss is preferred. The terminal forecast exhausts workshop cash in both
branches, so it cannot distinguish the earlier depletion caused by prepayment.
The monthly objective detects it. Baseline workshop cash at completed months
2–13 is `6,9,8,7,6,5,4,3,2,1,0,0`; with the proposed forward it is
`6,7,6,5,2,1,0,0,0,0,0,0`.

Actual balances first diverge at Close 3: workshop/town coins are `7/9` after
Terminal consent versus `9/7` after Exposure refusal. The Terminal agreement
signs in month 2, schedules admission for 3, activates in 4 and delivers its two
units by the end of the run with due month 5. This does not produce recurring
workshop output or adequate household nutrition.

The exposure workshop refuses all candidates in months 2–4. In month 5, the
first candidate (buyer program 0) still worsens its loss, `11934 → 11965`, while
a second (buyer program 2) improves it to `11884`. Both include work and an ore
bid; program 2 also includes the metal ask. The town also improves, `998 → 996`. That agreement signs in month
5, schedules admission for 6, activates in 7 and has due month 8. Its proposed
workshop cash path first falls to six coins, then grows to fourteen over the
forecast. The actual run sustains production and passes readiness.

This is evidence for the effect of the workshop's preference in this fixture.
It is not proof that delaying the same payment alone repairs the economy: the
selected productive program and later ordinary choices can also change. The
diagnostic command therefore prints program identities as well as contract terms.

## Where the both-stock plans diverge

All four variants sign in month 7, schedule admission for 8, activate in 9 and
have delivery due in 10. Cash balances agree through Close 12. The first ordinary
workshop decision divergence occurs in month 12, effective in month 13.

The Terminal workshop retains its current program. Retention and program 1 both
project terminal loss `993` (seven ending coins). The Exposure workshop instead
chooses program 1: projected exposure falls from `11910` to `11870`. That program
retains the work target and metal ask but drops the ore bid. It projects two
metal sales instead of four, ends with the same seven coins, and holds fewer
inputs and outputs. Exposure values the earlier cash more highly than this
additional inventory and sales. The next month it restores the combined ask/bid
program, effective in month 14; removal/restoration repeats at decisions 16/17.

The failed readiness window is months 13–24: eleven workshop batches and eleven
metal sales instead of twelve, while the town still completes twelve batches.
All twelve workshop and town labor hours are paid in that window. The later
windows, 25–36 and 37–48, each have twelve batches and twelve metal sales. The
sole readiness failure is `recurring production or market demand absent`; food
and wages pass. This distinguishes an ordinary operating-policy regression from
failed startup consent or insolvency.

A useful next experiment is to represent continuity of necessary inputs or
productive operation in the organization's signed objectives, then compare it
against these unchanged stock and cash controls. Do not silently equate a
cash-only objective with sustainable production, or change the gates to make a
preferred objective pass. No such new objective is implemented in this batch.

## Private forecast history

`ForecastContext` now removes old alternative projection payloads and forward
review diagnostics from its private World after cloning. It retains decision
rows, accepted command snapshots, dates, scalar losses, catalogs, ballots,
mandates and pending startup agreements. Live diagnostic history is unchanged.
This reduces repeated copying by phase adapters inside private rollouts. The
initial full clone still occurs; this is not a zero-copy forecast implementation.

Controls compare compact copies with copies rehydrated from the original live
history. They verify phase-by-phase State, ledger and reports, completed-month
paths, peer observations and normalized contexts. A pending forward actually
activates and delivers; an already delivered agreement stays delivered; an
accepted non-mint future process remains scheduled and completes. Peer evidence
is explicitly nonempty in the aged fixture. These controls exercise behavior,
not only the size of the copied tables.

A release-mode cost comparison ages each both-stock fixture for twelve months,
then measures two paired samples of frozen twelve-month Reference rollouts per
preference (one compact and one detailed rollout per sample). The
timed region is simulation construction plus execution. Setup, aging, rehydration
and equality checks are excluded; execution order alternates. Terminal scoring
took approximately 45 ms with compact history versus 58–62 ms with diagnostic
history restored. Monthly exposure took 46–47 ms versus 205–224 ms. State, ledger,
reports and normalized contexts agree. The example does not compare Path objects;
those are covered by the equivalence tests.

These are two local samples, not a whole-planner throughput claim. Both branches
use the current constructor for nested forecasts. Separately reported snapshot
construction time includes the initial clone and trim, with no old-constructor
baseline. Retained live history still grows, and repeated rejected proposals can
still require substantial work.

## Reproduce

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example liquidity_preferences -- metal terminal exposure
cargo +1.92.0 run --locked --release --example liquidity_preferences -- both exposure terminal
cargo +1.92.0 run --locked --release --example forecast_cost
cargo +1.92.0 test --locked --release --test financing_preferences --test financing_factorial --test forecast_diagnostics --test forecast_equivalence
cargo +1.92.0 test --locked --release --test financing_matrix --test forward_continuation
```

The first preference argument belongs to the workshop, the second to the town.
The diagnostic example prints actual agreements and balances separately from
projected admissions, deliveries, losses and coin paths. It includes ordinary
choice summaries throughout the run and detailed alternatives for the first
twelve months. Successful execution is not an assertion that readiness passes.
Generated logs belong under ignored `output/economics/`.

## Limits

This comparison does not change peer-response models or planning horizons. It
cannot establish a generally best liquidity objective. Constitutions, prices,
initial employment and consent machinery remain supplied. No-metal startup,
general finance discovery and robust planning across arbitrary counterparties
remain open. Neither passing accounting nor passing this fixture proves those
broader ambitions complete.

## Verification boundary

The focused checks pass 61 distinct tests across 14 suites, including all 26
library tests. This is a selected regression set, not the entire crate. It
includes the unchanged eight-cell stock matrix, eight independent-preference
cells, diagnostic isolation and equivalence, configuration controls, peer
expectations, forecast needs and forward assessment/admission/governance.
Economically failing cells are pinned expected results, not ignored test failures.

Four 48-month continuation cases cover both-stock Terminal success, metal-only
Exposure success, metal-only mixed-preference failure and both-stock Exposure's
production shortfall. CPU execution reconstructs Simulation after every phase;
ending World, State, complete ledger, reports and accounting audit match Reference.
Replay compares State at every phase and the final audit using the reference
World at each batch. Reordered independent catalogs preserve State, ledger and
audit. These are in-memory reconstruction checks, not serialized checkpoints or
World reconstruction from a bare ledger.

The diagnostic example was exercised for all eight combinations; a final run
also checks its explicit proposal-program and scheduled-admission labels.
Formatting, all-target release Clippy with warnings denied and repository
artifact checks pass. Raw output stays ignored.
