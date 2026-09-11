# Civic petition balance sweep

This is a game-balance and regression exercise, not historical calibration.
The comparison starts from parent revision `7c2b516`, with the petition switch,
response diagnostics and vessel rounding fix described below.

## Reproduction and scope

```sh
CARGO_INCREMENTAL=0 cargo run --example civic_balance -- \
  --output output/civic-balance-fixed.json
python3 scripts/analyze_civic_balance.py output/civic-balance-fixed.json
target/debug/examples/civic_balance --seeds 17,81 --crop-yield-scale 0.33 \
  --output output/civic-balance-harsh.json
python3 scripts/analyze_civic_balance.py output/civic-balance-harsh.json
```

Each seed runs twice, with new petition proposals disabled and enabled.
Both variants use the crew fix. Five normal-yield seeds are 17, 81, 256, 409
and 1024, each for 100 years; the lower-yield comparison uses 17 and 81.
Terrain resolution is 64 per face, ecology 16, with one geological epoch and
16 founding civilizations. History advances twelve monthly steps between
annual observations. The harness enables society, politics, governance,
offices and shipping, with their prerequisite systems and bundled catalogs.
It deliberately freezes environmental evolution during history. This is
not an all-options-on test of living ecology, expeditions and resource feedback.

Normal crop yield scale is 0.5; the pressure case uses 0.33. Other configuration
values retain bundled defaults. Hardware: NVIDIA Quadro RTX 5000 Max-Q,
Vulkan, Rust 1.89.0, optimized development profile. Raw observations and logs
stay in ignored `output/`; the example and analysis script regenerate them.

Population differences after a century are downstream associations in a
nonlinear simulation. They do not prove that a grant alone caused the change.
The immediate measured quantities are requests, delivered money, response
delays and resolution reasons. Residuals are the maximum absolute normalized
economy residual across six existing ledger checks at annual observations,
not a mass quantity or a check of every possible subsystem.

## Failure found and corrected

The initial sweep stopped on seed 409 with petitions enabled because fleet
validation rejected a crew-work value. Cash is held as f32 while a household
receives the exact f64 debit. Rounding can make that debit slightly greater
than the quoted wage. Converting all payment back into labor could exceed
the vessel's 0.25 worker-month reservation.

Crew work is now bounded by the actual reservation as well as payment.
The exact debit still reaches the household, preserving money; a rounding
premium cannot create extra labor or freight capacity. A regression uses a
large treasury and a low wage to reproduce the overpayment and verify the
work cap, zero-payment case, partial funding and conserved money.
The fleet validity tolerance was not relaxed.

## Results

The following endpoints compare petitions disabled → enabled. Requests and
grants are for the enabled run; money is in abstract currency units.

| Yield | Seed | Requests relief/learning/autonomy | Honored | Paid | Population off → on | Active towns off → on |
|---|---|---|---:|---:|---|---|
| 0.5 | 17 | 1/77/6 | 43 | 256.9 | 2628 → 2692 | 19 → 19 |
| 0.5 | 81 | 1/42/28 | 36 | 53.5 | 2183 → 2123 | 21 → 21 |
| 0.5 | 256 | 8/154/17 | 77 | 449.4 | 2667 → 2347 | 16 → 16 |
| 0.5 | 409 | 3/98/34 | 81 | 367.5 | 2084 → 2175 | 19 → 20 |
| 0.5 | 1024 | 5/66/3 | 21 | 111.4 | 2724 → 2594 | 23 → 25 |
| 0.33000001311302185 | 17 | 12/110/10 | 56 | 274.0 | 1237 → 1275 | 13 → 13 |
| 0.33000001311302185 | 81 | 1/60/20 | 36 | 97.8 | 1249 → 1168 | 12 → 14 |

All fourteen runs finished after the crew fix. No new proposals appeared in
the disabled controls. Every resolved request took 3–12 months, and no
payment exceeded its request. The maximum normalized annual ledger residual
was 2.47e-5 across the runs. Peak cumulative grant spending relative to
cumulative administrative payroll stayed between 1.37% and 5.53% in enabled
runs; this is a scale comparison, not the council's full budget share.

At normal yield, 543 requests produced 258 deliveries. Relief accounted for
18 requests (11 delivered), learning 437 (161 delivered), and autonomy 88
(86 delivered). The very high autonomy success rate deserves further
attention: the current model has few bargaining costs when a willing
government is present. Learning requests were most often refused for
political opposition (233 cases); 36 lost their institutional sponsor and
seven lacked council funds.

The lower-yield seed 17 increased relief requests from one to twelve.
Seed 81 still generated only one relief petition. Hardship alone is not
sufficient: local representation, an operational sponsor and the shared
cultural labor budget must also be available. Both lower-yield pairs retained
far fewer residents and active towns than their normal-yield counterparts;
petitions did not eliminate that pressure.

Normal-yield population effects ranged from −11.99% to +4.38%. Seed 256
lost about 320 people relative to its control, with divergence accumulating
over decades rather than an immediate disappearance. Faction-shift events
totaled 163 without petitions and 156 with them; war declarations totaled
78 versus 99. Lower-yield population differences were +3.06% and −6.45%.
These are meaningful trajectory changes, not a uniform welfare improvement.

Decision: retain existing petition costs, cooldown and political-credit
weights for this pass. The evidence supports bounded spending and responsive
requests, but not an optimal political balance. The next targeted experiment
should separate material grants, autonomy changes and faction credit in
matched short-horizon branches before reducing any one effect to counter
the observed war increase. Learning remains the dominant agenda; do not
force equal demand counts irrespective of institutions and local need.


## Verification and interpretation

The ordinary library run passed 60 tests, with 58 hardware tests left
unselected. The selected GPU petition fixture passed on its two seeds,
including disabled proposals consuming no labor, pending responses finishing
after disable, archived continuation, old archive defaults, finite transfers
and causal event links. The selected GPU crew-pay fixture also passed.

The common system selector now permits `--disable-system civic-petitions`
or explicit `--enable-system civic-petitions`. This controls future proposals;
existing requests still resolve. Response records now state the reason for
closure. Multiple impediments can coexist: the reported reason is the first
applicable one in the implementation order, not an exhaustive
diagnosis or a count of every month spent waiting.

These small worlds and annual summaries cannot establish balance at larger
resolutions, in long histories, or with every optional feedback enabled.
A town-count endpoint can hide abandonment and reoccupation between samples.
Small cash grants are not a substitute for grain delivery, and an honored
learning request does not prove improved education. No petition threshold,
grant amount or political-credit weight was changed solely to produce a
desired success rate.
