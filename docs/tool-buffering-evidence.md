# Tool reserve intervention contract

This experiment extends the [mine staffing evidence](mine-staffing-evidence.md). It tests whether accessible equipment buffers the mining → tools → agriculture connection. It changes no production coefficients.

At the shared checkpoint, each town's ordinary tools are partitioned once:

`accessible = fraction × (accessible + restricted)`

`restricted = previous total − new accessible`

Rounding cannot release more than the conserved total. Restricted tools occupy a separate experimental custody ledger, remain included in goods and C/N/P conservation, and cannot be consumed, traded, looted or weathered. This sealed store is an explicit experimental boundary, not a realistic warehouse. It survives abandonment and checkpoints. Newly produced/imported tools remain accessible. Releasing reserves transfers existing goods back; no initial inventory or production ledger is rewritten. Bronze/copper tools are not restricted; this experiment uses the legacy shared-resource economy without alloy processing.

The GPU records the actual production tool multiplier, cultivated hectares, effective tool kilograms and population at production. The multiplier is `0.75 + 0.25 × clamp(effective tools / max(1, 0.5 × population), 0, 1)`. Thus even complete tool deprivation can directly reduce this multiplier by at most 25%; this is a design assumption, not an empirical coefficient. Records after consumption alone cannot reconstruct that month's production inputs.

Run the full checks and three reserve levels with:

```sh
python3 scripts/evidence.py --profile full --tool-fractions 1,0.25,0 \
  --cargo 'mise exec rust@1.89.0 -- cargo' --output output/evidence-tools
```

Defaults use seeds 17, 81 and 256, 12 months of shared spinup, 60 months of mine closure and 60 months after reopening. Each reserve level includes open/closed mines under adaptive/fixed staffing. The report separates the mine effect within each reserve level from the reserve effect relative to unrestricted tools. It measures immediate production inputs before interpreting long-run population changes.

The source-resource residual is normalized and dimensionless (`|initial − remaining − extracted| / max(initial, 1)`). Earlier mine reports incorrectly labeled its zero value as absolute kg; the new report corrects the unit without changing archived observations.

Validation includes restricted-custody conservation, inaccessible-tool production, release, no-op equivalence, invalid interventions, exact checkpoint continuation, and execution-batch equivalence. These are implementation and mechanism tests, not historical calibration.

## Results at revision 38c66d4

All 36 branches completed: 4,320 monthly observations, all three no-op controls,
41 CPU Rust tests, 97 GPU Rust tests and five Python checks passed, along with
formatting and Clippy. Source hashes stayed unchanged throughout the run.
GPU: Quadro RTX 5000 with Max-Q Design, Vulkan, NVIDIA 595.84; Rust 1.89.0.

Restricting starting tools reduced five-year cumulative harvest and settled
population in **every** matched stock-intervention comparison (24 contrasts).
The first-month population-weighted production multiplier fell from approximately
0.9855 with unrestricted tools, to 0.8089 at quarter access, to exactly 0.75 at
zero access. Fixed staffing kept first-month cultivated area identical across
these interventions, providing a direct production-input comparison.

Mine closure nevertheless retained its positive adaptive-staffing population
effect at all reserve levels. With fixed staffing, reducing reserves consistently
increased the closure loss:

| Seed | Accessible fraction | Closure population effect: adaptive | Closure population effect: fixed |
|---|---:|---:|---:|
| 17 | 1 | +168.12 | −35.61 |
| 17 | 0.25 | +199.82 | −74.68 |
| 17 | 0 | +209.23 | −93.82 |
| 81 | 1 | +170.59 | −16.94 |
| 81 | 0.25 | +174.87 | −37.09 |
| 81 | 0 | +161.39 | −75.47 |
| 256 | 1 | +196.37 | −26.34 |
| 256 | 0.25 | +243.61 | −82.02 |
| 256 | 0 | +257.79 | −125.99 |

These are closed minus open comparisons **within the same reserve and staffing
configuration**, not population levels or a benefit from removing tools. For
example, seed 17's adaptive closed-mine branch at zero access has 170.41 fewer
people than its closed-mine branch at full access. Including travelers preserves
every mine-effect sign in the table.

The first-year adaptive farming-share increase under closure remains 13.18–15.60
percentage points with zero initial access. Five-year closure harvest gains under
adaptive staffing become smaller as reserves decline, while fixed-staffing harvest
losses become larger. Thus population alone would obscure part of the production
response; food timing, distribution and subsequent demographic decisions remain
active.

Existing tools do buffer the shock, but they are not the sole reason the adaptive
closure effect is positive. At zero initial access, the adaptive tool multiplier
at month 60 is 0.807–0.850 with closed mines, versus 0.959–0.997 with open mines.
Five years after reopening, the formerly closed branches reach 0.962–0.968,
without releasing the restricted tools. New production/imports remain available;
these measurements do not separately attribute recovery to individual supply paths.

## Verification and interpretation limits

All 1,440 unrestricted monthly observations exactly matched the prior
`mine-staffing-results.json.gz` archive after removing only the newly introduced
`production_probe` and `restricted_tools_kg` site fields. This checks all previously
recorded monthly measurements, not every hidden GPU field. No production rule was
changed to obtain the new results.

Maximum normalized economy residual was **3.306e-6**, source residual **0**,
ecology C/N/P relative error **3.680e-6**, ecology water relative error **2.539e-5**,
and population relative error **1.943e-7**. All monthly guards passed, including
the post-intervention stock samples. Restricted goods remained in accounted custody. A separate data audit checked
all 36 initial stock partitions and all 4,320 monthly custody/probe records:
accessible plus restricted starting tools matched the original quantities,
restricted amounts stayed constant, and active production multipliers stayed
within [0.75, 1].

CPU checks took 37.8 seconds, hardware checks 432.6 seconds, and the intervention
experiment 277.6 seconds. These are single-run wall times on a shared workstation,
including applicable compilation/cache effects, not controlled benchmarks.

This establishes active stock → production → food/population coupling and an
interaction with labor adaptation within the existing model. It does not validate
the 25% tool penalty, establish historical realism, identify a unique mediated
population effect, or generalize beyond the tested development seeds, resolution,
legacy economy and ten-year horizon. No parameter fitting occurred.

The [food-security allocator experiment](food-security-labor.md) now implements
and evaluates this follow-up as an opt-in policy under the same stock shocks.
The next checks should isolate the maintenance reservation and evaluate held-out
seeds before changing defaults. Empirical targets are still needed before calling
such tuning calibration.

## Archived evidence

- [All generated comparisons](evidence/tool-buffering-tables.md).
- [Artifact retention policy](evidence/README.md),
  40,668,403 compressed bytes.
- [Artifact retention policy](evidence/README.md).
- [Stage results and coverage limits](evidence/tool-buffering-run.md).

The decompressed archive SHA-256 equals `artifacts_sha256["mine/results.json"]` in
the manifest. Logs, JSONL journals and checkpoints remain in
`output/evidence-tools-38c66d4/`; the reproduction command regenerates them.
The evidence archive contains all branches rather than selected favorable seeds.
This documentation/data commit follows the recorded simulation revision.
