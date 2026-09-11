# Monthly history schedule

The schedule is a game rule, not a claim about historical time. All stages execute
once per simulated month, independent of API batch size. No additional planetary
readbacks or whole-world observation copies are introduced.

## Audit and port

Previously, relief arrived before consumption but market cargo arrived afterward.
Preparation, resource registration and culture synchronization also ran once after
the API's monthly loop. Living ecology ran outside the history coordinator, with
returns applied after the timeline had already been recorded.

The port makes five executable stages in `civilization.rs`. Observe and decide
are explicit substeps of opening/reservation rather than a generic scheduler.
Reservations remain subsystem-specific; this is not a simultaneous global auction.

| Stage | Operations and visibility |
| --- | --- |
| Open | Prepare society/politics/governance; increment month; inspect environmental disruption; settle due market cargo, relief and relocating households; answer appeals; restore sites; prepare economy and claims; refresh extraction inputs; patron assistance and legacy cohort cleanup. Arrival price observations are returned to the response stage. |
| Reserve | Prepare discoveries, reserve cultural work, prepare fisheries, allocate extraction allowances, plan production, prepare enterprises and vessels, prepare household retail. The extraction and retail plans are explicitly passed to execution. Earlier reservations have priority over later claims. |
| Execute/settle | Upload committed inputs; claim, fish and run GPU production/consumption; read town results; settle extraction and enterprises; storage, housing and waterworks; settle retail. No production policy chosen later in the month can retroactively change this dispatch. |
| Respond | Quote markets and dispatch new cargo using opening delivery evidence; release vessel/cultural work; expeditions, relocation, site lifecycle, society, genealogy, culture, offices and governance. Annual politics/shipping/expeditions/governance run here when due. Events remain available to later consumers in this stage. |
| Close | Synchronize society, politics, governance, economy, resource claims, culture and offices; refresh social indicators; validate. Frozen history records its timeline here. Living history then commits environmental returns, reconciles land, records its timeline and validates the coupled boundary. |

## Observation and response rules

- Both market and relief cargo due this month are usable before consumption,
  subject to their existing flood/delay/loss rules. New cargo dispatched in the
  response stage does not get another arrival pass that month.
- Market delivery price evidence survives across production as an explicit local
  value; moving deliveries earlier does not discard price-learning evidence.
- Production plans use policies already established before execution. Later
  governance/annual decisions affect subsequent production. Opening local relief
  is an explicit same-month intervention; traveling relief still requires arrival.
- Relocation reads this month's consumption shortage, remembered hunger history,
  current floods, and the last completed social housing projection. These are
  intentionally different time scales; closing projections do not retroactively
  change departures. Migration remains sequential and bounded by existing capacity
  reservations, not a simultaneous population-flow solver.
- Events are committed alongside actions and can influence later stages. Closing
  records summaries; it does not defer provenance IDs or conceal same-month events.
- Zero-duration frozen calls retain the existing initialization/claim synchronization
  used by enable/import APIs, but do not advance clocks or record a new timeline.
  For positive durations, closing work runs per month, not per batch.
- Living failure handling retains the existing incomplete-boundary marker; this
  port does not promise rollback of already-submitted GPU ecological work.

## Compatibility and verification

Old archives need no new fields: observation and reservation values are transient.
Continuation uses the new timing rules, so old-version trajectories are not promised
identical. Frozen batch/single-step/checkpoint tests compare complete history,
terrain, ecology and clocks over annual boundaries on seeds 17, 81 and 256.
Living tests compare full/gathered observation paths and checkpoint continuation.

Independent ordering means only operations without causal or resource dependencies
may commute. Sorting actors is not a fairness model; arbitrarily shuffling stages
would change the rules rather than test their implementation.

## Verification run (2026-09-11)

Quadro RTX 5000 with Max-Q Design; Rust 1.89.0, development/test profile;
terrain 64 and ecology 16 cells per face for integrated schedule fixtures.

- `cargo test --lib`: 62 passed, 66 hardware tests ignored. Includes finite
  extraction allowance settlement, reversed claimant order, and need-weighted
  relief allocation under reversed request order.
- `cargo test --lib economy:: -- --ignored`: 5 passed. Exercises retail/common
  ration accounting, industry income/food access, adaptive cost quotes and
  competing inland/sea freight reservations.
- `cargo test --test history_environment -- --ignored`: three fixtures cover
  current-month food delivery, three-seed frozen batch/checkpoint equivalence,
  and three-seed living gathered/full/checkpoint equivalence.
- Frozen seeds 17, 81, 256 compare a 24-month batch with 24 single steps, then
  12 months against checkpoint-resumed 5+7 months. Comparisons include the entire
  serialized history, terrain/ecology bytes and ecology clock.
- Living seeds 17, 81, 256 compare 36 monthly steps with storm/drought forcing,
  followed by four months against resumed execution. Existing compact-readback
  volume and ecology-budget assertions remain enabled.
- Food timing intervention transfers existing stocks into identical cargo in both
  worlds, changing only its due date. Due-now cargo increases the current consumed
  quantity and reduces the current shortage; future cargo stays in transit.
- Strict library Clippy and repository artifact checks are part of the commit gate.

Initial failures were useful: zero-duration enable calls still needed economic
initialization, so that path was retained explicitly. The food fixture initially
used unpaid cargo and deleted starting food; archive accounting rejected both.
The corrected fixture uses valid paid cargo and transfers, rather than deletes,
starting stocks. These were fixture/port errors, not relaxed budget tolerances.

These tests establish schedule and continuation behavior on this backend, not
cross-hardware equality, long-run balance, or universal actor-order independence.
The port deliberately changes market arrival timing. It does not establish that
new trajectories match the previous release, nor make every subsystem synchronous.
