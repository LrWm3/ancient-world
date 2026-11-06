# Main-engine labor receipts in a private economics replay

The optional `ancient-world-host` feature connects the experiment to the main
crate's real `History`, `Participation`, person identities and commitment types.
It maps already settled labor reservations into a private `host::Settlement` and
records the resulting economic work. This is a historical allocation comparison,
not a live replacement or another claim on the main world's workers.

The main engine remains the only writer of the source history. The replay does
not copy or change its cash, stocks, ecology, population, membership or production.
The example retains the skilled-workshop fixture's supplied economic endowments,
contracts and needs. Mapping a person to an experimental role is an explicit
counterfactual assignment; it does not establish that the host person has that
occupation or those assets. Household IDs are reported as identity evidence and
do not create membership agreements or confer household property.

## Source boundary and finite grants

The existing coordinator opens named availability and reserves work during
Reserve. Main `Participation` records each person's share of a dated commitment;
settlement retains the original grant and records actual completed work. Unused
work expires without becoming capacity for already completed production. The
bridge reads these records after the site's commitments have settled and
requires the participation month, history month and experimental host month to
match. It creates no new main-engine reservation and changes no scheduler phase.

Each experimental participant has exactly one binding. A positive endowment
requires a named source person and one selected `Activity`; zero-endowment
organizations may explicitly use `source: null`. One source person cannot fund
two experimental participants. All capacity resources must match the declared
resource. A valid mapped person with no matching commitment receives zero, even
if the main world reports unused personal availability or work in another duty.

The bridge reuses the main participation validator, then checks source identity,
the recorded opening presence, eligibility and settlement. Positive work requires
the main engine's working-age band. Missing people, stale receipts, unsettled
commitments, invalid shares and wrong sites fail before engine publication. A
same-month death, relocation or site abandonment after the reservation does not
erase historical work; presence is read from the recorded participation boundary.
A person already dead before that month cannot supply it.
The fixed binding stops if a mapped person is absent or already dead at a later
opening, even if no selected work was recorded. Replacement recruitment and
zero-capacity continuation for departed members remain separate work.

`LaborReplay` owns immutable bindings and pins the source's seed, environmental
source epoch, terrain resolution, site ID/cell/civilization/founding date, and each
mapped person's ID/birth/civilization. Consecutive frames must also match the
driver's next month. This catches common accidental source mixing and rebinding.
These fields are not an archive UUID, a cryptographic identity or proof of ancestry
between two divergent same-seed branches. The caller must provide one history
lineage. The lower-level `LaborBindings::capture` is a pure single-frame converter;
multi-month callers should use `LaborReplay`.

## Explicit units and rounding

`capacity_resource` names the experimental labor resource. The caller supplies
`ticks_per_worker_month`, a positive power of two, to give that resource an
explicit physical scale. For example, four ticks per worker-month make a
two-tick indivisible job require half a worker-month. No scale is guessed from
names such as “hours.” Power-of-two multiplication preserves binary host values
when converting f32 worker-months to f64.

Conversion is conservative at three boundaries:

1. Each host commitment receives `floor(granted × scale)` whole ticks. Every
   person's share is floored and bounded by the remaining common tick budget in
   the host's validated person-ID order, including people not mapped into the
   replay. This prevents rounded team shares from exceeding their shared grant.
2. Matching shares for one person are summed and bounded by that person's
   recorded capacity and total committed time, also rounded down.
3. The experimental catalog ceiling limits the submitted capacity. The existing
   engine can further reduce actual use through conditions or scenario limits.

Fractional shares from distinct commitments are not combined to create a whole
tick. Unrepresented work, conservative clipping and amounts beyond catalog demand
remain in `unused_worker_months`; they are not reassigned or carried forward.
Stable ordering here resolves numerical excess only. It is not a new sharing
policy or evidence of fair allocation, and coarse units can prevent useful work.

Every source share retains its monthly commitment index, reserved worker-months,
bounded tick share and the main engine's actual proportional completed work.
Replay grants come from reservations, not completions. A host job that used none
of its reservation can still provide the same bounded counterfactual allowance;
the source result remains zero in the receipt. Economic completions, needs and
stock changes come from ordinary experimental execution and accounting.

## Run against exported history

`--history-export` on the main executable produces the input `History` JSON.
Supply consecutive monthly exports from one world; one terminal export cannot
reconstruct earlier reservations. Reading exports requires no GPU. The optional
feature compiles the main crate's dependencies but leaves its manifest, toolchain,
lockfile and behavior unchanged. Ordinary experiment builds need no host feature.

Example binding shape for the supplied nine-agent fixture (replace person IDs
and selected activities with actual source identities and allocations):

```json
{
  "site": 0,
  "capacity_resource": 104,
  "ticks_per_worker_month": 8,
  "people": [
    {"agent": 0, "source": null},
    {"agent": 88, "source": {"person": 12, "activity": "Forestry"}},
    {"agent": 89, "source": {"person": 13, "activity": "Agriculture"}},
    {"agent": 91, "source": {"person": 14, "activity": "Workshop"}},
    {"agent": 810, "source": null},
    {"agent": 811, "source": {"person": 15, "activity": "Workshop"}},
    {"agent": 820, "source": null}
  ]
}
```

Run from `exp/economics`, keeping data and generated output under ignored output:

```sh
cargo +1.92.0 test --locked --release -j8 --features ancient-world-host \
  --test host_ancient_world
cargo +1.92.0 run --locked --release -j8 --features ancient-world-host \
  --example host_labor_replay -- \
  ../../output/economics/host-replay/bindings.json \
  ../../output/economics/host-replay/month-1.json \
  ../../output/economics/host-replay/month-2.json \
  > ../../output/economics/host-replay/replay.jsonl
```

The first source month maps to local experiment month one. Contracts retain their
local dates. Each successful month prints an input/receipt/outcome object; an
invalid later frame stops with an error, leaving previously printed months as a
partial successful replay, not a complete run or checkpoint.

## Validation and remaining work

Focused tests exercise the actual main reservation and settlement methods in
controlled CPU history fixtures. They compare Reference and CubeCL CPU outcomes,
verify the source history is unchanged, cover grants versus completed work and
unrelated service claims, and test indivisible work, absent demand, fractional
conversion, shared team ceilings, malformed inputs, identity pinning and JSON
history continuation.

Verification on 2026-11-05:

- All eight bridge tests passed, including exact Reference/CubeCL CPU agreement
  for the controlled reservation-to-production case and three consecutive frames
  across a JSON round trip.
- Feature-enabled release Clippy passed for all targets with warnings denied.
- Default-feature-disabled release checking passed for all targets; formatting
  and diff whitespace checks passed. The main manifest and lockfile are unchanged,
  and the experiment lockfile retains every prior dependency version.
- The CLI completed two controlled JSON history frames at host months 17 and 18
  (local months 1 and 2) with four named workers, each reserving 0.5 worker-month
  for Workshop. At eight ticks per worker-month, catalog ceilings reduced the
  combined 16 source ticks to 13 replay ticks: 4, 4, 2 and 3. Each month completed
  one production process and fulfilled all four food needs with zero deficit.
  These outcomes include the fixture's starting assets; they do not show that the
  source settlement could sustain those needs. Inputs, JSONL results and build
  logs remain under ignored `output/economics/`.
- Sub-agent review found that three f32 shares of a half-worker-month team grant
  could supply 32 extra ticks at a scale of 2^31. The shared commitment budget
  fixes that case; the regression also checks that unmapped teammates retain
  their share of the source budget. Review also prompted fixed source identity,
  working-age checks and preservation of work preceding same-month abandonment.

These are controlled fixtures using main-engine types and reservation methods,
not a generated-world rollout or a balance comparison.

This is a step toward real person/unit binding. Importing the source's economic
stocks, cash, contracts and complete population remains outstanding. A live
replacement must collect competing requests and commit exclusive grants within
the actual Reserve window, settle actual main-world use, and make host/engine
failure publication joint. The prior Open-forecast visibility limitation still
applies: this bridge uses the existing dated capacity override mechanism. It does
not introduce price calibration, serialized engine recovery, population scaling
evidence or a new readiness certification.
