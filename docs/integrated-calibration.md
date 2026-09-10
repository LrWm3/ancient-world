# Integrated century diagnostics

This evaluation asks whether governance, freight reservations, knowledge continuity
and service recovery operate together, and whether their combined outcomes expose
missing decisions. It is game balancing, not empirical calibration of a historical
society. The eight-area development worklist remains separate: a successful ensemble
does not establish full cross-scale continuity or close unimplemented systems.

## Reproduction

```sh
python3 scripts/build_history_evaluator.py --output output/joint-build
python3 scripts/integrated_history.py --binary output/joint-build/evaluator.bin --build-manifest output/joint-build/manifest.json --output output/joint-control --no-negotiated-autonomy
python3 scripts/integrated_history.py --binary output/joint-build/evaluator.bin --build-manifest output/joint-build/manifest.json --output output/joint-response
python3 scripts/analyze_integrated_history.py output/joint-control
python3 scripts/analyze_integrated_history.py output/joint-response
python3 scripts/compare_integrated_history.py output/joint-control output/joint-response
```

Each suite runs seeds 17, 81 and 256 for 100 years at terrain/ecology resolution 64,
with 16 starting civilizations, one geological epoch, living ecology, expeditions,
and the existing opt-in waterworks repair priority. Baseline crop yield is 0.5;
harsh yield is 0.33. Other catalog values are retained. The runner copies its
executable before launching: rebuilding the project cannot change later ensemble
members. It records executable and source hashes, base commit, exact commands,
exit status, elapsed time and raw-result hashes. Existing directories are rejected.
The build wrapper rejects source edits during compilation and binds its source map
to the executable checksum. A launch-time source snapshot alone is not proof of what
was compiled; externally supplied binaries must retain their own build provenance.

Annual JSONL samples are flushed as they complete. A failed trajectory keeps these
observations but is never marked complete or silently included in a success table.
The analyzer rejects incomplete trajectories, repeated/missing annual months and
raw checksums inconsistent with the manifest.

## Measures and their limits

| Measurement | Interpretation | Limitation |
|---|---|---|
| Population and active sites | Viability, expansion and contraction | Not a historical population fit |
| Shortage site-years | Annual food-stress observations | Demand changes as population and settlement count change |
| Free freight capacity | Existing cargo reservations relative to endpoint capacity | An annual observation can miss quarterly congestion; unused capacity does not prove unmet demand is absent |
| High-unrest site-years | Observed unrest above 0.65 | The event condition also requires foreign control and low loyalty |
| Local knowledge loss/gain | Holder count crossing zero between consecutive annual samples | Only continuously active towns compared; within-year changes and abandonment losses excluded |
| Water coverage and crowding | Physical service and shelter mediators | Final coverage is an unweighted town mean, not population coverage |
| Crises, concessions, recoveries, secessions | Recorded governance transitions | Recovery can mean negotiated jurisdiction, not elimination of hardship |
| Relative ledger residual | Existing resource, food, population, money and nutrient accounting checks | Does not establish plausible parameter values |
| Stage wall time | Setup, production/readback and social/validation cost | Shared workstation runs are not isolated hardware benchmarks |

Both counts and their observed active-site-year denominator remain in structured
summaries. In particular, a harsh world can have fewer shortage observations because
it supports fewer people and expands less. That must not be described as improved
food security without examining demand and individual settlements.

## Verification chain

- The controlled governance hardship fixture holds inventories and payroll constant
  while comparing negotiation, a funded retainer holdout, manual autonomy and no
  response. It checks real fiscal consequences, retained/lost control, causal event
  links, legacy archives and serialized continuation.
- Existing freight fixtures demonstrate that occupied capacity blocks a cheap
  supplier, an available supplier can serve instead, later quarters cannot renew
  occupied capacity, and delivery releases it without creating goods or money.
- Knowledge and service-recovery fixtures establish teacher-loss continuity and
  competing labor/material allocation; the ensemble observes their joint outcomes
  without claiming to replace those controlled tests.
- The matched policy comparison requires annual trajectories to agree before the
  first observed concession, ignoring only an explicit policy-toggle event count.
  A difference before that boundary fails the comparison. Later nonlinear changes
  in population and war are reported as joint trajectories, not isolated effects.
- Seed 409 is reserved from this development suite for a held-out policy comparison
  and a deliberately constrained freight run at 1 kg/person. No parameters should
  be refitted to make that seed achieve a desired outcome.

The first development suite exposed 40 crises and 40 secessions, with no governance
recoveries across its three baseline worlds. The available autonomy policy lacked
an endogenous decision-maker. The resulting bounded council response is documented
in [council crisis response](council-crisis-response.md); it uses existing policy and
fiscal mechanisms rather than resetting social pressure.

## Still not established

These experiments use one resolution and one Vulkan GPU. They are not resolution
convergence, cross-hardware equivalence, empirical fitting, global sensitivity
analysis or a guarantee that every subsystem is exercised. Historical plausibility
requires targets and uncertainty beyond these tests. Full source provenance,
regional activation, physical transport assets and the other structural worklist
items remain implementation work rather than implied conclusions of this report.

## September 2026 results

Fifteen century trajectories completed: six original baseline/harsh runs, six with
council responses, and three held-out seed-409 runs (response disabled, enabled,
and enabled with constrained freight). All existing run-time budget checks passed.
Raw trajectories, checksums and test logs are retained in the
[Artifact retention policy](evidence/README.md).

The [six matched comparisons](evidence/integrated-history/response/comparison.md)
retain 4–38 identical annual observations before the first concession. The original
70 crises/secessions became 42 crises/concessions/recoveries and zero secessions.
This establishes that the response is functioning, **not** that political balance
is solved. Concessions dominate every crisis in this small ensemble. The controlled
funded-retainer fixture still produces refusal and secession; larger faction and
jurisdiction work must assess whether those holdouts occur often enough organically.
No secession quota was imposed and thresholds were not fitted to seed 409.

The [held-out policy comparison](evidence/integrated-history/heldout-response/comparison.md)
agrees through year 11 before a concession appears. Twenty control secessions become
twelve negotiated recoveries. Final population changes from 2,826 to 2,758, while
high-unrest observations increase from 662 to 1,053. Negotiated jurisdiction therefore
preserves control without curing material hardship; the event must not be interpreted
as social well-being or unconditional population improvement.

| Seed 409, responses enabled | Normal freight (20 kg/person) | Constrained freight (1 kg/person) |
|---|---:|---:|
| Saturated annual town observations | 1 / 1,844 | 131 / 1,759 |
| Completed market deliveries | 31,469 | 30,715 |
| Shortage site-years | 632 | 611 |
| Final population | 2,758 | 2,826 |
| Maximum reported relative residual | 1.98e-5 | 2.04e-5 |

The transport restriction binds and changes trade. Lower capacity does not imply
monotonically lower final population in a coupled model; the later population and
shortage results are not claimed as a beneficial causal effect of poor transport.
The controlled reservation fixture remains the evidence for immediate conservation
and allocation behavior. Intermediary road capacity and physical carriers are still
absent, as documented in the transport worklist.

Initial baseline stage totals put social/validation work at 29.1–32.0 seconds per
century, versus 16.3–18.1 seconds for production/readback and 1.9–2.1 seconds for
history setup. That identifies social/validation as the next profiling target within
history, but does not isolate an individual function. Later runs shared CPU/GPU
resources with verification workloads; wall-time differences are not presented as
performance improvements or regressions.

The original before-run executable predates the policy change; use the response
build with `--no-negotiated-autonomy` for new controlled runs. Its explicit policy
change adds a factual event, which the comparator excludes while retaining all
physical/social mediator checks. The held-out variants used the exact same executable
checksum. Source snapshots taken when a later run starts can differ from its copied
binary; the evidence index identifies the response manifest as that binary's source
reference. The new build wrapper prevents this ambiguity for future experiments.
